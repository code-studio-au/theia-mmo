//! Checked development progression arithmetic. A proposal establishes neither
//! kill credit nor event deduplication, ownership, persistence or award success.
use crate::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
pub const MAX_LEVELS: usize = 32;
pub const MAX_EXPERIENCE: u32 = 10_000;
pub const MAX_KILL_DEFINITIONS: usize = 16;
pub const MAX_KILL_EXPERIENCE: u32 = 1000;
pub const MAX_REVISION: u64 = i64::MAX as u64;
pub const PROFILE_DOMAIN: &[u8] = b"theia-progression-profile-v1\0";
/// Receipt identity for a server-owned victim life, distinct from a persistent
/// character and a cast sequence. The adapter must still establish kill credit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KillId(EntityKey);
/// Trusted authority fact only. Source-to-character credit must be established
/// by the runtime adapter; neither amount nor recipient comes from the client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KillEvent {
    pub id: KillId,
    pub creature: ContentId,
}
impl KillId {
    pub fn from_victim(victim: EntityKey) -> Self {
        Self(victim)
    }
    pub fn victim(self) -> EntityKey {
        self.0
    }
    pub fn bytes(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&self.0.epoch.value().to_le_bytes());
        bytes[8..].copy_from_slice(&self.0.entity.value().to_le_bytes());
        bytes
    }
    pub fn from_bytes(bytes: [u8; 16]) -> Option<Self> {
        Some(Self(EntityKey {
            epoch: WorldEpoch::new(u64::from_le_bytes(bytes[..8].try_into().unwrap()))?,
            entity: EntityId::new(u64::from_le_bytes(bytes[8..].try_into().unwrap()))?,
        }))
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    level_thresholds: Vec<u32>,
    kill_rewards: Vec<RawKill>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawKill {
    creature: String,
    experience: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    thresholds: Vec<u32>,
    kills: BTreeMap<ContentId, u32>,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Progression profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid progression profile")?;
        if raw.schema != 1
            || !(2..=MAX_LEVELS).contains(&raw.level_thresholds.len())
            || raw.level_thresholds[0] != 0
            || raw.level_thresholds.windows(2).any(|w| w[0] >= w[1])
            || raw
                .level_thresholds
                .last()
                .is_none_or(|n| *n > MAX_EXPERIENCE)
            || !(1..=MAX_KILL_DEFINITIONS).contains(&raw.kill_rewards.len())
        {
            return Err("Progression curve/reward limits invalid");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid progression profile ID")?;
        let mut kills = BTreeMap::new();
        for reward in raw.kill_rewards {
            let creature =
                ContentId::new(&reward.creature).ok_or("Invalid progression creature ID")?;
            if !(1..=MAX_KILL_EXPERIENCE).contains(&reward.experience)
                || kills.insert(creature, reward.experience).is_some()
            {
                return Err("Invalid/duplicate progression kill reward");
            }
        }
        Ok(Self {
            id,
            thresholds: raw.level_thresholds,
            kills,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn thresholds(&self) -> &[u32] {
        &self.thresholds
    }
    pub fn maximum_experience(&self) -> u32 {
        *self.thresholds.last().unwrap()
    }
    pub fn kill_rewards(&self) -> impl ExactSizeIterator<Item = (&ContentId, u32)> {
        self.kills.iter().map(|(id, xp)| (id, *xp))
    }
    pub fn kill_experience(&self, creature: &ContentId) -> Option<u32> {
        self.kills.get(creature).copied()
    }
    fn validate(&self, state: &State) -> Result<(), Error> {
        if state.profile != self.id {
            return Err(Error::ProfileMismatch);
        }
        if state.revision > MAX_REVISION || state.total > self.maximum_experience() {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    pub fn progress(&self, state: &State) -> Result<Progress, Error> {
        self.validate(state)?;
        let level = self
            .thresholds
            .partition_point(|threshold| *threshold <= state.total);
        let start = self.thresholds[level - 1];
        Ok(Progress {
            level: level as u8,
            total: state.total,
            earned_in_level: state.total - start,
            required_in_level: self.thresholds.get(level).map(|next| next - start),
        })
    }
    /// Trusted pure proposal only. The durable event ledger must reject replay
    /// before invoking this against a newer state. No client chooses XP amounts.
    pub fn propose_kill(&self, state: &State, creature: &ContentId) -> Result<Proposal, Error> {
        let before = self.progress(state)?;
        let amount = self
            .kill_experience(creature)
            .ok_or(Error::UnknownCreature)?;
        let granted = amount.min(self.maximum_experience() - state.total);
        if granted == 0 {
            return Ok(Proposal {
                state: state.clone(),
                granted: 0,
                previous_level: before.level,
                level: before.level,
            });
        }
        let revision = state
            .revision
            .checked_add(1)
            .filter(|r| *r <= MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let next = State::new(
            self,
            revision,
            state
                .total
                .checked_add(granted)
                .ok_or(Error::InvalidState)?,
        )?;
        let level = self.progress(&next)?.level;
        Ok(Proposal {
            state: next,
            granted,
            previous_level: before.level,
            level,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    profile: ContentId,
    revision: u64,
    total: u32,
}
impl State {
    /// Logical identity is checked here; storage and wire must also bind the
    /// exact profile digest. Incompatible content is never silently migrated.
    pub fn new(profile: &Profile, revision: u64, total: u32) -> Result<Self, Error> {
        let state = Self {
            profile: profile.id.clone(),
            revision,
            total,
        };
        profile.validate(&state)?;
        Ok(state)
    }
    pub fn profile(&self) -> &ContentId {
        &self.profile
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn total(&self) -> u32 {
        self.total
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub level: u8,
    pub total: u32,
    pub earned_in_level: u32,
    pub required_in_level: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub state: State,
    pub granted: u32,
    pub previous_level: u8,
    pub level: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidState,
    ProfileMismatch,
    UnknownCreature,
    RevisionExhausted,
}
