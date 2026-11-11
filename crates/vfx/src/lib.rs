//! Helpers for future verified particle packages. This crate does not spawn effects.
use bevy::{
    asset::AssetServer,
    image::{Image, ImageLoaderSettings},
    prelude::Handle,
};
use bevy_sprinkles::{
    TextureRef,
    asset::{DrawPassMaterial, ParticlesAsset},
};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};
use theia_network::{particle_content::Catalog, shared_images::ColorSpace};

/// Queue catalog texture variants using standard Bevy image loading. `package`
/// is a relative package directory under the configured Bevy asset root; an
/// empty path selects the root package. The caller must verify the catalog and
/// texture files first and keep these handles alive until effects are removed.
/// Each variant needs its own asset path, and these must be the first requests
/// for those paths so Bevy applies the intended loader settings.
pub fn preload_textures(
    server: &AssetServer,
    catalog: &Catalog,
    package: &Path,
) -> Result<Vec<Handle<Image>>, String> {
    validate_package_path(package)?;
    let mut handles = Vec::with_capacity(catalog.textures().len());
    for texture in catalog.textures() {
        let path = package.join("textures").join(texture.filename());
        let path = path
            .to_str()
            .ok_or("Particle package path must be UTF-8")?
            .replace('\\', "/");
        let srgb = texture.color_space == ColorSpace::Srgb;
        handles.push(
            server
                .load_builder()
                .with_settings(move |settings: &mut ImageLoaderSettings| {
                    settings.is_srgb = srgb;
                })
                .load(path),
        );
    }
    Ok(handles)
}

fn validate_package_path(package: &Path) -> Result<(), String> {
    if package.components().any(|part| match part {
        Component::Normal(name) => name.to_str().is_none_or(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        }),
        _ => true,
    }) {
        return Err("Particle package path must be relative and contain no traversal".into());
    }
    Ok(())
}

/// Validate catalog texture references and rebase them under the package asset
/// path for Sprinkles. This does not verify file bytes, simulation parameters,
/// resource budgets or effect identities; callers must do those separately.
/// The returned clones are safe to modify without changing source assets.
pub fn rebase_effect_textures(
    catalog: &Catalog,
    effects: &[ParticlesAsset],
    package: &Path,
) -> Result<Vec<ParticlesAsset>, String> {
    validate_package_path(package)?;
    let expected: BTreeSet<_> = catalog
        .textures()
        .iter()
        .map(|texture| {
            (
                format!("textures/{}", texture.filename()),
                texture.color_space,
            )
        })
        .collect();
    let mut used = BTreeSet::new();
    let mut rebased = effects.to_vec();
    for effect in &mut rebased {
        if !effect.sprinkles_editor.is_empty() {
            return Err("Packaged particle effect contains editor paths".into());
        }
        for emitter in &mut effect.emitters {
            let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
                return Err("Packaged particle effect uses a custom shader".into());
            };
            for (texture, role) in [
                (&mut material.base_color_texture, ColorSpace::Srgb),
                (&mut material.emissive_texture, ColorSpace::Srgb),
                (&mut material.metallic_roughness_texture, ColorSpace::Linear),
                (&mut material.normal_map_texture, ColorSpace::Linear),
                (&mut material.occlusion_texture, ColorSpace::Linear),
            ] {
                let Some(texture) = texture else { continue };
                match texture {
                    TextureRef::Preset(_) => {}
                    TextureRef::Local(_) => {
                        return Err("Packaged particle texture cannot use a local path".into());
                    }
                    TextureRef::Asset(path) => {
                        let key = (path.clone(), role);
                        if !expected.contains(&key) {
                            return Err("Particle texture reference is absent from the catalog or has the wrong color space".into());
                        }
                        used.insert(key);
                        *path = package
                            .join(&*path)
                            .to_str()
                            .ok_or("Particle package path must be UTF-8")?
                            .replace('\\', "/");
                    }
                }
            }
        }
    }
    if used != expected {
        return Err("Particle catalog contains an unreferenced texture".into());
    }
    Ok(rebased)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::{AssetApp, AssetPlugin, Assets},
        image::{CompressedImageFormats, ImageLoader, ImagePlugin},
        prelude::*,
    };
    use serde_json::json;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, Instant},
    };
    use theia_network::terrain::sha256;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    const PNG: &[u8] = include_bytes!("../../network/tests/fixtures/shared-image.png");

    #[test]
    fn rebasing_requires_cataloged_role_specific_references_and_no_editor_paths() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"},
                        {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear"}]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        let color = format!("textures/{}", catalog.textures()[0].filename());
        let linear = format!("textures/{}", catalog.textures()[1].filename());
        let mut effect = ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let DrawPassMaterial::Standard(material) = &mut effect.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(color.clone()));
        material.normal_map_texture = Some(TextureRef::Asset(linear.clone()));
        let result =
            rebase_effect_textures(&catalog, &[effect.clone()], Path::new("world/revision"))
                .unwrap();
        let DrawPassMaterial::Standard(material) = &result[0].emitters[0].draw_pass.material else {
            unreachable!()
        };
        assert_eq!(
            material.base_color_texture,
            Some(TextureRef::Asset(format!("world/revision/{color}")))
        );
        assert_eq!(
            material.normal_map_texture,
            Some(TextureRef::Asset(format!("world/revision/{linear}")))
        );
        let DrawPassMaterial::Standard(original) = &effect.emitters[0].draw_pass.material else {
            unreachable!()
        };
        assert_eq!(
            original.base_color_texture,
            Some(TextureRef::Asset(color.clone()))
        );
        assert!(
            rebase_effect_textures(&catalog, &[effect.clone()], Path::new("../escape")).is_err()
        );
        let mut invalid = effect.clone();
        invalid
            .sprinkles_editor
            .assets_folder
            .push("/private/".into());
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect.clone();
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Local("/private/file.png".into()));
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect.clone();
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(linear));
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect;
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.normal_map_texture = None;
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
    }

    #[test]
    fn package_variants_load_with_distinct_color_spaces_and_safe_paths() {
        let root = std::env::temp_dir().join(format!(
            "theia-vfx-load-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("chunk_0_0/textures")).unwrap();
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"},
                        {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear"}]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        for texture in catalog.textures() {
            fs::write(
                root.join("chunk_0_0/textures").join(texture.filename()),
                PNG,
            )
            .unwrap();
        }
        catalog
            .verify_textures(&root.join("chunk_0_0/textures"))
            .unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: root.to_str().unwrap().into(),
                ..default()
            },
            ImagePlugin::default(),
        ));
        app.register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE));
        app.finish();
        app.cleanup();
        let server = app.world().resource::<AssetServer>();
        for invalid in [
            "../escape",
            "/absolute",
            "chunk_0_0/../other",
            "a\\..\\other",
            "source:other",
        ] {
            assert!(
                preload_textures(server, &catalog, Path::new(invalid)).is_err(),
                "{invalid}"
            );
        }
        let handles = preload_textures(server, &catalog, Path::new("chunk_0_0")).unwrap();
        assert_ne!(handles[0].id(), handles[1].id());
        for (texture, handle) in catalog.textures().iter().zip(&handles) {
            let path = format!("chunk_0_0/textures/{}", texture.filename());
            assert_eq!(server.load::<Image>(path).id(), handle.id());
        }
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            app.update();
            let images = app.world().resource::<Assets<Image>>();
            if let (Some(color), Some(data)) = (images.get(&handles[0]), images.get(&handles[1])) {
                assert!(color.texture_descriptor.format.is_srgb());
                assert!(!data.texture_descriptor.format.is_srgb());
                break;
            }
            assert!(Instant::now() < until, "Image loading did not complete");
            std::thread::sleep(Duration::from_millis(5));
        }
        drop(app);
        fs::remove_dir_all(root).unwrap();
    }
}
