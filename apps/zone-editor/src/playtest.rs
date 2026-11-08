//! Offline capsule sandbox. Uses the existing gameplay solver; no network session.
use super::*;
use avian3d::prelude::Collider;
use theia_core::{coordinates::LocalPosition, time::TickRate};
use theia_network::{
    kinematic::{self, Motion},
    physics::{CAMERA, GROUND, PlacedCollider, SOLID, StaticScene},
    wire::Intent,
};

pub(super) struct Playtest {
    scene: StaticScene,
    pub feet: LocalPosition,
    motion: Motion,
    spawn: LocalPosition,
}
pub(super) fn collision(zone: &Zone) -> Result<StaticScene, String> {
    zone.validate()?;
    let mut colliders = vec![PlacedCollider {
        shape: Collider::cuboid(1024., 1., 1024.),
        center: Vec3::new(0., -0.5, 0.),
        layer: SOLID | GROUND | CAMERA,
    }];
    for object in &zone.objects {
        let [x, y, z] = object.pose.size.map(|v| v as f32);
        let shape = match object.primitive {
            Primitive::Box => Collider::cuboid(x, y, z),
            Primitive::Sphere => Collider::sphere(x / 2.),
            Primitive::Cylinder => Collider::cylinder(x / 2., y),
        };
        let shape = Collider::compound(vec![(
            Vec3::ZERO,
            Quat::from_rotation_y(object.pose.yaw_degrees.to_radians() as f32),
            shape,
        )]);
        colliders.push(PlacedCollider {
            shape,
            center: Vec3::from_array(zone.local(object.pose.position)?),
            layer: SOLID | GROUND | CAMERA,
        });
    }
    Ok(StaticScene::new(colliders))
}
impl Playtest {
    pub fn new(zone: &Zone) -> Result<Self, String> {
        let scene = collision(zone)?;
        let guess = LocalPosition::new(zone.local(zone.spawn)?).ok_or("Invalid spawn")?;
        let spawn =
            kinematic::spawn(&scene, guess).map_err(|e| format!("Playtest spawn refused: {e}"))?;
        Ok(Self {
            scene,
            feet: spawn,
            spawn,
            motion: Motion::default(),
        })
    }
}
pub(super) fn step(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if session.play.is_none() {
        return;
    }
    let mut heading = session.heading;
    heading += (i32::from(keys.pressed(KeyCode::KeyA)) - i32::from(keys.pressed(KeyCode::KeyD)))
        as f32
        * 2.
        / 30.;
    session.heading = heading;
    let Some(play) = session.play.as_mut() else {
        return;
    };
    let forward =
        (i32::from(keys.pressed(KeyCode::KeyW)) - i32::from(keys.pressed(KeyCode::KeyS))) as f32;
    let right =
        (i32::from(keys.pressed(KeyCode::KeyE)) - i32::from(keys.pressed(KeyCode::KeyQ))) as f32;
    let delta = Quat::from_rotation_y(heading) * Vec3::new(right, 0., -forward);
    let intent = Intent {
        axes: [
            (delta.x * 1000.).round() as i16,
            (delta.z * 1000.).round() as i16,
        ],
        jump: keys.pressed(KeyCode::Space),
        ..default()
    };
    match kinematic::advance(
        &play.scene,
        &mut play.motion,
        play.feet,
        intent,
        TickRate::Hz30,
    ) {
        Ok(feet) if feet.metres().iter().all(|v| v.abs() < 500.) && feet.metres()[1] > -100. => {
            play.feet = feet
        }
        _ => {
            play.feet = play.spawn;
            play.motion = Motion::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_rotated_shapes_block_shared_capsule_and_bad_spawn_refuses() {
        let mut zone = Zone::default();
        zone.objects.push(Object {
            id: 1,
            name: "Wall".into(),
            primitive: Primitive::Box,
            pose: Pose {
                position: [0., 1.5, 0.],
                size: [12., 3., 1.],
                yaw_degrees: 15.,
            },
        });
        zone.objects.push(Object {
            id: 2,
            name: "Sphere".into(),
            primitive: Primitive::Sphere,
            pose: Pose {
                position: [8., 1., 0.],
                size: [2.; 3],
                yaw_degrees: 0.,
            },
        });
        zone.objects.push(Object {
            id: 3,
            name: "Pillar".into(),
            primitive: Primitive::Cylinder,
            pose: Pose {
                position: [-8., 1.5, 0.],
                size: [2., 3., 2.],
                yaw_degrees: 0.,
            },
        });
        let mut play = Playtest::new(&zone).unwrap();
        for _ in 0..90 {
            play.feet = kinematic::advance(
                &play.scene,
                &mut play.motion,
                play.feet,
                Intent {
                    axes: [0, -1000],
                    ..default()
                },
                TickRate::Hz30,
            )
            .unwrap();
        }
        let feet = play.feet.metres();
        let wall_local = Quat::from_rotation_y(-15_f32.to_radians()) * Vec3::from_array(feet);
        assert!(
            (0.83..0.9).contains(&wall_local.z),
            "Rotated wall contact {feet:?} -> {wall_local:?}"
        );
        assert!(feet[0] > 0.5, "Expected sliding along the authored wall");
        assert!(feet[1].abs() < 0.02);
        assert!(
            !play
                .scene
                .line_clear(Vec3::new(8., 1., 4.), Vec3::new(8., 1., -4.))
        );
        assert!(
            !play
                .scene
                .line_clear(Vec3::new(-8., 1., 4.), Vec3::new(-8., 1., -4.))
        );
        zone.spawn = [0., 0.1, 0.];
        assert!(Playtest::new(&zone).is_err());
    }
}
