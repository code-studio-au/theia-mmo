//! Opt-in actual Unity/Avian comparison, independent of runtime package admission.
use avian3d::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*, time::TimeUpdateStrategy};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mesh {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    origin: [f32; 3],
    direction: [f32; 3],
    maximum: f32,
    hit: bool,
    distance: f32,
    point: [f32; 3],
    normal: [f32; 3],
    #[serde(default)]
    normal_candidates: Vec<[f32; 3]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometryOracle {
    schema: u32,
    profile: String,
    fixture_sha256: String,
    objects: Vec<OracleObject>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleObject {
    id: String,
    probes: Vec<Probe>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileReference {
    schema: u32,
    profile: String,
    objects_sha256: String,
    heightfield_sha256: String,
    profiles: Vec<ProfileFile>,
    objects: Vec<BodyAddress>,
    terrain: Vec<FieldAddress>,
    #[serde(rename = "scope")]
    _scope: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileFile {
    owner: [i64; 2],
    file: String,
    sha256: String,
    metrics: serde_json::Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyAddress {
    id: String,
    owner: [i64; 2],
    collider: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldAddress {
    owner: [i64; 2],
    collider: usize,
}
#[derive(Deserialize)]
struct TerrainSamples {
    schema: u32,
    profile: String,
    vertices: Vec<[f32; 3]>,
    triangle_centers: Vec<[f32; 3]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Object {
    id: String,
    kind: String,
    matrix: [f32; 16],
    mesh: Option<usize>,
    convex: bool,
    center: [f32; 3],
    size: [f32; 3],
    radius: f32,
    height: f32,
    direction: usize,
    lossy_scale: [f32; 3],
    bounds_center: [f32; 3],
    bounds_size: [f32; 3],
    probes: Vec<Probe>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    profile: String,
    meshes: Vec<Mesh>,
    colliders: Vec<Object>,
}

fn matrix(values: &[f32; 16]) -> Mat4 {
    Mat4::from_cols_array_2d(&std::array::from_fn(|c| {
        std::array::from_fn(|r| values[r * 4 + c])
    }))
}
fn shape(object: &Object, meshes: &[Mesh]) -> Result<(Collider, Transform), String> {
    assert!(object.matrix.iter().all(|v| v.is_finite()));
    let matrix = matrix(&object.matrix);
    assert!(object.bounds_center.iter().all(|v| v.is_finite()));
    assert!(object.bounds_size.iter().all(|v| v.is_finite() && *v >= 0.));
    assert_eq!(object.probes.len(), 6);
    match object.kind.as_str() {
        "MeshCollider" => {
            let mesh = &meshes[object.mesh.expect("MeshCollider geometry")];
            let center = Vec3::from_array(object.bounds_center);
            let offset = matrix.w_axis.truncate() - center;
            let vertices = mesh
                .vertices
                .iter()
                .map(|v| matrix.transform_vector3(Vec3::from_array(*v)) + offset)
                .collect::<Vec<_>>();
            let collider = if object.convex {
                Collider::convex_hull(&vertices).ok_or("Convex hull construction failed")?
            } else {
                Collider::try_trimesh(vertices, mesh.indices.clone())
                    .map_err(|error| format!("Trimesh: {error:?}"))?
            };
            Ok((collider, Transform::from_translation(center)))
        }
        "BoxCollider" => {
            let columns = [
                matrix.x_axis.truncate(),
                matrix.y_axis.truncate(),
                matrix.z_axis.truncate(),
            ];
            let axes = columns.map(|v| v.normalize());
            if axes[0]
                .dot(axes[1])
                .abs()
                .max(axes[0].dot(axes[2]).abs())
                .max(axes[1].dot(axes[2]).abs())
                > 0.0001
            {
                return Err("Box shear requires separate qualification".into());
            }
            let size = Vec3::from_array(object.size) * Vec3::from_array(object.lossy_scale).abs();
            assert!(size.min_element() > 0. && size.is_finite());
            let rotation = Quat::from_mat3(&Mat3::from_cols(axes[0], axes[1], axes[2])).normalize();
            Ok((
                Collider::cuboid(size.x, size.y, size.z),
                Transform::from_translation(
                    matrix.transform_point3(Vec3::from_array(object.center)),
                )
                .with_rotation(rotation),
            ))
        }
        "CapsuleCollider" => {
            assert!(object.direction < 3 && object.radius > 0. && object.height > 0.);
            let scale = Vec3::from_array(object.lossy_scale).abs();
            let perpendicular = (0..3)
                .filter(|&i| i != object.direction)
                .map(|i| scale[i])
                .fold(0f32, f32::max);
            let radius = object.radius * perpendicular;
            let height = object.height * scale[object.direction];
            let mut axis = Vec3::ZERO;
            axis[object.direction] = 1.;
            let direction = matrix.transform_vector3(axis).normalize();
            let center = matrix.transform_point3(Vec3::from_array(object.center));
            let half = (height - 2. * radius).max(0.) * 0.5;
            Ok((
                Collider::capsule_endpoints(
                    radius,
                    center - direction * half,
                    center + direction * half,
                ),
                Transform::IDENTITY,
            ))
        }
        _ => Err("Unknown source object collider kind".into()),
    }
}

#[test]
#[ignore = "requires actual private Unity scene and geometry reference exports"]
fn actual_unity_object_rays_match_a_native_avian_world() {
    compare(false, false);
}

#[test]
#[ignore = "requires actual private source geometry and independent surface oracle"]
fn avian_object_rays_match_independent_source_geometry() {
    compare(true, false);
}

#[test]
#[ignore = "requires private source shape profiles, geometry oracle and height samples"]
fn schema4_reader_matches_actual_source_objects_and_full_sample_terrain() {
    compare(true, true);
}

fn compare(geometry: bool, profiles: bool) {
    let path = std::path::PathBuf::from(
        std::env::var("THEIA_UNITY_COLLISION_REFERENCE")
            .expect("Set THEIA_UNITY_COLLISION_REFERENCE"),
    );
    let output = std::path::PathBuf::from(
        std::env::var("THEIA_UNITY_COLLISION_REPORT").expect("Set THEIA_UNITY_COLLISION_REPORT"),
    );
    assert!(!output.exists(), "Use a fresh comparison report");
    assert!(std::fs::metadata(&path).unwrap().len() <= 128 * 1024 * 1024);
    let bytes = std::fs::read(&path).unwrap();
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.with_extension("receipt.json")).unwrap())
            .unwrap();
    let digest = theia_network::terrain::sha256(&bytes)
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect::<String>();
    assert_eq!(receipt["fixture_sha256"], digest);
    let mut document: Document = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(document.schema, 1);
    assert_eq!(document.profile, "unity-object-collision-comparison-v1");
    assert!((1..=128).contains(&document.meshes.len()));
    assert!((1..=8192).contains(&document.colliders.len()));
    if geometry {
        let path = std::path::PathBuf::from(
            std::env::var("THEIA_UNITY_GEOMETRY_ORACLE").expect("Set THEIA_UNITY_GEOMETRY_ORACLE"),
        );
        assert!(std::fs::metadata(&path).unwrap().len() <= 64 * 1024 * 1024);
        let bytes = std::fs::read(&path).unwrap();
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path.with_extension("receipt.json")).unwrap())
                .unwrap();
        let hash = theia_network::terrain::sha256(&bytes)
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>();
        assert_eq!(receipt["oracle_sha256"], hash);
        let oracle: GeometryOracle = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(oracle.schema, 1);
        assert_eq!(oracle.profile, "independent-source-collision-geometry-v1");
        assert_eq!(oracle.fixture_sha256, digest);
        assert_eq!(oracle.objects.len(), document.colliders.len());
        for (actual, expected) in document.colliders.iter_mut().zip(oracle.objects) {
            assert_eq!(actual.id, expected.id);
            assert_eq!(actual.probes.len(), expected.probes.len());
            for (native, reference) in actual.probes.iter().zip(&expected.probes) {
                assert_eq!(native.origin, reference.origin);
                assert_eq!(native.direction, reference.direction);
                assert_eq!(native.maximum, reference.maximum);
                assert!(reference.normal_candidates.len() <= 64);
                assert!(
                    reference
                        .normal_candidates
                        .iter()
                        .all(|n| Vec3::from_array(*n).is_normalized())
                );
            }
            actual.probes = expected.probes;
        }
    }
    for mesh in &document.meshes {
        assert!((1..=2_000_000).contains(&mesh.vertices.len()));
        assert!((1..=2_000_000).contains(&mesh.indices.len()));
        assert!(
            mesh.vertices
                .iter()
                .flatten()
                .all(|v| v.is_finite() && v.abs() <= 4096.)
        );
        assert!(
            mesh.indices
                .iter()
                .flatten()
                .all(|&v| (v as usize) < mesh.vertices.len())
        );
    }
    let started = Instant::now();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )));
    let mut entities = Vec::new();
    let mut field_entities = Vec::new();
    let mut terrain_samples = None;
    let mut failures = Vec::new();
    if profiles {
        use theia_core::coordinates::{ChunkId, GlobalPosition, INITIAL_CHUNK_METRES, LocalFrame};
        use theia_network::terrain::{Terrain, sha256};
        let directory = std::path::PathBuf::from(
            std::env::var("THEIA_SCHEMA4_REFERENCE").expect("Set THEIA_SCHEMA4_REFERENCE"),
        );
        let metadata_path = directory.join("reference.json");
        assert!(std::fs::metadata(&metadata_path).unwrap().len() <= 2 * 1024 * 1024);
        let reference: ProfileReference =
            serde_json::from_slice(&std::fs::read(metadata_path).unwrap()).unwrap();
        assert_eq!(reference.schema, 1);
        assert_eq!(reference.profile, "schema4-source-shape-reference-v1");
        assert_eq!(reference.objects_sha256, digest);
        assert_eq!(reference.objects.len(), document.colliders.len());
        assert_eq!(reference.terrain.len(), 4);
        assert!((1..=32).contains(&reference.profiles.len()));
        let frame = LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap());
        let mut owners = BTreeMap::new();
        for profile in reference.profiles {
            assert_eq!(
                profile.file,
                format!("profile_{}_{}.json", profile.owner[0], profile.owner[1])
            );
            let path = directory.join(profile.file);
            assert!(std::fs::metadata(&path).unwrap().len() <= 32 * 1024 * 1024);
            let bytes = std::fs::read(path).unwrap();
            let hash = sha256(&bytes)
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>();
            assert_eq!(hash, profile.sha256);
            let terrain = if let Some(root) = std::env::var_os("THEIA_SCHEMA4_PACKAGES") {
                let root = std::path::PathBuf::from(root);
                let package = root.join(format!("chunk_{}_{}", profile.owner[0], profile.owner[1]));
                let path = package.join("terrain.json");
                let actual_bytes = std::fs::read(&path).unwrap();
                let mut expected: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                let actual: serde_json::Value = serde_json::from_slice(&actual_bytes).unwrap();
                expected["visual_sha256"] = actual["visual_sha256"].clone();
                assert_eq!(expected, actual, "Published source shape data differs");
                let terrain = Terrain::load(&path).unwrap();
                terrain
                    .verify_visual_in(&package.join("terrain.glb"), &root)
                    .unwrap();
                terrain
            } else {
                Terrain::decode(&bytes).unwrap()
            };
            assert_eq!(
                terrain.physics.colliders.len(),
                profile.metrics["bodies"].as_u64().unwrap() as usize
            );
            let origin = ChunkId {
                x: profile.owner[0],
                z: profile.owner[1],
            }
            .origin(INITIAL_CHUNK_METRES)
            .unwrap();
            let offset = Vec3::from_array(frame.to_local(origin).unwrap().metres());
            let ids = terrain
                .physics
                .colliders
                .iter()
                .map(|source| {
                    let placed = theia_network::physics::PlacedCollider {
                        shape: source.shape.clone(),
                        center: source.center + offset,
                        layer: source.layer,
                    };
                    app.world_mut().spawn(placed.bundle()).id()
                })
                .collect::<Vec<_>>();
            assert!(owners.insert(profile.owner, ids).is_none());
        }
        let mut addresses = BTreeMap::new();
        for address in reference.objects {
            assert!(
                addresses
                    .insert(address.id, (address.owner, address.collider))
                    .is_none()
            );
        }
        for object in &document.colliders {
            let (owner, index) = addresses.get(&object.id).unwrap();
            entities.push(Some(owners[owner][*index]));
        }
        assert_eq!(
            entities
                .iter()
                .flatten()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            entities.len()
        );
        for address in reference.terrain {
            field_entities.push(owners[&address.owner][address.collider]);
        }
        let path = std::path::PathBuf::from(
            std::env::var("THEIA_FOREST_HEIGHTFIELD_REFERENCE")
                .expect("Set THEIA_FOREST_HEIGHTFIELD_REFERENCE"),
        );
        assert!(std::fs::metadata(&path).unwrap().len() <= 128 * 1024 * 1024);
        let bytes = std::fs::read(path).unwrap();
        let hash = sha256(&bytes)
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>();
        assert_eq!(hash, reference.heightfield_sha256);
        let samples: TerrainSamples = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(samples.schema, 1);
        assert_eq!(samples.profile, "source-full-grid-heightfield-reference-v1");
        assert_eq!(samples.vertices.len(), 263169);
        assert_eq!(samples.triangle_centers.len(), 524288);
        terrain_samples = Some(samples);
    } else {
        for object in &document.colliders {
            match shape(object, &document.meshes) {
                Ok((collider, transform)) => entities.push(Some(
                    app.world_mut()
                        .spawn((
                            RigidBody::Static,
                            collider,
                            transform,
                            CollisionLayers::new(1u32, 0u32),
                        ))
                        .id(),
                )),
                Err(error) => {
                    failures.push(json!({"id":object.id,"stage":"construction","error":error}));
                    entities.push(None);
                }
            }
        }
    }
    app.finish();
    app.cleanup();
    for (_, schedule) in app
        .world_mut()
        .resource_mut::<bevy::ecs::schedule::Schedules>()
        .iter_mut()
    {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    }
    app.update();
    app.update();
    let preparation_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    let mut state = SystemState::<SpatialQuery>::new(app.world_mut());
    let query = state.get(app.world_mut()).unwrap();
    let mut cases = 0;
    let mut mismatches = BTreeMap::<String, usize>::new();
    let mut maximum_distance_error = 0f32;
    for (object, entity) in document.colliders.iter().zip(entities) {
        let Some(entity) = entity else { continue };
        let support_vertices = if geometry && object.convex {
            let pose = matrix(&object.matrix);
            document.meshes[object.mesh.unwrap()]
                .vertices
                .iter()
                .map(|v| pose.transform_point3(Vec3::from_array(*v)))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        for (index, probe) in object.probes.iter().enumerate() {
            let origin = Vec3::from_array(probe.origin);
            let direction = Dir3::new(Vec3::from_array(probe.direction)).unwrap();
            assert!(origin.is_finite() && probe.maximum.is_finite() && probe.maximum > 0.);
            let hit = query.cast_ray_predicate(
                origin,
                direction,
                probe.maximum,
                false,
                &SpatialQueryFilter::from_mask(1u32),
                &|target| target == entity,
            );
            cases += 1;
            let error = hit.as_ref().map(|h| (h.distance - probe.distance).abs());
            let dot = hit.as_ref().map(|h| {
                if geometry && !probe.normal_candidates.is_empty() {
                    probe
                        .normal_candidates
                        .iter()
                        .map(|n| h.normal.dot(Vec3::from_array(*n)))
                        .fold(-1f32, f32::max)
                } else {
                    h.normal.dot(Vec3::from_array(probe.normal))
                }
            });
            let support_gap = hit.as_ref().and_then(|h| {
                (!support_vertices.is_empty()).then(|| {
                    let point = origin + direction.as_vec3() * h.distance;
                    support_vertices
                        .iter()
                        .map(|v| h.normal.dot(*v - point))
                        .fold(f32::NEG_INFINITY, f32::max)
                })
            });
            let normal_valid = if let Some(gap) = support_gap {
                gap.abs() <= 0.002
                    && hit.as_ref().unwrap().normal.dot(direction.as_vec3()) <= 0.0001
            } else {
                dot.is_some_and(|v| v >= 0.995)
            };
            if probe.hit && hit.is_some() {
                maximum_distance_error = maximum_distance_error.max(error.unwrap());
            }
            let point_error = hit.as_ref().map(|h| {
                (origin + direction.as_vec3() * h.distance).distance(Vec3::from_array(probe.point))
            });
            let passed = if probe.hit {
                hit.is_some()
                    && error.unwrap() <= 0.002
                    && point_error.unwrap() <= 0.002
                    && normal_valid
            } else {
                hit.is_none()
            };
            if !passed {
                *mismatches.entry(object.kind.clone()).or_default() += 1;
                if failures.len() < 4096 {
                    failures.push(json!({"id":object.id,"kind":object.kind,"probe":index,"reference_hit":probe.hit,"avian_hit":hit.is_some(),"distance_error":error,"normal_dot":dot,"normal_support_gap":support_gap,"point_error":point_error,"reference_distance":probe.distance,"avian_distance":hit.as_ref().map(|h|h.distance)}));
                }
            }
        }
    }
    let mut terrain_cases = 0usize;
    let mut terrain_misses = 0usize;
    let mut terrain_errors = 0usize;
    let mut terrain_maximum = 0f32;
    if let Some(samples) = terrain_samples {
        for point in samples.vertices.iter().chain(&samples.triangle_centers) {
            assert!(point.iter().all(|v| v.is_finite()));
            terrain_cases += 1;
            let hit = query.cast_ray_predicate(
                Vec3::new(point[0], 32., point[2]),
                Dir3::NEG_Y,
                4096.,
                false,
                &SpatialQueryFilter::from_mask(theia_network::physics::GROUND),
                &|e| field_entities.contains(&e),
            );
            if let Some(hit) = hit {
                let error = (32. - hit.distance - point[1]).abs();
                terrain_maximum = terrain_maximum.max(error);
                terrain_errors += usize::from(error > 0.0001);
            } else {
                terrain_misses += 1;
            }
        }
    }
    let passed =
        mismatches.is_empty() && failures.is_empty() && terrain_misses == 0 && terrain_errors == 0;
    let report = json!({"passed":passed,"schema4_reader":profiles,"comparison":if geometry {"independent source geometry"} else {"strict Unity native"},"platform":std::env::consts::OS,"avian_revision":theia_network::physics::AVIAN_REVISION,"fixture_sha256":digest,"object_colliders":document.colliders.len(),"ray_cases":cases,"terrain_ray_cases":terrain_cases,"terrain_misses":terrain_misses,"terrain_over_tolerance":terrain_errors,"terrain_max_height_error_metres":terrain_maximum,"preparation_ms":preparation_ms,"query_ms":started.elapsed().as_secs_f64()*1000.,"max_distance_error_metres":maximum_distance_error,"mismatches_by_kind":mismatches,"failure_samples":failures,"scope":"Actual static Bevy/Avian BVH with per-source-body predicate; source geometry uses standard two-sided triangles and valid convex support normals; strict Unity native comparison remains separate; no movement, visual publication or map adoption"});
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&report).unwrap().as_bytes())
        .unwrap();
    println!("{report}");
    assert!(
        passed,
        "Unity/Avian object-ray differences; inspect the comparison report"
    );
}
