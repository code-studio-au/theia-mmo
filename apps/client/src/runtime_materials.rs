//! Bound static-world projection materials. Standard Bevy assets and materials;
//! readiness is scoped to imported roots so a failed candidate retains its world.
use crate::triplanar::{Material, Triplanar, frame_phase};
use bevy::{
    asset::LoadState,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    gltf::GltfMaterialExtras,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    render::render_resource::TextureFormat,
    transform::TransformSystems,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use theia_network::material_profile::TriplanarProfile;

type Key = [u8; 32];
type MaterialFlags<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static Failed>,
        Option<&'static Pending>,
        Option<&'static Checked>,
    ),
    (With<GltfMaterialExtras>, With<Mesh3d>),
>;
const MAX_PROFILES: usize = 16;
#[derive(Component)]
struct Checked;
#[derive(Component)]
struct Pending {
    key: Key,
    source_id: u16,
}
#[derive(Component)]
struct Applied {
    key: Key,
    source_id: u16,
}
#[derive(Component)]
struct Failed(String);
struct Entry {
    profile: TriplanarProfile,
    images: [Handle<Image>; 4],
    material: Option<Handle<Material>>,
    error: Option<String>,
}
#[derive(Resource, Default)]
pub struct Catalog {
    entries: BTreeMap<Key, Entry>,
    phase_updates: u64,
    render_origin: Option<[f64; 3]>,
}
#[derive(Resource, Default)]
pub struct Diagnostic(pub u32);
#[derive(Component)]
struct DiagnosticCamera;
pub struct RuntimeMaterialsPlugin;
impl Plugin for RuntimeMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Catalog>()
            .init_resource::<Diagnostic>()
            .add_systems(PostUpdate, update.after(TransformSystems::Propagate));
    }
}
fn static_root(world: &World, mut entity: Entity) -> bool {
    for _ in 0..130 {
        if world
            .get::<crate::terrain_scene::SceneryRoot>(entity)
            .is_some()
        {
            return true;
        }
        let Some(parent) = world.get::<ChildOf>(entity) else {
            return false;
        };
        entity = parent.parent();
    }
    false
}
fn fail(world: &mut World, entity: Entity, error: String) {
    world
        .entity_mut(entity)
        .remove::<Pending>()
        .insert((Checked, Failed(error)));
}
fn update(world: &mut World) {
    let diagnostic = world.resource::<Diagnostic>().0;
    if diagnostic != 0 {
        let cameras = world
            .query_filtered::<Entity, (With<Camera3d>, Without<DiagnosticCamera>)>()
            .iter(world)
            .collect::<Vec<_>>();
        for entity in cameras {
            world
                .entity_mut(entity)
                .insert((Tonemapping::None, DebandDither::Disabled, DiagnosticCamera))
                .remove::<DistanceFog>();
        }
    }
    let mut catalog = world
        .remove_resource::<Catalog>()
        .expect("Installed material catalog");
    let alive = world
        .query_filtered::<(Option<&Pending>, Option<&Applied>), Or<(With<Pending>, With<Applied>)>>(
        )
        .iter(world)
        .map(|(pending, applied)| {
            pending
                .map(|p| p.key)
                .or_else(|| applied.map(|a| a.key))
                .unwrap()
        })
        .collect::<BTreeSet<_>>();
    catalog.entries.retain(|key, _| alive.contains(key));
    let incoming = world
        .query_filtered::<(Entity, &GltfMaterialExtras), (With<Mesh3d>, Without<Checked>)>()
        .iter(world)
        .filter(|(entity, _)| static_root(world, *entity))
        .map(|(entity, extra)| (entity, extra.value.clone()))
        .collect::<Vec<_>>();
    for (entity, extra) in incoming {
        match crate::asset_preview::render_flags::parse(&extra) {
            Ok(Some((cast, receive))) => {
                let mut part = world.entity_mut(entity);
                if cast {
                    part.remove::<bevy::light::NotShadowCaster>();
                } else {
                    part.insert(bevy::light::NotShadowCaster);
                }
                if receive {
                    part.remove::<bevy::light::NotShadowReceiver>();
                } else {
                    part.insert(bevy::light::NotShadowReceiver);
                }
            }
            Ok(None) => {}
            Err(error) => {
                fail(world, entity, error.to_owned());
                continue;
            }
        }
        let parsed = (|| -> Result<Option<TriplanarProfile>, String> {
            let value: Value = serde_json::from_str(&extra).map_err(|e| e.to_string())?;
            let profile = TriplanarProfile::from_extras(&value).map_err(|e| e.to_string())?;
            if profile.is_none() && value.get("theia_triplanar").is_some() {
                return Err(
                    "World projection mark requires a bound package material profile".into(),
                );
            }
            Ok(profile)
        })();
        world.entity_mut(entity).insert(Checked);
        let profile = match parsed {
            Ok(Some(profile)) => profile,
            Ok(None) => continue,
            Err(e) => {
                fail(world, entity, e);
                continue;
            }
        };
        let key = match profile.key() {
            Ok(key) => key,
            Err(e) => {
                fail(world, entity, e.to_string());
                continue;
            }
        };
        if !catalog.entries.contains_key(&key) {
            if catalog.entries.len() >= MAX_PROFILES {
                fail(
                    world,
                    entity,
                    "Active/staging material profile budget exceeded".into(),
                );
                continue;
            }
            let server = world.resource::<AssetServer>();
            let images = profile.textures().map(|(uri, srgb)| {
                server
                    .load_builder()
                    .with_settings(move |settings: &mut ImageLoaderSettings| {
                        settings.is_srgb = srgb;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            mipmap_filter: ImageFilterMode::Nearest,
                            ..default()
                        });
                    })
                    .load(
                        uri.strip_prefix("../")
                            .expect("Validated canonical URI")
                            .to_owned(),
                    )
            });
            catalog.entries.insert(
                key,
                Entry {
                    profile: profile.clone(),
                    images,
                    material: None,
                    error: None,
                },
            );
        }
        world.entity_mut(entity).insert(Pending {
            key,
            source_id: profile.id(),
        });
    }
    let render = world
        .resource::<crate::floating_origin::Frame>()
        .render
        .origin()
        .metres();
    let pending = world
        .query::<(Entity, &Pending, &Mesh3d)>()
        .iter(world)
        .map(|(entity, pending, mesh)| (entity, pending.key, pending.source_id, mesh.0.clone()))
        .collect::<Vec<_>>();
    let mut changed = false;
    for (entity, key, source_id, mesh) in pending {
        let entry = catalog
            .entries
            .get_mut(&key)
            .expect("Registered pending profile");
        if let Some(error) = &entry.error {
            fail(world, entity, error.clone());
            continue;
        }
        let server = world.resource::<AssetServer>();
        if entry
            .images
            .iter()
            .any(|h| matches!(server.get_load_state(h.id()), Some(LoadState::Failed(_))))
        {
            let error = "Bound material image dependency failed".to_owned();
            entry.error = Some(error.clone());
            fail(world, entity, error);
            continue;
        }
        if !entry
            .images
            .iter()
            .all(|h| server.is_loaded_with_dependencies(h.id()))
        {
            continue;
        }
        if entry.images.iter().enumerate().any(|(index, h)| {
            world
                .resource::<Assets<Image>>()
                .get(h)
                .is_none_or(|image| {
                    image.texture_descriptor.mip_level_count != 1
                        || image.width() > 4096
                        || image.height() > 4096
                        || image.texture_descriptor.format
                            != if index < 2 {
                                TextureFormat::Rgba8UnormSrgb
                            } else {
                                TextureFormat::Rgba8Unorm
                            }
                })
        }) {
            let error = "Bound material decoded image format/dimensions/mips differ".to_owned();
            entry.error = Some(error.clone());
            fail(world, entity, error);
            continue;
        }
        let has_tangents = world
            .resource::<Assets<Mesh>>()
            .get(&mesh)
            .map(|m| m.attribute(Mesh::ATTRIBUTE_TANGENT).is_some());
        let tangents = if has_tangents == Some(true) {
            Some(Ok(()))
        } else {
            world
                .resource_mut::<Assets<Mesh>>()
                .get_mut(&mesh)
                .map(|mut mesh| mesh.generate_tangents())
        };
        match tangents {
            None => continue,
            Some(Err(error)) => {
                fail(
                    world,
                    entity,
                    format!("Bound material MikkTSpace generation failed: {error:?}"),
                );
                continue;
            }
            Some(Ok(())) => {}
        }
        let material = entry
            .material
            .get_or_insert_with(|| {
                world.resource_mut::<Assets<Material>>().add(Material {
                    base: StandardMaterial {
                        perceptual_roughness: 1.,
                        metallic: 0.,
                        unlit: diagnostic != 0,
                        ..default()
                    },
                    extension: Triplanar {
                        values: Vec4::new(
                            entry.profile.tiling(),
                            entry.profile.falloff(),
                            diagnostic as f32,
                            0.,
                        ),
                        phase: frame_phase(
                            render,
                            entry.profile.source_anchor().metres(),
                            entry.profile.tiling(),
                        ),
                        top: entry.images[0].clone(),
                        side: entry.images[1].clone(),
                        top_normal: entry.images[2].clone(),
                        side_normal: entry.images[3].clone(),
                    },
                })
            })
            .clone();
        world
            .entity_mut(entity)
            .remove::<Pending>()
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((MeshMaterial3d(material), Applied { key, source_id }));
        changed = true;
    }
    if catalog.render_origin != Some(render) {
        for entry in catalog.entries.values() {
            let Some(handle) = &entry.material else {
                continue;
            };
            let phase = frame_phase(
                render,
                entry.profile.source_anchor().metres(),
                entry.profile.tiling(),
            );
            let mut materials = world.resource_mut::<Assets<Material>>();
            if materials
                .get(handle)
                .is_some_and(|m| m.extension.phase != phase)
            {
                let mut material = materials
                    .get_mut(handle)
                    .expect("Existing projected material");
                material.extension.phase = phase;
                catalog.phase_updates += 1;
                changed = true;
            }
        }
        catalog.render_origin = Some(render);
    }
    if changed {
        world
            .resource_mut::<crate::capture_ready::CaptureReady>()
            .reset();
    }
    world.insert_resource(catalog);
}

pub fn root_ready(world: &World, root: Entity) -> Result<bool, String> {
    let mut pending = vec![root];
    let mut ready = true;
    while let Some(entity) = pending.pop() {
        if let Some(error) = world.get::<Failed>(entity) {
            return Err(error.0.clone());
        }
        if world.get::<Pending>(entity).is_some()
            || (world.get::<GltfMaterialExtras>(entity).is_some()
                && world.get::<Mesh3d>(entity).is_some()
                && world.get::<Checked>(entity).is_none())
        {
            ready = false;
        }
        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter());
        }
    }
    Ok(ready)
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct Readiness<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    flags: MaterialFlags<'w, 's>,
}
impl Readiness<'_, '_> {
    pub fn ready(&self, root: Entity) -> Result<bool, String> {
        let mut pending = vec![root];
        let mut ready = true;
        while let Some(entity) = pending.pop() {
            if let Ok((failed, pending, checked)) = self.flags.get(entity) {
                if let Some(error) = failed {
                    return Err(error.0.clone());
                }
                if pending.is_some() || checked.is_none() {
                    ready = false;
                }
            }
            if let Ok(children) = self.children.get(entity) {
                pending.extend(children.iter());
            }
        }
        Ok(ready)
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub struct Evidence<'w, 's> {
    server: Res<'w, AssetServer>,
    catalog: Option<Res<'w, Catalog>>,
    parts: Query<'w, 's, (Entity, &'static Applied, &'static MeshMaterial3d<Material>)>,
    parents: Query<'w, 's, &'static ChildOf>,
    roots: Query<'w, 's, &'static crate::chunk_world::Root>,
    materials: Option<Res<'w, Assets<Material>>>,
    images: Res<'w, Assets<Image>>,
    flags: Query<
        'w,
        's,
        (
            Entity,
            Has<bevy::light::NotShadowCaster>,
            Has<bevy::light::NotShadowReceiver>,
        ),
        With<Checked>,
    >,
}
impl Evidence<'_, '_> {
    pub fn report(&self) -> Value {
        let Some(catalog) = &self.catalog else {
            return Value::Null;
        };
        let mut rows = Vec::new();
        let mut shadow_flags = Vec::new();
        for (entity, no_cast, no_receive) in &self.flags {
            if no_cast || no_receive {
                shadow_flags.push(json!({"entity":format!("{entity:?}"),"cast_shadows":!no_cast,"receive_shadows":!no_receive}));
            }
        }
        let mut shared_resources = self.images.iter().filter_map(|(id,image)| {
            let path=self.server.get_path(id)?.to_string();
            path.starts_with("shared/").then(||json!({"path":path,"handle":format!("{id:?}"),
                "dimensions":[image.width(),image.height()],"format":format!("{:?}",image.texture_descriptor.format),
                "mips":image.texture_descriptor.mip_level_count,"cpu_bytes":image.data.as_ref().map(Vec::len)}))
        }).collect::<Vec<_>>();
        shared_resources.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        for (entity, applied, material) in &self.parts {
            let mut ancestor = entity;
            let mut owner = None;
            for _ in 0..130 {
                if let Ok(root) = self.roots.get(ancestor) {
                    owner = Some([root.0.x, root.0.z]);
                    break;
                }
                let Ok(parent) = self.parents.get(ancestor) else {
                    break;
                };
                ancestor = parent.parent();
            }
            let Some(entry) = catalog.entries.get(&applied.key) else {
                continue;
            };
            let Some(value) = self.materials.as_ref().and_then(|m| m.get(&material.0)) else {
                continue;
            };
            let textures=[&value.extension.top,&value.extension.side,&value.extension.top_normal,&value.extension.side_normal].into_iter().enumerate().filter_map(|(i,h)|self.images.get(h).map(|image|json!({
                "uri":entry.profile.textures()[i].0,"path":self.server.get_path(h.id()).map(|p|p.to_string()),"handle":format!("{:?}",h.id()),"format":format!("{:?}",image.texture_descriptor.format),
                "dimensions":[image.width(),image.height()],"mips":image.texture_descriptor.mip_level_count,"sampler":format!("{:?}",image.sampler)
            }))).collect::<Vec<_>>();
            rows.push(json!({"root":owner,"source_id":applied.source_id,"key":applied.key,"material_handle":format!("{:?}",material.0.id()),
                "source_anchor":entry.profile.source_anchor().metres(),"tiling":value.extension.values.x,"falloff":value.extension.values.y,"diagnostic":value.extension.values.z,
                "phase":value.extension.phase.to_array(),"textures":textures}));
        }
        json!({"profiles":catalog.entries.len(),"phase_updates":catalog.phase_updates,"render_origin":catalog.render_origin,"bindings":rows,"shadow_flags":shadow_flags,"shared_resources":shared_resources,
            "scope":"Bound static-root materials and actual assets; resource/capacity claims require separate qualification"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_and_failed_materials_gate_only_their_own_imported_root() {
        let mut world = World::new();
        let retained = world.spawn(crate::terrain_scene::SceneryRoot).id();
        let candidate = world.spawn(crate::terrain_scene::SceneryRoot).id();
        let part = world
            .spawn((
                ChildOf(candidate),
                Mesh3d(Handle::default()),
                GltfMaterialExtras { value: "{}".into() },
            ))
            .id();
        assert!(root_ready(&world, retained).unwrap());
        assert!(!root_ready(&world, candidate).unwrap());
        world.entity_mut(part).insert((
            Checked,
            Pending {
                key: [0; 32],
                source_id: 1,
            },
        ));
        assert!(!root_ready(&world, candidate).unwrap());
        fail(&mut world, part, "Deliberate dependency failure".into());
        assert_eq!(
            root_ready(&world, candidate),
            Err("Deliberate dependency failure".into())
        );
        assert!(root_ready(&world, retained).unwrap());
        world.entity_mut(part).remove::<Failed>();
        assert!(root_ready(&world, candidate).unwrap());
    }
    #[test]
    fn projection_phase_subtracts_large_global_origins_before_f32_conversion() {
        let anchor = [128e9, 0., -128e9];
        let origin = [128e9 + 62., 0., -128e9 + 52.];
        let actual = frame_phase(origin, anchor, 0.7);
        assert_eq!(actual, crate::triplanar::texture_phase([62., 0., 52.], 0.7));
        let premature =
            std::array::from_fn(|i| f64::from(origin[i] as f32) - f64::from(anchor[i] as f32));
        assert_ne!(actual, crate::triplanar::texture_phase(premature, 0.7));
    }
}
