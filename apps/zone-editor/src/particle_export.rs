//! Bounded local dependency preflight for future particle world packaging.
//! This does not publish assets or establish that textures are portable.
use bevy_sprinkles::{
    TextureRef,
    asset::{DrawPassMaterial, versions},
};
use std::{collections::BTreeSet, fs::File, io::Read, path::Path};
use theia_zone::Zone;

const MAX_EFFECT_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn preflight(zone: &Zone, root: &Path) -> Result<usize, String> {
    let mut ids = BTreeSet::new();
    for placement in &zone.particles {
        ids.insert(placement.effect.as_str());
    }
    let mut external_textures = BTreeSet::new();
    for id in ids {
        let path = bevy_sprinkles_editor::io::effect_library_path_in(root, id)
            .ok_or_else(|| format!("{id}: invalid or unassigned Effect ID"))?;
        let file = File::open(path).map_err(|_| format!("{id}: local effect is missing"))?;
        let mut bytes = Vec::new();
        file.take(MAX_EFFECT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| format!("{id}: could not read local effect"))?;
        if bytes.len() as u64 > MAX_EFFECT_BYTES {
            return Err(format!("{id}: local effect exceeds 4 MiB"));
        }
        let asset = versions::migrate(&bytes)
            .map_err(|error| format!("{id}: invalid local effect: {error}"))?
            .asset;
        if let Some(index) = asset.unsupported_material_index() {
            return Err(format!(
                "{id}: emitter {index} uses an unsupported custom shader"
            ));
        }
        for (index, emitter) in asset.emitters.iter().enumerate() {
            let DrawPassMaterial::Standard(material) = &emitter.draw_pass.material else {
                unreachable!("custom material rejected above")
            };
            for (slot, texture) in [
                ("base color", &material.base_color_texture),
                ("emissive", &material.emissive_texture),
                ("metallic/roughness", &material.metallic_roughness_texture),
                ("normal map", &material.normal_map_texture),
                ("occlusion", &material.occlusion_texture),
            ] {
                let Some(texture) = texture else { continue };
                match texture {
                    TextureRef::Preset(_) => {}
                    TextureRef::Asset(_) | TextureRef::Local(_) => {
                        let resolved = texture.resolve_path(&asset.sprinkles_editor.assets_folder);
                        if resolved.is_empty() || !Path::new(&resolved).is_file() {
                            return Err(format!(
                                "{id}: emitter {index} {slot} texture is missing: {resolved}"
                            ));
                        }
                        external_textures.insert(resolved);
                    }
                }
            }
        }
    }
    Ok(external_textures.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::asset::ParticlesAsset;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    use theia_zone::particles::ParticlePlacement;

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "theia-particle-preflight-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(path.join("library")).unwrap();
            Self(path)
        }
        fn save(&self, asset: &ParticlesAsset) {
            fs::write(
                self.0.join("library/vfx.fire.ron"),
                ron::ser::to_string(asset).unwrap(),
            )
            .unwrap();
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn zone() -> Zone {
        let mut zone = Zone {
            schema: 8,
            ..Zone::default()
        };
        zone.particles.push(ParticlePlacement {
            id: 1,
            name: "Fire".into(),
            effect: "vfx.fire".into(),
            position: [0.; 3],
            yaw_degrees: 0.,
            scale: 1.,
        });
        zone
    }
    fn asset() -> ParticlesAsset {
        versions::migrate(include_bytes!("../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/3d-explosion.ron")).unwrap().asset
    }
    #[test]
    fn resolves_all_texture_slots_and_deduplicates_references() {
        let folder = Folder::new();
        let mut asset = asset();
        let images: Vec<_> = (0..5)
            .map(|i| folder.0.join(format!("texture-{i}.png")))
            .collect();
        for image in &images {
            fs::write(image, b"existence only").unwrap();
        }
        let textures: Vec<_> = images
            .iter()
            .map(|p| Some(TextureRef::Local(p.to_str().unwrap().to_owned())))
            .collect();
        let DrawPassMaterial::Standard(material) = &mut asset.emitters[0].draw_pass.material else {
            panic!()
        };
        material.base_color_texture = textures[0].clone();
        material.emissive_texture = textures[1].clone();
        material.metallic_roughness_texture = textures[2].clone();
        material.normal_map_texture = textures[3].clone();
        material.occlusion_texture = textures[4].clone();
        folder.save(&asset);
        assert_eq!(preflight(&zone(), &folder.0).unwrap(), 5);
        for image in &images {
            fs::remove_file(image).unwrap();
            assert!(
                preflight(&zone(), &folder.0)
                    .unwrap_err()
                    .contains("texture is missing")
            );
            fs::write(image, b"existence only").unwrap();
        }
        let DrawPassMaterial::Standard(material) = &mut asset.emitters[0].draw_pass.material else {
            panic!()
        };
        material.emissive_texture = textures[0].clone();
        folder.save(&asset);
        assert_eq!(preflight(&zone(), &folder.0).unwrap(), 4);
    }
    #[test]
    fn missing_malformed_oversize_and_custom_shader_are_reported() {
        let folder = Folder::new();
        let path = folder.0.join("library/vfx.fire.ron");
        assert!(
            preflight(&zone(), &folder.0)
                .unwrap_err()
                .contains("effect is missing")
        );
        fs::write(&path, b"broken").unwrap();
        assert!(
            preflight(&zone(), &folder.0)
                .unwrap_err()
                .contains("invalid local effect")
        );
        fs::write(&path, vec![0; MAX_EFFECT_BYTES as usize + 1]).unwrap();
        assert!(
            preflight(&zone(), &folder.0)
                .unwrap_err()
                .contains("exceeds 4 MiB")
        );
        let mut asset = asset();
        asset.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
            vertex_shader: None,
            fragment_shader: None,
        };
        folder.save(&asset);
        assert!(
            preflight(&zone(), &folder.0)
                .unwrap_err()
                .contains("unsupported custom shader")
        );
    }
}
