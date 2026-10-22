//! Capture actual rendered material/image bindings for native qualification.
use bevy::prelude::*;

#[derive(bevy::ecs::system::SystemParam)]
pub struct Bindings<'w, 's> {
    server: Res<'w, AssetServer>,
    materials: Res<'w, Assets<StandardMaterial>>,
    images: Res<'w, Assets<Image>>,
    parts: Query<'w, 's, (Entity, &'static MeshMaterial3d<StandardMaterial>)>,
    parents: Query<'w, 's, &'static ChildOf>,
    roots: Query<'w, 's, &'static crate::chunk_world::Root>,
}
impl Bindings<'_, '_> {
    pub fn report(&self) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        for (entity, material) in &self.parts {
            let mut ancestor = entity;
            let mut owner = None;
            for _ in 0..130 {
                if let Ok(root) = self.roots.get(ancestor) {
                    owner = Some(root.0);
                    break;
                }
                let Ok(parent) = self.parents.get(ancestor) else {
                    break;
                };
                ancestor = parent.parent();
            }
            let Some(owner) = owner else {
                continue;
            };
            let Some(material) = self.materials.get(&material.0) else {
                continue;
            };
            for (slot, handle) in [
                ("base_color", &material.base_color_texture),
                ("emissive", &material.emissive_texture),
                ("metallic_roughness", &material.metallic_roughness_texture),
                ("normal", &material.normal_map_texture),
                ("occlusion", &material.occlusion_texture),
            ] {
                let Some(handle) = handle else {
                    continue;
                };
                let Some(path) = self.server.get_path(handle.id()) else {
                    continue;
                };
                if path.label().is_some() || !path.path().starts_with("shared") {
                    continue;
                }
                let Some(image) = self.images.get(handle) else {
                    continue;
                };
                result.push(serde_json::json!({"root":[owner.x,owner.z],"slot":slot,"path":path.to_string(),"handle":format!("{:?}",handle.id()),"srgb":image.texture_descriptor.format.is_srgb(),"format":format!("{:?}",image.texture_descriptor.format),"sampler":format!("{:?}",image.sampler),"dimensions":[image.texture_descriptor.size.width,image.texture_descriptor.size.height],"unlit":material.unlit,"emissive":[material.emissive.red,material.emissive.green,material.emissive.blue]}));
            }
        }
        result.sort_by_cached_key(serde_json::Value::to_string);
        result
    }
}
