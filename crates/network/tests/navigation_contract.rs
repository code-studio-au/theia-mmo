//! Offline artifact contracts only. No runtime pathfinding or creature claim.
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_network::{
    navigation::{self as nav, Agent, Manifest, Route, Store, Tile, TileRecord},
    terrain::sha256,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn tile() -> Tile {
    Tile {
        schema: 1,
        id: [62, 62],
        source: [7; 32],
        agent: Agent::development(),
        vertices: vec![
            [0., 0., 0.],
            [0., 0., 128.],
            [128., 0., 128.],
            [128., 0., 0.],
        ],
        polygons: vec![vec![0, 1, 2, 3]],
        detail: nav::Detail {
            vertices: vec![
                [0., 0., 0.],
                [0., 0., 128.],
                [128., 0., 128.],
                [128., 0., 0.],
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            polygons: vec![[0, 4, 0, 2]],
        },
    }
}
fn manifest(bytes: &[u8]) -> Manifest {
    Manifest {
        schema: 1,
        source: [7; 32],
        agent: Agent::development(),
        tiles: vec![TileRecord {
            id: [62, 62],
            sha256: sha256(bytes),
        }],
        routes: vec![Route {
            id: "theia.dev.patrol".into(),
            points: vec![[8000., 0., 8000.], [8010., 0., 8010.]],
        }],
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "theia-nav-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn tile_rejects_unsupported_clearance_indices_winding_nonconvexity_and_global_vertices() {
    let good = tile();
    let bytes = good.encode().unwrap();
    Tile::decode(&bytes).unwrap();
    for mutation in 0..10 {
        let mut bad = good.clone();
        match mutation {
            0 => bad.schema = 2,
            1 => bad.id = [-1, 62],
            2 => bad.agent.radius = 0.1,
            3 => bad.agent.height = 0.5,
            4 => bad.agent.step = 0.5,
            5 => bad.agent.slope_radians = 55_f32.to_radians(),
            6 => bad.vertices[0][0] = 8000.,
            7 => bad.polygons[0][0] = 99,
            8 => bad.polygons[0].reverse(),
            9 => bad.vertices[1] = [100., 0., 28.],
            _ => unreachable!(),
        }
        assert!(bad.encode().is_err(), "mutation {mutation}");
    }
    let mut bad = good.clone();
    bad.polygons[0] = vec![0, 1, 1, 3];
    assert!(bad.encode().is_err());
    let mut bad = good.clone();
    bad.vertices[0][1] = f32::NAN;
    assert!(bad.encode().is_err());
    let mut bad = good.clone();
    bad.polygons = vec![vec![0, 1, 2, 3]; nav::MAX_POLYGONS + 1];
    assert!(bad.encode().is_err());
    let mut bad = serde_json::to_value(&good).unwrap();
    bad["unknown"] = true.into();
    assert!(Tile::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert!(Tile::decode(&vec![b' '; nav::MAX_TILE_BYTES + 1]).is_err());
}
#[test]
fn bounded_route_manifest_rejects_source_missing_tiles_duplicates_and_stationary_routes() {
    let bytes = tile().encode().unwrap();
    let good = manifest(&bytes);
    let encoded = good.encode().unwrap();
    Manifest::decode(&encoded, [7; 32]).unwrap();
    assert!(Manifest::decode(&encoded, [8; 32]).is_err());
    for mutation in 0..7 {
        let mut bad = good.clone();
        match mutation {
            0 => bad.tiles.push(bad.tiles[0].clone()),
            1 => bad.routes[0].points[1][0] = 9000.,
            2 => bad.routes[0].points[1][1] = f64::NAN,
            3 => bad.routes[0].points = vec![[8000., 0., 8000.]; nav::MAX_ROUTE_POINTS + 1],
            4 => bad.routes.push(bad.routes[0].clone()),
            5 => bad.routes[0].id = "../outside".into(),
            6 => bad.routes[0].points[1] = bad.routes[0].points[0],
            _ => unreachable!(),
        }
        assert!(bad.encode().is_err(), "mutation {mutation}");
    }
    assert!(Manifest::decode(&vec![b' '; nav::MAX_MANIFEST_BYTES + 1], [7; 32]).is_err());
}
#[test]
fn store_lazily_verifies_tile_hash_identity_profile_and_coordinates_before_publication() {
    let root = Temp::new();
    let mut data = tile();
    let mut bytes = data.encode().unwrap();
    fs::write(root.0.join(nav::tile_name(data.id)), &bytes).unwrap();
    fs::write(
        root.0.join(nav::MANIFEST),
        manifest(&bytes).encode().unwrap(),
    )
    .unwrap();
    let store = Store::open(&root.0, [7; 32]).unwrap();
    store.verify().unwrap();
    let identity = store.identity();
    assert!(store.tile([61, 62]).is_err());
    assert!(Store::open(&root.0, [8; 32]).is_err());
    bytes[0] = b'!';
    fs::write(root.0.join(nav::tile_name(data.id)), &bytes).unwrap();
    assert!(store.verify().is_err());
    for mutation in 0..3 {
        data = tile();
        match mutation {
            0 => data.id = [63, 62],
            1 => data.source = [9; 32],
            2 => data.agent.radius = 0.5,
            _ => unreachable!(),
        }
        bytes = data.encode().unwrap();
        fs::write(root.0.join(nav::tile_name([62, 62])), &bytes).unwrap();
        fs::write(
            root.0.join(nav::MANIFEST),
            manifest(&bytes).encode().unwrap(),
        )
        .unwrap();
        let store = Store::open(&root.0, [7; 32]).unwrap();
        assert_ne!(store.identity(), identity);
        assert!(store.verify().is_err());
    }
    fs::remove_file(root.0.join(nav::tile_name([62, 62]))).unwrap();
    assert!(store.verify().is_err());
}
#[cfg(unix)]
#[test]
fn tile_symlink_cannot_escape_the_selected_package() {
    let root = Temp::new();
    let outside = Temp::new();
    let bytes = tile().encode().unwrap();
    fs::write(outside.0.join("tile"), &bytes).unwrap();
    fs::write(
        root.0.join(nav::MANIFEST),
        manifest(&bytes).encode().unwrap(),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        outside.0.join("tile"),
        root.0.join(nav::tile_name([62, 62])),
    )
    .unwrap();
    assert!(Store::open(&root.0, [7; 32]).unwrap().verify().is_err());
}

#[test]
fn detail_height_mesh_rejects_bad_counts_boundaries_triangle_indices_and_unowned_data() {
    for mutation in 0..8 {
        let mut bad = tile();
        match mutation {
            0 => bad.detail.polygons.clear(),
            1 => bad.detail.polygons[0][0] = u32::MAX,
            2 => bad.detail.polygons[0][1] = 128,
            3 => bad.detail.triangles[0][0] = 4,
            4 => bad.detail.triangles[0].reverse(),
            5 => bad.detail.vertices[0][0] = 1.,
            6 => bad.detail.vertices.push([1., 0., 1.]),
            7 => bad.detail.vertices[0][1] = f32::INFINITY,
            _ => unreachable!(),
        }
        assert!(bad.encode().is_err(), "detail mutation {mutation}");
    }
}
