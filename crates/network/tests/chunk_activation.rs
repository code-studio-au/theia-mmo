#[path = "support/chunk_fixture.rs"]
mod chunk_fixture;
use chunk_fixture::Fixture;
use serde_json::json;
use theia_core::time::TickRate;
use theia_network::{
    chunk_world::{ChunkWorld, Occupant},
    simulation::World,
    wire::{Input, Intent, VERSION},
};
use theia_network::{content::Manifest, terrain::sha256};
fn add_triangles(fixture: &Fixture, offset: i64, triangles: Vec<[[f32; 3]; 3]>) {
    let id = fixture.id(offset);
    let directory = fixture.root.join(format!("chunk_{}_{}", id.x, id.z));
    let path = directory.join("terrain.json");
    let mut profile: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    profile["triangles"]
        .as_array_mut()
        .unwrap()
        .extend(triangles.into_iter().map(|t| json!(t)));
    let bytes = serde_json::to_vec(&profile).unwrap();
    std::fs::write(path, &bytes).unwrap();
    let path = directory.join("world.manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    manifest["collision_bytes"] = json!(bytes.len());
    manifest["collision_sha256"] = json!(sha256(&bytes));
    let bytes = serde_json::to_vec(&manifest).unwrap();
    std::fs::write(path, &bytes).unwrap();
    let identity = Manifest::decode(&bytes).unwrap().identity();
    let path = fixture.root.join("world.chunks.json");
    let mut index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for entry in index["chunks"].as_array_mut().unwrap() {
        if entry["id"] == json!([id.x, id.z]) {
            entry["package"] = json!(identity);
        }
    }
    std::fs::write(path, serde_json::to_vec(&index).unwrap()).unwrap();
}

fn step(world: &mut World, id: u64, seq: u64, axes: [i16; 2], jump: bool) -> [f32; 3] {
    assert!(world.input(
        id,
        Input {
            version: VERSION,
            epoch: world.epoch(),
            sequence: seq,
            intent: Intent {
                axes,
                jump,
                sprint: false
            }
        }
    ));
    let s = world.step();
    let player = s.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.applied_sequence, seq);
    let p = s.global_position(player).unwrap();
    let frame = world.chunks().unwrap().frame();
    frame.to_local(p).unwrap().metres()
}
#[test]
fn live_actor_capsule_prevents_prepared_unload_then_allows_safe_replacement() {
    let f = Fixture::new(0, false, false);
    let mut w = f.world(&[-1, 0], 2.);
    let id = w.join().unwrap();
    let plan = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap();
    let candidate = std::thread::spawn(move || plan.load().unwrap())
        .join()
        .unwrap();
    // The actor moves after planning. Its centre remains in chunk 0, but its
    // capsule crosses into -1: protecting only the owner would accept this.
    let mut seq = 0;
    let p = (0..30)
        .find_map(|_| {
            seq += 1;
            let p = step(&mut w, id, seq, [-1000, 0], false);
            (p[0] < 0.3).then_some(p)
        })
        .expect("Actor must reach the loaded seam within one second");
    assert!(p[0] > 0.);
    let before = w.global_position(id);
    assert!(w.activate_chunks(candidate).is_err());
    assert_eq!(w.global_position(id), before);
    assert_eq!(w.chunks().unwrap().residency().generation(), 1);
    assert_eq!(w.chunks().unwrap().residency().metrics().loads, 2);
    assert_eq!(w.chunks().unwrap().rejected(), 1);
    for _ in 0..12 {
        seq += 1;
        step(&mut w, id, seq, [1000, 0], false);
    }
    let candidate = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap().load().unwrap();
    let before = w.global_position(id);
    w.activate_chunks(candidate).unwrap();
    assert_eq!(w.global_position(id), before);
    assert_eq!(w.chunks().unwrap().residency().generation(), 2);
    seq += 1;
    let s = step(&mut w, id, seq, [1000, 0], false);
    assert!(s[0] > 2.);
    assert_eq!(w.movement_recoveries(), 0);
}
#[test]
fn missing_neighbor_holds_horizontal_motion_but_keeps_jump_gravity_and_input_progress() {
    for base in [-7_812_500, 0, 7_812_500] {
        for hz in [20, 30, 60] {
            let f = Fixture::new(base, false, false);
            let mut w = f.world(&[-1, 0], 126.);
            assert!(w.set_tick_rate(TickRate::from_hz(hz).unwrap()));
            let id = w.join().unwrap();
            let mut seq = 0;
            for _ in 0..hz {
                seq += 1;
                step(&mut w, id, seq, [1000, 0], false);
            }
            let x = w
                .chunks()
                .unwrap()
                .frame()
                .to_local(w.global_position(id).unwrap())
                .unwrap()
                .metres()[0];
            assert!((127.2..127.646).contains(&x), "{hz}: {x}");
            let z0 = w.global_position(id).unwrap().metres()[2];
            for _ in 0..hz {
                seq += 1;
                step(&mut w, id, seq, [1000, 1000], false);
            }
            let diagonal_z = w.global_position(id).unwrap().metres()[2] - z0;
            assert!(
                (3.52..3.55).contains(&diagonal_z),
                "Diagonal fallback changed speed: {diagonal_z}"
            );
            let y0 = w.global_position(id).unwrap().metres()[1];
            let mut ymax = y0;
            for _ in 0..hz {
                seq += 1;
                step(&mut w, id, seq, [1000, 0], true);
                ymax = ymax.max(w.global_position(id).unwrap().metres()[1]);
            }
            assert!((1.1..1.25).contains(&(ymax - y0)));
            assert!((w.global_position(id).unwrap().metres()[1] - y0).abs() < 0.02);
            let candidate = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap().load().unwrap();
            w.activate_chunks(candidate).unwrap();
            for _ in 0..hz {
                seq += 1;
                step(&mut w, id, seq, [1000, 0], false);
            }
            let x = w
                .chunks()
                .unwrap()
                .frame()
                .to_local(w.global_position(id).unwrap())
                .unwrap()
                .metres()[0];
            assert!(x > 132., "{hz}: {x}");
            assert_eq!(w.movement_recoveries(), 0);
            assert!(w.chunks().unwrap().blocked() > 0);
        }
    }
}
#[test]
fn publication_rejects_body_intrusion_and_grounded_or_airborne_support_removal() {
    for (raised, wall) in [(true, false), (false, true)] {
        let f = Fixture::new(0, raised, wall);
        let mut chunks = ChunkWorld::new(f.residency(&[-1, 0]), &[f.point(16., 0.)], -18.).unwrap();
        for grounded in [false, true] {
            let y = if raised {
                if grounded { 2.008 } else { 3. }
            } else {
                0.008
            };
            let candidate = chunks.plan(7, &[f.id(0), f.id(1)]).unwrap().load().unwrap();
            let error = chunks
                .activate(
                    7,
                    candidate,
                    &[Occupant {
                        feet: f.point(45., y),
                        grounded,
                    }],
                )
                .unwrap_err()
                .to_string();
            if wall {
                assert!(error.contains("penetrates"), "{error}");
            } else if grounded {
                assert!(error.contains("grounded"), "{error}");
            } else {
                assert!(error.contains("landing"), "{error}");
            }
            assert_eq!(chunks.residency().generation(), 1);
            assert_eq!(chunks.residency().loaded(), vec![f.id(-1), f.id(0)]);
        }
    }
}
#[test]
fn stale_generation_foreign_epoch_and_unloading_recovery_spawns_fail_closed() {
    let f = Fixture::new(0, false, false);
    let mut w = f.world(&[-1, 0], 16.);
    let a = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap().load().unwrap();
    let stale = w.plan_chunks(&[f.id(0)]).unwrap().load().unwrap();
    w.activate_chunks(a).unwrap();
    assert!(
        w.activate_chunks(stale)
            .unwrap_err()
            .to_string()
            .contains("Stale")
    );
    let other = f.world(&[0, 1], 16.);
    let foreign = other.plan_chunks(&[f.id(0)]).unwrap().load().unwrap();
    assert!(
        w.activate_chunks(foreign)
            .unwrap_err()
            .to_string()
            .contains("foreign")
    );
    let removed = w.plan_chunks(&[f.id(1)]).unwrap().load().unwrap();
    assert!(
        w.activate_chunks(removed)
            .unwrap_err()
            .to_string()
            .contains("spawn")
    );
    assert_eq!(w.chunks().unwrap().residency().generation(), 2);
    assert_eq!(w.chunks().unwrap().rejected(), 3);
    assert!(!w.set_origin(f.point(0., 0.)));
}
#[test]
fn failed_worker_and_candidate_release_leave_live_authority_usable() {
    let f = Fixture::new(0, false, false);
    let mut w = f.world(&[-1, 0], 16.);
    let id = w.join().unwrap();
    let identity = w.identity();
    let abandoned = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap().load().unwrap();
    drop(abandoned);
    assert_eq!(w.chunks().unwrap().residency().generation(), 1);
    let plan = w.plan_chunks(&[f.id(0), f.id(1)]).unwrap();
    std::fs::write(f.root.join("chunk_1_0/terrain.json"), b"broken").unwrap();
    assert!(
        std::thread::spawn(move || plan.load())
            .join()
            .unwrap()
            .is_err()
    );
    assert_eq!(w.identity(), identity);
    for seq in 1..=30 {
        step(&mut w, id, seq, [1000, 0], false);
    }
    let p = w
        .chunks()
        .unwrap()
        .frame()
        .to_local(w.global_position(id).unwrap())
        .unwrap();
    assert!(p.metres()[0] > 20.9);
    assert_eq!(w.chunks().unwrap().residency().generation(), 1);
    assert_eq!(w.movement_recoveries(), 0);
}

#[test]
fn logical_identity_ignores_entry_order_and_query_origin_but_activation_checks_frame() {
    let f = Fixture::new(0, false, false);
    let spawn = f.point(16., 0.);
    let mut original = ChunkWorld::new(f.residency(&[-1, 0]), &[spawn], -18.).unwrap();
    let path = f.root.join("world.chunks.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    document["query_origin"] = serde_json::json!([8., 0., 0.]);
    document["chunks"].as_array_mut().unwrap().reverse();
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let relocated = ChunkWorld::new(f.residency(&[-1, 0]), &[spawn], -18.).unwrap();
    assert_eq!(original.identity(), relocated.identity());
    let candidate = relocated
        .plan(7, &[f.id(0), f.id(1)])
        .unwrap()
        .load()
        .unwrap();
    assert!(
        original
            .activate(7, candidate, &[])
            .unwrap_err()
            .to_string()
            .contains("foreign")
    );
    let changed_spawn = ChunkWorld::new(f.residency(&[-1, 0]), &[f.point(17., 0.)], -18.).unwrap();
    let changed_plane = ChunkWorld::new(f.residency(&[-1, 0]), &[spawn], -19.).unwrap();
    assert_ne!(original.identity(), changed_spawn.identity());
    assert_ne!(original.identity(), changed_plane.identity());
}

#[test]
fn shared_startup_metadata_bounds_configuration_and_validates_initial_spawn_geometry() {
    use serde_json::json;
    use theia_network::chunk_world::Config;
    let f = Fixture::new(-7_812_500, false, false);
    let initial = json!({"schema":1,"resident_budget":2,"initial_chunks":[[f.base-1,f.base],[f.base,f.base]],"spawns":[f.point(16.,0.).metres()],"fall_plane":-18.});
    let path = f.root.join("world.session.json");
    let write = |document: &serde_json::Value| {
        std::fs::write(&path, serde_json::to_vec(document).unwrap()).unwrap()
    };
    write(&initial);
    let config = Config::load(&f.root).unwrap();
    assert_eq!(config.budget(), 2);
    assert_eq!(config.initial(), &[f.id(-1), f.id(0)]);
    assert_eq!(config.index().identity(), f.index().identity());
    let mut world = World::with_chunk_config(&config).unwrap();
    assert_eq!(world.identity(), config.identity());
    let id = world.join().unwrap();
    step(&mut world, id, 1, [1000, 0], false);
    let mut resized = initial.clone();
    resized["resident_budget"] = json!(3);
    write(&resized);
    let resized = Config::load(&f.root).unwrap();
    assert_ne!(resized.identity(), config.identity());
    assert_eq!(resized.index().identity(), config.index().identity());
    assert_eq!(
        World::with_chunk_config(&resized).unwrap().identity(),
        resized.identity()
    );
    let mut altered = initial.clone();
    altered["initial_chunks"] = json!([[f.base, f.base], [f.base + 1, f.base]]);
    write(&altered);
    let alternate = Config::load(&f.root).unwrap();
    assert_ne!(alternate.identity(), config.identity());
    assert_eq!(alternate.index().identity(), config.index().identity());
    assert!(alternate.prepare().is_ok());
    for (field, value) in [
        ("schema", json!(2)),
        ("resident_budget", json!(0)),
        (
            "resident_budget",
            json!(theia_network::chunks::MAX_RESIDENT + 1),
        ),
        ("initial_chunks", json!([])),
        (
            "initial_chunks",
            json!([[f.base, f.base], [f.base, f.base]]),
        ),
        ("initial_chunks", json!([[f.base + 9, f.base]])),
        ("spawns", json!([])),
        ("spawns", json!([f.point(160., 0.).metres()])),
        ("spawns", json!([[1e15, 0., 0.]])),
        ("fall_plane", json!(-5000.)),
        ("unexpected", json!(true)),
    ] {
        let mut bad = initial.clone();
        bad[field] = value;
        write(&bad);
        assert!(
            Config::load(&f.root).is_err(),
            "Accepted bad field {field}: {bad}"
        );
    }
    let mut unsupported = initial.clone();
    unsupported["spawns"] = json!([f.point(16., 3.).metres()]);
    write(&unsupported);
    assert!(
        Config::load(&f.root).unwrap().prepare().is_err(),
        "Startup cannot admit unsupported spawn guesses"
    );
    std::fs::write(&path, vec![b' '; 8193]).unwrap();
    assert!(Config::load(&f.root).is_err());
    assert_eq!(
        world.identity(),
        config.identity(),
        "Later descriptor edits cannot mutate an admitted world"
    );
}

#[test]
fn distant_loaded_actors_use_a_bounded_common_wire_frame_after_join_and_departure() {
    use theia_core::coordinates::ChunkId;
    use theia_network::chunks::Residency;
    let f = Fixture::new(0, false, false);
    let path = f.root.join("world.chunks.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for (old, new) in [(-1, -20), (1, 20)] {
        std::fs::rename(
            f.root.join(format!("chunk_{old}_0")),
            f.root.join(format!("chunk_{new}_0")),
        )
        .unwrap();
        for entry in document["chunks"].as_array_mut().unwrap() {
            if entry["id"][0] == old {
                entry["id"][0] = serde_json::json!(new);
            }
        }
    }
    std::fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    let mut residency = Residency::new(f.index(), 2).unwrap();
    residency
        .replace(&[ChunkId { x: -20, z: 0 }, ChunkId { x: 20, z: 0 }], &[])
        .unwrap();
    let mut world =
        World::with_residency(residency, &[f.point(-2500., 0.), f.point(2600., 0.)], -18.).unwrap();
    let left = world.join().unwrap();
    let right = world.join().unwrap();
    let snapshot = world.step();
    assert_eq!(snapshot.origin, world.origin());
    for player in &snapshot.players {
        assert_eq!(
            snapshot.global_position(player),
            world.global_position(player.id)
        );
    }
    assert!(
        theia_network::wire::Snapshot::decode(&theia_network::wire::encode(&snapshot).unwrap())
            .is_ok()
    );
    world.leave(right);
    let snapshot = world.step();
    assert_eq!(snapshot.origin[0], -2560.);
    assert_eq!(snapshot.players[0].id, left);
    world.join().unwrap();
    world.join().unwrap();
    let snapshot = world.step();
    assert_eq!(
        snapshot.origin,
        world.origin(),
        "Fresh distant admission must restore a common bounded frame"
    );
    assert_eq!(world.movement_recoveries(), 0);
}

#[test]
fn normal_chunk_world_requires_global_margin_for_its_bounded_local_frame() {
    for base in [-7_812_499_999, 7_812_499_999] {
        let f = Fixture::new(base, false, false);
        let error = World::with_residency(f.residency(&[-1, 0]), &[f.point(16., 0.)], -18.)
            .err()
            .expect("Unsafe global frame must fail before startup")
            .to_string();
        assert!(error.contains("global margin"), "{error}");
    }
}

#[test]
fn gravity_slide_cannot_cross_into_client_unready_collision() {
    use std::collections::BTreeSet;
    use theia_core::coordinates::LocalPosition;
    use theia_network::kinematic::Motion;
    for base in [-7_812_500, 0, 7_812_500] {
        for hz in [20, 30, 60] {
            let f = Fixture::new(base, false, false);
            add_triangles(
                &f,
                0,
                vec![
                    [[120., 18., 20.], [120., 18., 44.], [136., -14., 20.]],
                    [[136., -14., 20.], [120., 18., 44.], [136., -14., 44.]],
                ],
            );
            for kind in [0, 1, 2] {
                let offsets = if kind == 0 { vec![0] } else { vec![0, 1] };
                let mut chunks =
                    ChunkWorld::new(f.residency(&offsets), &[f.point(16., 0.)], -18.).unwrap();
                let mut ready = if kind == 2 {
                    BTreeSet::new()
                } else {
                    BTreeSet::from([f.id(0)])
                };
                let mut body = Motion::default();
                let mut feet = LocalPosition::new([127.4, 5., 32.]).unwrap();
                for _ in 0..hz * 2 {
                    feet = chunks
                        .advance_ready(
                            &mut body,
                            feet,
                            Intent::default(),
                            TickRate::from_hz(hz).unwrap(),
                            (kind != 0).then_some(&ready),
                        )
                        .unwrap();
                    assert!(
                        feet.metres()[0] < 127.646,
                        "{hz} Hz gravity crossed readiness boundary: {:?}",
                        feet.metres()
                    );
                }
                assert!(
                    feet.metres()[1] < 4.,
                    "Blocking XZ must not freeze free vertical fall"
                );
                assert!(body.vertical_speed() < -10.);
                assert!(chunks.blocked() > 0);
                if kind != 0 {
                    assert!(chunks.unready_steps() > 0);
                }
                if kind == 0 {
                    let prepared = chunks.plan(1, &[f.id(0), f.id(1)]).unwrap().load().unwrap();
                    chunks
                        .activate(
                            1,
                            prepared,
                            &[Occupant {
                                feet: chunks.frame().to_global(feet).unwrap(),
                                grounded: body.grounded(),
                            }],
                        )
                        .unwrap();
                }
                ready.insert(f.id(0));
                ready.insert(f.id(1));
                for _ in 0..hz {
                    feet = chunks
                        .advance_ready(
                            &mut body,
                            feet,
                            Intent::default(),
                            TickRate::from_hz(hz).unwrap(),
                            (kind != 0).then_some(&ready),
                        )
                        .unwrap();
                }
                assert!(
                    feet.metres()[0] > 128.,
                    "Readiness/load release must resume gravity slide: {:?}",
                    feet.metres()
                );
                assert!(body.grounded());
            }
        }
    }
}

#[test]
fn lookahead_retains_spare_chunks_until_a_new_provider_requires_the_space() {
    for base in [0, -7_812_500, 7_812_500] {
        let f = Fixture::new(base, false, false);
        let chunks = ChunkWorld::new(
            f.residency(&[-1, 0]),
            &[f.point(124., 0.), f.point(122., 0.)],
            -18.,
        )
        .unwrap();
        let walking = Intent {
            axes: [1000, 0],
            ..Intent::default()
        };
        // The trailing peer's input arrives before the leading peer's input.
        // It needs only current coverage, not an unload-only intermediate offer.
        let mut actors = vec![
            (f.point(124., 0.), Intent::default(), false),
            (f.point(122., 0.), walking, false),
        ];
        assert_eq!(chunks.demand(&actors), Some(vec![f.id(-1), f.id(0)]));
        actors.reverse();
        assert_eq!(chunks.demand(&actors), Some(vec![f.id(-1), f.id(0)]));
        actors[1].1 = walking;
        assert_eq!(chunks.demand(&actors), Some(vec![f.id(0), f.id(1)]));
        assert_eq!(chunks.residency().generation(), 1);
        assert_eq!(chunks.residency().loaded(), vec![f.id(-1), f.id(0)]);
    }
}

#[test]
fn falling_without_input_can_request_adjacent_collision_while_idle_grounded_actors_do_not_churn() {
    let f = Fixture::new(0, false, false);
    let chunks = ChunkWorld::new(f.residency(&[0]), &[f.point(16., 0.)], -18.).unwrap();
    assert!(
        chunks
            .demand(&[(f.point(127.6, 5.), Intent::default(), false)])
            .is_none()
    );
    assert_eq!(
        chunks.demand(&[(f.point(127.6, 5.), Intent::default(), true)]),
        Some(vec![f.id(0), f.id(1)])
    );
    assert_eq!(
        chunks.demand(&[(
            f.point(127.6, 5.),
            Intent {
                axes: [-1000, 0],
                jump: false,
                sprint: false
            },
            true
        )]),
        Some(vec![f.id(0), f.id(1)])
    );
    assert_eq!(chunks.residency().generation(), 1);
}

#[test]
fn presentation_retention_protects_full_roots_recovery_and_landing_support_without_mutation() {
    for base in [-7_812_500, 0, 7_812_500] {
        let f = Fixture::new(base, false, false);
        let old = f.residency(&[-1, 0]);
        let next = old.staged(&[f.id(0), f.id(1)], &[]).unwrap();
        let root = |x: f32, y: f32| f.point(x, y + 0.9);
        assert!(
            next.validate_presentation(
                &old,
                &[root(2., 0.008), root(40., 4.)],
                &[f.point(16., 0.)]
            )
            .is_ok()
        );
        for p in [root(-64., 0.008), root(0.2, 0.008)] {
            assert!(
                next.validate_presentation(&old, &[p], &[f.point(16., 0.)])
                    .is_err()
            );
        }
        assert!(
            next.validate_presentation(&old, &[root(2., 0.008)], &[f.point(-64., 0.)])
                .is_err()
        );
        assert!(
            next.validate_presentation(&old, &vec![root(2., 0.008); 25], &[f.point(16., 0.)])
                .is_err()
        );
        assert_eq!(old.generation(), 1);
        assert_eq!(next.generation(), 2);
        assert_eq!(old.loaded(), vec![f.id(-1), f.id(0)]);
    }
    let f = Fixture::new(0, true, false);
    let old = f.residency(&[-1, 0]);
    let next = old.staged(&[f.id(0), f.id(1)], &[]).unwrap();
    assert!(
        next.validate_presentation(&old, &[f.point(45., 3.9)], &[f.point(16., 0.)])
            .unwrap_err()
            .to_string()
            .contains("landing")
    );
    let f = Fixture::new(0, false, true);
    let old = f.residency(&[-1, 0]);
    let next = old.staged(&[f.id(0), f.id(1)], &[]).unwrap();
    assert!(
        next.validate_presentation(&old, &[f.point(45., 0.908)], &[f.point(16., 0.)])
            .is_err()
    );
}

#[test]
fn readiness_holds_intentional_motion_preserves_jump_and_cannot_be_poisoned_by_stale_reports() {
    use theia_network::wire::ChunkReady;
    for hz in [20, 30, 60] {
        let f = Fixture::new(0, false, false);
        let mut w = f.world(&[0, 1], 126.);
        w.set_tick_rate(TickRate::from_hz(hz).unwrap());
        w.require_client_chunks().unwrap();
        let actor = w.join().unwrap();
        let epoch = w.epoch();
        let mut sequence = 0;
        for _ in 0..hz {
            sequence += 1;
            let p = step(&mut w, actor, sequence, [1000, 0], false);
            assert!((p[0] - 126.).abs() < 0.0001);
        }
        let report = |seq, chunks| ChunkReady {
            version: VERSION,
            epoch,
            sequence: seq,
            chunks,
        };
        assert!(w.client_ready(actor, &report(1, vec![[-1, 0], [0, 0]])));
        let mut max_y = 0_f32;
        for _ in 0..hz {
            sequence += 1;
            let p = step(&mut w, actor, sequence, [1000, 0], true);
            max_y = max_y.max(p[1]);
            assert!(p[0] < 127.646);
        }
        assert!((2.0..2.2).contains(&max_y), "{hz}: root jump peak {max_y}");
        let old = w.global_position(actor).unwrap();
        let bad_epoch = ChunkReady {
            epoch: epoch.wrapping_add(1),
            ..report(u64::MAX, vec![[0, 0], [1, 0]])
        };
        assert!(!w.client_ready(actor, &bad_epoch));
        assert!(!w.client_ready(actor, &report(u64::MAX, vec![[99, 0]])));
        assert!(w.client_ready(actor, &report(2, vec![[0, 0], [1, 0]])));
        assert!(!w.client_ready(actor, &report(1, vec![])));
        for _ in 0..hz {
            sequence += 1;
            step(&mut w, actor, sequence, [1000, 0], false);
        }
        assert!(w.global_position(actor).unwrap().metres()[0] > old.metres()[0] + 4.9);
        assert!(w.client_ready(actor, &report(3, vec![])));
        let stopped = w.global_position(actor).unwrap();
        for _ in 0..hz {
            sequence += 1;
            step(&mut w, actor, sequence, [1000, 0], false);
        }
        assert_eq!(
            w.global_position(actor).unwrap().metres()[0],
            stopped.metres()[0]
        );
        assert_eq!(w.movement_recoveries(), 0);
        assert!(w.chunks().unwrap().unready_steps() > 0);
        w.leave(actor);
        let replacement = w.join().unwrap();
        assert!(!w.client_ready(actor, &report(4, vec![[0, 0], [1, 0]])));
        for seq in 1..=hz {
            let p = step(&mut w, replacement, u64::from(seq), [1000, 0], false);
            assert!((p[0] - 126.).abs() < 0.0001);
        }
        assert!(w.client_ready(replacement, &report(1, vec![[0, 0], [1, 0]])));
    }
}

#[test]
fn startup_exposes_resolved_recovery_feet_instead_of_raw_sloped_spawn_guesses() {
    use theia_network::{chunk_world::Config, kinematic};
    let f = Fixture::new(0, false, false);
    add_triangles(
        &f,
        0,
        vec![
            [[-1., -6.8, -1.], [-1., -6.8, 129.], [129., 45.2, -1.]],
            [[129., 45.2, -1.], [-1., -6.8, 129.], [129., 45.2, 129.]],
        ],
    );
    let raw = f.point(16., 0.);
    let doc = json!({"schema":1,"resident_budget":2,"initial_chunks":[[-1,0],[0,0]],"spawns":[raw.metres()],"fall_plane":-18.});
    std::fs::write(
        f.root.join("world.session.json"),
        serde_json::to_vec(&doc).unwrap(),
    )
    .unwrap();
    let config = Config::load(&f.root).unwrap();
    let (active, resolved) = config.prepare_startup().unwrap();
    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].metres()[1] > raw.metres()[1] + 0.02);
    assert!(
        kinematic::validate_pose(
            active.scene().unwrap(),
            config.index().frame().to_local(raw).unwrap(),
            true
        )
        .is_err()
    );
    assert!(active.validate_presentation(&active, &[], &[raw]).is_err());
    active
        .validate_presentation(&active, &[], &resolved)
        .unwrap();
    let mut world = World::with_chunk_config(&config).unwrap();
    let id = world.join().unwrap();
    assert!(
        (world.global_position(id).unwrap().metres()[1]
            - f64::from(theia_network::world::PAWN_Y)
            - resolved[0].metres()[1])
            .abs()
            < 0.001
    );
}

#[test]
fn decimal_f64_metadata_keeps_exact_ieee_bits_for_admission_identity() {
    for text in [
        "-0.10767337572414185",
        "-0.13055541686360841",
        "1.164313264334784",
        "-62.83568673566521",
    ] {
        let expected = text.parse::<f64>().unwrap();
        let typed = serde_json::from_str::<f64>(text).unwrap();
        assert_eq!(
            typed.to_bits(),
            expected.to_bits(),
            "JSON metadata changed decimal {text}"
        );
        let value: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(value.as_f64().unwrap().to_bits(), expected.to_bits());
        assert_eq!(
            serde_json::from_str::<f64>(&serde_json::to_string(&expected).unwrap())
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
    }
}
