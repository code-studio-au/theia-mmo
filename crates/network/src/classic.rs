//! Strict classic movement/facing profile. No camera or target state is accepted.
use crate::{prediction::Pose, wire::Intent};
use std::collections::BTreeSet;
use theia_core::{coordinates::ChunkId, time::TickRate};

pub fn profile_identity(source: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-classic-facing-targeting-v2\0".to_vec();
    bytes.extend(source);
    crate::terrain::sha256(&bytes)
}

pub fn terrain_step(
    patch: &crate::procedural::Patch,
    ready: &BTreeSet<ChunkId>,
    pose: Pose,
    intent: Intent,
    rate: TickRate,
) -> Result<Option<Pose>, &'static str> {
    let facing = pose.facing.ok_or("Missing classic character facing")?;
    if pose.dodge.is_some() || intent.dodge.is_some() {
        return Err("Dodge is excluded from classic movement");
    }
    let mut base = pose;
    base.facing = None;
    let mut movement = intent;
    movement.facing = None;
    Ok(
        crate::prediction::terrain_step(patch, ready, base, movement, rate, 1.)?.map(|p| Pose {
            facing: Some(intent.facing.unwrap_or(facing)),
            ..p
        }),
    )
}
