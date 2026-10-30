//! Actual offline generator/query integration, not MMO gameplay.
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame, LocalPosition};
use theia_nav_bake::{Query, bake};
use theia_network::{navigation::Agent, procedural::Geometry};
fn empty() -> Geometry {
    Geometry {
        positions: vec![],
        triangles: vec![],
    }
}
fn quad(g: &mut Geometry, p: [[f32; 3]; 4]) {
    let b = g.positions.len() as u32;
    g.positions.extend(p);
    g.triangles.extend([[b, b + 1, b + 2], [b, b + 2, b + 3]]);
}
fn floor(width: f32) -> Geometry {
    let mut g = empty();
    quad(
        &mut g,
        [
            [0., 0., 0.],
            [0., 0., 128.],
            [width, 0., 128.],
            [width, 0., 0.],
        ],
    );
    g
}
fn block(g: &mut Geometry, lo: [f32; 3], hi: [f32; 3]) {
    let [x, y, z] = lo;
    let [a, b, c] = hi;
    quad(g, [[x, b, z], [x, b, c], [a, b, c], [a, b, z]]);
    quad(g, [[x, y, z], [a, y, z], [a, y, c], [x, y, c]]);
    quad(g, [[x, y, z], [x, y, c], [x, b, c], [x, b, z]]);
    quad(g, [[a, y, z], [a, b, z], [a, b, c], [a, y, c]]);
    quad(g, [[x, y, z], [x, b, z], [a, b, z], [a, y, z]]);
    quad(g, [[x, y, c], [a, y, c], [a, b, c], [x, b, c]]);
}
fn point(id: ChunkId, x: f32, y: f32, z: f32) -> GlobalPosition {
    LocalFrame::new(id.origin(128).unwrap())
        .to_global(LocalPosition::new([x, y, z]).unwrap())
        .unwrap()
}
#[test]
fn generated_route_detours_static_solid_and_rejects_invalid_geometry_before_backend_calls() {
    let id = ChunkId { x: 62, z: 62 };
    let agent = Agent::development();
    let mut g = floor(128.);
    block(&mut g, [60., 0., 32.], [68., 3., 96.]);
    let tile = bake(id, [7; 32], agent, g).unwrap();
    tile.validate().unwrap();
    let query = Query::new(std::slice::from_ref(&tile)).unwrap();
    let route = query
        .route(point(id, 16., 0., 64.), point(id, 112., 0., 64.))
        .unwrap();
    let frame = LocalFrame::new(id.origin(128).unwrap());
    assert!(route.len() >= 4);
    assert!(route.iter().any(|p| {
        let z = frame.to_local(*p).unwrap().metres()[2];
        !(31.7..=96.3).contains(&z)
    }));
    assert!(query.sample(point(id, 60., 0., 64.)).is_err());
    let mut invalid = floor(128.);
    invalid.positions[0][0] = f32::NAN;
    assert!(bake(id, [7; 32], agent, invalid).is_err());
    let mut invalid = floor(128.);
    invalid.triangles[0][0] = 99;
    assert!(bake(id, [7; 32], agent, invalid).is_err());
    assert!(Query::new(&[tile.clone(), tile]).is_err());
}
#[test]
fn actual_128m_seam_is_connected_at_15km_and_a_missing_tile_has_no_fallback() {
    let a = ChunkId { x: 117, z: 117 };
    let b = ChunkId { x: 118, z: 117 };
    let left = bake(a, [7; 32], Agent::development(), floor(256.)).unwrap();
    let mut geometry = floor(256.);
    for p in &mut geometry.positions {
        p[0] -= 128.;
    }
    let right = bake(b, [7; 32], Agent::development(), geometry).unwrap();
    let query = Query::new(&[left.clone(), right.clone()]).unwrap();
    let route = query
        .route(point(a, 16., 0., 64.), point(b, 112., 0., 64.))
        .unwrap();
    assert!((2..=256).contains(&route.len()));
    assert!(route.first().unwrap().metres()[0] > 14976.);
    let missing = Query::new(std::slice::from_ref(&left)).unwrap();
    assert!(
        missing
            .route(point(a, 16., 0., 64.), point(b, 112., 0., 64.))
            .is_err()
    );
    let mut foreign = right;
    foreign.source = [8; 32];
    assert!(Query::new(&[left, foreign]).is_err());
}
#[test]
fn forbidden_slope_is_disconnected_and_low_headroom_cannot_snap_onto_roof() {
    let id = ChunkId { x: 0, z: 0 };
    let agent = Agent::development();
    let mut g = empty();
    quad(
        &mut g,
        [[0., 0., 0.], [0., 0., 20.], [6., 0., 20.], [6., 0., 0.]],
    );
    quad(
        &mut g,
        [[6., 0., 0.], [6., 0., 20.], [12., 8., 20.], [12., 8., 0.]],
    );
    quad(
        &mut g,
        [[12., 8., 0.], [12., 8., 20.], [20., 8., 20.], [20., 8., 0.]],
    );
    let steep = bake(id, [7; 32], agent, g).unwrap();
    let steep = Query::new(&[steep]).unwrap();
    assert!(steep.sample(point(id, 2., 0., 10.)).is_ok());
    assert!(steep.sample(point(id, 18., 8., 10.)).is_ok());
    assert!(
        steep
            .route(point(id, 2., 0., 10.), point(id, 18., 8., 10.))
            .is_err()
    );
    let mut g = empty();
    quad(
        &mut g,
        [[0., 0., 0.], [0., 0., 20.], [20., 0., 20.], [20., 0., 0.]],
    );
    block(&mut g, [0., 1.2, 0.], [20., 1.4, 20.]);
    let low = bake(id, [7; 32], agent, g).unwrap();
    let low = Query::new(&[low]).unwrap();
    assert!(low.sample(point(id, 10., 0., 10.)).is_err());
    assert!(low.sample(point(id, 10., 1.4, 10.)).is_ok());
}
