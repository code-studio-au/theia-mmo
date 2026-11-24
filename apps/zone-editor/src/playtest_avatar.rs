//! Optional standard glTF player art in the offline movement sandbox.
//! Animation previews do not issue abilities or alter the collision capsule.
use super::*;
use bevy::{
    animation::RepeatAnimation,
    asset::{
        AssetApp, LoadState,
        io::{AssetSourceBuilder, file::FileAssetReader},
    },
    camera::primitives::Aabb,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use std::{collections::BTreeMap, path::Path, time::Instant};
#[path = "../../common/locomotion_direction.rs"]
mod direction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Preview {
    #[default]
    Auto,
    Idle,
    Walk,
    Run,
    SwordIdle,
    SwordAttack,
    SpellCast,
    Death,
}
impl Preview {
    pub(super) fn clip(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::Idle => Some("Idle"),
            Self::Walk => Some("Walk"),
            Self::Run => Some("Run"),
            Self::SwordIdle => Some("SwordIdle"),
            Self::SwordAttack => Some("SwordAttack"),
            Self::SpellCast => Some("SpellCast"),
            Self::Death => Some("Death"),
        }
    }
}
#[derive(Resource)]
struct Avatar {
    model: Option<Handle<Gltf>>,
    catalog: Option<Catalog>,
    root: Option<Entity>,
    active: Option<String>,
    previous_direction: Option<u8>,
    started: Instant,
    error: Option<String>,
}
struct Catalog {
    scene: Handle<WorldAsset>,
    graph: Handle<AnimationGraph>,
    clips: BTreeMap<String, (AnimationNodeIndex, f32)>,
}
#[derive(Component)]
struct PlayerRoot;

pub(super) fn install(app: &mut App, directory: &Path) -> Result<(), String> {
    let directory = directory
        .canonicalize()
        .map_err(|e| format!("Player assets: {e}"))?;
    let path = directory.join("character.glb");
    let size = path
        .metadata()
        .map_err(|e| format!("Player character.glb: {e}"))?
        .len();
    if !(20..=16 * 1024 * 1024).contains(&size) {
        return Err("Player GLB must be at most 16 MiB".into());
    }
    app.register_asset_source(
        "playtest-player",
        AssetSourceBuilder::new(move || Box::new(FileAssetReader::new(directory.clone()))),
    );
    app.insert_resource(Avatar {
        model: None,
        catalog: None,
        root: None,
        active: None,
        previous_direction: None,
        started: Instant::now(),
        error: None,
    })
    .add_systems(Update, tick.after(viewport::sync));
    Ok(())
}
fn catalog(world: &mut World, avatar: &mut Avatar) -> Result<bool, String> {
    let server = world.resource::<AssetServer>();
    let handle = avatar
        .model
        .get_or_insert_with(|| server.load("playtest-player://character.glb"));
    if matches!(server.load_state(handle.id()), LoadState::Failed(_))
        || avatar.started.elapsed().as_secs() > 60
    {
        return Err("Player model failed to load; capsule retained".into());
    }
    if !server.is_loaded_with_dependencies(handle.id()) {
        return Ok(false);
    }
    let gltf = world
        .resource::<Assets<Gltf>>()
        .get(handle)
        .ok_or("Missing loaded player model")?;
    let scene = gltf
        .default_scene
        .clone()
        .ok_or("Player model needs a default scene")?;
    if gltf.named_animations.len() > 128 || gltf.skins.is_empty() {
        return Err("Player model requires a skin and at most 128 named clips".into());
    }
    for name in ["Idle", "Walk", "Run", "Jump"] {
        if !gltf.named_animations.contains_key(name) {
            return Err(format!("Player model lacks {name}"));
        }
    }
    let mut clips: Vec<_> = gltf
        .named_animations
        .iter()
        .map(|(name, h)| (name.to_string(), h.clone()))
        .collect();
    clips.sort_by(|a, b| a.0.cmp(&b.0));
    let mut durations = Vec::new();
    for (_, handle) in &clips {
        let duration = world
            .resource::<Assets<AnimationClip>>()
            .get(handle)
            .ok_or("Missing player clip")?
            .duration();
        if !duration.is_finite() || !(0.001..=60.).contains(&duration) {
            return Err("Invalid player clip duration".into());
        }
        durations.push(duration);
    }
    let (graph, nodes) = AnimationGraph::from_clips(clips.iter().map(|(_, h)| h.clone()));
    let graph = world.resource_mut::<Assets<AnimationGraph>>().add(graph);
    avatar.catalog = Some(Catalog {
        scene,
        graph,
        clips: clips
            .into_iter()
            .zip(nodes)
            .zip(durations)
            .map(|(((name, _), node), duration)| (name, (node, duration)))
            .collect(),
    });
    Ok(true)
}
fn descendants(world: &World, root: Entity) -> Vec<Entity> {
    let mut stack = vec![root];
    let mut result = Vec::new();
    while let Some(entity) = stack.pop() {
        result.push(entity);
        if let Some(children) = world.get::<Children>(entity) {
            stack.extend(children.iter());
        }
    }
    result
}
fn automatic(
    play: &playtest::Playtest,
    heading: f32,
    old: Option<u8>,
) -> (&'static str, Option<u8>) {
    if play.airborne() {
        return ("Jump", None);
    }
    let speed = play.velocity.x.hypot(play.velocity.z);
    if speed <= 0.1 {
        return ("Idle", None);
    }
    let local = Quat::from_rotation_y(-heading) * play.velocity;
    let sector = direction::sector(local.x.atan2(-local.z), old);
    let speeds = theia_network::classic::ground_speeds();
    let run = speed > (speeds.walk_metres_per_second + speeds.backward_run_metres_per_second) / 2.;
    let names = if run {
        [
            "Run",
            "RunForwardRight",
            "RunRight",
            "RunBackRight",
            "RunBack",
            "RunBackLeft",
            "RunLeft",
            "RunForwardLeft",
        ]
    } else {
        [
            "Walk",
            "WalkForwardRight",
            "WalkRight",
            "WalkBackRight",
            "WalkBack",
            "WalkBackLeft",
            "WalkLeft",
            "WalkForwardLeft",
        ]
    };
    (names[usize::from(sector)], Some(sector))
}
fn tick(world: &mut World) {
    world.resource_scope(|world, mut avatar: Mut<Avatar>| {
        if avatar.error.is_some() {
            return;
        }
        if avatar.catalog.is_none() {
            match catalog(world, &mut avatar) {
                Ok(false) => return,
                Ok(true) => (),
                Err(error) => {
                    world.resource_mut::<Session>().status = error.clone();
                    avatar.error = Some(error);
                    return;
                }
            }
        }
        let catalog = avatar.catalog.as_ref().unwrap();
        if world.resource::<Session>().player_clips.is_empty() {
            world.resource_mut::<Session>().player_clips = catalog.clips.keys().cloned().collect();
        }
        if avatar.root.is_none() {
            let scene = catalog.scene.clone();
            avatar.root = Some(
                world
                    .spawn((
                        PlayerRoot,
                        WorldAssetRoot(scene),
                        Transform::default(),
                        Visibility::Hidden,
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id(),
            );
        }
        let root = avatar.root.unwrap();
        let session = world.resource::<Session>();
        let Some(play) = session.play.as_ref().filter(|_| session.player_model) else {
            *world.get_mut::<Visibility>(root).unwrap() = Visibility::Hidden;
            avatar.active = None;
            avatar.previous_direction = None;
            return;
        };
        let position = Vec3::from_array(play.feet.metres());
        let heading = session.heading;
        let (name, sector) = session.player_animation.clip().map_or_else(
            || automatic(play, heading, avatar.previous_direction),
            |name| (name, None),
        );
        let catalog = avatar.catalog.as_ref().unwrap();
        let fallback = if name.starts_with("Run") {
            "Run"
        } else if name.starts_with("Walk") {
            "Walk"
        } else {
            "Idle"
        };
        let name = if catalog.clips.contains_key(name) {
            name
        } else {
            fallback
        };
        let (node, duration) = catalog.clips[name];
        let graph = catalog.graph.clone();
        let players: Vec<_> = descendants(world, root)
            .into_iter()
            .filter(|e| world.get::<AnimationPlayer>(*e).is_some())
            .collect();
        if players.is_empty() {
            if avatar.started.elapsed().as_secs() > 60 {
                let error = "Player scene has no animation player; capsule retained".to_owned();
                world.resource_mut::<Session>().status = error.clone();
                avatar.error = Some(error);
            }
            return;
        }
        if avatar.active.is_none() {
            // Same conservative bounds as the qualified game character. Keep
            // ordinary culling while accommodating the source sword poses.
            for entity in descendants(world, root) {
                if world.get::<Mesh3d>(entity).is_some() {
                    world.entity_mut(entity).insert(Aabb::from_min_max(
                        Vec3::new(-1., -0.25, -1.),
                        Vec3::new(1., 2.1, 2.1),
                    ));
                }
            }
        }
        let changed = avatar.active.as_deref() != Some(name);
        let moving = |clip: &str| clip.starts_with("Walk") || clip.starts_with("Run");
        let previous = avatar
            .active
            .as_deref()
            .filter(|old| moving(old) && moving(name))
            .and_then(|old| catalog.clips.get(old))
            .copied();
        for entity in players {
            if changed {
                let mut player = world.get_mut::<AnimationPlayer>(entity).unwrap();
                let phase = previous.and_then(|(old, old_duration)| {
                    player
                        .animation(old)
                        .and_then(|a| direction::phase(a.seek_time(), old_duration, duration))
                });
                player.stop_all();
                let active = player.start(node);
                if name == "Death" {
                    active.set_repeat(RepeatAnimation::Never);
                } else {
                    active.repeat();
                }
                if let Some(time) = phase {
                    active.seek_to(time);
                }
                // Use the source timing; animation never changes capsule movement speed.
                debug_assert!(duration > 0.);
            }
            world
                .entity_mut(entity)
                .insert(AnimationGraphHandle(graph.clone()));
        }
        avatar.active = Some(name.into());
        avatar.previous_direction = sector;
        *world.get_mut::<Transform>(root).unwrap() = Transform::from_translation(position)
            .with_rotation(Quat::from_rotation_y(heading + std::f32::consts::PI));
        *world.get_mut::<Visibility>(root).unwrap() = Visibility::Visible;
        let pawns: Vec<_> = world
            .query_filtered::<Entity, With<viewport::Pawn>>()
            .iter(world)
            .collect();
        for pawn in pawns {
            *world.get_mut::<Visibility>(pawn).unwrap() = Visibility::Hidden;
        }
    });
}

#[cfg(test)]
#[path = "playtest_avatar_tests.rs"]
mod tests;
