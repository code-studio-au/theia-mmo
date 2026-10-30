//! Exact objective profile and committed earned-progression results.
use crate::{Error, ProgressionApplied, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::quests::{Profile, State};
#[derive(Clone, Debug)]
pub struct QuestCatalog {
    profile: Arc<Profile>,
    hash: [u8; 32],
}
impl QuestCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let profile = Arc::new(Profile::from_json(bytes).map_err(Error::Invalid)?);
        let mut bound = theia_core::quests::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        Ok(Self {
            profile,
            hash: digest(&SHA256, &bound).as_ref().try_into().unwrap(),
        })
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub(crate) fn check_progression(&self, xp: &crate::ProgressionCatalog) -> Result<()> {
        if xp
            .profile()
            .kill_experience(self.profile.creature())
            .is_none()
        {
            return Err(Error::Invalid(
                "Quest creature is absent from progression profile",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestApplied {
    pub state: State,
    pub advanced: bool,
    pub completed_now: bool,
    pub replayed: bool,
}
/// Both enabled progression effects are committed before this is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EarnedProgression {
    pub experience: ProgressionApplied,
    pub quest: QuestApplied,
}
