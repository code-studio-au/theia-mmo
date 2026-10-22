//! Actual loaded images in the five core glTF PBR texture roles.
use bevy::{
    gltf::{Gltf, GltfMaterial},
    prelude::*,
};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};

pub fn inventory(world: &World, gltf: &Gltf) -> Option<Value> {
    let materials = world.resource::<Assets<GltfMaterial>>();
    let images = world.resource::<Assets<Image>>();
    let mut referenced: HashMap<AssetId<Image>, (Handle<Image>, BTreeSet<&str>)> = HashMap::new();
    let mut slots = 0usize;
    for handle in &gltf.materials {
        let material = materials.get(handle)?;
        for (role, texture) in [
            ("base_color", &material.base_color_texture),
            ("emissive", &material.emissive_texture),
            ("normal", &material.normal_map_texture),
            ("occlusion", &material.occlusion_texture),
            ("metallic_roughness", &material.metallic_roughness_texture),
        ] {
            if let Some(texture) = texture {
                slots += 1;
                referenced
                    .entry(texture.id())
                    .or_insert_with(|| (texture.clone(), BTreeSet::new()))
                    .1
                    .insert(role);
            }
        }
    }
    let mut records = Vec::new();
    let mut cpu_bytes = Some(0usize);
    for (handle, roles) in referenced.values() {
        let image = images.get(handle)?;
        let bytes = image.data.as_ref().map(Vec::len);
        cpu_bytes = cpu_bytes.and_then(|sum| bytes.and_then(|size| sum.checked_add(size)));
        let descriptor = &image.texture_descriptor;
        records.push(json!({
            "handle":format!("{:?}",handle.id()),
            "path":handle.path().map(ToString::to_string),
            "roles":roles,
            "dimensions":[descriptor.size.width,descriptor.size.height,descriptor.size.depth_or_array_layers],
            "format":format!("{:?}",descriptor.format),
            "mip_levels":descriptor.mip_level_count,
            "cpu_data_bytes":bytes,
            "sampler":format!("{:?}",image.sampler),
        }));
    }
    records.sort_by(|a, b| a["handle"].as_str().cmp(&b["handle"].as_str()));
    Some(json!({
        "referenced_slots":slots,
        "unique_handles":records.len(),
        "cpu_image_data_bytes":cpu_bytes,
        "images":records,
        "scope":"loaded CPU Image payloads in core glTF PBR roles, including replaced triplanar placeholders; excludes custom-profile images, process overhead and measured GPU allocation/capacity"
    }))
}
