//! Shared static support check for authored collectible locations.
use crate::{Result, procedural::Patch};
use theia_core::coordinates::GlobalPosition;

/// Uses only an already prepared collision patch. This checks nearby player
/// support/clearance, not navigation reachability from another location.
pub fn supported(patch: &Patch, position: GlobalPosition) -> Result<GlobalPosition> {
    patch
        .try_spawn_near(position)
        .ok_or_else(|| "Ground item has no supported placement with player clearance".into())
}
