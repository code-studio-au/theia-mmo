#[path = "support/saved_fixture.rs"]
mod saved_fixture;
use saved_fixture::Fixture;
use std::{
    fs,
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::{
    procedural::{self, Patch, Source},
    saved_terrain::{self, Store},
    simulation::World,
    terrain::sha256,
    wire::{ChunkReady, Intent, VERSION, Welcome},
};

#[test]
fn full_baked_continent_has_exact_samples_lods_seams_and_lazy_reads() {
    let fixture = Fixture::new();
    assert!(saved_terrain::bake(&fixture.root, "theia-test-terrain", 2).is_err());
    let store = fixture.open();
    assert_eq!(
        store.stats().tile_reads,
        0,
        "Opening must not read every tile"
    );
    store.verify().unwrap();
    assert_eq!(store.stats().tile_reads, 15625);
    for id in [
        ChunkId { x: 0, z: 0 },
        ChunkId { x: 62, z: 62 },
        ChunkId { x: 124, z: 124 },
    ] {
        for cells in [8, 32] {
            let saved = store.geometry(id, cells).unwrap();
            let generated = procedural::geometry(id, cells);
            assert_eq!(saved.positions, generated.positions);
            assert_eq!(saved.triangles, generated.triangles);
        }
        let source = Source::Saved(Arc::new(fixture.open()));
        let patch = Patch::load(id, &source).unwrap();
        let [x, _, z] = id.origin(128).unwrap().metres();
        let p = patch.try_spawn(x + 64., z + 64.).unwrap();
        let baseline = Patch::build(id).spawn(x + 64., z + 64.);
        assert_eq!(p, baseline);
    }
    assert!(store.geometry(ChunkId { x: -1, z: 0 }, 32).is_err());
    assert!(store.geometry(ChunkId { x: 0, z: 0 }, 16).is_err());
}

#[test]
fn corrupt_missing_wrong_coordinate_and_rehashed_invalid_tiles_fail_closed() {
    let fixture = Fixture::new();
    let id = ChunkId { x: 62, z: 62 };
    let path = fixture.tile(id);
    let original = fs::read(&path).unwrap();
    let store = fixture.open();
    fs::remove_file(&path).unwrap();
    assert!(store.heights(id).is_err());
    let mut bad = original.clone();
    bad[30] ^= 1;
    fs::write(&path, &bad).unwrap();
    assert!(store.heights(id).is_err());
    for bytes in [&original[..100], &[original.as_slice(), &[0]].concat()] {
        fs::write(&path, bytes).unwrap();
        fixture.rehash(id);
        assert!(fixture.open().heights(id).is_err());
    }
    for value in [f32::NAN, f32::INFINITY, 129.] {
        bad = original.clone();
        bad[24..28].copy_from_slice(&value.to_le_bytes());
        fs::write(&path, &bad).unwrap();
        fixture.rehash(id);
        assert!(fixture.open().heights(id).is_err());
    }
    bad = original.clone();
    bad[8..16].copy_from_slice(&61_i64.to_le_bytes());
    fs::write(&path, &bad).unwrap();
    fixture.rehash(id);
    assert!(fixture.open().heights(id).is_err());
    // A valid hash does not excuse discontinuous authoring at a shared edge.
    bad = original.clone();
    let h = f32::from_le_bytes(bad[24..28].try_into().unwrap()) + 1.;
    bad[24..28].copy_from_slice(&h.to_le_bytes());
    fs::write(&path, &bad).unwrap();
    fixture.rehash(id);
    let discontinuous = fixture.open();
    assert!(discontinuous.heights(id).is_ok());
    assert!(Patch::load(id, &Source::Saved(Arc::new(discontinuous))).is_err());
    // An edited interior really drives geometry and collision without the recipe.
    bad = original.clone();
    let offset = 24 + (16 * 33 + 16) * 4;
    let raised = f32::from_le_bytes(bad[offset..offset + 4].try_into().unwrap()) + 0.25;
    bad[offset..offset + 4].copy_from_slice(&raised.to_le_bytes());
    fs::write(&path, &bad).unwrap();
    fixture.rehash(id);
    let edited = fixture.open();
    assert_ne!(edited.identity(), store.identity());
    assert_eq!(
        edited.geometry(id, 32).unwrap().positions[16 * 33 + 16][1],
        raised
    );
    let patch = Patch::load(id, &Source::Saved(Arc::new(edited))).unwrap();
    let ground = patch.scene.ground([64., 64.], 100.).unwrap().0;
    assert!((ground - raised).abs() < 0.0001);
    // The old admitted manifest rejects replacement bytes even after rehashing.
    assert!(store.heights(id).is_err());
    assert_ne!(sha256(&original), sha256(&bad));
    let manifest_path = fixture.root.join(saved_terrain::MANIFEST);
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["schema"] = serde_json::json!(2);
    fs::write(manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(Store::open(&fixture.root).is_err());
}

#[test]
fn saved_authority_crosses_chunk_and_reloads_distant_collision_with_bounded_residency() {
    let fixture = Fixture::new();
    let store = Arc::new(fixture.open());
    let mut world =
        World::with_terrain_source([8062., 8000.], 1., Source::Saved(store.clone())).unwrap();
    world.enable_prediction().unwrap();
    let id = world.join().unwrap();
    let welcome = Welcome {
        terrain_stream: true,
        id,
        epoch: world.epoch(),
        world: world.identity(),
        origin: world.origin(),
    };
    assert_ne!(world.identity(), procedural::IDENTITY);
    assert_eq!(world.identity(), store.identity());
    assert_eq!(store.stats().tile_reads, 9);
    let ready = procedural::neighbours(ChunkId { x: 62, z: 62 }, 1);
    assert!(world.client_ready(
        id,
        &ChunkReady {
            version: VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks: ready.iter().map(|id| [id.x, id.z]).collect()
        }
    ));
    for tick in 1..=30 {
        let correction = world.correction(id).unwrap().state;
        assert_eq!(
            world.command(
                id,
                &theia_network::prediction_wire::Commands {
                    epoch: world.epoch(),
                    stream_epoch: correction.input.stream_epoch,
                    ready_sequence: 1,
                    commands: vec![theia_network::commands::Command {
                        sequence: tick,
                        tick: correction.input.tick + 1,
                        intent: Intent {
                            axes: [1000, 0],
                            sprint: true,
                            jump: tick <= 4
                        }
                    }]
                }
            ),
            1
        );
        world.step();
    }
    assert!(world.global_position(id).unwrap().metres()[0] > 8064.);
    assert!(welcome.accepts(&world.snapshot_for(id)));
    assert_eq!(
        store.stats().tile_reads,
        9,
        "Simulation steps must perform no terrain I/O"
    );
    let mut previous_stream = 1;
    for (visit, xz) in [
        [15800., 180.],
        [180., 15800.],
        [8062., 8000.],
        [15800., 180.],
    ]
    .into_iter()
    .enumerate()
    {
        let reads_before = store.stats().tile_reads;
        assert!(world.teleport_development(id, xz));
        let until = Instant::now() + Duration::from_secs(3);
        while world.correction(id).unwrap().state.input.stream_epoch == previous_stream {
            assert!(Instant::now() < until);
            world.poll_terrain();
            std::thread::sleep(Duration::from_millis(1));
        }
        previous_stream += 1;
        world.poll_terrain();
        assert!(welcome.accepts(&world.snapshot_for(id)));
        assert!(world.procedural().unwrap().patches.len() <= 2);
        assert!(world.procedural().unwrap().error.is_none());
        if visit == 3 {
            assert!(
                store.stats().tile_reads >= reads_before + 9,
                "Revisiting retired non-spawn collision must read saved tiles again"
            );
        }
        assert!(procedural::owner(GlobalPosition::new([xz[0], 0., xz[1]]).unwrap()).is_some());
    }
    assert!(store.stats().tile_reads >= 36);
    // Missing future terrain produces one explicit worker error, retaining support.
    let target = ChunkId { x: 1, z: 1 };
    fs::remove_file(fixture.tile(target)).unwrap();
    assert!(world.teleport_development(id, [180., 180.]));
    let until = Instant::now() + Duration::from_secs(3);
    while world.procedural().unwrap().error.is_none() {
        assert!(Instant::now() < until);
        world.poll_terrain();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        world.correction(id).unwrap().state.input.stream_epoch,
        previous_stream
    );
    assert!(
        world
            .procedural()
            .unwrap()
            .patch(world.global_position(id).unwrap())
            .is_some()
    );
}
