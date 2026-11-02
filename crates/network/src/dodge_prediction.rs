//! Inactive shared prediction prerequisite. Uses prepared terrain only; no
//! loading, transport, rendering or client-authority movement is introduced.
use crate::{
    dodging, kinematic,
    prediction::{Correction, Pose},
    procedural::Patch,
    wire::Intent,
};
use std::{collections::BTreeSet, sync::Arc};
use theia_core::{coordinates::ChunkId, dodge::Gate, time::TickRate};

/// Bind one immutable policy to the outer world's identity and fixed rate.
/// History/transport still own actor, life, movement-stream and readiness epochs.
pub struct Binding {
    profile: Arc<dodging::Profile>,
    rate: TickRate,
}
impl Binding {
    pub fn from_correction(
        source_world: [u8; 32],
        correction: Correction,
        profile: Arc<dodging::Profile>,
    ) -> Result<Self, &'static str> {
        if !correction.valid()
            || correction.world != dodging::profile_identity(source_world, &profile)
            || !correction
                .pose
                .dodge
                .is_some_and(|d| d.valid_at(&profile, correction.rate, correction.input.tick))
        {
            return Err("Unbound dodge prediction correction");
        }
        Ok(Self {
            profile,
            rate: correction.rate,
        })
    }
    /// One consecutive completed tick. Missing movement permission retires a
    /// dodge while retaining gravity; absent prepared collision suspends replay.
    #[allow(clippy::too_many_arguments)]
    pub fn terrain_step_at(
        &self,
        patch: &Patch,
        ready: &BTreeSet<ChunkId>,
        pose: Pose,
        intent: Intent,
        rate: TickRate,
        tick: u64,
        alive: bool,
    ) -> Result<Option<Pose>, &'static str> {
        if rate != self.rate || !intent.valid() {
            return Err("Invalid bound dodge prediction input");
        }
        let snapshot = pose.dodge.ok_or("Missing dodge prediction state")?;
        let mut policy =
            dodging::Controller::restore(&self.profile, rate, snapshot.policy.tick, snapshot)
                .map_err(|_| "Invalid dodge prediction state")?;
        let mut motion = kinematic::Motion::restore(pose.motion)?;
        if tick == snapshot.policy.tick {
            return Ok(Some(pose));
        }
        if snapshot.policy.tick.checked_add(1) != Some(tick) {
            return Err("Nonconsecutive dodge prediction tick");
        }
        if !patch.covers(pose.feet, None) {
            return Ok(None);
        }
        let Some(start) = patch.frame.to_local(pose.feet) else {
            return Ok(None);
        };
        let coverage = crate::chunks::Coverage::procedural_cells(
            patch.frame,
            patch.ids.intersection(ready).copied(),
        )
        .map_err(|_| "Invalid prepared dodge coverage")?;
        let advanced = policy
            .advance(
                &self.profile,
                rate,
                tick,
                intent.dodge,
                Gate {
                    alive,
                    grounded: motion.grounded(),
                    terrain_ready: coverage.contains(pose.feet),
                },
            )
            .map_err(|_| "Invalid dodge prediction clock")?;
        let mut movement = intent;
        movement.dodge = None;
        if !alive {
            movement.axes = [0, 0];
            movement.sprint = false;
            movement.jump = false;
        }
        let feet = match advanced.movement {
            dodging::Movement::None => return Ok(Some(pose)),
            dodging::Movement::Ordinary => kinematic::advance_development(
                &patch.scene,
                &mut motion,
                start,
                movement,
                rate,
                Some(&coverage),
                1.,
            )?,
            dodging::Movement::Dodge(step) => kinematic::advance_dodge(
                &patch.scene,
                &mut motion,
                start,
                movement,
                rate,
                Some(&coverage),
                step,
            )?,
        };
        let feet = patch
            .frame
            .to_global(feet)
            .ok_or("Dodge pose left global bounds")?;
        if !patch.covers(feet, None) {
            return Ok(None);
        }
        Ok(Some(Pose {
            facing: None,
            feet,
            motion: motion.snapshot(),
            dodge: Some(policy.snapshot()),
        }))
    }
}
