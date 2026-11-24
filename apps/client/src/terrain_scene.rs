//! Opt-in bounded imported terrain; capsules remain presentation of server poses.
use bevy::{asset::LoadState, prelude::*, world_serialization::WorldAssetRoot};
use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};
use theia_network::terrain::Terrain;

#[derive(Resource)]
pub struct Config {
    pub directory: PathBuf,
    pub terrain: Terrain,
    pub particles: Option<theia_vfx::PreparedCatalog>,
}
impl Config {
    pub fn parse(args: &mut Vec<String>) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        if args.first().is_none_or(|a| a != "--terrain-world") {
            return Ok(None);
        }
        if args.len() < 2 {
            return Err("--terrain-world needs a directory".into());
        }
        let directory = std::fs::canonicalize(&args[1])?;
        let terrain = Terrain::load(&directory.join("terrain.json")).map_err(|e| e.to_string())?;
        if !terrain.has_spawns() {
            return Err(
                "World requires a validated spawn; use chunk packages through residency".into(),
            );
        }
        let particles = if let Some(manifest) = &terrain.package
            && manifest.has_particle_catalog()
        {
            let file = std::fs::File::open(directory.join("particles/catalog.json"))?;
            let mut bytes = Vec::new();
            file.take(theia_network::particle_content::MAX_CATALOG_BYTES + 1)
                .read_to_end(&mut bytes)?;
            let catalog = manifest
                .verify_particle_catalog(&bytes)
                .map_err(|e| e.to_string())?;
            manifest
                .verify_visual_for_particle_catalog(&directory.join("terrain.glb"), &bytes)
                .map_err(|e| e.to_string())?;
            let origin = theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap();
            let frame = theia_core::coordinates::LocalFrame::new(origin);
            let prepared = theia_vfx::prepare_catalog_with_image_limit(
                &catalog,
                &directory.join("particles/effects"),
                &directory.join("particles/textures"),
                Path::new("particles"),
                frame,
                frame,
                crate::particle_world::LIMITS,
                crate::particle_world::MAX_IMAGE_BYTES,
            )?;
            let image_bytes = prepared
                .textures
                .iter()
                .try_fold(0usize, |sum, texture| {
                    sum.checked_add(texture.image.data.as_ref().map_or(0, Vec::len))
                })
                .ok_or("Particle decoded image size overflow")?;
            if image_bytes > crate::particle_world::MAX_IMAGE_BYTES {
                return Err("Particle decoded images exceed client budget".into());
            }
            Some(prepared)
        } else {
            terrain
                .verify_visual(&directory.join("terrain.glb"))
                .map_err(|e| e.to_string())?;
            None
        };
        args.drain(..2);
        Ok(Some(Self {
            directory,
            terrain: terrain.into_world(),
            particles,
        }))
    }
}
#[derive(Resource)]
pub struct Loading {
    asset: Handle<Gltf>,
    spawned: bool,
    started: Instant,
}
#[derive(Resource)]
pub struct Ready;
#[derive(bevy::ecs::system::SystemParam)]
pub struct SceneReadiness<'w, 's> {
    meshes: Query<'w, 's, &'static Mesh3d>,
    ready: Option<Res<'w, Ready>>,
    roots: Query<'w, 's, Entity, With<SceneryRoot>>,
    materials: crate::runtime_materials::Readiness<'w, 's>,
}

/// Static imported scenery only; animated actors never receive this marker.
#[derive(Component)]
pub struct CameraScenery;

#[derive(Component)]
pub struct SceneryRoot;

pub fn mark_camera_scenery(
    mut commands: Commands,
    roots: Query<Entity, With<SceneryRoot>>,
    children: Query<&Children>,
    meshes: Query<(), (With<Mesh3d>, Without<CameraScenery>)>,
) {
    for root in &roots {
        for entity in children.iter_descendants(root) {
            if meshes.contains(entity) {
                commands.entity(entity).insert((
                    CameraScenery,
                    bevy::picking::mesh_picking::ray_cast::RayCastBackfaces,
                ));
            }
        }
    }
}

pub fn load(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Loading {
        asset: server.load("terrain.glb"),
        spawned: false,
        started: Instant::now(),
    });
}
pub fn tick(
    mut commands: Commands,
    mut loading: ResMut<Loading>,
    server: Res<AssetServer>,
    assets: Res<Assets<Gltf>>,
    view: SceneReadiness,
    mut pipelines: ResMut<crate::capture_ready::CaptureReady>,
    mut exit: (
        MessageWriter<AppExit>,
        Option<ResMut<crate::loading::State>>,
    ),
) {
    if view.ready.is_some()
        || exit
            .1
            .as_ref()
            .is_some_and(|s| s.phase == crate::loading::Phase::Failed)
    {
        return;
    }
    if matches!(server.load_state(loading.asset.id()), LoadState::Failed(_))
        || loading.started.elapsed().as_secs() > 120
    {
        error!("Terrain scene dependency/pipeline loading failed or timed out");
        crate::loading::exit_unless_presented(
            &mut exit.0,
            exit.1.as_deref_mut(),
            "World assets could not be prepared.",
        );
        return;
    }
    if !server.is_loaded_with_dependencies(loading.asset.id()) {
        return;
    }
    if !loading.spawned {
        let gltf = assets.get(&loading.asset).expect("Loaded terrain GLB");
        let scene = gltf
            .default_scene
            .clone()
            .or_else(|| gltf.scenes.first().cloned())
            .expect("Terrain scene");
        commands.spawn((
            SceneryRoot,
            crate::floating_origin::Anchored::ZERO,
            WorldAssetRoot(scene),
            Transform::default(),
        ));
        loading.spawned = true;
        pipelines.reset();
    } else if view.meshes.iter().count() > 3 && pipelines.ready() {
        for root in &view.roots {
            match view.materials.ready(root) {
                Ok(true) => {}
                Ok(false) => return,
                Err(error) => {
                    error!("Terrain material validation failed: {error}");
                    crate::loading::exit_unless_presented(
                        &mut exit.0,
                        exit.1.as_deref_mut(),
                        "World assets could not be prepared.",
                    );
                    return;
                }
            }
        }
        info!("Terrain scene ready; shared world collision and matching visuals loaded");
        commands.insert_resource(Ready);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_markers_follow_only_the_imported_static_hierarchy() {
        let mut app = App::new();
        app.add_systems(Update, mark_camera_scenery);
        let root = app.world_mut().spawn(SceneryRoot).id();
        let actor = app.world_mut().spawn(Mesh3d::default()).id();
        app.update();
        // Scene children can arrive after the initial loading frame.
        let child = app.world_mut().spawn(ChildOf(root)).id();
        let mesh = app
            .world_mut()
            .spawn((Mesh3d::default(), ChildOf(child)))
            .id();
        app.update();
        assert!(app.world().get::<CameraScenery>(mesh).is_some());
        assert!(
            app.world()
                .get::<bevy::picking::mesh_picking::ray_cast::RayCastBackfaces>(mesh)
                .is_some()
        );
        assert!(app.world().get::<CameraScenery>(actor).is_none());
    }
}
