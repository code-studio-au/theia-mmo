//! Original development props on the inspected POLYGON hand joint. Standard
//! Bevy meshes, hierarchy and animation; no colliders or gameplay authority.
use crate::{
    movement::LocalPawn,
    network::{Network, RemotePawn},
    reference_scene::{self, Visual},
};
use bevy::{mesh::skinning::SkinnedMesh, prelude::*, transform::TransformSystems};
use theia_core::{
    content::ContentId,
    coordinates::{GlobalPosition, LocalPosition},
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::Catalog,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Sword,
    Wand,
}
impl Kind {
    fn for_id(id: &ContentId) -> Option<Self> {
        match id.as_str() {
            "theia.dev.simple-sword" => Some(Self::Sword),
            "theia.dev.simple-wand" => Some(Self::Wand),
            _ => None,
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Sword => 0,
            Self::Wand => 1,
        }
    }
}
#[derive(Clone)]
struct Part {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
}
#[derive(Resource)]
struct Props([Vec<Part>; 2]);
#[derive(Component)]
pub(crate) struct Attachment {
    pub(crate) owner: Entity,
    pub(crate) actor: EntityKey,
    pub(crate) hand: Entity,
    pub(crate) appearance: ContentId,
    last: Option<GlobalPosition>,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub frames: u64,
    pub active: usize,
    pub attached: u64,
    pub removed: u64,
    pub local_samples: u64,
    pub remote_samples: u64,
    pub local_visible_samples: u64,
    pub remote_visible_samples: u64,
    pub visible_moving_samples: u64,
    pub moving_samples: u64,
    pub max_socket_error: f32,
    pub max_grip_offset: f32,
    pub max_scale_error: f32,
    pub readiness_waits: u64,
    pub last_error: Option<String>,
}
fn socket() -> Transform {
    // Inspected SyntyRig carries centimetre scale. Source hand +Y goes towards
    // fingers; move grip 3 cm into the palm, point the blade towards hand -Z.
    Transform {
        translation: Vec3::new(0., 3., 0.),
        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        scale: Vec3::splat(100.),
    }
}
pub(crate) fn install(app: &mut App, catalog: &Catalog) -> Result<(), String> {
    if !app
        .world()
        .get_resource::<reference_scene::Config>()
        .is_some_and(|c| c.polygon)
    {
        return Err("Equipment requires the inspected POLYGON development character".into());
    }
    for definition in catalog.definitions() {
        if let Some(id) = definition.main_hand_appearance()
            && Kind::for_id(id).is_none()
        {
            return Err(format!(
                "No development appearance binding for {}",
                id.as_str()
            ));
        }
    }
    app.init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, sync.after(TransformSystems::Propagate))
        .add_systems(Last, observe);
    Ok(())
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let steel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.82, 0.95),
        metallic: 0.85,
        perceptual_roughness: 0.28,
        ..default()
    });
    let wood = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.065, 0.025),
        perceptual_roughness: 0.7,
        ..default()
    });
    let brass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.45, 0.08),
        metallic: 0.7,
        perceptual_roughness: 0.35,
        ..default()
    });
    let crystal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.8, 0.95),
        emissive: LinearRgba::new(0.05, 0.35, 0.5, 1.),
        perceptual_roughness: 0.35,
        ..default()
    });
    let mut cube = |size: Vec3, y: f32, material: Handle<StandardMaterial>| Part {
        mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
        material,
        transform: Transform::from_xyz(0., y, 0.),
    };
    let sword = vec![
        cube(Vec3::new(0.055, 0.6, 0.025), 0.38, steel),
        cube(Vec3::new(0.18, 0.03, 0.04), 0.08, brass),
        cube(Vec3::new(0.035, 0.14, 0.035), 0., wood.clone()),
    ];
    let mut wand = vec![cube(Vec3::new(0.028, 0.45, 0.028), 0.15, wood)];
    wand.push(Part {
        mesh: meshes.add(Sphere::new(0.055)),
        material: crystal,
        transform: Transform::from_xyz(0., 0.43, 0.),
    });
    commands.insert_resource(Props([sword, wand]));
}
fn report(world: &mut World, error: String) {
    let mut metrics = world.resource_mut::<Metrics>();
    if metrics.last_error.as_ref() != Some(&error) {
        error!("{error}");
        metrics.last_error = Some(error);
    }
    metrics.readiness_waits += 1;
}
fn sync(world: &mut World) {
    world.resource_mut::<Metrics>().frames += 1;
    let network = world.resource::<Network>();
    let state = network.state();
    let connected = network.connected();
    let desired = if connected {
        state.appearance.as_ref()
    } else {
        None
    };
    let epoch = state
        .welcome
        .as_ref()
        .and_then(|w| WorldEpoch::new(w.epoch));
    let visuals: Vec<_> = world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .filter(|(_, v)| v.kind == 0 && !v.players.is_empty())
        .map(|(e, v)| (e, v.owner))
        .collect();
    let mut wanted = std::collections::BTreeMap::new();
    for (visual, owner) in visuals {
        let id = if world.get::<LocalPawn>(owner).is_some() {
            state.id
        } else {
            world.get::<RemotePawn>(owner).map(RemotePawn::actor_id)
        };
        let Some(key) = id
            .and_then(EntityId::new)
            .zip(epoch)
            .map(|(entity, epoch)| EntityKey { epoch, entity })
        else {
            continue;
        };
        if let Some(appearance) = desired
            .and_then(|d| d.actors.iter().find(|a| a.key == key))
            .and_then(|a| a.main_hand.clone())
        {
            wanted.insert(owner, (visual, key, appearance));
        }
    }
    let existing: Vec<_> = world
        .query::<(Entity, &Attachment)>()
        .iter(world)
        .map(|(e, a)| (e, a.owner, a.actor, a.hand, a.appearance.clone()))
        .collect();
    let mut retained = std::collections::BTreeSet::new();
    for (entity, owner, key, hand, appearance) in existing {
        let keep = wanted
            .get(&owner)
            .is_some_and(|(_, k, id)| *k == key && *id == appearance)
            && world.entities().contains(hand)
            && retained.insert(owner);
        if !keep {
            world.entity_mut(entity).despawn();
            world.resource_mut::<Metrics>().removed += 1;
        }
    }
    for (owner, (visual, key, appearance)) in wanted {
        if retained.contains(&owner) {
            continue;
        }
        let descendants = reference_scene::descendants(world, visual);
        let hands: Vec<_> = descendants
            .iter()
            .copied()
            .filter(|e| {
                world
                    .get::<Name>(*e)
                    .is_some_and(|n| n.as_str() == "Hand_R")
            })
            .collect();
        if hands.len() != 1 {
            report(
                world,
                format!(
                    "Expected exactly one POLYGON Hand_R joint; found {}",
                    hands.len()
                ),
            );
            continue;
        }
        let hand = hands[0];
        let skin_ok = descendants.iter().any(|e| {
            world
                .get::<SkinnedMesh>(*e)
                .is_some_and(|s| s.joints.len() == 50 && s.joints.contains(&hand))
        });
        let Some(global) = world.get::<GlobalTransform>(hand).copied() else {
            continue;
        };
        let scale = global.to_scale_rotation_translation().0;
        if !skin_ok || !scale.abs_diff_eq(Vec3::splat(0.01), 0.0001) {
            report(
                world,
                "POLYGON equipment joint/centimetre transform contract failed".into(),
            );
            continue;
        }
        let Some(kind) = Kind::for_id(&appearance) else {
            report(
                world,
                format!("Unknown equipment appearance {}", appearance.as_str()),
            );
            continue;
        };
        let parts = world.resource::<Props>().0[kind.index()].clone();
        let local = socket();
        let global = global.mul_transform(local);
        let root = world
            .spawn((
                Name::new("Development main-hand attachment"),
                local,
                global,
                Visibility::Inherited,
                ChildOf(hand),
                Attachment {
                    owner,
                    actor: key,
                    hand,
                    appearance,
                    last: None,
                },
            ))
            .id();
        for part in parts {
            world.spawn((
                Mesh3d(part.mesh),
                MeshMaterial3d(part.material),
                part.transform,
                global.mul_transform(part.transform),
                ChildOf(root),
            ));
        }
        world.resource_mut::<Metrics>().attached += 1;
    }
    let active = world.query::<&Attachment>().iter(world).count();
    world.resource_mut::<Metrics>().active = active;
}
fn observe(world: &mut World) {
    let roots: Vec<_> = world
        .query::<(Entity, &Attachment, &GlobalTransform)>()
        .iter(world)
        .map(|(e, a, t)| (e, a.owner, a.hand, *t, a.last))
        .collect();
    for (entity, owner, hand, actual, last) in roots {
        let Some(hand) = world.get::<GlobalTransform>(hand).copied() else {
            continue;
        };
        let expected = hand.mul_transform(socket());
        let socket_error = actual.translation().distance(expected.translation());
        let grip_offset = actual.translation().distance(hand.translation());
        let scale_error = (actual.to_scale_rotation_translation().0 - Vec3::ONE)
            .abs()
            .max_element();
        let position = LocalPosition::new(actual.translation().to_array()).and_then(|p| {
            world
                .resource::<crate::floating_origin::Frame>()
                .render
                .to_global(p)
        });
        let moving = position.zip(last).is_some_and(|(a, b)| {
            a.metres()
                .iter()
                .zip(b.metres())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                > 0.00000001
        });
        let local = world.get::<LocalPawn>(owner).is_some();
        let visible = reference_scene::descendants(world, entity)
            .into_iter()
            .any(|e| {
                world.get::<Mesh3d>(e).is_some()
                    && world.get::<ViewVisibility>(e).is_some_and(|v| v.get())
            });
        let mut metrics = world.resource_mut::<Metrics>();
        metrics.local_samples += u64::from(local);
        metrics.remote_samples += u64::from(!local);
        metrics.local_visible_samples += u64::from(local && visible);
        metrics.remote_visible_samples += u64::from(!local && visible);
        metrics.visible_moving_samples += u64::from(moving && visible);
        metrics.moving_samples += u64::from(moving);
        metrics.max_socket_error = metrics.max_socket_error.max(socket_error);
        metrics.max_grip_offset = metrics.max_grip_offset.max(grip_offset);
        metrics.max_scale_error = metrics.max_scale_error.max(scale_error);
        world.get_mut::<Attachment>(entity).unwrap().last = position;
    }
}
