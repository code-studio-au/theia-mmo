//! Bounded local dependency preflight for future particle world packaging.
//! This does not publish assets or establish that textures are portable.
use bevy_sprinkles::{
    TextureRef,
    asset::{DrawPassMaterial, ParticleMesh, ParticlesAsset, versions},
    budget::emitter_counts,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};
use theia_network::{
    particle_content::{Catalog, MAX_EFFECT_BYTES, MAX_TOTAL_EFFECT_BYTES},
    terrain::sha256,
};
use theia_zone::Zone;

#[derive(Debug, Default)]
pub(super) struct Preflight {
    pub textures: usize,
    pub vertices: u64,
    pub indices: u64,
    pub particle_slots: u64,
    pub history_entries: u64,
}

pub(super) fn preflight(zone: &Zone, root: &Path) -> Result<Preflight, String> {
    let mut ids = BTreeMap::new();
    for placement in &zone.particles {
        *ids.entry(placement.effect.as_str()).or_insert(0u64) += 1;
    }
    let mut external_textures = BTreeSet::new();
    let mut effects = Vec::new();
    let mut total_bytes = 0u64;
    let mut can_draft = true;
    let mut result = Preflight::default();
    for (id, instances) in ids {
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
        total_bytes += bytes.len() as u64;
        if total_bytes > MAX_TOTAL_EFFECT_BYTES {
            return Err("Local particle effects exceed the catalog byte budget".into());
        }
        let asset = versions::migrate(&bytes)
            .map_err(|error| format!("{id}: invalid local effect: {error}"))?
            .asset;
        if let Some(index) = asset.unsupported_material_index() {
            return Err(format!(
                "{id}: emitter {index} uses an unsupported custom shader"
            ));
        }
        let mut has_external = false;
        for (index, emitter) in asset.emitters.iter().enumerate() {
            if !finite_mesh_dimensions(&emitter.draw_pass.mesh) {
                return Err(format!(
                    "{id}: emitter {index} has non-finite mesh dimensions"
                ));
            }
            let counts = emitter_counts(emitter).ok_or_else(|| {
                format!("{id}: emitter {index} has invalid allocation arithmetic")
            })?;
            for (total, count) in [
                (&mut result.vertices, u64::from(counts.vertices)),
                (
                    &mut result.indices,
                    u64::try_from(counts.indices)
                        .map_err(|_| "Particle allocation count overflow")?,
                ),
                (&mut result.particle_slots, u64::from(counts.particle_slots)),
                (
                    &mut result.history_entries,
                    u64::from(counts.trail_history_entries),
                ),
            ] {
                *total = total
                    .checked_add(
                        count
                            .checked_mul(instances)
                            .ok_or("Particle allocation count overflow")?,
                    )
                    .ok_or("Particle allocation count overflow")?;
            }
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
                        has_external = true;
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
        if has_external {
            can_draft = false;
        } else {
            let snapshot = preset_snapshot(&asset)?;
            effects.push(
                serde_json::json!({"id":id,"bytes":snapshot.len(),"sha256":sha256(&snapshot)}),
            );
        }
    }
    if can_draft {
        draft_catalog(zone, effects)?;
    }
    result.textures = external_textures.len();
    Ok(result)
}

fn finite_mesh_dimensions(mesh: &ParticleMesh) -> bool {
    match mesh {
        ParticleMesh::Quad {
            size, subdivide, ..
        } => size.is_finite() && subdivide.is_finite(),
        ParticleMesh::Sphere { radius, .. } | ParticleMesh::TubeTrail { radius, .. } => {
            radius.is_finite()
        }
        ParticleMesh::Cuboid { half_size } => half_size.is_finite(),
        ParticleMesh::Cylinder {
            top_radius,
            bottom_radius,
            height,
            ..
        } => top_radius.is_finite() && bottom_radius.is_finite() && height.is_finite(),
        ParticleMesh::Prism {
            left_to_right,
            size,
            subdivide,
        } => left_to_right.is_finite() && size.is_finite() && subdivide.is_finite(),
        ParticleMesh::RibbonTrail { size, .. } => size.is_finite(),
    }
}

/// Only the preset-only subset is suitable for this first snapshot. External
/// texture paths require a separate dependency format and rewrite.
fn preset_snapshot(asset: &ParticlesAsset) -> Result<Vec<u8>, String> {
    for emitter in &asset.emitters {
        if !finite_mesh_dimensions(&emitter.draw_pass.mesh) {
            return Err("Non-finite particle mesh dimensions cannot be snapshotted".into());
        }
        let DrawPassMaterial::Standard(material) = &emitter.draw_pass.material else {
            return Err("Custom particle shaders cannot be snapshotted".into());
        };
        if [
            &material.base_color_texture,
            &material.emissive_texture,
            &material.metallic_roughness_texture,
            &material.normal_map_texture,
            &material.occlusion_texture,
        ]
        .into_iter()
        .flatten()
        .any(|texture| !matches!(texture, TextureRef::Preset(_)))
        {
            return Err("External particle textures require packaging".into());
        }
    }
    let mut snapshot = asset.clone();
    snapshot.sprinkles_editor = Default::default();
    let encoded = ron::ser::to_string(&snapshot)
        .map_err(|error| error.to_string())?
        .into_bytes();
    if encoded.len() as u64 > MAX_EFFECT_BYTES {
        return Err("Serialized particle effect exceeds 4 MiB".into());
    }
    Ok(encoded)
}

fn draft_catalog(zone: &Zone, effects: Vec<serde_json::Value>) -> Result<Vec<u8>, String> {
    let mut placements = zone.particles.iter().collect::<Vec<_>>();
    placements.sort_by_key(|p| p.id);
    let placements = placements
        .into_iter()
        .map(|p| {
            Ok(
                serde_json::json!({"id":p.id,"effect":p.effect,"position":zone.local(p.position)?,
                "yaw_degrees":p.yaw_degrees as f32,"scale":p.scale as f32}),
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    let encoded = serde_json::to_vec(
        &serde_json::json!({"schema":1,"effects":effects,"placements":placements}),
    )
    .map_err(|error| error.to_string())?;
    Catalog::decode(&encoded).map_err(|error| error.to_string())?;
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn rejects_nonfinite_mesh_dimensions_for_each_shape() {
        use bevy::prelude::{Vec2, Vec3};
        use bevy_sprinkles::asset::{QuadOrientation, RibbonTrailShape};
        let invalid = [
            ParticleMesh::Quad {
                orientation: QuadOrientation::FaceZ,
                size: Vec2::new(f32::NAN, 1.),
                subdivide: Vec2::ZERO,
            },
            ParticleMesh::Sphere {
                radius: f32::INFINITY,
                segments: 8,
                rings: 4,
            },
            ParticleMesh::Cuboid {
                half_size: Vec3::splat(f32::NAN),
            },
            ParticleMesh::Cylinder {
                top_radius: 1.,
                bottom_radius: f32::NAN,
                height: 1.,
                radial_segments: 8,
                rings: 2,
                cap_top: true,
                cap_bottom: true,
            },
            ParticleMesh::Prism {
                left_to_right: 0.5,
                size: Vec3::ONE,
                subdivide: Vec3::splat(f32::INFINITY),
            },
            ParticleMesh::TubeTrail {
                radius: f32::NAN,
                radial_steps: 8,
                sections: 4,
                section_rings: 1,
            },
            ParticleMesh::RibbonTrail {
                size: f32::NAN,
                sections: 4,
                section_rings: 1,
                shape: RibbonTrailShape::Flat,
            },
        ];
        for mesh in invalid {
            assert!(!finite_mesh_dimensions(&mesh), "{mesh:?}");
            let mut asset = asset();
            asset.emitters[0].draw_pass.mesh = mesh;
            assert!(preset_snapshot(&asset).is_err());
        }
    }
    #[test]
    fn preset_snapshot_omits_editor_paths_and_rejects_external_references() {
        let mut asset = asset();
        asset
            .sprinkles_editor
            .assets_folder
            .push("/private/authoring/".into());
        let snapshot = preset_snapshot(&asset).unwrap();
        assert!(!String::from_utf8_lossy(&snapshot).contains("/private/authoring/"));
        let reloaded = versions::migrate(&snapshot).unwrap().asset;
        assert!(reloaded.sprinkles_editor.assets_folder.is_empty());
        assert_eq!(reloaded.name, asset.name);
        assert_eq!(reloaded.emitters.len(), asset.emitters.len());
        assert_eq!(preset_snapshot(&asset).unwrap(), snapshot);
        let DrawPassMaterial::Standard(material) = &mut asset.emitters[0].draw_pass.material else {
            panic!()
        };
        material.normal_map_texture = Some(TextureRef::Local("/private/texture.png".into()));
        assert!(preset_snapshot(&asset).unwrap_err().contains("External"));
    }
    #[test]
    fn draft_uses_checked_far_frame_and_preserves_shared_effect_identity() {
        let mut zone = zone();
        zone.origin = [128_000_000_000., 4., -128_000_000_000.];
        zone.particles[0].position = [zone.origin[0] + 0.125, 5., zone.origin[2] - 0.25];
        let mut other = zone.particles[0].clone();
        other.id = 2;
        zone.particles.insert(0, other);
        let effect = serde_json::json!({"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")});
        let bytes = draft_catalog(&zone, vec![effect]).unwrap();
        let catalog = Catalog::decode(&bytes).unwrap();
        assert_eq!(catalog.effects().len(), 1);
        assert_eq!(catalog.placements()[0].id, 1);
        assert_eq!(catalog.placements()[0].position, [0.125, 1., -0.25]);
        assert_eq!(catalog.placements()[1].effect, "vfx.fire");
        zone.particles[0].position[0] += 5000.;
        assert!(draft_catalog(&zone, vec![]).is_err());
    }
    #[test]
    fn repeated_placements_count_each_instance_and_invalid_layout_fails() {
        let folder = Folder::new();
        let mut asset = asset();
        folder.save(&asset);
        let mut zone = zone();
        let one = preflight(&zone, &folder.0).unwrap();
        let mut second = zone.particles[0].clone();
        second.id = 2;
        zone.particles.push(second);
        let two = preflight(&zone, &folder.0).unwrap();
        assert_eq!(two.particle_slots, one.particle_slots * 2);
        assert_eq!(two.vertices, one.vertices * 2);
        assert_eq!(two.indices, one.indices * 2);
        assert_eq!(two.history_entries, one.history_entries * 2);
        asset.emitters[0].emission.particles_amount = u32::MAX;
        folder.save(&asset);
        assert!(
            preflight(&zone, &folder.0)
                .unwrap_err()
                .contains("allocation arithmetic")
        );
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
        assert_eq!(preflight(&zone(), &folder.0).unwrap().textures, 5);
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
        assert_eq!(preflight(&zone(), &folder.0).unwrap().textures, 4);
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
