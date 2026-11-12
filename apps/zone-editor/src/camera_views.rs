//! Standard Bevy editor projections and canonical authoring viewpoints.
use super::*;
use bevy::camera::ScalingMode;

/// Centre the actual authored patch, including its height, in the local frame.
pub(super) fn terrain_target(zone: &Zone) -> Option<Vec3> {
    let patch = zone.terrain.as_ref()?;
    let start = patch.origin().ok()?;
    let half = patch.extent_metres() / 2.;
    let centre = zone
        .surface_position([start[0] + half, start[2] + half])
        .ok()?;
    Some(Vec3::from_array(zone.local(centre).ok()?))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Preset {
    #[default]
    Perspective,
    Top,
    Front,
    Right,
}
impl Preset {
    pub fn select(self, view: &mut View) {
        view.preset = self;
        (view.yaw, view.pitch) = match self {
            Self::Perspective => (0.6, 0.8),
            Self::Top => (0., 1.5),
            Self::Front => (0., 0.),
            Self::Right => (std::f32::consts::FRAC_PI_2, 0.),
        };
    }
    pub fn pan_axes(self) -> (Vec3, Vec3) {
        match self {
            Self::Perspective | Self::Top => (Vec3::X, Vec3::NEG_Z),
            Self::Front => (Vec3::X, Vec3::Y),
            Self::Right => (Vec3::NEG_Z, Vec3::Y),
        }
    }
}
pub(super) fn transform(view: &View) -> Transform {
    let (direction, up) = match view.preset {
        Preset::Top => (Vec3::Y, Vec3::NEG_Z),
        Preset::Front => (Vec3::Z, Vec3::Y),
        Preset::Right => (Vec3::X, Vec3::Y),
        Preset::Perspective => (
            Vec3::new(
                view.yaw.sin() * view.pitch.cos(),
                view.pitch.sin(),
                view.yaw.cos() * view.pitch.cos(),
            ),
            Vec3::Y,
        ),
    };
    let distance = if view.preset == Preset::Perspective {
        view.distance
    } else {
        2. * theia_zone::ZONE_EXTENT as f32
    };
    Transform::from_translation(view.target + direction * distance).looking_at(view.target, up)
}
pub(super) fn projection(view: &View, play: bool, projection: &mut Projection) {
    if play || view.preset == Preset::Perspective {
        if !matches!(projection, Projection::Perspective(_)) {
            *projection = Projection::Perspective(PerspectiveProjection::default());
        }
        if let Projection::Perspective(perspective) = projection {
            perspective.far = if play { 1000. } else { 4096. };
        }
    } else if let Projection::Orthographic(orthographic) = projection {
        if !matches!(orthographic.scaling_mode,ScalingMode::FixedVertical { viewport_height } if viewport_height==view.distance)
        {
            orthographic.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: view.distance,
            };
        }
    } else {
        *projection = Projection::Orthographic(OrthographicProjection {
            near: 0.01,
            far: 4096.,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: view.distance,
            },
            ..OrthographicProjection::default_3d()
        });
    }
}

/// Drag the scene with the pointer in the camera plane through its target.
/// This changes only the editor's bounded local view, never authored positions.
pub(super) fn pan(view: &mut View, delta: Vec2, viewport_height: f32) {
    if !delta.is_finite() || !viewport_height.is_finite() || viewport_height <= 0. {
        return;
    }
    let span = if view.preset == Preset::Perspective {
        2. * view.distance * (PerspectiveProjection::default().fov * 0.5).tan()
    } else {
        view.distance
    };
    let (right, up) = if view.preset == Preset::Perspective {
        let rotation = transform(view).rotation;
        (rotation * Vec3::X, rotation * Vec3::Y)
    } else {
        view.preset.pan_axes()
    };
    let offset = (-right * delta.x + up * delta.y) * (span / viewport_height);
    if !offset.is_finite() {
        return;
    }
    view.target = (view.target + offset).clamp(
        Vec3::splat(-theia_zone::ZONE_EXTENT as f32),
        Vec3::splat(theia_zone::ZONE_EXTENT as f32),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terrain_start_view_uses_full_span_and_surface_height_at_local_and_far_origins() {
        for xz in [8192., 128_000_000_000., -128_000_000_000.] {
            for span in 1..=4 {
                let mut patch = theia_zone::terrain::HeightPatch::flat([
                    (xz / 128.) as i64,
                    (-xz / 128.) as i64,
                ]);
                patch.span_chunks = span;
                patch.heights = vec![0.; patch.axis() * patch.axis()];
                let centre = patch.axis() / 2;
                let index = centre * patch.axis() + centre;
                patch.heights[index] = 7.;
                let zone = Zone {
                    schema: 11,
                    origin: [xz, 32., -xz],
                    spawn: [xz, 32.1, -xz],
                    terrain: Some(patch),
                    ..default()
                };
                zone.validate().unwrap();
                let before = zone.clone();
                assert_eq!(
                    terrain_target(&zone),
                    Some(Vec3::new(64. * f32::from(span), 7., 64. * f32::from(span)))
                );
                assert_eq!(zone, before);
            }
        }
        assert_eq!(terrain_target(&Zone::default()), None);
    }
    fn view() -> View {
        View {
            target: Vec3::new(64., 20., 64.),
            show_grid: true,
            yaw: 0.6,
            pitch: 0.8,
            distance: 180.,
            preset: Preset::Perspective,
        }
    }
    #[test]
    fn pointer_pan_uses_the_projection_plane_and_zoom_scale() {
        for preset in [
            Preset::Top,
            Preset::Front,
            Preset::Right,
            Preset::Perspective,
        ] {
            let mut view = view();
            preset.select(&mut view);
            let before = view.target;
            let rotation = transform(&view).rotation;
            let span = if preset == Preset::Perspective {
                2. * view.distance * (PerspectiveProjection::default().fov * 0.5).tan()
            } else {
                view.distance
            };
            let expected =
                before + (rotation * Vec3::X * (-100.) + rotation * Vec3::Y * 50.) * (span / 1440.);
            pan(&mut view, Vec2::new(100., 50.), 1440.);
            assert!((view.target - expected).length() < 0.0001);
            let offset = view.target - before;
            match preset {
                Preset::Top => assert_eq!(view.target.y, before.y),
                Preset::Front => assert_eq!(view.target.z, before.z),
                Preset::Right => assert_eq!(view.target.x, before.x),
                Preset::Perspective => {}
            }
            view.target = before;
            view.distance *= 0.5;
            pan(&mut view, Vec2::new(100., 50.), 1440.);
            assert!((view.target - before - offset * 0.5).length() < 0.0001);
        }
    }
    #[test]
    fn pointer_pan_rejects_bad_input_and_bounds_the_local_target() {
        let mut view = view();
        Preset::Front.select(&mut view);
        let before = view.target;
        for (delta, height) in [
            (Vec2::splat(f32::NAN), 1440.),
            (Vec2::ONE, 0.),
            (Vec2::ONE, f32::INFINITY),
        ] {
            pan(&mut view, delta, height);
            assert_eq!(view.target, before);
        }
        pan(&mut view, Vec2::new(-100_000., 100_000.), 1440.);
        assert_eq!(view.target, Vec3::new(512., 512., before.z));
    }
    #[test]
    fn canonical_views_have_exact_axes_and_look_at_the_existing_target() {
        for (preset, forward, up, right) in [
            (Preset::Top, Vec3::NEG_Y, Vec3::NEG_Z, Vec3::X),
            (Preset::Front, Vec3::NEG_Z, Vec3::Y, Vec3::X),
            (Preset::Right, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Z),
        ] {
            let mut view = view();
            preset.select(&mut view);
            let camera = transform(&view);
            assert!((camera.rotation * Vec3::NEG_Z - forward).length() < 0.00001);
            assert!((camera.rotation * Vec3::Y - up).length() < 0.00001);
            assert!((camera.rotation * Vec3::X - right).length() < 0.00001);
            assert!(
                (camera.translation + forward * (2. * theia_zone::ZONE_EXTENT as f32)
                    - view.target)
                    .length()
                    < 0.00001
            );
            assert_eq!(preset.pan_axes(), (right, up));
        }
    }
    #[test]
    fn orthographic_zoom_and_playtest_projection_restore_without_changing_view() {
        let mut view = view();
        Preset::Top.select(&mut view);
        let mut camera = Projection::default();
        projection(&view, false, &mut camera);
        assert!(
            matches!(&camera,Projection::Orthographic(o) if matches!(o.scaling_mode,ScalingMode::FixedVertical { viewport_height:180. }))
        );
        let position = transform(&view).translation;
        view.distance = 24.;
        assert_eq!(transform(&view).translation, position);
        projection(&view, false, &mut camera);
        assert!(
            matches!(&camera,Projection::Orthographic(o) if matches!(o.scaling_mode,ScalingMode::FixedVertical { viewport_height:24. }))
        );
        projection(&view, true, &mut camera);
        assert!(matches!(camera, Projection::Perspective(_)));
        projection(&view, false, &mut camera);
        assert!(matches!(camera, Projection::Orthographic(_)));
        assert_eq!(view.preset, Preset::Top);
        Preset::Perspective.select(&mut view);
        projection(&view, false, &mut camera);
        assert!(matches!(camera, Projection::Perspective(_)));
    }
}
