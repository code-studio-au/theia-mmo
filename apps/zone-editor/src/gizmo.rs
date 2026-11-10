//! World-axis move and yaw rotation handles built from standard Bevy meshes and ray queries.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Tool {
    #[default]
    Move,
    Rotate,
    Scale,
}
#[derive(Clone, Copy)]
pub(super) enum Gesture {
    Plane(PlaneDrag),
    Move(AxisDrag),
    Rotate(YawDrag),
    Scale(ScaleDrag),
}
impl Gesture {
    pub fn with_grid(mut self, grid: snapping::Grid) -> Self {
        match &mut self {
            Self::Plane(d) => d.grid = grid,
            Self::Move(d) => d.grid = grid,
            _ => {}
        }
        self
    }
    pub fn begin(
        tool: Tool,
        ray: Ray3d,
        anchor: Vec3,
        axis: usize,
        reference: f32,
    ) -> Option<Self> {
        match tool {
            Tool::Move => AxisDrag::begin(ray, anchor, axis).map(Self::Move),
            Tool::Rotate => YawDrag::begin(ray, anchor).map(Self::Rotate),
            Tool::Scale => (reference.is_finite() && reference > 0.01)
                .then_some(reference)
                .and_then(|reference| {
                    Some(Self::Scale(ScaleDrag {
                        axis: AxisDrag::begin(ray, anchor, 1)?,
                        reference: f64::from(reference),
                    }))
                }),
        }
    }
    pub fn plane(ray: Ray3d, anchor: Vec3) -> Option<Self> {
        Some(Self::Plane(PlaneDrag {
            height: anchor.y,
            start: plane_point(ray, anchor.y)?,
            grid: default(),
        }))
    }
    pub fn tool(self) -> Tool {
        match self {
            Self::Plane(_) | Self::Move(_) => Tool::Move,
            Self::Rotate(_) => Tool::Rotate,
            Self::Scale(_) => Tool::Scale,
        }
    }
    pub fn axis(self) -> Option<usize> {
        match self {
            Self::Plane(_) => None,
            Self::Move(d) => Some(d.axis),
            Self::Rotate(_) | Self::Scale(_) => Some(1),
        }
    }
    fn scale_extent(self, before: &Pose, preview: &Pose) -> Option<f32> {
        match self {
            Self::Scale(d) => {
                Some((d.reference * preview.size[0] / before.size[0]).clamp(0.1, 2048.) as f32)
            }
            _ => None,
        }
    }
    pub fn preview(&mut self, ray: Ray3d, before: &Pose, snap: bool) -> Option<Pose> {
        let mut pose = before.clone();
        match self {
            Self::Plane(d) => {
                let delta = plane_point(ray, d.height)? - d.start;
                if delta.x.hypot(delta.z) < 0.00001 {
                    return Some(pose);
                }
                for axis in [0, 2] {
                    pose.position[axis] += f64::from(delta[axis]);
                    if snap {
                        pose.position[axis] = d.grid.coordinate(pose.position[axis], axis)?;
                    }
                }
            }
            Self::Move(d) => pose.position = d.position(ray, before.position, snap)?,
            Self::Rotate(d) => pose.yaw_degrees = d.yaw(ray, before.yaw_degrees, snap)?,
            Self::Scale(d) => {
                let delta = f64::from(parameter(ray, d.axis.anchor, 1)? - d.axis.start);
                if delta.abs() < 0.00001 {
                    return Some(pose);
                }
                let factor = 1. + delta / d.reference;
                let factor = if snap {
                    (factor * 10.).round() / 10.
                } else {
                    factor
                };
                pose.size = before.size.map(|v| v * factor);
            }
        }
        Some(pose)
    }
}
#[derive(Clone, Copy)]
pub(super) struct ScaleDrag {
    axis: AxisDrag,
    reference: f64,
}
#[derive(Clone, Copy)]
pub(super) struct YawDrag {
    anchor: Vec3,
    last: f32,
    total: f64,
}
#[derive(Clone, Copy)]
pub(super) struct PlaneDrag {
    grid: snapping::Grid,
    height: f32,
    start: Vec3,
}
fn plane_point(ray: Ray3d, height: f32) -> Option<Vec3> {
    if ray.direction.y.abs() < 0.00001 {
        return None;
    }
    let distance = (height - ray.origin.y) / ray.direction.y;
    (0. ..4096.)
        .contains(&distance)
        .then(|| ray.get_point(distance))
}
fn angle(ray: Ray3d, anchor: Vec3) -> Option<f32> {
    let offset = plane_point(ray, anchor.y)? - anchor;
    (offset.x.hypot(offset.z) > 0.01).then(|| (-offset.z).atan2(offset.x))
}

impl YawDrag {
    fn begin(ray: Ray3d, anchor: Vec3) -> Option<Self> {
        Some(Self {
            anchor,
            last: angle(ray, anchor)?,
            total: 0.,
        })
    }
    fn yaw(&mut self, ray: Ray3d, before: f64, snap: bool) -> Option<f64> {
        let current = angle(ray, self.anchor)?;
        let delta = current - self.last;
        self.total += f64::from(delta.sin().atan2(delta.cos()));
        self.last = current;
        if self.total.abs() < 0.000001 {
            return Some(before);
        }
        let yaw = before + self.total.to_degrees();
        Some(if snap { (yaw / 15.).round() * 15. } else { yaw })
    }
}
#[derive(Component)]
pub(super) struct TransformHandle {
    pub axis: usize,
    pub head: bool,
    pub tool: Tool,
}
#[derive(Resource)]
pub(super) struct Scene {
    colours: [Handle<StandardMaterial>; 3],
    active: Handle<StandardMaterial>,
    uniform: Handle<StandardMaterial>,
}
#[derive(Clone, Copy)]
pub(super) struct AxisDrag {
    grid: snapping::Grid,
    pub axis: usize,
    anchor: Vec3,
    start: f32,
}
/// Closest point on an infinite world axis to a forward view ray. Near-parallel
/// views have no useful drag plane and deliberately refuse to begin/update.
fn parameter(ray: Ray3d, anchor: Vec3, axis: usize) -> Option<f32> {
    let direction = Vec3::AXES[axis];
    let dot = ray.direction.dot(direction);
    let denominator = 1. - dot * dot;
    if denominator < 0.0001 {
        return None;
    }
    let offset = ray.origin - anchor;
    let value = (offset.dot(direction) - dot * offset.dot(*ray.direction)) / denominator;
    let distance = dot * value - offset.dot(*ray.direction);
    (value.is_finite() && (0. ..4096.).contains(&distance)).then_some(value)
}
impl AxisDrag {
    pub fn begin(ray: Ray3d, anchor: Vec3, axis: usize) -> Option<Self> {
        Some(Self {
            axis,
            anchor,
            start: parameter(ray, anchor, axis)?,
            grid: default(),
        })
    }
    pub fn position(self, ray: Ray3d, before: [f64; 3], snap: bool) -> Option<[f64; 3]> {
        let delta = f64::from(parameter(ray, self.anchor, self.axis)? - self.start);
        if delta.abs() < 0.00001 {
            return Some(before);
        }
        let mut position = before;
        position[self.axis] += delta;
        if snap {
            position[self.axis] = self.grid.coordinate(position[self.axis], self.axis)?;
        }
        Some(position)
    }
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = |colour: Color| StandardMaterial {
        base_color: colour,
        unlit: true,
        ..default()
    };
    let colours = [
        Color::srgb(0.95, 0.16, 0.12),
        Color::srgb(0.15, 0.9, 0.24),
        Color::srgb(0.15, 0.45, 1.),
    ]
    .map(|c| materials.add(material(c)));
    let active = materials.add(material(Color::WHITE));
    let shaft = meshes.add(Cylinder::new(1., 1.));
    let head = meshes.add(Cone::new(1., 1.));
    for (axis, colour) in colours.iter().enumerate() {
        for (is_head, mesh) in [(false, shaft.clone()), (true, head.clone())] {
            commands.spawn((
                TransformHandle {
                    axis,
                    head: is_head,
                    tool: Tool::Move,
                },
                Mesh3d(mesh),
                MeshMaterial3d(colour.clone()),
                Transform::default(),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
            ));
        }
    }
    commands.spawn((
        TransformHandle {
            axis: 1,
            head: true,
            tool: Tool::Rotate,
        },
        Mesh3d(meshes.add(Torus::new(0.95, 1.05))),
        MeshMaterial3d(colours[1].clone()),
        Transform::default(),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
    let uniform = materials.add(material(Color::srgb(0.65, 0.85, 1.)));
    for (is_head, mesh) in [(false, shaft), (true, meshes.add(Cuboid::new(1., 1., 1.)))] {
        commands.spawn((
            TransformHandle {
                axis: 1,
                head: is_head,
                tool: Tool::Scale,
            },
            Mesh3d(mesh),
            MeshMaterial3d(uniform.clone()),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
    commands.insert_resource(Scene {
        colours,
        active,
        uniform,
    });
}
pub(super) fn sync(
    session: Res<Session>,
    scene: Res<Scene>,
    camera: Query<(&GlobalTransform, &Projection), With<crate::viewport::ZoneCamera>>,
    mut handles: Query<(
        &TransformHandle,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let pose = session
        .editor
        .zone()
        .objects
        .iter()
        .find(|o| Some(o.id) == session.selected)
        .filter(|_| {
            session.show_move_handles
                && session.palette.is_none()
                && !session.terrain_mode
                && !session.path_mode
                && !session.marker_mode
                && !session.vegetation_mode
                && !session.has_modal()
                && session.play.is_none()
        })
        .map(|o| {
            session
                .drag
                .as_ref()
                .filter(|d| d.id == o.id)
                .map_or(&o.pose, |d| &d.preview)
        });
    for (handle, mut transform, mut visibility, mut material) in &mut handles {
        let Some(pose) = pose.filter(|_| handle.tool == session.gizmo_tool) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let centre = Vec3::from_array(
            session
                .editor
                .zone()
                .local(pose.position)
                .expect("Validated preview"),
        );
        let Ok((camera, projection)) = camera.single() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let size = match projection {
            Projection::Orthographic(o) => match o.scaling_mode {
                bevy::camera::ScalingMode::FixedVertical { viewport_height } => {
                    (viewport_height * o.scale * 0.06).clamp(0.05, 24.)
                }
                _ => (camera.translation().distance(centre) * 0.06).clamp(1.2, 24.),
            },
            _ => (camera.translation().distance(centre) * 0.06).clamp(1.2, 24.),
        };
        let length = (pose.size.iter().copied().fold(0., f64::max) as f32 * 0.5 + size * 0.7)
            .max(size * 1.5);
        let length = session
            .drag
            .as_ref()
            .and_then(|d| d.gesture.scale_extent(&d.before, &d.preview))
            .unwrap_or(length);
        let direction = Vec3::AXES[handle.axis];
        let (offset, scale) = if handle.head && handle.tool == Tool::Scale {
            (length, Vec3::splat(size * 0.22))
        } else if handle.head {
            (length, Vec3::new(size * 0.16, size * 0.5, size * 0.16))
        } else {
            (
                (length - size * 0.25).max(0.02) * 0.5,
                Vec3::new(size * 0.045, (length - size * 0.25).max(0.02), size * 0.045),
            )
        };
        *transform = if handle.tool == Tool::Rotate {
            Transform::from_translation(centre).with_scale(Vec3::splat(length))
        } else {
            Transform {
                translation: centre + direction * offset,
                rotation: Quat::from_rotation_arc(Vec3::Y, direction),
                scale,
            }
        };
        *visibility = Visibility::Visible;
        material.0 = if session
            .drag
            .as_ref()
            .map(|d| d.gesture)
            .is_some_and(|g| g.tool() == handle.tool && g.axis() == Some(handle.axis))
        {
            scene.active.clone()
        } else if handle.tool == Tool::Scale {
            scene.uniform.clone()
        } else {
            scene.colours[handle.axis].clone()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_move_grids_quantize_only_active_axes_and_preserve_stationary_values() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let before = Pose {
                position: [origin[0] + 8.123, origin[1] + 1.25, origin[2] - 8.5],
                yaw_degrees: 17.25,
                size: [2., 3., 4.],
            };
            let zone = Zone {
                origin,
                spawn: origin,
                ..default()
            };
            let anchor = Vec3::from_array(zone.local(before.position).unwrap());
            let ray = |target: Vec3| {
                let eye = target + Vec3::new(6., 8., 10.);
                Ray3d::new(eye, Dir3::new(target - eye).unwrap())
            };
            for step in snapping::STEPS {
                let grid = snapping::Grid { origin, step };
                for (axis, grid_origin) in origin.into_iter().enumerate() {
                    let mut gesture = Gesture::begin(Tool::Move, ray(anchor), anchor, axis, 1.)
                        .unwrap()
                        .with_grid(grid);
                    assert_eq!(gesture.preview(ray(anchor), &before, true).unwrap(), before);
                    let pose = gesture
                        .preview(ray(anchor + Vec3::AXES[axis] * 2.3), &before, true)
                        .unwrap();
                    assert_eq!(((pose.position[axis] - grid_origin) / step).fract(), 0.);
                    for other in 0..3 {
                        if other != axis {
                            assert_eq!(pose.position[other], before.position[other]);
                        }
                    }
                    assert_eq!(pose.yaw_degrees, before.yaw_degrees);
                    assert_eq!(pose.size, before.size);
                }
                let mut plane = Gesture::plane(ray(anchor), anchor).unwrap().with_grid(grid);
                assert_eq!(plane.preview(ray(anchor), &before, true).unwrap(), before);
                let pose = plane
                    .preview(ray(anchor + Vec3::new(2.3, 0., -1.3)), &before, true)
                    .unwrap();
                for axis in [0, 2] {
                    assert_eq!(((pose.position[axis] - origin[axis]) / step).fract(), 0.);
                }
                assert_eq!(pose.position[1], before.position[1]);
                assert_eq!(pose.yaw_degrees, before.yaw_degrees);
                assert_eq!(pose.size, before.size);
            }
        }
    }
    #[test]
    fn body_plane_drag_preserves_stationary_f64_values_and_moves_relative_to_original() {
        let origin = [128_000_000_000., 20., -128_000_000_000.];
        let before = Pose {
            position: [
                origin[0] + 400.012345,
                origin[1] + 1.23456789,
                origin[2] - 300.98765,
            ],
            yaw_degrees: 17.25,
            size: [2., 3., 4.],
        };
        let zone = Zone {
            origin,
            spawn: origin,
            ..default()
        };
        let anchor = Vec3::from_array(zone.local(before.position).unwrap());
        let camera = anchor + Vec3::splat(20.);
        let ray = |point: Vec3| Ray3d::new(camera, Dir3::new(point - camera).unwrap());
        let mut gesture = Gesture::plane(ray(anchor), anchor).unwrap();
        for snap in [false, true] {
            assert_eq!(gesture.preview(ray(anchor), &before, snap).unwrap(), before);
        }
        let precise = gesture
            .preview(ray(anchor + Vec3::new(2.25, 0., -1.5)), &before, false)
            .unwrap();
        assert!((precise.position[0] - before.position[0] - 2.25).abs() < 0.0002);
        assert!((precise.position[2] - before.position[2] + 1.5).abs() < 0.0002);
        assert_eq!(precise.position[1], before.position[1]);
        assert_eq!(precise.yaw_degrees, before.yaw_degrees);
        assert_eq!(precise.size, before.size);
        let snapped = gesture
            .preview(ray(anchor + Vec3::new(2.25, 0., -1.5)), &before, true)
            .unwrap();
        assert_eq!(
            snapped.position,
            [
                precise.position[0].round(),
                before.position[1],
                precise.position[2].round()
            ]
        );
        assert!(Gesture::plane(Ray3d::new(camera, Dir3::X), anchor).is_none());
    }
    #[test]
    fn uniform_scale_keeps_position_yaw_and_proportions_with_relative_snapping() {
        let anchor = Vec3::new(8., 1., -8.);
        let camera = anchor + Vec3::splat(20.);
        let ray = |height: f32| {
            Ray3d::new(
                camera,
                Dir3::new(anchor + Vec3::Y * height - camera).unwrap(),
            )
        };
        for size in [[2., 3., 4.], [2.; 3], [2., 3., 2.]] {
            let before = Pose {
                position: [128_000_000_008.125, 21.25, -128_000_000_008.5],
                yaw_degrees: 17.25,
                size,
            };
            let mut gesture = Gesture::begin(Tool::Scale, ray(3.), anchor, 1, 3.).unwrap();
            assert_eq!(gesture.preview(ray(3.), &before, true).unwrap(), before);
            let scaled = gesture.preview(ray(4.35), &before, false).unwrap();
            assert_eq!(scaled.position, before.position);
            assert_eq!(scaled.yaw_degrees, before.yaw_degrees);
            for (actual, initial) in scaled.size.into_iter().zip(size) {
                assert!((actual / initial - 1.45).abs() < 0.00001);
            }
            let snapped = gesture.preview(ray(4.38), &before, true).unwrap();
            assert_eq!(snapped.size, size.map(|v| v * 1.5));
            assert!((gesture.scale_extent(&before, &snapped).unwrap() - 4.5).abs() < 0.00001);
        }
        assert!(Gesture::begin(Tool::Scale, ray(3.), anchor, 1, 0.).is_none());
    }
    #[test]
    fn yaw_gestures_unwrap_and_snap_without_changing_pose_coordinates_or_size() {
        let anchor = Vec3::new(8., 1., -8.);
        let camera = anchor + Vec3::new(20., 20., 20.);
        let ray = |degrees: f32| {
            let angle = degrees.to_radians();
            let point = anchor + Vec3::new(angle.cos() * 3., 0., -angle.sin() * 3.);
            Ray3d::new(camera, Dir3::new(point - camera).unwrap())
        };
        let before = Pose {
            position: [128_000_000_008.125, 21.25, -128_000_000_008.5],
            yaw_degrees: 17.25,
            size: [2.5, 3., 1.5],
        };
        let mut gesture = Gesture::begin(Tool::Rotate, ray(170.), anchor, 1, 3.).unwrap();
        assert_eq!(gesture.preview(ray(170.), &before, true).unwrap(), before);
        let changed = gesture.preview(ray(-170.), &before, false).unwrap();
        assert!((changed.yaw_degrees - 37.25).abs() < 0.001);
        assert_eq!(changed.position, before.position);
        assert_eq!(changed.size, before.size);
        let snapped = gesture.preview(ray(-160.), &before, true).unwrap();
        assert_eq!(snapped.yaw_degrees, 45.);
        for degrees in [-90., 0., 90., 170.] {
            gesture.preview(ray(degrees), &before, false).unwrap();
        }
        let full = gesture.preview(ray(170.), &before, false).unwrap();
        assert!((full.yaw_degrees - before.yaw_degrees - 360.).abs() < 0.001);
        assert!(Gesture::begin(Tool::Rotate, Ray3d::new(camera, Dir3::X), anchor, 1, 3.).is_none());
        assert!(
            Gesture::begin(
                Tool::Rotate,
                Ray3d::new(camera, Dir3::new(anchor - camera).unwrap()),
                anchor,
                1,
                3.
            )
            .is_none()
        );
    }
    #[test]
    fn axis_drag_preserves_other_far_coordinates_and_snaps_only_active_axis() {
        let anchor = Vec3::new(8., 1., -8.);
        for axis in 0..3 {
            let camera = anchor + Vec3::splat(20.);
            let ray = |point: Vec3| Ray3d::new(camera, Dir3::new(point - camera).unwrap());
            let drag = AxisDrag::begin(ray(anchor), anchor, axis).unwrap();
            let before = [128_000_000_008.125, 21.25, -128_000_000_008.5];
            assert_eq!(drag.position(ray(anchor), before, true).unwrap(), before);
            let moved = drag
                .position(ray(anchor + Vec3::AXES[axis] * 2.25), before, false)
                .unwrap();
            assert!((moved[axis] - before[axis] - 2.25).abs() < 0.0001);
            for i in 0..3 {
                if i != axis {
                    assert_eq!(moved[i], before[i]);
                }
            }
            let snapped = drag
                .position(ray(anchor + Vec3::AXES[axis] * 2.25), before, true)
                .unwrap();
            assert_eq!(snapped[axis], moved[axis].round());
        }
    }
    #[test]
    fn parallel_and_behind_camera_axis_queries_refuse() {
        assert!(AxisDrag::begin(Ray3d::new(Vec3::ONE, Dir3::X), Vec3::ZERO, 0).is_none());
        let ray = Ray3d::new(Vec3::new(0., 0., 10.), Dir3::Z);
        assert!(AxisDrag::begin(ray, Vec3::ZERO, 0).is_none());
    }
}
