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

/// Fixed-height pawn against expanded XZ boxes. Swept axes prevent tunnelling
/// and allow wall sliding. X then Z is deterministic, with square corner clearance.
/// Roads are walkable; overhead roof geometry does not block a standing pawn.
pub fn move_player(mut position: [f32; 2], intent: Intent, seconds: f32) -> [f32; 2] {
    if !intent.valid() || !seconds.is_finite() || seconds < 0. {
        return position;
    }
    let axes = intent.axes.map(|v| f32::from(v) / 1000.);
    let length = axes[0].hypot(axes[1]).max(1.);
    let speed = if intent.sprint { 8. } else { 5. };
    let delta = axes.map(|axis| axis / length * speed * seconds);
    for axis in 0..2 {
        let other = 1 - axis;
        let mut end = (position[axis] + delta[axis]).clamp(-WORLD_LIMIT, WORLD_LIMIT);
        for prop in PROPS {
            let bottom = prop.center[1] - prop.size[1] / 2.;
            let top = prop.center[1] + prop.size[1] / 2.;
            if top <= 0.05 || bottom >= PAWN_HEIGHT {
                continue;
            }
            let c = [prop.center[0], prop.center[2]];
            let half = [
                prop.size[0] / 2. + PAWN_RADIUS,
                prop.size[2] / 2. + PAWN_RADIUS,
            ];
            // A tangent contact must slide freely rather than snag the next axis.
            if position[other] <= c[other] - half[other]
                || position[other] >= c[other] + half[other]
            {
                continue;
            }
            let min = c[axis] - half[axis];
            let max = c[axis] + half[axis];
            if position[axis] <= min && end > min {
                end = end.min(min);
            }
            if position[axis] >= max && end < max {
                end = end.max(max);
            }
        }
        position[axis] = end;
    }
    position
}

/// A small swept camera volume against the same static 3D boxes. Return the
/// unobstructed fraction along the target-to-eye segment; no character collision.
pub fn camera_fraction(target: [f32; 3], eye: [f32; 3]) -> f32 {
    let mut fraction = 1.0f32;
    for prop in PROPS {
        let mut near = 0.0f32;
        let mut far = 1.0f32;
        for axis in 0..3 {
            let half = prop.size[axis] / 2. + 0.15;
            let min = prop.center[axis] - half;
            let max = prop.center[axis] + half;
            let direction = eye[axis] - target[axis];
            if direction.abs() < 1e-7 {
                if target[axis] < min || target[axis] > max {
                    far = -1.;
                    break;
                }
            } else {
                let a = (min - target[axis]) / direction;
                let b = (max - target[axis]) / direction;
                near = near.max(a.min(b));
                far = far.min(a.max(b));
            }
        }
        if near <= far {
            fraction = fraction.min(near);
        }
    }
    fraction
}

#[cfg(test)]
mod tests {
    use super::*;
    fn intent(axes: [i16; 2]) -> Intent {
        Intent {
            axes,
            sprint: false,
        }
    }
    #[test]
    fn closed_door_building_and_rocks_block_sweeps_in_both_directions() {
        assert!((move_player([-7., 0.], intent([0, -1000]), 10.)[1] + 2.54).abs() < 0.00001);
        assert!((move_player([-7., -15.], intent([0, 1000]), 10.)[1] + 11.35).abs() < 0.00001);
        assert!((move_player([-15., -7.], intent([1000, 0]), 10.)[0] + 11.35).abs() < 0.00001);
        assert!((move_player([0., -7.], intent([-1000, 0]), 10.)[0] + 2.65).abs() < 0.00001);
        assert!((move_player([8., 0.], intent([0, 1000]), 10.)[1] - 3.).abs() < 0.00001);
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
