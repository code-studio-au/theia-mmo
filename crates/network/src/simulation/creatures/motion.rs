//! Shared authoritative creature displacement through static world collision.
//! Input and output are global body centres; only the checked local controller
//! receives feet positions. A refused step leaves motion state unchanged.
use crate::{Result, chunks::Coverage, kinematic, procedural::Patch, wire::Intent};
use theia_core::{
    coordinates::{GlobalPosition, LocalPosition},
    time::TickRate,
};

pub(in crate::simulation) fn step(
    patch: &Patch,
    coverage: &Coverage,
    body: &mut kinematic::Motion,
    position: GlobalPosition,
    intent: Intent,
    rate: TickRate,
) -> Result<GlobalPosition> {
    let mut feet = patch
        .frame
        .to_local(position)
        .ok_or("Creature outside collision frame")?
        .metres();
    feet[1] -= crate::world::PAWN_Y;
    let feet = LocalPosition::new(feet).ok_or("Invalid creature feet")?;
    let mut proposed = *body;
    let feet = kinematic::advance_development(
        &patch.scene,
        &mut proposed,
        feet,
        intent,
        rate,
        Some(coverage),
        1.,
    )?;
    let mut centre = feet.metres();
    centre[1] += crate::world::PAWN_Y;
    let position = LocalPosition::new(centre)
        .and_then(|p| patch.frame.to_global(p))
        .ok_or("Invalid creature global position")?;
    *body = proposed;
    Ok(position)
}
