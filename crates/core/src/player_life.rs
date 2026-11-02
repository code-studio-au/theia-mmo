//! Checked transient development health/respawn. No saved-state migration,
//! movement teleport or client damage authority is supplied by this contract.
use crate::{
    content::ContentId,
    identity::EntityKey,
    resources::{self, Pool},
    time::TickRate,
};
use serde::Deserialize;
use std::{num::NonZeroU64, time::Duration};
pub const MAX_PROFILE_BYTES: usize = 4096;
pub const PROFILE_DOMAIN: &[u8] = b"theia-player-respawn-profile-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    stats: String,
    delay_ms: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    stats: resources::Profile,
    delay_ms: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8], stats: &resources::Profile) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Respawn profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid respawn profile")?;
        if raw.schema != 1 {
            return Err("Unsupported respawn schema");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid respawn profile ID")?;
        if ContentId::new(&raw.stats).as_ref() != Some(stats.id())
            || !(100..=60000).contains(&raw.delay_ms)
        {
            return Err("Respawn stats reference or delay is invalid");
        }
        Ok(Self {
            id,
            stats: stats.clone(),
            delay_ms: raw.delay_ms,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn delay_ms(&self) -> u32 {
        self.delay_ms
    }
    pub fn stats(&self) -> &resources::Profile {
        &self.stats
    }
}
/// A player's session identity plus a particular spawned life within it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifeKey {
    pub actor: EntityKey,
    pub generation: NonZeroU64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    StaleLife,
    StaleTick,
    TimeExhausted,
    LifeExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Damage {
    pub applied: u32,
    pub remaining: u32,
    pub died: bool,
    pub respawn_at: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    actor: EntityKey,
    profile: Profile,
    health: Pool,
    delay_ticks: u64,
    generation: NonZeroU64,
    respawn_at: Option<u64>,
    observed: Option<u64>,
}
impl State {
    pub fn new(profile: Profile, rate: TickRate, actor: EntityKey) -> Self {
        let maximum = profile.stats.maximum_health();
        Self {
            actor,
            health: Pool::new(maximum, maximum).unwrap(),
            delay_ticks: rate
                .ticks_for(Duration::from_millis(u64::from(profile.delay_ms)))
                .unwrap(),
            profile,
            generation: NonZeroU64::MIN,
            respawn_at: None,
            observed: None,
        }
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn health(&self) -> u32 {
        self.health.current()
    }
    pub fn maximum_health(&self) -> u32 {
        self.health.maximum()
    }
    pub fn alive(&self) -> bool {
        self.health.current() > 0
    }
    pub fn respawn_at(&self) -> Option<u64> {
        self.respawn_at
    }
    pub fn key(&self) -> LifeKey {
        LifeKey {
            actor: self.actor,
            generation: self.generation,
        }
    }
    fn check_tick(&self, now: u64) -> Result<(), Error> {
        if self.observed.is_some_and(|last| now < last) {
            Err(Error::StaleTick)
        } else {
            Ok(())
        }
    }
    /// Only an already authorized server hit calls this. Overkill clips; repeated
    /// hits on a corpse do not extend the one scheduled respawn. Failure is atomic.
    pub fn damage(&mut self, now: u64, expected: LifeKey, amount: u32) -> Result<Damage, Error> {
        if expected != self.key() {
            return Err(Error::StaleLife);
        }
        self.check_tick(now)?;
        let applied = amount.min(self.health.current());
        let died = applied > 0 && applied == self.health.current();
        let respawn_at = if died {
            Some(
                now.checked_add(self.delay_ticks)
                    .ok_or(Error::TimeExhausted)?,
            )
        } else {
            self.respawn_at
        };
        let mut health = self.health;
        assert!(health.spend(applied));
        self.health = health;
        self.respawn_at = respawn_at;
        self.observed = Some(now);
        Ok(Damage {
            applied,
            remaining: health.current(),
            died,
            respawn_at,
        })
    }
    /// A due respawn is explicit and emitted once. The authority must complete
    /// its pose/input/cast barrier before advertising the new life to the client.
    pub fn advance(&mut self, now: u64) -> Result<bool, Error> {
        self.check_tick(now)?;
        let due = self.respawn_at.is_some_and(|at| now >= at);
        if due {
            let generation = self
                .generation
                .get()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or(Error::LifeExhausted)?;
            self.health = Pool::new(self.health.maximum(), self.health.maximum()).unwrap();
            self.generation = generation;
            self.respawn_at = None;
        }
        self.observed = Some(now);
        Ok(due)
    }
}
