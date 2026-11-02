//! Exact-byte profile prerequisite for opt-in player life/creature retaliation.
//! Private frame publication and native presentation are separate adoption gates.
use crate::{Result, simulation::creatures};
use std::sync::Arc;
use theia_core::{content::ContentId, player_life, resources, retaliation};
#[derive(Clone, Debug)]
pub struct Profile {
    hash: [u8; 32],
    stats_hash: [u8; 32],
    creature_hash: [u8; 32],
    creature: ContentId,
    life: Arc<player_life::Profile>,
    retaliation: Arc<retaliation::Profile>,
}
impl Profile {
    pub fn from_json(
        stats: &[u8],
        creature: &[u8],
        retaliation: &[u8],
        respawn: &[u8],
    ) -> Result<Self> {
        let stats_profile = resources::Profile::from_json(stats)?;
        let creature_profile = creatures::Profile::from_json(creature)?;
        let policy = retaliation::Profile::from_json(retaliation)?;
        let life = player_life::Profile::from_json(respawn, &stats_profile)?;
        if policy.creature() != creature_profile.id() {
            return Err("Retaliation references another creature".into());
        }
        let mut bytes = b"theia-player-life-and-retaliation-profile-v1\0".to_vec();
        for raw in [stats, creature, retaliation, respawn] {
            bytes.extend((raw.len() as u32).to_le_bytes());
            bytes.extend(raw)
        }
        Ok(Self {
            hash: crate::terrain::sha256(&bytes),
            stats_hash: crate::terrain::sha256(stats),
            creature_hash: crate::terrain::sha256(creature),
            creature: creature_profile.id().clone(),
            life: Arc::new(life),
            retaliation: Arc::new(policy),
        })
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn stats_hash(&self) -> [u8; 32] {
        self.stats_hash
    }
    pub fn creature_hash(&self) -> [u8; 32] {
        self.creature_hash
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn life(&self) -> &player_life::Profile {
        &self.life
    }
    pub fn retaliation(&self) -> &retaliation::Profile {
        &self.retaliation
    }
}
pub fn profile_identity(base: [u8; 32], profile: &Profile) -> [u8; 32] {
    let mut bytes = b"theia-player-life-and-retaliation-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile.hash);
    crate::terrain::sha256(&bytes)
}

use crate::{
    prediction::Correction,
    simulation::combat::player_life::{Observation, Windup},
};
use std::{num::NonZeroU64, time::Duration};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
pub const TAG: &[u8; 4] = b"PL01";
pub const MIN_FRAME_BYTES: usize = 86;
pub const MAX_FRAME_BYTES: usize = 110;
pub fn is_state(bytes: &[u8]) -> bool {
    bytes.starts_with(TAG)
}
pub fn is_any_state(bytes: &[u8]) -> bool {
    is_state(bytes) || crate::combat_membership_wire::is_state(bytes)
}
fn valid(profile: &Profile, rate: TickRate, value: &Observation) -> bool {
    if value.combat.is_some()
        || value.profile != profile.hash()
        || value.movement_stream == 0
        || value.maximum_health != profile.life().stats().maximum_health()
        || value.health > value.maximum_health
        || (value.health == 0) != value.respawn_at.is_some()
    {
        return false;
    }
    let minimum_stream = value
        .key
        .generation
        .get()
        .checked_sub(1)
        .and_then(|g| g.checked_mul(2))
        .and_then(|v| v.checked_add(if value.health == 0 { 2 } else { 1 }));
    if minimum_stream.is_none_or(|minimum| value.movement_stream < minimum)
        || value.respawn_at == Some(0)
    {
        return false;
    }
    let ticks = |ms| {
        rate.ticks_for(Duration::from_millis(u64::from(ms)))
            .unwrap()
    };
    if value
        .respawn_at
        .is_some_and(|at| at.saturating_sub(value.tick) > ticks(profile.life().delay_ms()))
    {
        return false;
    }
    if let Some(w) = value.windup
        && (value.health == 0
            || w.target != value.key
            || w.creature.epoch != value.key.actor.epoch
            || w.creature == value.key.actor
            || w.completes <= value.tick
            || w.completes - value.tick > ticks(profile.retaliation().windup_ms()))
    {
        return false;
    }
    true
}
fn write_key(out: &mut Vec<u8>, key: EntityKey) {
    out.extend(key.epoch.value().to_le_bytes());
    out.extend(key.entity.value().to_le_bytes());
}
pub fn encode(
    profile: &Profile,
    rate: TickRate,
    value: &Observation,
) -> std::result::Result<Vec<u8>, &'static str> {
    if !valid(profile, rate, value) {
        return Err("Invalid player-life observation");
    }
    let mut out = Vec::with_capacity(MAX_FRAME_BYTES);
    out.extend(TAG);
    out.push(rate.hz() as u8);
    out.extend(value.profile);
    write_key(&mut out, value.key.actor);
    out.extend(value.key.generation.get().to_le_bytes());
    out.extend(value.tick.to_le_bytes());
    out.extend(value.movement_stream.to_le_bytes());
    out.extend(value.health.to_le_bytes());
    out.extend(value.maximum_health.to_le_bytes());
    out.push(u8::from(value.respawn_at.is_some()) | (u8::from(value.windup.is_some()) << 1));
    if let Some(at) = value.respawn_at {
        out.extend(at.to_le_bytes());
    }
    if let Some(w) = value.windup {
        write_key(&mut out, w.creature);
        out.extend(w.completes.to_le_bytes());
    }
    debug_assert!(out.len() <= MAX_FRAME_BYTES);
    Ok(out)
}
struct Reader<'a>(&'a [u8]);
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> std::result::Result<[u8; N], &'static str> {
        if self.0.len() < N {
            return Err("Truncated player-life frame");
        }
        let (a, b) = self.0.split_at(N);
        self.0 = b;
        Ok(a.try_into().unwrap())
    }
    fn u64(&mut self) -> std::result::Result<u64, &'static str> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn key(&mut self) -> std::result::Result<EntityKey, &'static str> {
        Ok(EntityKey {
            epoch: WorldEpoch::new(self.u64()?).ok_or("Invalid life epoch")?,
            entity: EntityId::new(self.u64()?).ok_or("Invalid life actor")?,
        })
    }
}
/// Bounded header inspection only. Full decode must succeed before publishing.
pub fn frame_rate(bytes: &[u8]) -> std::result::Result<TickRate, &'static str> {
    if !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len()) || !is_state(bytes) {
        return Err("Invalid player-life frame header");
    }
    TickRate::from_hz(u32::from(bytes[4])).ok_or("Unsupported player-life tick rate")
}
pub fn decode(
    profile: &Profile,
    actor: EntityKey,
    rate: TickRate,
    bytes: &[u8],
) -> std::result::Result<Observation, &'static str> {
    if !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len()) {
        return Err("Player-life frame exceeds bounds");
    }
    if frame_rate(bytes)? != rate {
        return Err("Player-life tick rate mismatch");
    }
    let mut r = Reader(bytes);
    if &r.take::<4>()? != TAG {
        return Err("Unknown player-life frame");
    }
    r.take::<1>()?;
    let profile_hash = r.take()?;
    let source = r.key()?;
    if source != actor {
        return Err("Foreign player-life observation");
    }
    let key = LifeKey {
        actor: source,
        generation: NonZeroU64::new(r.u64()?).ok_or("Invalid player life generation")?,
    };
    let tick = r.u64()?;
    let movement_stream = r.u64()?;
    let health = u32::from_le_bytes(r.take()?);
    let maximum_health = u32::from_le_bytes(r.take()?);
    let flags = r.take::<1>()?[0];
    if flags & !3 != 0 {
        return Err("Unknown player-life flags");
    }
    let respawn_at = if flags & 1 != 0 { Some(r.u64()?) } else { None };
    let windup = if flags & 2 != 0 {
        Some(Windup {
            creature: r.key()?,
            target: key,
            completes: r.u64()?,
        })
    } else {
        None
    };
    let value = Observation {
        combat: None,
        profile: profile_hash,
        key,
        tick,
        movement_stream,
        health,
        maximum_health,
        respawn_at,
        windup,
    };
    if !r.0.is_empty() || !valid(profile, rate, &value) {
        return Err("Invalid player-life observation");
    }
    Ok(value)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    Changed,
    Duplicate,
    Stale,
}
/// Own-session receive state. Life/stream changes establish a correction barrier;
/// ordinary health/windup updates do not repeatedly pause movement prediction.
#[derive(Clone, Debug)]
pub struct ClientState {
    profile: Arc<Profile>,
    actor: EntityKey,
    rate: TickRate,
    world: [u8; 32],
    current: Option<Observation>,
    membership: Option<Arc<crate::combat_membership_wire::Profile>>,
    barrier_tick: u64,
}
impl ClientState {
    pub fn new(
        profile: Arc<Profile>,
        actor: EntityKey,
        rate: TickRate,
        world: [u8; 32],
    ) -> std::result::Result<Self, &'static str> {
        if world == [0; 32] {
            return Err("Invalid life world identity");
        }
        Ok(Self {
            profile,
            actor,
            rate,
            world,
            current: None,
            membership: None,
            barrier_tick: 0,
        })
    }
    pub fn with_membership(
        profile: Arc<Profile>,
        membership: Arc<crate::combat_membership_wire::Profile>,
        actor: EntityKey,
        rate: TickRate,
        world: [u8; 32],
    ) -> std::result::Result<Self, &'static str> {
        let mut state = Self::new(profile, actor, rate, world)?;
        state.membership = Some(membership);
        Ok(state)
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn rate(&self) -> TickRate {
        self.rate
    }
    pub fn current(&self) -> Option<&Observation> {
        self.current.as_ref()
    }
    pub fn apply(&mut self, bytes: &[u8]) -> std::result::Result<Applied, &'static str> {
        let value = if let Some(membership) = &self.membership {
            crate::combat_membership_wire::decode(
                &self.profile,
                membership,
                self.actor,
                self.rate,
                bytes,
            )?
        } else {
            decode(&self.profile, self.actor, self.rate, bytes)?
        };
        if let Some(old) = self.current {
            if value.key.generation > old.key.generation
                && value.movement_stream <= old.movement_stream
            {
                return Err("New life has no movement reset");
            }
            if value.key.generation < old.key.generation
                || value.movement_stream < old.movement_stream
                || value.tick < old.tick
            {
                return Ok(Applied::Stale);
            }
            if value.tick == old.tick {
                return if value == old {
                    Ok(Applied::Duplicate)
                } else {
                    Err("Conflicting life observations at one tick")
                };
            }
            if value.key.generation == old.key.generation && value.health > old.health {
                return Err("Health increased without a new life");
            }
            if value.key.generation == old.key.generation
                && old.health > 0
                && value.health == 0
                && value.movement_stream == old.movement_stream
            {
                return Err("Death has no movement reset");
            }
            if value.key.generation != old.key.generation
                || value.movement_stream != old.movement_stream
            {
                self.barrier_tick = value.tick;
            }
        } else {
            self.barrier_tick = value.tick;
        }
        self.current = Some(value);
        Ok(Applied::Changed)
    }
    pub fn movement_allowed(&self, correction: &Correction) -> bool {
        self.current.is_some_and(|value| {
            value.health > 0
                && correction.actor == self.actor
                && correction.world == self.world
                && correction.rate == self.rate
                && correction.input.stream_epoch == value.movement_stream
                && correction.input.tick >= self.barrier_tick
        })
    }
}
