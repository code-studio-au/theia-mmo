//! Shared Avian scene queries for authoritative movement and client clearance.
//! Only approved world geometry is inserted. Characters are never obstacles.
use avian3d::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*, time::TimeUpdateStrategy};
use std::{sync::Mutex, time::Duration};

pub const SOLID: u32 = 1;
pub const GROUND: u32 = 2;
pub const CAMERA: u32 = 4;
pub const AVIAN_REVISION: &str = "1070759b2145dc078abedace91dc7fe78013a45c";

/// Obstacle surface normal and bounded travel distance for kinematic resolution.
#[derive(Clone, Copy, Debug)]
pub struct SweepHit {
    pub distance: f32,
    pub normal: Vec3,
}

#[derive(Clone)]
pub struct PlacedCollider {
    pub shape: Collider,
    pub center: Vec3,
    pub layer: u32,
}
impl PlacedCollider {
    pub fn bundle(&self) -> impl Bundle {
        (
            RigidBody::Static,
            self.shape.clone(),
            Transform::from_translation(self.center),
            // Query membership only: no automatic character/character response.
            CollisionLayers::new(self.layer, 0u32),
        )
    }
}

type QueryState = SystemState<SpatialQuery<'static, 'static>>;

/// An immutable, renderer-free Bevy/Avian world. Its query state is reused rather
/// than constructing an ECS system per cast. The mutex only guards query access.
pub struct StaticScene {
    pub colliders: Vec<PlacedCollider>,
    query: Mutex<(World, QueryState)>,
}

impl StaticScene {
    pub fn new(colliders: Vec<PlacedCollider>) -> Self {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, PhysicsPlugins::default()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )));
        for collider in &colliders {
            app.world_mut().spawn(collider.bundle());
        }
        app.finish();
        app.cleanup();
        // Query-scene startup can run inside another Bevy system. Execute its
        // tiny schedules locally rather than requiring free shared-pool workers.
        for (_, schedule) in app
            .world_mut()
            .resource_mut::<bevy::ecs::schedule::Schedules>()
            .iter_mut()
        {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        }
        // Initialize transforms and static broad phase before the first query.
        app.update();
        app.update();
        let mut world = std::mem::take(app.world_mut());
        let state = QueryState::new(&mut world);
        Self {
            colliders,
            query: Mutex::new((world, state)),
        }
    }

    pub fn ground(&self, p: [f32; 2], ceiling: f32) -> Option<(f32, bool)> {
        let mut data = self.query.lock().expect("Avian query lock");
        let (world, state) = &mut *data;
        let query = state.get(world).expect("Initialized Avian query state");
        let hit = query.cast_ray(
            Vec3::new(p[0], ceiling, p[1]),
            Dir3::NEG_Y,
            4096.,
            false,
            &SpatialQueryFilter::from_mask(GROUND),
        )?;
        Some((
            ceiling - hit.distance,
            hit.normal.y >= 40_f32.to_radians().cos() - 1e-5,
        ))
    }

    pub fn cast(&self, shape: &Collider, origin: Vec3, delta: Vec3, mask: u32) -> f32 {
        let Ok(direction) = Dir3::new(delta) else {
            return 1.;
        };
        let length = delta.length();
        let mut data = self.query.lock().expect("Avian query lock");
        let (world, state) = &mut *data;
        let query = state.get(world).expect("Initialized Avian query state");
        let config = ShapeCastConfig {
            max_distance: length,
            target_distance: 0.0005,
            ignore_origin_penetration: true,
            ..default()
        };
        query
            .cast_shape(
                shape,
                origin,
                Quat::IDENTITY,
                direction,
                &config,
                &SpatialQueryFilter::from_mask(mask),
            )
            .map_or(1., |hit| {
                let distance = (hit.distance - 0.00001).max(0.);
                // Suppress solver-scale contact creep under persistent input.
                if distance < 0.0001 {
                    0.
                } else {
                    (distance / length).min(1.)
                }
            })
    }

    pub fn overlaps(&self, shape: &Collider, position: Vec3, mask: u32) -> bool {
        let mut data = self.query.lock().expect("Avian query lock");
        let (world, state) = &mut *data;
        !state
            .get(world)
            .expect("Initialized Avian query state")
            .shape_intersections(
                shape,
                position,
                Quat::IDENTITY,
                &SpatialQueryFilter::from_mask(mask),
            )
            .is_empty()
    }

    pub fn sweep(
        &self,
        shape: &Collider,
        origin: Vec3,
        delta: Vec3,
        mask: u32,
        skin: f32,
    ) -> Option<SweepHit> {
        let direction = Dir3::new(delta).ok()?;
        let mut data = self.query.lock().expect("Avian query lock");
        let (world, state) = &mut *data;
        let query = state.get(world).expect("Initialized Avian query state");
        let mut closest: Option<SweepHit> = None;
        query.shape_hits_callback(
            shape,
            origin,
            Quat::IDENTITY,
            direction,
            &ShapeCastConfig {
                max_distance: delta.length(),
                target_distance: skin,
                ignore_origin_penetration: false,
                compute_contact_on_penetration: true,
            },
            &SpatialQueryFilter::from_mask(mask),
            |hit| {
                // At exact initial contact, some convex CCD normals are noisy.
                // Query the same obstacle from a short backtracked pose to get
                // an impact normal before projecting motion. This moves only
                // the query origin, never the character or another collider.
                let mut normal = if hit.distance < 0.0001 {
                    let backtrack = skin * 8.;
                    query
                        .cast_shape_predicate(
                            shape,
                            origin + hit.normal1 * backtrack,
                            Quat::IDENTITY,
                            Dir3::new(-hit.normal1).unwrap_or(-direction),
                            &ShapeCastConfig {
                                max_distance: backtrack + skin * 2.,
                                target_distance: skin,
                                ignore_origin_penetration: false,
                                compute_contact_on_penetration: true,
                            },
                            &SpatialQueryFilter::from_mask(mask),
                            &|entity| entity == hit.entity,
                        )
                        .filter(|refined| refined.distance > 0.001)
                        .map_or(hit.normal1, |refined| refined.normal1)
                } else {
                    hit.normal1
                };
                // CCD can conservatively stop short of the requested skin.
                // Resolve downward support and head-on wall distances with
                // signed contact at the candidate pose. In particular, changing
                // support height must not turn wall-distance noise into creep.
                // Keep glancing contacts and edge-feature changes conservative,
                // and never extend a hit beyond this sweep.
                let mut distance = hit.distance.max(0.).min(delta.length());
                let support = direction.y < -0.999 && normal.y >= 0.76604444;
                let wall = normal.y.abs() < 0.76604444 && normal.dot(*direction) < -0.99;
                if support || wall {
                    let collider = world.get::<Collider>(hit.entity).unwrap();
                    let position = world.get::<Position>(hit.entity).unwrap().0;
                    let rotation = world.get::<Rotation>(hit.entity).unwrap().0;
                    if let Ok(Some(contact)) = avian3d::collision::collider::contact_query::contact(
                        collider,
                        position,
                        rotation,
                        shape,
                        origin + *direction * distance,
                        Quat::IDENTITY,
                        0.05,
                    ) {
                        let mut up = rotation * contact.local_normal1;
                        let mut penetration = contact.penetration;
                        let mut wall_manifold = false;
                        if wall {
                            // A single support-map contact can have the same
                            // noisy normal as CCD. The feature manifold gives
                            // stable face contacts without quantizing axes or
                            // suppressing intentional small tangential input.
                            let mut manifolds = Vec::new();
                            avian3d::collision::collider::contact_query::contact_manifolds(
                                collider,
                                position,
                                rotation,
                                shape,
                                origin + *direction * distance,
                                Quat::IDENTITY,
                                0.05,
                                &mut manifolds,
                            );
                            if let Some((manifold, point)) = manifolds
                                .iter()
                                .filter(|m| m.normal.dot(normal) > 0.99)
                                .filter_map(|m| m.find_deepest_contact().map(|p| (m, p)))
                                .max_by(|a, b| a.1.penetration.total_cmp(&b.1.penetration))
                            {
                                up = manifold.normal;
                                penetration = point.penetration;
                                wall_manifold = true;
                            }
                        }
                        let approach = -up.dot(*direction);
                        if (support && up.y >= 0.76604444) || (wall_manifold && approach > 0.1) {
                            let correction = (-penetration - skin) / approach;
                            if correction.abs() <= 0.02 {
                                distance = (distance + correction).clamp(0., delta.length());
                                if wall {
                                    normal = up;
                                }
                            }
                        }
                    }
                }
                // Initial tangencies and separating contacts must not obscure
                // another blocking collider, or prevent retreat along a wall.
                if normal.dot(*direction) < -0.00001
                    && closest.is_none_or(|old| distance < old.distance)
                {
                    closest = Some(SweepHit { distance, normal });
                }
                true
            },
        );
        closest
    }

    /// Binary GJK intersections can conservatively include near-separated shapes.
    /// Resolve only those BVH candidates with signed contact depth before
    /// rejecting a kinematic pose. Unsupported pairs fail closed.
    pub fn penetrates(&self, shape: &Collider, position: Vec3, mask: u32, tolerance: f32) -> bool {
        let mut data = self.query.lock().expect("Avian query lock");
        let (world, state) = &mut *data;
        let query = state.get(world).expect("Initialized Avian query state");
        query
            .shape_intersections(
                shape,
                position,
                Quat::IDENTITY,
                &SpatialQueryFilter::from_mask(mask),
            )
            .into_iter()
            .any(|entity| {
                let collider = world.get::<Collider>(entity).unwrap();
                let body_position = world.get::<Position>(entity).unwrap().0;
                let rotation = world.get::<Rotation>(entity).unwrap().0;
                match avian3d::collision::collider::contact_query::contact(
                    collider,
                    body_position,
                    rotation,
                    shape,
                    position,
                    Quat::IDENTITY,
                    0.,
                ) {
                    Ok(Some(contact)) => contact.penetration > tolerance,
                    Ok(None) => false,
                    Err(_) => true,
                }
            })
    }

    pub fn camera_fraction(&self, target: [f32; 3], eye: [f32; 3]) -> f32 {
        let origin = Vec3::from_array(target);
        self.cast(
            &Collider::cuboid(0.3, 0.3, 0.3),
            origin,
            Vec3::from_array(eye) - origin,
            GROUND | CAMERA,
        )
    }
}

pub fn character_shape(clearance: f32) -> Collider {
    Collider::capsule(
        crate::world::PAWN_RADIUS - clearance,
        crate::world::PAWN_HEIGHT - 0.05 - 2. * crate::world::PAWN_RADIUS,
    )
}
pub fn character_center(p: [f32; 2], ground: f32) -> Vec3 {
    Vec3::new(p[0], ground + (crate::world::PAWN_HEIGHT + 0.05) / 2., p[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn avian_casts_stop_at_thin_walls_allow_retreat_and_filter_layers() {
        let scene = StaticScene::new(vec![PlacedCollider {
            shape: Collider::cuboid(0.02, 4., 8.),
            center: Vec3::ZERO,
            layer: SOLID,
        }]);
        let shape = character_shape(0.);
        let origin = Vec3::new(-2., 0., 0.);
        let fraction = scene.cast(&shape, origin, Vec3::X * 4., SOLID);
        assert!(fraction > 0.4 && fraction < 0.42);
        let contact = origin + Vec3::X * 4. * fraction;
        assert_eq!(scene.cast(&shape, contact, Vec3::NEG_X, SOLID), 1.);
        assert_eq!(scene.cast(&shape, origin, Vec3::X * 4., GROUND), 1.);
        assert!(scene.overlaps(&shape, Vec3::ZERO, SOLID));
    }
}
