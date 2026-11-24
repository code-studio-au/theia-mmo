//! Standard Bevy GLB presentation for bounded authored prediction worlds.
//! Movement readiness requires the actual scene, GPU assets and world pipelines.
use bevy::{
    asset::{
        AssetApp, LoadState,
        io::{
            AssetSourceBuilder,
            memory::{Dir, MemoryAssetReader, Value},
        },
    },
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstance, WorldInstanceSpawner},
};
use std::{path::Path, sync::Arc, time::Instant};
use theia_network::authored_world::Store;

#[derive(Resource)]
pub(crate) struct Scene {
    store: Arc<Store>,
    asset: Option<Handle<Gltf>>,
    root: Option<Entity>,
    cpu_ready: bool,
    gpu_clear_frames: u32,
    waiting_since: Option<Instant>,
    diagnostics: Diagnostics,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Diagnostics {
    ready: bool,
    retractions: u64,
    cpu_ready: bool,
    meshes: usize,
    assets_ready: bool,
    pipelines: usize,
    pipelines_ready: bool,
}
impl Scene {
    pub(crate) fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    pub(crate) fn required_at(&self, point: theia_core::coordinates::GlobalPosition) -> bool {
        self.store.imported().is_none()
            || theia_network::procedural::owner(point)
                .is_some_and(|owner| self.store.objects_intersect_patch(owner))
    }
    pub(crate) fn has_streamed_ground(&self) -> bool {
        self.store.imported().is_some()
    }
    pub(crate) fn map_contains(&self, point: theia_core::coordinates::GlobalPosition) -> bool {
        self.ready() && self.store.map_contains(point)
    }
    pub(crate) fn map_height(&self, point: theia_core::coordinates::GlobalPosition) -> Option<f32> {
        self.ready().then(|| self.store.map_height(point)).flatten()
    }
    pub(crate) fn ready(&self) -> bool {
        self.cpu_ready && self.gpu_clear_frames >= 3
    }
}
#[derive(Component)]
struct AuthoredMesh;
// Temporarily make every mesh eligible for standard Bevy specialization. A
// camera-facing subset is not sufficient to qualify a whole authored world.
#[derive(Component)]
struct PreparingPipeline;

pub(crate) fn install(app: &mut App, store: Arc<Store>) {
    let root = Dir::default();
    root.insert_asset(Path::new("terrain.glb"), Value::Vec(store.visual()));
    app.register_asset_source(
        "authored",
        AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: root.clone() })),
    );
    app.insert_resource(Scene {
        store,
        asset: None,
        root: None,
        cpu_ready: false,
        gpu_clear_frames: 0,
        waiting_since: Some(Instant::now()),
        diagnostics: Diagnostics::default(),
    })
    .add_systems(
        Update,
        tick.after(crate::network::present)
            .before(crate::graybox::tick),
    );
}
fn tick(world: &mut World) {
    if crate::loading::failed(world) {
        return;
    }
    let mut scene = world.remove_resource::<Scene>().expect("Authored scene");
    let result = prepare(world, &mut scene);
    if let Err(error) = result {
        error!("Authored scene failed: {error}");
        scene.cpu_ready = false;
        scene.gpu_clear_frames = 0;
        crate::loading::fail_or_exit(world, "The world could not be prepared.");
    }
    world.insert_resource(scene);
}
fn prepare(world: &mut World, scene: &mut Scene) -> Result<(), String> {
    if scene.asset.is_none() {
        scene.asset = Some(
            world
                .resource::<AssetServer>()
                .load("authored://terrain.glb"),
        );
    }
    let asset = scene.asset.as_ref().unwrap();
    let server = world.resource::<AssetServer>();
    if let Some(LoadState::Failed(error)) = server.get_load_state(asset.id()) {
        return Err(error.to_string());
    }
    let needed = world
        .get_resource::<crate::network::Network>()
        .and_then(|n| n.state().authority_focus())
        .is_none_or(|p| scene.required_at(p));
    readiness_deadline(
        scene.ready(),
        &mut scene.waiting_since,
        Instant::now(),
        needed && !crate::frontend::active(world),
    )?;
    if !server.is_loaded_with_dependencies(asset.id()) {
        scene.cpu_ready = false;
        return Ok(());
    }
    if scene.root.is_none() {
        let gltf = world
            .resource::<Assets<Gltf>>()
            .get(asset)
            .ok_or("Missing loaded GLB")?;
        if !gltf.skins.is_empty() || !gltf.animations.is_empty() {
            return Err("Authored world must be static".into());
        }
        let handle = gltf
            .default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
            .ok_or("Missing authored scene")?;
        scene.root = Some(
            world
                .spawn((
                    crate::terrain_scene::SceneryRoot,
                    WorldAssetRoot(handle),
                    Transform::default(),
                ))
                .id(),
        );
    }
    let root = scene.root.unwrap();
    let local = world
        .resource::<crate::floating_origin::Frame>()
        .render
        .to_local(scene.store.origin());
    // Before admission the initial frame can be distant. Do not display misplaced
    // geometry or grant readiness until the server-owned focus establishes a frame.
    let Some(local) = local else {
        if needed {
            scene.cpu_ready = false;
        }
        world.entity_mut(root).insert(Visibility::Hidden);
        return Ok(());
    };
    world.entity_mut(root).insert(Visibility::Inherited);
    world
        .get_mut::<Transform>(root)
        .ok_or("Authored root removed")?
        .translation = Vec3::from_array(local.metres());
    let instantiated = world.get::<WorldInstance>(root).is_some_and(|instance| {
        world
            .resource::<WorldInstanceSpawner>()
            .instance_is_ready(**instance)
    });
    if !instantiated {
        scene.cpu_ready = false;
        return Ok(());
    }
    let mut pending = vec![root];
    let mut count = 0;
    while let Some(entity) = pending.pop() {
        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter());
        }
        if world.get::<Mesh3d>(entity).is_some() {
            if world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_none()
            {
                return Err("Authored static mesh requires a standard material".into());
            }
            let mesh = world.get::<Mesh3d>(entity).unwrap();
            let material = world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .unwrap();
            if !world.resource::<Assets<Mesh>>().contains(mesh.id())
                || !world
                    .resource::<Assets<StandardMaterial>>()
                    .contains(material.id())
            {
                scene.cpu_ready = false;
                return Ok(());
            }
            world.entity_mut(entity).insert((
                AuthoredMesh,
                crate::terrain_scene::CameraScenery,
                bevy::picking::mesh_picking::ray_cast::RayCastBackfaces,
            ));
            if scene.ready() {
                if world.get::<PreparingPipeline>(entity).is_some() {
                    world.entity_mut(entity).remove::<(
                        PreparingPipeline,
                        bevy::camera::visibility::NoFrustumCulling,
                    )>();
                }
            } else if world
                .get::<bevy::camera::visibility::NoFrustumCulling>(entity)
                .is_none()
            {
                world.entity_mut(entity).insert((
                    PreparingPipeline,
                    bevy::camera::visibility::NoFrustumCulling,
                ));
            }
            count += 1;
        }
    }
    scene.cpu_ready = count > 0 && crate::runtime_materials::root_ready(world, root)?;
    Ok(())
}

// Readiness can retract after loading (for example while GPU assets prepare).
// Bound each uninterrupted wait, not the total lifetime of a playable scene.
fn readiness_deadline(
    ready: bool,
    waiting_since: &mut Option<Instant>,
    now: Instant,
    required: bool,
) -> Result<(), String> {
    if ready || !required {
        *waiting_since = None;
    } else if now
        .duration_since(*waiting_since.get_or_insert(now))
        .as_secs()
        > 120
    {
        return Err("Scene dependencies or GPU preparation timed out".into());
    }
    Ok(())
}

pub(crate) struct ReadinessPlugin;
impl Plugin for ReadinessPlugin {
    fn build(&self, app: &mut App) {
        app.sub_app_mut(bevy::render::RenderApp)
            .add_systems(bevy::render::ExtractSchedule, observe);
    }
}
fn observe(
    mut main: ResMut<bevy::render::MainWorld>,
    meshes: Res<bevy::render::render_asset::RenderAssets<bevy::render::mesh::RenderMesh>>,
    materials: Res<
        bevy::render::erased_render_asset::ErasedRenderAssets<bevy::pbr::PreparedMaterial>,
    >,
    specialized: Res<bevy::pbr::SpecializedMaterialPipelineCache>,
    pipelines: Res<bevy::render::render_resource::PipelineCache>,
) {
    let Some(scene) = main.get_resource::<Scene>() else {
        return;
    };
    let cpu_ready = scene.cpu_ready;
    let entries: Vec<_> = main
        .query_filtered::<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), With<AuthoredMesh>>(
        )
        .iter(&main)
        .map(|(entity, mesh, material)| {
            (
                bevy::render::sync_world::MainEntity::from(entity),
                mesh.id(),
                material.id(),
            )
        })
        .collect();
    let assets_ready = !entries.is_empty()
        && entries.iter().all(|(_, mesh, material)| {
            meshes.get(*mesh).is_some() && materials.get(material.untyped()).is_some()
        });
    let world_pipelines: Vec<_> = specialized
        .values()
        .flat_map(|view| {
            entries
                .iter()
                .filter_map(move |(entity, _, _)| view.get(entity))
        })
        .collect();
    let pipelines_ready = !entries.is_empty()
        && entries
            .iter()
            .all(|(entity, _, _)| specialized.values().any(|view| view.contains_key(entity)))
        && world_pipelines
            .iter()
            .all(|id| pipelines.get_render_pipeline(**id).is_some());
    let mut scene = main.resource_mut::<Scene>();
    scene.gpu_clear_frames = if cpu_ready && assets_ready && pipelines_ready {
        scene.gpu_clear_frames.saturating_add(1)
    } else {
        0
    };
    let ready = scene.ready();
    let retracted = scene.diagnostics.ready && !ready;
    scene.diagnostics = Diagnostics {
        ready,
        retractions: scene.diagnostics.retractions + u64::from(retracted),
        cpu_ready,
        meshes: entries.len(),
        assets_ready,
        pipelines: world_pipelines.len(),
        pipelines_ready,
    };
    if retracted {
        info!(
            "authored_readiness {}",
            serde_json::to_string(&scene.diagnostics).unwrap()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_timeout_bounds_startup_and_recovery_instead_of_scene_age() {
        use std::time::Duration;
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut waiting = Some(start);
        assert!(readiness_deadline(false, &mut waiting, at(121), true).is_err());

        // Menus have no admitted render origin. Time spent there must not
        // consume the world-entry budget, even if CPU assets loaded already.
        assert!(readiness_deadline(false, &mut waiting, at(600), false).is_ok());
        assert!(waiting.is_none());
        assert!(readiness_deadline(false, &mut waiting, at(1200), true).is_ok());
        assert!(readiness_deadline(false, &mut waiting, at(1321), true).is_err());

        // A scene that loaded successfully may need to prepare again hours later.
        assert!(readiness_deadline(true, &mut waiting, at(1322), true).is_ok());
        assert!(readiness_deadline(false, &mut waiting, at(7200), true).is_ok());
        assert!(readiness_deadline(false, &mut waiting, at(7319), true).is_ok());
        assert!(readiness_deadline(true, &mut waiting, at(7320), true).is_ok());
        assert!(readiness_deadline(false, &mut waiting, at(14400), true).is_ok());
        // Recovery is still bounded; repeated checks must not extend its deadline.
        assert!(readiness_deadline(false, &mut waiting, at(14521), true).is_err());
    }
    #[test]
    #[ignore = "requires THEIA_AUTHORED_WORLD, offscreen Vulkan GPU and optional THEIA_AUTHORED_CAPTURE"]
    fn actual_authored_gpu_readiness_without_window() {
        let root = std::env::var_os("THEIA_AUTHORED_WORLD").expect("Local authored package");
        let store = Arc::new(Store::open(Path::new(&root)).unwrap());
        let origin = store.origin();
        let mut app = App::new();
        install(&mut app, store);
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..default()
                })
                .disable::<bevy::winit::WinitPlugin>()
                .disable::<bevy::audio::AudioPlugin>()
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
        )
        .add_plugins((
            ReadinessPlugin,
            crate::runtime_materials::RuntimeMaterialsPlugin,
            crate::capture_ready::CaptureReadyPlugin,
        ));
        let mut frame = crate::floating_origin::Frame::default();
        frame.render = theia_core::coordinates::LocalFrame::new(origin);
        app.insert_resource(frame);
        app.insert_resource(GlobalAmbientLight {
            brightness: 350.,
            ..default()
        });
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                1024,
                768,
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                None,
            ));
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                bevy::core_pipeline::tonemapping::Tonemapping::Reinhard,
                bevy::camera::RenderTarget::Image(image.clone().into()),
                Transform::from_xyz(45., 65., 80.).looking_at(Vec3::ZERO, Vec3::Y),
            ))
            .id();
        app.world_mut().spawn((
            DirectionalLight::default(),
            Transform::from_xyz(10., 20., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        while app.plugins_state() != bevy::app::PluginsState::Ready {
            std::thread::yield_now();
        }
        app.finish();
        app.cleanup();
        let info = app
            .sub_app(bevy::render::RenderApp)
            .world()
            .resource::<bevy::render::renderer::RenderAdapterInfo>();
        let adapter = serde_json::json!({"name":info.name,"backend":format!("{:?}",info.backend),"driver":info.driver,"driver_info":info.driver_info});
        assert!(!app.world().resource::<Scene>().ready());
        let start = Instant::now();
        loop {
            app.update();
            if app.world().resource::<Scene>().ready() {
                break;
            }
            assert!(
                start.elapsed().as_secs() < 90,
                "Authored GPU readiness timed out"
            );
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        assert_eq!(
            app.world_mut().query::<&Window>().iter(app.world()).count(),
            0
        );
        let meshes = app
            .world_mut()
            .query_filtered::<Entity, With<AuthoredMesh>>()
            .iter(app.world())
            .count();
        assert!(meshes >= 8);
        assert!(app.world().resource::<Scene>().diagnostics.pipelines >= meshes);
        // Startup must prepare offscreen geometry too. Looking at a previously
        // unseen part of the same retained scene must not revoke chunk readiness
        // and make every contact disappear. Ordinary culling resumes afterwards.
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<PreparingPipeline>>()
                .iter(app.world())
                .count(),
            0
        );
        let initial_camera = *app.world().get::<Transform>(camera).unwrap();
        for quadrant in 0..4 {
            let yaw = quadrant as f32 * std::f32::consts::FRAC_PI_2;
            let eye = Vec3::new(f32::cos(yaw) * 30., 3., f32::sin(yaw) * 30.);
            *app.world_mut().get_mut::<Transform>(camera).unwrap() =
                Transform::from_translation(eye).looking_at(Vec3::Y, Vec3::Y);
            for _ in 0..8 {
                app.update();
                assert!(
                    app.world().resource::<Scene>().ready(),
                    "Turning toward prepared world geometry retracted readiness"
                );
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        }
        *app.world_mut().get_mut::<Transform>(camera).unwrap() = initial_camera;
        assert_eq!(app.world().resource::<Scene>().diagnostics.retractions, 0);
        println!(
            "{}",
            serde_json::json!({"adapter":adapter,"authored_meshes":meshes,"movement_ready":true,"seconds":start.elapsed().as_secs_f64(),"scope":"Actual offscreen GPU scene/assets/pipeline readiness; no window, OS input or connected gameplay"})
        );
        if let Some(path) = std::env::var_os("THEIA_AUTHORED_CAPTURE") {
            #[derive(Resource, Default)]
            struct Captured(bool);
            app.init_resource::<Captured>();
            app.world_mut()
                .spawn(bevy::render::view::screenshot::Screenshot::image(image))
                .observe(bevy::render::view::screenshot::save_to_disk(
                    std::path::PathBuf::from(path),
                ))
                .observe(
                    |_: On<bevy::render::view::screenshot::ScreenshotCaptured>,
                     mut captured: ResMut<Captured>| {
                        captured.0 = true;
                    },
                );
            let start = Instant::now();
            while !app.world().resource::<Captured>().0 {
                app.update();
                assert!(
                    start.elapsed().as_secs() < 30,
                    "Offscreen capture timed out"
                );
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        }
        // An absent GPU mesh must retract readiness even though scene entities
        // remain. Reintroducing that exact CPU asset must prepare it again.
        let mesh_id = app
            .world_mut()
            .query_filtered::<&Mesh3d, With<AuthoredMesh>>()
            .iter(app.world())
            .next()
            .unwrap()
            .id();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .remove(mesh_id)
            .unwrap();
        for _ in 0..8 {
            app.update();
        }
        assert!(
            !app.world().resource::<Scene>().ready(),
            "Removed mesh retained movement readiness"
        );
        app.world_mut()
            .resource_mut::<Assets<Mesh>>()
            .insert(mesh_id, mesh)
            .unwrap();
        let mut shifted = origin.metres();
        shifted[0] += 128.;
        shifted[2] -= 128.;
        app.world_mut()
            .resource_mut::<crate::floating_origin::Frame>()
            .render = theia_core::coordinates::LocalFrame::new(
            theia_core::coordinates::GlobalPosition::new(shifted).unwrap(),
        );
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation += Vec3::new(-128., 0., 128.);
        let start = Instant::now();
        loop {
            app.update();
            if app.world().resource::<Scene>().ready() {
                break;
            }
            assert!(
                start.elapsed().as_secs() < 30,
                "Restored asset never regained readiness"
            );
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        let root = app.world().resource::<Scene>().root.unwrap();
        assert_eq!(
            app.world().get::<Transform>(root).unwrap().translation,
            Vec3::new(-128., 0., 128.)
        );
        println!(
            "authored_rebase=true removed_mesh_withholds_readiness=true restored_mesh_ready=true"
        );
    }
}
