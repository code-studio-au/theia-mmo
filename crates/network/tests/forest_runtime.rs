//! Explicit private-map probes; never a substitute for ordinary fixture tests.
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    chunks::{Index, Residency},
    kinematic::{self, Motion},
    wire::Intent,
};

#[test]
#[ignore = "requires explicitly staged private full-scene forest packages"]
fn full_scene_capsule_route_survey() {
    let directory = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_RUNTIME_WORLD").expect("Set THEIA_FOREST_RUNTIME_WORLD"),
    );
    let report = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_RUNTIME_REPORT").expect("Set THEIA_FOREST_RUNTIME_REPORT"),
    );
    assert!(!report.exists(), "Use a fresh survey report");
    let index = Index::load(&directory).unwrap();
    let ids = index.ids().collect::<Vec<_>>();
    let mut residency = Residency::new(index, ids.len()).unwrap();
    residency.replace(&ids, &[]).unwrap();
    let scene = residency.scene().unwrap();
    let mut routes = Vec::new();
    let mut rejected = Vec::new();
    let mut checked = 0;
    for x in (-28..=28).step_by(4) {
        for z in (-28..=28).step_by(4) {
            checked += 1;
            let Some((y, true)) = scene.ground([x as f32, z as f32], 12.) else {
                continue;
            };
            let Ok(start) =
                kinematic::spawn(scene, LocalPosition::new([x as f32, y, z as f32]).unwrap())
            else {
                continue;
            };
            let mut ends = Vec::new();
            for axes in [[1000, 0], [0, -1000]] {
                let mut feet = start;
                let mut motion = Motion::default();
                let mut failure = None;
                for tick in 0..60 {
                    let advanced = kinematic::advance(
                        scene,
                        &mut motion,
                        feet,
                        Intent {
                            axes,
                            jump: false,
                            sprint: false,
                        },
                        TickRate::from_hz(30).unwrap(),
                    );
                    match advanced {
                        Ok(next) => feet = next,
                        Err(error) => {
                            failure = Some((tick, error));
                            break;
                        }
                    }
                    if let Err(error) = kinematic::validate_pose(scene, feet, motion.grounded()) {
                        failure = Some((tick, error));
                        break;
                    }
                }
                if let Some((tick, error)) = failure {
                    rejected.push(serde_json::json!({"start":start.metres(),"axes":axes,"tick":tick,"feet":feet.metres(),"error":error}));
                }
                ends.push(serde_json::json!({"axes":axes,"feet":feet.metres(),"grounded":motion.grounded(),
                    "valid":failure.is_none(),
                    "solver_rollbacks":motion.solver_rollbacks(),
                    "distance":(feet.metres()[0]-start.metres()[0]).hypot(feet.metres()[2]-start.metres()[2])}));
            }
            if ends.iter().all(|e| {
                e["distance"].as_f64().unwrap() > 4. && e["grounded"] == true && e["valid"] == true
            }) {
                routes.push(serde_json::json!({"start":start.metres(),"ends":ends}));
            }
        }
    }
    std::fs::write(&report, serde_json::to_vec_pretty(&serde_json::json!({
        "survey_completed":true,"grid_points":checked,"routes":routes,"rejected":rejected,"packages":ids.len(),
        "static_colliders":scene.colliders.len(),"platform":std::env::consts::OS,
        "index_sha256":theia_network::terrain::sha256(&std::fs::read(directory.join("world.chunks.json")).unwrap()),
        "scope":"Survey of clear two-second capsule routes through actual package collision; no water policy, navmesh or complete map coverage claim"
    })).unwrap()).unwrap();
    assert!(!routes.is_empty(), "No clear two-direction source route");
}

#[test]
#[ignore = "requires private full-scene packages and a recorded rejected-route survey"]
fn recorded_full_scene_routes_return_valid_poses_at_all_fixed_rates() {
    #[derive(serde::Deserialize)]
    struct Seed {
        start: [f32; 3],
        axes: [i16; 2],
    }
    #[derive(serde::Deserialize)]
    struct Survey {
        rejected: Vec<Seed>,
    }
    let directory = std::path::PathBuf::from(std::env::var("THEIA_FOREST_RUNTIME_WORLD").unwrap());
    let input = std::path::PathBuf::from(std::env::var("THEIA_FOREST_REJECTED_SURVEY").unwrap());
    let output = std::path::PathBuf::from(std::env::var("THEIA_FOREST_RUNTIME_REPORT").unwrap());
    assert!(!output.exists(), "Use a fresh replay report");
    let bytes = std::fs::read(&input).unwrap();
    assert!(bytes.len() < 4 * 1024 * 1024);
    let seeds: Survey = serde_json::from_slice(&bytes).unwrap();
    assert!((1..=512).contains(&seeds.rejected.len()));
    let index = Index::load(&directory).unwrap();
    let ids = index.ids().collect::<Vec<_>>();
    let mut residency = Residency::new(index, ids.len()).unwrap();
    residency.replace(&ids, &[]).unwrap();
    let scene = residency.scene().unwrap();
    let mut cases = Vec::new();
    for seed in seeds.rejected {
        for hz in [20, 30, 60] {
            for jump in [false, true] {
                let mut feet = LocalPosition::new(seed.start).unwrap();
                let mut motion = Motion::default();
                kinematic::validate_pose(scene, feet, false).unwrap();
                for tick in 0..hz * 2 {
                    feet = kinematic::advance(
                        scene,
                        &mut motion,
                        feet,
                        Intent {
                            axes: seed.axes,
                            jump: jump && tick < hz / 4,
                            sprint: false,
                        },
                        TickRate::from_hz(hz).unwrap(),
                    )
                    .unwrap_or_else(|e| {
                        panic!(
                            "{e}; hz={hz} jump={jump} start={:?} tick={tick}",
                            seed.start
                        )
                    });
                    kinematic::validate_pose(scene, feet, motion.grounded()).unwrap_or_else(|e| {
                        panic!(
                            "{e}; hz={hz} jump={jump} start={:?} tick={tick}",
                            seed.start
                        )
                    });
                }
                cases.push(serde_json::json!({"hz":hz,"jump":jump,"start":seed.start,"axes":seed.axes,
                    "feet":feet.metres(),"grounded":motion.grounded(),"solver_rollbacks":motion.solver_rollbacks()}));
            }
        }
    }
    std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({
        "passed":true,"source_survey_sha256":theia_network::terrain::sha256(&bytes),
        "index_sha256":theia_network::terrain::sha256(&std::fs::read(directory.join("world.chunks.json")).unwrap()),
        "cases":cases,"scope":"Recorded source rejections replayed at 20/30/60 Hz, walking and jump; every returned pose checked; blocking allowed, no complete-map or capacity claim"
    })).unwrap()).unwrap();
}
