//! Bounded, non-pickable standard mesh references; never source or collision.
use super::*;
const HALF: i32 = 32;
#[derive(Component)]
pub(super) struct Line {
    pub slot: i32,
    pub along_first: bool,
}
#[derive(Resource)]
pub(super) struct Materials([Handle<StandardMaterial>; 5]);
pub(super) fn surface(session: &Session, view: &View) -> bool {
    session.editor.zone().terrain.is_some()
        && matches!(
            view.preset,
            camera_views::Preset::Perspective | camera_views::Preset::Top
        )
}
pub(super) fn step(span: f32) -> f32 {
    let desired = (span / 20.).max(0.1);
    let decade = 10f32.powf(desired.log10().floor());
    [1., 2., 5., 10.]
        .into_iter()
        .map(|n| n * decade)
        .find(|n| *n >= desired)
        .unwrap_or(decade * 10.)
}
pub(super) fn spacing(session: &Session, view: &View) -> f32 {
    if surface(session, view) {
        session
            .editor
            .zone()
            .terrain
            .as_ref()
            .unwrap()
            .sample_metres() as f32
    } else {
        step(view.distance)
    }
}
fn axes(preset: camera_views::Preset) -> (Vec3, Vec3, Vec3, usize, usize) {
    match preset {
        camera_views::Preset::Front => (Vec3::X, Vec3::Y, Vec3::Z, 2, 3),
        camera_views::Preset::Right => (Vec3::Z, Vec3::Y, Vec3::X, 4, 3),
        _ => (Vec3::X, Vec3::Z, Vec3::Y, 2, 4),
    }
}
fn line(view: &View, slot: i32, along_first: bool) -> (Transform, usize) {
    let step = step(view.distance);
    let (first, second, normal, first_colour, second_colour) = axes(view.preset);
    let (along, across, colour) = if along_first {
        (first, second, first_colour)
    } else {
        (second, first, second_colour)
    };
    let coordinate = (view.target.dot(across) / step).round() + slot as f32;
    let centre = (view.target.dot(along) / step).round() * step;
    let length = 2. * HALF as f32 * step;
    let width = view.distance * 0.0005;
    let major = coordinate.rem_euclid(5.) == 0.;
    let axis = coordinate == 0.;
    let thickness = width
        * if axis {
            2.
        } else if major {
            1.4
        } else {
            1.
        };
    let scale = along.abs() * length + across.abs() * thickness + normal.abs() * thickness;
    (
        Transform::from_translation(along * centre + across * coordinate * step + normal * 0.004)
            .with_scale(scale),
        if axis { colour } else { usize::from(major) },
    )
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1., 1., 1.));
    let palette = [
        Color::srgb(0.18, 0.23, 0.25),
        Color::srgb(0.30, 0.37, 0.40),
        Color::srgb(0.65, 0.18, 0.16),
        Color::srgb(0.18, 0.55, 0.26),
        Color::srgb(0.18, 0.33, 0.72),
    ]
    .map(|base_color| {
        materials.add(StandardMaterial {
            base_color,
            unlit: true,
            ..default()
        })
    });
    for slot in -HALF..=HALF {
        for along_first in [true, false] {
            commands.spawn((
                Line { slot, along_first },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(palette[0].clone()),
                Transform::default(),
                Visibility::Hidden,
                bevy::light::NotShadowCaster,
                bevy::light::NotShadowReceiver,
                bevy::picking::Pickable::IGNORE,
            ));
        }
    }
    commands.insert_resource(Materials(palette));
}
pub(super) fn sync(
    session: Res<Session>,
    view: Res<View>,
    materials: Res<Materials>,
    mut lines: Query<(
        &Line,
        &mut Transform,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let visible = view.show_grid && session.play.is_none() && !surface(&session, &view);
    for (reference, mut transform, mut visibility, mut material) in &mut lines {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if visible {
            let (pose, colour) = line(&view, reference.slot, reference.along_first);
            *transform = pose;
            material.0 = materials.0[colour].clone();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adaptive_spacing_covers_ultrawide_view_with_bounded_density() {
        for span in [3., 8., 24., 32., 120., 180., 512.] {
            let spacing = step(span);
            assert!(spacing >= span / 20. && spacing <= span / 4.);
            assert!(HALF as f32 * spacing >= span * (3440. / 1440.) / 2.);
        }
    }
    #[test]
    fn grids_follow_canonical_planes_and_pan_on_origin_aligned_lines() {
        for preset in [
            camera_views::Preset::Top,
            camera_views::Preset::Front,
            camera_views::Preset::Right,
        ] {
            let view = View {
                target: Vec3::new(24.3, 17.9, -38.2),
                yaw: 0.,
                pitch: 0.,
                distance: 32.,
                preset,
                show_grid: true,
            };
            let (a, b, n, _, _) = axes(preset);
            for along_first in [true, false] {
                let (pose, _) = line(&view, 0, along_first);
                let across = if along_first { b } else { a };
                assert!((pose.translation.dot(n) - 0.004).abs() < 0.00001);
                assert!(
                    (pose.translation.dot(across) / step(view.distance))
                        .fract()
                        .abs()
                        < 0.00001
                );
                assert!(pose.scale.is_finite() && pose.scale.min_element() > 0.);
            }
        }
    }
}
