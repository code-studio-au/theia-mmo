//! Opt-in source layer/static lighting preview through standard Bevy components.
use bevy::{
    asset::LoadState,
    camera::visibility::RenderLayers,
    gltf::GltfExtras,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    light::{DirectionalLightTexture, SpotLightTexture},
    pbr::{DistanceFog, FogFalloff},
    prelude::*,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ambient {
    color: [f32; 3],
    brightness: f32,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fog {
    color: [f32; 3],
    density: f32,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cookie {
    file: String,
    sha256: String,
    dimensions: [u32; 2],
    wrap: u8,
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Directional,
    Point,
    Spot,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Light {
    id: String,
    kind: Kind,
    position: [f32; 3],
    direction: [f32; 3],
    up: [f32; 3],
    color: [f32; 3],
    intensity: f32,
    range: f32,
    mask: u32,
    shadows: bool,
    near_z: f32,
    cone: [f32; 2],
    cookie: Option<usize>,
    cookie_size: f32,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    profile: String,
    geometry_sha256: String,
    layer_roots: usize,
    ambient: Ambient,
    fog: Option<Fog>,
    cookies: Vec<Cookie>,
    lights: Vec<Light>,
}
#[derive(Resource)]
pub struct Config {
    document: Document,
    textures: Vec<Handle<Image>>,
    applied_roots: Option<usize>,
}
#[derive(Component)]
struct SourceLight(String);
#[derive(Resource)]
struct LayersInspected;
#[derive(Resource)]
struct LayerFailure;

fn digest(data: &[u8]) -> String {
    theia_network::terrain::sha256(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
}
fn scalar(value: f32, low: f32, high: f32) -> bool {
    value.is_finite() && (low..=high).contains(&value)
}
fn colour(value: [f32; 3]) -> bool {
    value.into_iter().all(|v| scalar(v, 0., 1.))
}
fn validate(d: &Document) -> Result<(), &'static str> {
    if d.schema != 1
        || d.profile != "unity-static-lighting-preview-v1"
        || !hash(&d.geometry_sha256)
        || !(1..=65536).contains(&d.layer_roots)
        || d.lights.is_empty()
        || d.lights.len() > 64
        || d.cookies.len() > 8
        || !colour(d.ambient.color)
        || !scalar(d.ambient.brightness, 0., 10000.)
    {
        return Err("Invalid bounded lighting profile/ambient");
    }
    if d.fog
        .as_ref()
        .is_some_and(|f| !colour(f.color) || !scalar(f.density, 0., 1.))
    {
        return Err("Invalid source fog");
    }
    let mut ids = BTreeSet::new();
    for c in &d.cookies {
        if !hash(&c.sha256)
            || c.file != format!("lighting-cookie-{}.png", c.sha256)
            || c.dimensions.into_iter().any(|n| !(1..=4096).contains(&n))
            || c.wrap > 1
        {
            return Err("Invalid bounded cookie descriptor");
        }
    }
    for l in &d.lights {
        // Bevy RenderLayers select lights per camera and shadow casters, not
        // individual PBR receivers. Accepting Unity culling masks here would
        // silently illuminate excluded surfaces in this single-camera preview.
        if l.mask != u32::MAX {
            return Err("Selective light receiver masks are unsupported by the standard preview");
        }
        let direction = Vec3::from_array(l.direction);
        let up = Vec3::from_array(l.up);
        if l.id.is_empty()
            || l.id.len() > 128
            || !ids.insert(&l.id)
            || !colour(l.color)
            || !l.position.into_iter().all(|v| scalar(v, -4096., 4096.))
            || !direction.is_finite()
            || !up.is_finite()
            || (direction.length() - 1.).abs() > 0.0001
            || (up.length() - 1.).abs() > 0.0001
            || direction.dot(up).abs() > 0.0001
            || !scalar(l.intensity, 0., 100_000_000.)
            || !scalar(l.range, 0.001, 10000.)
            || !scalar(l.near_z, 0.001, l.range)
            || !scalar(l.cookie_size, 0.001, 4096.)
            || l.cookie.is_some_and(|n| n >= d.cookies.len())
            || (l.kind == Kind::Point && l.cookie.is_some())
        {
            return Err("Invalid source light identity/pose/range/cookie");
        }
        if l.kind == Kind::Spot {
            if !scalar(l.cone[0], 0., l.cone[1])
                || !scalar(l.cone[1], 0.001, std::f32::consts::FRAC_PI_2)
            {
                return Err("Invalid source spot cone");
            }
        } else if l.cone != [0., 0.] {
            return Err("Unexpected non-spot cone");
        }
    }
    Ok(())
}
impl Config {
    pub fn load(path: &Path, geometry: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.canonicalize()?;
        let geometry = geometry.canonicalize()?;
        if path.parent() != geometry.parent()
            || path.metadata()?.len() > 65536
            || geometry.metadata()?.len() > 256 * 1024 * 1024
        {
            return Err("Lighting metadata/geometry containment or byte budget differs".into());
        }
        let document: Document = serde_json::from_slice(&std::fs::read(&path)?)?;
        validate(&document)?;
        if digest(&std::fs::read(&geometry)?) != document.geometry_sha256 {
            return Err("Lighting geometry hash differs".into());
        }
        for c in &document.cookies {
            let requested = path.parent().unwrap().join(&c.file);
            let resolved = requested.canonicalize()?;
            if resolved != requested || resolved.metadata()?.len() > 16 * 1024 * 1024 {
                return Err("Cookie image containment/byte budget differs".into());
            }
            let bytes = std::fs::read(resolved)?;
            if digest(&bytes) != c.sha256
                || bytes.len() < 33
                || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
                || &bytes[12..16] != b"IHDR"
                || bytes[24..29] != [8, 0, 0, 0, 0]
                || u32::from_be_bytes(bytes[16..20].try_into()?) != c.dimensions[0]
                || u32::from_be_bytes(bytes[20..24].try_into()?) != c.dimensions[1]
            {
                return Err("Cookie hash/gray8 PNG dimensions differ".into());
            }
        }
        Ok(Self {
            document,
            textures: Vec::new(),
            applied_roots: None,
        })
    }
    pub fn fog(&self) -> Option<DistanceFog> {
        self.document.fog.as_ref().map(|f| DistanceFog {
            color: Color::linear_rgb(f.color[0], f.color[1], f.color[2]),
            falloff: FogFalloff::ExponentialSquared { density: f.density },
            ..default()
        })
    }
}
fn layers(mask: u32) -> RenderLayers {
    RenderLayers::from_layers(&(0..32).filter(|i| mask & (1 << i) != 0).collect::<Vec<_>>())
}
pub fn camera_layers() -> RenderLayers {
    layers(u32::MAX)
}

pub fn setup(commands: &mut Commands, server: &AssetServer, config: &mut Config) {
    let ambient = &config.document.ambient;
    commands.insert_resource(GlobalAmbientLight {
        color: Color::linear_rgb(ambient.color[0], ambient.color[1], ambient.color[2]),
        brightness: ambient.brightness,
        ..default()
    });
    for c in &config.document.cookies {
        let address = if c.wrap == 0 {
            ImageAddressMode::Repeat
        } else {
            ImageAddressMode::ClampToEdge
        };
        config.textures.push(
            server
                .load_builder()
                .with_settings(move |s: &mut ImageLoaderSettings| {
                    s.is_srgb = false;
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: address,
                        address_mode_v: address,
                        mag_filter: ImageFilterMode::Linear,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    });
                })
                .load(c.file.clone()),
        );
    }
    for l in &config.document.lights {
        let transform = Transform::from_translation(Vec3::from_array(l.position))
            .looking_to(Vec3::from_array(l.direction), Vec3::from_array(l.up));
        let colour = Color::linear_rgb(l.color[0], l.color[1], l.color[2]);
        let mut entity = commands.spawn((SourceLight(l.id.clone()), layers(l.mask), transform));
        match l.kind {
            Kind::Directional => {
                entity.insert(DirectionalLight {
                    color: colour,
                    illuminance: l.intensity,
                    shadow_maps_enabled: l.shadows,
                    ..default()
                });
                if let Some(index) = l.cookie {
                    entity.insert((
                        DirectionalLightTexture {
                            image: config.textures[index].clone(),
                            tiled: config.document.cookies[index].wrap == 0,
                        },
                        transform.with_scale(Vec3::new(l.cookie_size, l.cookie_size, 1.)),
                    ));
                }
            }
            Kind::Point => {
                entity.insert(PointLight {
                    color: colour,
                    intensity: l.intensity,
                    range: l.range,
                    shadow_maps_enabled: l.shadows,
                    shadow_map_near_z: l.near_z,
                    ..default()
                });
            }
            Kind::Spot => {
                entity.insert(SpotLight {
                    color: colour,
                    intensity: l.intensity,
                    range: l.range,
                    shadow_maps_enabled: l.shadows,
                    shadow_map_near_z: l.near_z,
                    inner_angle: l.cone[0],
                    outer_angle: l.cone[1],
                    ..default()
                });
                if let Some(index) = l.cookie {
                    entity.insert(SpotLightTexture {
                        image: config.textures[index].clone(),
                    });
                }
            }
        }
    }
}
fn parse_layer(value: &str) -> Result<Option<usize>, &'static str> {
    let value: Value = serde_json::from_str(value).map_err(|_| "Invalid node extras JSON")?;
    value
        .get("theia_render_layer")
        .map(|v| {
            v.as_u64()
                .filter(|n| *n < 32)
                .map(|n| n as usize)
                .ok_or("Invalid source render layer")
        })
        .transpose()
}
pub fn apply_layers(world: &mut World) {
    if world.contains_resource::<LayersInspected>() || world.contains_resource::<LayerFailure>() {
        return;
    }
    let preview = world.resource::<super::Preview>();
    if !preview.instanced {
        return;
    }
    let Some(root) = preview.root else {
        return;
    };
    let enabled = world.contains_resource::<Config>();
    if world
        .get_resource::<Config>()
        .is_some_and(|c| c.applied_roots.is_some())
    {
        return;
    }
    let mut stack = vec![root];
    let mut marked = Vec::new();
    let mut meshes = BTreeSet::new();
    while let Some(entity) = stack.pop() {
        if world.get::<Mesh3d>(entity).is_some() {
            meshes.insert(entity);
        }
        if let Some(extra) = world.get::<GltfExtras>(entity) {
            match parse_layer(&extra.value) {
                Ok(Some(layer)) => marked.push((entity, layer)),
                Ok(None) => {}
                Err(e) => {
                    error!("{e}");
                    world.insert_resource(LayerFailure);
                    world.write_message(AppExit::error());
                    return;
                }
            }
        }
        if let Some(children) = world.get::<Children>(entity) {
            stack.extend(children.iter());
        }
    }
    if !enabled {
        if !marked.is_empty() {
            error!("Source layers require --lighting metadata");
            world.insert_resource(LayerFailure);
            world.write_message(AppExit::error());
        } else {
            world.insert_resource(LayersInspected);
        }
        return;
    }
    if marked.len() != world.resource::<Config>().document.layer_roots {
        error!("Source layer roots differ from lighting metadata");
        world.insert_resource(LayerFailure);
        world.write_message(AppExit::error());
        return;
    }
    let mut owned = BTreeSet::new();
    for (entity, layer) in &marked {
        let mut descendants = vec![*entity];
        while let Some(entity) = descendants.pop() {
            if world.get::<Mesh3d>(entity).is_some() {
                if !owned.insert(entity) {
                    error!("Overlapping source render layer ownership");
                    world.insert_resource(LayerFailure);
                    world.write_message(AppExit::error());
                    return;
                }
                world.entity_mut(entity).insert(RenderLayers::layer(*layer));
            }
            if let Some(children) = world.get::<Children>(entity) {
                descendants.extend(children.iter());
            }
        }
    }
    if owned != meshes {
        error!("Source render layers omit imported mesh entities");
        world.insert_resource(LayerFailure);
        world.write_message(AppExit::error());
        return;
    }
    world.resource_mut::<Config>().applied_roots = Some(marked.len());
    world.insert_resource(LayersInspected);
}
pub fn ready(world: &World) -> Result<bool, &'static str> {
    if world.contains_resource::<LayerFailure>() {
        return Err("Source layer application failed");
    }
    let Some(c) = world.get_resource::<Config>() else {
        return Ok(true);
    };
    let server = world.resource::<AssetServer>();
    if c.textures
        .iter()
        .any(|h| matches!(server.get_load_state(h.id()), Some(LoadState::Failed(_))))
    {
        return Err("Source lighting cookie dependency failed");
    }
    Ok(c.applied_roots.is_some()
        && c.textures
            .iter()
            .all(|h| world.resource::<Assets<Image>>().contains(h)))
}
pub fn report(world: &mut World) -> Value {
    let Some(config) = world.get_resource::<Config>() else {
        return Value::Null;
    };
    let roots = config.applied_roots;
    let textures = config.textures.clone();
    let mut records = Vec::new();
    for (id, transform, layer, directional, point, spot, dir_cookie, spot_cookie) in world
        .query::<(
            &SourceLight,
            &GlobalTransform,
            &RenderLayers,
            Option<&DirectionalLight>,
            Option<&PointLight>,
            Option<&SpotLight>,
            Option<&DirectionalLightTexture>,
            Option<&SpotLightTexture>,
        )>()
        .iter(world)
    {
        let (kind, colour, intensity, range, shadows) = if let Some(l) = directional {
            (
                "directional",
                l.color,
                l.illuminance,
                None,
                l.shadow_maps_enabled,
            )
        } else if let Some(l) = point {
            (
                "point",
                l.color,
                l.intensity,
                Some(l.range),
                l.shadow_maps_enabled,
            )
        } else {
            let l = spot.unwrap();
            (
                "spot",
                l.color,
                l.intensity,
                Some(l.range),
                l.shadow_maps_enabled,
            )
        };
        records.push(json!({"id":id.0,"kind":kind,"position":transform.translation().to_array(),"direction":transform.forward().to_array(),"up":transform.up().to_array(),"color":colour.to_linear().to_f32_array(),"intensity":intensity,"range":range,"shadows":shadows,"layers":layer.iter().collect::<Vec<_>>(),"directional_cookie":dir_cookie.is_some(),"spot_cookie":spot_cookie.is_some()}));
    }
    records.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let images = world.resource::<Assets<Image>>();
    let cookies=textures.iter().filter_map(|h| images.get(h).map(|i| json!({"handle":format!("{:?}",h.id()),"dimensions":[i.texture_descriptor.size.width,i.texture_descriptor.size.height],"format":format!("{:?}",i.texture_descriptor.format),"mip_levels":i.texture_descriptor.mip_level_count,"sampler":format!("{:?}",i.sampler)}))).collect::<Vec<_>>();
    let fogs=world.query_filtered::<&DistanceFog,With<Camera3d>>().iter(world).map(|f|match &f.falloff {FogFalloff::ExponentialSquared {density}=>json!({"color":f.color.to_linear().to_f32_array(),"mode":"exponential_squared","density":density}),other=>json!({"mode":format!("{other:?}")})}).collect::<Vec<_>>();
    let ambient = world.resource::<GlobalAmbientLight>();
    json!({"applied_layer_roots":roots,"lights":records,"cookies":cookies,"ambient":{"color":ambient.color.to_linear().to_f32_array(),"brightness":ambient.brightness},"fog":fogs,"scope":"actual standard Bevy static light/layer/cookie components and loaded images; projected pixel behavior qualified separately; origin-zero inspector only"})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> Value {
        json!({"schema":1,"profile":"unity-static-lighting-preview-v1","geometry_sha256":digest(b"fixture"),"layer_roots":1,
            "ambient":{"color":[0.1,0.2,0.3],"brightness":300},"fog":null,"cookies":[],
            "lights":[{"id":"fixture","kind":"point","position":[0,5,0],"direction":[0,-1,0],"up":[0,0,1],"color":[1,1,1],"intensity":100000,"range":10,"mask":4294967295u32,"shadows":false,"near_z":0.1,"cone":[0,0],"cookie":null,"cookie_size":1}]})
    }
    #[test]
    fn lighting_boundary_rejects_unknown_fields_duplicate_ids_and_invalid_physical_parameters() {
        let good: Document = serde_json::from_value(document()).unwrap();
        validate(&good).unwrap();
        for fault in [
            "unknown",
            "roots",
            "position",
            "direction",
            "up",
            "colour",
            "intensity",
            "range",
            "near",
            "cone",
            "cookie",
            "duplicate",
            "ambient",
            "fog",
            "receiver_mask",
        ] {
            let mut value = document();
            match fault {
                "unknown" => value["unexpected"] = json!(true),
                "roots" => value["layer_roots"] = json!(65537),
                "position" => value["lights"][0]["position"] = json!([4097, 0, 0]),
                "direction" => value["lights"][0]["direction"] = json!([0, 0, 0]),
                "up" => value["lights"][0]["up"] = json!([0, -1, 0]),
                "colour" => value["lights"][0]["color"] = json!([2, 0, 0]),
                "intensity" => value["lights"][0]["intensity"] = json!(-1),
                "range" => value["lights"][0]["range"] = json!(0),
                "near" => value["lights"][0]["near_z"] = json!(11),
                "cone" => value["lights"][0]["cone"] = json!([0, 0.5]),
                "cookie" => value["lights"][0]["cookie"] = json!(0),
                "duplicate" => {
                    let light = value["lights"][0].clone();
                    value["lights"].as_array_mut().unwrap().push(light);
                }
                "ambient" => value["ambient"]["brightness"] = json!(-1),
                "fog" => value["fog"] = json!({"color":[1,1,1],"density":-1}),
                "receiver_mask" => value["lights"][0]["mask"] = json!(4),
                _ => unreachable!(),
            }
            assert!(
                serde_json::from_value::<Document>(value)
                    .map(|d| validate(&d).is_err())
                    .unwrap_or(true),
                "{fault}"
            );
        }
    }
    #[test]
    fn layer_marks_are_optional_bounded_integers_and_mask_bits_stay_distinct() {
        assert_eq!(parse_layer(r#"{"source_instance":"fixture"}"#), Ok(None));
        for n in 0..32 {
            assert_eq!(
                parse_layer(&format!("{{\"theia_render_layer\":{n}}}")),
                Ok(Some(n))
            );
        }
        for v in ["true", "-1", "32", "1.5", "null", "\"2\""] {
            assert!(parse_layer(&format!("{{\"theia_render_layer\":{v}}}")).is_err());
        }
        assert_eq!(layers(4).iter().collect::<Vec<_>>(), [2]);
        assert_eq!(layers(128).iter().collect::<Vec<_>>(), [7]);
        assert_eq!(camera_layers().iter().count(), 32);
    }
    fn layer_world(unowned: bool) -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<Messages<AppExit>>();
        let root = world.spawn_empty().id();
        let parent = world
            .spawn(GltfExtras {
                value: r#"{"theia_render_layer":2}"#.into(),
            })
            .id();
        let mesh = world.spawn(Mesh3d::default()).id();
        world.entity_mut(root).add_child(parent);
        world.entity_mut(parent).add_child(mesh);
        if unowned {
            let extra = world.spawn(Mesh3d::default()).id();
            world.entity_mut(root).add_child(extra);
        }
        world.insert_resource(super::super::Preview {
            source: Default::default(),
            capture: None,
            handle: Default::default(),
            root: Some(root),
            instanced: true,
            stats: None,
            center: Vec3::ZERO,
            radius: 1.,
            started: std::time::Instant::now(),
            framed_at: None,
            requested: false,
            focus: None,
        });
        let d: Document = serde_json::from_value(document()).unwrap();
        world.insert_resource(Config {
            document: d,
            textures: vec![],
            applied_roots: None,
        });
        (world, mesh)
    }
    #[test]
    fn source_layers_apply_to_real_descendants_and_reject_unowned_meshes() {
        let (mut world, mesh) = layer_world(false);
        apply_layers(&mut world);
        assert_eq!(
            world
                .get::<RenderLayers>(mesh)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            [2]
        );
        assert_eq!(world.resource::<Config>().applied_roots, Some(1));
        assert!(world.contains_resource::<LayersInspected>());
        let (mut world, _) = layer_world(true);
        apply_layers(&mut world);
        assert!(world.contains_resource::<LayerFailure>());
        assert_eq!(world.resource::<Config>().applied_roots, None);
    }
}
