//! Trusted restore admission validates prepared static world collision; no DB mocks.
use std::{
    thread,
    time::{Duration, Instant},
};
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::{
    procedural::{self, Patch},
    simulation::World,
    wire::MAX_PLAYERS,
};
fn body(patch: &Patch, x: f64, z: f64) -> GlobalPosition {
    let mut p = patch.try_spawn(x, z).unwrap().metres();
    p[1] += f64::from(theia_network::world::PAWN_Y);
    GlobalPosition::new(p).unwrap()
}
#[test]
fn restored_positions_require_prepared_supported_finite_geometry_and_fresh_runtime_identity() {
    let mut first = World::with_procedural([8000., 8000.], 1.).unwrap();
    let initial = first.development_player_spawn(1).unwrap();
    let previous = first.join_restored(initial).unwrap();
    let old_key = first.entity_key(previous).unwrap();
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let mut airborne = initial.metres();
    airborne[1] += 0.5;
    let airborne = GlobalPosition::new(airborne).unwrap();
    let id = world.join_restored(airborne).unwrap();
    assert_eq!(world.global_position(id), Some(airborne));
    assert_ne!(world.entity_key(id).unwrap(), old_key);
    let correction = world.correction(id).unwrap();
    assert_eq!(correction.state.input.stream_epoch, 1);
    assert_eq!(correction.state.input.applied_sequence, 0);
    let far = body(&Patch::build(ChunkId { x: 120, z: 100 }), 15400., 12860.);
    assert!(
        world.join_restored(far).is_none(),
        "Unprepared geometry must never admit a saved pose"
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while !world.terrain_prepared_for(far) {
        world.poll_terrain_keeping(&[far]).unwrap();
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
    let second = world.join_restored(far).unwrap();
    assert_eq!(world.global_position(second), Some(far));
    assert_eq!(
        world
            .snapshot_for(second)
            .players
            .iter()
            .filter(|p| p.id == second)
            .count(),
        1
    );
    world.poll_terrain();
    assert!(world.terrain_prepared_for(far));
    for _ in 0..10 {
        world.step();
    }
    assert!((world.global_position(second).unwrap().metres()[0] - far.metres()[0]).abs() < 0.001);
    for position in [
        [-1., 0., 1.],
        [8000., -1000., 8000.],
        [8000., 1000., 8000.],
        [0.01, 0., 8000.],
    ] {
        let position = GlobalPosition::new(position).unwrap();
        assert!(world.join_restored(position).is_none());
    }
    assert!(
        world
            .poll_terrain_keeping(&[initial; MAX_PLAYERS + 1])
            .is_err()
    );
    assert!(
        world
            .poll_terrain_keeping(&[GlobalPosition::new([-1., 0., 0.]).unwrap()])
            .is_err()
    );
    assert_eq!(procedural::owner(far), Some(ChunkId { x: 120, z: 100 }));
}
#[test]
fn fixture_slot_spawns_stay_supported_near_finite_edges_without_changing_normal_join() {
    for spawn in [[1., 1.], [15999., 15999.], [8000., 8000.]] {
        let mut world = World::with_procedural(spawn, 1.).unwrap();
        for slot in 1..=MAX_PLAYERS as u8 {
            let position = world.development_player_spawn(slot).unwrap();
            world.validate_restored_player(position).unwrap();
        }
        assert!(world.development_player_spawn(0).is_none());
        assert!(world.development_player_spawn(9).is_none());
        assert!(world.join().is_some());
    }
}
