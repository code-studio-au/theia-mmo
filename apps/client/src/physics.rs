//! Standard Avian integration. Server poses remain authoritative; characters
//! are query shapes, never physical obstacles to players or other characters.
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct WorldPhysicsPlugin;
#[derive(Resource)]
pub struct Backend {
    pub static_colliders: usize,
    pub frame_revision: u64,
    pub probes: u64,
    pub max_ground_error: f32,
    pub generation: u64,
    pub probed_generation: u64,
    pub residency_probes: u64,
}
#[derive(Component)]
struct StaticWorld;
impl Plugin for WorldPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::floating_origin::Frame>()
            .add_plugins(PhysicsPlugins::new(PostUpdate))
            .add_systems(Startup, spawn_world)
            .add_systems(
                PostUpdate,
                avian3d::collider_tree::update_moved_collider_aabbs::<Collider>
                    .after(PhysicsSystems::Prepare)
                    .before(PhysicsSystems::StepSimulation)
                    .run_if(query_frame_changed),
            )
            .add_systems(Last, probe.before(crate::network::capture));
    }
}
// Avian skips PhysicsSchedule when frame delta is zero (including startup).
// Refresh its standard BVH after preparation when anchored coordinates change,
// so first-frame admission and paused/zero-delta rebases remain query-correct.
fn query_frame_changed(
    frame: Res<crate::floating_origin::Frame>,
    backend: Res<Backend>,
    mut previous: Local<Option<([f32; 3], u64)>>,
) -> bool {
    let offset = frame.render.origin().metres().map(|v| v as f32);
    let changed = *previous != Some((offset, backend.generation));
    *previous = Some((offset, backend.generation));
    changed
}

fn spawn_world(
    mut commands: Commands,
    terrain: Option<Res<crate::terrain_scene::Config>>,
    chunks: Option<Res<crate::chunk_world::Config>>,
    graybox: Option<Res<crate::graybox::Graybox>>,
) {
    if graybox.is_some() {
        commands.insert_resource(Backend {
            static_colliders: 0,
            frame_revision: 0,
            probes: 0,
            max_ground_error: 0.,
            generation: 0,
            probed_generation: 0,
            residency_probes: 0,
        });
        return;
    }
    let scene = if let Some(chunks) = &chunks {
        chunks.scene()
    } else if let Some(terrain) = &terrain {
        &terrain.terrain.physics
    } else {
        theia_network::world::physics_scene()
    };
    for collider in &scene.colliders {
        commands.spawn((
            StaticWorld,
            Name::new("Avian static world collider"),
            crate::floating_origin::Anchored::at(collider.center),
            collider.bundle(),
        ));
    }
    commands.insert_resource(Backend {
        static_colliders: scene.colliders.len(),
        frame_revision: 0,
        probes: 0,
        max_ground_error: 0.,
        generation: chunks.as_ref().map_or(0, |c| c.active.generation()),
        probed_generation: 0,
        residency_probes: 0,
    });
    info!(
        "Physics: Avian {} | {} static colliders | characters nonblocking",
        theia_network::physics::AVIAN_REVISION,
        scene.colliders.len()
    );
}

pub(crate) fn replace_scene(
    world: &mut World,
    scene: &theia_network::physics::StaticScene,
    generation: u64,
) {
    let mut query = world.query_filtered::<Entity, With<StaticWorld>>();
    let remove: Vec<_> = query.iter(world).collect();
    for entity in remove {
        world.despawn(entity);
    }
    for collider in &scene.colliders {
        let position = world
            .resource::<crate::floating_origin::Frame>()
            .anchored(collider.center.to_array().map(f64::from));
        world
            .spawn((
                StaticWorld,
                crate::floating_origin::Anchored::at(collider.center),
                collider.bundle(),
            ))
            .insert(Transform::from_translation(position));
    }
    let mut backend = world.resource_mut::<Backend>();
    backend.static_colliders = scene.colliders.len();
    backend.generation = generation;
    backend.probed_generation = 0;
    backend.probes = 0;
}

/// Probe the actual client BVH after PostUpdate physics, in the current render
/// frame. A render-only shift with stale physics cannot pass this check.
pub(crate) fn probe(
    query: SpatialQuery,
    pawn: Query<&Transform, With<crate::movement::LocalPawn>>,
    frame: Res<crate::floating_origin::Frame>,
    mut backend: ResMut<Backend>,
    network: Option<Res<crate::network::Network>>,
    terrain: Option<Res<crate::terrain_scene::Config>>,
    chunks: Option<Res<crate::chunk_world::Config>>,
) {
    if network.as_ref().is_some_and(|n| {
        n.state()
            .welcome
            .is_some_and(|w| w.world == theia_network::procedural::IDENTITY)
    }) {
        return;
    }
    backend.frame_revision = frame.revision;
    if network.as_ref().is_none_or(|n| !n.connected()) {
        return;
    }
    let Ok(pawn) = pawn.single() else {
        return;
    };
    let content = frame.content_point(pawn.translation);
    let scene: &theia_network::physics::StaticScene = if let Some(chunks) = &chunks {
        chunks.scene()
    } else if let Some(terrain) = &terrain {
        &terrain.terrain.physics
    } else {
        theia_network::world::physics_scene()
    };
    let expected = scene.ground([content.x, content.z], content.y);
    let actual = query.cast_ray(
        pawn.translation,
        Dir3::NEG_Y,
        4096.,
        true,
        &SpatialQueryFilter::from_mask(theia_network::physics::GROUND),
    );
    assert_eq!(
        actual.is_some(),
        expected.is_some(),
        "Client Avian/render ground presence mismatch: pawn={:?}, content={content:?}, expected={expected:?}, revision={} origin={:?}",
        pawn.translation,
        frame.revision,
        frame.render.origin().metres()
    );
    let error = if let (Some(hit), Some((height, _))) = (actual, expected) {
        (hit.distance - (content.y - height)).abs()
    } else {
        0.
    };
    backend.probes += 1;
    backend.probed_generation = backend.generation;
    backend.max_ground_error = backend.max_ground_error.max(error);
    assert!(error < 0.002, "Client Avian/render frame mismatch: {error}");
    if let Some(chunks) = &chunks {
        for id in chunks.data.index().ids() {
            let [x, _, z] = id
                .origin(theia_core::coordinates::INITIAL_CHUNK_METRES)
                .unwrap()
                .metres();
            let y = frame.content.origin().metres()[1] + f64::from(content.y);
            let Some(global) = theia_core::coordinates::GlobalPosition::new([x + 64., y, z + 32.])
            else {
                continue;
            };
            let (Some(local), Some(render)) = (
                frame.content.to_local(global),
                frame.render.to_local(global),
            ) else {
                continue;
            };
            let p = local.metres();
            let expected = scene.ground([p[0], p[2]], p[1]);
            let actual = query.cast_ray(
                Vec3::from_array(render.metres()),
                Dir3::NEG_Y,
                4096.,
                true,
                &SpatialQueryFilter::from_mask(theia_network::physics::GROUND),
            );
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "Live chunk BVH disagrees at {id:?}"
            );
            if let (Some(actual), Some((ground, _))) = (actual, expected) {
                assert!((actual.distance - (p[1] - ground)).abs() < 0.002);
            }
            backend.residency_probes += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{ecs::system::SystemState, time::TimeUpdateStrategy};
    use std::time::Duration;

    #[test]
    fn client_plugin_installs_real_static_colliders_without_character_bodies() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, WorldPhysicsPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )));
        app.finish();
        app.cleanup();
        app.update();
        app.update();
        assert_eq!(app.world().resource::<Backend>().static_colliders, 8);
        let mut state = SystemState::<SpatialQuery>::new(app.world_mut());
        let query = state.get(app.world()).unwrap();
        let hit = query
            .cast_ray(
                Vec3::new(-7., 0.9, 0.),
                Dir3::NEG_Z,
                10.,
                true,
                &SpatialQueryFilter::from_mask(theia_network::physics::SOLID),
            )
            .unwrap();
        assert!(hit.distance > 2. && hit.distance < 3.);
        assert!(
            app.world()
                .iter_entities()
                .all(|e| e.get::<RigidBody>().is_none_or(|b| *b == RigidBody::Static))
        );
    }
    #[test]
    fn zero_delta_startup_and_rebase_keep_actual_avian_queries_current() {
        use crate::floating_origin::{Frame, place_static};
        use theia_core::coordinates::{GlobalPosition, LocalFrame};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, WorldPhysicsPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .add_systems(Update, place_static);
        app.finish();
        app.cleanup();
        for (revision, origin) in [[32., 0., 32.], [-64., 2., 64.]].into_iter().enumerate() {
            {
                let mut frame = app.world_mut().resource_mut::<Frame>();
                frame.render = LocalFrame::new(GlobalPosition::new(origin).unwrap());
                frame.revision = revision as u64 + 1;
            }
            app.update();
            assert_eq!(
                app.world().resource::<Time<Physics>>().delta(),
                Duration::ZERO
            );
            let ray = app.world().resource::<Frame>().anchored([-7., 0.9, 0.]);
            let mut state = SystemState::<SpatialQuery>::new(app.world_mut());
            let query = state.get(app.world()).unwrap();
            let floor = query
                .cast_ray(
                    ray,
                    Dir3::NEG_Y,
                    10.,
                    true,
                    &SpatialQueryFilter::from_mask(theia_network::physics::GROUND),
                )
                .unwrap();
            assert!((floor.distance - 0.9).abs() < 0.00001);
            let wall = query
                .cast_ray(
                    ray,
                    Dir3::NEG_Z,
                    10.,
                    true,
                    &SpatialQueryFilter::from_mask(theia_network::physics::SOLID),
                )
                .unwrap();
            assert!(wall.distance > 2. && wall.distance < 3.);
        }
    }
    #[test]
    fn zero_delta_generation_replacement_removes_old_bodies_and_refreshes_actual_bvh() {
        use theia_network::physics::{GROUND, PlacedCollider, SOLID, StaticScene};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, WorldPhysicsPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.finish();
        app.cleanup();
        app.update();
        let scene = StaticScene::new(vec![PlacedCollider {
            shape: Collider::cuboid(10., 1., 10.),
            center: Vec3::new(0., 1.5, 0.),
            layer: GROUND | SOLID,
        }]);
        replace_scene(app.world_mut(), &scene, 2);
        app.update();
        let mut state = SystemState::<SpatialQuery>::new(app.world_mut());
        let query = state.get(app.world()).unwrap();
        let hit = query
            .cast_ray(
                Vec3::new(0., 5., 0.),
                Dir3::NEG_Y,
                10.,
                true,
                &SpatialQueryFilter::from_mask(GROUND),
            )
            .unwrap();
        assert!((hit.distance - 3.).abs() < 0.0001);
        assert!(
            query
                .cast_ray(
                    Vec3::new(-7., 0.9, 0.),
                    Dir3::NEG_Z,
                    10.,
                    true,
                    &SpatialQueryFilter::from_mask(SOLID)
                )
                .is_none()
        );
        assert_eq!(app.world().resource::<Backend>().static_colliders, 1);
    }
    #[test]
    fn rebase_updates_actual_avian_tree_and_late_children_in_one_frame() {
        use crate::floating_origin::{Anchored, Frame};
        use theia_core::coordinates::{GlobalPosition, LocalPosition};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, WorldPhysicsPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .add_systems(Update, crate::floating_origin::place_static);
        let root = app
            .world_mut()
            .spawn((
                Anchored::at(Vec3::new(20., 3., -17.)),
                Transform::from_xyz(20., 3., -17.),
            ))
            .id();
        let child_pose =
            Transform::from_xyz(0.125, 0.25, -0.5).with_rotation(Quat::from_rotation_x(0.3));
        let child = app.world_mut().spawn((ChildOf(root), child_pose)).id();
        app.finish();
        app.cleanup();
        app.update();
        app.update();
        for sign in [-1., 1.] {
            let origin = [sign * 1e9 + 0.01, sign * 1e9 + 0.02, sign * 1e9 - 0.03];
            {
                let mut frame = app.world_mut().resource_mut::<Frame>();
                frame.span = 2.;
                frame.admit(&theia_network::wire::Welcome {
                    id: 1,
                    epoch: if sign < 0. { 1 } else { 2 },
                    world: [0; 32],
                    origin,
                });
                let global = frame
                    .content
                    .to_global(LocalPosition::new([10., 0.9, -10.]).unwrap())
                    .unwrap();
                frame.focus(global);
            }
            app.update();
            let frame = app.world().resource::<Frame>();
            let ray = frame.anchored([-7., 0.9, 0.]);
            let expected_child = frame.anchored([20.125, 3.25, -17.5]);
            assert!(
                app.world()
                    .get::<GlobalTransform>(child)
                    .unwrap()
                    .translation()
                    .distance(expected_child)
                    < 0.0001
            );
            assert_eq!(*app.world().get::<Transform>(child).unwrap(), child_pose);
            for e in app.world().iter_entities() {
                if let (Some(anchor), Some(position)) = (e.get::<Anchored>(), e.get::<Position>()) {
                    assert!(position.0.distance(frame.anchored(anchor.0)) < 0.00001);
                }
            }
            let mut state = SystemState::<SpatialQuery>::new(app.world_mut());
            let hit = state
                .get(app.world())
                .unwrap()
                .cast_ray(
                    ray,
                    Dir3::NEG_Z,
                    10.,
                    true,
                    &SpatialQueryFilter::from_mask(theia_network::physics::SOLID),
                )
                .unwrap();
            assert!(hit.distance > 2. && hit.distance < 3.);
            let late = app
                .world_mut()
                .spawn((ChildOf(root), Transform::from_xyz(-0.5, 0.25, 0.)))
                .id();
            app.update();
            let frame = app.world().resource::<Frame>();
            assert!(
                app.world()
                    .get::<GlobalTransform>(late)
                    .unwrap()
                    .translation()
                    .distance(frame.anchored([19.5, 3.25, -17.]))
                    < 0.0001
            );
            assert!(GlobalPosition::new(frame.render.origin().metres()).is_some());
        }
    }
}
