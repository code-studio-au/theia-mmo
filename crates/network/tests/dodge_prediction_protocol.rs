use theia_core::{
    coordinates::GlobalPosition,
    dodge::Gate,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    commands::{Command, Timeline},
    dodge_prediction_wire as wire,
    dodging::{Controller, Profile, Request},
    kinematic::Motion,
    prediction::{Correction, Pose},
    prediction_wire::{Commands, Corrected},
};
fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-dodge.json"
    ))
    .unwrap()
}
fn request() -> Request {
    Request {
        nonce: 7,
        axes: [1000, -500],
    }
}
fn correction(rate: TickRate, tick: u64, controller: Controller) -> Corrected {
    Corrected {
        speed: 1,
        state: Correction {
            world: [42; 32],
            actor: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(2).unwrap(),
            },
            rate,
            input: Timeline::new(3, rate, tick).unwrap().state(),
            pose: Pose {
                facing: None,
                feet: GlobalPosition::new([8000., 20., 8000.]).unwrap(),
                motion: Motion::default().snapshot(),
                dodge: Some(controller.snapshot()),
            },
            ready_sequence: 4,
        },
    }
}
#[test]
fn commands_roundtrip_at_bound_and_old_codec_cannot_erase_edges() {
    let mut value = Commands {
        epoch: 7,
        stream_epoch: 3,
        ready_sequence: 4,
        commands: (1..=4)
            .map(|n| Command {
                sequence: n,
                tick: 90 + n,
                intent: theia_network::wire::Intent {
                    facing: None,
                    dodge: (n % 2 == 0).then_some(request()),
                    ..Default::default()
                },
            })
            .collect(),
    };
    let bytes = wire::encode_commands(&value).unwrap();
    assert_eq!(bytes.len(), wire::MAX_COMMAND_BYTES);
    assert_eq!(
        wire::decode_commands(&bytes).unwrap().commands,
        value.commands
    );
    assert!(value.encode().is_err());
    assert!(Commands::decode(&bytes).is_err());
    for n in 0..bytes.len() {
        assert!(wire::decode_commands(&bytes[..n]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(wire::decode_commands(&bad).is_err());
    for command in &mut value.commands {
        command.intent.dodge = None;
    }
    let old = value.encode().unwrap();
    assert_eq!(old.len(), 113);
    assert!(wire::decode_commands(&old).is_err());
    assert_eq!(Commands::decode(&old).unwrap().commands, value.commands);
}
#[test]
fn noncanonical_absent_requests_and_invalid_edges_fail() {
    let mut value = Commands {
        epoch: 1,
        stream_epoch: 1,
        ready_sequence: 0,
        commands: vec![Command {
            sequence: 1,
            tick: 1,
            intent: Default::default(),
        }],
    };
    let original = wire::encode_commands(&value).unwrap();
    for offset in 50..63 {
        let mut fault = original.clone();
        fault[offset] = 255;
        assert!(wire::decode_commands(&fault).is_err(), "{offset}");
    }
    for r in [
        Request {
            nonce: 0,
            ..request()
        },
        Request {
            axes: [0, 0],
            ..request()
        },
        Request {
            axes: [1001, 0],
            ..request()
        },
    ] {
        value.commands[0].intent.dodge = Some(r);
        assert!(wire::encode_commands(&value).is_err());
    }
    value.commands[0].intent.dodge = Some(request());
    let original = wire::encode_commands(&value).unwrap();
    let mut fault = original;
    fault[50] = 2;
    assert!(wire::decode_commands(&fault).is_err());
}
#[test]
fn every_active_and_completed_tick_roundtrips_without_resetting_budget() {
    let p = profile();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut controller = Controller::new(&p, rate, 0);
        for tick in 1..=u64::from(rate.hz()) * 4 {
            controller
                .advance(
                    &p,
                    rate,
                    tick,
                    (tick == 1).then_some(request()),
                    Gate {
                        alive: true,
                        grounded: true,
                        terrain_ready: true,
                    },
                )
                .unwrap();
            let mut value = correction(rate, tick, controller);
            value.state.input.intent.dodge = Some(request());
            let bytes = wire::encode_correction(value, &p).unwrap();
            assert_eq!(bytes.len(), wire::CORRECTION_BYTES);
            assert_eq!(wire::decode_correction(&bytes, &p).unwrap(), value);
            assert!(value.encode().is_err());
            assert!(Corrected::decode(&bytes).is_err());
        }
    }
}
#[test]
fn correction_rejects_malformed_policy_lengths_flags_speed_and_old_profile() {
    let p = profile();
    let mut value = correction(TickRate::Hz30, 20, Controller::new(&p, TickRate::Hz30, 20));
    let original = wire::encode_correction(value, &p).unwrap();
    for length in 0..original.len() {
        assert!(wire::decode_correction(&original[..length], &p).is_err());
    }
    let mut extra = original.clone();
    extra.push(0);
    assert!(wire::decode_correction(&extra, &p).is_err());
    for (offset, byte) in [
        (0, 0),
        (127, 8),
        (128, 2),
        (129, 1),
        (149, 255),
        (157, 2),
        (158, 1),
        (166, 1),
    ] {
        let mut fault = original.clone();
        fault[offset] = byte;
        assert!(wire::decode_correction(&fault, &p).is_err(), "{offset}");
    }
    value.state.pose.dodge.as_mut().unwrap().policy.tick += 1;
    assert!(wire::encode_correction(value, &p).is_err());
    value.state.pose.dodge = None;
    assert!(wire::encode_correction(value, &p).is_err());
    let old = value.encode().unwrap();
    assert!(wire::decode_correction(&old, &p).is_err());
}
#[test]
fn impossible_active_cooldowns_nonce_and_axes_fail() {
    let p = profile();
    let rate = TickRate::Hz30;
    let mut controller = Controller::new(&p, rate, 0);
    controller
        .advance(
            &p,
            rate,
            1,
            Some(request()),
            Gate {
                alive: true,
                grounded: true,
                terrain_ready: true,
            },
        )
        .unwrap();
    let original = wire::encode_correction(correction(rate, 1, controller), &p).unwrap();
    let mut no_nonce = original.clone();
    no_nonce[141..149].fill(0);
    assert!(wire::decode_correction(&no_nonce, &p).is_err());
    let mut no_axes = original.clone();
    no_axes[166..170].fill(0);
    assert!(wire::decode_correction(&no_axes, &p).is_err());
    let mut wrong_cooldown = original;
    wrong_cooldown[149] += 1;
    assert!(wire::decode_correction(&wrong_cooldown, &p).is_err());
}
