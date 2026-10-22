//! Normal authoritative terrain path: geometry/support, bounds and recovery.
use serde_json::json;
use theia_core::time::TickRate;
use theia_network::{simulation::World, terrain::Terrain, wire::*};
fn plane(x0: f32, x1: f32, y: f32) -> serde_json::Value {
    json!([
        [[x0, y, -20.], [x1, y, 20.], [x1, y, -20.]],
        [[x0, y, -20.], [x0, y, 20.], [x1, y, 20.]]
    ])
}
fn make_world(triangles: serde_json::Value, solids: serde_json::Value) -> World {
    World::with_terrain(
        Terrain::decode(
            &serde_json::to_vec(&json!({
                "schema":2,"source_origin":[0,0,0],"visual_sha256":vec![0;32],
                "bounds":[[-20,-20],[20,20]],"triangles":triangles,"solids":solids
            }))
            .unwrap(),
        )
        .unwrap(),
    )
}
fn tick(world: &mut World, id: u64, sequence: u64, axes: [i16; 2], jump: bool) -> Player {
    assert!(world.input(
        id,
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence,
            intent: Intent {
                axes,
                sprint: false,
                jump
            }
        }
    ));
    world
        .step()
        .players
        .into_iter()
        .find(|p| p.id == id)
        .unwrap()
}
#[test]
fn normal_terrain_jumps_and_retains_lower_floor_under_an_overpass() {
    let mut triangles = plane(-20., 20., 0.).as_array().unwrap().clone();
    triangles.extend(plane(8., 16., 3.).as_array().unwrap().clone());
    for hz in [20, 30, 60] {
        let mut world = make_world(json!(triangles), json!([]));
        assert!(world.set_tick_rate(TickRate::from_hz(hz).unwrap()));
        let id = world.join().unwrap();
        let mut peak = 0f32;
        for seq in 1..=hz * 3 {
            let p = tick(&mut world, id, seq.into(), [1000, 0], true);
            peak = peak.max(p.y);
            if seq > hz * 2 {
                assert!(p.y < 0.92, "Lower floor must remain selected: {p:?}");
            }
        }
        assert!(peak > 2.1 && peak < 2.2);
        assert_eq!(world.movement_recoveries(), 0);
    }
}
#[test]
fn unsupported_fall_recovers_to_validated_spawn_without_held_jump_retrigger() {
    let mut world = make_world(plane(-20., 8., 0.), json!([]));
    let id = world.join().unwrap();
    let mut jump = false;
    let mut recovered = false;
    for seq in 1..400 {
        let p = tick(&mut world, id, seq, [1000, 0], jump);
        if p.xz[0] > 9. {
            jump = true;
        }
        if world.movement_recoveries() > 0 {
            assert!(jump && p.xz[0].abs() < 0.001 && (p.y - 0.908).abs() < 0.002);
            let next = tick(&mut world, id, seq + 1, [0, 0], true);
            assert!(next.y < 0.92, "Recovery must preserve held jump: {next:?}");
            recovered = true;
            break;
        }
    }
    assert!(recovered, "Unsupported falls need finite recovery");
}
#[test]
fn world_bounds_block_exit_allow_tangential_motion_and_water_stays_solid() {
    let mut world = make_world(plane(-20., 20., 0.), json!([]));
    let id = world.join().unwrap();
    for seq in 1..150 {
        tick(&mut world, id, seq, [1000, 0], false);
    }
    let a = tick(&mut world, id, 150, [1000, 0], false);
    let b = tick(&mut world, id, 151, [1000, 1000], false);
    assert!(a.xz[0] > 19.4 && a.xz[0] < 19.65);
    assert!(
        (b.xz[0] - a.xz[0]).abs() < 0.001 && b.xz[1] > a.xz[1] + 0.11,
        "{a:?} -> {b:?}"
    );
    assert_eq!(world.movement_recoveries(), 0);
    let mut world = make_world(
        plane(-20., 20., 0.),
        json!([{"center":[9,0,0],"size":[1,100,40]}]),
    );
    let id = world.join().unwrap();
    for seq in 1..150 {
        tick(&mut world, id, seq, [1000, 0], true);
    }
    let p = tick(&mut world, id, 150, [1000, 0], true);
    assert!(
        p.xz[0] > 8.1 && p.xz[0] < 8.15,
        "Water exclusion column: {p:?}"
    );
    assert_eq!(world.movement_recoveries(), 0);
}

#[test]
#[ignore = "requires private prepared map packages; run with THEIA_MAP_CONTROLLER_FIXTURES"]
fn prepared_map_routes_use_normal_authority_without_recovery() {
    let paths =
        std::env::var_os("THEIA_MAP_CONTROLLER_FIXTURES").expect("Explicit map directories");
    for directory in std::env::split_paths(&paths) {
        let access: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("access.json")).unwrap()).unwrap();
        let goals: Vec<[f32; 2]> = serde_json::from_value(access["route"].clone()).unwrap();
        for hz in [20, 30, 60] {
            let mut world =
                World::with_terrain(Terrain::load(&directory.join("terrain.json")).unwrap());
            assert!(world.set_tick_rate(TickRate::from_hz(hz).unwrap()));
            let id = world.join().unwrap();
            let mut p = world.step().players[0].clone();
            let initial_y = p.y;
            let mut stage = 0;
            let mut wall_ticks = 0;
            let mut samples = Vec::new();
            for seq in 1u64..=2000 {
                let goal = goals[stage.min(5)];
                let delta = [(goal[0] - p.xz[0]) / 0.5, (goal[1] - p.xz[1]) / 0.5];
                let distance = delta[0].hypot(delta[1]);
                if stage == 2 {
                    wall_ticks += 1;
                }
                if (stage == 2 && wall_ticks > hz * 5 / 2) || (stage != 2 && distance < 0.24) {
                    samples.push(p.clone());
                    stage += 1;
                    if stage == 6 {
                        break;
                    }
                    continue;
                }
                let scale = 1000. / distance.max(1.);
                p = tick(
                    &mut world,
                    id,
                    seq,
                    [(delta[0] * scale) as i16, (delta[1] * scale) as i16],
                    false,
                );
                assert_eq!(
                    world.movement_recoveries(),
                    0,
                    "{} at {hz} Hz stage {stage}: {p:?}",
                    directory.display()
                );
            }
            assert_eq!(stage, 6, "{} at {hz} Hz: {p:?}", directory.display());
            assert!(samples[1].y > initial_y + 0.5);
            assert!(samples[2].applied_sequence > samples[1].applied_sequence + u64::from(hz));
            println!(
                "map_motion path={} hz={hz} stages={stage} recoveries={} samples={samples:?}",
                directory.display(),
                world.movement_recoveries()
            );
        }
    }
}

#[test]
fn world_envelope_is_idempotent_and_chunk_collision_keeps_open_seams() {
    use avian3d::prelude::Collider;
    use bevy::prelude::Vec3;
    use theia_network::physics::{GROUND, SOLID};
    let data = serde_json::to_vec(&json!({"schema":2,"source_origin":[0,0,0],
        "visual_sha256":vec![0;32],"bounds":[[-20,-20],[20,20]],
        "triangles":plane(-20.,20.,0.)}))
    .unwrap();
    let chunk = Terrain::decode(&data).unwrap();
    let identity = chunk.identity;
    let body = Collider::capsule(0.35, 1.1);
    let origin = Vec3::new(19., 0.908, 0.);
    let delta = Vec3::X * 2.;
    assert!(
        chunk
            .physics
            .sweep(&body, origin, delta, GROUND | SOLID, 0.005)
            .is_none(),
        "An individual chunk must not block its ownership seam"
    );
    let bounded = chunk.into_world().into_world();
    assert_eq!(bounded.identity, identity);
    assert_eq!(bounded.physics.colliders.len(), 5);
    let hit = bounded
        .physics
        .sweep(&body, origin, delta, GROUND | SOLID, 0.005)
        .unwrap();
    assert!((hit.distance - 0.645).abs() < 0.001, "{hit:?}");
}
