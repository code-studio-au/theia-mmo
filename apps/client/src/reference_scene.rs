//! Opt-in local compatibility art. No imported model determines server geometry.
use crate::movement::{Gait, LocalPawn, Motion};
use bevy::{
    asset::LoadState, camera::primitives::Aabb, prelude::*, world_serialization::WorldAsset,
};
use std::{path::PathBuf, time::Instant};
#[path = "player_attack.rs"]
pub(crate) mod attack;
#[path = "player_grip.rs"]
pub(crate) mod grip;
#[path = "player_locomotion.rs"]
pub(crate) mod locomotion;

#[derive(Resource)]
pub struct Config {
    pub directory: PathBuf,
    pub capture: Option<PathBuf>,
    pub players_only: bool,
    pub polygon: bool,
}
impl Config {
    pub fn parse(args: &mut Vec<String>) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let players_only = args.first().is_some_and(|s| s == "--player-assets");
        if !players_only && args.first().is_none_or(|s| s != "--reference-scene") {
            return Ok(None);
        }
        if args.len() < 2 {
            return Err("Reference/player assets need an asset directory".into());
        }
        let directory = std::fs::canonicalize(&args[1])?;
        let polygon = players_only && directory.join("character.glb").is_file();
        let files = if polygon {
            &["character.glb"][..]
        } else if players_only {
            &["human.glb"][..]
        } else {
            &["human.glb", "wolf.glb", "house.glb"][..]
        };
        for file in files {
            if !directory.join(file).is_file() {
                return Err(format!("Missing reference asset: {file}").into());
            }
        }
        args.drain(..2);
        let capture = if !players_only && args.first().is_some_and(|a| a == "--reference-smoke") {
            if args.len() != 2 {
                return Err("--reference-smoke <output> is offline only".into());
            }
            let output = PathBuf::from(&args[1]);
            std::fs::create_dir_all(&output)?;
            for file in ["reference.json", "scene.png"] {
                std::fs::remove_file(output.join(file)).or_else(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(e)
                    }
                })?;
            }
            args.clear();
            Some(output)
        } else {
            None
        };
        Ok(Some(Self {
            directory,
            capture,
            players_only,
            polygon,
        }))
    }
}

#[derive(Resource)]
struct Loading {
    models: Vec<Handle<Gltf>>,
    started: Instant,
}
#[derive(Resource)]
struct Catalog {
    scenes: Vec<Handle<WorldAsset>>,
    graphs: Vec<Handle<AnimationGraph>>,
    indices: Vec<Vec<AnimationNodeIndex>>,
    durations: Vec<Vec<f32>>,
    optional_slots: [Option<usize>; 2],
    attack_slot: Option<usize>,
    dodge_slots: [Option<usize>; 4],
    dodge_durations: [f32; 4],
    locomotion_slots: [Option<usize>; locomotion::COUNT],
    grip: Option<grip::Graph>,
}
impl Catalog {
    fn slot(&self, pose: usize) -> usize {
        if pose < 4 {
            pose
        } else if pose < 6 {
            self.optional_slots[pose - 4].expect("Selected optional clip must be available")
        } else if pose < locomotion::FIRST {
            self.dodge_slots[pose - 6].expect("Selected dodge clip must be available")
        } else {
            self.locomotion_slots[pose - locomotion::FIRST]
                .expect("Selected direction must be available")
        }
    }
    fn index(&self, kind: usize, pose: usize) -> AnimationNodeIndex {
        self.indices[kind][self.slot(pose)]
    }
    fn duration(&self, kind: usize, pose: usize) -> f32 {
        self.durations[kind][self.slot(pose)]
    }
}
fn optional_slots(names: &[&str]) -> [Option<usize>; 2] {
    ["Death", "SwordIdle"].map(|name| names.iter().position(|candidate| *candidate == name))
}
#[derive(Component)]
pub struct Wolf;
#[derive(Component)]
struct Attached;
#[derive(Resource)]
pub struct Ready;
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct LifeAnimationMetrics {
    pub available: bool,
    pub death_starts: u64,
    pub remote_death_starts: u64,
    pub remote_dead_frames: u64,
    pub remote_life_restarts: u64,
    pub maximum_remote_death_pose_changes: usize,
    pub life_restarts: u64,
    pub dead_frames: u64,
    pub maximum_death_pose_changes: usize,
    pub maximum_death_elapsed: f32,
    pub death_finished_frames: u64,
    pub sword_stance_available: bool,
    pub sword_stance_local_starts: u64,
    pub sword_stance_remote_starts: u64,
    pub sword_stance_local_exits: u64,
    pub sword_stance_remote_exits: u64,
    pub sword_stance_local_frames: u64,
    pub sword_stance_remote_frames: u64,
    pub maximum_sword_pose_changes: usize,
}
#[derive(Component)]
pub struct Visual {
    pub owner: Entity,
    pub kind: usize,
    pub players: Vec<Entity>,
    pub gait: Option<Gait>,
    clip: Option<usize>,
    pub pose_changes: [usize; 3],
    samples: [Vec<(Entity, Transform)>; locomotion::POSES],
    life_key: Option<theia_core::player_life::LifeKey>,
    grip_active: bool,
    layer_frames: u8,
    attack_sequence: Option<u64>,
    dodge_key: Option<crate::dodge::animation::Key>,
}
impl Visual {
    pub(crate) fn sword_stance_active(&self) -> bool {
        self.clip == Some(5) && self.attack_sequence.is_none()
    }
    fn observe_life(&mut self, key: theia_core::player_life::LifeKey) -> bool {
        let changed = self.life_key.is_some_and(|old| old != key);
        if changed {
            self.clip = None;
            self.grip_active = false;
            self.layer_frames = 0;
            self.attack_sequence = None;
            self.dodge_key = None;
            self.samples = std::array::from_fn(|_| Vec::new());
        }
        self.life_key = Some(key);
        changed
    }
}
/// Owner presentation stays on the private life barrier. Remote bodies use only
/// the public visual identity/dead flag, with no access to another actor's HP.
fn presentation_life(
    state: Option<&theia_network::client::State>,
    actor: Option<theia_core::identity::EntityKey>,
    local: bool,
) -> Option<(theia_core::player_life::LifeKey, bool)> {
    let state = state?;
    let actor = actor?;
    if local {
        return state
            .player_life
            .as_ref()?
            .current()
            .filter(|p| p.key.actor == actor)
            .map(|p| (p.key, p.health == 0));
    }
    state
        .public_avatar_life
        .as_ref()?
        .actor(actor)
        .map(|p| (p.key, p.dead))
}

fn select_clip(
    polygon: bool,
    death_available: bool,
    dead: bool,
    airborne: bool,
    gait: Gait,
    sword_stance: bool,
) -> usize {
    if death_available && dead {
        4
    } else if polygon && airborne {
        3
    } else if polygon && sword_stance && !dead && gait == Gait::Stand {
        5
    } else {
        gait.index()
    }
}
#[derive(Component)]
pub struct House;

pub fn load(mut commands: Commands, server: Res<AssetServer>, config: Res<Config>) {
    commands.init_resource::<LifeAnimationMetrics>();
    commands.init_resource::<locomotion::Metrics>();
    commands.insert_resource(grip::Metrics {
        capture: std::env::var_os("THEIA_EQUIPMENT_QUALIFY").is_some()
            || std::env::var_os("THEIA_DODGE_ANIMATION_QUALIFY").is_some()
            || std::env::var_os("THEIA_DIRECTIONAL_LOCOMOTION_QUALIFY").is_some(),
        ..default()
    });
    commands.insert_resource(Loading {
        models: if config.players_only {
            vec![server.load(if config.polygon {
                "players://character.glb"
            } else {
                "players://human.glb"
            })]
        } else {
            vec![
                server.load("human.glb"),
                server.load("wolf.glb"),
                server.load("house.glb"),
            ]
        },
        started: Instant::now(),
    });
}

fn fail(world: &mut World, message: &str) {
    error!("Reference scene failed: {message}");
    world.write_message(AppExit::error());
}

fn catalog(world: &mut World) -> Result<Option<Catalog>, String> {
    let loading = world.resource::<Loading>();
    let server = world.resource::<AssetServer>();
    for model in &loading.models {
        if matches!(server.load_state(model.id()), LoadState::Failed(_))
            || !server.is_loaded_with_dependencies(model.id())
                && loading.started.elapsed().as_secs() > 60
        {
            return Err("Asset/dependency loading failed or timed out".into());
        }
        if !server.is_loaded_with_dependencies(model.id()) {
            return Ok(None);
        }
    }
    let mut scenes = Vec::new();
    let mut clips = Vec::new();
    let mut durations = Vec::new();
    let mut locomotion_slots = [None; locomotion::COUNT];
    let mut player_optional = [None; 2];
    let mut attack_slot = None;
    let mut dodge_slots = [None; 4];
    let mut dodge_durations = [0.; 4];
    for (i, handle) in loading.models.iter().enumerate() {
        let gltf = world
            .resource::<Assets<Gltf>>()
            .get(handle)
            .ok_or("Missing loaded GLB")?;
        scenes.push(
            gltf.default_scene
                .clone()
                .or_else(|| gltf.scenes.first().cloned())
                .ok_or("Missing GLB scene")?,
        );
        if i < 2 {
            let mut names = if i == 0 && world.resource::<Config>().polygon {
                &["Idle", "Walk", "Run", "Jump"][..]
            } else if i == 0 {
                &[
                    "Stand (ID 0 variation 0)",
                    "Walk (ID 4 variation 0)",
                    "Run (ID 5 variation 0)",
                ][..]
            } else {
                &["Stand (ID 0 variation 0)"][..]
            }
            .to_vec();
            if i == 0 && world.resource::<Config>().polygon {
                for name in ["Death", "SwordIdle", "SwordAttack"] {
                    if gltf.named_animations.contains_key(name) {
                        names.push(name);
                    }
                }
                player_optional = optional_slots(&names);
                attack_slot = names.iter().position(|name| *name == "SwordAttack");
                let dodge_count = crate::dodge::animation::NAMES
                    .iter()
                    .filter(|name| gltf.named_animations.contains_key(**name))
                    .count();
                if !matches!(dodge_count, 0 | 4) {
                    return Err("Directional dodge art requires all four inspected clips".into());
                }
                if dodge_count == 4 {
                    for (slot, name) in crate::dodge::animation::NAMES.into_iter().enumerate() {
                        dodge_slots[slot] = Some(names.len());
                        names.push(name);
                        let handle = &gltf.named_animations[name];
                        let duration = world
                            .resource::<Assets<AnimationClip>>()
                            .get(handle)
                            .ok_or("Missing dodge animation asset")?
                            .duration();
                        if !duration.is_finite() || (duration - 17. / 30.).abs() > 0.01 {
                            return Err(
                                "Dodge animation duration differs from inspected short clips"
                                    .into(),
                            );
                        }
                        dodge_durations[slot] = duration;
                    }
                }
                locomotion_slots = locomotion::append_slots(&mut names, |name| {
                    gltf.named_animations.contains_key(name)
                })?;
            }
            let mut selected = Vec::new();
            let mut selected_durations = Vec::new();
            for name in names {
                let clip = gltf
                    .named_animations
                    .get(name)
                    .ok_or_else(|| format!("Missing clip {name}"))?
                    .clone();
                let duration = world
                    .resource::<Assets<AnimationClip>>()
                    .get(&clip)
                    .map(|c| c.duration())
                    .filter(|d| d.is_finite() && *d > 0.)
                    .ok_or_else(|| format!("Empty clip {name}"))?;
                selected.push(clip);
                selected_durations.push(duration);
            }
            clips.push(selected);
            durations.push(selected_durations);
        }
    }
    let mut graphs = Vec::new();
    let mut indices = Vec::new();
    for clips in clips {
        let (graph, nodes) = AnimationGraph::from_clips(clips);
        graphs.push(world.resource_mut::<Assets<AnimationGraph>>().add(graph));
        indices.push(nodes);
    }
    Ok(Some(Catalog {
        scenes,
        graphs,
        indices,
        durations,
        optional_slots: player_optional,
        attack_slot,
        dodge_slots,
        dodge_durations,
        locomotion_slots,
        grip: None,
    }))
}

pub fn descendants(world: &World, root: Entity) -> Vec<Entity> {
    let mut result = vec![root];
    let mut i = 0;
    while i < result.len() {
        if let Some(children) = world.get::<Children>(result[i]) {
            result.extend(children.iter());
        }
        i += 1;
    }
    result
}

pub fn house_transform() -> Transform {
    // Measured export bounds, fitted uniformly inside the existing solid graybox.
    let low = Vec3::new(-9.562015, -0.14303547, -9.259302);
    let high = Vec3::new(9.714225, 17.085135, 9.171431);
    let size = high - low;
    let scale = (8. / size.x).min(6.8 / size.y).min(8. / size.z);
    Transform::from_xyz(
        -7. - (low.x + high.x) * 0.5 * scale,
        -low.y * scale,
        -7. - (low.z + high.z) * 0.5 * scale,
    )
    .with_scale(Vec3::splat(scale))
}

pub fn tick(world: &mut World) {
    if !world.contains_resource::<Catalog>() {
        match catalog(world) {
            Ok(Some(assets)) => {
                world.resource_mut::<locomotion::Metrics>().available =
                    assets.locomotion_slots.iter().all(Option::is_some);
                if !world.resource::<Config>().players_only {
                    world.spawn((
                        House,
                        crate::floating_origin::Anchored::at(house_transform().translation),
                        WorldAssetRoot(assets.scenes[2].clone()),
                        house_transform(),
                    ));
                    // A nonblocking enemy presentation spawn; AI/combat are separate.
                    world.spawn((
                        Wolf,
                        crate::floating_origin::Anchored::at(Vec3::new(3., 0.9, -4.)),
                        Motion::default(),
                        Transform::from_xyz(3., 0.9, -4.),
                        Visibility::Inherited,
                    ));
                }
                let available = assets.optional_slots[0].is_some();
                world.resource_mut::<LifeAnimationMetrics>().available = available;
                world
                    .resource_mut::<LifeAnimationMetrics>()
                    .sword_stance_available = assets.optional_slots[1].is_some();
                if let Some(mut animation) =
                    world.get_resource_mut::<crate::dodge::animation::State>()
                {
                    animation.available = assets.dodge_slots.iter().all(Option::is_some);
                }
                world.insert_resource(assets);
            }
            Ok(None) => return,
            Err(error) => {
                fail(world, &error);
                return;
            }
        }
    }
    let network_state = world
        .get_resource::<crate::network::Network>()
        .filter(|network| network.connected())
        .map(|network| network.state());

    let actors: Vec<_> = world
        .query_filtered::<Entity, (
            Or<(
                With<LocalPawn>,
                With<crate::network::RemotePawn>,
                With<Wolf>,
            )>,
            Without<Attached>,
        )>()
        .iter(world)
        .collect();
    for actor in actors {
        let kind = usize::from(world.get::<Wolf>(actor).is_some());
        let scene = world.resource::<Catalog>().scenes[kind].clone();
        world
            .entity_mut(actor)
            .remove::<Mesh3d>()
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(Attached);
        world.spawn((
            WorldAssetRoot(scene),
            if world.resource::<Config>().polygon {
                polygon_transform()
            } else {
                actor_transform(kind)
            },
            ChildOf(actor),
            Visual {
                owner: actor,
                kind,
                players: Vec::new(),
                gait: None,
                clip: None,
                pose_changes: [0; 3],
                samples: std::array::from_fn(|_| Vec::new()),
                life_key: None,
                grip_active: false,
                layer_frames: 0,
                attack_sequence: None,
                dodge_key: None,
            },
        ));
    }
    let visuals: Vec<_> = world
        .query_filtered::<Entity, With<Visual>>()
        .iter(world)
        .collect();
    for root in visuals {
        let visual = world.get::<Visual>(root).unwrap();
        if visual.players.is_empty() {
            let players: Vec<_> = descendants(world, root)
                .into_iter()
                .filter(|e| world.get::<AnimationPlayer>(*e).is_some())
                .collect();
            if players.is_empty() {
                if world.resource::<Loading>().started.elapsed().as_secs() > 60 {
                    fail(world, "No animation player instantiated");
                }
                continue;
            }
            if visual.kind == 0
                && (world.resource::<LifeAnimationMetrics>().available
                    || world
                        .resource::<LifeAnimationMetrics>()
                        .sword_stance_available
                    || world.resource::<Catalog>().dodge_slots[0].is_some()
                    || world.resource::<Catalog>().locomotion_slots[0].is_some())
            {
                // Retained body bounds also cover the four inspected short
                // dodges, with preview/interpolation margin and normal culling.
                for e in descendants(world, root) {
                    if world.get::<Mesh3d>(e).is_some() {
                        world.entity_mut(e).insert(Aabb::from_min_max(
                            Vec3::new(-1., -0.25, -1.),
                            Vec3::new(1., 2.1, 2.1),
                        ));
                    }
                }
            }
            let kind = world.get::<Visual>(root).unwrap().kind;
            if kind == 0
                && world.resource::<Catalog>().optional_slots[1].is_some()
                && let Err(error) = grip::prepare(world, root)
            {
                fail(world, &error);
                return;
            }
            let catalog = world.resource::<Catalog>();
            let handle = if kind == 0 {
                catalog.grip.as_ref().map(|g| g.handle.clone())
            } else {
                None
            }
            .unwrap_or_else(|| catalog.graphs[kind].clone());
            for player in &players {
                world
                    .entity_mut(*player)
                    .insert(AnimationGraphHandle(handle.clone()));
            }
            world.get_mut::<Visual>(root).unwrap().players = players;
        }
        let (owner, kind) = {
            let v = world.get::<Visual>(root).unwrap();
            (v.owner, v.kind)
        };
        let actor_key = network_state.as_ref().and_then(|state| {
            use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
            let id = if world.get::<LocalPawn>(owner).is_some() {
                state.id
            } else {
                world
                    .get::<crate::network::RemotePawn>(owner)
                    .map(|pawn| pawn.actor_id())
            };
            id.and_then(EntityId::new)
                .zip(
                    state
                        .welcome
                        .as_ref()
                        .and_then(|w| WorldEpoch::new(w.epoch)),
                )
                .map(|(entity, epoch)| EntityKey { entity, epoch })
        });
        let local = world.get::<LocalPawn>(owner).is_some();
        let life = presentation_life(network_state.as_ref(), actor_key, local);
        let life_changed = if let Some((key, _)) = life {
            let changed = world.get_mut::<Visual>(root).unwrap().observe_life(key);
            let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
            metrics.life_restarts += u64::from(changed);
            metrics.remote_life_restarts += u64::from(changed && !local);
            changed
        } else {
            false
        };
        let dead = life.is_some_and(|(_, dead)| dead);
        world.resource_mut::<LifeAnimationMetrics>().dead_frames += u64::from(dead);
        world
            .resource_mut::<LifeAnimationMetrics>()
            .remote_dead_frames += u64::from(dead && !local);
        let directional = kind == 0
            && world.resource::<locomotion::Metrics>().available
            && network_state
                .as_ref()
                .and_then(|s| s.correction)
                .is_some_and(|c| c.state.pose.facing.is_some());
        let gait = if kind == 0 {
            locomotion::gait(world.get::<Motion>(owner).map_or(0., |m| m.0), directional)
        } else {
            Gait::Stand
        };
        let airborne = world
            .get_resource::<crate::graybox::Graybox>()
            .is_some_and(|g| {
                world.get::<Transform>(owner).is_some_and(|t| {
                    world
                        .resource::<crate::floating_origin::Frame>()
                        .render
                        .to_global(
                            theia_core::coordinates::LocalPosition::new(t.translation.to_array())
                                .unwrap(),
                        )
                        .is_some_and(|p| g.airborne(p))
                })
            });
        if (dead || life_changed)
            && let Some(key) = actor_key
            && let Some(mut cues) = world.get_resource_mut::<attack::Cues>()
        {
            cues.invalidate(key);
        }
        let sword_stance = kind == 0
            && world
                .resource::<LifeAnimationMetrics>()
                .sword_stance_available
            && actor_key.is_some_and(|key| {
                crate::equipment::sword_equipped(
                    network_state.as_ref().and_then(|s| s.appearance.as_ref()),
                    key,
                )
            });
        let attack = if sword_stance
            && !dead
            && world
                .resource::<Catalog>()
                .grip
                .as_ref()
                .is_some_and(|g| g.attack.is_some())
        {
            actor_key.and_then(|key| {
                world
                    .get_resource::<attack::Cues>()
                    .and_then(|c| c.pose(key, Instant::now()))
            })
        } else {
            None
        };
        let mut clip = select_clip(
            world.resource::<Config>().polygon,
            world.resource::<LifeAnimationMetrics>().available,
            dead,
            airborne,
            gait,
            sword_stance,
        );
        if attack.is_some() {
            clip = if airborne { 3 } else { gait.index() };
        }
        if directional && !dead && locomotion::moving(clip) {
            let segment = if local {
                world
                    .get_resource::<crate::network::Network>()
                    .and_then(|n| n.local_motion_segment())
            } else {
                world
                    .get::<crate::network::RemotePawn>(owner)
                    .map(|p| p.global_endpoints())
            };
            if let Some(transform) = world.get::<Transform>(owner) {
                clip = locomotion::choose(
                    gait,
                    segment,
                    transform.rotation,
                    world.get::<Visual>(root).unwrap().clip,
                );
            }
        }
        // Existing critical sword cues keep priority over this local cosmetic pose.
        let dodge = (local && !dead && attack.is_none())
            .then(|| {
                world
                    .get_resource::<crate::dodge::animation::State>()
                    .and_then(|s| s.current)
            })
            .flatten();
        if let Some(cue) = dodge {
            clip = 6 + cue.slot;
        }
        if clip == 5 {
            let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
            metrics.sword_stance_local_frames += u64::from(local);
            metrics.sword_stance_remote_frames += u64::from(!local);
        }
        let grip_active = sword_stance
            && !dead
            && (locomotion::base_name(clip).is_some() || dodge.is_some())
            && attack.is_none();
        if let Some(mut metrics) = world.get_resource_mut::<attack::Metrics>()
            && attack.is_some()
        {
            metrics.local_frames += u64::from(local);
            metrics.remote_frames += u64::from(!local);
            metrics.moving_frames += u64::from(gait != Gait::Stand);
        }
        let visual = world.get::<Visual>(root).unwrap();
        let changed = visual.clip != Some(clip);
        if directional {
            world
                .resource_mut::<locomotion::Metrics>()
                .record(local, clip, changed);
        }
        let visual = world.get::<Visual>(root).unwrap();
        if visual.clip != Some(clip)
            || visual.grip_active != grip_active
            || visual.attack_sequence != attack.map(|a| a.0)
            || visual.dodge_key != dodge.map(|c| c.key)
        {
            let leaving_stance = visual.sword_stance_active();
            let catalog = world.resource::<Catalog>();
            let index = if attack.is_some() {
                catalog
                    .grip
                    .as_ref()
                    .unwrap()
                    .attack
                    .as_ref()
                    .unwrap()
                    .base(clip)
            } else if grip_active {
                catalog.grip.as_ref().unwrap().base(clip)
            } else {
                catalog.index(visual.kind, clip)
            };
            let hold = grip_active.then(|| catalog.grip.as_ref().unwrap().hold);
            let strike = attack.map(|(_, elapsed)| {
                (
                    catalog
                        .grip
                        .as_ref()
                        .unwrap()
                        .attack
                        .as_ref()
                        .unwrap()
                        .strike,
                    elapsed,
                )
            });
            let new_attack = attack.is_some() && visual.attack_sequence != attack.map(|a| a.0);
            let dodge_playback = dodge.map(|cue| {
                let speed = catalog.dodge_durations[cue.slot] / cue.duration;
                (speed, cue.elapsed * speed)
            });
            let players = visual.players.clone();
            let owner = visual.owner;
            let phase = if directional
                && !life_changed
                && visual.grip_active == grip_active
                && visual.attack_sequence == attack.map(|a| a.0)
                && visual.clip.is_some_and(locomotion::moving)
                && locomotion::moving(clip)
            {
                let old = visual.clip.unwrap();
                let old_index = if attack.is_some() {
                    catalog
                        .grip
                        .as_ref()
                        .unwrap()
                        .attack
                        .as_ref()
                        .unwrap()
                        .base(old)
                } else if grip_active {
                    catalog.grip.as_ref().unwrap().base(old)
                } else {
                    catalog.index(kind, old)
                };
                players.iter().find_map(|e| {
                    world
                        .get::<AnimationPlayer>(*e)
                        .and_then(|p| p.animation(old_index))
                        .and_then(|a| {
                            locomotion::phase(
                                a.seek_time(),
                                catalog.duration(kind, old),
                                catalog.duration(kind, clip),
                            )
                        })
                })
            } else {
                None
            };
            if phase.is_some() {
                world
                    .resource_mut::<locomotion::Metrics>()
                    .phase_preservations += 1;
            }
            for player in players {
                // Death plays once and holds its final pose; locomotion loops.
                let mut player = world.get_mut::<AnimationPlayer>(player).unwrap();
                let active = player.stop_all().play(index);
                if clip != 4 && dodge.is_none() {
                    active.repeat();
                }
                if let Some(time) = phase {
                    active.seek_to(time);
                }
                if let Some((speed, elapsed)) = dodge_playback {
                    active.set_speed(speed).seek_to(elapsed);
                }
                if let Some((index, elapsed)) = strike {
                    player.play(index).seek_to(elapsed);
                }
                if let Some(hold) = hold {
                    // Frozen authored closed grip; wrist, prop socket and body
                    // remain on the moving clip through disjoint Bevy masks.
                    player.play(hold).pause();
                }
            }
            if new_attack && let Some(mut metrics) = world.get_resource_mut::<attack::Metrics>() {
                metrics.local_starts += u64::from(local);
                metrics.remote_starts += u64::from(!local);
            }
            world.resource_mut::<LifeAnimationMetrics>().death_starts += u64::from(clip == 4);
            world
                .resource_mut::<LifeAnimationMetrics>()
                .remote_death_starts += u64::from(clip == 4 && !local);
            if leaving_stance {
                let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
                metrics.sword_stance_local_exits += u64::from(local);
                metrics.sword_stance_remote_exits += u64::from(!local);
            }
            if clip == 5 {
                let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
                metrics.sword_stance_local_starts += u64::from(local);
                metrics.sword_stance_remote_starts += u64::from(!local);
            }
            info!("Reference actor {:?}: {:?}", owner, gait);
            let mut visual = world.get_mut::<Visual>(root).unwrap();
            visual.gait = Some(gait);
            visual.clip = Some(clip);
            visual.grip_active = grip_active;
            visual.attack_sequence = attack.map(|a| a.0);
            visual.dodge_key = dodge.map(|c| c.key);
            visual.layer_frames = 0;
        }
        if dead && clip == 4 {
            let visual = world.get::<Visual>(root).unwrap();
            let index = world.resource::<Catalog>().index(visual.kind, clip);
            let (elapsed, finished) =
                visual
                    .players
                    .iter()
                    .fold((0_f32, false), |(elapsed, finished), e| {
                        world
                            .get::<AnimationPlayer>(*e)
                            .and_then(|p| p.animation(index))
                            .map_or((elapsed, finished), |a| {
                                (elapsed.max(a.elapsed()), finished || a.is_finished())
                            })
                    });
            let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
            metrics.maximum_death_elapsed = metrics.maximum_death_elapsed.max(elapsed);
            metrics.death_finished_frames += u64::from(finished);
        }
    }
    if world.contains_resource::<crate::graybox::Graybox>()
        || world
            .get_resource::<crate::network::Network>()
            .is_some_and(|n| n.smoke.is_some())
    {
        sample_poses(world);
    }
    let house = world
        .query_filtered::<Entity, With<House>>()
        .iter(world)
        .next();
    let house_ready = house.is_some_and(|h| {
        descendants(world, h)
            .iter()
            .any(|e| world.get::<Mesh3d>(*e).is_some())
    });
    let players_only = world.resource::<Config>().players_only;
    if (house_ready || players_only)
        && world
            .query::<&Visual>()
            .iter(world)
            .filter(|v| !v.players.is_empty())
            .count()
            >= if players_only { 1 } else { 2 }
    {
        world.insert_resource(Ready);
    }
}

fn polygon_transform() -> Transform {
    // Source bind-pose toes extend along +Z; gameplay forward is -Z.
    Transform::from_xyz(0., -theia_network::world::PAWN_Y, 0.)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
}

fn actor_transform(kind: usize) -> Transform {
    let scale = if kind == 0 { 1.8 / 2.1322405 } else { 0.5 };
    let floor_offset = if kind == 0 {
        0.01880305 * scale
    } else {
        -0.0018289355 * scale
    };
    Transform::from_xyz(0., -theia_network::world::PAWN_Y + floor_offset, 0.)
        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::splat(scale))
}

fn sample_poses(world: &mut World) {
    let roots: Vec<_> = world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .filter_map(|(e, v)| v.clip.or(v.gait.map(|g| g.index())).map(|g| (e, g)))
        .collect();
    for (root, gait) in roots {
        let visual = world.get::<Visual>(root).unwrap();
        if visual.samples[gait].is_empty() {
            let samples = descendants(world, root)
                .into_iter()
                .filter(|e| *e != root)
                .filter_map(|e| world.get::<Transform>(e).map(|t| (e, *t)))
                .collect();
            world.get_mut::<Visual>(root).unwrap().samples[gait] = samples;
        } else {
            let changes = visual.samples[gait]
                .iter()
                .filter(|(e, before)| {
                    world.get::<Transform>(*e).is_some_and(|now| {
                        now.translation.distance_squared(before.translation) > 1e-10
                            || 1. - now.rotation.dot(before.rotation).abs() > 1e-7
                    })
                })
                .count();
            let local = world
                .get::<LocalPawn>(world.get::<Visual>(root).unwrap().owner)
                .is_some();
            let mut visual = world.get_mut::<Visual>(root).unwrap();
            if let Some(index @ 0..3) = locomotion::gait_index(gait) {
                visual.pose_changes[index] = visual.pose_changes[index].max(changes);
            }
            if gait == 4 && !local {
                let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
                metrics.maximum_remote_death_pose_changes =
                    metrics.maximum_remote_death_pose_changes.max(changes);
            }
            if gait == 4 && local {
                let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
                metrics.maximum_death_pose_changes =
                    metrics.maximum_death_pose_changes.max(changes);
            }
            if gait == 5 {
                let mut metrics = world.resource_mut::<LifeAnimationMetrics>();
                metrics.maximum_sword_pose_changes =
                    metrics.maximum_sword_pose_changes.max(changes);
            }
            if (6..10).contains(&gait)
                && let Some(mut animation) =
                    world.get_resource_mut::<crate::dodge::animation::State>()
            {
                animation.maximum_pose_changes[gait - 6] =
                    animation.maximum_pose_changes[gait - 6].max(changes);
            }
            if let Some(index) = locomotion::gait_index(gait)
                && let Some(mut gray) = world.get_resource_mut::<crate::graybox::Graybox>()
            {
                gray.pose_changes[index] = gray.pose_changes[index].max(changes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_clip_slots_do_not_alias_when_death_is_absent() {
        assert_eq!(
            optional_slots(&["Idle", "Walk", "Run", "Jump"]),
            [None, None]
        );
        assert_eq!(
            optional_slots(&["Idle", "Walk", "Run", "Jump", "SwordIdle"]),
            [None, Some(4)]
        );
        assert_eq!(
            optional_slots(&["Idle", "Walk", "Run", "Jump", "Death"]),
            [Some(4), None]
        );
        assert_eq!(
            optional_slots(&["Idle", "Walk", "Run", "Jump", "SwordIdle", "Death"]),
            [Some(5), Some(4)]
        );
    }
    #[test]
    fn sword_stance_only_replaces_grounded_standing() {
        assert_eq!(select_clip(true, true, false, false, Gait::Stand, true), 5);
        assert_eq!(select_clip(true, true, false, false, Gait::Stand, false), 0);
        assert_eq!(select_clip(true, true, false, false, Gait::Walk, true), 1);
        assert_eq!(select_clip(true, true, false, false, Gait::Run, true), 2);
        assert_eq!(select_clip(true, true, false, true, Gait::Stand, true), 3);
        assert_eq!(select_clip(true, true, true, false, Gait::Stand, true), 4);
        assert_eq!(select_clip(true, false, true, false, Gait::Stand, true), 0);
        assert_eq!(
            select_clip(false, false, false, false, Gait::Stand, true),
            0
        );
    }
    #[test]
    fn stance_pose_probe_tracks_bones_without_counting_owner_motion_as_gait() {
        let mut world = World::new();
        world.init_resource::<LifeAnimationMetrics>();
        let owner = world.spawn((LocalPawn, Transform::default())).id();
        let root = world
            .spawn((
                Transform::default(),
                ChildOf(owner),
                Visual {
                    owner,
                    kind: 0,
                    players: Vec::new(),
                    gait: Some(Gait::Stand),
                    clip: Some(5),
                    pose_changes: [0; 3],
                    samples: std::array::from_fn(|_| Vec::new()),
                    life_key: None,
                    grip_active: false,
                    layer_frames: 0,
                    attack_sequence: None,
                    dodge_key: None,
                },
            ))
            .id();
        let bone = world.spawn((Transform::default(), ChildOf(root))).id();
        sample_poses(&mut world);
        world.get_mut::<Transform>(owner).unwrap().translation.x = 200.;
        sample_poses(&mut world);
        assert_eq!(
            world
                .resource::<LifeAnimationMetrics>()
                .maximum_sword_pose_changes,
            0
        );
        world.get_mut::<Transform>(bone).unwrap().rotation = Quat::from_rotation_y(0.4);
        sample_poses(&mut world);
        assert_eq!(
            world
                .resource::<LifeAnimationMetrics>()
                .maximum_sword_pose_changes,
            1
        );
        assert_eq!(world.get::<Visual>(root).unwrap().pose_changes, [0; 3]);
    }
    #[test]
    fn death_overrides_airborne_and_running_and_legacy_rig_keeps_existing_selection() {
        assert_eq!(select_clip(true, true, true, true, Gait::Run, true), 4);
        assert_eq!(select_clip(true, true, false, true, Gait::Run, true), 3);
        assert_eq!(
            select_clip(true, false, true, false, Gait::Walk, false),
            Gait::Walk.index()
        );
    }
    #[test]
    fn remote_visual_life_uses_public_generation_while_owner_keeps_private_barrier() {
        use std::{num::NonZeroU64, sync::Arc};
        use theia_core::{
            identity::{EntityId, EntityKey, WorldEpoch},
            player_life::LifeKey,
            time::TickRate,
        };
        use theia_network::{
            avatar_life_wire as avatar, client::State, player_life_wire as private,
        };
        let life = Arc::new(
            private::Profile::from_json(
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
                include_bytes!(
                    "../../../content/gameplay/development-creature-deflect-pursuit.json"
                ),
                include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
                include_bytes!("../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        );
        let key = |id| EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        };
        let profile = Arc::new(avatar::Profile::new(life.clone()));
        let mut public =
            avatar::ClientState::new(profile.clone(), key(1), TickRate::Hz30, [9; 32]).unwrap();
        let mut view = avatar::Observation {
            profile: profile.hash(),
            epoch: key(1).epoch,
            tick: 10,
            actors: vec![
                avatar::Actor {
                    key: LifeKey {
                        actor: key(1),
                        generation: NonZeroU64::MIN,
                    },
                    dead: true,
                },
                avatar::Actor {
                    key: LifeKey {
                        actor: key(2),
                        generation: NonZeroU64::MIN,
                    },
                    dead: true,
                },
            ],
        };
        public
            .apply(&avatar::encode(&profile, TickRate::Hz30, &view).unwrap())
            .unwrap();
        let mut state = State {
            public_avatar_life: Some(public),
            ..Default::default()
        };
        assert!(presentation_life(Some(&state), Some(key(1)), true).is_none());
        assert_eq!(
            presentation_life(Some(&state), Some(key(2)), false),
            Some((view.actors[1].key, true))
        );
        assert!(presentation_life(Some(&state), Some(key(3)), false).is_none());
        let private_view = theia_network::simulation::combat::player_life::Observation {
            profile: life.hash(),
            key: view.actors[0].key,
            tick: 10,
            movement_stream: 1,
            health: 100,
            maximum_health: 100,
            respawn_at: None,
            windup: None,
            combat: None,
        };
        let mut own =
            private::ClientState::new(life.clone(), key(1), TickRate::Hz30, [9; 32]).unwrap();
        own.apply(&private::encode(&life, TickRate::Hz30, &private_view).unwrap())
            .unwrap();
        state.player_life = Some(own);
        assert_eq!(
            presentation_life(Some(&state), Some(key(1)), true),
            Some((private_view.key, false))
        );
        view.tick += 1;
        view.actors[1].key.generation = NonZeroU64::new(2).unwrap();
        view.actors[1].dead = false;
        state
            .public_avatar_life
            .as_mut()
            .unwrap()
            .apply(&avatar::encode(&profile, TickRate::Hz30, &view).unwrap())
            .unwrap();
        assert_eq!(
            presentation_life(Some(&state), Some(key(2)), false),
            Some((view.actors[1].key, false))
        );
        assert!(presentation_life(None, Some(key(2)), false).is_none());
    }
    #[test]
    fn fresh_life_resets_pose_and_phase_including_new_actor_in_same_generation() {
        use theia_core::{
            identity::{EntityId, EntityKey, WorldEpoch},
            player_life::LifeKey,
        };
        let mut visual = Visual {
            owner: Entity::PLACEHOLDER,
            kind: 0,
            players: Vec::new(),
            gait: Some(Gait::Run),
            clip: Some(4),
            pose_changes: [0; 3],
            samples: std::array::from_fn(|_| Vec::new()),
            life_key: None,
            grip_active: false,
            layer_frames: 0,
            attack_sequence: None,
            dodge_key: None,
        };
        let key = LifeKey {
            actor: EntityKey {
                entity: EntityId::new(1).unwrap(),
                epoch: WorldEpoch::new(1).unwrap(),
            },
            generation: std::num::NonZeroU64::MIN,
        };
        assert!(!visual.observe_life(key));
        assert!(!visual.observe_life(key));
        visual.samples[4].push((Entity::PLACEHOLDER, Transform::default()));
        let next = LifeKey {
            actor: EntityKey {
                entity: EntityId::new(2).unwrap(),
                ..key.actor
            },
            ..key
        };
        assert!(visual.observe_life(next));
        assert_eq!(visual.clip, None);
        assert!(visual.samples.iter().all(Vec::is_empty));
        visual.clip = Some(4);
        assert!(visual.observe_life(LifeKey {
            generation: std::num::NonZeroU64::new(2).unwrap(),
            ..next
        }));
        assert_eq!(visual.clip, None);
    }
    #[test]
    fn polygon_feet_and_source_positive_z_follow_gameplay_forward() {
        let t = polygon_transform();
        assert!((t.rotation * Vec3::Z).distance(Vec3::NEG_Z) < 1e-6);
        assert!((t.transform_point(Vec3::ZERO).y + theia_network::world::PAWN_Y).abs() < 1e-6);
    }
    #[test]
    fn human_feet_follow_the_authoritative_ground_and_forward_axis() {
        let transform = actor_transform(0);
        let foot = transform.transform_point(Vec3::new(0., -0.01880305, 0.));
        for ground in [-2., 0., 5., 30.] {
            assert!((ground + theia_network::world::PAWN_Y + foot.y - ground).abs() < 1e-5);
        }
        let forward = transform.rotation * Vec3::X;
        assert!(forward.distance(Vec3::NEG_Z) < 1e-6);
        assert!((transform.scale.x * 2.1322405 - 1.8).abs() < 1e-6);
    }
    #[test]
    fn pose_probe_ignores_terrain_root_motion_and_children_despawn_with_player() {
        let mut world = World::new();
        let owner = world.spawn(Transform::default()).id();
        let root = world
            .spawn((
                Transform::default(),
                ChildOf(owner),
                Visual {
                    owner,
                    kind: 0,
                    players: Vec::new(),
                    gait: Some(Gait::Walk),
                    clip: None,
                    pose_changes: [0; 3],
                    samples: std::array::from_fn(|_| Vec::new()),
                    life_key: None,
                    grip_active: false,
                    layer_frames: 0,
                    attack_sequence: None,
                    dodge_key: None,
                },
            ))
            .id();
        let bone = world.spawn((Transform::default(), ChildOf(root))).id();
        sample_poses(&mut world);
        world.get_mut::<Transform>(owner).unwrap().translation.y = 50.;
        sample_poses(&mut world);
        assert_eq!(world.get::<Visual>(root).unwrap().pose_changes, [0; 3]);
        world.get_mut::<Transform>(bone).unwrap().rotation = Quat::from_rotation_x(0.5);
        sample_poses(&mut world);
        assert_eq!(world.get::<Visual>(root).unwrap().pose_changes, [0, 1, 0]);
        world.entity_mut(owner).despawn();
        assert!(world.get_entity(root).is_err());
        assert!(world.get_entity(bone).is_err());
    }
    #[test]
    fn house_fits_the_existing_solid_building_envelope() {
        let transform = house_transform();
        for x in [-9.562015, 9.714225] {
            for y in [-0.14303547, 17.085135] {
                for z in [-9.259302, 9.171431] {
                    let p = transform.transform_point(Vec3::new(x, y, z));
                    assert!((-11.00001..=-2.99999).contains(&p.x));
                    assert!((-11.00001..=-2.99999).contains(&p.z));
                    assert!((-0.00001..=6.80001).contains(&p.y));
                }
            }
        }
    }
}
