//! Opt-in checks against private full-resolution forest ground packages.
use theia_core::{coordinates::GlobalPosition, time::TickRate};
use theia_network::{
    chunk_world::{ChunkWorld, Config},
    kinematic::{self, Motion},
    simulation::World,
    wire::Intent,
};

#[test]
#[ignore = "requires explicitly staged private forest source and ground packages"]
fn source_ground_queries_and_cross_chunk_motion_match_full_sample_packages() {
    let directory = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_TERRAIN_WORLD").expect("Set THEIA_FOREST_TERRAIN_WORLD"),
    );
    let source = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_TERRAIN_SOURCE").expect("Set THEIA_FOREST_TERRAIN_SOURCE"),
    );
    let config = Config::load(&directory).unwrap();
    assert_eq!(config.budget(), 4);
    assert_eq!(config.initial().len(), 4);
    assert_eq!(config.index().ids().count(), 4);
    let (residency, resolved) = config.prepare_startup().unwrap();
    assert_eq!(resolved.len(), 2);
    let raw = std::fs::read(source.join("heights.u16")).unwrap();
    assert_eq!(raw.len(), 513 * 513 * 2);
    let heights = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&v| u16::from_le_bytes(v))
        .collect::<Vec<_>>();
    let mut maximum_error = 0f32;
    let scene = residency.scene().unwrap();
    for row in (0..513).step_by(64) {
        for col in (0..513).step_by(64) {
            let x = 75. - col as f32 * 0.29296875;
            let z = -75. + row as f32 * 0.29296875;
            let expected = -20. + f32::from(heights[row * 513 + col]) / 32766. * 100.;
            let (actual, _) = scene.ground([x, z], 32.).unwrap();
            maximum_error = maximum_error.max((actual - expected).abs());
        }
    }
    assert!(
        maximum_error < 0.0001,
        "Source ground error {maximum_error}"
    );
    let mut cases = 0;
    for hz in [20, 30, 60] {
        for (start, axes) in [
            ([-2., -2.], [1000, 0]),
            ([-2., 2.], [0, -1000]),
            ([-1., -1.], [1000, 1000]),
        ] {
            for jump in [false, true] {
                let y = scene.ground(start, 32.).unwrap().0;
                let guess =
                    GlobalPosition::new([f64::from(start[0]), f64::from(y), f64::from(start[1])])
                        .unwrap();
                let mut world = ChunkWorld::with_bounds(
                    residency.clone(),
                    &[guess],
                    config.fall_plane(),
                    config.world_bounds(),
                )
                .unwrap();
                let mut feet = world.spawn(0);
                let initial = feet.metres();
                let mut body = Motion::default();
                let mut peak = initial[1];
                for tick in 0..hz * 3 {
                    let p = feet.metres();
                    let distance = (p[0] - initial[0]).hypot(p[2] - initial[2]);
                    feet = world
                        .advance(
                            &mut body,
                            feet,
                            Intent {
                                axes: if distance < 4. { axes } else { [0, 0] },
                                // Launch from the validated support before the
                                // route descends onto uneven source terrain.
                                jump: jump && tick < hz / 4,
                                sprint: false,
                            },
                            TickRate::from_hz(hz).unwrap(),
                        )
                        .unwrap();
                    peak = peak.max(feet.metres()[1]);
                    kinematic::validate_pose(scene, feet, body.grounded()).unwrap();
                }
                let final_pose = feet.metres();
                assert!(body.grounded(), "Not grounded at {hz}/{start:?}/{jump}");
                assert!(
                    (final_pose[0] - initial[0]).hypot(final_pose[2] - initial[2]) > 3.8,
                    "No full seam crossing at {hz}/{start:?}/{jump}: {final_pose:?}"
                );
                if axes[0] > 0 {
                    assert!(final_pose[0] > 0.5);
                }
                if axes[1] != 0 {
                    assert!(final_pose[2] * initial[2] < 0.);
                }
                if jump {
                    assert!(peak - initial[1] > 1., "Jump absent at {hz}/{start:?}");
                }
                cases += 1;
            }
        }
    }
    // Use the actual authority, with two independently admitted nonblocking actors.
    let mut authority = World::with_chunk_config(&config).unwrap();
    let a = authority.join().unwrap();
    let b = authority.join().unwrap();
    assert_ne!(a, b);
    for _ in 0..90 {
        authority.step();
    }
    assert_eq!(authority.movement_recoveries(), 0);
    println!(
        "forest_ground_pass packages=4 samples=263169 triangles=524288 source_probes=81 max_height_error={maximum_error} motion_cases={cases} authority_actors=2 recoveries=0"
    );
}
