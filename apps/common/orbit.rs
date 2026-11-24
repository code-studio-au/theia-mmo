//! Shared game/editor orbit geometry and sensitivity; callers own input and collision.
use bevy::prelude::*;

#[derive(Resource)]
pub struct Orbit {
    pub yaw: f32,
    pub(crate) direction_yaw: f32,
    pub(crate) pitch: f32,
    pub(crate) distance: f32,
    pub(crate) max_distance: f32,
    pub(crate) zoom_step: f32,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.,
            direction_yaw: 0.,
            pitch: 7.0f32.atan2(12.),
            distance: 7.0f32.hypot(12.),
            max_distance: 22.,
            zoom_step: 1.,
        }
    }
}
impl Orbit {
    pub fn distance(&self) -> f32 {
        self.distance
    }
    pub fn world_direction(&self, direction: Vec2) -> Vec2 {
        // Normalize before rotating: per-axis wire clamping would otherwise
        // distort diagonals whose rotated component exceeds one.
        let direction = direction.clamp_length_max(1.);
        let rotated =
            Quat::from_rotation_y(self.direction_yaw) * Vec3::new(direction.x, 0., direction.y);
        Vec2::new(rotated.x, rotated.z)
    }
    pub(crate) fn adjust(&mut self, motion: Vec2, scroll: f32) {
        self.yaw = (self.yaw - motion.x * 0.004 + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        self.pitch = (self.pitch + motion.y * 0.003).clamp(-1.2, 1.35);
        self.distance = (self.distance - scroll * self.zoom_step).clamp(2.5, self.max_distance);
    }
    pub fn frame_asset(&mut self, radius: f32) {
        // Default perspective is 45 degrees vertically. Fit the bounding sphere
        // with margin; preserve the source geometry's actual scale and pivot.
        self.distance = (radius.max(1.) / (std::f32::consts::FRAC_PI_8).sin() * 1.15).max(2.5);
        self.max_distance = (self.distance * 3.).max(22.);
        self.zoom_step = (radius * 0.1).max(1.);
        self.yaw = 0.5;
        self.direction_yaw = self.yaw;
    }
    pub fn set_angles(&mut self, yaw: f32, pitch: f32) {
        self.yaw =
            (yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        self.direction_yaw = self.yaw;
        self.pitch = pitch.clamp(-1.2, 1.35);
    }
    pub fn angles(&self) -> Vec2 {
        Vec2::new(self.yaw, self.pitch)
    }
    pub fn unobstructed_eye(&self, target: Vec3) -> Vec3 {
        let offset = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ) * self.distance;
        target + offset
    }
    pub fn asset_far_plane(&self, radius: f32) -> f32 {
        (self.distance + radius * 4.).max(1000.)
    }
    #[cfg(test)]
    pub fn eye(&self, target: Vec3) -> Vec3 {
        let offset = self.unobstructed_eye(target) - target;
        let desired = target + offset;
        target
            + offset * theia_network::world::camera_fraction(target.to_array(), desired.to_array())
    }
}
