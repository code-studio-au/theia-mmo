//! Real dependency files and canonical package identity; GPU sharing is separate.
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_network::{
    content::Manifest,
    shared_images::SharedImage,
    terrain::{Terrain, sha256},
};
const PNG: &[u8] = include_bytes!("fixtures/shared-image.png");
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    directory: PathBuf,
    document: Value,
    manifest: Value,
    files: Vec<PathBuf>,
}
impl Fixture {
    fn new(png: &[u8]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-shared-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let directory = root.join("chunk_0_0");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::create_dir(root.join("shared")).unwrap();
        let descriptors = vec![
            json!({"sha256":sha256(png),"bytes":png.len(),"color_space":"srgb","sampler":[10497,10497,0,0]}),
            json!({"sha256":sha256(png),"bytes":png.len(),"color_space":"linear","sampler":[33071,33071,9728,9728]}),
        ];
        let images = descriptors
            .iter()
            .map(|value| serde_json::from_value::<SharedImage>(value.clone()).unwrap())
            .collect::<Vec<_>>();
        let mut files = Vec::new();
        for image in &images {
            let path = root.join("shared").join(image.filename());
            std::fs::write(&path, png).unwrap();
            files.push(path);
        }
        let document = json!({"asset":{"version":"2.0"},"buffers":[{"byteLength":4}],"images":images.iter().map(|i|json!({"uri":i.uri(),"mimeType":"image/png"})).collect::<Vec<_>>(),"textures":[{"source":0},{"source":1,"sampler":0}],"samplers":[{"wrapS":33071,"wrapT":33071,"minFilter":9728,"magFilter":9728}],"materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}},"normalTexture":{"index":1}}]});
        let manifest = json!({"schema":2,"content_id":"theia.test.shared","revision":2,"collision_schema":3,"shared_images":descriptors});
        let mut fixture = Self {
            root,
            directory,
            document,
            manifest,
            files,
        };
        fixture.write();
        fixture
    }
    fn write(&mut self) {
        let mut text = serde_json::to_vec(&self.document).unwrap();
        while !text.len().is_multiple_of(4) {
            text.push(b' ');
        }
        let mut glb = b"glTF".to_vec();
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(28u32 + text.len() as u32 + 4).to_le_bytes());
        glb.extend_from_slice(&(text.len() as u32).to_le_bytes());
        glb.extend_from_slice(b"JSON");
        glb.extend(text);
        glb.extend_from_slice(&4u32.to_le_bytes());
        glb.extend_from_slice(b"BIN\0");
        glb.extend_from_slice(&[0; 4]);
        let profile=serde_json::to_vec(&json!({"schema":3,"bounds":[[0.,0.],[128.,128.]],"source_origin":[0.,0.,0.],"visual_sha256":sha256(&glb),"triangles":[[[0.,0.,0.],[0.,0.,128.],[128.,0.,0.]]]})).unwrap();
        self.manifest["collision_bytes"] = json!(profile.len());
        self.manifest["collision_sha256"] = json!(sha256(&profile));
        self.manifest["visual_bytes"] = json!(glb.len());
        self.manifest["visual_sha256"] = json!(sha256(&glb));
        std::fs::write(self.directory.join("terrain.glb"), glb).unwrap();
        std::fs::write(self.directory.join("terrain.json"), profile).unwrap();
        std::fs::write(
            self.directory.join("world.manifest.json"),
            serde_json::to_vec(&self.manifest).unwrap(),
        )
        .unwrap();
    }
    fn manifest(&self) -> Manifest {
        Manifest::decode(&serde_json::to_vec(&self.manifest).unwrap()).unwrap()
    }
    fn verify(&self) -> theia_network::Result<()> {
        self.manifest()
            .verify_visual_in(&self.directory.join("terrain.glb"), &self.root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn bound_profiles_require_versioned_manifest_and_real_role_bound_pngs() {
    let mut fixture = Fixture::new(PNG);
    let mut uris = Vec::new();
    for color in ["srgb", "linear"] {
        let descriptor = json!({"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":color,"sampler":[10497,10497,9729,9729]});
        let image: SharedImage = serde_json::from_value(descriptor.clone()).unwrap();
        let path = fixture.root.join("shared").join(image.filename());
        std::fs::write(&path, PNG).unwrap();
        fixture.files.push(path);
        uris.push(image.uri());
        fixture.manifest["shared_images"]
            .as_array_mut()
            .unwrap()
            .push(descriptor);
    }
    fixture.document["materials"][0]["extras"] = json!({"theia_triplanar":1,"theia_triplanar_profile":{"schema":1,"profile":"synty-triplanar-world-v1","id":1,"tiling":0.5,"falloff":4.1,"source_anchor":[0,0,0],"textures":{"top":uris[0],"side":uris[0],"top_normal":uris[1],"side_normal":uris[1]}}});
    fixture.manifest["schema"] = json!(3);
    fixture.write();
    fixture.verify().unwrap();
    let identity = fixture.manifest().identity();
    // The v3 capability requirement is independent of whether these PNGs also
    // happen to appear in an ordinary glTF image array. Older readers reject it.
    fixture.manifest["schema"] = json!(2);
    fixture.write();
    assert_ne!(identity, fixture.manifest().identity());
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("schema 3")
    );
    fixture.manifest["schema"] = json!(3);
    fixture.write();
    let material = fixture.document["materials"][0].clone();
    fixture.document["materials"] = json!(vec![material.clone(); 17]);
    for (n, value) in fixture.document["materials"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        let id = n % 16 + 1;
        value["extras"]["theia_triplanar"] = json!(id);
        value["extras"]["theia_triplanar_profile"]["id"] = json!(id);
    }
    fixture.write();
    fixture.verify().unwrap();
    for (n, value) in fixture.document["materials"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        value["extras"]["theia_triplanar_profile"]["tiling"] = json!((n + 1) as f64 * 0.1);
    }
    fixture.write();
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("profile budget")
    );
    fixture.document["materials"] = json!([material]);
    fixture.manifest["shared_images"]
        .as_array_mut()
        .unwrap()
        .pop();
    fixture.write();
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("Unbound projected")
    );
}

#[test]
fn declaration_bound_static_materials_survive_real_package_dependency_admission() {
    let mut fixture = Fixture::new(PNG);
    fixture.document["extensionsUsed"] =
        json!(["KHR_materials_unlit", "KHR_materials_emissive_strength"]);
    fixture.document["extensionsRequired"] = json!(["KHR_materials_unlit"]);
    fixture.document["materials"][0]["extensions"] = json!({"KHR_materials_unlit":{},"KHR_materials_emissive_strength":{"emissiveStrength":2.6}});
    fixture.write();
    fixture.verify().unwrap();
    let original = fixture.document.clone();
    for fault in [
        "unknown",
        "duplicate",
        "undeclared",
        "required",
        "node",
        "texture",
        "payload",
        "strength",
        "boolean",
        "extra_index",
    ] {
        fixture.document = original.clone();
        match fault {
            "unknown" => fixture.document["extensionsUsed"] = json!(["KHR_materials_transmission"]),
            "duplicate" => {
                fixture.document["extensionsUsed"] =
                    json!(["KHR_materials_unlit", "KHR_materials_unlit"])
            }
            "undeclared" => fixture.document["extensionsUsed"] = json!([]),
            "required" => fixture.document["extensionsRequired"] = json!(["KHR_texture_transform"]),
            "node" => {
                fixture.document["nodes"] = json!([{"extensions":{"KHR_materials_unlit":{}}}])
            }
            "texture" => {
                fixture.document["textures"][0]["extensions"] = json!({"KHR_materials_unlit":{}})
            }
            "payload" => {
                fixture.document["materials"][0]["extensions"]["KHR_materials_unlit"] =
                    json!({"index":0})
            }
            "strength" => {
                fixture.document["materials"][0]["extensions"]["KHR_materials_emissive_strength"]
                    ["emissiveStrength"] = json!(17)
            }
            "boolean" => {
                fixture.document["materials"][0]["extensions"]["KHR_materials_emissive_strength"]
                    ["emissiveStrength"] = json!(true)
            }
            "extra_index" => {
                fixture.document["materials"][0]["extensions"]["KHR_materials_emissive_strength"]
                    ["index"] = json!(0)
            }
            _ => unreachable!(),
        }
        // Rebind the GLB, collision and manifest bytes: rejection must be
        // semantic dependency validation, not a stale file hash.
        fixture.write();
        assert!(fixture.verify().is_err(), "accepted {fault}");
    }
}

#[test]
fn shared_package_golden_identity_is_canonical_and_binds_image_settings() {
    let source = include_bytes!("fixtures/content-manifest-shared.json");
    let manifest = Manifest::decode(source).unwrap();
    let hash = manifest
        .identity()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        hash,
        "9f940fe1bbc96c62789c98e2d947c8b59f6e8169f52c1bfdf4b763efcd106092"
    );
    let mut value: Value = serde_json::from_slice(source).unwrap();
    value["shared_images"].as_array_mut().unwrap().reverse();
    assert_eq!(
        Manifest::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .identity(),
        manifest.identity()
    );
    value["shared_images"][0]["sampler"] = json!([10497, 10497, 0, 0]);
    assert_ne!(
        Manifest::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .identity(),
        manifest.identity()
    );
}
#[test]
fn real_shared_png_hashes_are_verified_without_adding_visual_io_to_headless_collision_load() {
    let fixture = Fixture::new(PNG);
    fixture.verify().unwrap();
    assert!(
        fixture
            .manifest()
            .verify_visual(&fixture.directory.join("terrain.glb"))
            .is_err()
    );
    assert!(
        fixture
            .manifest()
            .verify_visual_in(
                &fixture.directory.join("terrain.glb"),
                &fixture.root.join("shared")
            )
            .is_err()
    );
    std::fs::write(&fixture.files[0], b"corrupt").unwrap();
    assert!(fixture.verify().is_err());
    std::fs::remove_file(&fixture.files[0]).unwrap();
    assert!(fixture.verify().is_err());
    let terrain = Terrain::load(&fixture.directory.join("terrain.json")).unwrap();
    assert_eq!(terrain.identity, fixture.manifest().identity());
}
#[test]
fn arbitrary_uris_variant_conflicts_and_legacy_downgrades_cannot_load_unbound_images() {
    let mut fixture = Fixture::new(PNG);
    let original = fixture.document.clone();
    for uri in [
        "../../private.png",
        "https://example.invalid/image.png",
        "../shared/a.png",
        "../shared/%2e%2e/secret.png",
    ] {
        fixture.document = original.clone();
        fixture.document["images"][0]["uri"] = json!(uri);
        fixture.write();
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("Unbound")
        );
    }
    fixture.document = original.clone();
    fixture.document["textures"][0]["sampler"] = json!(0);
    fixture.write();
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("variant")
    );
    fixture.document = original.clone();
    fixture.document["materials"][0]["normalTexture"]["index"] = json!(0);
    fixture.write();
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("separate")
    );
    fixture.document = original;
    fixture.manifest["schema"] = json!(1);
    fixture
        .manifest
        .as_object_mut()
        .unwrap()
        .remove("shared_images");
    fixture.write();
    assert!(
        fixture
            .verify()
            .unwrap_err()
            .to_string()
            .contains("Unbound")
    );
}
#[test]
fn duplicate_invalid_and_excessive_resource_declarations_fail_before_file_access() {
    let original = Fixture::new(PNG).manifest.clone();
    for (field, value) in [
        ("bytes", json!(0)),
        ("bytes", json!(16 * 1024 * 1024 + 1)),
        ("sampler", json!([1, 2, 3, 4])),
        ("color_space", json!("other")),
    ] {
        let mut document = original.clone();
        document["shared_images"][0][field] = value;
        assert!(Manifest::decode(&serde_json::to_vec(&document).unwrap()).is_err());
    }
    let mut duplicate = original.clone();
    let entry = duplicate["shared_images"][0].clone();
    duplicate["shared_images"]
        .as_array_mut()
        .unwrap()
        .push(entry);
    assert!(Manifest::decode(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut excessive = original.clone();
    excessive["shared_images"] = json!(vec![original["shared_images"][0].clone(); 513]);
    assert!(Manifest::decode(&serde_json::to_vec(&excessive).unwrap()).is_err());
    let mut legacy = original;
    legacy["schema"] = json!(1);
    assert!(Manifest::decode(&serde_json::to_vec(&legacy).unwrap()).is_err());
}
#[test]
fn png_headers_are_bounded_even_when_all_artifact_hashes_match() {
    for (offset, value) in [(16, 16384u32.to_be_bytes().to_vec()), (24, vec![16])] {
        let mut png = PNG.to_vec();
        png[offset..offset + value.len()].copy_from_slice(&value);
        let fixture = Fixture::new(&png);
        assert!(
            fixture
                .verify()
                .unwrap_err()
                .to_string()
                .contains("PNG header")
        );
    }
}
#[cfg(unix)]
#[test]
fn shared_file_alias_is_rejected_even_when_its_target_has_the_right_bytes() {
    let fixture = Fixture::new(PNG);
    let target = fixture.root.join("outside-shared.png");
    std::fs::rename(&fixture.files[0], &target).unwrap();
    std::os::unix::fs::symlink(target, &fixture.files[0]).unwrap();
    assert!(fixture.verify().unwrap_err().to_string().contains("alias"));
}

#[test]
#[ignore = "requires an explicitly staged private world package"]
fn staged_shared_world_verifies_actual_manifest_collision_and_all_image_dependencies() {
    let directory = PathBuf::from(
        std::env::var("THEIA_SHARED_WORLD_FIXTURE")
            .expect("Set THEIA_SHARED_WORLD_FIXTURE to the staged package"),
    );
    let terrain = Terrain::load(&directory.join("terrain.json")).unwrap();
    let manifest = terrain.package.as_ref().unwrap();
    assert!(!manifest.shared_images().is_empty());
    terrain
        .verify_visual_in(&directory.join("terrain.glb"), directory.parent().unwrap())
        .unwrap();
    println!(
        "shared_world content_id={} revision={} images={} identity={:?}",
        manifest.content_id().as_str(),
        manifest.revision(),
        manifest.shared_images().len(),
        manifest.identity()
    );
}
