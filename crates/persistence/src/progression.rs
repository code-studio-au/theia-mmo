//! Exact immutable progression profile and committed-store result. Core proposals
//! alone do not establish an earned kill, receipt or durable award.
use crate::{Error, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::progression::{Profile, State};
#[derive(Clone, Debug)]
pub struct ProgressionCatalog {
    profile: Arc<Profile>,
    hash: [u8; 32],
}
impl ProgressionCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let profile = Arc::new(Profile::from_json(bytes).map_err(Error::Invalid)?);
        let mut bound = theia_core::progression::PROFILE_DOMAIN.to_vec();
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
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressionApplied {
    pub state: State,
    /// Newly granted quantity. Replays and at-cap no-ops grant zero.
    pub granted: u32,
    pub replayed: bool,
}
