//! Server-owned static targets and explicit opt-in profile identity; no physics bodies.
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};

pub const MAX_TARGETS: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub key: EntityKey,
    /// Authoritative body centre in checked global metres.
    pub position: GlobalPosition,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub selected: Option<EntityKey>,
    pub targets: Vec<Target>,
}
/// Superseding local choice. None in the watch mailbox means no request yet;
/// a Choice containing None explicitly clears selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub target: Option<theia_core::identity::EntityId>,
}
/// Match the existing player snapshot's bounded XZ relevance policy.
/// This is neither line of sight nor an ability hit/range test.
pub(crate) fn relevant(from: GlobalPosition, to: GlobalPosition) -> bool {
    let a = from.metres();
    let b = to.metres();
    (a[0] - b[0]).abs() <= 1024. && (a[2] - b[2]).abs() <= 1024.
}

/// Bind this opt-in protocol profile independently of immutable terrain identity.
pub fn profile_identity(terrain: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-targeting-profile-v1\0".to_vec();
    bytes.extend(terrain);
    crate::terrain::sha256(&bytes)
}
