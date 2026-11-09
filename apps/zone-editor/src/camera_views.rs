//! Standard Bevy editor projections and canonical authoring viewpoints.
use super::*;
use bevy::camera::ScalingMode;

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

#[cfg(test)]
mod tests {
    use super::*;
    fn view() -> View {
        View {
            target: Vec3::new(64., 20., 64.),
            yaw: 0.6,
            pitch: 0.8,
            distance: 180.,
            preset: Preset::Perspective,
        }
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
