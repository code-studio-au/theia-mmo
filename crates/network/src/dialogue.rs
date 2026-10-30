//! Owner-side informational interaction contracts. Transport encoding and Bevy
//! adoption are separate; none of these structs authorize a client actor ID.
use std::sync::Arc;
use theia_core::{
    content::ContentId, coordinates::GlobalPosition, dialogue::Profile as CoreProfile,
    identity::EntityKey,
};
pub const INTERACTION_METRES: f64 = 3.;
pub const REQUESTS_PER_SECOND: u8 = 8;

#[derive(Clone, Debug)]
pub struct Profile {
    profile: Arc<CoreProfile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> crate::Result<Self> {
        let profile = Arc::new(CoreProfile::from_json(bytes)?);
        let mut bound = theia_core::dialogue::PROFILE_DOMAIN.to_vec();
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
pub fn profile_identity(base: [u8; 32], profile: [u8; 32], placement: [f64; 2]) -> [u8; 32] {
    let mut bytes = b"theia-informational-guide-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile);
    for value in placement {
        bytes.extend(value.to_le_bytes());
    }
    crate::terrain::sha256(&bytes)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub key: EntityKey,
    pub position: GlobalPosition,
}
pub fn in_range(actor: GlobalPosition, guide: GlobalPosition) -> bool {
    let a = actor.metres();
    let b = guide.metres();
    a.into_iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        <= INTERACTION_METRES * INTERACTION_METRES
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Open(EntityKey),
    Choose(ContentId),
    Close,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub sequence: u64,
    pub expected_revision: u64,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    pub guide: EntityKey,
    pub node: ContentId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    /// Includes closure, preventing a delayed prior node from reopening a panel.
    pub revision: u64,
    pub open: Option<Open>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Unavailable,
    RateLimited,
    StaleSequence,
    StaleRevision,
    InvalidAction,
    OutOfRange,
    Exhausted,
}
