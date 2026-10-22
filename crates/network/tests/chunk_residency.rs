//! Real file/package loading and Avian scene replacement, without renderer claims.
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalPosition};
use theia_network::{
    chunks::{Index, Residency},
    content::Manifest,
    terrain::sha256,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    base: i64,
    document: Value,
}
impl Fixture {
    fn new(base: i64) -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-chunks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let mut entries = Vec::new();
        for x in [base - 1, base, base + 1] {
            let directory = root.join(format!("chunk_{x}_{base}"));
            std::fs::create_dir(&directory).unwrap();
            let visual = b"Chunk binding fixture; no rendering claimed";
            let profile=serde_json::to_vec(&json!({"schema":2,"bounds":[[-1.,-1.],[129.,129.]],"source_origin":[0.,0.,0.],"visual_sha256":sha256(visual),"triangles":[[[-1.,0.,-1.],[-1.,0.,129.],[129.,0.,-1.]],[[129.,0.,-1.],[-1.,0.,129.],[129.,0.,129.]]]})).unwrap();
            let manifest=serde_json::to_vec(&json!({"schema":1,"content_id":format!("theia.test.chunk.x{x}-z{base}"),"revision":1,"collision_schema":2,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)})).unwrap();
            std::fs::write(directory.join("terrain.json"), &profile).unwrap();
            std::fs::write(directory.join("terrain.glb"), visual).unwrap();
            std::fs::write(directory.join("world.manifest.json"), &manifest).unwrap();
            entries.push(
                json!({"id":[x,base],"package":Manifest::decode(&manifest).unwrap().identity()}),
            );
        }
        let document = json!({"schema":1,"chunk_metres":128,"query_origin":[base as f64*128.,0.,base as f64*128.],"chunks":entries});
        let fixture = Self {
            root,
            base,
            document,
        };
        fixture.write();
        fixture
    }
    fn write(&self) {
        std::fs::write(
            self.root.join("world.chunks.json"),
            serde_json::to_vec(&self.document).unwrap(),
        )
        .unwrap();
    }
    fn index(&self) -> Index {
        Index::load(&self.root).unwrap()
    }
    fn with_contributors(base: i64) -> Self {
        let mut fixture = Self::new(base);
        fixture.document["schema"] = json!(2);
        for n in 0..3 {
            let id = fixture.id(n - 1);
            let directory = fixture.root.join(format!("chunk_{}_{}", id.x, id.z));
            let mut profile: Value =
                serde_json::from_slice(&std::fs::read(directory.join("terrain.json")).unwrap())
                    .unwrap();
            profile["schema"] = json!(3);
            profile["bounds"] = json!([[0., 0.], [136., 136.]]);
            profile["triangles"] = json!([
                [[0., 0., 0.], [0., 0., 128.], [128., 0., 0.]],
                [[128., 0., 0.], [0., 0., 128.], [128., 0., 128.]]
            ]);
            let bounds = if n == 1 {
                profile["solids"] = json!([{"center":[132.,1.,132.],"size":[0.2,2.,8.]}]);
                json!([[0., 0.], [133., 136.]])
            } else {
                json!([[0., 0.], [128., 128.]])
            };
            let bytes = serde_json::to_vec(&profile).unwrap();
            let visual = std::fs::read(directory.join("terrain.glb")).unwrap();
            let manifest = serde_json::to_vec(&json!({"schema":1,"content_id":format!("theia.test.contributors.x{}-z{}",id.x,id.z),"revision":2,"collision_schema":3,"collision_bytes":bytes.len(),"collision_sha256":sha256(&bytes),"visual_bytes":visual.len(),"visual_sha256":sha256(&visual)})).unwrap();
            std::fs::write(directory.join("terrain.json"), bytes).unwrap();
            std::fs::write(directory.join("world.manifest.json"), &manifest).unwrap();
            fixture.document["chunks"][n as usize]["package"] =
                json!(Manifest::decode(&manifest).unwrap().identity());
            fixture.document["chunks"][n as usize]["bounds"] = bounds;
        }
        let source = fixture.root.join(format!("chunk_{}_{}", base + 1, base));
        let corner = fixture
            .root
            .join(format!("chunk_{}_{}", base + 1, base + 1));
        std::fs::create_dir(&corner).unwrap();
        for name in ["terrain.json", "terrain.glb", "world.manifest.json"] {
            std::fs::copy(source.join(name), corner.join(name)).unwrap();
        }
        let mut entry = fixture.document["chunks"][2].clone();
        entry["id"] = json!([base + 1, base + 1]);
        fixture.document["chunks"]
            .as_array_mut()
            .unwrap()
            .push(entry);
        fixture.write();
        fixture
    }
    fn id(&self, offset: i64) -> ChunkId {
        ChunkId {
            x: self.base + offset,
            z: self.base,
        }
    }
    fn point(&self, x: f32) -> GlobalPosition {
        self.index()
            .frame()
            .to_global(LocalPosition::new([x, 0., 32.]).unwrap())
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn resident_collision_and_visual_ownership_swap_together_across_seams() {
    let fixture = Fixture::new(0);
    let mut chunks = Residency::new(fixture.index(), 2).unwrap();
    chunks
        .replace(&[fixture.id(-1), fixture.id(0)], &[fixture.point(-0.01)])
        .unwrap();
    for x in [-0.01, 0., 0.01, 127.99] {
        assert!(
            chunks
                .scene()
                .unwrap()
                .ground([x, 32.], 5.)
                .unwrap()
                .0
                .abs()
                < 0.001
        );
    }
    assert_eq!(
        chunks.index().owner(fixture.point(-0.01)),
        Some(fixture.id(-1))
    );
    assert_eq!(chunks.index().owner(fixture.point(0.)), Some(fixture.id(0)));
    assert_eq!(chunks.visuals().len(), 2);
    assert!(
        chunks
            .replace(&[fixture.id(0), fixture.id(1)], &[fixture.point(-0.01)])
            .is_err()
    );
    chunks
        .replace(&[fixture.id(0), fixture.id(1)], &[fixture.point(64.)])
        .unwrap();
    assert!(
        chunks.scene().unwrap().ground([-64., 32.], 5.).is_none(),
        "Unloaded collision must actually disappear"
    );
    assert!(chunks.scene().unwrap().ground([128.01, 32.], 5.).is_some());
    assert_eq!(
        chunks.index().owner(fixture.point(128.)),
        Some(fixture.id(1))
    );
    for root in chunks.visuals() {
        assert_eq!(root.origin.chunk(128), Some(root.chunk));
        assert!(root.asset.is_file());
    }
    chunks.replace(&[], &[]).unwrap();
    assert!(chunks.scene().is_none() && chunks.visuals().is_empty());
    assert_eq!(
        (
            chunks.metrics().loads,
            chunks.metrics().unloads,
            chunks.metrics().max_active
        ),
        (3, 3, 2)
    );
}
#[test]
fn rejected_or_corrupt_replacements_preserve_active_collision_and_metrics() {
    let fixture = Fixture::new(0);
    let mut chunks = Residency::new(fixture.index(), 2).unwrap();
    chunks
        .replace(&[fixture.id(-1), fixture.id(0)], &[])
        .unwrap();
    let old = chunks.loaded();
    for ids in [
        vec![fixture.id(-1), fixture.id(0), fixture.id(1)],
        vec![fixture.id(0), fixture.id(0)],
        vec![fixture.id(9)],
    ] {
        assert!(chunks.replace(&ids, &[]).is_err());
        assert_eq!(chunks.loaded(), old);
    }
    let path = fixture.root.join("chunk_1_0/terrain.json");
    let original = std::fs::read(&path).unwrap();
    std::fs::write(&path, b"{}").unwrap();
    assert!(
        chunks
            .replace(&[fixture.id(0), fixture.id(1)], &[fixture.point(64.)])
            .is_err()
    );
    assert_eq!(chunks.loaded(), old);
    assert_eq!(chunks.metrics().loads, 2);
    assert_eq!(chunks.metrics().unloads, 0);
    assert!(chunks.scene().unwrap().ground([-64., 32.], 5.).is_some());
    std::fs::write(path, original).unwrap();
    std::fs::remove_file(fixture.root.join("chunk_1_0/world.manifest.json")).unwrap();
    assert!(
        chunks
            .replace(&[fixture.id(0), fixture.id(1)], &[])
            .is_err(),
        "No legacy downgrade for indexed chunks"
    );
    assert_eq!(chunks.loaded(), old);
}
#[test]
fn signed_billion_metre_seams_keep_owner_and_query_frame_consistent() {
    for base in [-7_812_500, 7_812_500] {
        let fixture = Fixture::new(base);
        let mut chunks = Residency::new(fixture.index(), 2).unwrap();
        chunks
            .replace(&[fixture.id(-1), fixture.id(0)], &[])
            .unwrap();
        for (x, offset) in [(-0.01, -1), (0., 0), (0.01, 0), (127.99, 0)] {
            let global = fixture.point(x);
            assert_eq!(chunks.index().owner(global), Some(fixture.id(offset)));
            let local = chunks.index().frame().to_local(global).unwrap().metres();
            assert!((local[0] - x).abs() < 0.00001);
            assert!(
                chunks
                    .scene()
                    .unwrap()
                    .ground([local[0], local[2]], 5.)
                    .is_some()
            );
        }
    }
}
#[test]
fn index_schema_bounds_duplicate_ids_and_resident_budgets_are_checked() {
    let mut fixture = Fixture::new(0);
    let original = fixture.document.clone();
    for (key, value) in [
        ("schema", json!(2)),
        ("chunk_metres", json!(0)),
        ("chunk_metres", json!(256)),
        ("query_origin", json!([1e13, 0., 0.])),
        ("chunks", json!([])),
        ("unknown", json!(true)),
    ] {
        fixture.document = original.clone();
        fixture.document[key] = value;
        fixture.write();
        assert!(Index::load(&fixture.root).is_err(), "{key}");
    }
    fixture.document = original.clone();
    fixture.document["chunks"][1] = fixture.document["chunks"][0].clone();
    fixture.write();
    assert!(Index::load(&fixture.root).is_err());
    fixture.document = original;
    fixture.write();
    assert!(Residency::new(fixture.index(), 0).is_err());
    assert!(Residency::new(fixture.index(), theia_network::chunks::MAX_RESIDENT + 1).is_err());
    let mut chunks = Residency::new(fixture.index(), 2).unwrap();
    assert!(
        chunks
            .replace(&[fixture.id(0)], &[fixture.point(300.)])
            .is_err()
    );
}

#[test]
fn staging_and_visual_validation_never_mutate_the_published_set() {
    let fixture = Fixture::new(0);
    let mut published = Residency::new(fixture.index(), 2).unwrap();
    published
        .replace(&[fixture.id(-1), fixture.id(0)], &[])
        .unwrap();
    let candidate = published
        .staged(&[fixture.id(0), fixture.id(1)], &[fixture.point(64.)])
        .unwrap();
    candidate.verify_visuals().unwrap();
    assert_eq!(published.loaded(), vec![fixture.id(-1), fixture.id(0)]);
    assert_eq!(candidate.loaded(), vec![fixture.id(0), fixture.id(1)]);
    assert!(published.scene().unwrap().ground([-64., 32.], 5.).is_some());
    assert!(candidate.scene().unwrap().ground([-64., 32.], 5.).is_none());
    std::fs::write(
        fixture.root.join("chunk_1_0/terrain.glb"),
        b"corrupt candidate visual",
    )
    .unwrap();
    assert!(candidate.verify_visuals().is_err());
    assert_eq!(published.metrics().loads, 2);
    assert_eq!(published.visuals().len(), 2);
}

#[test]
fn contributor_coverage_requires_nonlocal_wall_even_when_owner_floor_is_loaded() {
    use theia_core::time::TickRate;
    use theia_network::{chunk_world::ChunkWorld, kinematic::Motion, wire::Intent};
    for base in [0, -7_812_500, 7_812_500] {
        let fixture = Fixture::with_contributors(base);
        let corner = ChunkId {
            x: base + 1,
            z: base + 1,
        };
        let index = fixture.index();
        assert_eq!(
            index.contributors(corner).unwrap(),
            &[fixture.id(0), corner]
        );
        let position = index
            .frame()
            .to_global(LocalPosition::new([140., 0., 132.]).unwrap())
            .unwrap();
        let mut resident = Residency::new(index.clone(), 2).unwrap();
        resident.replace(&[corner], &[]).unwrap();
        assert!(resident.scene().unwrap().ground([140., 132.], 4.).is_some());
        assert!(!index.coverage([corner]).unwrap().contains(position));
        assert!(ChunkWorld::new(resident.clone(), &[position], -18.).is_err());
        resident.replace(&[fixture.id(0), corner], &[]).unwrap();
        assert!(
            index
                .coverage(resident.loaded())
                .unwrap()
                .contains(position)
        );
        for (x, axis) in [(140., -1000), (129., 1000)] {
            let spawn = index
                .frame()
                .to_global(LocalPosition::new([x, 0., 132.]).unwrap())
                .unwrap();
            for hz in [20, 30, 60] {
                let mut world = ChunkWorld::new(resident.clone(), &[spawn], -18.).unwrap();
                assert_eq!(
                    world
                        .demand(&[(
                            spawn,
                            Intent {
                                axes: [1000, 0],
                                ..Intent::default()
                            },
                            false
                        )])
                        .unwrap(),
                    vec![fixture.id(0), corner]
                );
                let mut body = Motion::default();
                let mut p = world.spawn(0);
                let missing = std::collections::BTreeSet::from([corner]);
                let intent = Intent {
                    axes: [axis, 0],
                    ..Intent::default()
                };
                for _ in 0..hz {
                    p = world
                        .advance_ready(
                            &mut body,
                            p,
                            intent,
                            TickRate::from_hz(hz).unwrap(),
                            Some(&missing),
                        )
                        .unwrap();
                }
                assert_eq!(
                    p.metres()[0],
                    x,
                    "Missing contributor must block movement on the loaded owner floor"
                );
                let complete = std::collections::BTreeSet::from([fixture.id(0), corner]);
                for _ in 0..hz * 2 {
                    p = world
                        .advance_ready(
                            &mut body,
                            p,
                            intent,
                            TickRate::from_hz(hz).unwrap(),
                            Some(&complete),
                        )
                        .unwrap();
                }
                let final_x = p.metres()[0];
                assert!(
                    if axis < 0 {
                        (132.44..132.47).contains(&final_x)
                    } else {
                        (131.53..131.56).contains(&final_x)
                    },
                    "Nonlocal wall did not block from {x} at {hz} Hz: {p:?}"
                );
            }
        }
        let generation = resident.generation();
        assert!(resident.replace(&[corner], &[position]).is_err());
        assert_eq!(resident.generation(), generation);
        assert_eq!(resident.loaded(), vec![fixture.id(0), corner]);
        let candidate = resident.staged(&[corner], &[]).unwrap();
        let root = index
            .frame()
            .to_global(LocalPosition::new([140., 0.9, 132.]).unwrap())
            .unwrap();
        assert!(
            candidate
                .validate_presentation(&resident, &[root], &[])
                .is_err()
        );
        assert!(
            candidate
                .validate_presentation(&resident, &[], &[position])
                .is_err()
        );
        std::fs::write(fixture.root.join("world.session.json"),serde_json::to_vec(&json!({"schema":1,"resident_budget":2,"initial_chunks":[[base+1,base+1]],"spawns":[position.metres()],"fall_plane":-18.})).unwrap()).unwrap();
        assert!(theia_network::chunk_world::Config::load(&fixture.root).is_err());
    }
}

#[test]
fn contributor_bounds_bind_identity_and_reject_incomplete_collision_without_publication() {
    let mut fixture = Fixture::with_contributors(0);
    let original = fixture.document.clone();
    let identity = fixture.index().identity();
    fixture.document["chunks"].as_array_mut().unwrap().reverse();
    fixture.document["query_origin"] = json!([8., 25., -8.]);
    fixture.write();
    assert_eq!(fixture.index().identity(), identity);
    fixture.document = original.clone();
    fixture.document["chunks"][1]["bounds"][1][0] = json!(134.);
    fixture.write();
    assert_ne!(fixture.index().identity(), identity);
    fixture.document = original.clone();
    fixture.document["chunks"][1]["bounds"][1][0] = json!(128.);
    fixture.write();
    let mut resident = Residency::new(fixture.index(), 2).unwrap();
    resident.replace(&[fixture.id(1)], &[]).unwrap();
    let generation = resident.generation();
    assert!(
        resident
            .replace(&[fixture.id(0), fixture.id(1)], &[])
            .unwrap_err()
            .to_string()
            .contains("Collision exceeds")
    );
    assert_eq!(resident.generation(), generation);
    assert_eq!(resident.metrics().loads, 1);
    assert_eq!(resident.loaded(), vec![fixture.id(1)]);
    for bounds in [
        json!([[1., 0.], [0., 128.]]),
        json!([[-1024., 0.], [128., 128.]]),
        Value::Null,
    ] {
        fixture.document = original.clone();
        fixture.document["chunks"][1]["bounds"] = bounds;
        fixture.write();
        assert!(Index::load(&fixture.root).is_err());
    }
    fixture.document = original.clone();
    fixture.document["schema"] = json!(1);
    fixture.write();
    assert!(
        Index::load(&fixture.root).is_err(),
        "No schema downgrade with contributor bounds"
    );
    fixture.document = original;
    fixture.document["chunks"] = json!((0..6).flat_map(|x| (0..6).map(move |z| json!({"id":[x,z],"package":vec![0u8;32],"bounds":[[-900.,-900.],[900.,900.]]}))).collect::<Vec<_>>());
    fixture.write();
    assert!(
        Index::load(&fixture.root)
            .err()
            .unwrap()
            .to_string()
            .contains("contributors exceed")
    );
}
