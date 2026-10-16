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
        return Err("Use --inspect-asset <file.glb> [--capture <directory>] [--animation <clip>] [--focus x,y,z,radius[,yaw,pitch]]".into());
    }
    let mut capture_path = None;
    let mut clip = None;
    let mut focus = None;
    for flag in args[2..].as_chunks::<2>().0 {
        match flag[0].as_str() {
            "--capture" if capture_path.is_none() => capture_path = Some(PathBuf::from(&flag[1])),
            "--animation" if clip.is_none() => clip = Some(flag[1].clone()),
            "--focus" if focus.is_none() => focus = Some(Focus::parse(&flag[1])?),
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
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
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
    .add_systems(PostUpdate, frame.after(TransformSystems::Propagate))
    .add_systems(Last, (animation::sample_motion, capture).chain());
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
fn setup(mut commands: Commands, server: Res<AssetServer>, mut preview: ResMut<Preview>) {
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
    commands.spawn((
        Camera3d::default(),
        Tonemapping::Reinhard,
        DebandDither::Disabled,
        Transform::from_xyz(0., 7., 12.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10., 20., 12.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
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
    let mut stack = vec![root];
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut instances = 0usize;
    let mut vertices = 0usize;
    let mut triangles = 0usize;
    let mut normal_meshes = 0usize;
    let mut uv_meshes = 0usize;
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
        vertices += positions.len();
        referenced_vertices += mesh.indices().map_or(positions.len(), |v| {
            v.iter().collect::<std::collections::HashSet<_>>().len()
        });
        triangles += mesh.indices().map_or(positions.len(), |v| v.len()) / 3;
        normal_meshes += usize::from(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        uv_meshes += usize::from(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }
    if instances == 0 || !min.is_finite() || !max.is_finite() {
        error!("GLB scene contains no finite renderable geometry");
        world.write_message(AppExit::error());
        return;
    }
    let center = (min + max) / 2.;
    let size = max - min;
    let gltf = world
        .resource::<Assets<Gltf>>()
        .get(&world.resource::<Preview>().handle)
        .unwrap();
    let source = gltf.source.as_ref().expect("Retained glTF source");
    let materials = gltf.materials.len();
    let images = source.images().count();
    let animations = source.animations().count();
    let stats = json!({"bounds":{"min":min.to_array(),"max":max.to_array(),"size":size.to_array()},"mesh_instances":instances,"render_vertices":vertices,"referenced_vertices":referenced_vertices,"triangles":triangles,"meshes_with_normals":normal_meshes,"meshes_with_uv0":uv_meshes,"authored_materials":materials,"images":images,"animations":animations,"skins":source.skins().count(),"skin_joints":source.skins().map(|s|s.joints().count()).collect::<Vec<_>>(),"source_meshes":gltf.meshes.len()});
    info!("GLB instantiated: {stats}");
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
) {
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
) {
    use bevy::render::view::window::screenshot::{Screenshot, ScreenshotCaptured};
    let Some(output) = preview.capture.clone() else {
        return;
    };
    assert!(
        preview.started.elapsed() < Duration::from_secs(90),
        "Asset screenshot timed out"
    );
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
    let report = json!({"schema":1,"completed":true,"file":filename,"stats":stats,"view":view,"pipelines":pipelines,"animation":playback.report(),"gpu":{"name":adapter.name,"backend":format!("{:?}",adapter.backend)},"scope":"actual Bevy glTF import/rendering and optional sampled animation playback; no gameplay integration or asset-rights qualification"});
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
