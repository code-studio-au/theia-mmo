//! Opt-in bounded imported terrain; capsules remain presentation of server poses.
use bevy::{asset::LoadState, prelude::*, world_serialization::WorldAssetRoot};
use std::{path::PathBuf, time::Instant};
use theia_network::terrain::{Terrain, sha256};

#[derive(Resource)]
pub struct Config {
    pub directory: PathBuf,
    pub terrain: Terrain,
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
        if sha256(&std::fs::read(directory.join("terrain.glb"))?) != terrain.visual_sha256 {
            return Err("Terrain visual does not match collision profile".into());
        }
        args.drain(..2);
        Ok(Some(Self { directory, terrain }))
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
    (meshes, ready): (Query<&Mesh3d>, Option<Res<Ready>>),
    mut pipelines: ResMut<crate::capture_ready::CaptureReady>,
    mut exit: MessageWriter<AppExit>,
) {
    if ready.is_some() {
        return;
    }
    if matches!(server.load_state(loading.asset.id()), LoadState::Failed(_))
        || loading.started.elapsed().as_secs() > 60
    {
        error!("Terrain scene dependency/pipeline loading failed or timed out");
        exit.write(AppExit::error());
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
        commands.spawn((WorldAssetRoot(scene), Transform::default()));
        loading.spawned = true;
        pipelines.reset();
    } else if meshes.iter().count() > 3 && pipelines.ready() {
        info!("Terrain walking scene ready; collision and visuals share the qualified fixture");
        commands.insert_resource(Ready);
    }
}
