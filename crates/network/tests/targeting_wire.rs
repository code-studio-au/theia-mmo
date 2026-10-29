use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
};
use theia_network::{
    procedural,
    simulation::World,
    targeting::{self, MAX_TARGETS, Snapshot, Target},
    targeting_wire::{Selection, State},
    wire::MAX_BYTES,
};
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn state() -> State {
    State::new(
        7,
        42,
        1,
        true,
        Snapshot {
            selected: Some(key(3)),
            targets: vec![Target {
                key: key(3),
                position: GlobalPosition::new([1e9 + 0.01, 12., -1e9 + 0.02]).unwrap(),
            }],
        },
    )
}
#[test]
fn selection_roundtrips_explicit_clear_and_rejects_bad_identity_or_frame() {
    for target in [None, Some(EntityId::new(3).unwrap())] {
        let value = Selection {
            epoch: 7,
            sequence: 1,
            target,
        };
        let bytes = value.encode().unwrap();
        assert_eq!(Selection::decode(&bytes).unwrap(), value);
        for len in 0..bytes.len() {
            assert!(Selection::decode(&bytes[..len]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Selection::decode(&trailing).is_err());
        let mut foreign = bytes.clone();
        foreign[0] = 0;
        assert!(Selection::decode(&foreign).is_err());
    }
    assert!(
        Selection {
            epoch: 0,
            sequence: 1,
            target: None
        }
        .encode()
        .is_err()
    );
    assert!(
        Selection {
            epoch: 7,
            sequence: 0,
            target: None
        }
        .encode()
        .is_err()
    );
    let mut bytes = Selection {
        epoch: 7,
        sequence: 1,
        target: None,
    }
    .encode()
    .unwrap();
    bytes[12..20].fill(0);
    assert!(Selection::decode(&bytes).is_err());
}
#[test]
fn target_state_preserves_global_centimetres_and_only_timestamp_changes_are_coalescible() {
    let value = state();
    let bytes = value.encode().unwrap();
    assert_eq!(State::decode(&bytes).unwrap(), value);
    assert!(bytes.len() <= MAX_BYTES);
    let mut next = value.clone();
    next.tick += 1;
    assert!(value.same_view(&next));
    next.request_sequence += 1;
    assert!(!value.same_view(&next));
    next = value.clone();
    next.accepted = false;
    assert!(!value.same_view(&next));
    next = value.clone();
    next.selected = None;
    assert!(!value.same_view(&next));
    next = value.clone();
    next.targets[0].position = GlobalPosition::new([1e9 + 0.02, 12., -1e9 + 0.02]).unwrap();
    assert!(!value.same_view(&next));
}
#[test]
fn membership_identity_count_boolean_and_nonfinite_coordinate_faults_fail_closed() {
    let value = state();
    let bytes = value.encode().unwrap();
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(State::decode(&bad).is_err());
    let mut bad = bytes.clone();
    bad[28] = 2;
    assert!(State::decode(&bad).is_err());
    for coordinate in [f64::NAN, f64::INFINITY, 1e20] {
        let mut bad = bytes.clone();
        bad[46..54].copy_from_slice(&coordinate.to_le_bytes());
        assert!(State::decode(&bad).is_err());
    }
    let mut bad = value.clone();
    bad.selected = Some(EntityId::new(99).unwrap());
    assert!(bad.encode().is_err());
    let mut bad = value.clone();
    bad.targets.push(bad.targets[0]);
    assert!(bad.encode().is_err());
    let mut bad = value.clone();
    bad.targets[0].key.epoch = WorldEpoch::new(8).unwrap();
    assert!(bad.encode().is_err());
    let mut bad = value.clone();
    bad.request_sequence = 0;
    assert!(bad.encode().is_err());
    let mut full = value;
    full.selected = None;
    full.targets = (1..=MAX_TARGETS as u64)
        .map(|id| Target {
            key: key(id),
            position: GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        })
        .collect();
    let bytes = full.encode().unwrap();
    assert_eq!(bytes.len(), 550);
    assert!(bytes.len() < MAX_BYTES);
    assert_eq!(State::decode(&bytes).unwrap(), full);
    full.targets.push(Target {
        key: key(MAX_TARGETS as u64 + 1),
        position: GlobalPosition::new([8000., 0., 8000.]).unwrap(),
    });
    assert!(full.encode().is_err());
    let mut bad = bytes;
    bad[37] = (MAX_TARGETS + 1) as u8;
    assert!(State::decode(&bad).is_err());
}
#[test]
fn profile_binding_rejects_baseline_world_identity_and_survives_target_removal() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert_eq!(world.identity(), procedural::IDENTITY);
    assert!(!world.targeting_enabled());
    let target = world.spawn_development_target([8008., 8000.]).unwrap();
    let identity = targeting::profile_identity(procedural::IDENTITY);
    assert_ne!(identity, procedural::IDENTITY);
    assert_eq!(world.identity(), identity);
    assert!(world.targeting_enabled());
    assert!(world.remove_development_target(target));
    assert_eq!(
        world.identity(),
        identity,
        "Removing the last NPC must not change session profile"
    );
    assert!(world.targeting_enabled());
}
