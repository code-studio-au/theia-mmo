//! Actual World + prepared Avian terrain. No rendered/transport qualification.
use theia_core::{facing::Facing, time::TickRate};
use theia_network::{
    classic, classic_wire,
    commands::Command,
    prediction::{History, Predicted, Reconciled},
    prediction_wire::Commands,
    procedural::{self, Patch},
    simulation::World,
    wire::{self, ChunkReady, Intent, VERSION},
};
fn world(rate: TickRate) -> (World, u64, std::sync::Arc<Patch>) {
    let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(w.set_tick_rate(rate));
    w.enable_prediction().unwrap();
    w.enable_classic().unwrap();
    let id = w.join().unwrap();
    let patch = w
        .procedural()
        .unwrap()
        .patch(w.global_position(id).unwrap())
        .unwrap()
        .clone();
    assert!(w.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: w.epoch(),
            sequence: 1,
            chunks: patch.ids.iter().map(|c| [c.x, c.z]).collect(),
        }
    ));
    w.step();
    (w, id, patch)
}
fn input(facing: u16, local: [i16; 2]) -> Intent {
    let f = Facing::from_units(facing);
    Intent {
        facing: Some(f),
        axes: f.world_axes(local).unwrap(),
        ..Intent::default()
    }
}
fn packet(w: &World, id: u64, seq: u64, intent: Intent) -> Commands {
    let c = w.correction(id).unwrap().state;
    Commands {
        epoch: w.epoch(),
        stream_epoch: c.input.stream_epoch,
        ready_sequence: c.ready_sequence,
        commands: vec![Command {
            sequence: seq,
            tick: c.input.tick + 1,
            intent,
        }],
    }
}
#[test]
fn fixed_step_authority_and_prepared_prediction_agree_on_independent_facing_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, patch) = world(rate);
        let mut pose = w.correction(id).unwrap().state.pose;
        let initial = pose.feet;
        for n in 1..=rate.hz() * 4 {
            let mut intent = match n / rate.hz() {
                0 => input(16384, [0, 0]),    // Turn in place without travel.
                1 => input(16384, [1000, 0]), // Strafe while looking left.
                2 => input(16384, [0, 1000]), // Backpedal, preserve facing.
                _ => input(32768, [0, -1000]),
            };
            // Exercise both v3 walk and run through actual World/prediction.
            intent.sprint = n % rate.hz() >= rate.hz() / 2;
            let p = packet(&w, id, u64::from(n), intent);
            assert_eq!(w.command(id, &p), 1);
            w.step();
            pose = classic::terrain_step(&patch, &patch.ids, pose, intent, rate)
                .unwrap()
                .unwrap();
            let truth = w.correction(id).unwrap().state.pose;
            let encoded = classic_wire::encode_correction(w.correction(id).unwrap()).unwrap();
            assert_eq!(
                classic_wire::decode_correction(&encoded).unwrap(),
                w.correction(id).unwrap()
            );
            assert_eq!(pose.facing, truth.facing);
            assert_eq!(pose.motion, truth.motion);
            // Authority stores a body centre, prediction stores capsule feet.
            // The existing f32 centre/feet conversion introduces sub-mm noise.
            for (a, b) in pose.feet.metres().into_iter().zip(truth.feet.metres()) {
                assert!((a - b).abs() < 0.0001, "{pose:?} != {truth:?}");
            }
            assert_eq!(w.snapshot_for(id).players[0].facing, pose.facing);
            if n < rate.hz() {
                assert_eq!(pose.feet.metres()[0], initial.metres()[0]);
                assert_eq!(pose.feet.metres()[2], initial.metres()[2]);
            }
        }
    }
}
#[test]
fn expiry_stops_travel_but_keeps_facing_and_conflicting_or_replayed_commands_cannot_turn() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut w, id, _) = world(rate);
        let p = packet(&w, id, 1, input(16384, [1000, 0]));
        assert_eq!(w.command(id, &p), 1);
        let mut conflict = p.clone();
        conflict.commands[0].intent = input(32768, [0, 0]);
        assert_eq!(w.command(id, &conflict), 0);
        assert_eq!(w.command(id, &p), 0);
        for _ in 0..rate.hz() {
            w.step();
        }
        let stopped = w.correction(id).unwrap().state.pose;
        assert_eq!(stopped.facing, Some(Facing::from_units(16384)));
        assert_eq!(
            w.correction(id).unwrap().state.input.intent,
            Intent::default()
        );
        assert_eq!(w.command(id, &conflict), 0);
        for _ in 0..rate.hz() {
            w.step();
        }
        assert_eq!(w.correction(id).unwrap().state.pose, stopped);
    }
}
#[test]
fn delayed_correction_replays_travel_and_facing_without_using_camera_or_target_state() {
    let (mut w, id, patch) = world(TickRate::Hz30);
    let mut history = History::new(w.correction(id).unwrap().state).unwrap();
    let solve =
        |pose, intent, rate, _tick| classic::terrain_step(&patch, &patch.ids, pose, intent, rate);
    for n in 1..=90 {
        let intent = input(
            if n < 30 { 16384 } else { 32768 },
            if n < 15 { [0, 0] } else { [1000, 0] },
        );
        let Predicted::Advanced(Some(command)) =
            history.predict_at(Some(intent), 1, solve).unwrap()
        else {
            panic!("Expected command")
        };
        // Drop one turn command, then deliver later authority correction.
        if n != 30 {
            let mut p = packet(&w, id, command.sequence, intent);
            p.commands = vec![command];
            assert_eq!(w.command(id, &p), 1);
        }
        w.step();
        if n % 7 == 0 || n == 90 {
            assert!(matches!(
                history
                    .reconcile_at(w.correction(id).unwrap().state, solve)
                    .unwrap(),
                Reconciled::Applied { .. }
            ));
            assert_eq!(history.pose(), w.correction(id).unwrap().state.pose);
        }
    }
}
#[test]
fn codecs_are_canonical_bounded_and_legacy_profiles_refuse_facing() {
    let (w, id, _) = world(TickRate::Hz30);
    let p = packet(&w, id, 1, input(65535, [0, 0]));
    let bytes = classic_wire::encode_commands(&p).unwrap();
    assert_eq!(bytes.len(), 52);
    assert_eq!(
        classic_wire::decode_commands(&bytes).unwrap().commands,
        p.commands
    );
    assert!(p.encode().is_err());
    assert!(theia_network::dodge_prediction_wire::encode_commands(&p).is_err());
    for len in 0..bytes.len() {
        assert!(classic_wire::decode_commands(&bytes[..len]).is_err());
    }
    let mut appended = bytes.clone();
    appended.push(0);
    assert!(classic_wire::decode_commands(&appended).is_err());
    let mut excess = p.clone();
    excess.commands = vec![p.commands[0]; 5];
    assert!(classic_wire::encode_commands(&excess).is_err());
    let c = w.correction(id).unwrap();
    assert!(c.encode().is_err());
    let bytes = classic_wire::encode_correction(c).unwrap();
    assert_eq!(bytes.len(), classic_wire::CORRECTION_BYTES);
    assert_eq!(classic_wire::decode_correction(&bytes).unwrap(), c);
    for len in 0..bytes.len() {
        assert!(classic_wire::decode_correction(&bytes[..len]).is_err());
    }
    let mut bad = bytes.clone();
    bad[132] = 1;
    assert!(classic_wire::decode_correction(&bad).is_err());
    let mut bad = bytes;
    bad[130] = 2;
    assert!(classic_wire::decode_correction(&bad).is_err());
}
#[test]
fn only_inactive_normal_speed_predicted_worlds_accept_classic_profile() {
    let mut w = World::default();
    assert!(w.enable_classic().is_err());
    let mut fast = World::with_procedural([8000., 8000.], 8.).unwrap();
    fast.enable_prediction().unwrap();
    assert!(fast.enable_classic().is_err());
    let mut w = World::with_procedural([8000., 8000.], 1.).unwrap();
    w.enable_prediction().unwrap();
    let old = w.identity();
    w.enable_classic().unwrap();
    assert_eq!(w.identity(), classic::profile_identity(old));
    assert!(w.enable_classic().is_err());
    let id = w.join().unwrap();
    assert!(w.enable_classic().is_err());
    assert_eq!(w.command(id, &packet(&w, id, 1, Intent::default())), 0);
    let mut old = World::with_procedural([8000., 8000.], 1.).unwrap();
    old.enable_prediction().unwrap();
    let id = old.join().unwrap();
    assert_eq!(old.command(id, &packet(&old, id, 1, input(1, [0, 0]))), 0);
}
#[test]
fn eight_player_snapshot_stays_within_existing_datagram_budget_with_facing() {
    let (mut w, id, _) = world(TickRate::Hz30);
    for _ in 1..wire::MAX_PLAYERS {
        w.join().unwrap();
    }
    let snapshot = w.snapshot_for(id);
    let bytes = wire::encode(&snapshot).unwrap();
    assert!(bytes.len() <= wire::MAX_BYTES);
    assert_eq!(wire::Snapshot::decode(&bytes).unwrap(), snapshot);
    assert!(snapshot.players.iter().all(|p| p.facing.is_some()));
    assert_eq!(
        classic::profile_identity(procedural::IDENTITY),
        w.identity()
    );
}
