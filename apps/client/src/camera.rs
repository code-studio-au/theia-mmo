use crate::movement::LocalPawn;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

#[derive(Resource)]
pub struct Orbit {
    pub yaw: f32,
    pitch: f32,
    distance: f32,
    max_distance: f32,
    zoom_step: f32,
}
impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.,
            pitch: 7.0f32.atan2(12.),
            distance: 7.0f32.hypot(12.),
            max_distance: 22.,
            zoom_step: 1.,
        }
    }
}
impl Orbit {
    pub fn world_direction(&self, direction: Vec2) -> Vec2 {
        // Normalize before rotating: per-axis wire clamping would otherwise
        // distort diagonals whose rotated component exceeds one.
        let direction = direction.clamp_length_max(1.);
        let rotated = Quat::from_rotation_y(self.yaw) * Vec3::new(direction.x, 0., direction.y);
        Vec2::new(rotated.x, rotated.z)
    }
    fn adjust(&mut self, motion: Vec2, scroll: f32) {
        self.yaw = (self.yaw - motion.x * 0.004 + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        self.pitch = (self.pitch + motion.y * 0.003).clamp(0.12, 1.35);
        self.distance = (self.distance - scroll * self.zoom_step).clamp(2.5, self.max_distance);
    }
    pub fn frame_asset(&mut self, radius: f32) {
        // Default perspective is 45 degrees vertically. Fit the bounding sphere
        // with margin; preserve the source geometry's actual scale and pivot.
        self.distance = (radius.max(1.) / (std::f32::consts::FRAC_PI_8).sin() * 1.15).max(2.5);
        self.max_distance = (self.distance * 3.).max(22.);
        self.zoom_step = (radius * 0.1).max(1.);
        self.yaw = 0.5;
    }
    pub fn set_angles(&mut self, yaw: f32, pitch: f32) {
        self.yaw =
            (yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        self.pitch = pitch.clamp(0.12, 1.35);
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
    pub fn eye(&self, target: Vec3) -> Vec3 {
        let offset = self.unobstructed_eye(target) - target;
        let desired = target + offset;
        target
            + offset * theia_network::world::camera_fraction(target.to_array(), desired.to_array())
    }
}

pub fn controls(
    mut orbit: ResMut<Orbit>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut window: Query<(&Window, &mut CursorOptions)>,
    smoke: Option<Res<crate::smoke::Run>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Some(smoke) = smoke {
        if smoke.world() {
            *orbit = Orbit::default();
            if (264..280).contains(&smoke.frame) {
                orbit.yaw = std::f32::consts::PI;
            }
            if smoke.frame >= 280 {
                orbit.yaw = 0.8;
                orbit.pitch = 0.6;
                orbit.distance = 10.;
            }
        }
        return;
    }
    let Ok((window, mut cursor)) = window.single_mut() else {
        return;
    };
    let rotating = window.focused && buttons.pressed(MouseButton::Right);
    cursor.visible = !rotating;
    cursor.grab_mode = if rotating {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if !window.focused {
        return;
    }
    if keys.just_pressed(KeyCode::Home) {
        *orbit = Orbit::default();
    }
    let lines = scroll.delta.y
        * if scroll.unit == MouseScrollUnit::Pixel {
            1. / 40.
        } else {
            1.
        };
    orbit.adjust(if rotating { motion.delta } else { Vec2::ZERO }, lines);
}

pub fn follow(
    orbit: Res<Orbit>,
    mut pawn: Query<(&Transform, &mut Visibility), With<LocalPawn>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<LocalPawn>)>,
    network: Option<Res<crate::network::Network>>,
) {
    if let (Ok((pawn, mut visible)), Ok(mut camera)) = (pawn.single_mut(), camera.single_mut()) {
        let eye = orbit.eye(pawn.translation);
        *camera = Transform::from_translation(eye).looking_at(pawn.translation, Vec3::Y);
        // Avoid filling the view with the local body when a wall forces the eye in.
        *visible = if eye.distance(pawn.translation) < 1. || network.is_some_and(|n| !n.connected())
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_view_angles_wrap_yaw_and_keep_pitch_in_the_orbit_range() {
        let mut orbit = Orbit::default();
        orbit.set_angles(-std::f32::consts::FRAC_PI_2, 0.9);
        assert!((orbit.angles().x + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert_eq!(orbit.angles().y, 0.9);
        orbit.set_angles(20., -10.);
        assert!(orbit.angles().x.abs() <= std::f32::consts::PI);
        assert_eq!(orbit.angles().y, 0.12);
    }
    #[test]
    fn asset_framing_keeps_large_models_inside_the_zoom_range() {
        let mut orbit = Orbit::default();
        orbit.frame_asset(20.);
        let distance = orbit.distance;
        assert!(distance > 22.);
        orbit.adjust(Vec2::ZERO, 0.);
        assert_eq!(orbit.distance, distance);
        orbit.adjust(Vec2::ZERO, 1e6);
        assert_eq!(orbit.distance, 2.5);
    }
    #[test]
    fn terrain_framing_keeps_the_entire_tile_inside_the_camera_depth_range() {
        let mut orbit = Orbit::default();
        orbit.frame_asset(380.);
        assert!(orbit.distance > 1000.);
        assert!(orbit.asset_far_plane(380.) > orbit.distance + 380.);
        let distance = orbit.distance;
        orbit.adjust(Vec2::ZERO, 1.);
        assert!((distance - orbit.distance - 38.).abs() < 0.001);
        orbit.adjust(Vec2::ZERO, -10000.);
        assert!(orbit.asset_far_plane(380.) > orbit.distance + 380.);
    }
    #[test]
    fn orbit_clamps_pitch_and_zoom_and_rotates_movement() {
        let mut orbit = Orbit::default();
        assert!(
            orbit
                .eye(Vec3::new(0., 0.9, 0.))
                .distance(Vec3::new(0., 7.9, 12.))
                < 0.00001
        );
        orbit.adjust(Vec2::new(1e6, 1e6), 1e6);
        assert_eq!(orbit.pitch, 1.35);
        assert_eq!(orbit.distance, 2.5);
        assert!(orbit.yaw.abs() <= std::f32::consts::PI);
        orbit.adjust(Vec2::new(0., -1e6), -1e6);
        assert_eq!(orbit.pitch, 0.12);
        assert_eq!(orbit.distance, 22.);
        orbit.yaw = std::f32::consts::FRAC_PI_2;
        assert!(orbit.world_direction(Vec2::NEG_Y).distance(Vec2::NEG_X) < 0.00001);
        orbit.yaw = 0.3;
        let diagonal = orbit.world_direction(Vec2::new(1., -1.));
        let expected = Quat::from_rotation_y(0.3) * Vec3::new(1., 0., -1.).normalize();
        assert!(diagonal.distance(Vec2::new(expected.x, expected.z)) < 0.00001);
        assert!((diagonal.length() - 1.).abs() < 0.00001);
    }
    #[test]
    fn mouse_capture_and_focus_loss_use_bevy_cursor_state() {
        let mut app = App::new();
        app.init_resource::<Orbit>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<AccumulatedMouseScroll>()
            .add_systems(Update, controls);
        let entity = app
            .world_mut()
            .spawn((
                Window {
                    focused: true,
                    ..default()
                },
                CursorOptions::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(20., 10.);
        app.update();
        assert_eq!(
            app.world().get::<CursorOptions>(entity).unwrap().grab_mode,
            CursorGrabMode::Locked
        );
        let yaw = app.world().resource::<Orbit>().yaw;
        assert!(yaw < 0.);
        app.world_mut().get_mut::<Window>(entity).unwrap().focused = false;
        app.update();
        assert_eq!(app.world().resource::<Orbit>().yaw, yaw);
        let cursor = app.world().get::<CursorOptions>(entity).unwrap();
        assert!(cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
        app.world_mut().get_mut::<Window>(entity).unwrap().focused = true;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Right);
        let before = app.world().resource::<Orbit>().distance;
        let mut scroll = app.world_mut().resource_mut::<AccumulatedMouseScroll>();
        scroll.unit = MouseScrollUnit::Pixel;
        scroll.delta = Vec2::new(0., 80.);
        app.update();
        assert_eq!(app.world().resource::<Orbit>().distance, before - 2.);
        assert_eq!(app.world().resource::<Orbit>().yaw, yaw);
    }
    #[test]
    fn camera_obstruction_hides_close_body_and_restores_it_after_orbiting() {
        let mut app = App::new();
        app.init_resource::<Orbit>().add_systems(Update, follow);
        let target = Vec3::new(-7., 0.9, -2.54);
        let pawn = app
            .world_mut()
            .spawn((
                LocalPawn,
                Transform::from_translation(target),
                Visibility::Inherited,
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        app.world_mut().resource_mut::<Orbit>().yaw = std::f32::consts::PI;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Hidden
        );
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .translation
                .distance(target)
                < 1.
        );
        app.world_mut().resource_mut::<Orbit>().yaw = 0.;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Inherited
        );
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .translation
                .distance(target)
                > 10.
        );
    }
}
