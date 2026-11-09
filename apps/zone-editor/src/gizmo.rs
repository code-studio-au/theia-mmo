//! World-axis move handles built from standard Bevy meshes and ray queries.
use super::*;

#[derive(Component)]
pub(super) struct MoveHandle {
    pub axis: usize,
    pub head: bool,
}
#[derive(Resource)]
pub(super) struct Scene {
    colours: [Handle<StandardMaterial>; 3],
    active: Handle<StandardMaterial>,
}
#[derive(Clone, Copy)]
pub(super) struct AxisDrag {
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
    (value.is_finite() && (0. ..2000.).contains(&distance)).then_some(value)
}
impl AxisDrag {
    pub fn begin(ray: Ray3d, anchor: Vec3, axis: usize) -> Option<Self> {
        Some(Self {
            axis,
            anchor,
            start: parameter(ray, anchor, axis)?,
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
            position[self.axis] = position[self.axis].round();
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
                MoveHandle {
                    axis,
                    head: is_head,
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
    commands.insert_resource(Scene { colours, active });
}
pub(super) fn sync(
    session: Res<Session>,
    scene: Res<Scene>,
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut handles: Query<(
        &MoveHandle,
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
        let Some(pose) = pose else {
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
        let Ok(camera) = camera.single() else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let size = (camera.translation().distance(centre) * 0.06).clamp(1.2, 24.);
        let length = (pose.size.iter().copied().fold(0., f64::max) as f32 * 0.5 + size * 0.7)
            .max(size * 1.5);
        let direction = Vec3::AXES[handle.axis];
        let (offset, scale) = if handle.head {
            (length, Vec3::new(size * 0.16, size * 0.5, size * 0.16))
        } else {
            (
                (length - size * 0.25) * 0.5,
                Vec3::new(size * 0.045, length - size * 0.25, size * 0.045),
            )
        };
        *transform = Transform {
            translation: centre + direction * offset,
            rotation: Quat::from_rotation_arc(Vec3::Y, direction),
            scale,
        };
        *visibility = Visibility::Visible;
        material.0 = if session
            .drag
            .as_ref()
            .and_then(|d| d.axis)
            .is_some_and(|a| a.axis == handle.axis)
        {
            scene.active.clone()
        } else {
            scene.colours[handle.axis].clone()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
