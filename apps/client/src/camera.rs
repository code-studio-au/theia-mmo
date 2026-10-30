use crate::movement::LocalPawn;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility},
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::{sync::OnceLock, time::Instant};

/// Chunk worlds use authored camera-query geometry; legacy proxy maps retain
/// additional visual-mesh refinement.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionMode {
    World,
    Visual,
}
impl CollisionMode {
    pub fn parse(args: &mut Vec<String>) -> Result<Option<Self>, &'static str> {
        let Some(index) = args.iter().position(|s| s == "--camera-collision") else {
            return Ok(None);
        };
        let mode = match args.get(index + 1).map(String::as_str) {
            Some("world") => Self::World,
            Some("visual") => Self::Visual,
            _ => return Err("--camera-collision requires world or visual"),
        };
        args.drain(index..index + 2);
        if args.iter().any(|s| s == "--camera-collision") {
            return Err("Camera collision mode is repeated");
        }
        Ok(Some(mode))
    }
}

#[derive(Resource, Default)]
pub struct SceneryCameraStats {
    pub checks: u64,
    pub retractions: u64,
    timings: Option<CameraTimings>,
}

struct CameraTimings {
    previous: Instant,
    reported: Instant,
    frame_ms: Vec<f64>,
    mesh_ms: Vec<f64>,
}

fn performance_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("THEIA_PERFORMANCE_LOG").is_some())
}

impl SceneryCameraStats {
    fn record_timing(&mut self, started: Option<Instant>, world_only: bool) {
        let Some(started) = started else {
            return;
        };
        let now = Instant::now();
        let timer = self.timings.get_or_insert_with(|| CameraTimings {
            previous: now,
            reported: now,
            frame_ms: Vec::new(),
            mesh_ms: Vec::new(),
        });
        timer
            .frame_ms
            .push(now.duration_since(timer.previous).as_secs_f64() * 1000.);
        timer
            .mesh_ms
            .push(now.duration_since(started).as_secs_f64() * 1000.);
        timer.previous = now;
        if now.duration_since(timer.reported).as_secs_f64() >= 1. {
            eprintln!(
                "{}",
                serde_json::json!({"camera_performance":{
                "world_only":world_only,
                "frame_ms":std::mem::take(&mut timer.frame_ms),
                "mesh_ms":std::mem::take(&mut timer.mesh_ms)}})
            );
            timer.reported = now;
        }
    }
}
type SceneryMaterials<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static MeshMaterial3d<StandardMaterial>>,
        Option<&'static MeshMaterial3d<crate::triplanar::Material>>,
    ),
    With<crate::terrain_scene::CameraScenery>,
>;

/// Refine the existing conservative camera path against actual static scenery.
/// This presentation query never supplies movement or gameplay collision.
pub fn scenery_clearance(
    mut ray_cast: MeshRayCast,
    mut stats: ResMut<SceneryCameraStats>,
    scenery: SceneryMaterials,
    materials: Res<Assets<StandardMaterial>>,
    (projected, mode): (
        Option<Res<Assets<crate::triplanar::Material>>>,
        Option<Res<CollisionMode>>,
    ),
    mut pawn: Query<(&Transform, &mut Visibility), With<LocalPawn>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<LocalPawn>)>,
) {
    if scenery.is_empty() {
        return;
    }
    let timing = performance_enabled().then(Instant::now);
    if mode.is_some_and(|m| *m == CollisionMode::World) {
        stats.record_timing(timing, true);
        return;
    }
    stats.checks += 1;
    let (Ok((pawn, mut visible)), Ok(mut camera)) = (pawn.single_mut(), camera.single_mut()) else {
        return;
    };
    let target = pawn.translation;
    let delta = camera.translation - target;
    let Ok(direction) = Dir3::new(delta) else {
        return;
    };
    let distance = delta.length();
    let filter = |entity| {
        scenery.get(entity).is_ok_and(|(standard, extension)| {
            let alpha = standard
                .and_then(|handle| materials.get(&handle.0).map(|m| m.alpha_mode))
                .or_else(|| {
                    extension.and_then(|handle| {
                        projected
                            .as_ref()?
                            .get(&handle.0)
                            .map(|m| m.base.alpha_mode)
                    })
                });
            alpha.is_some_and(|mode| matches!(mode, AlphaMode::Opaque | AlphaMode::Mask(_)))
        })
    };
    let settings = MeshRayCastSettings::default()
        .with_visibility(RayCastVisibility::Visible)
        .with_filter(&filter);
    let mut clear = distance;
    // A small cross of parallel probes protects the camera near plane; source
    // alpha-cutout triangles are conservatively solid for these geometry queries.
    let right = direction.cross(Vec3::Y).normalize_or_zero() * 0.12;
    let up = right.cross(*direction).normalize_or_zero() * 0.12;
    for offset in [Vec3::ZERO, right, -right, up, -up] {
        if let Some((_, hit)) = ray_cast
            .cast_ray(Ray3d::new(target + offset, direction), &settings)
            .first()
        {
            clear = clear.min((hit.distance - 0.18).max(0.));
        }
    }
    if clear < distance {
        stats.retractions += 1;
        *camera =
            Transform::from_translation(target + *direction * clear).looking_at(target, Vec3::Y);
        if clear < 1. {
            *visible = Visibility::Hidden;
        }
    }
    stats.record_timing(timing, false);
}

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

#[allow(clippy::too_many_arguments)]
pub fn controls(
    mut orbit: ResMut<Orbit>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut window: Query<(&Window, &mut CursorOptions)>,
    smoke: Option<Res<crate::smoke::Run>>,
    keys: Res<ButtonInput<KeyCode>>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    inventory: Option<Res<crate::inventory::Ui>>,
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
    let lines = if dialogue.as_ref().is_some_and(|ui| ui.open)
        || inventory.as_ref().is_some_and(|ui| ui.open)
    {
        0.
    } else {
        scroll.delta.y
            * if scroll.unit == MouseScrollUnit::Pixel {
                1. / 40.
            } else {
                1.
            }
    };
    orbit.adjust(if rotating { motion.delta } else { Vec2::ZERO }, lines);
}

pub fn follow(
    orbit: Res<Orbit>,
    mut pawn: Query<(&Transform, &mut Visibility), With<LocalPawn>>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<LocalPawn>)>,
    network: Option<Res<crate::network::Network>>,
    terrain: Option<Res<crate::terrain_scene::Config>>,
    (chunks, chunks_ready): (
        Option<Res<crate::chunk_world::Config>>,
        Option<Res<crate::chunk_world::Ready>>,
    ),
    (frame, graybox): (
        Option<Res<crate::floating_origin::Frame>>,
        Option<Res<crate::graybox::Graybox>>,
    ),
) {
    if let (Ok((pawn, mut visible)), Ok(mut camera)) = (pawn.single_mut(), camera.single_mut()) {
        let desired = if network.as_ref().is_some_and(|n| n.camera_test_blocked()) {
            pawn.translation + Vec3::new(0., 7., -12.)
        } else {
            orbit.unobstructed_eye(pawn.translation)
        };
        let content = |p| {
            if graybox.is_some() {
                p
            } else {
                frame.as_ref().map_or(p, |f| f.content_point(p))
            }
        };
        let target = content(pawn.translation);
        let desired_content = content(desired);
        let eye = if let (Some(gray), Some(frame)) = (&graybox, &frame) {
            pawn.translation
                + (desired - pawn.translation)
                    * gray.camera_fraction(frame, pawn.translation, desired)
        } else if let Some(chunks) = &chunks {
            pawn.translation
                + (desired - pawn.translation)
                    * chunks
                        .scene()
                        .camera_fraction(target.to_array(), desired_content.to_array())
        } else if let Some(terrain) = terrain {
            let fraction = terrain
                .terrain
                .camera_fraction(target.to_array(), desired_content.to_array());
            pawn.translation + (desired - pawn.translation) * fraction
        } else {
            pawn.translation + (orbit.eye(target) - target)
        };
        *camera = Transform::from_translation(eye).looking_at(pawn.translation, Vec3::Y);
        // Avoid filling the view with the local body when a wall forces the eye in.
        *visible = if eye.distance(pawn.translation) < 1.
            || network.is_some_and(|n| !n.connected())
            || (chunks.is_some() && chunks_ready.is_none())
        {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// The camera is a root entity. Mesh clearance runs after propagation, so write
/// its final global pose explicitly before extraction without repropagating bones.
type RootCamera = (With<Camera3d>, Without<ChildOf>);
pub fn sync_camera_global(mut cameras: Query<(&Transform, &mut GlobalTransform), RootCamera>) {
    for (transform, mut global) in &mut cameras {
        *global = GlobalTransform::from(*transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn world_camera_keeps_accelerated_wall_clearance_without_visual_mesh_scans() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(CollisionMode::World)
            .init_resource::<SceneryCameraStats>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Orbit>()
            .add_systems(Update, (follow, scenery_clearance).chain());
        let target = Vec3::new(-7., 0.9, -2.54);
        app.world_mut().resource_mut::<Orbit>().yaw = std::f32::consts::PI;
        let expected = app.world().resource::<Orbit>().eye(target);
        assert!(expected.distance(target) > 0. && expected.distance(target) < 1.);
        app.world_mut().spawn((
            LocalPawn,
            Transform::from_translation(target),
            Visibility::Inherited,
        ));
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::default()))
            .id();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(10., 10., 10.));
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        app.world_mut().spawn((
            crate::terrain_scene::CameraScenery,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            bevy::camera::primitives::Aabb::from_min_max(Vec3::splat(-5.), Vec3::splat(5.)),
            GlobalTransform::from_translation(target),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ));
        app.update();
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .translation
                .distance(expected)
                < 0.0001
        );
        assert_eq!(app.world().resource::<SceneryCameraStats>().checks, 0);
    }
    #[test]
    fn scenery_ray_cast_retracts_at_real_geometry_ignores_actors_and_translucency() {
        use crate::terrain_scene::CameraScenery;
        use bevy::camera::primitives::Aabb;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SceneryCameraStats>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Update, scenery_clearance);
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(4., 4., 0.2));
        let opaque = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let translucent = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                alpha_mode: AlphaMode::Blend,
                ..default()
            });
        let pawn = app
            .world_mut()
            .spawn((
                LocalPawn,
                Transform::from_xyz(0., 1., 0.),
                Visibility::Inherited,
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Transform::from_xyz(0., 1., 12.)))
            .id();
        let obstacle = app
            .world_mut()
            .spawn((
                CameraScenery,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(opaque.clone()),
                Aabb::from_min_max(Vec3::new(-2., -2., -0.1), Vec3::new(2., 2., 0.1)),
                GlobalTransform::from(Transform::from_xyz(0., 1., 4.)),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .id();
        // A nearer animated actor and translucent scenery must not obstruct.
        let actor = app
            .world_mut()
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(opaque),
                Aabb::from_min_max(Vec3::new(-2., -2., -0.1), Vec3::new(2., 2., 0.1)),
                GlobalTransform::from(Transform::from_xyz(0., 1., 1.)),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .id();
        app.world_mut().spawn((
            CameraScenery,
            Mesh3d(mesh),
            MeshMaterial3d(translucent),
            Aabb::from_min_max(Vec3::new(-2., -2., -0.1), Vec3::new(2., 2., 0.1)),
            GlobalTransform::from(Transform::from_xyz(0., 1., 2.)),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ));
        app.update();
        let z = app.world().get::<Transform>(camera).unwrap().translation.z;
        assert!(
            (z - 3.72).abs() < 1e-4,
            "Camera retracts before the actual mesh: {z}"
        );
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Inherited
        );
        // Move the static wall away; the actor remains in the path, without the
        // static marker. Rebuild the desired orbit as follow() does every frame.
        app.world_mut().entity_mut(obstacle).despawn();
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation
            .z = 12.;
        app.update();
        assert_eq!(
            app.world().get::<Transform>(camera).unwrap().translation.z,
            12.
        );
        // Add a genuinely close static obstacle and require body hiding.
        app.world_mut().entity_mut(actor).insert(CameraScenery);
        app.update();
        assert!(app.world().get::<Transform>(camera).unwrap().translation.z < 1.);
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Hidden
        );
    }
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
    #[test]
    fn rebased_camera_queries_content_collision_without_changing_global_clearance() {
        use crate::floating_origin::Frame;
        use theia_core::coordinates::LocalPosition;
        let mut frame = Frame::default();
        frame.span = 2.;
        frame.admit(&theia_network::wire::Welcome {
            terrain_stream: false,
            id: 1,
            epoch: 1,
            world: [0; 32],
            origin: [1e9 + 0.01, 1e9 + 0.02, -1e9 - 0.03],
        });
        let global = frame
            .content
            .to_global(LocalPosition::new([-7., 0.9, -2.54]).unwrap())
            .unwrap();
        frame.focus(global);
        let mut app = App::new();
        let target = frame.project(global);
        app.insert_resource(frame)
            .init_resource::<Orbit>()
            .add_systems(Update, follow);
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
        let before = app.world().get::<Transform>(camera).unwrap().translation - target;
        assert!(before.length() < 1.);
        {
            let mut frame = app.world_mut().resource_mut::<Frame>();
            let focus = frame
                .content
                .to_global(LocalPosition::new([10., 0.9, 10.]).unwrap())
                .unwrap();
            frame.focus(focus);
        }
        let target = app.world().resource::<Frame>().project(global);
        app.world_mut()
            .get_mut::<Transform>(pawn)
            .unwrap()
            .translation = target;
        app.update();
        let after = app.world().get::<Transform>(camera).unwrap().translation - target;
        assert!(before.distance(after) < 0.0001);
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().resource_mut::<Orbit>().yaw = 0.;
        app.update();
        assert!(
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .translation
                .distance(target)
                > 10.
        );
        assert_eq!(
            *app.world().get::<Visibility>(pawn).unwrap(),
            Visibility::Inherited
        );
    }
}
