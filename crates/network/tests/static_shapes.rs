//! Real schema-4 decoding/query/movement and fail-closed data boundaries.
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use serde_json::{Value, json};
use theia_core::time::TickRate;
use theia_network::{
    kinematic::{self, Motion},
    physics::SOLID,
    terrain::Terrain,
    wire::Intent,
};

fn affine() -> Value {
    json!([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1])
}
fn profile() -> Value {
    json!({"schema":4,"bounds":[[-8,-8],[8,8]],"source_origin":[0,0,0],"visual_sha256":vec![0;32],"triangles":[],"spawns":[[-5,0,0],[-4,0,3]],
        "static_shapes":{
            "meshes":[{"vertices":[[0,0,0],[1,0,0],[0,1,0],[0,0,1]],"indices":[[0,2,1],[0,1,3],[0,3,2],[1,2,3]]}],
            "objects":[
                {"kind":"box","center":[0,1,0],"size":[1,2,12],"rotation":[0,0,0,1]},
                {"kind":"box","center":[3,1,3],"size":[1,2,2],"rotation":[0,0.38268343,0,0.9238795]},
                {"kind":"capsule","center":[-3,1.5,4],"radius":0.3,"half_segment":[0.4,0.8,0]},
                {"kind":"mesh","mesh":0,"affine":[1,0,0,-3, 0,1,0,0, 0,0,1,-4, 0,0,0,1],"convex":true},
                {"kind":"box","center":[3,3,-3],"size":[3,0.25,3],"rotation":[0,0,0,1]}],
            "heightfields":[{"rows":5,"columns":5,"heights_column_major":vec![0;25],"center":[0,0,0],"scale":[16,1,16]}]}})
}
fn decode(value: &Value) -> theia_network::Result<Terrain> {
    Terrain::decode(&serde_json::to_vec(value).unwrap())
}

#[test]
fn mixed_standard_shapes_support_authoritative_movement_and_lower_room_camera() {
    let terrain = decode(&profile()).unwrap();
    assert_eq!(terrain.physics.colliders.len(), 6);
    assert_eq!(terrain.physics.ground([6., 6.], 16.).unwrap().0, 0.);
    assert!(
        terrain
            .physics
            .overlaps(&Collider::sphere(0.1), Vec3::new(-3., 1.5, 4.), SOLID)
    );
    assert!(
        terrain
            .physics
            .overlaps(&Collider::sphere(0.05), Vec3::new(-2.8, 0.2, -3.8), SOLID)
    );
    assert!(terrain.camera_fraction([2., 1.2, 3.], [4., 1.2, 3.]) < 0.75);
    assert!((terrain.camera_fraction([3., 1.5, -3.], [3., 1.8, -2.]) - 1.).abs() < 1e-5);
    for hz in [20, 30, 60] {
        let mut feet = terrain.spawn(0).unwrap();
        let mut body = Motion::default();
        for _ in 0..hz * 2 {
            feet = kinematic::advance(
                &terrain.physics,
                &mut body,
                feet,
                Intent {
                    axes: [1000, 0],
                    sprint: false,
                    jump: false,
                },
                TickRate::from_hz(hz).unwrap(),
            )
            .unwrap();
            kinematic::validate_pose(&terrain.physics, feet, body.grounded()).unwrap();
        }
        assert!((-0.9..-0.8).contains(&feet.metres()[0]), "{hz}: {feet:?}");
        assert!(body.grounded());
    }
}

#[test]
fn schema_shape_references_frames_and_physical_parameters_fail_closed() {
    for fault in [
        "old_schema",
        "missing_shapes",
        "unknown_kind",
        "unknown_field",
        "mesh_ref",
        "index",
        "unused_pool",
        "singular",
        "projective",
        "frame",
        "rotation",
        "radius",
        "layers",
        "height_count",
        "height_scale",
        "height_frame",
    ] {
        let mut value = profile();
        match fault {
            "old_schema" => value["schema"] = json!(3),
            "missing_shapes" => {
                value.as_object_mut().unwrap().remove("static_shapes");
            }
            "unknown_kind" => value["static_shapes"]["objects"][0]["kind"] = json!("dynamic"),
            "unknown_field" => value["static_shapes"]["objects"][0]["mass"] = json!(10),
            "mesh_ref" => value["static_shapes"]["objects"][3]["mesh"] = json!(1),
            "index" => value["static_shapes"]["meshes"][0]["indices"][0][0] = json!(999),
            "unused_pool" => {
                value["static_shapes"]["objects"].as_array_mut().unwrap()[3] =
                    value["static_shapes"]["objects"][0].clone()
            }
            "singular" => value["static_shapes"]["objects"][3]["affine"][0] = json!(0),
            "projective" => value["static_shapes"]["objects"][3]["affine"][12] = json!(1),
            "frame" => value["static_shapes"]["objects"][3]["affine"][3] = json!(1024),
            "rotation" => value["static_shapes"]["objects"][0]["rotation"] = json!([0, 0, 0, 0]),
            "radius" => value["static_shapes"]["objects"][2]["radius"] = json!(-1),
            "layers" => value["static_shapes"]["objects"][0]["queries"] = json!(8),
            "height_count" => {
                value["static_shapes"]["heightfields"][0]["heights_column_major"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "height_scale" => value["static_shapes"]["heightfields"][0]["scale"][0] = json!(0),
            "height_frame" => value["static_shapes"]["heightfields"][0]["center"][1] = json!(1024),
            _ => unreachable!(),
        }
        assert!(decode(&value).is_err(), "accepted {fault}");
    }
}

#[test]
fn pooled_geometry_cannot_bypass_instantiated_work_or_sample_budgets() {
    let mut value = profile();
    value["spawns"] = json!([]);
    value["static_shapes"]["heightfields"] = json!([]);
    value["static_shapes"]["meshes"] =
        json!([{"vertices":[[0,0,0],[1,0,0],[0,0,1]],"indices":vec![[0,2,1];65]}]);
    let object = json!({"kind":"mesh","mesh":0,"affine":affine(),"convex":false});
    value["static_shapes"]["objects"] = json!(vec![object.clone(); 4096]);
    assert!(
        decode(&value)
            .err()
            .unwrap()
            .to_string()
            .contains("Instantiated")
    );
    value["static_shapes"]["meshes"][0]["indices"] = json!([[0, 2, 1]]);
    let mut vertices = vec![[0., 0., 0.]; 257];
    vertices[1] = [1., 0., 0.];
    vertices[2] = [0., 0., 1.];
    value["static_shapes"]["meshes"][0]["vertices"] = json!(vertices);
    assert!(
        decode(&value)
            .err()
            .unwrap()
            .to_string()
            .contains("Instantiated")
    );
    value["static_shapes"]["objects"] = json!([]);
    value["static_shapes"]["meshes"] = json!([]);
    value["static_shapes"]["heightfields"] = json!([{"rows":513,"columns":514,"heights_column_major":vec![0;513*514],"center":[0,0,0],"scale":[16,1,16]}]);
    assert!(
        decode(&value)
            .err()
            .unwrap()
            .to_string()
            .contains("sample budget")
    );
    value["static_shapes"]["heightfields"] = json!([]);
    let object = json!({"kind":"box","center":[0,1,0],"size":[1,2,1],"rotation":[0,0,0,1]});
    value["static_shapes"]["objects"] = json!(vec![object; 4097]);
    assert!(
        decode(&value)
            .err()
            .unwrap()
            .to_string()
            .contains("body/pool budget")
    );
}
