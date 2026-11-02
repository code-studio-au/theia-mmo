//! Opt-in fixed-step authority. Transport selects the matching versioned movement profile.
use super::World;
use std::sync::Arc;

impl World {
    /// Bind before admission; default worlds and legacy movement stay unchanged.
    /// Renet selects TC02/TP02 only when this immutable profile is enabled.
    pub fn enable_dodge(&mut self, profile: Arc<crate::dodging::Profile>) -> crate::Result<()> {
        if !self.predicted
            || self.tick != 0
            || !self.players.is_empty()
            || self.dodge.is_some()
            || self.classic
            || self.dev_tour
            || self.terrain_teleport.is_some()
            || self.procedural.as_ref().is_none_or(|r| r.speed != 1.)
        {
            return Err("Dodge requires inactive normal-speed predicted terrain".into());
        }
        self.dodge = Some(profile);
        Ok(())
    }
    pub fn dodge_profile(&self) -> Option<&Arc<crate::dodging::Profile>> {
        self.dodge.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commands::Command,
        dodging::Request,
        prediction_wire::Commands,
        wire::{ChunkReady, Intent, VERSION},
    };
    use theia_core::{coordinates::ChunkId, time::TickRate};
    fn profile() -> Arc<crate::dodging::Profile> {
        Arc::new(
            crate::dodging::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-dodge.json"
            ))
            .unwrap(),
        )
    }
    fn world(rate: TickRate) -> (World, u64) {
        let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(w.set_tick_rate(rate));
        w.enable_prediction().unwrap();
        w.enable_dodge(profile()).unwrap();
        let id = w.join().unwrap();
        ready(&mut w, id, 1, true);
        w.step();
        (w, id)
    }
    fn ready(w: &mut World, id: u64, sequence: u64, enabled: bool) {
        let chunks = if enabled {
            crate::procedural::neighbours(ChunkId { x: 62, z: 62 }, 1)
                .into_iter()
                .map(|c| [c.x, c.z])
                .collect()
        } else {
            vec![]
        };
        assert!(w.client_ready(
            id,
            &ChunkReady {
                version: VERSION,
                epoch: w.epoch(),
                sequence,
                chunks
            }
        ));
    }
    fn send(w: &mut World, id: u64, seq: u64, nonce: u64) -> Commands {
        let state = w.correction(id).unwrap().state;
        let packet = Commands {
            epoch: w.epoch(),
            stream_epoch: state.input.stream_epoch,
            ready_sequence: state.ready_sequence,
            commands: vec![Command {
                sequence: seq,
                tick: state.input.tick + 1,
                intent: Intent {
                    facing: None,
                    dodge: Some(Request {
                        nonce,
                        axes: [1000, 0],
                    }),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(w.command(id, &packet), 1);
        packet
    }
    #[test]
    fn activation_is_opt_in_normal_speed_and_binds_exact_profile_identity() {
        let mut w = World::default();
        let before = w.identity();
        assert!(w.enable_dodge(profile()).is_err());
        assert_eq!(w.identity(), before);
        let mut w = World::with_procedural([8000., 8000.], 8.).unwrap();
        w.enable_prediction().unwrap();
        assert!(w.enable_dodge(profile()).is_err());
        let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
        w.enable_prediction().unwrap();
        let before = w.identity();
        let p = profile();
        w.enable_dodge(p.clone()).unwrap();
        assert_eq!(w.identity(), crate::dodging::profile_identity(before, &p));
        assert!(w.enable_dodge(p.clone()).is_err());
        w.join().unwrap();
        assert!(w.enable_dodge(p).is_err());
    }
    #[test]
    fn actual_authority_and_bound_prediction_match_every_tick_at_all_rates() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut w, id) = world(rate);
            let initial = w.correction(id).unwrap().state;
            let binding = crate::dodge_prediction::Binding::from_correction(
                crate::procedural::IDENTITY,
                initial,
                profile(),
            )
            .unwrap();
            let patch = w
                .procedural
                .as_ref()
                .unwrap()
                .patch(w.global_position(id).unwrap())
                .unwrap()
                .clone();
            let ready = w.players[&id].ready.clone().unwrap();
            let mut pose = initial.pose;
            for n in 1..=rate.hz() * 4 {
                send(&mut w, id, u64::from(n), 1);
                w.step();
                let c = w.correction(id).unwrap().state;
                pose = binding
                    .terrain_step_at(
                        &patch,
                        &ready,
                        pose,
                        c.input.effective_intent(pose.motion),
                        rate,
                        c.input.tick,
                        true,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(pose.dodge, c.pose.dodge);
                assert_eq!(pose.motion, c.pose.motion);
                for (a, b) in pose.feet.metres().into_iter().zip(c.pose.feet.metres()) {
                    assert!((a - b).abs() < 1e-5, "{rate:?}: {a} {b}");
                }
                assert!(
                    c.pose
                        .dodge
                        .unwrap()
                        .valid_at(&profile(), rate, c.input.tick)
                );
            }
            let end = w.correction(id).unwrap().state.pose;
            assert!((end.feet.metres()[0] - initial.pose.feet.metres()[0] - 3.).abs() < 0.002);
            assert_eq!(end.dodge.unwrap().last_nonce, 1);
            assert!(end.dodge.unwrap().policy.active.is_none());
        }
    }
    #[test]
    fn readiness_loss_cancels_and_consumes_edges_without_delayed_distance() {
        let (mut w, id) = world(TickRate::Hz30);
        send(&mut w, id, 1, 1);
        w.step();
        let started = w.correction(id).unwrap().state.pose.dodge.unwrap();
        ready(&mut w, id, 2, false);
        send(&mut w, id, 2, 2);
        w.step();
        let held = w.correction(id).unwrap().state.pose;
        let state = held.dodge.unwrap();
        assert!(state.policy.active.is_none());
        assert_eq!(state.policy.ready_at, started.policy.ready_at);
        assert_eq!(state.last_nonce, 2);
        ready(&mut w, id, 3, true);
        for n in 3..=110 {
            send(&mut w, id, n, 2);
            w.step();
        }
        assert_eq!(
            w.correction(id).unwrap().state.pose.feet.metres()[0],
            held.feet.metres()[0]
        );
        send(&mut w, id, 111, 3);
        w.step();
        assert!(
            w.correction(id)
                .unwrap()
                .state
                .pose
                .dodge
                .unwrap()
                .policy
                .active
                .is_some()
        );
    }
    #[test]
    fn missing_prepared_patch_advances_policy_clock_and_never_catches_up() {
        let (mut w, id) = world(TickRate::Hz30);
        send(&mut w, id, 1, 1);
        w.step();
        let before = w.correction(id).unwrap().state.pose;
        let patches = std::mem::take(&mut w.procedural.as_mut().unwrap().patches);
        for _ in 0..6 {
            w.step();
            let c = w.correction(id).unwrap().state;
            assert!(
                c.pose
                    .dodge
                    .unwrap()
                    .valid_at(&profile(), c.rate, c.input.tick)
            );
            assert!(c.pose.dodge.unwrap().policy.active.is_none());
            assert_eq!(c.pose.feet, before.feet);
        }
        w.procedural.as_mut().unwrap().patches = patches;
        w.step();
        assert_eq!(
            w.correction(id).unwrap().state.pose.feet.metres()[0],
            before.feet.metres()[0]
        );
    }
    #[test]
    fn teleport_cancels_active_motion_preserves_nonce_cooldown_and_rejects_old_stream() {
        let (mut w, id) = world(TickRate::Hz30);
        let packet = send(&mut w, id, 1, 1);
        w.step();
        let before = w.correction(id).unwrap().state;
        assert!(w.teleport_development(id, [8004., 8000.]));
        w.poll_terrain();
        let after = w.correction(id).unwrap().state;
        assert_eq!(after.input.stream_epoch, before.input.stream_epoch + 1);
        let d = after.pose.dodge.unwrap();
        assert!(d.policy.active.is_none());
        assert_eq!(d.last_nonce, 1);
        assert_eq!(
            d.policy.ready_at,
            before.pose.dodge.unwrap().policy.ready_at
        );
        assert_eq!(w.command(id, &packet), 0);
        ready(&mut w, id, 2, true);
        send(&mut w, id, 1, 1);
        w.step();
        assert!(
            w.correction(id)
                .unwrap()
                .state
                .pose
                .dodge
                .unwrap()
                .policy
                .active
                .is_none()
        );
    }
    #[test]
    fn legacy_world_rejects_dodge_payload_without_changing_completed_input() {
        let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
        w.enable_prediction().unwrap();
        let id = w.join().unwrap();
        let before = w.correction(id).unwrap();
        let packet = Commands {
            epoch: w.epoch(),
            stream_epoch: 1,
            ready_sequence: 0,
            commands: vec![Command {
                sequence: 1,
                tick: 1,
                intent: Intent {
                    facing: None,
                    dodge: Some(Request {
                        nonce: 1,
                        axes: [1000, 0],
                    }),
                    ..Default::default()
                },
            }],
        };
        assert_eq!(w.command(id, &packet), 0);
        assert_eq!(w.correction(id).unwrap(), before);
    }
}
