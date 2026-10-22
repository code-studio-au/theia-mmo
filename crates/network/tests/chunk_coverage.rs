//! Address-envelope and actual Avian controller tests. The index-only entries
//! below do not claim package loading or rendered scene publication.
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition, LocalPosition},
    time::TickRate,
};
use theia_network::{
    chunks::Index,
    kinematic::{self, Motion},
    physics::{GROUND, PlacedCollider, SOLID, StaticScene},
    wire::Intent,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    base: i64,
    index: Index,
}
impl Fixture {
    fn new(base: i64) -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-coverage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let chunks = [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)]
            .map(|(x, z)| serde_json::json!({"id":[base+x,base+z],"package":vec![0u8;32]}));
        std::fs::write(root.join("world.chunks.json"),serde_json::to_vec(&serde_json::json!({"schema":1,"chunk_metres":128,"query_origin":[base as f64*128.+8.,25.,base as f64*128.-8.],"chunks":chunks})).unwrap()).unwrap();
        let index = Index::load(&root).unwrap();
        Self { root, base, index }
    }
    fn id(&self, x: i64, z: i64) -> ChunkId {
        ChunkId {
            x: self.base + x,
            z: self.base + z,
        }
    }
    fn point(&self, x: f32, z: f32) -> GlobalPosition {
        GlobalPosition::new([
            self.base as f64 * 128. + f64::from(x),
            0.,
            self.base as f64 * 128. + f64::from(z),
        ])
        .unwrap()
    }
    fn local(&self, x: f32, z: f32) -> Vec3 {
        Vec3::from_array(
            self.index
                .frame()
                .to_local(self.point(x, z))
                .unwrap()
                .metres(),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn bounded_coverage_rejects_unknown_duplicate_and_excess_ids_and_keeps_vertical_motion() {
    let f = Fixture::new(0);
    assert!(f.index.coverage([f.id(0, 0), f.id(0, 0)]).is_err());
    assert!(f.index.coverage([f.id(9, 9)]).is_err());
    assert!(
        f.index
            .coverage([f.id(0, 0), f.id(1, 0), f.id(2, 0), f.id(0, 1), f.id(1, 1)])
            .is_ok()
    );
    let empty = f.index.coverage([]).unwrap();
    assert!(!empty.contains(f.point(16., 16.)));
    assert!(empty.sweep(f.local(16., 16.), Vec3::NEG_Y * 10.).is_none());
    let hit = empty
        .sweep(f.local(16., 16.), Vec3::new(1., -10., 1.))
        .unwrap();
    assert_eq!(hit.distance, 0.);
    assert_eq!(hit.normal.y, 0.);
}

#[test]
fn continuous_coverage_sweeps_do_not_shortcut_gaps_and_allow_shared_seams_at_distant_frames() {
    for base in [-7_812_500, 0, 7_812_500] {
        let f = Fixture::new(base);
        let disconnected = f.index.coverage([f.id(0, 0), f.id(2, 0)]).unwrap();
        assert!(disconnected.contains(f.point(64., 32.)));
        assert!(disconnected.contains(f.point(320., 32.)));
        let hit = disconnected
            .sweep(f.local(64., 32.), Vec3::X * 256.)
            .unwrap();
        assert!((hit.distance - (128. - kinematic::FOOTPRINT_RADIUS - 64.)).abs() < 0.0001);
        assert_eq!(hit.normal, Vec3::NEG_X);
        let joined = f.index.coverage([f.id(0, 0), f.id(1, 0)]).unwrap();
        assert!(joined.contains(f.point(128., 32.)));
        assert!(joined.sweep(f.local(126., 32.), Vec3::X * 5.).is_none());
        let square = f
            .index
            .coverage([f.id(0, 0), f.id(1, 0), f.id(0, 1), f.id(1, 1)])
            .unwrap();
        assert!(square.contains(f.point(128., 128.)));
        assert!(
            square
                .sweep(f.local(126., 126.), Vec3::new(5., 0., 5.))
                .is_none()
        );
    }
}

#[test]
fn actual_controller_slides_around_missing_corner_without_leaving_full_footprint_coverage() {
    for base in [-7_812_500, 0, 7_812_500] {
        for hz in [20, 30, 60] {
            let f = Fixture::new(base);
            let coverage = f
                .index
                .coverage([f.id(0, 0), f.id(1, 0), f.id(1, 1)])
                .unwrap();
            let scene = StaticScene::new(vec![PlacedCollider {
                shape: Collider::cuboid(512., 1., 512.),
                center: f.local(128., 128.) - Vec3::Y * 0.5,
                layer: GROUND | SOLID,
            }]);
            let mut body = Motion::default();
            let mut feet = LocalPosition::new(f.local(130., 130.).to_array()).unwrap();
            for tick in 0..hz * 2 {
                feet = kinematic::advance_bounded(
                    &scene,
                    &mut body,
                    feet,
                    Intent {
                        axes: [-1000, -1000],
                        jump: false,
                        sprint: false,
                    },
                    TickRate::from_hz(hz).unwrap(),
                    Some(&coverage),
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "{base} {hz} tick {tick} feet {:?} body {:?}: {error}",
                        feet.metres(),
                        body
                    )
                });
                assert!(
                    coverage.contains(f.index.frame().to_global(feet).unwrap()),
                    "{base} {hz}: {:?}",
                    feet.metres()
                );
            }
            assert!(
                feet.metres()[0] < f.local(127., 127.).x
                    && feet.metres()[2] < f.local(127., 127.).z,
                "Must get around the corner using ordinary movement"
            );
            assert!(body.grounded());
        }
    }
}
