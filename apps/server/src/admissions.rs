//! Account-owned reservations and final-pose retirement. All synchronous work is
//! bounded by eight slots; only `flush` publishes to the asynchronous SQL owner.
#[path = "admissions/inventory.rs"]
mod inventory;
#[path = "admissions/progression.rs"]
mod progression;
use ring::rand::{SecureRandom, SystemRandom};
use std::{
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;
use theia_auth::{SessionAuthorization, Sessions};
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
use theia_network::{
    Result,
    renet_runtime::{
        AccountAdmissions, AdmissionLease, IssuedTicket, PlayerAdmission, TicketIssuer,
    },
    wire::MAX_PLAYERS,
};
use theia_persistence::{Binding, OwnerId, Publisher, SavedCharacter, WorkerStatus};

const TICKET_SECONDS: u64 = 20;
const TAG: &[u8; 8] = b"THEIA-A1";
type AccountSessions = Arc<Mutex<Sessions<OwnerId>>>;
#[derive(Clone, Copy, Debug)]
pub enum Error {
    Unauthorized,
    Conflict,
    Busy,
    Unavailable,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unauthorized => "Account authorization required",
            Self::Conflict => "Account or character already reserved",
            Self::Busy => "Game admission capacity reached",
            Self::Unavailable => "Game admission unavailable",
        })
    }
}
impl std::error::Error for Error {}
type GateResult<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Selecting,
    Reserved,
    Consumed,
    Active,
    Retiring,
}
struct Entry {
    progression: Option<progression::Session>,
    inventory: Option<inventory::Session>,
    generation: u64,
    binding: Binding,
    context: SessionAuthorization<OwnerId>,
    deadline: Instant,
    phase: Phase,
    nonce: [u8; 32],
    saved: Option<SavedCharacter>,
    actor: Option<EntityKey>,
    tick: u64,
    version: u64,
    queued: Option<(u64, u64)>, // exact pose version and SQL mailbox generation
}
struct State {
    slots: [Option<Entry>; MAX_PLAYERS],
    generation: u64,
    sequence: u64,
    failed: bool,
}
pub struct GameAdmissions {
    progression: Option<progression::Config>,
    inventory: Option<inventory::Config>,
    state: Mutex<State>,
    sessions: AccountSessions,
    issuer: TicketIssuer,
    world: [u8; 32],
}
/// Held across character SELECT so concurrent requests cannot read an old pose
/// before a previous admission's retirement commit. Cancellation frees the slot.
pub struct Reservation {
    gate: Arc<GameAdmissions>,
    slot: usize,
    generation: u64,
    armed: bool,
}
struct Lease {
    gate: Arc<GameAdmissions>,
    slot: usize,
    generation: u64,
}
impl GameAdmissions {
    pub(crate) fn new(
        sessions: AccountSessions,
        issuer: TicketIssuer,
        world: [u8; 32],
    ) -> Arc<Self> {
        Arc::new(Self {
            progression: None,
            inventory: None,
            state: Mutex::new(State {
                slots: std::array::from_fn(|_| None),
                generation: 0,
                sequence: 0,
                failed: false,
            }),
            sessions,
            issuer,
            world,
        })
    }
    pub(crate) fn new_inventory(
        sessions: AccountSessions,
        issuer: TicketIssuer,
        world: [u8; 32],
        profile: theia_persistence::InventoryCatalog,
        publisher: Publisher,
    ) -> Result<Arc<Self>> {
        let config = inventory::Config::new(profile, publisher)?;
        let mut gate = Self::new(sessions, issuer, world);
        Arc::get_mut(&mut gate)
            .expect("Unshared new gate")
            .inventory = Some(config);
        Ok(gate)
    }
    pub(crate) fn new_rewards(
        sessions: AccountSessions,
        issuer: TicketIssuer,
        world: [u8; 32],
        rewards: theia_persistence::RewardCatalog,
        wire: Arc<theia_network::reward_wire::Profile>,
        publisher: Publisher,
    ) -> Result<Arc<Self>> {
        let mut config =
            inventory::Config::new(rewards.items().clone(), publisher)?.with_rewards(rewards)?;
        config.set_reward_wire(wire)?;
        let mut gate = Self::new(sessions, issuer, world);
        Arc::get_mut(&mut gate)
            .expect("Unshared new gate")
            .inventory = Some(config);
        Ok(gate)
    }
    pub(crate) fn with_progression(
        gate: &mut Arc<Self>,
        profile: theia_persistence::ProgressionCatalog,
        wire: Arc<theia_network::progression_wire::Profile>,
        publisher: Publisher,
    ) -> Result<()> {
        Arc::get_mut(gate)
            .ok_or("XP configuration requires unshared admission gate")?
            .progression = Some(progression::Config::new(profile, wire, publisher)?);
        Ok(())
    }
    pub(crate) fn with_vendor(
        gate: &mut Arc<Self>,
        profile: theia_persistence::VendorCatalog,
        wire: Arc<theia_network::vendor_wire::Profile>,
    ) -> Result<()> {
        Arc::get_mut(gate)
            .ok_or("Vendor configuration requires unshared gate")?
            .inventory
            .as_mut()
            .ok_or("Vendor requires item owner")?
            .set_vendor(profile, wire)
    }
    pub(crate) fn with_quest(
        gate: &mut Arc<Self>,
        profile: theia_persistence::QuestCatalog,
        wire: Arc<theia_network::quest_wire::Profile>,
    ) -> Result<()> {
        Arc::get_mut(gate)
            .ok_or("Objective configuration requires unshared admission gate")?
            .progression
            .as_mut()
            .ok_or("Objectives require progression owner")?
            .set_quest(profile, wire)
    }
    fn lock(&self) -> GateResult<MutexGuard<'_, State>> {
        let state = self.state.lock().map_err(|_| Error::Unavailable)?;
        if state.failed {
            return Err(Error::Unavailable);
        }
        Ok(state)
    }
    fn authorized(&self, entry: &Entry) -> GateResult<bool> {
        let mut sessions = self.sessions.lock().map_err(|_| Error::Unavailable)?;
        Ok(sessions.validate(&entry.context).is_ok())
    }
    fn prune(&self, state: &mut State) -> GateResult<()> {
        for slot in &mut state.slots {
            if let Some(entry) = slot.as_ref()
                && matches!(entry.phase, Phase::Selecting | Phase::Reserved)
                && (Instant::now() >= entry.deadline || !self.authorized(entry)?)
            {
                *slot = None;
            }
        }
        Ok(())
    }
    pub fn reserve(
        self: &Arc<Self>,
        context: SessionAuthorization<OwnerId>,
        binding: Binding,
    ) -> GateResult<Reservation> {
        let mut state = self.lock()?;
        self.prune(&mut state)?;
        if self
            .sessions
            .lock()
            .map_err(|_| Error::Unavailable)?
            .validate(&context)
            .ok()
            != Some(binding.owner)
        {
            return Err(Error::Unauthorized);
        }
        if state
            .slots
            .iter()
            .flatten()
            .any(|e| e.binding.owner == binding.owner || e.binding.character == binding.character)
        {
            return Err(Error::Conflict);
        }
        let slot = state
            .slots
            .iter()
            .position(Option::is_none)
            .ok_or(Error::Busy)?;
        let generation = state.generation.checked_add(1).ok_or(Error::Unavailable)?;
        let mut nonce = [0; 32];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| Error::Unavailable)?;
        state.generation = generation;
        state.slots[slot] = Some(Entry {
            progression: None,
            inventory: None,
            generation,
            binding,
            context,
            deadline: Instant::now() + Duration::from_secs(10),
            phase: Phase::Selecting,
            nonce,
            saved: None,
            actor: None,
            tick: 0,
            version: 0,
            queued: None,
        });
        Ok(Reservation {
            gate: self.clone(),
            slot,
            generation,
            armed: true,
        })
    }
    /// Called by the async owner outside the simulation. Every pending final pose
    /// stays in every superseding batch until its exact version is committed.
    pub fn flush(&self, publisher: &Publisher, status: &WorkerStatus) -> GateResult<usize> {
        let mut state = self.lock()?;
        if status.failed || status.stale_records != 0 {
            state.failed = true;
            return Err(Error::Unavailable);
        }
        self.prune(&mut state)?;
        for slot in &mut state.slots {
            let release = slot.as_ref().is_some_and(|entry| {
                entry.phase == Phase::Retiring
                    && entry.inventory.as_ref().is_none_or(|s| s.finished())
                    && entry.progression.as_ref().is_none_or(|s| s.finished())
                    && (entry.actor.is_none()
                        || entry.queued.is_some_and(|(version, generation)| {
                            version == entry.version && generation <= status.completed_generation
                        }))
            });
            if release {
                *slot = None;
            }
        }
        let dirty = state.slots.iter().flatten().any(|entry| {
            entry.actor.is_some()
                && entry
                    .queued
                    .is_none_or(|(version, _)| version != entry.version)
        });
        if dirty {
            state.sequence = state
                .sequence
                .checked_add(1)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or(Error::Unavailable)?;
            let sequence = state.sequence;
            let records: Vec<_> = state
                .slots
                .iter_mut()
                .flatten()
                .filter(|entry| entry.actor.is_some())
                .map(|entry| {
                    let saved = entry
                        .saved
                        .as_mut()
                        .expect("Joined entries have a selected record");
                    saved.sequence = sequence;
                    *saved
                })
                .collect();
            let generation = publisher.publish(&records).map_err(|_| {
                state.failed = true;
                Error::Unavailable
            })?;
            for entry in state
                .slots
                .iter_mut()
                .flatten()
                .filter(|entry| entry.actor.is_some())
            {
                if entry
                    .queued
                    .is_none_or(|(version, _)| version != entry.version)
                {
                    entry.queued = Some((entry.version, generation));
                }
            }
        }
        Ok(state
            .slots
            .iter()
            .flatten()
            .filter(|e| matches!(e.phase, Phase::Consumed | Phase::Active | Phase::Retiring))
            .count())
    }
    pub fn fail_closed(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.failed = true;
        }
    }
    fn abandon(&self, slot: usize, generation: u64) {
        if let Ok(mut state) = self.state.lock()
            && let Some(entry) = state.slots[slot].as_mut()
            && entry.generation == generation
        {
            if let Some(session) = &entry.progression {
                session.stop();
            }
            if let Some(session) = &entry.inventory {
                session.stop();
            }
            if entry.actor.is_some()
                || entry.inventory.as_ref().is_some_and(|s| !s.finished())
                || entry.progression.as_ref().is_some_and(|s| !s.finished())
            {
                entry.phase = Phase::Retiring;
            } else {
                state.slots[slot] = None;
            }
        }
    }
}
impl Reservation {
    pub fn issue(mut self, saved: SavedCharacter) -> GateResult<IssuedTicket> {
        let mut state = self.gate.lock()?;
        let entry = state.slots[self.slot]
            .as_mut()
            .filter(|e| e.generation == self.generation && e.phase == Phase::Selecting)
            .ok_or(Error::Unavailable)?;
        if !self.gate.authorized(entry)? || Instant::now() >= entry.deadline {
            return Err(Error::Unauthorized);
        }
        if saved.binding != entry.binding
            || saved.world != self.gate.world
            || saved.sequence > i64::MAX as u64
        {
            return Err(Error::Unavailable);
        }
        let [x, _, z] = saved.position.metres();
        let radius = f64::from(theia_network::kinematic::FOOTPRINT_RADIUS);
        if !(radius..=16000. - radius).contains(&x) || !(radius..=16000. - radius).contains(&z) {
            return Err(Error::Unavailable);
        }
        let mut data = [0; 256];
        data[..8].copy_from_slice(TAG);
        data[8..16].copy_from_slice(&entry.generation.to_le_bytes());
        data[16..48].copy_from_slice(&entry.nonce);
        let ticket = self
            .gate
            .issuer
            .issue(self.slot as u64 + 1, Some(&data), TICKET_SECONDS)
            .map_err(|_| Error::Unavailable)?;
        entry.saved = Some(saved);
        entry.phase = Phase::Reserved;
        entry.deadline = Instant::now() + Duration::from_secs(TICKET_SECONDS);
        state.sequence = state.sequence.max(saved.sequence);
        self.armed = false;
        Ok(ticket)
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.armed {
            self.gate.abandon(self.slot, self.generation);
        }
    }
}
impl AdmissionLease for Lease {
    fn progression(&self) -> Result<Option<theia_network::renet_runtime::progression::Port>> {
        let state = self.gate.lock()?;
        let entry = state.slots[self.slot]
            .as_ref()
            .filter(|e| e.generation == self.generation)
            .ok_or(Error::Unavailable)?;
        Ok(entry.progression.as_ref().map(|s| s.port.clone()))
    }

    fn inventory(&self) -> Result<Option<theia_network::renet_runtime::inventory::Port>> {
        let state = self.gate.lock()?;
        let entry = state.slots[self.slot]
            .as_ref()
            .filter(|e| e.generation == self.generation)
            .ok_or(Error::Unavailable)?;
        Ok(entry.inventory.as_ref().map(|s| s.port.clone()))
    }
    fn live(&self) -> Result<bool> {
        let state = self.gate.lock()?;
        let Some(entry) = state.slots[self.slot].as_ref().filter(|e| {
            e.generation == self.generation && matches!(e.phase, Phase::Consumed | Phase::Active)
        }) else {
            return Ok(false);
        };
        Ok(self.gate.authorized(entry)?)
    }
    fn record(&self, actor: EntityKey, position: GlobalPosition, tick: u64) -> Result<()> {
        let mut state = self.gate.lock()?;
        let entry = state.slots[self.slot]
            .as_mut()
            .filter(|e| {
                e.generation == self.generation
                    && matches!(e.phase, Phase::Consumed | Phase::Active)
            })
            .ok_or(Error::Unavailable)?;
        if entry.actor.is_some_and(|old| old != actor) || tick < entry.tick {
            state.failed = true;
            return Err(Error::Unavailable.into());
        }
        if entry.actor.is_none() || entry.saved.as_ref().is_none_or(|r| r.position != position) {
            entry.version = entry.version.checked_add(1).ok_or(Error::Unavailable)?;
        }
        entry.saved.as_mut().ok_or(Error::Unavailable)?.position = position;
        entry.actor = Some(actor);
        entry.tick = tick;
        entry.phase = Phase::Active;
        Ok(())
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.gate.abandon(self.slot, self.generation);
    }
}
/// The Arc owns consumed leases, keeping their reservation manager alive until
/// every removal path (including authority cancellation) has retired its pose.
impl AccountAdmissions for GameAdmissions {
    fn consume(
        self: Arc<Self>,
        client_id: u64,
        data: &[u8; 256],
    ) -> Result<Option<PlayerAdmission>> {
        if !(1..=MAX_PLAYERS as u64).contains(&client_id)
            || &data[..8] != TAG
            || data[48..].iter().any(|b| *b != 0)
        {
            return Ok(None);
        }
        let slot = client_id as usize - 1;
        let mut state = self.lock()?;
        let Some(entry) = state.slots[slot].as_mut() else {
            return Ok(None);
        };
        if entry.phase != Phase::Reserved
            || Instant::now() >= entry.deadline
            || entry.generation != u64::from_le_bytes(data[8..16].try_into().unwrap())
            || !bool::from(entry.nonce.ct_eq(&data[16..48]))
            || !self.authorized(entry)?
        {
            return Ok(None);
        }
        entry.phase = Phase::Consumed;
        if let Some(config) = &self.inventory {
            entry.inventory = Some(config.start(
                Arc::downgrade(&self),
                slot,
                entry.generation,
                entry.binding,
                self.world,
            ));
        }
        if let Some(config) = &self.progression {
            entry.progression = Some(config.start(
                Arc::downgrade(&self),
                slot,
                entry.generation,
                entry.binding,
                self.world,
            ));
        }
        let position = entry.saved.as_ref().ok_or(Error::Unavailable)?.position;
        Ok(Some(PlayerAdmission {
            position,
            lease: Box::new(Lease {
                gate: self.clone(),
                slot,
                generation: entry.generation,
            }),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; MAX_PLAYERS]> {
        let mut state = self.lock()?;
        self.prune(&mut state)?;
        Ok(std::array::from_fn(|i| {
            state.slots[i]
                .as_ref()
                .filter(|e| e.phase != Phase::Active)
                .and_then(|e| e.saved.map(|r| r.position))
        }))
    }
}
