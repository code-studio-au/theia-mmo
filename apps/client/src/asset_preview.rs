//! Standard Bevy glTF import qualification, separate from the authoritative world.
use crate::camera::Orbit;
use bevy::{
    asset::{AssetPlugin, LoadState, RecursiveDependencyLoadState},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    gltf::GltfLoaderSettings,
    prelude::*,
    text::FontSize,
    transform::TransformSystems,
    window::WindowResolution,
    world_serialization::WorldInstanceReady,
};
use serde_json::{Value, json};
use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
#[path = "preview_animation.rs"]
mod animation;
#[path = "preview_images.rs"]
mod images;
#[path = "preview_lighting.rs"]
mod lighting;
#[path = "preview_render_flags.rs"]
pub(crate) mod render_flags;

#[derive(Resource)]
struct Preview {
    source: PathBuf,
    capture: Option<PathBuf>,
    handle: Handle<Gltf>,
    root: Option<Entity>,
    instanced: bool,
    stats: Option<Value>,
    center: Vec3,
    radius: f32,
    started: Instant,
    framed_at: Option<Instant>,
    requested: bool,
    focus: Option<Focus>,
}
#[derive(Clone, Copy, Debug)]
struct Focus {
    center: Vec3,
    radius: f32,
    angles: Option<Vec2>,
}
impl Focus {
    fn parse(value: &str) -> Result<Self, &'static str> {
        let values: Vec<f32> = value
            .split(',')
            .map(|v| v.trim().parse::<f32>())
            .collect::<Result<_, _>>()
            .map_err(|_| "Focus must be x,y,z,radius[,yaw,pitch]")?;
        if !matches!(values.len(), 4 | 6)
            || values.iter().any(|v| !v.is_finite())
            || values[3] <= 0.
            || !(values[3] * 10.).is_finite()
        {
            return Err(
                "Focus needs x,y,z,radius with optional yaw,pitch; finite numbers and positive radius required",
            );
        }
        Ok(Self {
            center: Vec3::new(values[0], values[1], values[2]),
            radius: values[3],
            angles: (values.len() == 6).then(|| Vec2::new(values[4], values[5])),
        })
    }
}
#[derive(Component)]
struct Hud;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return Err("Use --inspect-asset <file.glb> [--capture <directory>] [--animation <clip>] [--focus x,y,z,radius[,yaw,pitch]] [--lighting <metadata>] [--triplanar <metadata> | --triplanar-diagnostic <metadata> | --triplanar-normals <metadata>] [--texture-origin x,y,z]".into());
    }
    let mut capture_path = None;
    let mut clip = None;
    let mut focus = None;
    let mut triplanar = None;
    let mut lighting = None;
    let mut diagnostic = 0;
    let mut texture_origin = None;
    for flag in args[2..].as_chunks::<2>().0 {
        match flag[0].as_str() {
            "--capture" if capture_path.is_none() => capture_path = Some(PathBuf::from(&flag[1])),
            "--lighting" if lighting.is_none() => lighting = Some(PathBuf::from(&flag[1])),
            "--animation" if clip.is_none() => clip = Some(flag[1].clone()),
            "--triplanar" if triplanar.is_none() => triplanar = Some(PathBuf::from(&flag[1])),
            "--triplanar-diagnostic" if triplanar.is_none() => {
                triplanar = Some(PathBuf::from(&flag[1]));
                diagnostic = 1;
            }
            "--triplanar-normals" if triplanar.is_none() => {
                triplanar = Some(PathBuf::from(&flag[1]));
                diagnostic = 2;
            }
            "--focus" if focus.is_none() => focus = Some(Focus::parse(&flag[1])?),
            "--texture-origin" if texture_origin.is_none() => {
                let values = flag[1]
                    .split(',')
                    .map(str::parse::<f64>)
                    .collect::<Result<Vec<_>, _>>()?;
                texture_origin =
                    Some(<[f64; 3]>::try_from(values).map_err(|_| "Texture origin needs x,y,z")?);
            }
            _ => return Err(format!("Unknown or duplicate option {}", flag[0]).into()),
        }
    }
    let source = PathBuf::from(&args[1]).canonicalize()?;
    let mut file = std::fs::File::open(&source)?;
    let mut header = [0; 12];
    file.read_exact(&mut header)?;
    validate_header(header, file.metadata()?.len())?;
    let parent = source
        .parent()
        .ok_or("Asset needs a parent directory")?
        .to_str()
        .ok_or("Asset path must be UTF-8")?
        .to_owned();
    let capture_output = if let Some(output) = capture_path {
        std::fs::create_dir_all(&output)?;
        for name in ["asset.json", "scene.png"] {
            let path = output.join(name);
            if path.exists() {
                std::fs::remove_file(path)?;
            }
        }
        Some(output)
    } else {
        None
    };
    let mut triplanar = triplanar
        .map(|p| crate::triplanar::Config::load(&p, &source, diagnostic))
        .transpose()?;
    if let Some(origin) = texture_origin {
        triplanar
            .as_mut()
            .ok_or("--texture-origin requires triplanar inspection")?
            .set_texture_origin(origin)?;
    }
    if lighting.is_some() && diagnostic != 0 {
        return Err("Source lighting is only available in lit inspection".into());
    }
    let lighting = lighting
        .map(|p| lighting::Config::load(&p, &source))
        .transpose()?;
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::audio::AudioPlugin>()
            .set(AssetPlugin {
                file_path: parent,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Theia — GLB inspection".into(),
                    resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(crate::capture_ready::CaptureReadyPlugin)
    .insert_resource(Preview {
        source,
        capture: capture_output,
        handle: default(),
        root: None,
        instanced: false,
        stats: None,
        center: Vec3::ZERO,
        radius: 1.,
        started: Instant::now(),
        framed_at: None,
        requested: false,
        focus,
    })
    .insert_resource(ClearColor(Color::srgb(0.10, 0.15, 0.22)))
    .insert_resource(animation::Playback::new(clip))
    .init_resource::<Orbit>()
    .add_systems(Startup, setup)
    .add_systems(
        PreUpdate,
        crate::camera::controls.after(bevy::input::InputSystems),
    )
    .add_systems(Update, (load, animation::start, follow).chain())
    .add_systems(
        PostUpdate,
        (render_flags::apply, lighting::apply_layers, frame)
            .chain()
            .after(TransformSystems::Propagate),
    )
    .add_systems(Last, (animation::sample_motion, capture).chain());
    if let Some(config) = lighting {
        app.insert_resource(config);
    }
    if let Some(config) = triplanar {
        app.insert_resource(config)
            .add_plugins(crate::triplanar::TriplanarPlugin);
    }
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => {
            Err(format!("Asset inspection failed ({code}); inspect logs").into())
        }
    }
}
fn validate_header(header: [u8; 12], length: u64) -> Result<(), Box<dyn std::error::Error>> {
    if &header[..4] != b"glTF"
        || u32::from_le_bytes(header[4..8].try_into()?) != 2
        || u64::from(u32::from_le_bytes(header[8..12].try_into()?)) != length
    {
        return Err("Expected a complete glTF 2.0 binary (.glb) file".into());
    }
    Ok(())
}
fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut preview: ResMut<Preview>,
    config: Option<Res<crate::triplanar::Config>>,
    mut lighting: Option<ResMut<lighting::Config>>,
) {
    let name = preview
        .source
        .file_name()
        .unwrap()
        .to_str()
        .expect("UTF-8 asset path")
        .to_owned();
    preview.handle = server
        .load_builder()
        .with_settings(|settings: &mut GltfLoaderSettings| {
            settings.include_source = true;
        })
        .load(name);
    let mut camera = commands.spawn((
        Camera3d::default(),
        if config.as_ref().is_some_and(|c| c.diagnostic()) {
            Tonemapping::None
        } else {
            Tonemapping::Reinhard
        },
        DebandDither::Disabled,
        Transform::from_xyz(0., 7., 12.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    if let Some(config) = lighting.as_ref() {
        camera.insert(lighting::camera_layers());
        if let Some(fog) = config.fog() {
            camera.insert(fog);
        }
    }
    if let Some(config) = lighting.as_mut() {
        lighting::setup(&mut commands, &server, config);
    } else {
        commands.spawn((
            DirectionalLight {
                illuminance: 8000.,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(10., 20., 12.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }
    commands.spawn((
        Hud,
        Text::new("THEIA | Asset inspection\nLoading GLB..."),
        TextFont {
            font_size: FontSize::Px(16.),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            ..default()
        },
    ));
}
fn load(
    mut commands: Commands,
    server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut preview: ResMut<Preview>,
    mut exit: MessageWriter<AppExit>,
) {
    if let Some(LoadState::Failed(error)) = server.get_load_state(preview.handle.id()) {
        error!("GLB load failed: {error}");
        exit.write(AppExit::error());
        return;
    }
    if let Some(RecursiveDependencyLoadState::Failed(error)) =
        server.get_recursive_dependency_load_state(preview.handle.id())
    {
        error!("GLB dependency failed: {error}");
        exit.write(AppExit::error());
        return;
    }
    if preview.root.is_none() && server.is_loaded_with_dependencies(preview.handle.id()) {
        let gltf = gltfs.get(&preview.handle).expect("Loaded GLB");
        let Some(scene) = gltf
            .default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
        else {
            error!("GLB has no scene");
            exit.write(AppExit::error());
            return;
        };
        let entity = commands
            .spawn(WorldAssetRoot(scene))
            .observe(|_: On<WorldInstanceReady>, mut preview: ResMut<Preview>| {
                preview.instanced = true;
            })
            .id();
        preview.root = Some(entity);
    }
}

/// Run once after Bevy has instantiated and propagated the actual imported nodes.
fn frame(world: &mut World) {
    let preview = world.resource::<Preview>();
    if !preview.instanced || preview.stats.is_some() {
        return;
    }
    let root = preview.root.unwrap();
    let triplanar = crate::triplanar::report(world);
    let extras: Vec<String> = world
        .query::<(&bevy::gltf::GltfMaterialExtras, &Mesh3d)>()
        .iter(world)
        .map(|(extra, _)| extra.value.clone())
        .collect();
    if let Some(config) = world.get_resource::<crate::triplanar::Config>() {
        let material_count = world
            .resource::<Assets<Gltf>>()
            .get(&world.resource::<Preview>().handle)
            .unwrap()
            .materials
            .len();
        match config.validate_scene(material_count, &extras) {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                error!("{error}");
                world.write_message(AppExit::error());
                return;
            }
        }
    } else if extras
        .iter()
        .any(|extra| crate::triplanar::material_profile(extra) != Ok(None))
    {
        error!("Triplanar material requires --triplanar metadata");
        world.write_message(AppExit::error());
        return;
    }
    match lighting::ready(world) {
        Ok(true) => {}
        Ok(false) => return,
        Err(e) => {
            error!("{e}");
            world.write_message(AppExit::error());
            return;
        }
    }
    let mut stack = vec![root];
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut instances = 0usize;
    let mut standard_material_instances = 0usize;
    let mut vertices = 0usize;
    let mut triangles = 0usize;
    let mut normal_meshes = 0usize;
    let mut uv_meshes = 0usize;
    let mut uv1_meshes = 0usize;
    let mut material_uv_channels = std::collections::BTreeMap::new();
    let mut mesh_layers = std::collections::BTreeMap::new();
    let mut unlit_instances = 0usize;
    let mut emissive_max_rgb = Vec3::ZERO;
    let mut no_shadow_casters = 0usize;
    let mut no_shadow_receivers = 0usize;
    let mut referenced_vertices = 0usize;
    while let Some(entity) = stack.pop() {
        if let Some(children) = world.get::<Children>(entity) {
            stack.extend(children.iter());
        }
        let (Some(handle), Some(transform)) = (
            world.get::<Mesh3d>(entity),
            world.get::<GlobalTransform>(entity),
        ) else {
            continue;
        };
        let Some(mesh) = world.resource::<Assets<Mesh>>().get(&handle.0) else {
            return;
        };
        let Some(positions) = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|v| v.as_float3())
        else {
            continue;
        };
        for position in positions {
            let position = transform
                .affine()
                .transform_point3(Vec3::from_array(*position));
            min = min.min(position);
            max = max.max(position);
        }
        instances += 1;
        let layers = world
            .get::<bevy::camera::visibility::RenderLayers>(entity)
            .map(|v| v.iter().collect::<Vec<_>>())
            .unwrap_or_else(|| vec![0]);
        *mesh_layers.entry(layers).or_insert(0usize) += 1;
        standard_material_instances += usize::from(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_some(),
        );
        if let Some(handle) = world.get::<MeshMaterial3d<StandardMaterial>>(entity) {
            let Some(material) = world.resource::<Assets<StandardMaterial>>().get(&handle.0) else {
                return;
            };
            let channel = |value: &bevy::mesh::UvChannel| match value {
                bevy::mesh::UvChannel::Uv0 => 0_u8,
                bevy::mesh::UvChannel::Uv1 => 1_u8,
            };
            unlit_instances += usize::from(material.unlit);
            emissive_max_rgb = emissive_max_rgb.max(Vec3::new(
                material.emissive.red,
                material.emissive.green,
                material.emissive.blue,
            ));
            *material_uv_channels
                .entry((
                    channel(&material.base_color_channel),
                    channel(&material.normal_map_channel),
                    channel(&material.occlusion_channel),
                ))
                .or_insert(0usize) += 1;
        }
        vertices += positions.len();
        no_shadow_casters +=
            usize::from(world.get::<bevy::light::NotShadowCaster>(entity).is_some());
        no_shadow_receivers += usize::from(
            world
                .get::<bevy::light::NotShadowReceiver>(entity)
                .is_some(),
        );
        referenced_vertices += mesh.indices().map_or(positions.len(), |v| {
            v.iter().collect::<std::collections::HashSet<_>>().len()
        });
        triangles += mesh.indices().map_or(positions.len(), |v| v.len()) / 3;
        normal_meshes += usize::from(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        uv_meshes += usize::from(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        uv1_meshes += usize::from(mesh.attribute(Mesh::ATTRIBUTE_UV_1).is_some());
    }
    if instances == 0 || !min.is_finite() || !max.is_finite() {
        error!("GLB scene contains no finite renderable geometry");
        world.write_message(AppExit::error());
        return;
    }
    let center = (min + max) / 2.;
    if !triplanar.is_null()
        && (!min.is_finite()
            || !max.is_finite()
            || min.abs().max_element() > 4096.
            || max.abs().max_element() > 4096.)
    {
        error!("Triplanar prototype exceeds bounded source frame");
        world.write_message(AppExit::error());
        return;
    }
    let size = max - min;
    let lighting_report = lighting::report(world);
    let gltf = world
        .resource::<Assets<Gltf>>()
        .get(&world.resource::<Preview>().handle)
        .unwrap();
    let Some(image_inventory) = images::inventory(world, gltf) else {
        return;
    };
    let source = gltf.source.as_ref().expect("Retained glTF source");
    let materials = gltf.materials.len();
    let images = source.images().count();
    let animations = source.animations().count();
    let mut stats = json!({"bounds":{"min":min.to_array(),"max":max.to_array(),"size":size.to_array()},"mesh_instances":instances,"render_vertices":vertices,"referenced_vertices":referenced_vertices,"triangles":triangles,"meshes_with_normals":normal_meshes,"meshes_with_uv0":uv_meshes,"authored_materials":materials,"images":images,"animations":animations,"skins":source.skins().count(),"skin_joints":source.skins().map(|s|s.joints().count()).collect::<Vec<_>>(),"source_meshes":gltf.meshes.len()});
    stats["meshes_with_uv1"] = json!(uv1_meshes);
    stats["mesh_render_layers"] = json!(
        mesh_layers
            .into_iter()
            .map(|(layers, primitives)| json!({"layers":layers,"primitives":primitives}))
            .collect::<Vec<_>>()
    );
    stats["gltf_core_image_inventory"] = image_inventory;
    if !lighting_report.is_null() {
        stats["lighting"] = lighting_report;
    }
    stats["unlit_standard_material_instances"] = json!(unlit_instances);
    stats["standard_material_emissive_max_rgb"] = json!(emissive_max_rgb.to_array());
    stats["meshes_not_shadow_casters"] = json!(no_shadow_casters);
    stats["meshes_not_shadow_receivers"] = json!(no_shadow_receivers);
    stats["standard_material_uv_channels"] = json!(
        material_uv_channels
            .into_iter()
            .map(|((base_color, normal, occlusion), primitives)| {
                json!({"base_color":base_color,"normal":normal,"occlusion":occlusion,"primitives":primitives})
            })
            .collect::<Vec<_>>()
    );
    if !triplanar.is_null() {
        stats["triplanar"] = triplanar.clone();
        stats["standard_material_instances"] = json!(standard_material_instances);
    }
    info!("GLB instantiated: {stats}");
    if !triplanar.is_null() {
        info!("Triplanar material applied: {triplanar}");
    }
    if !world
        .get_resource::<crate::triplanar::Config>()
        .is_some_and(|c| c.diagnostic())
    {
        let floor_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Plane3d::default().mesh().size(
                size.x.max(size.z).max(5.) * 3.,
                size.x.max(size.z).max(5.) * 3.,
            ));
        let floor_material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(Color::srgb(0.18, 0.24, 0.19));
        world.spawn((
            Mesh3d(floor_mesh),
            MeshMaterial3d(floor_material),
            Transform::from_xyz(center.x, min.y - 0.03, center.z),
        ));
        let reference_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Capsule3d::new(0.35, 1.1));
        let reference_material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(Color::srgb(0.12, 0.42, 0.95));
        world.spawn((
            Mesh3d(reference_mesh),
            MeshMaterial3d(reference_material),
            Transform::from_xyz(max.x + 1.5, min.y + 0.9, center.z),
        ));
    }
    let focus = world.resource::<Preview>().focus.unwrap_or(Focus {
        center,
        radius: size.length() / 2.,
        angles: None,
    });
    {
        let mut orbit = world.resource_mut::<Orbit>();
        orbit.frame_asset(focus.radius);
        if let Some(angles) = focus.angles {
            orbit.set_angles(angles.x, angles.y);
        }
    }
    world
        .resource_mut::<crate::capture_ready::CaptureReady>()
        .reset();
    let mut preview = world.resource_mut::<Preview>();
    preview.center = focus.center;
    preview.radius = focus.radius;
    preview.stats = Some(stats);
    preview.framed_at = Some(Instant::now());
    let filename = preview
        .source
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let text = format!(
        "THEIA | {filename}\n{triangles} triangles | {materials} authored materials | {images} images\nSize: {:.2} x {:.2} x {:.2} source units | Blue capsule: 1.8 units\nHold RMB: orbit | Wheel: zoom | Home: reset | Esc: quit\nVisual inspection only; no gameplay collision",
        size.x, size.y, size.z
    );
    let mut hud = world.query_filtered::<&mut Text, With<Hud>>();
    for mut value in hud.iter_mut(world) {
        **value = text.clone();
    }
}
fn follow(
    preview: Res<Preview>,
    mut orbit: ResMut<Orbit>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
    mut exit: MessageWriter<AppExit>,
    ready: Res<crate::capture_ready::CaptureReady>,
) {
    if ready.failed > 0 {
        error!("Asset render pipeline compilation failed");
        exit.write(AppExit::error());
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if preview.stats.is_some() {
        if keys.just_pressed(KeyCode::Home) {
            orbit.frame_asset(preview.radius);
            if let Some(angles) = preview.focus.and_then(|f| f.angles) {
                orbit.set_angles(angles.x, angles.y);
            }
        }
        if let Ok((mut camera, mut projection)) = camera.single_mut() {
            *camera = Transform::from_translation(orbit.unobstructed_eye(preview.center))
                .looking_at(preview.center, Vec3::Y);
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.far = orbit.asset_far_plane(preview.radius);
            }
        }
    }
    if preview.started.elapsed() > Duration::from_secs(60) && preview.stats.is_none() {
        error!("GLB loading/instantiation timed out");
        exit.write(AppExit::error());
    }
}
fn capture(
    mut commands: Commands,
    mut preview: ResMut<Preview>,
    adapter: Res<bevy::render::renderer::RenderAdapterInfo>,
    playback: Res<animation::Playback>,
    orbit: Res<Orbit>,
    ready: Res<crate::capture_ready::CaptureReady>,
    camera: Query<(&Projection, &GlobalTransform), With<Camera3d>>,
) {
    use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
    let Some(output) = preview.capture.clone() else {
        return;
    };
    if preview.started.elapsed() >= Duration::from_secs(90) {
        error!("Asset screenshot timed out");
        commands.write_message(AppExit::error());
        return;
    }
    if preview.requested
        || !preview
            .framed_at
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
    {
        return;
    }
    if !playback.ready_to_capture() || !ready.ready() {
        return;
    }
    preview.requested = true;
    let stats = preview.stats.clone().unwrap();
    let filename = preview
        .source
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let view = json!({"center":preview.center.to_array(),"radius":preview.radius,"explicit_focus":preview.focus.is_some(),"angles":orbit.angles().to_array()});
    let pipelines = json!({"pending":ready.pending,"failed":ready.failed,"compiled":ready.compiled,"clear_frames":ready.clear_frames});
    let clip_from_world = camera.single().ok().map(|(projection, transform)| {
        (projection.get_clip_from_view() * transform.to_matrix().inverse()).to_cols_array()
    });
    let report = json!({"clip_from_world":clip_from_world,"schema":1,"completed":true,"file":filename,"stats":stats,"view":view,"pipelines":pipelines,"animation":playback.report(),"gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend)},"scope":"actual Bevy glTF import/rendering and optional sampled animation playback; no gameplay integration or asset-rights qualification"});
    commands.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .unwrap()
                .save(output.join("scene.png"))
                .unwrap();
            std::fs::write(
                output.join("asset.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            exit.write(AppExit::Success);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_parses_world_coordinates_and_rejects_invalid_camera_ranges() {
        let focus = Focus::parse("245, 40,2250,70").unwrap();
        assert_eq!(focus.center, Vec3::new(245., 40., 2250.));
        assert_eq!(focus.radius, 70.);
        assert!(focus.angles.is_none());
        let focus = Focus::parse("55,22,2490,100,-1.57,0.9").unwrap();
        assert_eq!(focus.angles.unwrap(), Vec2::new(-1.57, 0.9));
        for value in [
            "",
            "1,2,3",
            "1,2,3,4,5",
            "1,nan,3,4",
            "1,2,3,0",
            "1,2,3,-4",
            "1,2,3,inf",
            "1,2,3,3e38",
            "1,2,3,4,nan,0.9",
        ] {
            assert!(Focus::parse(value).is_err(), "{value}");
        }
    }
    #[test]
    fn glb_preflight_rejects_wrong_versions_and_incomplete_files() {
        let mut header = *b"glTF\x02\x00\x00\x00\x14\x00\x00\x00";
        assert!(validate_header(header, 20).is_ok());
        assert!(validate_header(header, 19).is_err());
        header[4] = 1;
        assert!(validate_header(header, 20).is_err());
        header[4] = 2;
        header[0] = 0;
        assert!(validate_header(header, 20).is_err());
    }
}
