//! Explicit World-to-owner routing. Normal Renet/manifest/client adoption is
//! separate. Profiles bind exact content bytes and the XP policy, not a story MD.
use crate::{
    Result,
    dialogue::{Rejection, Request},
    progression_wire,
    renet_runtime::progression::{Input, JournalInput, JournalOperation, Port},
    simulation::{World, combat::Event, quest_journal::Prepared},
};
use std::sync::Arc;
use theia_core::{
    identity::EntityKey,
    quests::{catalog::Catalog, journal::PROFILE_DOMAIN},
};
pub fn profile_identity(base: [u8; 32], journal: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-quest-journal-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(journal);
    crate::terrain::sha256(&bytes)
}
#[derive(Clone, Debug)]
pub struct Profile {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
    xp: [u8; 32],
    areas: Option<Arc<theia_core::quests::areas::Catalog>>,
    inventory: Option<(Arc<theia_core::inventory::Catalog>, [u8; 32])>,
}
impl Profile {
    pub fn from_json(bytes: &[u8], xp: &progression_wire::Profile) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes)?);
        let mut bound = PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        bound.extend(xp.hash());
        Ok(Self {
            catalog,
            hash: crate::terrain::sha256(&bound),
            xp: xp.hash(),
            areas: None,
            inventory: None,
        })
    }
    /// Explicit experimental area mode. The legacy constructor/hash is unchanged.
    pub fn from_json_with_areas(
        bytes: &[u8],
        area_bytes: &[u8],
        xp: &progression_wire::Profile,
    ) -> Result<Self> {
        let mut profile = Self::from_json(bytes, xp)?;
        let areas = Arc::new(theia_core::quests::areas::Catalog::from_json(area_bytes)?);
        areas.validate_quests(&profile.catalog)?;
        let mut bound = theia_core::quests::areas::PROFILE_DOMAIN.to_vec();
        bound.extend(profile.hash);
        bound.extend(area_bytes);
        profile.hash = crate::terrain::sha256(&bound);
        profile.areas = Some(areas);
        Ok(profile)
    }
    /// Bind current-possession semantics to the exact inventory profile.
    /// A created profile is not World admission or durable ownership.
    pub fn with_inventory(
        mut self,
        inventory: &crate::renet_runtime::inventory::Profile,
    ) -> Result<Self> {
        if self.inventory.is_some() || !self.catalog.quests().any(|q| q.has_possession_objectives())
        {
            return Err("Inventory binding requires unbound possession objectives".into());
        }
        self.catalog
            .validate_possession_items(inventory.catalog())?;
        let mut bytes = theia_core::quests::journal::ITEM_PROFILE_DOMAIN.to_vec();
        bytes.extend(self.hash);
        bytes.extend(inventory.hash());
        self.hash = crate::terrain::sha256(&bytes);
        self.inventory = Some((Arc::new(inventory.catalog().clone()), inventory.hash()));
        Ok(self)
    }
    pub fn inventory_hash(&self) -> Option<[u8; 32]> {
        self.inventory.as_ref().map(|(_, hash)| *hash)
    }
    pub fn inventory_catalog(&self) -> Option<&theia_core::inventory::Catalog> {
        self.inventory.as_ref().map(|(catalog, _)| catalog.as_ref())
    }
    pub(crate) fn shared_areas(&self) -> Option<Arc<theia_core::quests::areas::Catalog>> {
        self.areas.clone()
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub(crate) fn check_progression(&self, xp: &progression_wire::Profile) -> Result<()> {
        if self.xp != xp.hash() {
            return Err("Journal/progression policy mismatch".into());
        }
        Ok(())
    }
    pub(crate) fn shared_catalog(&self) -> Arc<Catalog> {
        self.catalog.clone()
    }
    pub(crate) fn bind_authority(self: &Arc<Self>, world: Prepared, actor: EntityKey) -> Router {
        PreparedRouter {
            profile: self.clone(),
            world,
        }
        .bind(actor)
    }
    /// Prepare only after all World content/modes are configured, before join.
    pub fn prepare(self: &Arc<Self>, world: &World) -> Result<PreparedRouter> {
        Ok(PreparedRouter {
            profile: self.clone(),
            world: world.prepare_quest_journal_profile(self)?,
        })
    }
}
pub struct PreparedRouter {
    profile: Arc<Profile>,
    world: Prepared,
}
impl PreparedRouter {
    pub fn bind(self, actor: EntityKey) -> Router {
        Router {
            profile: self.profile,
            world: self.world,
            actor,
            sequence: 0,
            command_sequence: 0,
            pending: None,
            last_areas: None,
        }
    }
}
/// One admitted actor and one trusted-producer sequence per retained owner.
/// Queue acceptance never acknowledges persistence; observe committed owner state.
pub struct Router {
    profile: Arc<Profile>,
    world: Prepared,
    actor: EntityKey,
    sequence: u64,
    command_sequence: u64,
    pending: Option<Pending>,
    last_areas: Option<theia_core::quests::journal::ReachedAreas>,
}
struct Pending {
    owner_sequence: u64,
    request: crate::quest_journal_wire::Request,
}
impl Router {
    fn bound(&self, world: &World, port: &Port) -> bool {
        if port.input.max_capacity() != crate::renet_runtime::progression::CAPACITY
            || world.entity_key(self.actor.entity.value()) != Some(self.actor)
            || port.journal != Some(self.profile.hash)
            || port.profile.hash() != self.profile.xp
            || port.quest.is_some()
            || port.lifecycle.is_some()
        {
            return false;
        }
        let state = port.state.borrow();
        !state.failed
            && !state.closed
            && state
                .journal
                .as_ref()
                .is_none_or(|j| j.catalog == self.profile.hash)
    }
    fn ready(&self, world: &World, port: &Port) -> bool {
        self.bound(world, port) && port.state.borrow().journal.is_some()
    }
    /// Sample only after reserving bounded owner capacity. A full queue does
    /// not consume/cache the observation; retry while occupancy remains eligible.
    pub fn observe_areas(&mut self, world: &World, port: &Port) -> Result<bool> {
        if self.profile.areas.is_none() {
            return Ok(false);
        }
        if !self.world.matches_world(world) || !self.bound(world, port) {
            return Err("Quest area router binding changed".into());
        }
        if !self.ready(world, port) || world.quest_area_feet(&self.world, self.actor).is_none() {
            return Ok(false);
        }
        let Ok(permit) = port.input.try_reserve() else {
            return Ok(false);
        };
        let state = port.state.borrow();
        let journal = &state
            .journal
            .as_ref()
            .ok_or("Quest area owner is not initialized")?
            .journal;
        let observation = world.observe_quest_areas(&self.world, self.actor, journal)?;
        if observation.is_empty() || self.last_areas.as_ref() == Some(&observation) {
            return Ok(false);
        }
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("Quest area sequence exhausted")?;
        let mut bytes = b"theia-journal-area-occurrence-v1\0".to_vec();
        bytes.extend(world.identity());
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(sequence.to_le_bytes());
        let hash = crate::terrain::sha256(&bytes);
        let id =
            std::num::NonZeroU128::new(u128::from_le_bytes(hash[..16].try_into().unwrap()).max(1))
                .unwrap();
        permit.send(Input::Journal(JournalInput {
            actor: self.actor,
            catalog: self.profile.hash,
            sequence,
            operation: JournalOperation::Areas {
                id,
                observation: observation.clone(),
            },
        }));
        self.sequence = sequence;
        self.last_areas = Some(observation);
        Ok(true)
    }
    /// Reserve owner capacity before the dialogue authority changes state.
    /// Full/closed/not-initialized/mismatched owners refuse without processing it.
    pub fn interact(
        &mut self,
        world: &mut World,
        request: &Request,
        port: &Port,
    ) -> std::result::Result<bool, Rejection> {
        if !self.ready(world, port) {
            return Err(Rejection::Unavailable);
        }
        let sequence = self.sequence.checked_add(1).ok_or(Rejection::Exhausted)?;
        let permit = port
            .input
            .try_reserve()
            .map_err(|_| Rejection::Unavailable)?;
        let Some(fact) =
            world.interact_quest_journal(&self.world, self.actor.entity.value(), request)?
        else {
            return Ok(false);
        };
        permit.send(Input::Journal(JournalInput {
            actor: fact.actor,
            catalog: self.profile.hash,
            sequence,
            operation: JournalOperation::Event {
                id: fact.id,
                event: fact.event,
            },
        }));
        self.sequence = sequence;
        Ok(true)
    }
    /// A single pending command shares this producer's sequence with events.
    /// Refusals are local; Ok only means queued. Poll `take_reply` for COMMIT.
    pub fn command(
        &mut self,
        world: &World,
        request: &crate::quest_journal_wire::Request,
        port: &Port,
    ) -> std::result::Result<(), crate::quest_journal_wire::Rejection> {
        use crate::quest_journal_wire::Rejection as Refusal;
        if request.encode(&self.profile).is_err()
            || request.actor != self.actor
            || !self.world.matches_world(world)
            || !world.guide_interaction_eligible(
                self.actor.entity.value(),
                request.dialogue_revision,
                request.npc,
            )
        {
            return Err(Refusal::NotEntitled);
        }
        if request.sequence <= self.command_sequence
            || request.sequence.saturating_sub(self.command_sequence) > 128
        {
            return Err(Refusal::StaleSequence);
        }
        self.command_sequence = request.sequence;
        if self.pending.is_some() || !self.ready(world, port) {
            return Err(Refusal::Busy);
        }
        let sequence = self.sequence.checked_add(1).ok_or(Refusal::Exhausted)?;
        let permit = port.input.try_reserve().map_err(|_| Refusal::Busy)?;
        permit.send(Input::Journal(JournalInput {
            actor: self.actor,
            catalog: self.profile.hash,
            sequence,
            operation: JournalOperation::Command(request.command.clone()),
        }));
        self.sequence = sequence;
        self.pending = Some(Pending {
            owner_sequence: sequence,
            request: request.clone(),
        });
        Ok(())
    }
    pub fn take_reply(
        &mut self,
        world: &World,
        port: &Port,
    ) -> Result<Option<crate::quest_journal_wire::Reply>> {
        use crate::quest_journal_wire::{Rejection as Refusal, Reply};
        use crate::renet_runtime::progression::JournalRejection;
        use theia_core::quests::journal::Error as Rule;
        let Some(pending) = &self.pending else {
            return Ok(None);
        };
        if !self.ready(world, port) {
            return Err("Pending quest owner unavailable".into());
        }
        let state = port.state.borrow();
        let journal = state
            .journal
            .as_ref()
            .ok_or("Pending journal unavailable")?;
        let Some(result) = &journal.reply else {
            return Ok(None);
        };
        if result.sequence < pending.owner_sequence {
            return Ok(None);
        }
        if result.sequence != pending.owner_sequence {
            return Err("Quest command acknowledgement overwritten".into());
        }
        let rejection = match result.rejection {
            None => None,
            Some(JournalRejection::Busy) => Some(Refusal::Busy),
            Some(JournalRejection::Conflict) => Some(Refusal::Conflict),
            Some(JournalRejection::Rule(rule)) => Some(match rule {
                Rule::AlreadyAccepted | Rule::Prerequisite => Refusal::NotAvailable,
                Rule::StaleRevision => Refusal::StaleRevision,
                Rule::NotReady => Refusal::NotReady,
                Rule::Capacity => Refusal::Capacity,
                Rule::RevisionExhausted => Refusal::Exhausted,
                _ => return Err("Unexpected quest owner rule failure".into()),
            }),
        };
        let quest = pending.request.command.quest.clone();
        let reply = Reply {
            actor: self.actor,
            profile_hash: self.profile.hash,
            sequence: pending.request.sequence,
            request_id: pending.request.command.request_id,
            revision: journal.revision,
            quest_revision: journal.journal.progress(&quest).map_or(0, |p| p.revision()),
            quest,
            rejection,
        };
        reply.validate_request(&self.profile, &pending.request)?;
        self.pending = None;
        Ok(Some(reply))
    }
    /// All-or-none queue reservation for this actor's trusted kill batch. On
    /// refusal, caller retains the original batch for retry or fails closed;
    /// it must never acknowledge or discard earned work as committed.
    pub fn forward_kills(&mut self, world: &World, events: &[Event], port: &Port) -> Result<usize> {
        if world.entity_key(self.actor.entity.value()) != Some(self.actor) {
            return Err("Quest router owner binding unavailable".into());
        }
        let facts: Vec<_> = world
            .quest_kill_occurrences(&self.world, events)?
            .into_iter()
            .filter(|fact| fact.actor == self.actor)
            .collect();
        if facts.is_empty() {
            return Ok(0);
        }
        // Immutable admission binding permits trusted kills to queue while the
        // retained owner initializes saved state. Commands still need a view.
        if !self.bound(world, port) {
            return Err("Quest kill owner binding unavailable".into());
        }
        let count = facts.len();
        let end = self
            .sequence
            .checked_add(count as u64)
            .ok_or("Quest router sequence exhausted")?;
        let permits = port
            .input
            .try_reserve_many(count)
            .map_err(|_| "Quest router owner queue unavailable")?;
        for (offset, (permit, fact)) in permits.zip(facts).enumerate() {
            permit.send(Input::Journal(JournalInput {
                actor: fact.actor,
                catalog: self.profile.hash,
                sequence: self.sequence + offset as u64 + 1,
                operation: JournalOperation::Kill(fact.event),
            }));
        }
        self.sequence = end;
        Ok(count)
    }
}
