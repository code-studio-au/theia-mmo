//! Imported 1 m/2 m terrain: exact source surfaces, bounded edges and authority.
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::Arc, time::Instant};
use theia_core::{coordinates::ChunkId, time::TickRate};
use theia_network::{
    procedural::{Patch, Source},
    saved_terrain::Store,
    simulation::World,
    terrain::sha256,
    wire::{ChunkReady, Input, Intent, VERSION},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(cells: usize) -> Self {
        let root =
            std::env::temp_dir().join(format!("theia-import-{}-{cells}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let mut hashes = vec![];
        for x in 15_i64..17 {
            let mut bytes = b"THTILE01".to_vec();
            bytes.extend(x.to_le_bytes());
            bytes.extend(14_i64.to_le_bytes());
            for z in 0..=cells {
                for i in 0..=cells {
                    let wx = (x - 15) as f32 * 128. + i as f32 * 128. / cells as f32;
                    let wz = z as f32 * 128. / cells as f32;
                    let h = 300.
                        + wx * 0.03125
                        + wz * 0.0625
                        + if wx == 7. && wz == 3. { 0.25 } else { 0. };
                    bytes.extend(h.to_le_bytes());
                }
            }
            fs::write(root.join(format!("tile_{x}_14.height")), &bytes).unwrap();
            hashes.push(sha256(&bytes));
        }
        fs::write(
            root.join("terrain.manifest.json"),
            serde_json::to_vec(&json!({
                "schema":2,"content_id":"theia.test.import","revision":1,"chunk_metres":128,
                "chunks_xz":[2,1],"chunk_origin":[15,14],"samples_per_axis":cells+1,"tiles":hashes
            }))
            .unwrap(),
        )
        .unwrap();
        Self(root)
    }
    fn store(&self) -> Arc<Store> {
        Arc::new(Store::open(&self.0).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check_surface(store: &Arc<Store>, owner: ChunkId) -> f32 {
    let patch = Patch::load(owner, &Source::Saved(store.clone())).unwrap();
    let g = store.geometry(owner, store.cells()).unwrap();
    let mut error = 0_f32;
    for p in g
        .positions
        .iter()
        .copied()
        .chain(g.triangles.iter().map(|t| {
            std::array::from_fn(|axis| {
                t.iter()
                    .map(|i| g.positions[*i as usize][axis])
                    .sum::<f32>()
                    / 3.
            })
        }))
    {
        let actual = patch.scene.ground([p[0], p[2]], p[1] + 2.).unwrap().0;
        error = error.max((actual - p[1]).abs());
    }
    assert!(error < 0.001, "source surface error {error} at {owner:?}");
    error
}

#[test]
fn imported_grids_preserve_triangles_high_elevations_and_rectangle_edges() {
    for cells in [64, 128] {
        let fixture = Fixture::new(cells);
        let store = fixture.store();
        assert_eq!(store.stats().tile_reads, 0);
        store.verify().unwrap();
        assert_eq!(store.stats().tile_reads, 2);
        assert_eq!(store.bounds(), [[1920., 1792.], [2176., 1920.]]);
        assert_eq!(store.cells(), cells);
        let id = ChunkId { x: 15, z: 14 };
        check_surface(&store, id);
        let patch = Patch::load(id, &Source::Saved(store.clone())).unwrap();
        assert_eq!(patch.ids.len(), 2);
        assert!(patch.try_spawn(1984., 1856.).unwrap().metres()[1] > 300.);
        assert!(store.heights(ChunkId { x: 14, z: 14 }).is_err());
        assert!(Patch::load(ChunkId { x: 15, z: 15 }, &Source::Saved(store.clone())).is_err());
        for coarse in [8, 32] {
            let full = store.geometry(id, cells).unwrap();
            let lod = store.geometry(id, coarse).unwrap();
            for (i, p) in lod.positions.iter().enumerate() {
                let x = i % (coarse + 1) * cells / coarse;
                let z = i / (coarse + 1) * cells / coarse;
                assert_eq!(*p, full.positions[z * (cells + 1) + x]);
            }
        }
        for hz in [20, 30, 60] {
            let mut world =
                World::with_terrain_source([2044., 1856.], 1., Source::Saved(store.clone()))
                    .unwrap();
            assert!(world.set_tick_rate(TickRate::from_hz(hz).unwrap()));
            let actor = world.join().unwrap();
            assert!(world.client_ready(
                actor,
                &ChunkReady {
                    version: VERSION,
                    epoch: world.epoch(),
                    sequence: 1,
                    chunks: vec![[15, 14], [16, 14]],
                }
            ));
            let before = store.stats().tile_reads;
            for sequence in 1..=u64::from(hz) * 2 {
                assert!(world.input(
                    actor,
                    Input {
                        version: VERSION,
                        epoch: world.epoch(),
                        sequence,
                        intent: Intent {
                            axes: [1000, 0],
                            jump: sequence < 4,
                            ..Intent::default()
                        },
                    }
                ));
                world.step();
            }
            assert!(world.global_position(actor).unwrap().metres()[0] > 2052.);
            assert_eq!(world.movement_recoveries(), 0);
            assert_eq!(
                store.stats().tile_reads,
                before,
                "Fixed simulation performed terrain I/O"
            );
            assert!(!world.teleport_development(actor, [8000., 8000.]));
        }
        // Rehashed source discontinuity must reject before collision publication.
        let path = fixture.0.join("tile_16_14.height");
        let mut bytes = fs::read(&path).unwrap();
        bytes[24..28].copy_from_slice(&400_f32.to_le_bytes());
        fs::write(path, &bytes).unwrap();
        let path = fixture.0.join("terrain.manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["tiles"][1] = json!(sha256(&bytes));
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(fixture.store().verify().is_err());
        assert!(Patch::load(id, &Source::Saved(fixture.store())).is_err());
        for (key, value) in [
            ("schema", json!(1)),
            ("samples_per_axis", json!(257)),
            ("chunk_origin", json!([124, 14])),
            ("chunks_xz", json!([0, 1])),
        ] {
            let mut bad = manifest.clone();
            bad[key] = value;
            fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(Store::open(&fixture.0).is_err());
        }
    }
}

#[test]
#[ignore = "requires the user's staged Millhaven source; set THEIA_IMPORTED_TERRAIN and THEIA_IMPORTED_REPORT"]
fn millhaven_complete_package_and_representative_avian_surfaces() {
    let root = PathBuf::from(std::env::var_os("THEIA_IMPORTED_TERRAIN").unwrap());
    let output = PathBuf::from(std::env::var_os("THEIA_IMPORTED_REPORT").unwrap());
    assert!(!output.exists());
    let store = Arc::new(Store::open(&root).unwrap());
    assert_eq!(store.bounds(), [[0., 0.], [4096., 3072.]]);
    assert_eq!(store.cells(), 128);
    store.verify().unwrap();
    assert_eq!(store.stats().tile_reads, 768);
    let mut cases = vec![];
    for [x, z] in [
        [0, 0],
        [31, 0],
        [0, 23],
        [31, 23],
        [15, 14],
        [16, 14],
        [18, 18],
        [19, 19],
        [24, 13],
        [20, 10],
    ] {
        let start = Instant::now();
        let error = check_surface(&store, ChunkId { x, z });
        cases.push(json!({"owner":[x,z],"surface_error_m":error,"check_ms":start.elapsed().as_secs_f64()*1000.}));
    }
    let world =
        World::with_terrain_source([2048., 1810.], 1., Source::Saved(store.clone())).unwrap();
    assert_eq!(world.identity(), store.identity());
    fs::write(output,serde_json::to_vec_pretty(&json!({"passed":true,"tiles_verified":768,
        "vertex_and_triangle_centroid_queries":10*(129*129+2*128*128),"cases":cases,
        "scope":"Full package seams and ten actual Avian tile surfaces; separate native/network qualification"})).unwrap()).unwrap();
}
