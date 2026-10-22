//! Opt-in bounded source triplanar inspection through standard Bevy PBR.
use bevy::{
    asset::embedded_asset,
    gltf::GltfMaterialExtras,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(crate) type Material = ExtendedMaterial<StandardMaterial, Triplanar>;
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct Triplanar {
    #[uniform(100)]
    pub(crate) values: Vec4,
    #[uniform(100)]
    pub(crate) phase: Vec4,
    #[texture(101)]
    #[sampler(102)]
    pub(crate) top: Handle<Image>,
    #[texture(103)]
    #[sampler(104)]
    pub(crate) side: Handle<Image>,
    #[texture(105)]
    #[sampler(106)]
    pub(crate) top_normal: Handle<Image>,
    #[texture(107)]
    #[sampler(108)]
    pub(crate) side_normal: Handle<Image>,
}
impl MaterialExtension for Triplanar {
    fn fragment_shader() -> ShaderRef {
        "embedded://theia_client/triplanar.wesl".into()
    }
}

#[derive(Resource)]
pub struct Config {
    profiles: BTreeMap<u16, Profile>,
    legacy: bool,
    diagnostic: u32,
    texture_origin: [f64; 3],
}
struct Profile {
    paths: [String; 4],
    tiling: f32,
    falloff: f32,
    textures: Option<[Handle<Image>; 4]>,
    material: Option<Handle<Material>>,
    applied: usize,
}
fn digest(bytes: &[u8]) -> String {
    theia_network::terrain::sha256(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
impl Config {
    pub fn set_texture_origin(&mut self, origin: [f64; 3]) -> Result<(), &'static str> {
        if origin.iter().any(|v| !v.is_finite() || v.abs() > 1023.) {
            return Err("Inspection texture origin must be finite and within 1023 metres");
        }
        self.texture_origin = origin;
        Ok(())
    }
    pub fn diagnostic(&self) -> bool {
        self.diagnostic != 0
    }
    pub fn load(
        path: &Path,
        geometry: &Path,
        diagnostic: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if geometry.metadata()?.len() > 256 * 1024 * 1024 {
            return Err("Triplanar geometry exceeds inspection budget".into());
        }
        if path.metadata()?.len() > 64 * 1024 {
            return Err("Triplanar metadata exceeds budget".into());
        }
        let value: Value = serde_json::from_slice(&std::fs::read(path)?)?;
        if diagnostic > 2 || value["geometry_sha256"] != digest(&std::fs::read(geometry)?) {
            return Err("Triplanar schema/profile/geometry binding differs".into());
        }
        let parent = geometry
            .parent()
            .ok_or("Missing asset parent")?
            .canonicalize()?;
        let legacy = value["schema"] == 1;
        let entries: Vec<(u16, &Value)> = if legacy {
            vec![(1, &value)]
        } else if value["schema"] == 2 && value["profile"] == "synty-triplanar-catalog-v1" {
            let profiles = value["profiles"]
                .as_array()
                .ok_or("Missing triplanar profiles")?;
            if !(1..=16).contains(&profiles.len()) {
                return Err("Triplanar profile count exceeds budget".into());
            }
            profiles
                .iter()
                .map(|p| {
                    let id = p["id"]
                        .as_u64()
                        .filter(|n| (1..=16).contains(n))
                        .ok_or("Invalid triplanar profile ID")?;
                    Ok((id as u16, p))
                })
                .collect::<Result<_, Box<dyn std::error::Error>>>()?
        } else {
            return Err("Triplanar schema/profile/geometry binding differs".into());
        };
        let mut profiles = BTreeMap::new();
        let mut images = BTreeSet::new();
        let mut decoded_bytes = 0_u64;
        for (id, value) in entries {
            if profiles.contains_key(&id) {
                return Err("Duplicate triplanar profile ID".into());
            }
            let profile = Profile::load(value, &parent, legacy, &mut images, &mut decoded_bytes)?;
            profiles.insert(id, profile);
        }
        Ok(Self {
            profiles,
            legacy,
            diagnostic,
            texture_origin: [0.; 3],
        })
    }

    pub fn validate_scene(
        &self,
        material_count: usize,
        extras: &[String],
    ) -> Result<bool, &'static str> {
        let mut used = BTreeMap::<u16, usize>::new();
        for extra in extras {
            if let Some(id) = material_profile(extra)? {
                *used.entry(id).or_default() += 1;
            }
        }
        if self.legacy && (material_count != 1 || used.keys().copied().collect::<Vec<_>>() != [1]) {
            return Err("Triplanar metadata requires one marked source material");
        }
        if !used.keys().eq(self.profiles.keys()) {
            return Err("Triplanar profile marks differ from metadata");
        }
        Ok(used
            .iter()
            .all(|(id, count)| self.profiles[id].applied == *count))
    }
}

pub fn material_profile(extra: &str) -> Result<Option<u16>, &'static str> {
    let value: Value = serde_json::from_str(extra).map_err(|_| "Invalid material extras JSON")?;
    let Some(id) = value.get("theia_triplanar") else {
        return Ok(None);
    };
    let id = id
        .as_u64()
        .filter(|n| (1..=16).contains(n))
        .ok_or("Invalid triplanar material mark")?;
    Ok(Some(id as u16))
}

impl Profile {
    fn load(
        value: &Value,
        parent: &Path,
        legacy: bool,
        images: &mut BTreeSet<String>,
        decoded_bytes: &mut u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if value["profile"] != "synty-triplanar-world-v1" {
            return Err("Unknown triplanar shader profile".into());
        }
        let scalar = |key: &str, max: f64| -> Result<f32, Box<dyn std::error::Error>> {
            let v = value[key].as_f64().ok_or("Missing triplanar scalar")?;
            if !v.is_finite() || v <= 0. || v > max || !(v as f32).is_finite() || v as f32 <= 0. {
                return Err("Invalid positive bounded f32 triplanar scalar".into());
            }
            Ok(v as f32)
        };
        let mut paths = Vec::new();
        for role in ["top", "side", "top_normal", "side_normal"] {
            let hash = value["textures"][role]["sha256"]
                .as_str()
                .ok_or("Missing triplanar image hash")?;
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err("Invalid triplanar image hash".into());
            }
            let filename = value["textures"][role]["file"]
                .as_str()
                .ok_or("Missing triplanar texture")?;
            let kind = if role.ends_with("normal") {
                "normal"
            } else {
                "color"
            };
            let expected = if legacy {
                format!("triplanar-{role}.png")
            } else {
                format!("triplanar-{kind}-{hash}.png")
            };
            if filename != expected {
                return Err("Invalid triplanar texture path".into());
            }
            let texture = parent.join(filename).canonicalize()?;
            if !texture.starts_with(parent)
                || texture.metadata()?.len() > 64 * 1024 * 1024
                || value["textures"][role]["sha256"] != digest(&std::fs::read(&texture)?)
            {
                return Err("Triplanar image containment/hash differs".into());
            }
            let bytes = std::fs::read(&texture)?;
            if bytes.len() < 24
                || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
                || &bytes[12..16] != b"IHDR"
                || [
                    u32::from_be_bytes(bytes[16..20].try_into()?),
                    u32::from_be_bytes(bytes[20..24].try_into()?),
                ]
                .iter()
                .any(|n| !(1..=4096).contains(n))
            {
                return Err("Triplanar PNG dimensions exceed profile".into());
            }
            if !legacy && (bytes.len() < 29 || bytes[24] != 8) {
                return Err("Catalog requires bounded eight-bit PNG images".into());
            }
            if images.insert(filename.to_owned()) {
                *decoded_bytes += u64::from(u32::from_be_bytes(bytes[16..20].try_into()?))
                    * u64::from(u32::from_be_bytes(bytes[20..24].try_into()?))
                    * 4;
                if images.len() > 16 || *decoded_bytes > 256 * 1024 * 1024 {
                    return Err("Triplanar distinct image/decoded byte budget exceeded".into());
                }
            }
            paths.push(filename.to_owned());
        }
        Ok(Self {
            paths: paths.try_into().unwrap(),
            tiling: scalar("tiling", 10.)?,
            falloff: scalar("falloff", 100.)?,
            textures: None,
            material: None,
            applied: 0,
        })
    }
}
pub struct TriplanarPlugin;
pub(crate) fn texture_phase(origin: [f64; 3], tiling: f32) -> Vec4 {
    let values: [f32; 3] = std::array::from_fn(|i| {
        let mirror = if i == 0 { -1. } else { 1. };
        (origin[i] * mirror * f64::from(tiling)).rem_euclid(1.) as f32
    });
    Vec4::new(values[0], values[1], values[2], 0.)
}
pub(crate) fn frame_phase(render: [f64; 3], source: [f64; 3], tiling: f32) -> Vec4 {
    // Subtract checked f64 origins before reducing to a bounded GPU phase.
    texture_phase(std::array::from_fn(|i| render[i] - source[i]), tiling)
}
impl Plugin for TriplanarPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "triplanar.wesl");
        app.add_plugins(MaterialPlugin::<Material>::default())
            .add_systems(Update, apply);
    }
}
pub fn report(world: &World) -> Value {
    let Some(config) = world.get_resource::<Config>() else {
        return Value::Null;
    };
    let images = world.resource::<Assets<Image>>();
    let mut unique = BTreeSet::new();
    let profiles: Vec<Value> = config.profiles.iter().map(|(id, profile)| {
        let textures: Vec<_> = profile.textures.iter().flatten().filter_map(|h| {
            let image = images.get(h)?;
            unique.insert(format!("{:?}", h.id()));
            Some(json!({"handle":format!("{:?}",h.id()),"format":format!("{:?}",image.texture_descriptor.format),"dimensions":[image.width(),image.height()],"mip_levels":image.texture_descriptor.mip_level_count,"sampler":format!("{:?}",image.sampler)}))
        }).collect();
        json!({"id":id,"applied_primitives":profile.applied,"tiling":profile.tiling,"falloff":profile.falloff,"textures":textures,"texture_phase":texture_phase(config.texture_origin,profile.tiling).to_array()})
    }).collect();
    let mut result = json!({"applied_primitives":config.profiles.values().map(|p|p.applied).sum::<usize>(),"diagnostic_unlit":config.diagnostic,"profiles":profiles,"unique_image_handles":unique.len(),"material_profiles":config.profiles.values().filter(|p|p.material.is_some()).count(),"texture_origin":config.texture_origin,"scope":"bounded inspection with explicit source texture phase; no normal-world rebase/streaming integration"});
    // Preserve existing single-profile capture consumers and regression receipts.
    if config.profiles.len() == 1 {
        for key in ["tiling", "falloff", "textures"] {
            result[key] = result["profiles"][0][key].clone();
        }
    }
    result
}
fn apply(world: &mut World) {
    let Some(mut config) = world.remove_resource::<Config>() else {
        return;
    };
    let server = world.resource::<AssetServer>();
    for profile in config.profiles.values_mut() {
        if profile.textures.is_none() {
            profile.textures = Some(std::array::from_fn(|i| {
                server
                    .load_builder()
                    .with_settings(move |settings: &mut ImageLoaderSettings| {
                        settings.is_srgb = i < 2;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            mipmap_filter: ImageFilterMode::Nearest,
                            ..default()
                        });
                    })
                    .load(profile.paths[i].clone())
            }));
        }
    }
    let handles: Vec<_> = config
        .profiles
        .values()
        .flat_map(|p| p.textures.as_ref().unwrap().iter())
        .collect();
    if handles.iter().any(|h| {
        matches!(
            server.get_load_state(h.id()),
            Some(bevy::asset::LoadState::Failed(_))
        )
    }) {
        error!("Triplanar image dependency failed");
        world.write_message(AppExit::error());
        world.insert_resource(config);
        return;
    }
    if !handles
        .iter()
        .all(|h| server.is_loaded_with_dependencies(*h))
    {
        world.insert_resource(config);
        return;
    }
    for profile in config.profiles.values() {
        for (i, handle) in profile.textures.as_ref().unwrap().iter().enumerate() {
            let expected = if i < 2 {
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb
            } else {
                bevy::render::render_resource::TextureFormat::Rgba8Unorm
            };
            if !world
                .resource::<Assets<Image>>()
                .get(handle)
                .is_some_and(|image| {
                    image.texture_descriptor.format == expected
                        && image.texture_descriptor.mip_level_count == 1
                })
            {
                error!("Triplanar decoded image format/mips differ from profile");
                world.write_message(AppExit::error());
                world.insert_resource(config);
                return;
            }
        }
    }
    let candidates = world
        .query::<(
            Entity,
            &GltfMaterialExtras,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
        )>()
        .iter(world)
        .filter_map(|(e, extra, mesh, _)| match material_profile(&extra.value) {
            Ok(Some(id)) => Some(Ok((e, id, mesh.0.clone()))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>();
    let candidates = match candidates {
        Ok(candidates)
            if candidates
                .iter()
                .all(|(_, id, _)| config.profiles.contains_key(id)) =>
        {
            candidates
        }
        _ => {
            error!("Triplanar material mark has no valid metadata profile");
            world.write_message(AppExit::error());
            world.insert_resource(config);
            return;
        }
    };
    for (_, _, handle) in &candidates {
        let result = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            meshes.get_mut(handle).map(|mut mesh| {
                if mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_none() {
                    mesh.generate_tangents()
                } else {
                    Ok(())
                }
            })
        };
        match result {
            None => {
                world.insert_resource(config);
                return;
            }
            Some(Err(error)) => {
                error!("Triplanar MikkTSpace generation failed: {error:?}");
                world.write_message(AppExit::error());
                world.insert_resource(config);
                return;
            }
            Some(Ok(())) => {}
        }
    }
    let changed = !candidates.is_empty();
    for (entity, id, _) in candidates {
        let profile = config.profiles.get_mut(&id).unwrap();
        let material = profile
            .material
            .get_or_insert_with(|| {
                let textures = profile.textures.as_ref().unwrap();
                world.resource_mut::<Assets<Material>>().add(Material {
                    base: StandardMaterial {
                        perceptual_roughness: 1.,
                        metallic: 0.,
                        unlit: config.diagnostic != 0,
                        ..default()
                    },
                    extension: Triplanar {
                        values: Vec4::new(
                            profile.tiling,
                            profile.falloff,
                            config.diagnostic as f32,
                            0.,
                        ),
                        phase: texture_phase(config.texture_origin, profile.tiling),
                        top: textures[0].clone(),
                        side: textures[1].clone(),
                        top_normal: textures[2].clone(),
                        side_normal: textures[3].clone(),
                    },
                })
            })
            .clone();
        world
            .entity_mut(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(material));
        profile.applied += 1;
    }
    if changed {
        world
            .resource_mut::<crate::capture_ready::CaptureReady>()
            .reset();
    }
    world.insert_resource(config);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mirrored_signed_phase_is_periodic_and_keeps_translations_bounded() {
        assert_eq!(texture_phase([0.; 3], 0.5), Vec4::ZERO);
        assert_eq!(
            texture_phase([13.25, -7.5, 9.75], 0.5),
            Vec4::new(0.375, 0.25, 0.875, 0.)
        );
        assert_eq!(
            texture_phase([-13.25, 7.5, -9.75], 0.5),
            Vec4::new(0.625, 0.75, 0.125, 0.)
        );
        assert_eq!(texture_phase([1022., -1022., 512.], 0.5), Vec4::ZERO);
        let (root, value) = fixture();
        std::fs::write(
            root.join("triplanar.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let mut config =
            Config::load(&root.join("triplanar.json"), &root.join("preview.glb"), 0).unwrap();
        config.set_texture_origin([13.25, -7.5, 9.75]).unwrap();
        for invalid in [f64::NAN, f64::INFINITY, -1024., 1024.] {
            assert!(config.set_texture_origin([invalid, 0., 0.]).is_err());
            assert_eq!(config.texture_origin, [13.25, -7.5, 9.75]);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    fn fixture() -> (std::path::PathBuf, Value) {
        let root = std::env::temp_dir().join(format!(
            "theia-triplanar-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("preview.glb"), b"metadata-binding-fixture").unwrap();
        let mut value = json!({"schema":1,"profile":"synty-triplanar-world-v1","geometry_sha256":digest(b"metadata-binding-fixture"),"tiling":0.2,"falloff":4.1,"textures":{}});
        // Minimal PNG header for pre-decode containment/dimension checks.
        let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        bytes.extend(2u32.to_be_bytes());
        bytes.extend(2u32.to_be_bytes());
        for role in ["top", "side", "top_normal", "side_normal"] {
            let name = format!("triplanar-{role}.png");
            std::fs::write(root.join(&name), &bytes).unwrap();
            value["textures"][role] = json!({"file":name,"sha256":digest(&bytes)});
        }
        (root, value)
    }
    fn load(root: &Path, value: &Value) -> bool {
        std::fs::write(
            root.join("triplanar.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
        Config::load(&root.join("triplanar.json"), &root.join("preview.glb"), 0).is_ok()
    }
    #[test]
    fn metadata_requires_bound_geometry_profile_and_positive_bounded_scalars() {
        let (root, value) = fixture();
        assert!(load(&root, &value));
        for (key, bad) in [
            ("schema", json!(2)),
            ("profile", json!("unknown")),
            ("geometry_sha256", json!("0")),
            ("tiling", json!(0)),
            ("falloff", json!(101)),
            ("tiling", Value::Null),
            ("tiling", json!(1e-100)),
        ] {
            let mut changed = value.clone();
            changed[key] = bad;
            assert!(!load(&root, &changed), "{key}");
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn texture_path_hash_and_png_dimensions_remain_checked_after_metadata_rebinding() {
        let (root, value) = fixture();
        let mut changed = value.clone();
        changed["textures"]["top"]["file"] = json!("../triplanar-top.png");
        assert!(!load(&root, &changed));
        let mut changed = value.clone();
        changed["textures"]["top"]["sha256"] = json!("0");
        assert!(!load(&root, &changed));
        let path = root.join("triplanar-top.png");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[16..20].copy_from_slice(&4097u32.to_be_bytes());
        std::fs::write(&path, &bytes).unwrap();
        let mut changed = value.clone();
        changed["textures"]["top"]["sha256"] = json!(digest(&bytes));
        assert!(!load(&root, &changed));
        std::fs::remove_dir_all(root).unwrap();
    }

    fn catalog_fixture() -> (std::path::PathBuf, Value) {
        let (root, legacy) = fixture();
        let mut profile = legacy.clone();
        profile["id"] = json!(1);
        for role in ["top", "side", "top_normal", "side_normal"] {
            let mut bytes = std::fs::read(root.join(format!("triplanar-{role}.png"))).unwrap();
            // Metadata-only header fixture, not a qualified decoded PNG.
            bytes.extend([8, 6, 0, 0, 0]);
            let hash = digest(&bytes);
            let kind = if role.ends_with("normal") {
                "normal"
            } else {
                "color"
            };
            let filename = format!("triplanar-{kind}-{hash}.png");
            std::fs::write(root.join(&filename), bytes).unwrap();
            profile["textures"][role] = json!({"file":filename,"sha256":hash});
        }
        let mut second = profile.clone();
        second["id"] = json!(2);
        second["tiling"] = json!(0.5);
        let value = json!({"schema":2,"profile":"synty-triplanar-catalog-v1","geometry_sha256":legacy["geometry_sha256"],"profiles":[profile,second]});
        (root, value)
    }

    #[test]
    fn catalog_requires_exact_profile_mapping_and_waits_for_every_application() {
        let (root, value) = catalog_fixture();
        assert!(load(&root, &value));
        let mut config =
            Config::load(&root.join("triplanar.json"), &root.join("preview.glb"), 0).unwrap();
        assert_eq!(config.profiles[&1].paths, config.profiles[&2].paths);
        assert_eq!(config.profiles[&1].paths[0], config.profiles[&1].paths[1]);
        let extras = vec![
            r#"{"theia_triplanar":1}"#.to_owned(),
            r#"{"theia_triplanar":1}"#.to_owned(),
            r#"{"theia_triplanar":2}"#.to_owned(),
            r#"{"source":"ordinary Standard"}"#.to_owned(),
        ];
        assert_eq!(config.validate_scene(3, &extras), Ok(false));
        config.profiles.get_mut(&1).unwrap().applied = 2;
        assert_eq!(config.validate_scene(3, &extras), Ok(false));
        config.profiles.get_mut(&2).unwrap().applied = 1;
        assert_eq!(config.validate_scene(3, &extras), Ok(true));
        assert!(config.validate_scene(3, &extras[..2]).is_err());
        let mut unknown = extras.clone();
        unknown[2] = r#"{"theia_triplanar":3}"#.to_owned();
        assert!(config.validate_scene(3, &unknown).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_rejects_duplicate_out_of_range_ids_and_unknown_profiles() {
        let (root, value) = catalog_fixture();
        for (field, bad) in [
            ("id", json!(0)),
            ("id", json!(17)),
            ("id", json!(1)),
            ("profile", json!("unknown")),
        ] {
            let mut changed = value.clone();
            changed["profiles"][1][field] = bad;
            assert!(!load(&root, &changed));
        }
        let mut changed = value.clone();
        changed["profiles"] = json!([]);
        assert!(!load(&root, &changed));
        let mut changed = value.clone();
        changed["profiles"] = json!(vec![value["profiles"][0].clone(); 17]);
        assert!(!load(&root, &changed));
        for bad in [
            json!(0),
            json!(-1),
            json!(17),
            json!(1.5),
            json!(true),
            Value::Null,
            json!("1"),
        ] {
            assert!(material_profile(&json!({"theia_triplanar":bad}).to_string()).is_err());
        }
        assert_eq!(material_profile(r#"{"other":1}"#), Ok(None));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_image_budget_counts_shared_paths_once_and_rejects_rebound_oversize() {
        let (root, mut value) = catalog_fixture();
        for (i, role) in ["top", "side", "top_normal", "side_normal"]
            .into_iter()
            .enumerate()
        {
            let mut bytes = std::fs::read(
                root.join(
                    value["profiles"][0]["textures"][role]["file"]
                        .as_str()
                        .unwrap(),
                ),
            )
            .unwrap();
            bytes[16..20].copy_from_slice(&4096_u32.to_be_bytes());
            bytes[20..24].copy_from_slice(&4096_u32.to_be_bytes());
            bytes.push(i as u8);
            let hash = digest(&bytes);
            let kind = if role.ends_with("normal") {
                "normal"
            } else {
                "color"
            };
            let filename = format!("triplanar-{kind}-{hash}.png");
            std::fs::write(root.join(&filename), bytes).unwrap();
            for profile in value["profiles"].as_array_mut().unwrap() {
                profile["textures"][role] = json!({"file":filename,"sha256":hash});
            }
        }
        assert!(load(&root, &value)); // Four 64-MiB decoded images, referenced twice.
        let mut bytes = std::fs::read(
            root.join(
                value["profiles"][0]["textures"]["top"]["file"]
                    .as_str()
                    .unwrap(),
            ),
        )
        .unwrap();
        bytes.push(99);
        let hash = digest(&bytes);
        let filename = format!("triplanar-color-{hash}.png");
        std::fs::write(root.join(&filename), bytes).unwrap();
        value["profiles"][1]["textures"]["top"] = json!({"file":filename,"sha256":hash});
        assert!(!load(&root, &value)); // Fifth distinct decoded image exceeds 256 MiB.
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_distinct_image_limit_is_independent_of_small_decoded_size() {
        let (root, mut value) = catalog_fixture();
        let seed = value["profiles"][0].clone();
        let mut profiles = Vec::new();
        for id in 1..=5 {
            let mut profile = seed.clone();
            profile["id"] = json!(id);
            for (i, role) in ["top", "side", "top_normal", "side_normal"]
                .into_iter()
                .enumerate()
            {
                let mut bytes =
                    std::fs::read(root.join(seed["textures"][role]["file"].as_str().unwrap()))
                        .unwrap();
                bytes.extend([id as u8, i as u8]);
                let hash = digest(&bytes);
                let kind = if role.ends_with("normal") {
                    "normal"
                } else {
                    "color"
                };
                let filename = format!("triplanar-{kind}-{hash}.png");
                std::fs::write(root.join(&filename), bytes).unwrap();
                profile["textures"][role] = json!({"file":filename,"sha256":hash});
            }
            profiles.push(profile);
        }
        value["profiles"] = json!(&profiles[..4]);
        assert!(load(&root, &value));
        value["profiles"] = json!(profiles);
        assert!(!load(&root, &value));
        std::fs::remove_dir_all(root).unwrap();
    }
}
