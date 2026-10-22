//! Opt-in local compatibility art. No imported model determines server geometry.
use crate::movement::{Gait, LocalPawn, Motion};
use bevy::{asset::LoadState, prelude::*, world_serialization::WorldAsset};
use std::{path::PathBuf, time::Instant};

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
}
#[derive(Component)]
pub struct Wolf;
#[derive(Component)]
struct Attached;
#[derive(Resource)]
pub struct Ready;
#[derive(Component)]
pub struct Visual {
    pub owner: Entity,
    pub kind: usize,
    pub players: Vec<Entity>,
    pub gait: Option<Gait>,
    clip: Option<usize>,
    pub pose_changes: [usize; 3],
    samples: [Vec<(Entity, Transform)>; 4],
}
#[derive(Component)]
pub struct House;

pub fn load(mut commands: Commands, server: Res<AssetServer>, config: Res<Config>) {
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
            let names = if i == 0 && world.resource::<Config>().polygon {
                &["Idle", "Walk", "Run", "Jump"][..]
            } else if i == 0 {
                &[
                    "Stand (ID 0 variation 0)",
                    "Walk (ID 4 variation 0)",
                    "Run (ID 5 variation 0)",
                ][..]
            } else {
                &["Stand (ID 0 variation 0)"][..]
            };
            let mut selected = Vec::new();
            for name in names {
                let clip = gltf
                    .named_animations
                    .get(*name)
                    .ok_or_else(|| format!("Missing clip {name}"))?
                    .clone();
                if world
                    .resource::<Assets<AnimationClip>>()
                    .get(&clip)
                    .is_none_or(|c| c.duration() <= 0.)
                {
                    return Err(format!("Empty clip {name}"));
                }
                selected.push(clip);
            }
            clips.push(selected);
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
                world.insert_resource(assets);
            }
            Ok(None) => return,
            Err(error) => {
                fail(world, &error);
                return;
            }
        }
    }
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
            let handle = world.resource::<Catalog>().graphs[visual.kind].clone();
            for player in &players {
                world
                    .entity_mut(*player)
                    .insert(AnimationGraphHandle(handle.clone()));
            }
            world.get_mut::<Visual>(root).unwrap().players = players;
        }
        let visual = world.get::<Visual>(root).unwrap();
        let gait = if visual.kind == 0 {
            Gait::from_speed(world.get::<Motion>(visual.owner).map_or(0., |m| m.0))
        } else {
            Gait::Stand
        };
        let airborne = world
            .get_resource::<crate::graybox::Graybox>()
            .is_some_and(|g| {
                world.get::<Transform>(visual.owner).is_some_and(|t| {
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
        let clip = if world.resource::<Config>().polygon && airborne {
            3
        } else {
            gait.index()
        };
        if visual.clip != Some(clip) {
            let index = world.resource::<Catalog>().indices[visual.kind][clip];
            let players = visual.players.clone();
            let owner = visual.owner;
            for player in players {
                // Change only on state transitions, so clips keep their phase.
                world
                    .get_mut::<AnimationPlayer>(player)
                    .unwrap()
                    .stop_all()
                    .play(index)
                    .repeat();
            }
            info!("Reference actor {:?}: {:?}", owner, gait);
            let mut visual = world.get_mut::<Visual>(root).unwrap();
            visual.gait = Some(gait);
            visual.clip = Some(clip);
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
            let mut visual = world.get_mut::<Visual>(root).unwrap();
            if gait < 3 {
                visual.pose_changes[gait] = visual.pose_changes[gait].max(changes);
            }
            if let Some(mut gray) = world.get_resource_mut::<crate::graybox::Graybox>() {
                gray.pose_changes[gait] = gray.pose_changes[gait].max(changes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
