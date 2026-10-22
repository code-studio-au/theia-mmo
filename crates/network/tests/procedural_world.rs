//! Actual Avian movement/streaming against the large procedural world, independent
//! of renderer visibility. Native client qualification covers that separately.
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    time::TickRate,
};
use theia_network::{
    procedural as terrain,
    simulation::World,
    wire::{ChunkReady, Input, Intent, Snapshot, VERSION, Welcome, encode},
};

fn ready(world: &mut World, id: u64, sequence: u64) {
    let owner = terrain::owner(world.global_position(id).unwrap()).unwrap();
    assert!(
        world.client_ready(
            id,
            &ChunkReady {
                version: VERSION,
                epoch: world.epoch(),
                sequence,
                chunks: terrain::neighbours(owner, 1)
                    .iter()
                    .map(|id| [id.x, id.z])
                    .collect()
            }
        )
    );
}
fn drive(world: &mut World, id: u64, sequence: u64, axes: [i16; 2], jump: bool) {
    world.poll_terrain();
    // Simulated time must not outrun real asynchronous preparation by thousands
    // of ticks; native qualification measures actual continuous travel stalls.
    let owner = terrain::owner(world.global_position(id).unwrap()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !world.procedural().unwrap().patches.contains_key(&owner) {
        assert!(
            std::time::Instant::now() < deadline,
            "Terrain worker did not prepare current owner"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
        world.poll_terrain();
    }
    ready(world, id, sequence);
    assert!(world.input(
        id,
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence,
            intent: Intent {
                axes,
                sprint: true,
                jump
            }
        }
    ));
    world.step();
}
#[test]
fn all_125_chunk_edges_share_identical_samples_and_the_world_is_exactly_256_km2() {
    assert_eq!(125 * 128, 16000);
    for n in 0..124 {
        for z in [0, 62, 124] {
            let a = terrain::geometry(ChunkId { x: n, z }, 32);
            let b = terrain::geometry(ChunkId { x: n + 1, z }, 32);
            for row in 0..=32 {
                assert_eq!(a.positions[row * 33 + 32][1], b.positions[row * 33][1]);
            }
        }
        let a = terrain::geometry(ChunkId { x: 62, z: n }, 32);
        let b = terrain::geometry(ChunkId { x: 62, z: n + 1 }, 32);
        for x in 0..=32 {
            assert_eq!(a.positions[32 * 33 + x][1], b.positions[x][1]);
        }
    }
    assert!(terrain::owner(GlobalPosition::new([16000., 0., 10.]).unwrap()).is_none());
    assert!(terrain::owner(GlobalPosition::new([-0.001, 0., 10.]).unwrap()).is_none());
}
#[test]
fn actual_collision_matches_visual_triangle_planes_at_distant_world_locations() {
    for owner in [
        ChunkId { x: 0, z: 0 },
        ChunkId { x: 62, z: 62 },
        ChunkId { x: 124, z: 124 },
    ] {
        let patch = terrain::Patch::build(owner);
        let geometry = terrain::geometry(owner, 32);
        for triangle in geometry.triangles.iter().step_by(41) {
            let vertices = triangle.map(|i| geometry.positions[i as usize]);
            let centroid: [f32; 3] =
                std::array::from_fn(|axis| vertices.iter().map(|v| v[axis]).sum::<f32>() / 3.);
            let hit = patch
                .scene
                .ground([centroid[0], centroid[2]], 100.)
                .unwrap();
            assert!((hit.0 - centroid[1]).abs() < 0.0001);
        }
    }
}
#[test]
fn readiness_freezes_unloaded_travel_and_rejects_stale_or_foreign_reports() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    let id = world.join().unwrap();
    let before = world.global_position(id).unwrap();
    for sequence in 1..=30 {
        assert!(world.input(
            id,
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
        world.step();
    }
    let held = world.global_position(id).unwrap().metres();
    assert_eq!(before.metres()[0], held[0]);
    assert_eq!(before.metres()[2], held[2]);
    assert!(
        (before.metres()[1] - held[1]).abs() < 0.002,
        "Vertical support must continue while XZ is held"
    );
    ready(&mut world, id, 31);
    assert!(!world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 31,
            chunks: vec![]
        }
    ));
    assert!(!world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 32,
            chunks: vec![[125, 0]]
        }
    ));
    for seq in 32..=62 {
        drive(&mut world, id, seq, [1000, 0], false);
    }
    assert!(world.global_position(id).unwrap().metres()[0] > 8007.);
}
#[test]
fn full_continent_crossing_return_and_teleports_keep_physics_and_replication_bounded() {
    let mut world = World::with_procedural([10., 8000.], 32.).unwrap();
    let id = world.join().unwrap();
    let welcome = Welcome {
        id,
        epoch: world.epoch(),
        world: world.identity(),
        origin: world.origin(),
    };
    let mut sequence = 0;
    for target in [15980., 20.] {
        let direction = if target > 8000. { 1000 } else { -1000 };
        for _ in 0..2600 {
            sequence += 1;
            drive(&mut world, id, sequence, [direction, 0], false);
            let position = world.global_position(id).unwrap();
            let snapshot = world.snapshot_for(id);
            let decoded = Snapshot::decode(&encode(&snapshot).unwrap()).unwrap();
            assert!(welcome.accepts(&decoded));
            let received = decoded
                .global_position(&decoded.players[0])
                .unwrap()
                .metres();
            for (a, b) in position.metres().into_iter().zip(received) {
                assert!((a - b).abs() < 0.0001);
            }
            assert!(world.procedural().unwrap().patches.len() <= 4);
            let x = position.metres()[0];
            if (direction > 0 && x >= target) || (direction < 0 && x <= target) {
                break;
            }
            // Let the bounded real worker progress even on fast test CPUs.
            if sequence % 8 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        let x = world.global_position(id).unwrap().metres()[0];
        assert!(
            (x - target).abs() < 20.,
            "Traversal stalled at {x}, target {target}"
        );
    }
    for target in [[15900., 15900.], [100., 100.], [8000., 8000.], [10., 8000.]] {
        assert!(world.teleport_development(id, target));
        for _ in 0..2000 {
            world.poll_terrain();
            let p = world.global_position(id).unwrap().metres();
            if (p[0] - target[0]).abs() < 0.1 && (p[2] - target[1]).abs() < 0.1 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let p = world.global_position(id).unwrap().metres();
        assert!((p[0] - target[0]).abs() < 0.1 && (p[2] - target[1]).abs() < 0.1);
        for _ in 0..3 {
            sequence += 1;
            drive(&mut world, id, sequence, [0, 0], false);
        }
    }
    assert_eq!(world.movement_recoveries(), 0);
    let runtime = world.procedural().unwrap();
    assert!(runtime.generated > 100 && runtime.retired > 100);
    eprintln!(
        "full_continent generated={} retired={} resident={} recoveries={}",
        runtime.generated,
        runtime.retired,
        runtime.patches.len(),
        world.movement_recoveries()
    );
}
#[test]
fn supported_jumps_and_chunk_crossings_work_at_all_configured_tick_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = World::with_procedural([127., 127.], 1.).unwrap();
        world.set_tick_rate(rate);
        let id = world.join().unwrap();
        let mut peak = 0f64;
        let start_y = world.global_position(id).unwrap().metres()[1];
        for sequence in 1..=u64::from(rate.hz()) * 5 {
            drive(
                &mut world,
                id,
                sequence,
                [1000, 1000],
                sequence > 5 && sequence < 12,
            );
            peak = peak.max(world.global_position(id).unwrap().metres()[1] - start_y);
        }
        let p = world.global_position(id).unwrap().metres();
        assert!(p[0] > 150. && p[2] > 150.);
        assert!(peak > 0.5);
        assert_eq!(world.movement_recoveries(), 0);
    }
}

#[test]
fn readiness_withdrawal_holds_horizontal_motion_but_keeps_airborne_gravity_and_landing() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    let id = world.join().unwrap();
    for sequence in 1..=5 {
        drive(&mut world, id, sequence, [0, 0], false);
    }
    drive(&mut world, id, 6, [0, 0], true);
    let takeoff = world.global_position(id).unwrap().metres();
    assert!(world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 7,
            chunks: vec![]
        }
    ));
    let mut peak = takeoff[1];
    for sequence in 8..=97 {
        assert!(world.input(
            id,
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
        world.step();
        let p = world.global_position(id).unwrap().metres();
        assert!((p[0] - takeoff[0]).abs() < 0.00001);
        assert!((p[2] - takeoff[2]).abs() < 0.00001);
        peak = peak.max(p[1]);
    }
    let landed = world.global_position(id).unwrap().metres();
    assert!(peak > takeoff[1] + 0.3);
    assert!(landed[1] < takeoff[1] - 0.1);
    assert_eq!(world.movement_recoveries(), 0);
}
