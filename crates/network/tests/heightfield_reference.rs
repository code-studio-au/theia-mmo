//! Standard Avian heightfield representation; runtime content stays unchanged.
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use serde::Deserialize;
use serde_json::json;
use std::time::Instant;
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    chunk_world::Config,
    kinematic::{self, Motion},
    physics::{CAMERA, GROUND, PlacedCollider, SOLID, StaticScene},
    wire::Intent,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    schema: u32,
    profile: String,
    source_sha256: String,
    rows: usize,
    columns: usize,
    scale: [f32; 3],
    center: [f32; 3],
    heights_column_major: Vec<f32>,
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
    triangle_centers: Vec<[f32; 3]>,
}

fn field(
    rows: usize,
    columns: usize,
    values: Vec<f32>,
    scale: Vec3,
    center: Vec3,
) -> PlacedCollider {
    assert!(rows >= 2 && columns >= 2);
    assert_eq!(values.len(), rows * columns);
    // Avian forwards this flat data to Parry's column-major Array2. The nested
    // Vec dimensions carry row/column counts; they do not change flat ordering.
    let wrapped = values.chunks_exact(columns).map(<[f32]>::to_vec).collect();
    PlacedCollider {
        shape: Collider::heightfield(wrapped, scale),
        center,
        layer: GROUND | SOLID | CAMERA,
    }
}

#[test]
fn rectangular_heightfields_keep_axis_order_and_default_diagonal() {
    let rows = 3;
    let columns = 5;
    let mut values = Vec::new();
    let mut points = Vec::new();
    for x in 0..columns {
        for z in 0..rows {
            let px = x as f32 * 1.5;
            let pz = -9. + z as f32 * 2.;
            let y = 0.1 * px * px + 0.2 * pz * pz + 0.05 * px * pz;
            values.push(y);
            points.push(Vec3::new(px, y, pz));
        }
    }
    let scene = StaticScene::new(vec![field(
        rows,
        columns,
        values,
        Vec3::new(6., 1., 4.),
        Vec3::new(3., 0., -7.),
    )]);
    let mut samples = points.clone();
    for x in 0..columns - 1 {
        for z in 0..rows - 1 {
            let p00 = points[x * rows + z];
            let p10 = points[x * rows + z + 1];
            let p01 = points[(x + 1) * rows + z];
            let p11 = points[(x + 1) * rows + z + 1];
            samples.extend([(p00 + p10 + p01) / 3., (p10 + p11 + p01) / 3.]);
        }
    }
    for p in samples {
        let actual = scene
            .ground([p.x, p.z], 64.)
            .expect("Inclusive heightfield surface")
            .0;
        assert!((actual - p.y).abs() < 0.0001, "{p:?}: {actual}");
    }
}

fn motion(scene: &StaticScene) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    for hz in [20, 30, 60] {
        let rate = TickRate::from_hz(hz).unwrap();
        for (start, axes) in [
            ([-2., -2.], [1000, 0]),
            ([-2., 2.], [0, -1000]),
            ([-1., -1.], [1000, 1000]),
        ] {
            for jump in [false, true] {
                let y = scene.ground(start, 32.).unwrap().0;
                let mut feet =
                    kinematic::spawn(scene, LocalPosition::new([start[0], y, start[1]]).unwrap())
                        .unwrap();
                let initial = feet.metres();
                let mut body = Motion::default();
                let mut peak = initial[1];
                for tick in 0..hz * 3 {
                    let p = feet.metres();
                    let distance = (p[0] - initial[0]).hypot(p[2] - initial[2]);
                    feet = kinematic::advance(
                        scene,
                        &mut body,
                        feet,
                        Intent {
                            axes: if distance < 4. { axes } else { [0, 0] },
                            jump: jump && tick < hz / 4,
                            sprint: false,
                        },
                        rate,
                    )
                    .unwrap();
                    peak = peak.max(feet.metres()[1]);
                    kinematic::validate_pose(scene, feet, body.grounded()).unwrap();
                }
                let final_pose = feet.metres();
                assert!(body.grounded(), "{hz}/{start:?}/{jump}: {final_pose:?}");
                assert!((final_pose[0] - initial[0]).hypot(final_pose[2] - initial[2]) > 3.8);
                if axes[0] > 0 {
                    assert!(final_pose[0] > 0.5);
                }
                if axes[1] != 0 {
                    assert!(final_pose[2] * initial[2] < 0.);
                }
                if jump {
                    assert!(peak - initial[1] > 1.);
                }
                results.push(
                    json!({"hz":hz,"start":initial,"jump":jump,"final":final_pose,"peak":peak}),
                );
            }
        }
    }
    results
}

#[test]
#[ignore = "requires private full-resolution terrain and retained ground packages"]
fn complete_source_heightfields_and_seams_match_the_retained_ground() {
    let path = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_HEIGHTFIELD_REFERENCE")
            .expect("Set THEIA_FOREST_HEIGHTFIELD_REFERENCE"),
    );
    let output = std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_HEIGHTFIELD_REPORT")
            .expect("Set THEIA_FOREST_HEIGHTFIELD_REPORT"),
    );
    assert!(!output.exists(), "Use a fresh heightfield report");
    assert!(std::fs::metadata(&path).unwrap().len() <= 128 * 1024 * 1024);
    let bytes = std::fs::read(&path).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.with_extension("receipt.json")).unwrap())
            .unwrap();
    let hash = theia_network::terrain::sha256(&bytes)
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect::<String>();
    assert_eq!(receipt["fixture_sha256"], hash);
    let source: Reference = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(source.schema, 1);
    assert_eq!(source.profile, "source-full-grid-heightfield-reference-v1");
    assert_eq!(receipt["source_sha256"], source.source_sha256);
    assert_eq!((source.rows, source.columns), (513, 513));
    assert_eq!(
        source.heights_column_major.len(),
        source.rows * source.columns
    );
    assert_eq!(source.vertices.len(), 263169);
    assert_eq!(source.indices.len(), 524288);
    assert_eq!(source.triangle_centers.len(), source.indices.len());
    assert!(
        source
            .vertices
            .iter()
            .flatten()
            .chain(source.heights_column_major.iter())
            .all(|v| v.is_finite() && v.abs() <= 4096.)
    );
    assert!(
        source
            .indices
            .iter()
            .flatten()
            .all(|&v| (v as usize) < source.vertices.len())
    );
    let started = Instant::now();
    let full = StaticScene::new(vec![field(
        source.rows,
        source.columns,
        source.heights_column_major.clone(),
        Vec3::from_array(source.scale),
        Vec3::from_array(source.center),
    )]);
    let full_preparation_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    let mut quarters = Vec::new();
    for x in 0..2 {
        for z in 0..2 {
            let values = (0..257)
                .flat_map(|cx| (0..257).map(move |cz| (x * 256 + cx) * source.rows + z * 256 + cz))
                .map(|i| source.heights_column_major[i])
                .collect();
            let mut center = Vec3::from_array(source.center);
            center.x += (x as f32 - 0.5) * source.scale[0] * 0.5;
            center.z += (z as f32 - 0.5) * source.scale[2] * 0.5;
            quarters.push(field(
                257,
                257,
                values,
                Vec3::new(
                    source.scale[0] * 0.5,
                    source.scale[1],
                    source.scale[2] * 0.5,
                ),
                center,
            ));
        }
    }
    let quartered = StaticScene::new(quarters);
    let quarter_preparation_ms = started.elapsed().as_secs_f64() * 1000.;
    let config = Config::load(&std::path::PathBuf::from(
        std::env::var("THEIA_FOREST_TERRAIN_WORLD").expect("Set THEIA_FOREST_TERRAIN_WORLD"),
    ))
    .unwrap();
    let (residency, _) = config.prepare_startup().unwrap();
    let mut results = Vec::new();
    let mut passed = true;
    for (label, scene) in [
        ("retained-four-trimeshes", residency.scene().unwrap()),
        ("single-heightfield", &full),
        ("four-heightfields", &quartered),
    ] {
        let started = Instant::now();
        let mut maximum = 0f32;
        let mut missed = 0;
        let mut excessive = 0;
        for point in source.vertices.iter().chain(&source.triangle_centers) {
            if let Some((actual, _)) = scene.ground([point[0], point[2]], 32.) {
                let error = (actual - point[1]).abs();
                maximum = maximum.max(error);
                excessive += usize::from(error > 0.0001);
            } else {
                missed += 1;
            }
        }
        let query_ms = started.elapsed().as_secs_f64() * 1000.;
        let movement = motion(scene);
        passed &= missed == 0 && excessive == 0;
        println!(
            "heightfield comparison {label}: missing={missed} excessive={excessive} max_error={maximum} routes={}",
            movement.len()
        );
        results.push(json!({"representation":label,"sample_queries":source.vertices.len()+source.triangle_centers.len(),"missed":missed,"over_tolerance":excessive,"max_height_error_metres":maximum,"query_ms":query_ms,"motion":movement}));
    }
    let report = json!({"passed":passed,"fixture_sha256":hash,"source_sha256":source.source_sha256,"full_preparation_ms":full_preparation_ms,"quartered_preparation_ms":quarter_preparation_ms,"results":results,"scope":"All source vertices and triangle centroids, inclusive boundaries, real 20/30/60 Hz capsule seam routes; standard Avian heightfields vs retained terrain packages; no runtime replacement or Unity TerrainCollider parity"});
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
        .unwrap();
    assert!(
        passed,
        "Heightfield source-surface differences; inspect the report"
    );
}
