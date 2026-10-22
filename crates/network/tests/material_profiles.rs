//! Original source-frame/URI contracts; no renderer or external assets.
use serde_json::{Value, json};
use theia_network::material_profile::TriplanarProfile;

#[test]
fn projected_package_identity_matches_the_independent_publisher_byte_contract() {
    use theia_network::content::Manifest;
    let bytes = include_bytes!("fixtures/content-manifest-projected.json");
    let id = Manifest::decode(bytes)
        .unwrap()
        .identity()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        id,
        "3183cb4425e6bc0f9f7b069e1a8b909c1c5f7d845edf5852bc359168f493a61d"
    );
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    value["schema"] = json!(2);
    assert_ne!(
        Manifest::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .identity(),
        Manifest::decode(bytes).unwrap().identity()
    );
}

fn fixture() -> Value {
    let color = format!("../shared/{}-s-10497-10497-9729-9729.png", "0".repeat(64));
    let normal = format!("../shared/{}-l-10497-10497-9729-9729.png", "0".repeat(64));
    json!({"theia_triplanar":1,"theia_triplanar_profile":{"schema":1,"profile":"synty-triplanar-world-v1","id":1,"tiling":0.5,"falloff":4.1,"source_anchor":[128e9,0,-128e9],"textures":{"top":color,"side":color,"top_normal":normal,"side_normal":normal}}})
}
fn decode(value: &Value) -> theia_network::Result<Option<TriplanarProfile>> {
    TriplanarProfile::from_extras(value)
}
#[test]
fn checked_global_source_frame_and_content_keys_separate_resources_from_local_numeric_ids() {
    assert!(decode(&json!({"theia_triplanar":1})).unwrap().is_none());
    let value = fixture();
    let profile = decode(&value).unwrap().unwrap();
    assert_eq!(profile.id(), 1);
    assert_eq!(profile.source_anchor().metres(), [128e9, 0., -128e9]);
    assert_eq!(
        profile.textures().map(|(_, srgb)| srgb),
        [true, true, false, false]
    );
    let mut alias = value.clone();
    alias["theia_triplanar"] = json!(2);
    alias["theia_triplanar_profile"]["id"] = json!(2);
    assert_eq!(
        profile.key().unwrap(),
        decode(&alias).unwrap().unwrap().key().unwrap()
    );
    for field in ["tiling", "falloff", "source_anchor"] {
        let mut changed = value.clone();
        changed["theia_triplanar_profile"][field] = if field == "source_anchor" {
            json!([128e9, 0.25, -128e9])
        } else {
            json!(0.7)
        };
        assert_ne!(
            profile.key().unwrap(),
            decode(&changed).unwrap().unwrap().key().unwrap()
        );
    }
}
#[test]
fn rebound_unknown_fields_frames_roles_samplers_and_unsafe_urls_fail_closed() {
    for fault in [
        "unknown",
        "schema",
        "profile",
        "marker",
        "boolean_marker",
        "zero_id",
        "zero_tiling",
        "underflow",
        "boolean_tiling",
        "falloff",
        "source_bound",
        "source_type",
        "missing_role",
        "wrong_role",
        "sampler",
        "path",
        "url",
    ] {
        let mut value = fixture();
        let data = &mut value["theia_triplanar_profile"];
        match fault {
            "unknown" => data["shader"] = json!("custom.wgsl"),
            "schema" => data["schema"] = json!(2),
            "profile" => data["profile"] = json!("arbitrary"),
            "marker" => value["theia_triplanar"] = json!(2),
            "boolean_marker" => value["theia_triplanar"] = json!(true),
            "zero_id" => data["id"] = json!(0),
            "zero_tiling" => data["tiling"] = json!(0),
            "underflow" => data["tiling"] = json!(1e-100),
            "boolean_tiling" => data["tiling"] = json!(true),
            "falloff" => data["falloff"] = json!(101),
            "source_bound" => data["source_anchor"] = json!([1e13, 0, 0]),
            "source_type" => data["source_anchor"] = json!([true, 0, 0]),
            "missing_role" => {
                data["textures"].as_object_mut().unwrap().remove("side");
            }
            "wrong_role" => data["textures"]["top"] = data["textures"]["top_normal"].clone(),
            "sampler" => {
                data["textures"]["top"] = json!(
                    data["textures"]["top"]
                        .as_str()
                        .unwrap()
                        .replace("9729-9729", "9728-9728")
                )
            }
            "path" => data["textures"]["top"] = json!("../../private.png"),
            "url" => data["textures"]["top"] = json!("https://example.invalid/image.png"),
            _ => unreachable!(),
        }
        assert!(decode(&value).is_err(), "accepted {fault}");
    }
}
