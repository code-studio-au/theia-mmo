//! Shared opt-in client clock/command driver, used by the rendered client and
//! real-UDP qualification. Transport and prepared terrain remain caller-owned.
use crate::{
    client::State,
    commands::{Command, FUTURE_TICKS},
    prediction::{History, Pose, Predicted, Reconciled},
    prediction_wire::{Commands, MAX_COMMANDS},
    procedural::Patch,
    wire::Intent,
};
use std::{
    collections::{BTreeSet, VecDeque},
    time::Instant,
};
#[derive(Default, Clone, Debug, serde::Serialize)]
pub struct Stats {
    pub predicted_steps: u64,
    pub corrections: u64,
    pub replayed_steps: u64,
    pub resets: u64,
    pub clock_resets: u64,
    pub waiting_frames: u64,
    pub queue_drops: u64,
    pub max_history: usize,
    pub max_correction_metres: f64,
    pub max_lead_ticks: u64,
    pub same_tick_samples: u64,
    pub unmatched_tick_samples: u64,
    pub max_same_tick_error_metres: f64,
    pub max_same_tick_horizontal_metres: f64,
    pub max_same_tick_vertical_metres: f64,
    pub max_same_tick_vertical_speed_mps: f64,
    pub same_tick_grounded_mismatches: u64,
    pub same_tick_jump_held_mismatches: u64,
    /// Inclusive upper bounds below, then one overflow bucket. No sample log.
    pub same_tick_error_buckets: [u64; 8],
    pub pending_readiness_samples: u64,
    pub max_pending_readiness_error_metres: f64,
    pub stable_readiness_samples: u64,
    pub max_stable_readiness_error_metres: f64,
    pub stable_readiness_error_buckets: [u64; 8],
    pub max_clock_slew: f64,
    pub max_clock_phase_error_ticks: f64,
    pub clock_phase_backsteps: u64,
    pub clock_acquisitions: u64,
    pub held_intent_agreement_samples: u64,
    pub held_intent_mismatch_samples: u64,
    pub applied_sequence_mismatches: u64,
    pub input_freshness_mismatches: u64,
    pub max_held_intent_agreement_error_metres: f64,
    pub max_held_intent_mismatch_error_metres: f64,
}
pub const ERROR_BOUNDS_METRES: [f64; 7] = [0.0001, 0.001, 0.01, 0.05, 0.1, 0.5, 1.];
impl Stats {
    fn record_error(&mut self, error: Option<crate::prediction::PoseError>) {
        let Some(error) = error else {
            self.unmatched_tick_samples += 1;
            return;
        };
        let bucket = ERROR_BOUNDS_METRES.partition_point(|&bound| error.metres > bound);
        if error.readiness_pending {
            self.pending_readiness_samples += 1;
            self.max_pending_readiness_error_metres =
                self.max_pending_readiness_error_metres.max(error.metres);
        } else {
            self.stable_readiness_samples += 1;
            self.max_stable_readiness_error_metres =
                self.max_stable_readiness_error_metres.max(error.metres);
            self.stable_readiness_error_buckets[bucket] += 1;
        }
        if error.held_intent_mismatch {
            self.held_intent_mismatch_samples += 1;
            self.max_held_intent_mismatch_error_metres =
                self.max_held_intent_mismatch_error_metres.max(error.metres);
        } else {
            self.held_intent_agreement_samples += 1;
            self.max_held_intent_agreement_error_metres = self
                .max_held_intent_agreement_error_metres
                .max(error.metres);
        }
        self.applied_sequence_mismatches += u64::from(error.applied_sequence_mismatch);
        self.input_freshness_mismatches += u64::from(error.freshness_mismatch);
        self.same_tick_samples += 1;
        self.max_same_tick_error_metres = self.max_same_tick_error_metres.max(error.metres);
        self.max_same_tick_horizontal_metres = self
            .max_same_tick_horizontal_metres
            .max(error.horizontal_metres);
        self.max_same_tick_vertical_metres = self
            .max_same_tick_vertical_metres
            .max(error.vertical_metres);
        self.max_same_tick_vertical_speed_mps = self
            .max_same_tick_vertical_speed_mps
            .max(error.vertical_speed_mps);
        self.same_tick_grounded_mismatches += u64::from(error.grounded_mismatch);
        self.same_tick_jump_held_mismatches += u64::from(error.jump_held_mismatch);
        self.same_tick_error_buckets[bucket] += 1;
    }
}
#[derive(Default)]
pub struct Driver {
    history: Option<History>,
    accepted: Option<(u64, u64, u64)>,
    recent: VecDeque<Command>,
    clock: Option<(Instant, f64)>,
    clock_rate: f64,
    clock_ready: bool,
    pub stats: Stats,
}
pub struct Driven {
    pub pose: Pose,
    pub tick: u64,
    pub packet: Option<Commands>,
    pub reset: bool,
}
impl Driver {
    pub fn pose(&self) -> Option<Pose> {
        self.history.as_ref().map(History::pose)
    }
    pub fn tick(
        &mut self,
        state: &State,
        intent: Intent,
        patch: Option<&Patch>,
        now: Instant,
    ) -> Result<Option<Driven>, &'static str> {
        let (Some(corrected), Some(received)) = (state.correction, state.correction_received)
        else {
            return Ok(None);
        };
        let c = corrected.state;
        let ready: BTreeSet<_> = state
            .correction_ready
            .as_ref()
            .filter(|r| r.sequence == c.ready_sequence)
            .map(|r| {
                r.chunks
                    .iter()
                    .filter(|id| {
                        state
                            .chunk_report
                            .as_ref()
                            .is_some_and(|current| current.chunks.contains(id))
                    })
                    .map(|&[x, z]| theia_core::coordinates::ChunkId { x, z })
                    .collect()
            })
            .unwrap_or_default();
        let solve = |pose, intent, rate| match patch {
            Some(patch) => crate::prediction::terrain_step(
                patch,
                &ready,
                pose,
                intent,
                rate,
                f32::from(corrected.speed),
            ),
            None => Ok(None),
        };
        let readiness_pending = state
            .chunk_report
            .as_ref()
            .is_none_or(|r| r.sequence != c.ready_sequence)
            || state.correction_ready.as_ref().is_some_and(|report| {
                report.sequence == c.ready_sequence
                    && report.chunks.iter().any(|&[x, z]| {
                        patch.is_none_or(|p| {
                            !p.ids.contains(&theia_core::coordinates::ChunkId { x, z })
                        })
                    })
            });
        if let Some(history) = &mut self.history {
            history.measure_pending_readiness(readiness_pending);
        }
        let key = (c.input.stream_epoch, c.input.tick, c.ready_sequence);
        let mut reset = false;
        let fresh = self.accepted != Some(key);
        if fresh {
            self.recent.retain(|command| {
                command.tick > c.input.tick && command.sequence > c.input.applied_sequence
            });
            self.accepted = Some(key);
            if let Some(history) = &mut self.history {
                match history.reconcile(c, solve)? {
                    Reconciled::Applied {
                        replayed,
                        correction_metres,
                        same_tick,
                    } => {
                        self.stats.corrections += 1;
                        self.stats.record_error(same_tick);
                        self.stats.replayed_steps += replayed as u64;
                        self.stats.max_correction_metres =
                            self.stats.max_correction_metres.max(correction_metres);
                    }
                    Reconciled::Reset(_) => {
                        self.stats.resets += 1;
                        self.recent.clear();
                        reset = true;
                    }
                    Reconciled::Stale => {}
                }
            } else {
                self.history = Some(History::new(c)?);
                reset = true;
            }
        }
        let history = self.history.as_mut().unwrap();
        history.measure_pending_readiness(readiness_pending);
        let elapsed = now.saturating_duration_since(received).as_secs_f64();
        let half_rtt = state
            .transport
            .as_ref()
            .map_or(0., |s| s.rtt_ms.max(0.) / 2000.);
        let window = FUTURE_TICKS.min(
            c.rate
                .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
                .unwrap(),
        );

        // Unsupported/stale clocks suspend command production instead of
        // inventing unbounded catch-up work or repeatedly sending late input.
        if !half_rtt.is_finite()
            || elapsed > 0.5
            || half_rtt * f64::from(c.rate.hz()) > (window - 2) as f64
        {
            self.stats.waiting_frames += 1;
            return Ok(Some(Driven {
                pose: history.pose(),
                tick: history.input().tick,
                packet: None,
                reset,
            }));
        }
        let lead = ((half_rtt * f64::from(c.rate.hz())).ceil() as u64 + 2).min(window - 1);
        self.stats.max_lead_ticks = self.stats.max_lead_ticks.max(lead);
        // Track prediction time, including input lead, at a positive bounded rate.
        // Ordinary RTT/lead changes adjust frequency rather than jumping phase.
        if c.input.tick >= (1u64 << 53) - 128 {
            return Err("Prediction clock precision exhausted");
        }
        let measured =
            c.input.tick as f64 + (elapsed + half_rtt) * f64::from(c.rate.hz()) + lead as f64;
        // Acquire the measured latency buffer once, during initial terrain
        // readiness. Do not start play catching up from a zero RTT estimate.
        let acquire = !self.clock_ready && half_rtt > 0. && !ready.is_empty() && patch.is_some();
        self.clock_ready |= acquire;
        if acquire {
            self.stats.clock_acquisitions += 1;
        }
        let mut estimate = self.clock.map_or(measured, |(last, tick)| {
            tick + now.saturating_duration_since(last).as_secs_f64()
                * f64::from(c.rate.hz())
                * self.clock_rate
        });
        if self.clock.is_none() || acquire {
            estimate = measured;
            self.clock_rate = 1.;
            reset |= acquire;
        } else if fresh {
            let error = measured - estimate;
            self.stats.max_clock_phase_error_ticks =
                self.stats.max_clock_phase_error_ticks.max(error.abs());
            if error.abs() > 8. {
                estimate = measured;
                self.clock_rate = 1.;
                reset = true;
            } else {
                self.clock_rate = (1. + error * 0.05).clamp(0.95, 1.05);
            }
        }
        self.stats.max_clock_slew = self.stats.max_clock_slew.max((self.clock_rate - 1.).abs());
        if !reset && self.clock.is_some_and(|(_, old)| estimate < old) {
            self.stats.clock_phase_backsteps += 1;
        }
        self.clock = Some((now, estimate));
        let target = estimate.floor() as u64;
        if target.saturating_sub(history.input().tick) > 16 || reset {
            // Bootstrap/resume to the estimated clock using held input, with no
            // historical commands fabricated for a newly sampled intent.
            history.resync_clock();
            self.recent.clear();
            self.stats.clock_resets += 1;
            reset = true;
            for _ in c.input.tick..target {
                if matches!(
                    history.predict(None, c.ready_sequence, solve)?,
                    Predicted::Suspended(_)
                ) {
                    break;
                }
            }
        }
        let report_sequence = c.ready_sequence;
        let mut produced = false;
        for _ in history.input().tick..target {
            match history.predict(Some(intent), report_sequence, solve)? {
                Predicted::Advanced(Some(command)) => {
                    self.stats.predicted_steps += 1;
                    produced = true;
                    self.recent.push_back(command);
                    while self.recent.len() > MAX_COMMANDS {
                        self.recent.pop_front();
                    }
                }
                Predicted::Suspended(_) => {
                    self.stats.waiting_frames += 1;
                    self.recent.clear();
                    reset = true;
                    break;
                }
                Predicted::Advanced(None) => unreachable!(),
            }
        }
        self.stats.max_history = self.stats.max_history.max(history.retained());
        let packet = produced.then(|| Commands {
            epoch: c.actor.epoch.value(),
            stream_epoch: c.input.stream_epoch,
            ready_sequence: report_sequence,
            commands: self.recent.iter().copied().collect(),
        });
        Ok(Some(Driven {
            pose: history.pose(),
            tick: history.input().tick,
            packet,
            reset,
        }))
    }
}
