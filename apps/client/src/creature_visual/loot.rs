//! Bounded private corpse art reconstructed from committed loot availability.
use super::*;
use crate::loot::{CorpsePick, CorpseVisual};
#[derive(Resource)]
struct PickAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    sparkle_mesh: Handle<Mesh>,
    sparkle_material: Handle<StandardMaterial>,
}

pub(super) fn reconcile(world: &mut World, state: &theia_network::client::State) {
    let connected = world.resource::<crate::network::Network>().connected();
    reconcile_observation(world, state, connected);
}
fn reconcile_observation(world: &mut World, state: &theia_network::client::State, connected: bool) {
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
        let sparkle_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Sphere::new(0.045).mesh().ico(1).unwrap());
        let sparkle_material =
            world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(1., 0.78, 0.24),
                    emissive: LinearRgba::rgb(3., 1.8, 0.25),
                    unlit: true,
                    ..default()
                });
        world.insert_resource(PickAssets {
            mesh,
            material,
            sparkle_mesh,
            sparkle_material,
        });
    }
    let frame = world.resource::<crate::floating_origin::Frame>().render;
    let mut sources = Vec::new();
    if connected && state.error.is_none() {
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
        // Eight reusable motes per authoritative private corpse. Child ownership
        // removes every mote with the last committed claim (or chunk departure).
        let assets = world.resource::<PickAssets>();
        let mesh = assets.sparkle_mesh.clone();
        let material = assets.sparkle_material.clone();
        for index in 0..8 {
            world.spawn((
                Sparkle(index),
                ChildOf(owner),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::default(),
                bevy::picking::Pickable::IGNORE,
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
            ));
        }
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

#[derive(Component)]
pub(super) struct Sparkle(u8);

pub(super) fn animate(time: Res<Time>, mut motes: Query<(&Sparkle, &mut Transform)>) {
    for (mote, mut transform) in &mut motes {
        let phase = (time.elapsed_secs() * 0.45 + f32::from(mote.0) / 8.).fract();
        let angle = f32::from(mote.0) * 2.39996 + phase * 0.7;
        transform.translation = Vec3::new(
            angle.cos() * 0.45,
            -theia_network::world::PAWN_Y + 0.25 + phase * 1.35,
            angle.sin() * 0.7,
        );
        transform.scale = Vec3::splat((phase * std::f32::consts::PI).sin().max(0.) * 1.3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        content::ContentId,
        progression::KillId,
        rewards::{ClaimId, CorpseSource},
    };
    #[test]
    fn partial_loot_keeps_one_effect_and_last_claim_disconnect_or_unload_removes_it() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<crate::floating_origin::Frame>();
        world.insert_resource(Catalog {
            scene: default(),
            graph: default(),
            nodes: vec![],
            durations: vec![],
        });
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        state.chunk_report = Some(theia_network::wire::ChunkReady {
            version: theia_network::wire::VERSION,
            epoch: actor.epoch.value(),
            sequence: 1,
            chunks: vec![[0, 0]],
        });
        let source = CorpseSource {
            life: KillId::from_victim(actor),
            position: GlobalPosition::new([2., 0.9, 2.]).unwrap(),
        };
        state.rewards = Some(theia_network::reward_wire::State {
            actor,
            generation: 1,
            profile_hash: [1; 32],
            available: (1..=2)
                .map(|id| theia_network::reward_wire::Available {
                    reward: ContentId::new("theia.test.loot").unwrap(),
                    claim: ClaimId::new([id; 16]).unwrap(),
                    corpse: Some(source),
                })
                .collect(),
        });
        let count = |world: &mut World| world.query::<&Sparkle>().iter(world).count();
        reconcile_observation(&mut world, &state, true);
        assert_eq!(count(&mut world), 8);
        let root = world
            .query_filtered::<Entity, With<CorpseVisual>>()
            .single(&world)
            .unwrap();
        // A refused pickup changes no committed availability. Neither refusal
        // nor a successful partial pickup restarts/duplicates the effect.
        reconcile_observation(&mut world, &state, true);
        state.rewards.as_mut().unwrap().available.pop();
        reconcile_observation(&mut world, &state, true);
        assert_eq!(count(&mut world), 8);
        assert!(world.get_entity(root).is_ok());
        let available = state.clone();
        state.rewards.as_mut().unwrap().available.clear();
        reconcile_observation(&mut world, &state, true);
        assert_eq!(count(&mut world), 0);
        reconcile_observation(&mut world, &available, true);
        assert_eq!(count(&mut world), 8);
        reconcile_observation(&mut world, &available, false);
        assert_eq!(count(&mut world), 0);
        state = available;
        state.chunk_report.as_mut().unwrap().chunks.clear();
        reconcile_observation(&mut world, &state, true);
        assert_eq!(count(&mut world), 0);
    }
}
