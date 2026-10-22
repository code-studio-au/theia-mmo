//! P2-04 core qualification using real Avian terrain, with simulated delivery
//! schedules. These are not native rendered or real-UDP prediction tests.
use theia_core::{
    coordinates::ChunkId,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    commands::{Command, State, Timeline},
    kinematic::Motion,
    prediction::{self, Correction, History, MAX_HISTORY, Pose, Predicted, Reconciled, Reset},
    procedural::{self, Patch},
    wire::Intent,
};
fn anchor(patch: &Patch, rate: TickRate) -> Correction {
    Correction {
        world: procedural::IDENTITY,
        actor: EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(1).unwrap(),
        },
        rate,
        input: Timeline::new(1, rate, 0).unwrap().state(),
        pose: Pose {
            feet: patch.spawn(8062., 8000.),
            motion: Motion::default().snapshot(),
        },
        ready_sequence: 1,
    }
}
fn moving(tick: u64) -> Intent {
    Intent {
        axes: if tick <= 140 { [1000, 0] } else { [0, 0] },
        sprint: true,
        jump: (20..=60).contains(&tick) || (100..=105).contains(&tick),
    }
}
fn unchanged(pose: Pose, _: Intent, _: TickRate) -> Result<Option<Pose>, &'static str> {
    Ok(Some(pose))
}
fn distance(a: Pose, b: Pose) -> f64 {
    a.feet
        .metres()
        .into_iter()
        .zip(b.feet.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
#[test]
fn immediate_prediction_reconciles_delayed_lost_reordered_commands_across_a_real_terrain_seam() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let initial = anchor(&patch, rate);
        let solve = |pose, intent, rate| {
            prediction::terrain_step(&patch, &patch.ids, pose, intent, rate, 1.)
        };
        let mut client = History::new(initial).unwrap();
        let mut authority = Timeline::restore(rate, initial.input).unwrap();
        let mut authoritative_pose = initial.pose;
        let mut commands: Vec<(u64, Command)> = Vec::new();
        let mut corrections: Vec<(u64, Correction)> = Vec::new();
        let lead = 4;
        for _ in 0..lead {
            let tick = client.input().tick + 1;
            let Predicted::Advanced(Some(command)) =
                client.predict(Some(moving(tick)), 1, solve).unwrap()
            else {
                panic!("Prediction did not advance");
            };
            commands.push((command.sequence % 3, command));
        }
        // No server step or server pose was needed for the first local response.
        assert!(client.pose().feet.metres()[0] > initial.pose.feet.metres()[0]);
        assert_eq!(authority.state().tick, 0);
        let mut max_correction = 0f64;
        let mut stale = 0;
        let mut replayed = 0;
        let mut airborne = false;
        let mut landed = false;
        for tick in 1..=220 {
            let mut due = Vec::new();
            commands.retain(|&(delivery, command)| {
                if delivery < tick {
                    due.push(command);
                    false
                } else {
                    true
                }
            });
            // Missing commands hold/expire; reversed duplicates cannot create steps.
            for command in due.into_iter().rev() {
                if command.sequence % 7 != 0 {
                    let _ = authority.offer(1, command);
                    let _ = authority.offer(1, command);
                }
            }
            let applied = authority.advance().unwrap();
            authoritative_pose = solve(authoritative_pose, applied.intent, rate)
                .unwrap()
                .unwrap();
            if (21..=60).contains(&tick) && !authoritative_pose.motion.grounded {
                airborne = true;
            }
            if tick > 70 && authoritative_pose.motion.grounded {
                landed = true;
            }
            let truth = Correction {
                input: applied,
                pose: authoritative_pose,
                ..initial
            };
            if tick % 11 != 0 {
                corrections.push((tick + 2 + tick % 5, truth));
            }
            let mut due = Vec::new();
            corrections.retain(|&(delivery, correction)| {
                if delivery <= tick {
                    due.push(correction);
                    false
                } else {
                    true
                }
            });
            for correction in due.into_iter().rev() {
                match client.reconcile(correction, solve).unwrap() {
                    Reconciled::Stale => stale += 1,
                    Reconciled::Applied {
                        replayed: count,
                        correction_metres,
                    } => {
                        replayed += count;
                        max_correction = max_correction.max(correction_metres);
                    }
                    other => panic!("Unexpected reset: {other:?}"),
                }
            }
            let next = client.input().tick + 1;
            let Predicted::Advanced(Some(command)) =
                client.predict(Some(moving(next)), 1, solve).unwrap()
            else {
                panic!("Prediction stopped");
            };
            commands.push((tick + command.sequence % 3, command));
            assert!(client.retained() <= 12);
            assert!(commands.len() <= 16 && corrections.len() <= 16);
        }
        assert!(airborne && landed);
        assert!(replayed > 500 && stale > 0);
        assert!(
            max_correction > 0.01,
            "Loss must actually exercise correction"
        );
        assert!(
            authoritative_pose.feet.metres()[0] > 8064.,
            "Must cross chunk seam"
        );
        let truth = Correction {
            input: authority.state(),
            pose: authoritative_pose,
            ..initial
        };
        client.reconcile(truth, solve).unwrap();
        assert!(distance(client.pose(), authoritative_pose) < 0.003);
        assert_eq!(client.pose().motion, authoritative_pose.motion);
        assert!(client.suspended().is_none());
        println!(
            "{rate:?}: max correction {max_correction:.6}m, replayed {replayed}, stale {stale}, retained {}",
            client.retained()
        );
    }
}
#[test]
fn overflow_suspends_and_resumes_only_from_new_truth_without_reusing_sequences() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let mut client = History::new(initial).unwrap();
    for _ in 0..MAX_HISTORY {
        assert!(matches!(
            client.predict(Some(moving(1)), 1, unchanged).unwrap(),
            Predicted::Advanced(Some(_))
        ));
    }
    assert_eq!(client.retained(), MAX_HISTORY);
    assert_eq!(
        client.predict(Some(moving(1)), 1, unchanged).unwrap(),
        Predicted::Suspended(Reset::HistoryFull)
    );
    assert_eq!(client.retained(), 0);
    assert_eq!(client.pose(), initial.pose);
    assert_eq!(
        client.reconcile(initial, unchanged).unwrap(),
        Reconciled::Stale
    );
    assert_eq!(client.suspended(), Some(Reset::HistoryFull));
    let truth = Correction {
        input: State {
            tick: 10,
            ..initial.input
        },
        ..initial
    };
    client.reconcile(truth, unchanged).unwrap();
    let Predicted::Advanced(Some(command)) = client.predict(Some(moving(1)), 1, unchanged).unwrap()
    else {
        panic!()
    };
    assert_eq!(command.sequence, MAX_HISTORY as u64 + 1);
    assert_eq!(command.tick, 11);
}
#[test]
fn teleports_reset_the_movement_stream_and_foreign_or_old_corrections_cannot_poison_it() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let mut client = History::new(initial).unwrap();
    for _ in 0..8 {
        client.predict(Some(moving(1)), 1, unchanged).unwrap();
    }
    let remote = Patch::build(ChunkId { x: 123, z: 1 });
    let teleport = Correction {
        pose: Pose {
            feet: remote.spawn(15800., 180.),
            ..initial.pose
        },
        input: State {
            stream_epoch: 2,
            tick: 5,
            fresh_tick: 5,
            ..initial.input
        },
        ..initial
    };
    assert_eq!(
        client.reconcile(teleport, unchanged).unwrap(),
        Reconciled::Reset(Reset::StreamChanged)
    );
    assert_eq!(client.retained(), 0);
    assert_eq!(client.pose(), teleport.pose);
    let old = Correction {
        input: State {
            tick: 100,
            ..initial.input
        },
        ..initial
    };
    assert_eq!(client.reconcile(old, unchanged).unwrap(), Reconciled::Stale);
    let foreign = Correction {
        world: [255; 32],
        ..teleport
    };
    assert!(client.reconcile(foreign, unchanged).is_err());
    let foreign = Correction {
        actor: EntityKey {
            entity: EntityId::new(2).unwrap(),
            ..initial.actor
        },
        ..teleport
    };
    assert!(client.reconcile(foreign, unchanged).is_err());
    let Predicted::Advanced(Some(command)) = client.predict(Some(moving(1)), 1, unchanged).unwrap()
    else {
        panic!()
    };
    assert_eq!((command.sequence, command.tick), (1, 6));
    assert_eq!(client.input().stream_epoch, 2);
    assert_eq!(client.retained(), 1);
}
#[test]
fn missing_collision_and_replay_solver_failure_restore_latest_truth_without_partial_history() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let mut client = History::new(initial).unwrap();
    for _ in 0..8 {
        client.predict(Some(moving(1)), 1, unchanged).unwrap();
    }
    let truth = Correction {
        input: State {
            tick: 3,
            ..initial.input
        },
        ..initial
    };
    let mut attempts = 0;
    let result = client
        .reconcile(truth, |pose, _, _| {
            attempts += 1;
            if attempts == 3 {
                Err("Injected unavailable query scene")
            } else {
                Ok(Some(pose))
            }
        })
        .unwrap();
    assert_eq!(result, Reconciled::Reset(Reset::SolverFailed));
    assert_eq!(attempts, 3);
    assert_eq!(client.pose(), truth.pose);
    assert_eq!(client.input(), truth.input);
    assert_eq!(client.retained(), 0);
    let truth = Correction {
        input: State {
            tick: 4,
            ..initial.input
        },
        ..initial
    };
    client.reconcile(truth, unchanged).unwrap();
    assert_eq!(
        client
            .predict(Some(moving(1)), 1, |_, _, _| Ok(None))
            .unwrap(),
        Predicted::Suspended(Reset::CollisionUnavailable)
    );
    assert_eq!(client.retained(), 0);
    assert_eq!(client.pose(), truth.pose);
}
#[test]
fn readiness_withdrawal_discards_old_replay_but_still_allows_airborne_gravity_and_landing() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let solve =
        |pose, intent, rate| prediction::terrain_step(&patch, &patch.ids, pose, intent, rate, 1.);
    let mut client = History::new(initial).unwrap();
    let jump = Intent {
        axes: [1000, 0],
        jump: true,
        sprint: false,
    };
    for _ in 0..6 {
        client.predict(Some(jump), 1, solve).unwrap();
    }
    let in_flight = client.pose();
    assert!(!in_flight.motion.grounded);
    assert_eq!(
        client.predict(Some(jump), 2, solve).unwrap(),
        Predicted::Suspended(Reset::ReadinessChanged)
    );
    let truth = Correction {
        pose: in_flight,
        input: State {
            tick: 6,
            fresh_tick: 6,
            applied_sequence: 6,
            intent: jump,
            ..initial.input
        },
        ready_sequence: 2,
        ..initial
    };
    assert_eq!(
        client.reconcile(truth, solve).unwrap(),
        Reconciled::Reset(Reset::ReadinessChanged)
    );
    let empty = Default::default();
    let solve =
        |pose, intent, rate| prediction::terrain_step(&patch, &empty, pose, intent, rate, 1.);
    for _ in 0..40 {
        client.predict(Some(jump), 2, solve).unwrap();
    }
    let landed = client.pose();
    assert_eq!(landed.feet.metres()[0], in_flight.feet.metres()[0]);
    assert_eq!(landed.feet.metres()[2], in_flight.feet.metres()[2]);
    assert!(landed.motion.grounded && landed.motion.jump_held);
    assert!(landed.feet.metres()[1] < in_flight.feet.metres()[1]);
}
#[test]
fn invalid_corrections_inputs_and_exhausted_clocks_cannot_corrupt_history() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let mut client = History::new(initial).unwrap();
    let invalid_intent = Intent {
        axes: [1001, 0],
        ..Intent::default()
    };
    assert!(client.predict(Some(invalid_intent), 1, unchanged).is_err());
    assert_eq!(client.input(), initial.input);
    let invalid = Correction {
        pose: Pose {
            motion: theia_network::kinematic::MotionState {
                vertical_speed: f32::NAN,
                ..initial.pose.motion
            },
            ..initial.pose
        },
        ..initial
    };
    assert!(client.reconcile(invalid, unchanged).is_err());
    assert_eq!(client.pose(), initial.pose);
    for input in [
        State {
            tick: u64::MAX,
            fresh_tick: u64::MAX,
            ..initial.input
        },
        State {
            applied_sequence: u64::MAX,
            ..initial.input
        },
    ] {
        let mut client = History::new(Correction { input, ..initial }).unwrap();
        assert_eq!(
            client.predict(Some(moving(1)), 1, unchanged).unwrap(),
            Predicted::Suspended(Reset::ClockExhausted)
        );
        assert_eq!(client.retained(), 0);
    }
}

#[test]
fn completed_sequences_and_readiness_cannot_regress_within_a_movement_stream() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let initial = anchor(&patch, TickRate::Hz30);
    let initial = Correction {
        input: State {
            tick: 10,
            fresh_tick: 10,
            applied_sequence: 7,
            ..initial.input
        },
        ready_sequence: 9,
        ..initial
    };
    let mut client = History::new(initial).unwrap();
    for correction in [
        Correction {
            input: State {
                tick: 11,
                applied_sequence: 6,
                ..initial.input
            },
            ..initial
        },
        Correction {
            input: State {
                tick: 11,
                ..initial.input
            },
            ready_sequence: 8,
            ..initial
        },
        Correction {
            input: State {
                tick: 11,
                applied_sequence: 6,
                ..initial.input
            },
            ready_sequence: 10,
            ..initial
        },
        Correction {
            rate: TickRate::Hz60,
            ..initial
        },
    ] {
        assert!(client.reconcile(correction, unchanged).is_err());
        assert_eq!(client.input(), initial.input);
        assert_eq!(client.pose(), initial.pose);
        assert_eq!(client.retained(), 0);
    }
}

#[test]
fn new_stream_bootstrap_preserves_jump_latch_until_an_actual_release_command() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let mut initial = anchor(&patch, TickRate::Hz30);
    initial.pose.motion.jump_held = true;
    let mut history = History::new(initial).unwrap();
    let solve =
        |pose, intent, rate| prediction::terrain_step(&patch, &patch.ids, pose, intent, rate, 1.);
    for _ in 0..20 {
        history.predict(None, 1, solve).unwrap();
        assert!(history.pose().motion.jump_held && history.pose().motion.grounded);
    }
    history
        .predict(
            Some(Intent {
                jump: true,
                ..Intent::default()
            }),
            1,
            solve,
        )
        .unwrap();
    assert!(
        history.pose().motion.grounded,
        "Held jump retriggered across bootstrap"
    );
    history.predict(Some(Intent::default()), 1, solve).unwrap();
    assert!(!history.pose().motion.jump_held);
    history
        .predict(
            Some(Intent {
                jump: true,
                ..Intent::default()
            }),
            1,
            solve,
        )
        .unwrap();
    assert!(!history.pose().motion.grounded && history.pose().motion.vertical_speed > 0.);
}
