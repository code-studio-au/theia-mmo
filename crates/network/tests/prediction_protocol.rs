use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    commands::{Command, Timeline},
    kinematic::Motion,
    prediction::{Correction, Pose},
    prediction_wire::{Commands, Corrected},
    wire::Intent,
};
#[test]
fn fixed_size_corrections_round_trip_and_reject_truncation_trailing_flags_and_nonfinite_state() {
    let value = Corrected {
        speed: 32,
        state: Correction {
            world: [42; 32],
            actor: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(2).unwrap(),
            },
            rate: TickRate::Hz60,
            input: Timeline::new(3, TickRate::Hz60, 90).unwrap().state(),
            pose: Pose {
                feet: GlobalPosition::new([15800.5, 19.125, 8000.25]).unwrap(),
                motion: Motion::default().snapshot(),
            },
            ready_sequence: 4,
        },
    };
    let bytes = value.encode().unwrap();
    assert_eq!(bytes.len(), 128);
    assert_eq!(Corrected::decode(&bytes).unwrap(), value);
    for length in 0..bytes.len() {
        assert!(Corrected::decode(&bytes[..length]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(Corrected::decode(&extra).is_err());
    for (offset, bad) in [(0, 255), (52, 0), (118, 4), (127, 2)] {
        let mut fault = bytes.clone();
        fault[offset] = bad;
        assert!(Corrected::decode(&fault).is_err());
    }
    let mut fault = bytes;
    fault[114..118].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(Corrected::decode(&fault).is_err());
}
#[test]
fn command_packets_are_bounded_monotonic_and_never_contain_client_coordinates() {
    let mut value = Commands {
        epoch: 7,
        stream_epoch: 3,
        ready_sequence: 4,
        commands: (1..=4)
            .map(|n| Command {
                sequence: n,
                tick: 90 + n,
                intent: Intent::default(),
            })
            .collect(),
    };
    let bytes = value.encode().unwrap();
    assert_eq!(bytes.len(), 113);
    assert_eq!(Commands::decode(&bytes).unwrap().commands, value.commands);
    for length in 0..bytes.len() {
        assert!(Commands::decode(&bytes[..length]).is_err());
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(Commands::decode(&extra).is_err());
    value.commands.push(value.commands[3]);
    assert!(value.encode().is_err());
    value.commands.truncate(4);
    value.commands.swap(0, 1);
    assert!(value.encode().is_err());
    value.commands.clear();
    assert!(value.encode().is_err());
}
