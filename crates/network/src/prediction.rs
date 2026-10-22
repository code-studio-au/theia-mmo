//! Bounded P2-04 prediction/reconciliation core; not yet a wire-v11 client mode.
//! Callers own the fixed clock, transport and prepared collision. No I/O, asset
//! generation, rendering types or client-supplied authority positions live here.
use crate::{
    commands::{Command, State, Timeline},
    kinematic::MotionState,
    wire::Intent,
};
use std::collections::VecDeque;
use theia_core::{coordinates::GlobalPosition, identity::EntityKey, time::TickRate};

/// At most ~1.07 seconds at 60 Hz, ~3.2 seconds at 20 Hz. Overflow suspends
/// speculation rather than evicting input that may still require replay.
pub const MAX_HISTORY: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Capsule feet, not the presentation model's centre.
    pub feet: GlobalPosition,
    pub motion: MotionState,
}
/// Shared-controller adapter for an already-prepared graybox query patch.
/// Permission limits XZ sweeps; unavailable permission still allows gravity.
/// The owner supplies the same qualified development speed as the authority.
pub fn terrain_step(
    patch: &crate::procedural::Patch,
    ready: &std::collections::BTreeSet<theia_core::coordinates::ChunkId>,
    pose: Pose,
    intent: Intent,
    rate: TickRate,
    speed: f32,
) -> Result<Option<Pose>, &'static str> {
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
    .map_err(|_| "Invalid prepared terrain coverage")?;
    let mut motion = crate::kinematic::Motion::restore(pose.motion)?;
    let feet = crate::kinematic::advance_development(
        &patch.scene,
        &mut motion,
        start,
        intent,
        rate,
        Some(&coverage),
        speed,
    )?;
    let feet = patch
        .frame
        .to_global(feet)
        .ok_or("Terrain pose left global bounds")?;
    if !patch.covers(feet, None) {
        return Ok(None);
    }
    Ok(Some(Pose {
        feet,
        motion: motion.snapshot(),
    }))
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Correction {
    pub world: [u8; 32],
    pub actor: EntityKey,
    pub rate: TickRate,
    /// Includes the independent movement-stream epoch and completed server tick.
    pub input: State,
    pub pose: Pose,
    /// Exact authority-accepted terrain readiness report; zero is initial/empty.
    pub ready_sequence: u64,
}
impl Correction {
    pub fn valid(self) -> bool {
        self.pose.motion.valid() && Timeline::restore(self.rate, self.input).is_ok()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    StreamChanged,
    ReadinessChanged,
    HistoryFull,
    CollisionUnavailable,
    SolverFailed,
    ClockExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reconciled {
    Stale,
    Applied {
        replayed: usize,
        correction_metres: f64,
    },
    Reset(Reset),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Predicted {
    Advanced(Option<Command>),
    Suspended(Reset),
}
#[derive(Clone, Copy)]
struct Step {
    tick: u64,
    command: Option<Command>,
}

pub struct History {
    anchor: Correction,
    input: Timeline,
    pose: Pose,
    steps: VecDeque<Step>,
    /// Never reuse a sequence after dropping speculative history in this stream.
    last_sequence: u64,
    suspended: Option<Reset>,
}
impl History {
    pub fn new(anchor: Correction) -> Result<Self, &'static str> {
        if !anchor.valid() {
            return Err("Invalid prediction anchor");
        }
        Ok(Self {
            anchor,
            input: Timeline::restore(anchor.rate, anchor.input)?,
            pose: anchor.pose,
            steps: VecDeque::with_capacity(MAX_HISTORY),
            last_sequence: anchor.input.applied_sequence,
            suspended: None,
        })
    }
    pub fn pose(&self) -> Pose {
        self.pose
    }
    pub fn input(&self) -> State {
        self.input.state()
    }
    pub fn retained(&self) -> usize {
        self.steps.len()
    }
    pub fn suspended(&self) -> Option<Reset> {
        self.suspended
    }
    /// Drop speculative clock debt while preserving sequence allocation in this
    /// movement stream. The latest validated correction remains the anchor.
    pub fn resync_clock(&mut self) {
        self.reset(Reset::ClockExhausted);
        self.suspended = None;
    }
    fn reset(&mut self, reason: Reset) {
        self.input = Timeline::restore(self.anchor.rate, self.anchor.input)
            .expect("Previously validated anchor");
        self.pose = self.anchor.pose;
        self.steps.clear();
        self.suspended = Some(reason);
    }
    /// One fixed speculative step. A command is optimistically received just
    /// before its target tick; corrections replace that guess with server truth.
    /// `None` advances held-input expiry without creating a command. The solver
    /// must use already-prepared, readiness-constrained collision and return None
    /// when it cannot safely query it. Never send the returned pose as authority.
    pub fn predict(
        &mut self,
        intent: Option<Intent>,
        ready_sequence: u64,
        mut solve: impl FnMut(Pose, Intent, TickRate) -> Result<Option<Pose>, &'static str>,
    ) -> Result<Predicted, &'static str> {
        if intent.is_some_and(|intent| !intent.valid()) {
            return Err("Invalid speculative input");
        }
        if let Some(reason) = self.suspended {
            return Ok(Predicted::Suspended(reason));
        }
        let reason = if ready_sequence != self.anchor.ready_sequence {
            Some(Reset::ReadinessChanged)
        } else if self.steps.len() == MAX_HISTORY {
            Some(Reset::HistoryFull)
        } else {
            None
        };
        if let Some(reason) = reason {
            self.reset(reason);
            return Ok(Predicted::Suspended(reason));
        }
        let Some(tick) = self.input.state().tick.checked_add(1) else {
            self.reset(Reset::ClockExhausted);
            return Ok(Predicted::Suspended(Reset::ClockExhausted));
        };
        let command = if let Some(intent) = intent {
            let Some(sequence) = self.last_sequence.checked_add(1) else {
                self.reset(Reset::ClockExhausted);
                return Ok(Predicted::Suspended(Reset::ClockExhausted));
            };
            Some(Command {
                sequence,
                tick,
                intent,
            })
        } else {
            None
        };
        if let Some(command) = command {
            self.input.offer(self.anchor.input.stream_epoch, command)?;
        }
        let applied = self.input.advance()?;
        let pose = match solve(
            self.pose,
            applied.effective_intent(self.pose.motion),
            self.anchor.rate,
        ) {
            Ok(Some(pose)) if pose.motion.valid() => pose,
            Ok(None) => {
                self.reset(Reset::CollisionUnavailable);
                return Ok(Predicted::Suspended(Reset::CollisionUnavailable));
            }
            _ => {
                self.reset(Reset::SolverFailed);
                return Ok(Predicted::Suspended(Reset::SolverFailed));
            }
        };
        self.pose = pose;
        if let Some(command) = command {
            self.last_sequence = command.sequence;
        }
        self.steps.push_back(Step { tick, command });
        Ok(Predicted::Advanced(command))
    }
    /// Validate identity before mutation; old corrections cannot rewind progress.
    /// Restore completed truth, discard completed ticks and replay only the suffix.
    /// A changed stream/readiness resets instead of replaying incompatible input.
    pub fn reconcile(
        &mut self,
        correction: Correction,
        mut solve: impl FnMut(Pose, Intent, TickRate) -> Result<Option<Pose>, &'static str>,
    ) -> Result<Reconciled, &'static str> {
        if !correction.valid()
            || correction.actor != self.anchor.actor
            || correction.world != self.anchor.world
            || correction.rate != self.anchor.rate
        {
            return Err("Invalid or foreign prediction correction");
        }
        if correction.input.stream_epoch < self.anchor.input.stream_epoch
            || correction.input.tick < self.anchor.input.tick
            || (correction.input.stream_epoch == self.anchor.input.stream_epoch
                && correction.input.tick == self.anchor.input.tick)
        {
            return Ok(Reconciled::Stale);
        }
        let changed = if correction.input.stream_epoch != self.anchor.input.stream_epoch {
            Some(Reset::StreamChanged)
        } else if correction.ready_sequence != self.anchor.ready_sequence {
            Some(Reset::ReadinessChanged)
        } else {
            None
        };
        if correction.input.stream_epoch == self.anchor.input.stream_epoch
            && (correction.input.applied_sequence < self.anchor.input.applied_sequence
                || correction.ready_sequence < self.anchor.ready_sequence)
        {
            return Err("Completed input or readiness sequence regressed");
        }
        let before = self.pose.feet.metres();
        self.anchor = correction;
        if let Some(reason) = changed {
            self.last_sequence = if reason == Reset::StreamChanged {
                correction.input.applied_sequence
            } else {
                self.last_sequence.max(correction.input.applied_sequence)
            };
            self.reset(reason);
            // The fresh correction is already the new stream/readiness anchor.
            self.suspended = None;
            return Ok(Reconciled::Reset(reason));
        }
        self.input = Timeline::restore(correction.rate, correction.input)?;
        self.pose = correction.pose;
        self.suspended = None;
        self.last_sequence = self.last_sequence.max(correction.input.applied_sequence);
        while self
            .steps
            .front()
            .is_some_and(|step| step.tick <= correction.input.tick)
        {
            self.steps.pop_front();
        }
        let mut replayed = 0;
        for step in &self.steps {
            // A server-ahead correction drains the entire history. Otherwise
            // the retained suffix must be contiguous; no guessed catch-up ticks.
            if self.input.state().tick.checked_add(1) != Some(step.tick) {
                self.reset(Reset::ClockExhausted);
                return Ok(Reconciled::Reset(Reset::ClockExhausted));
            }
            if let Some(command) = step.command
                && command.sequence > correction.input.applied_sequence
            {
                self.input.offer(correction.input.stream_epoch, command)?;
            }
            let applied = self.input.advance()?;
            self.pose = match solve(
                self.pose,
                applied.effective_intent(self.pose.motion),
                correction.rate,
            ) {
                Ok(Some(pose)) if pose.motion.valid() => pose,
                Ok(None) => {
                    self.reset(Reset::CollisionUnavailable);
                    return Ok(Reconciled::Reset(Reset::CollisionUnavailable));
                }
                _ => {
                    self.reset(Reset::SolverFailed);
                    return Ok(Reconciled::Reset(Reset::SolverFailed));
                }
            };
            replayed += 1;
        }
        let after = self.pose.feet.metres();
        let correction_metres = before
            .into_iter()
            .zip(after)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        Ok(Reconciled::Applied {
            replayed,
            correction_metres,
        })
    }
}
