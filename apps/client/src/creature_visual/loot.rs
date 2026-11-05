//! Bounded private corpse art reconstructed from committed loot availability.
use super::*;
use crate::loot::{CorpsePick, CorpseVisual};
#[derive(Resource)]
struct PickAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

pub(super) fn reconcile(world: &mut World, state: &theia_network::client::State) {
    if !world.contains_resource::<PickAssets>() {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(1.1, 0.5, 2.6));
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgba(0., 0., 0., 0.),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
        world.insert_resource(PickAssets { mesh, material });
    }
    let frame = world.resource::<crate::floating_origin::Frame>().render;
    let mut sources = Vec::new();
    if world.resource::<crate::network::Network>().connected() && state.error.is_none() {
        for source in state
            .rewards
            .iter()
            .flat_map(|r| &r.available)
            .filter_map(|r| r.corpse)
        {
            let resident = source.position.chunk(128).is_some_and(|id| {
                state
                    .chunk_report
                    .as_ref()
                    .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
            });
            if resident
                && frame.to_local(source.position).is_some()
                && !sources
                    .iter()
                    .any(|s: &theia_core::rewards::CorpseSource| s.life == source.life)
            {
                sources.push(source);
            }
        }
    }
    let existing: Vec<_> = world
        .query::<(Entity, &CorpseVisual)>()
        .iter(world)
        .map(|(e, c)| (e, c.0.life))
        .collect();
    for (entity, life) in &existing {
        if let Some(source) = sources.iter().find(|s| s.life == *life) {
            world.get_mut::<Transform>(*entity).unwrap().translation =
                Vec3::from_array(frame.to_local(source.position).unwrap().metres());
        } else {
            world.despawn(*entity);
        }
    }
    for source in &sources {
        if existing.iter().any(|(_, life)| *life == source.life) {
            continue;
        }
        let scene = world.resource::<Catalog>().scene.clone();
        let owner = world
            .spawn((
                CorpseVisual(*source),
                Transform::from_translation(Vec3::from_array(
                    frame.to_local(source.position).unwrap().metres(),
                )),
                Visibility::Inherited,
            ))
            .id();
        // Mesh picking reads the unskinned CPU positions in Bevy 0.20. This
        // bounded dead-body pick surface follows the rendered wolf pose instead
        // of exposing an invisible standing/bind-pose silhouette. No collider.
        let pick = world.resource::<PickAssets>();
        let mesh = pick.mesh.clone();
        let material = pick.material.clone();
        world.spawn((
            CorpsePick,
            ChildOf(owner),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.1, -theia_network::world::PAWN_Y + 0.25, -0.1),
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
        world.spawn((
            WorldAssetRoot(scene),
            ChildOf(owner),
            Transform::from_xyz(0., -theia_network::world::PAWN_Y, 0.)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            Visual {
                owner,
                previous: source.position,
                locomotion: Locomotion::default(),
                players: Vec::new(),
                active: None,
            },
        ));
    }
    // Replace the transient public death visual while its durable private body
    // is shown. Live replacement NPCs have a different incarnation/key.
    let old: Vec<_> = world
        .query::<(Entity, &crate::targeting::TargetVisual)>()
        .iter(world)
        .filter(|(_, v)| sources.iter().any(|s| s.life.victim() == v.key))
        .map(|(e, _)| e)
        .collect();
    for entity in old {
        world
            .get_mut::<Visibility>(entity)
            .unwrap()
            .clone_from(&Visibility::Hidden);
    }
}
