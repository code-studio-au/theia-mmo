use std::time::{Duration, Instant};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    client::{State, TransportStats},
    commands::Timeline,
    kinematic::Motion,
    prediction::{Correction, Pose},
    prediction_driver::Driver,
    prediction_wire::Corrected,
    procedural::{self, Patch},
    wire::{ChunkReady, Intent, VERSION},
};
fn state(patch: &Patch, now: Instant) -> State {
    let mut state = State {
        correction: Some(Corrected {
            speed: 1,
            state: Correction {
                world: procedural::IDENTITY,
                actor: EntityKey {
                    epoch: WorldEpoch::new(7).unwrap(),
                    entity: EntityId::new(1).unwrap(),
                },
                rate: TickRate::Hz30,
                input: Timeline::new(1, TickRate::Hz30, 0).unwrap().state(),
                pose: Pose {
                    facing: None,
                    feet: patch.spawn(8000., 8000.),
                    motion: Motion::default().snapshot(),
                    dodge: None,
                },
                ready_sequence: 1,
            },
        }),
        correction_received: Some(now),
        chunk_report: Some(ChunkReady {
            version: VERSION,
            epoch: 7,
            sequence: 1,
            chunks: patch.ids.iter().map(|id| [id.x, id.z]).collect(),
        }),
        ..State::default()
    };
    state.correction_ready = state.chunk_report.clone();
    state
}

#[test]
fn first_playable_terrain_acquires_latency_lead_before_sampling_new_commands() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    state.chunk_report.as_mut().unwrap().chunks.clear();
    state.correction_ready = state.chunk_report.clone();
    let mut driver = Driver::default();
    driver
        .tick(&state, Intent::default(), Some(&patch), now)
        .unwrap();
    // Geometry can be ready while RTT is still the unknown initial zero.
    state.chunk_report.as_mut().unwrap().chunks = patch.ids.iter().map(|id| [id.x, id.z]).collect();
    state.correction_ready = state.chunk_report.clone();
    driver
        .tick(
            &state,
            Intent::default(),
            Some(&patch),
            now + Duration::from_millis(50),
        )
        .unwrap();
    assert_eq!(
        driver.stats.clock_acquisitions, 0,
        "Do not lock in an unknown RTT"
    );
    // Acquire only after the real link RTT becomes known.
    state.correction.as_mut().unwrap().state.input.tick = 3;
    state.correction.as_mut().unwrap().state.ready_sequence = 2;
    state.correction_received = Some(now + Duration::from_millis(100));
    state.transport = Some(TransportStats {
        rtt_ms: 200.,
        ..TransportStats::default()
    });
    let report = state.chunk_report.as_mut().unwrap();
    report.sequence = 2;
    report.chunks = patch.ids.iter().map(|id| [id.x, id.z]).collect();
    state.correction_ready = state.chunk_report.clone();
    let intent = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    let bootstrap = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(100),
        )
        .unwrap()
        .unwrap();
    assert!(
        bootstrap.reset && bootstrap.packet.is_none(),
        "Acquisition must not fabricate historical input"
    );
    let next = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(140),
        )
        .unwrap()
        .unwrap();
    let packet = next.packet.unwrap();
    // At receipt the authority is three one-way ticks beyond this correction;
    // delivery plus 40 ms puts it at tick 10. Old warm-up commands arrived late.
    assert!(packet.commands.iter().all(|c| c.tick > 10 && c.tick <= 18));
    assert_eq!(driver.stats.clock_acquisitions, 1);
    assert_eq!(driver.stats.clock_phase_backsteps, 0);
}

#[test]
fn latency_samples_crossing_lead_thresholds_cannot_burst_or_pause_the_prediction_clock() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let now = Instant::now();
        let mut state = state(&patch, now);
        state.correction.as_mut().unwrap().state.rate = rate;
        state
            .correction
            .as_mut()
            .unwrap()
            .state
            .pose
            .motion
            .grounded = true;
        let mut driver = Driver::default();
        let mut server_tick = u64::MAX;
        let mut previous_tick = None;
        let mut previous_advance = None;
        let mut intervals = 0;
        // 800 FPS exposes sub-frame clock phase jumps; RTT straddles exactly
        // one lead-quantization threshold on every fresh server sample.
        for frame in 0..3200 {
            let elapsed = f64::from(frame) / 800.;
            let tick = (elapsed * f64::from(rate.hz())).floor() as u64;
            let sample_time = now + Duration::from_secs_f64(elapsed);
            if tick != server_tick {
                server_tick = tick;
                state.correction.as_mut().unwrap().state.input.tick = tick;
                state.correction_received = Some(sample_time);
                let half_rtt = 1. / f64::from(rate.hz())
                    + if tick.is_multiple_of(2) {
                        -0.0005
                    } else {
                        0.0005
                    };
                state.transport = Some(TransportStats {
                    rtt_ms: half_rtt * 2000.,
                    ..TransportStats::default()
                });
            }
            let driven = driver
                .tick(&state, Intent::default(), Some(&patch), sample_time)
                .unwrap()
                .unwrap();
            if let Some(previous) = previous_tick {
                assert!(driven.tick >= previous, "Prediction tick rewound");
                if driven.tick > previous && !driven.reset {
                    assert_eq!(
                        driven.tick - previous,
                        1,
                        "RTT rounding caused a multi-step burst"
                    );
                    if let Some(last) = previous_advance {
                        let interval = elapsed - last;
                        assert!(
                            interval > 0.85 / f64::from(rate.hz())
                                && interval < 1.15 / f64::from(rate.hz()),
                            "Prediction cadence jumped: {interval} at {rate:?}"
                        );
                        intervals += 1;
                    }
                    previous_advance = Some(elapsed);
                }
            }
            previous_tick = Some(driven.tick);
        }
        assert!(intervals > 50);
        assert_eq!(driver.stats.clock_phase_backsteps, 0);
        assert!(driver.stats.max_clock_slew <= 0.05 + f64::EPSILON);
        assert!(driver.stats.queue_drops == 0 && driver.stats.max_history <= 64);
    }
}
#[test]
fn stale_clocks_do_not_emit_commands_and_fresh_truth_resumes_without_sequence_reuse() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    let intent = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    driver.tick(&state, intent, Some(&patch), now).unwrap();
    let sent = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap()
        .packet
        .unwrap();
    let sequence = sent.commands.last().unwrap().sequence;
    let stale = driver
        .tick(&state, intent, Some(&patch), now + Duration::from_secs(3))
        .unwrap()
        .unwrap();
    assert!(stale.packet.is_none());
    let packet = state.correction.as_mut().unwrap();
    packet.state.input.tick = 90;
    packet.state.input.fresh_tick = 90;
    state.correction_received = Some(now + Duration::from_secs(3));
    let driven = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(3040),
        )
        .unwrap()
        .unwrap();
    let resumed = driven.packet.unwrap();
    assert!(
        resumed
            .commands
            .iter()
            .all(|c| c.sequence > sequence && c.tick > 90)
    );
    assert!(driver.stats.max_history <= 64);
}
#[test]
fn unsupported_latency_and_missing_collision_cannot_generate_unbounded_clock_debt() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    state.transport = Some(TransportStats {
        rtt_ms: 10000.,
        ..TransportStats::default()
    });
    let result = driver
        .tick(&state, Intent::default(), Some(&patch), now)
        .unwrap()
        .unwrap();
    assert!(result.packet.is_none());
    assert_eq!(driver.stats.predicted_steps, 0);
    state.transport = None;
    let result = driver
        .tick(
            &state,
            Intent::default(),
            None,
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap();
    assert!(result.packet.is_none());
    assert!(driver.stats.max_history <= 64);
}

#[test]
fn pending_readiness_replacement_preserves_only_still_loaded_authorized_overlap() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::default();
    state.chunk_report.as_mut().unwrap().sequence = 2;
    state.chunk_report.as_mut().unwrap().chunks =
        procedural::neighbours(ChunkId { x: 63, z: 62 }, 1)
            .iter()
            .map(|id| [id.x, id.z])
            .collect();
    let initial = state.correction.unwrap().state.pose.feet.metres()[0];
    let moving = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    driver.tick(&state, moving, Some(&patch), now).unwrap();
    let driven = driver
        .tick(
            &state,
            moving,
            Some(&patch),
            now + Duration::from_millis(40),
        )
        .unwrap()
        .unwrap();
    assert!(driven.pose.feet.metres()[0] > initial);
    assert_eq!(
        driven.packet.unwrap().ready_sequence,
        1,
        "Command describes observed authority permission"
    );
    state.chunk_report.as_mut().unwrap().sequence = 3;
    state.chunk_report.as_mut().unwrap().chunks.clear();
    let held = driver
        .tick(
            &state,
            moving,
            Some(&patch),
            now + Duration::from_millis(80),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        held.pose.feet.metres()[0],
        driven.pose.feet.metres()[0],
        "Withdrawn live geometry cannot be predicted into"
    );
}
#[test]
fn correction_focus_can_precede_the_snapshot_across_a_continent_teleport() {
    use theia_network::wire::{Player, Snapshot};
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    state.id = Some(1);
    state.snapshot = Some(Snapshot {
        version: VERSION,
        epoch: 7,
        origin: [8000., 0., 8000.],
        tick: 10,
        tick_hz: 30,
        players: vec![Player {
            facing: None,
            id: 1,
            xz: [0., 0.],
            y: 1.,
            applied_sequence: 0,
        }],
    });
    let correction = state.correction.as_mut().unwrap();
    correction.state.input.tick = 11;
    correction.state.pose.feet = GlobalPosition::new([15800., 25., 180.]).unwrap();
    assert_eq!(
        state.authority_focus().unwrap().metres(),
        [15800., 25. + f64::from(theia_network::world::PAWN_Y), 180.]
    );
    state.snapshot.as_mut().unwrap().tick = 12;
    assert_eq!(
        state.authority_focus().unwrap().metres(),
        [8000., 1., 8000.]
    );
}

fn life_update(state: &mut State, tick: u64, stream: u64, generation: u64, health: u32) {
    use std::{num::NonZeroU64, sync::Arc};
    use theia_core::player_life::LifeKey;
    use theia_network::{player_life_wire as wire, simulation::combat::player_life::Observation};
    let c = state.correction.unwrap().state;
    let profile = Arc::new(
        wire::Profile::from_json(
            include_bytes!("../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../content/gameplay/development-creature-patrol.json"),
            include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap(),
    );
    let value = Observation {
        combat: None,
        profile: profile.hash(),
        key: LifeKey {
            actor: c.actor,
            generation: NonZeroU64::new(generation).unwrap(),
        },
        tick,
        movement_stream: stream,
        health,
        maximum_health: 100,
        respawn_at: (health == 0).then_some(tick + u64::from(c.rate.hz()) * 3),
        windup: None,
    };
    let bytes = wire::encode(&profile, c.rate, &value).unwrap();
    let life = state
        .player_life
        .get_or_insert_with(|| wire::ClientState::new(profile, c.actor, c.rate, c.world).unwrap());
    life.apply(&bytes).unwrap();
}

#[test]
fn opt_in_life_prediction_waits_for_the_initial_observation_before_sampling_input() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::with_player_life();
    let intent = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    for offset in [0, 40, 100] {
        let held = driver
            .tick(
                &state,
                intent,
                Some(&patch),
                now + Duration::from_millis(offset),
            )
            .unwrap()
            .unwrap();
        assert_eq!(held.pose, state.correction.unwrap().state.pose);
        assert!(held.packet.is_none() && driver.life_held());
    }
    assert_eq!(driver.stats.predicted_steps, 0);
    assert_eq!(driver.stats.life_waiting_frames, 3);
    life_update(&mut state, 0, 1, 1, 100);
    let resumed = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(120),
        )
        .unwrap()
        .unwrap();
    assert!(resumed.reset && resumed.packet.is_none());
    assert!(!driver.life_held());
    let next = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(160),
        )
        .unwrap()
        .unwrap();
    assert!(!next.packet.unwrap().commands.is_empty());
}

#[test]
fn death_and_reordered_respawn_discard_input_and_clock_debt_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let patch = Patch::build(ChunkId { x: 62, z: 62 });
        let now = Instant::now();
        let mut state = state(&patch, now);
        state.correction.as_mut().unwrap().state.rate = rate;
        state.correction.as_mut().unwrap().state.input = Timeline::new(1, rate, 0).unwrap().state();
        let mut driver = Driver::with_player_life();
        let intent = Intent {
            axes: [1000, 0],
            jump: true,
            ..Intent::default()
        };
        life_update(&mut state, 0, 1, 1, 100);
        driver.tick(&state, intent, Some(&patch), now).unwrap();
        let moved = driver
            .tick(
                &state,
                intent,
                Some(&patch),
                now + Duration::from_millis(80),
            )
            .unwrap()
            .unwrap();
        assert!(moved.packet.is_some());
        life_update(&mut state, 5, 2, 1, 0);
        let held = driver
            .tick(
                &state,
                intent,
                Some(&patch),
                now + Duration::from_millis(90),
            )
            .unwrap()
            .unwrap();
        assert!(held.reset && held.packet.is_none() && driver.life_held());
        assert_eq!(held.pose, state.correction.unwrap().state.pose);
        // A correction for life 2 arriving first is still blocked by the dead
        // observation, and a later life frame cannot authorize an older tick.
        let future = now + Duration::from_secs(600);
        state.correction_received = Some(future);
        state.correction.as_mut().unwrap().state.input =
            Timeline::new(3, rate, 99).unwrap().state();
        assert!(
            driver
                .tick(&state, intent, Some(&patch), future)
                .unwrap()
                .unwrap()
                .packet
                .is_none()
        );
        assert!(driver.life_held());
        life_update(&mut state, 100, 3, 2, 100);
        assert!(
            driver
                .tick(&state, intent, Some(&patch), future)
                .unwrap()
                .unwrap()
                .packet
                .is_none()
        );
        assert!(driver.life_held());
        state.correction.as_mut().unwrap().state.input =
            Timeline::new(3, rate, 100).unwrap().state();
        let before = driver.stats.predicted_steps;
        let resumed = driver
            .tick(&state, intent, Some(&patch), future)
            .unwrap()
            .unwrap();
        assert!(resumed.reset && resumed.packet.is_none() && !driver.life_held());
        assert_eq!(driver.stats.predicted_steps, before);
        let next = driver
            .tick(
                &state,
                intent,
                Some(&patch),
                future + Duration::from_millis(60),
            )
            .unwrap()
            .unwrap();
        let packet = next.packet.unwrap();
        assert_eq!(packet.stream_epoch, 3);
        assert_eq!(packet.commands[0].sequence, 1);
        assert!(packet.commands.iter().all(|command| command.tick > 100));
        assert!(packet.commands.len() <= 4);
    }
}

#[test]
fn ordinary_life_damage_does_not_pause_or_reacquire_the_prediction_clock() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut state = state(&patch, now);
    let mut driver = Driver::with_player_life();
    life_update(&mut state, 0, 1, 1, 100);
    let intent = Intent {
        axes: [1000, 0],
        ..Intent::default()
    };
    driver.tick(&state, intent, Some(&patch), now).unwrap();
    driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(40),
        )
        .unwrap();
    let resets = driver.stats.clock_resets;
    // Latest health can legitimately precede the independently delivered
    // correction; the established movement barrier remains at life admission.
    life_update(&mut state, 5, 1, 1, 95);
    let moving = driver
        .tick(
            &state,
            intent,
            Some(&patch),
            now + Duration::from_millis(80),
        )
        .unwrap()
        .unwrap();
    assert!(moving.packet.is_some() && !moving.reset && !driver.life_held());
    assert_eq!(driver.stats.clock_resets, resets);
    assert_eq!(driver.stats.life_waiting_frames, 0);
}

fn dodge_profile() -> std::sync::Arc<theia_network::dodging::Profile> {
    std::sync::Arc::new(
        theia_network::dodging::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-dodge.json"
        ))
        .unwrap(),
    )
}
fn dodge_state(patch: &Patch, now: Instant) -> State {
    let mut s = state(patch, now);
    let c = &mut s.correction.as_mut().unwrap().state;
    c.world = theia_network::dodging::profile_identity(procedural::IDENTITY, &dodge_profile());
    c.pose.motion.grounded = true;
    c.pose.dodge = Some(
        theia_network::dodging::Controller::new(&dodge_profile(), c.rate, c.input.tick).snapshot(),
    );
    s
}
#[test]
fn dodge_driver_requires_exact_binding_and_cannot_silently_downgrade() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut s = dodge_state(&patch, now);
    assert!(
        Driver::default()
            .tick(&s, Intent::default(), Some(&patch), now)
            .is_err()
    );
    assert!(
        Driver::with_dodge([0; 32], dodge_profile(), false)
            .tick(&s, Intent::default(), Some(&patch), now)
            .is_err()
    );
    let mut driver = Driver::with_dodge(procedural::IDENTITY, dodge_profile(), false);
    assert!(
        driver
            .tick(&s, Intent::default(), Some(&patch), now)
            .unwrap()
            .is_some()
    );
    s.correction.as_mut().unwrap().state.pose.dodge = None;
    assert!(
        driver
            .tick(&s, Intent::default(), Some(&patch), now)
            .is_err()
    );
    let mut s = dodge_state(&patch, now);
    s.correction.as_mut().unwrap().speed = 8;
    assert!(
        driver
            .tick(&s, Intent::default(), Some(&patch), now)
            .is_err()
    );
}
#[test]
fn actual_dodge_driver_and_authority_converge_with_one_tap_at_120_render_hz_and_all_server_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let now = Instant::now();
        let mut w = theia_network::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(w.set_tick_rate(rate));
        w.enable_prediction().unwrap();
        let source = w.identity();
        w.enable_dodge(dodge_profile()).unwrap();
        let id = w.join().unwrap();
        let patch = w
            .procedural()
            .unwrap()
            .patch(w.global_position(id).unwrap())
            .unwrap()
            .clone();
        let report = ChunkReady {
            version: VERSION,
            epoch: w.epoch(),
            sequence: 1,
            chunks: patch.ids.iter().map(|c| [c.x, c.z]).collect(),
        };
        assert!(w.client_ready(id, &report));
        let mut s = State {
            chunk_report: Some(report.clone()),
            correction_ready: Some(report),
            ..State::default()
        };
        let start = w.correction(id).unwrap().state.pose.feet;
        let mut driver = Driver::with_dodge(source, dodge_profile(), false);
        let mut edge = theia_network::dodge_input::Input::new(source, dodge_profile());
        let mut sent = 0;
        for frame in 0..600u64 {
            let time = now + Duration::from_secs_f64(frame as f64 / 120.);
            if frame % u64::from(120 / rate.hz()) == 0 {
                w.step();
                s.correction = Some(w.correction(id).unwrap());
                s.correction_received = Some(time);
            }
            let request = edge
                .update(
                    s.correction.map(|c| c.state),
                    true,
                    (frame == 40).then_some([1000, 0]),
                    time,
                )
                .unwrap();
            let driven = driver
                .tick(
                    &s,
                    Intent {
                        facing: None,
                        dodge: request,
                        ..Default::default()
                    },
                    Some(&patch),
                    time,
                )
                .unwrap()
                .unwrap();
            if let Some(packet) = driven.packet {
                sent += 1;
                w.command(id, &packet);
            }
        }
        assert!(sent > 0);
        let authority = w.correction(id).unwrap().state.pose;
        assert_eq!(authority.dodge.unwrap().last_nonce, 1);
        assert!((authority.feet.metres()[0] - start.metres()[0] - 3.).abs() < 0.002);
        assert!(
            (driver.pose().unwrap().feet.metres()[0] - authority.feet.metres()[0]).abs() < 0.001
        );
        assert!(
            driver.stats.max_same_tick_error_metres < 0.001,
            "{:?}",
            driver.stats
        );
    }
}
#[test]
fn dodge_driver_holds_unmatched_life_and_missing_collision_without_commands() {
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let now = Instant::now();
    let mut s = dodge_state(&patch, now);
    let mut d = Driver::with_dodge(procedural::IDENTITY, dodge_profile(), true);
    let held = d
        .tick(&s, Intent::default(), Some(&patch), now)
        .unwrap()
        .unwrap();
    assert!(held.packet.is_none());
    assert!(d.life_held());
    life_update(&mut s, 0, 1, 1, 100);
    let ready = d
        .tick(&s, Intent::default(), Some(&patch), now)
        .unwrap()
        .unwrap();
    assert!(ready.packet.is_none());
    assert!(!d.life_held());
    let held = d
        .tick(
            &s,
            Intent::default(),
            None,
            now + Duration::from_millis(100),
        )
        .unwrap()
        .unwrap();
    assert!(held.packet.is_none());
    life_update(&mut s, 1, 2, 1, 0);
    let held = d
        .tick(
            &s,
            Intent::default(),
            Some(&patch),
            now + Duration::from_millis(110),
        )
        .unwrap()
        .unwrap();
    assert!(held.packet.is_none());
    assert!(d.life_held());
}
