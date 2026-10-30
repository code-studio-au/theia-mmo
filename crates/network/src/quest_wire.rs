//! Private committed objective state. No client credit/count command exists.
use crate::Result;
use std::sync::Arc;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::{Profile as CoreProfile, State as CoreState},
};
pub const STATE_TAG: &[u8; 4] = b"QO01";
pub const STATE_BYTES: usize = 64;
#[derive(Clone, Debug)]
pub struct Profile {
    profile: Arc<CoreProfile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let profile = Arc::new(CoreProfile::from_json(bytes)?);
        let mut bound = theia_core::quests::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        Ok(Self {
            profile,
            hash: crate::terrain::sha256(&bound),
        })
    }
    pub fn profile(&self) -> &CoreProfile {
        &self.profile
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
pub fn profile_identity(base: [u8; 32], profile: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-objective-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile);
    crate::terrain::sha256(&bytes)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    pub progress: CoreState,
}
impl State {
    pub fn encode(&self, profile: &Profile) -> Result<Vec<u8>> {
        if self.profile_hash == [0; 32] || self.profile_hash != profile.hash() {
            return Err("Objective observation profile mismatch".into());
        }
        profile
            .profile()
            .progress(&self.progress)
            .map_err(|_| "Invalid objective state")?;
        let mut bytes = STATE_TAG.to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.profile_hash);
        bytes.extend(self.progress.revision().to_le_bytes());
        bytes.extend(self.progress.count().to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], profile: &Profile) -> Result<Self> {
        if bytes.len() != STATE_BYTES || !bytes.starts_with(STATE_TAG) {
            return Err("Invalid objective observation frame".into());
        }
        let word = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        let state = Self {
            actor: EntityKey {
                epoch: WorldEpoch::new(word(4)).ok_or("Invalid objective epoch")?,
                entity: EntityId::new(word(12)).ok_or("Invalid objective actor")?,
            },
            profile_hash: bytes[20..52].try_into().unwrap(),
            progress: CoreState::new(
                profile.profile(),
                word(52),
                u32::from_le_bytes(bytes[60..64].try_into().unwrap()),
            )
            .map_err(|_| "Invalid objective progress")?,
        };
        state.encode(profile)?;
        Ok(state)
    }
    pub fn validate_successor(&self, actor: EntityKey, prior: Option<&Self>) -> Result<()> {
        if self.actor != actor
            || prior.is_some_and(|old| {
                self.profile_hash != old.profile_hash
                    || self.progress.revision() < old.progress.revision()
                    || self.progress.count() < old.progress.count()
                    || (self.progress.revision() == old.progress.revision() && self != old)
                    || (self.progress.revision() > old.progress.revision()
                        && self.progress.count() <= old.progress.count())
            })
        {
            return Err("Foreign/regressed objective observation".into());
        }
        Ok(())
    }
}
