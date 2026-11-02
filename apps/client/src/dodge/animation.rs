//! Local predicted presentation only. No pose moves the authority or capsule.
use bevy::prelude::*;
use std::{sync::Arc, time::Instant};
use theia_core::{identity::EntityKey, time::TickRate};
use theia_network::dodging::Snapshot;
pub const NAMES: [&str; 4] = ["DodgeForward", "DodgeBack", "DodgeLeft", "DodgeRight"];
#[derive(Resource)]
pub struct Policy(pub Arc<theia_network::dodging::Profile>);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Context {
    pub actor: EntityKey,
    pub stream: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
    pub context: Context,
    pub first_start: u64,
}
#[derive(Clone, Copy)]
pub struct Cue {
    pub key: Key,
    pub slot: usize,
    pub elapsed: f32,
    pub duration: f32,
    heading: Quat,
}
#[derive(Resource, Default)]
pub struct State {
    pub available: bool,
    pub current: Option<Cue>,
    context: Option<Context>,
    seen_start: Option<u64>,
    consumed: u64,
    heading: Option<Quat>,
    cue: Option<Cue>,
    pub starts: [u64; 4],
    pub resumes: u64,
    pub maximum_pose_changes: [usize; 4],
}
fn slot(axes: [i16; 2], heading: Quat) -> usize {
    let local = (heading.conjugate() * Vec3::new(f32::from(axes[0]), 0., f32::from(axes[1]))).xz();
    if local.y.abs() >= local.x.abs() {
        usize::from(local.y >= 0.)
    } else if local.x < 0. {
        2
    } else {
        3
    }
}
impl State {
    pub fn clear(&mut self) {
        self.current = None;
        self.heading = None;
    }
    pub fn advance(
        &mut self,
        context: Context,
        snapshot: Snapshot,
        rate: TickRate,
        duration: f32,
        heading: Quat,
        moving: bool,
    ) {
        if self.context != Some(context) {
            self.clear();
            self.context = Some(context);
            self.seen_start = None;
            self.consumed = 0;
            self.cue = None;
        }
        let previous = self.current.take();
        if let Some(active) = snapshot.policy.active {
            let elapsed = (snapshot.policy.tick.saturating_sub(active.started_at) as f32
                / rate.hz() as f32)
                .min(duration);
            let window = (duration * rate.hz() as f32).ceil() as u64;
            let same = self.seen_start.is_some_and(|old| {
                active.started_at == old
                    || (active.started_at > old
                        && active.started_at - old <= window
                        && snapshot.last_nonce <= self.consumed)
            });
            if self.seen_start.is_none_or(|old| active.started_at >= old) {
                let mut cue = if same {
                    self.cue.unwrap()
                } else {
                    let slot = slot(active.axes, heading);
                    self.starts[slot] += 1;
                    self.heading = Some(heading);
                    Cue {
                        key: Key {
                            context,
                            first_start: active.started_at,
                        },
                        slot,
                        elapsed: 0.,
                        duration,
                        heading,
                    }
                };
                if same && previous.is_none() {
                    self.resumes += 1;
                }
                cue.elapsed = cue.elapsed.max(elapsed);
                self.heading = Some(cue.heading);
                self.current = Some(cue);
                self.cue = Some(cue);
                self.seen_start = Some(active.started_at);
            }
        } else if moving {
            self.heading = None;
        }
        self.consumed = self.consumed.max(snapshot.last_nonce);
    }
}
#[allow(clippy::too_many_arguments)]
pub fn present(
    mut animation: ResMut<State>,
    policy: Res<Policy>,
    network: Res<crate::network::Network>,
    intent: Res<crate::movement::MoveIntent>,
    mut pawn: Query<&mut Transform, With<crate::movement::LocalPawn>>,
) {
    let now = Instant::now();
    let state = network.state();
    let allowed = animation.available
        && network.connected()
        && network.can_act()
        && !network.prediction.as_ref().is_some_and(|d| d.life_held())
        && state
            .correction_received
            .is_some_and(|at| now.saturating_duration_since(at).as_secs_f64() <= 0.5);
    let sample = state.correction.and_then(|c| {
        network
            .prediction
            .as_ref()?
            .pose()?
            .dodge
            .map(|snapshot| (c.state, snapshot))
    });
    let Ok(mut pawn) = pawn.single_mut() else {
        return;
    };
    if let Some((correction, snapshot)) = sample.filter(|_| allowed) {
        animation.advance(
            Context {
                actor: correction.actor,
                stream: correction.input.stream_epoch,
            },
            snapshot,
            correction.rate,
            policy.0.policy().duration().as_secs_f32(),
            pawn.rotation,
            intent.direction.length_squared() > 0.0001,
        );
        if let Some(heading) = animation.heading {
            pawn.rotation = heading;
        }
    } else {
        animation.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        dodge::{Active, Snapshot as CoreSnapshot},
        identity::{EntityId, WorldEpoch},
    };
    fn context(stream: u64) -> Context {
        Context {
            actor: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(1).unwrap(),
            },
            stream,
        }
    }
    fn sample(start: u64, tick: u64, nonce: u64, axes: [i16; 2]) -> Snapshot {
        Snapshot {
            last_nonce: nonce,
            policy: CoreSnapshot {
                tick,
                ready_at: start + 90,
                active: Some(Active {
                    started_at: start,
                    axes,
                }),
            },
        }
    }
    #[test]
    fn directional_selection_uses_pre_dodge_facing_including_rotated_camera_travel() {
        assert_eq!(slot([0, -1000], Quat::IDENTITY), 0);
        assert_eq!(slot([0, 1000], Quat::IDENTITY), 1);
        assert_eq!(slot([-1000, 0], Quat::IDENTITY), 2);
        assert_eq!(slot([1000, 0], Quat::IDENTITY), 3);
        assert_eq!(
            slot(
                [-1000, 0],
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            ),
            0
        );
    }
    #[test]
    fn refused_nonce_during_active_motion_cannot_restart_or_redirect_the_clip() {
        let mut s = State::default();
        s.advance(
            context(1),
            sample(10, 10, 1, [0, -1000]),
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        let key = s.current.unwrap().key;
        s.advance(
            context(1),
            sample(10, 11, 2, [0, -1000]),
            TickRate::Hz30,
            0.25,
            Quat::from_rotation_y(1.),
            false,
        );
        assert_eq!(s.current.unwrap().key, key);
        assert_eq!(s.starts, [1, 0, 0, 0]);
        assert_eq!(s.heading, Some(Quat::IDENTITY));
    }
    #[test]
    fn delayed_start_correction_resumes_one_visual_action_without_rewinding_phase() {
        let mut s = State::default();
        s.advance(
            context(1),
            sample(10, 11, 1, [0, 1000]),
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        let before = s.current.unwrap();
        s.clear();
        s.advance(
            context(1),
            sample(11, 11, 1, [0, 1000]),
            TickRate::Hz30,
            0.25,
            Quat::from_rotation_y(1.),
            false,
        );
        let after = s.current.unwrap();
        assert_eq!(after.key, before.key);
        assert_eq!(after.elapsed, before.elapsed);
        assert_eq!(s.starts, [0, 1, 0, 0]);
        assert_eq!(s.resumes, 1);
        assert_eq!(s.heading, Some(Quat::IDENTITY));
    }
    #[test]
    fn fresh_stream_cannot_inherit_old_cue_and_older_motion_cannot_replay() {
        let mut s = State::default();
        s.advance(
            context(1),
            sample(100, 100, 2, [1000, 0]),
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        s.advance(
            context(1),
            sample(10, 11, 1, [0, -1000]),
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        assert!(s.current.is_none());
        assert_eq!(s.starts, [0, 0, 0, 1]);
        s.advance(
            context(2),
            sample(10, 10, 1, [0, -1000]),
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        assert_eq!(s.current.unwrap().key.context, context(2));
        assert_eq!(s.starts, [1, 0, 0, 1]);
    }
    #[test]
    fn ordinary_travel_releases_the_heading_after_dodge_but_stationary_recovery_keeps_it() {
        let mut s = State::default();
        let mut snapshot = sample(10, 10, 1, [0, 1000]);
        s.advance(
            context(1),
            snapshot,
            TickRate::Hz30,
            0.25,
            Quat::IDENTITY,
            false,
        );
        snapshot.policy.active = None;
        snapshot.policy.tick = 18;
        s.advance(
            context(1),
            snapshot,
            TickRate::Hz30,
            0.25,
            Quat::from_rotation_y(1.),
            false,
        );
        assert!(s.current.is_none());
        assert_eq!(s.heading, Some(Quat::IDENTITY));
        s.advance(
            context(1),
            snapshot,
            TickRate::Hz30,
            0.25,
            Quat::from_rotation_y(1.),
            true,
        );
        assert!(s.heading.is_none());
    }
    #[test]
    fn fresh_action_after_cooldown_counts_once_and_phase_stays_inside_policy_duration() {
        let mut s = State::default();
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            s.advance(
                context(u64::from(rate.hz())),
                sample(10, 10, 1, [0, -1000]),
                rate,
                0.25,
                Quat::IDENTITY,
                false,
            );
            s.advance(
                context(u64::from(rate.hz())),
                sample(200, 201, 2, [1000, 0]),
                rate,
                0.25,
                Quat::IDENTITY,
                false,
            );
            assert!(s.current.unwrap().elapsed > 0. && s.current.unwrap().elapsed <= 0.25);
            assert_eq!(s.current.unwrap().slot, 3);
        }
        assert_eq!(s.starts, [3, 0, 0, 3]);
    }
}
