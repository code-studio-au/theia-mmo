use std::time::{Duration, Instant};
use theia_core::{
    identity::{EntityKey, WorldEpoch},
    time::TickRate,
};
use theia_network::{
    simulation::World,
    targeting::MAX_TARGETS,
    wire::{ChunkReady, Input, Intent, VERSION},
};
fn world() -> World {
    World::with_procedural([8060., 8000.], 1.).unwrap()
}
fn wrong_epoch(world: &World, key: EntityKey) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(if world.epoch() == 1 { 2 } else { 1 }).unwrap(),
        ..key
    }
}
fn ready(world: &mut World, id: u64) {
    let position = world.global_position(id).unwrap();
    let chunks = world
        .procedural()
        .unwrap()
        .patch(position)
        .unwrap()
        .ids
        .iter()
        .map(|id| [id.x, id.z])
        .collect();
    assert!(world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks
        }
    ));
}
#[test]
fn real_authority_traverses_a_selected_target_without_actor_collision_at_every_rate() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = world();
        assert!(world.set_tick_rate(rate));
        let target = world.spawn_development_target([8064., 8000.]).unwrap();
        let actor = world.join().unwrap();
        ready(&mut world, actor);
        assert_ne!(target, world.entity_key(actor).unwrap());
        assert!(world.select_target(actor, Some(target)));
        let start = world.global_position(actor).unwrap().metres();
        for sequence in 1..=u64::from(rate.hz()) {
            assert!(world.input(
                actor,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [1000, 0],
                        sprint: true,
                        jump: false
                    }
                }
            ));
            let snapshot = world.step();
            assert_eq!(snapshot.players.len(), 1, "Targets are not fake players");
        }
        let end = world.global_position(actor).unwrap().metres();
        assert!(
            start[0] < 8064. && end[0] > 8066.,
            "Target blocked movement"
        );
        assert_eq!(
            world.targeting_snapshot(actor).unwrap().selected,
            Some(target)
        );
        assert_eq!(world.movement_recoveries(), 0);
    }
}
#[test]
fn selection_validates_identity_and_removal_or_reconnect_cannot_transfer_selection() {
    let mut world = world();
    let target = world.spawn_development_target([8064., 8000.]).unwrap();
    let a = world.join().unwrap();
    let b = world.join().unwrap();
    assert!(world.select_target(a, Some(target)) && world.select_target(b, Some(target)));
    assert!(!world.select_target(a, Some(wrong_epoch(&world, target))));
    assert!(
        !world.select_target(a, Some(world.entity_key(b).unwrap())),
        "A player is not this development NPC"
    );
    assert!(!world.select_target(u64::MAX, Some(target)));
    assert_eq!(world.targeting_snapshot(a).unwrap().selected, Some(target));
    assert!(world.select_target(a, None));
    assert_eq!(world.targeting_snapshot(a).unwrap().selected, None);
    assert_eq!(world.targeting_snapshot(b).unwrap().selected, Some(target));
    assert!(!world.remove_development_target(wrong_epoch(&world, target)));
    world.leave(b);
    let replacement = world.join().unwrap();
    assert_ne!(b, replacement);
    assert!(!world.select_target(b, Some(target)));
    assert_eq!(
        world.targeting_snapshot(replacement).unwrap().selected,
        None
    );
    assert!(world.select_target(replacement, Some(target)));
    assert!(world.remove_development_target(target));
    assert_eq!(
        world.targeting_snapshot(replacement).unwrap().selected,
        None
    );
    assert!(
        world
            .targeting_snapshot(replacement)
            .unwrap()
            .targets
            .is_empty()
    );
    assert!(!world.remove_development_target(target));
    assert!(!world.select_target(replacement, Some(target)));
}
#[test]
fn development_placement_requires_prepared_support_inactive_world_and_bounded_identity_budget() {
    assert!(World::default().spawn_development_target([0., 0.]).is_err());
    let mut world = world();
    for xz in [
        [-1., 8000.],
        [16000., 8000.],
        [f64::NAN, 8000.],
        [12000., 8000.],
    ] {
        assert!(world.spawn_development_target(xz).is_err());
    }
    let first = world.spawn_development_target([8064., 8000.]).unwrap();
    assert_eq!(
        first.entity.value(),
        1,
        "Rejected placement must not consume IDs"
    );
    for _ in 1..MAX_TARGETS {
        world.spawn_development_target([8064., 8000.]).unwrap();
    }
    assert!(world.spawn_development_target([8064., 8000.]).is_err());
    let actor = world.join().unwrap();
    let snapshot = world.targeting_snapshot(actor).unwrap();
    assert_eq!(snapshot.targets.len(), MAX_TARGETS);
    assert!(
        snapshot
            .targets
            .windows(2)
            .all(|pair| pair[0].key.entity.value() < pair[1].key.entity.value())
    );
    assert!(world.spawn_development_target([8064., 8000.]).is_err());
}
#[test]
fn authoritative_distant_teleport_prunes_interest_and_rejects_invisible_selection() {
    let mut world = world();
    let target = world.spawn_development_target([8064., 8000.]).unwrap();
    let actor = world.join().unwrap();
    assert!(world.select_target(actor, Some(target)));
    assert!(world.teleport_development(actor, [12000., 8000.]));
    let deadline = Instant::now() + Duration::from_secs(3);
    while world.global_position(actor).unwrap().metres()[0] < 11000. {
        assert!(Instant::now() < deadline);
        world.poll_terrain();
        std::thread::sleep(Duration::from_millis(1));
    }
    world.step();
    let snapshot = world.targeting_snapshot(actor).unwrap();
    assert!(snapshot.targets.is_empty());
    assert_eq!(snapshot.selected, None);
    assert!(!world.select_target(actor, Some(target)));
    assert_eq!(world.movement_recoveries(), 0);
}
