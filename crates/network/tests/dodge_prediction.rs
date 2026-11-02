//! Prepared real-Avian terrain and simulated correction delivery. No live
//! World/driver/transport selects dodge yet; these are shared replay checks.
#[path = "support/saved_fixture.rs"]
#[allow(dead_code)] // This harness uses the shared fixture without corruption helpers.
mod saved_fixture;
use std::{
    collections::{BTreeSet, VecDeque},
    sync::Arc,
};
use theia_core::{
    coordinates::ChunkId,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    commands::Timeline,
    dodge_prediction::Binding,
    dodging::{self, Controller, Profile, Request},
    kinematic::{self, MotionState},
    prediction::{Correction, History, Pose, Predicted, Reconciled, Reset},
    procedural::{self, Patch, Source},
    wire::Intent,
};
fn profile() -> Arc<Profile> {
    Arc::new(
        Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-dodge.json"
        ))
        .unwrap(),
    )
}
fn anchor(patch: &Patch, rate: TickRate, source: [u8; 32], profile: &Profile) -> Correction {
    Correction {
        world: dodging::profile_identity(source, profile),
        actor: EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(1).unwrap(),
        },
        rate,
        input: Timeline::new(1, rate, 0).unwrap().state(),
        ready_sequence: 1,
        pose: Pose {
            facing: None,
            feet: patch.spawn(8062., 8000.),
            motion: MotionState {
                vertical_speed: 0.,
                grounded: true,
                jump_held: false,
            },
            dodge: Some(Controller::new(profile, rate, 0).snapshot()),
        },
    }
}
fn edge(nonce: u64) -> Intent {
    Intent {
        facing: None,
        dodge: Some(Request {
            nonce,
            axes: [1000, 0],
        }),
        ..Default::default()
    }
}
fn step(
    binding: &Binding,
    patch: &Patch,
    ready: &BTreeSet<ChunkId>,
    pose: Pose,
    intent: Intent,
    rate: TickRate,
    tick: u64,
) -> Pose {
    binding
        .terrain_step_at(patch, ready, pose, intent, rate, tick, true)
        .unwrap()
        .unwrap()
}
#[test]
fn lost_first_edge_and_delayed_corrections_replay_without_restarting_distance_or_cooldown() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let initial = anchor(&patch, rate, procedural::IDENTITY, &p);
        let binding = Binding::from_correction(procedural::IDENTITY, initial, p.clone()).unwrap();
        let mut history = History::new(initial).unwrap();
        let mut authority = initial;
        let mut input = Timeline::restore(rate, initial.input).unwrap();
        let mut corrections = VecDeque::new();
        let mut mismatches = 0;
        let mut replayed = 0;
        for tick in 1..=u64::from(rate.hz()) * 4 {
            let intent = if (2..=7).contains(&tick) || tick > u64::from(rate.hz()) * 3 + 10 {
                edge(1)
            } else {
                Intent::default()
            };
            let Predicted::Advanced(Some(command)) = history
                .predict_at(Some(intent), 1, |pose, intent, rate, tick| {
                    binding.terrain_step_at(&patch, &patch.ids, pose, intent, rate, tick, true)
                })
                .unwrap()
            else {
                panic!("Expected speculative command");
            };
            if tick != 2 {
                input.offer(1, command).unwrap();
            }
            authority.input = input.advance().unwrap();
            authority.pose = step(
                &binding,
                &patch,
                &patch.ids,
                authority.pose,
                authority.input.effective_intent(authority.pose.motion),
                rate,
                tick,
            );
            if tick % 5 == 0 {
                corrections.push_back(authority);
            }
            if let Some(correction) =
                corrections.pop_front_if(|c: &mut Correction| c.input.tick + 3 <= tick)
            {
                let Reconciled::Applied {
                    same_tick: Some(error),
                    replayed: count,
                    ..
                } = history
                    .reconcile_at(correction, |pose, intent, rate, tick| {
                        binding.terrain_step_at(&patch, &patch.ids, pose, intent, rate, tick, true)
                    })
                    .unwrap()
                else {
                    panic!("Expected replay");
                };
                mismatches += usize::from(error.dodge_mismatch);
                replayed += count;
                assert_eq!(history.pose(), authority.pose, "{rate:?}/{tick}");
            }
        }
        assert!(mismatches >= 1 && replayed > 0);
        assert_eq!(history.pose(), authority.pose);
        assert!(
            (authority.pose.feet.metres()[0] - initial.pose.feet.metres()[0] - 3.).abs() < 1e-4
        );
        let snapshot = authority.pose.dodge.unwrap();
        assert_eq!(snapshot.last_nonce, 1);
        assert!(snapshot.policy.active.is_none());
        assert_eq!(snapshot.policy.ready_at, 3 + u64::from(rate.hz()) * 3);
    }
}
#[test]
fn tick_aware_history_preserves_completed_ticks_for_held_input_and_replay() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let mut initial = anchor(&patch, TickRate::Hz30, procedural::IDENTITY, &p);
    initial.pose.dodge = None;
    let mut history = History::new(initial).unwrap();
    let mut seen = Vec::new();
    for _ in 0..3 {
        history
            .predict_at(None, 1, |pose, _, _, tick| {
                seen.push(tick);
                Ok(Some(pose))
            })
            .unwrap();
    }
    assert_eq!(seen, [1, 2, 3]);
    seen.clear();
    let mut corrected = initial;
    corrected.input.tick = 1;
    history
        .reconcile_at(corrected, |pose, _, _, tick| {
            seen.push(tick);
            Ok(Some(pose))
        })
        .unwrap();
    assert_eq!(seen, [2, 3]);
}
#[test]
fn solver_cannot_return_a_dodge_snapshot_from_a_different_completed_tick() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let initial = anchor(&patch, TickRate::Hz30, procedural::IDENTITY, &p);
    let mut history = History::new(initial).unwrap();
    assert_eq!(
        history
            .predict_at(None, 1, |pose, _, _, _| Ok(Some(pose)))
            .unwrap(),
        Predicted::Suspended(Reset::SolverFailed)
    );
    assert_eq!(history.pose(), initial.pose);
    assert_eq!(history.retained(), 0);
}
#[test]
fn timed_state_cannot_be_erased_by_solver_or_correction_in_the_same_world() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let initial = anchor(&patch, TickRate::Hz30, procedural::IDENTITY, &p);
    let mut history = History::new(initial).unwrap();
    let mut correction = initial;
    correction.input.tick = 1;
    correction.pose.dodge = None;
    assert!(
        history
            .reconcile_at(correction, |pose, _, _, _| Ok(Some(pose)))
            .is_err()
    );
    assert_eq!(history.pose(), initial.pose);
    assert_eq!(
        history
            .predict_at(None, 1, |mut pose, _, _, _| {
                pose.dodge = None;
                Ok(Some(pose))
            })
            .unwrap(),
        Predicted::Suspended(Reset::SolverFailed)
    );
    assert_eq!(history.pose(), initial.pose);
}
#[test]
fn binding_rejects_foreign_world_changed_policy_and_missing_or_invalid_state() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let initial = anchor(&patch, TickRate::Hz30, procedural::IDENTITY, &p);
    assert!(Binding::from_correction([9; 32], initial, p.clone()).is_err());
    let other = Arc::new(
        Profile::from_json(
            &include_bytes!("../../../content/gameplay/development-dodge.json")
                .iter()
                .copied()
                .chain(*b" ")
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    );
    assert!(Binding::from_correction(procedural::IDENTITY, initial, other).is_err());
    let mut bad = initial;
    bad.pose.dodge = None;
    assert!(Binding::from_correction(procedural::IDENTITY, bad, p.clone()).is_err());
    bad = initial;
    bad.pose.dodge.as_mut().unwrap().policy.tick = 1;
    assert!(Binding::from_correction(procedural::IDENTITY, bad, p.clone()).is_err());
    let binding = Binding::from_correction(procedural::IDENTITY, initial, p).unwrap();
    assert!(
        binding
            .terrain_step_at(
                &patch,
                &patch.ids,
                initial.pose,
                edge(1),
                TickRate::Hz60,
                1,
                true
            )
            .is_err()
    );
    assert!(
        binding
            .terrain_step_at(
                &patch,
                &patch.ids,
                initial.pose,
                edge(1),
                TickRate::Hz30,
                2,
                true
            )
            .is_err()
    );
    assert_eq!(
        step(
            &binding,
            &patch,
            &patch.ids,
            initial.pose,
            edge(1),
            TickRate::Hz30,
            0
        ),
        initial.pose
    );
}
#[test]
fn dead_or_unprepared_owners_cancel_without_refund_or_late_edge_activation() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let initial = anchor(&patch, TickRate::Hz30, procedural::IDENTITY, &p);
    let binding = Binding::from_correction(procedural::IDENTITY, initial, p).unwrap();
    let started = step(
        &binding,
        &patch,
        &patch.ids,
        initial.pose,
        edge(1),
        initial.rate,
        1,
    );
    for (alive, ready) in [(false, patch.ids.clone()), (true, BTreeSet::new())] {
        let mut intent = edge(2);
        intent.axes = [1000, 0];
        intent.jump = true;
        intent.sprint = true;
        let retired = binding
            .terrain_step_at(&patch, &ready, started, intent, initial.rate, 2, alive)
            .unwrap()
            .unwrap();
        assert!((retired.feet.metres()[0] - started.feet.metres()[0]).abs() < 1e-4);
        let snapshot = retired.dodge.unwrap();
        assert_eq!(snapshot.last_nonce, 2);
        assert!(snapshot.policy.active.is_none());
        assert_eq!(
            snapshot.policy.ready_at,
            started.dodge.unwrap().policy.ready_at
        );
        let restored = step(
            &binding,
            &patch,
            &patch.ids,
            retired,
            edge(2),
            initial.rate,
            3,
        );
        assert!(restored.dodge.unwrap().policy.active.is_none());
    }
}
#[test]
fn unloaded_neighbour_stops_real_capsule_but_never_stores_displacement_debt() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let p = profile();
    let ready = patch.ids.iter().copied().filter(|id| id.x <= 62).collect();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let initial = anchor(&patch, rate, procedural::IDENTITY, &p);
        let binding = Binding::from_correction(procedural::IDENTITY, initial, p.clone()).unwrap();
        let mut pose = initial.pose;
        for tick in 1..=u64::from(rate.hz()) {
            pose = step(
                &binding,
                &patch,
                &ready,
                pose,
                if tick == 1 {
                    edge(1)
                } else {
                    Intent::default()
                },
                rate,
                tick,
            );
            kinematic::validate_pose(
                &patch.scene,
                patch.frame.to_local(pose.feet).unwrap(),
                pose.motion.grounded,
            )
            .unwrap();
        }
        let x = pose.feet.metres()[0];
        assert!((8063.5..8063.7).contains(&x));
        assert!(pose.dodge.unwrap().policy.active.is_none());
        pose = step(
            &binding,
            &patch,
            &patch.ids,
            pose,
            Intent::default(),
            rate,
            u64::from(rate.hz()) + 1,
        );
        assert!((pose.feet.metres()[0] - x).abs() < 1e-4);
    }
}
#[test]
fn different_prepared_local_frames_keep_global_dodge_continuous_across_the_seam() {
    let first = Patch::build(ChunkId { x: 62, z: 62 });
    let second = Patch::build(ChunkId { x: 63, z: 62 });
    let p = profile();
    let initial = anchor(&first, TickRate::Hz30, procedural::IDENTITY, &p);
    let binding = Binding::from_correction(procedural::IDENTITY, initial, p).unwrap();
    let mut a = initial.pose;
    let mut b = a;
    for tick in 1..=12 {
        let intent = if tick == 1 {
            edge(1)
        } else {
            Intent::default()
        };
        a = step(&binding, &first, &first.ids, a, intent, initial.rate, tick);
        b = step(
            &binding,
            &second,
            &second.ids,
            b,
            intent,
            initial.rate,
            tick,
        );
        assert!(
            a.feet
                .metres()
                .iter()
                .zip(b.feet.metres())
                .all(|(a, b)| (a - b).abs() < 1e-4)
        );
        assert_eq!(a.dodge, b.dodge);
    }
    assert!(a.feet.metres()[0] > 8064.);
}
#[test]
fn saved_prepared_terrain_needs_no_store_reads_and_missing_patch_suspends_history() {
    let fixture = saved_fixture::Fixture::new();
    let store = Arc::new(fixture.open());
    let patch = Patch::load(ChunkId { x: 62, z: 62 }, &Source::Saved(store.clone())).unwrap();
    let p = profile();
    let initial = anchor(&patch, TickRate::Hz30, store.identity(), &p);
    let binding = Binding::from_correction(store.identity(), initial, p).unwrap();
    let reads = store.stats().tile_reads;
    let mut pose = initial.pose;
    for tick in 1..=12 {
        pose = step(
            &binding,
            &patch,
            &patch.ids,
            pose,
            if tick == 1 {
                edge(1)
            } else {
                Intent::default()
            },
            initial.rate,
            tick,
        );
    }
    assert!(pose.feet.metres()[0] > 8064.);
    assert_eq!(store.stats().tile_reads, reads);
    let far = Patch::build(ChunkId { x: 72, z: 72 });
    let mut history = History::new(initial).unwrap();
    assert_eq!(
        history
            .predict_at(Some(edge(1)), 1, |pose, intent, rate, tick| binding
                .terrain_step_at(&far, &far.ids, pose, intent, rate, tick, true))
            .unwrap(),
        Predicted::Suspended(Reset::CollisionUnavailable)
    );
    assert_eq!(history.pose(), initial.pose);
}
