//! Shared authored graybox. Environment geometry is immutable; characters never
//! enter the collision set. Flat-ground clearance is conservative, not full physics.
use crate::wire::{Intent, WORLD_LIMIT};

pub const PAWN_RADIUS: f32 = 0.35;
pub const PAWN_HEIGHT: f32 = 1.8;
pub const PAWN_Y: f32 = PAWN_HEIGHT / 2.;

#[derive(Clone, Copy)]
pub enum Surface {
    Road,
    Stone,
    Wood,
}
#[derive(Clone, Copy)]
pub struct Prop {
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub surface: Surface,
}
pub const PROPS: [Prop; 7] = [
    Prop {
        center: [0., 0.015, 0.],
        size: [3., 0.03, 40.],
        surface: Surface::Road,
    },
    Prop {
        center: [0., 0.016, -5.],
        size: [40., 0.03, 3.],
        surface: Surface::Road,
    },
    Prop {
        center: [-7., 3.2, -7.],
        size: [8., 6.4, 8.],
        surface: Surface::Stone,
    },
    Prop {
        center: [-7., 6.6, -7.],
        size: [8.6, 0.4, 8.6],
        surface: Surface::Wood,
    },
    Prop {
        center: [-7., 0.9, -2.95],
        size: [1.6, 1.8, 0.12],
        surface: Surface::Wood,
    },
    Prop {
        center: [8., 0.6, 4.],
        size: [1.8, 1.2, 1.3],
        surface: Surface::Stone,
    },
    Prop {
        center: [10., 0.4, 6.],
        size: [1.2, 0.8, 1.8],
        surface: Surface::Stone,
    },
];

/// Avian world queries shared by the offline client and authoritative graybox.
/// Actors are deliberately absent from the obstacle set.
pub fn physics_scene() -> &'static crate::physics::StaticScene {
    static SCENE: std::sync::OnceLock<crate::physics::StaticScene> = std::sync::OnceLock::new();
    if let Some(scene) = SCENE.get() {
        return scene;
    }
    // Build outside the OnceLock initializer: Bevy startup can execute other
    // callers on its shared pool. Holding the initializer across that work can
    // deadlock those callers (including recursive work-stealing on this thread).
    // Concurrent cold callers may build this tiny scene more than once; only one
    // complete immutable scene is published and all callers use that winner.
    let scene = {
        use crate::physics::{CAMERA, GROUND, PlacedCollider, SOLID, StaticScene};
        use avian3d::prelude::Collider;
        use bevy::prelude::Vec3;
        let mut colliders: Vec<_> = PROPS
            .iter()
            .map(|p| PlacedCollider {
                shape: Collider::cuboid(p.size[0], p.size[1], p.size[2]),
                center: Vec3::from_array(p.center),
                layer: SOLID | CAMERA,
            })
            .collect();
        colliders.push(PlacedCollider {
            shape: Collider::cuboid(40., 0.1, 40.),
            center: Vec3::new(0., -0.05, 0.),
            layer: GROUND,
        });
        StaticScene::new(colliders)
    };
    SCENE.get_or_init(|| scene)
}

/// Fixed-height sandbox controller with Avian capsule sweeps and axis sliding.
/// Its flat-ground contract is retained; it does not implement gravity/jumping.
pub fn move_player(mut position: [f32; 2], intent: Intent, seconds: f32) -> [f32; 2] {
    use crate::physics::{self, SOLID};
    use bevy::prelude::Vec3;
    if !intent.valid()
        || !seconds.is_finite()
        || seconds < 0.
        || position.iter().any(|v| !v.is_finite())
    {
        return position;
    }
    let axes = intent.axes.map(|v| f32::from(v) / 1000.);
    let length = axes[0].hypot(axes[1]).max(1.);
    let speed = if intent.sprint { 8. } else { 5. };
    let delta = axes.map(|axis| axis / length * speed * seconds);
    let shape = physics::character_shape(0.);
    for axis in 0..2 {
        let end = (position[axis] + delta[axis]).clamp(-WORLD_LIMIT, WORLD_LIMIT);
        let mut next = position;
        next[axis] = end;
        let fraction = physics_scene().cast(
            &shape,
            physics::character_center(position, 0.),
            Vec3::new(next[0] - position[0], 0., next[1] - position[1]),
            SOLID,
        );
        position[axis] += (end - position[axis]) * fraction;
    }
    position
}

pub fn camera_fraction(target: [f32; 3], eye: [f32; 3]) -> f32 {
    physics_scene().camera_fraction(target, eye)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn intent(axes: [i16; 2]) -> Intent {
        Intent {
            jump: false,
            axes,
            sprint: false,
        }
    }
    #[test]
    fn closed_door_building_and_rocks_block_sweeps_in_both_directions() {
        assert!((move_player([-7., 0.], intent([0, -1000]), 10.)[1] + 2.54).abs() < 0.001);
        assert!((move_player([-7., -15.], intent([0, 1000]), 10.)[1] + 11.35).abs() < 0.001);
        assert!((move_player([-15., -7.], intent([1000, 0]), 10.)[0] + 11.35).abs() < 0.001);
        assert!((move_player([0., -7.], intent([-1000, 0]), 10.)[0] + 2.65).abs() < 0.001);
        assert!((move_player([8., 0.], intent([0, 1000]), 10.)[1] - 3.).abs() < 0.001);
    }
    #[test]
    fn walls_allow_sliding_and_retreat_without_entering_solid_geometry() {
        let start = [-2.65, -7.];
        let slid = move_player(start, intent([-1000, 1000]), 0.1);
        assert_eq!(slid[0], start[0]);
        assert!(slid[1] > start[1]);
        let away = move_player(start, intent([1000, 0]), 0.1);
        assert!(away[0] > start[0]);
        assert_eq!(move_player([0., 0.], intent([0, -1000]), 1.), [0., -5.]);
    }
    #[test]
    fn camera_retracts_at_static_geometry_and_restores_in_clear_space() {
        let blocked = camera_fraction([-7., 0.9, -2.54], [-7., 7.9, -14.54]);
        assert!(blocked > 0. && blocked < 0.1);
        assert_eq!(camera_fraction([0., 0.9, 0.], [0., 7.9, 12.]), 1.);
        // Roof is included for the camera, even though overhead pawn movement ignores it.
        assert!(camera_fraction([-13., 7., -7.], [0., 5., -7.]) < 1.);
    }
}
