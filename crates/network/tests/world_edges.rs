#[path = "support/chunk_fixture.rs"]
mod fixture;
use bevy::prelude::Vec3;
use fixture::Fixture;
use serde_json::json;
use theia_core::{
    coordinates::{GlobalPosition, LocalPosition},
    time::TickRate,
};
use theia_network::{
    chunk_world::{ChunkWorld, Config},
    kinematic::{Boundary, FOOTPRINT_RADIUS, Motion},
    simulation::World,
    wire::Intent,
    world_bounds::WorldBounds,
};

fn bounds(f: &Fixture) -> WorldBounds {
    let o = f.index().frame().origin().metres();
    WorldBounds::new([[o[0] - 96., o[2] + 8.], [o[0] + 126.1, o[2] + 120.]]).unwrap()
}
#[test]
fn finite_bounds_sweeps_cover_every_edge_corner_and_leave_vertical_motion_free() {
    for base in [-7_812_500, 0, 7_812_500] {
        let f = Fixture::new(base, false, false);
        let frame = f.index().frame();
        let bounds = bounds(&f);
        let local = bounds.in_frame(frame).unwrap();
        assert!(
            local
                .sweep(Vec3::new(16., 2., 32.), Vec3::NEG_Y * 200.)
                .is_none()
        );
        for [x, z] in [
            [-1, 0],
            [1, 0],
            [0, -1],
            [0, 1],
            [-1, -1],
            [-1, 1],
            [1, -1],
            [1, 1],
        ] {
            let delta = Vec3::new(x as f32, 0., z as f32).normalize() * 1000.;
            let start = Vec3::new(16., 2., 32.);
            let hit = local.sweep(start, delta).unwrap();
            assert!(hit.normal.dot(delta) < 0.);
            let at = start + delta.normalize() * hit.distance;
            let global = frame
                .to_global(LocalPosition::new(at.to_array()).unwrap())
                .unwrap();
            // Sweep arithmetic can round the contact point by one local ULP.
            let p = bounds.clamp(global);
            let g = global.metres();
            let p = p.metres();
            assert!((g[0] - p[0]).abs() < 0.0001 && (g[2] - p[2]).abs() < 0.0001);
        }
    }
    assert!(WorldBounds::new([[0., 0.], [0.5, 20.]]).is_none());
    assert!(WorldBounds::new([[0., 0.], [f64::NAN, 20.]]).is_none());
    assert!(WorldBounds::new([[0., 0.], [1e15, 20.]]).is_none());
}
#[test]
fn real_avian_motion_stops_at_all_edges_and_corners_with_jump_gravity_and_retreat() {
    for base in [-7_812_500, 0, 7_812_500] {
        let f = Fixture::new(base, false, false);
        let bounds = bounds(&f);
        let frame = f.index().frame();
        for hz in [20, 30, 60] {
            for [dx, dz] in [
                [-1, 0],
                [1, 0],
                [0, -1],
                [0, 1],
                [-1, -1],
                [-1, 1],
                [1, -1],
                [1, 1],
            ] {
                let mut world = ChunkWorld::with_bounds(
                    f.residency(&[-1, 0]),
                    &[f.point(16., 0.)],
                    -18.,
                    Some(bounds),
                )
                .unwrap();
                let x = match dx {
                    -1 => -95.,
                    1 => 125.,
                    _ => 16.,
                };
                let z = match dz {
                    -1 => 9.,
                    1 => 119.,
                    _ => 32.,
                };
                let mut feet = LocalPosition::new([x, 0.008, z]).unwrap();
                let mut body = Motion::default();
                let mut peak = 0f32;
                for tick in 0..hz * 3 {
                    feet = world
                        .advance(
                            &mut body,
                            feet,
                            Intent {
                                axes: [dx * 1000, dz * 1000],
                                jump: (1..10).contains(&tick),
                                sprint: false,
                            },
                            TickRate::from_hz(hz).unwrap(),
                        )
                        .unwrap();
                    assert!(
                        bounds.contains(frame.to_global(feet).unwrap()),
                        "{base}/{hz}/{dx},{dz}: {:?}",
                        feet.metres()
                    );
                    peak = peak.max(feet.metres()[1]);
                }
                assert!(peak > 1., "Jump suppressed at {hz}/{dx},{dz}: {peak}");
                assert!(body.grounded());
                assert!(world.blocked() > 0);
                let stopped = feet.metres();
                for _ in 0..hz / 2 {
                    feet = world
                        .advance(
                            &mut body,
                            feet,
                            Intent {
                                axes: [-dx * 1000, -dz * 1000],
                                jump: false,
                                sprint: false,
                            },
                            TickRate::from_hz(hz).unwrap(),
                        )
                        .unwrap();
                }
                let now = feet.metres();
                assert!((now[0] - stopped[0]).hypot(now[2] - stopped[2]) > 1.);
            }
        }
    }
}
#[test]
fn metadata_binds_finite_bounds_and_retains_legacy_identity_and_open_internal_seams() {
    let f = Fixture::new(7_812_500, false, false);
    let bounds = bounds(&f);
    let mut document = json!({"schema":1,"resident_budget":2,"initial_chunks":[[f.base-1,f.base],[f.base,f.base]],"spawns":[f.point(16.,0.).metres()],"fall_plane":-18.});
    let path = f.root.join("world.session.json");
    let write =
        |v: &serde_json::Value| std::fs::write(&path, serde_json::to_vec(v).unwrap()).unwrap();
    write(&document);
    let legacy = Config::load(&f.root).unwrap();
    assert_eq!(legacy.identity(), f.world(&[-1, 0], 16.).identity());
    document["world_bounds"] = json!(bounds.metres());
    write(&document);
    assert!(Config::load(&f.root).is_err());
    document["schema"] = json!(2);
    write(&document);
    let config = Config::load(&f.root).unwrap();
    assert_ne!(config.identity(), legacy.identity());
    assert_eq!(
        World::with_chunk_config(&config).unwrap().identity(),
        config.identity()
    );
    let index_path = f.root.join("world.chunks.json");
    let original_index = std::fs::read(&index_path).unwrap();
    let mut shifted: serde_json::Value = serde_json::from_slice(&original_index).unwrap();
    let origin = f.index().frame().origin().metres();
    shifted["query_origin"] = json!([origin[0] + 8., 25., origin[2] - 8.]);
    std::fs::write(&index_path, serde_json::to_vec(&shifted).unwrap()).unwrap();
    let shifted = Config::load(&f.root).unwrap();
    assert_eq!(shifted.identity(), config.identity());
    assert_eq!(
        World::with_chunk_config(&shifted).unwrap().identity(),
        config.identity()
    );
    std::fs::write(&index_path, original_index).unwrap();
    let mut changed = document.clone();
    changed["world_bounds"][1][0] = json!(bounds.metres()[1][0] - 1.);
    write(&changed);
    assert_ne!(Config::load(&f.root).unwrap().identity(), config.identity());
    let mut world = ChunkWorld::with_bounds(
        f.residency(&[-1, 0]),
        &[f.point(16., 0.)],
        -18.,
        Some(bounds),
    )
    .unwrap();
    let mut feet = LocalPosition::new([-1., 0.008, 32.]).unwrap();
    let mut body = Motion::default();
    for _ in 0..30 {
        feet = world
            .advance(
                &mut body,
                feet,
                Intent {
                    axes: [1000, 0],
                    jump: false,
                    sprint: false,
                },
                TickRate::from_hz(30).unwrap(),
            )
            .unwrap();
    }
    assert!(feet.metres()[0] > 2.);
    assert_eq!(world.blocked(), 0);
    let edge = GlobalPosition::new([
        bounds.metres()[1][0] - f64::from(FOOTPRINT_RADIUS) - 0.01,
        0.,
        bounds.metres()[0][1] + 24.,
    ])
    .unwrap();
    let demand = world
        .demand(&[(
            edge,
            Intent {
                axes: [1000, 0],
                jump: false,
                sprint: false,
            },
            true,
        )])
        .unwrap();
    assert_eq!(demand, vec![f.id(-1), f.id(0)]);
    let mut bad = document.clone();
    bad["world_bounds"] = json!([[0., 0.], [20., 20.]]);
    write(&bad);
    assert!(Config::load(&f.root).is_err());
    let mut bad = document;
    bad["spawns"] = json!([[bounds.metres()[1][0], 0., bounds.metres()[0][1] + 24.]]);
    write(&bad);
    assert!(Config::load(&f.root).is_err());
}

#[test]
fn unsupported_fall_recovers_inside_finite_world_without_retriggering_held_jump() {
    use theia_network::{
        content::Manifest,
        terrain::sha256,
        wire::{Input, VERSION},
    };
    let f = Fixture::new(-7_812_500, false, false);
    // Keep the cell resident but remove floor beyond x=24. This distinguishes
    // a real unsupported fall inside the finite world from a loading boundary.
    let id = f.id(0);
    let directory = f.root.join(format!("chunk_{}_{}", id.x, id.z));
    let path = directory.join("terrain.json");
    let mut profile: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    profile["triangles"] = json!([
        [[-1., 0., -1.], [-1., 0., 129.], [24., 0., -1.]],
        [[24., 0., -1.], [-1., 0., 129.], [24., 0., 129.]]
    ]);
    let bytes = serde_json::to_vec(&profile).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let manifest_path = directory.join("world.manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["collision_bytes"] = json!(bytes.len());
    manifest["collision_sha256"] = json!(sha256(&bytes));
    let bytes = serde_json::to_vec(&manifest).unwrap();
    std::fs::write(manifest_path, &bytes).unwrap();
    let package = Manifest::decode(&bytes).unwrap().identity();
    let index_path = f.root.join("world.chunks.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&index_path).unwrap()).unwrap();
    for entry in index["chunks"].as_array_mut().unwrap() {
        if entry["id"] == json!([id.x, id.z]) {
            entry["package"] = json!(package);
        }
    }
    std::fs::write(index_path, serde_json::to_vec(&index).unwrap()).unwrap();
    let bounds = bounds(&f);
    let document = json!({"schema":2,"resident_budget":2,"initial_chunks":[[f.base-1,f.base],[f.base,f.base]],"spawns":[f.point(16.,0.).metres()],"fall_plane":-18.,"world_bounds":bounds.metres()});
    std::fs::write(
        f.root.join("world.session.json"),
        serde_json::to_vec(&document).unwrap(),
    )
    .unwrap();
    let config = Config::load(&f.root).unwrap();
    for hz in [20, 30, 60] {
        let mut world = World::with_chunk_config(&config).unwrap();
        assert!(world.set_tick_rate(TickRate::from_hz(hz).unwrap()));
        let actor = world.join().unwrap();
        let mut sequence = 0;
        for _ in 0..hz * 10 {
            sequence += 1;
            assert!(world.input(
                actor,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [1000, 0],
                        jump: true,
                        sprint: false
                    }
                }
            ));
            world.step();
            if world.movement_recoveries() > 0 {
                break;
            }
        }
        assert_eq!(
            world.movement_recoveries(),
            1,
            "{hz}: falling inside resident cells must recover"
        );
        let recovered = world.global_position(actor).unwrap();
        assert!(bounds.contains(recovered));
        assert!((recovered.metres()[0] - f.point(16., 0.).metres()[0]).abs() < 0.001);
        for _ in 0..hz {
            sequence += 1;
            assert!(world.input(
                actor,
                Input {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence,
                    intent: Intent {
                        axes: [0, 0],
                        jump: true,
                        sprint: false
                    }
                }
            ));
            world.step();
            assert!(
                (world.global_position(actor).unwrap().metres()[1] - recovered.metres()[1]).abs()
                    < 0.01
            );
        }
        assert_eq!(world.movement_recoveries(), 1);
    }
}
