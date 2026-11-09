//! Surface spawn authoring through checked coordinates and the retained SetSpawn edit.
use super::*;

pub(super) fn position(
    zone: &Zone,
    local: Vec3,
    snap: bool,
    step: f64,
) -> Result<[f64; 3], String> {
    let mut global = zone.global(local.to_array())?;
    if snap {
        for axis in [0, 2] {
            global[axis] = theia_zone::snap(global[axis], zone.origin[axis], step)?;
        }
    }
    let mut surface = zone.surface_position([global[0], global[2]])?;
    surface[1] += 0.1;
    Ok(surface)
}

pub(super) fn at_cursor(session: &mut Session, point: Option<Vec3>, held: bool) {
    if session.stroke.is_some() || held {
        session.status = "Finish or cancel the stroke before placing spawn".into();
        return;
    }
    let result = point
        .ok_or_else(|| "Point at ground to place the playtest spawn".into())
        .and_then(|p| position(session.editor.zone(), p, session.snap, session.move_step));
    match result {
        Ok(position) if position == session.editor.zone().spawn => {
            session.status = "Spawn already at this surface position".into();
        }
        Ok(position) => session.edit(vec![Edit::SetSpawn { position }]),
        Err(error) => session.status = error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::terrain::{HeightPatch, Resolution};

    #[test]
    fn selected_grid_step_quantizes_spawn_xz_without_quantizing_surface_height() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let zone = Zone {
                origin,
                spawn: origin,
                ..default()
            };
            let local = Vec3::new(1.31, 30., 2.8);
            for step in snapping::STEPS {
                let result = position(&zone, local, true, step).unwrap();
                for axis in [0, 2] {
                    assert_eq!(((result[axis] - origin[axis]) / step).fract(), 0.);
                }
                assert_eq!(result[1], origin[1] + 0.1);
            }
            assert!(position(&zone, local, true, 0.).is_err());
        }
    }
    #[test]
    fn snapped_and_precise_spawn_use_exact_surface_in_both_far_frames() {
        for resolution in [Resolution::FourMetres, Resolution::TwoMetres] {
            for sign in [-1_i64, 1] {
                let mut patch = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
                patch.resolution = resolution;
                let axis = patch.axis();
                patch.heights = (0..axis * axis)
                    .map(|i| {
                        if i % axis == 0
                            || i % axis == axis - 1
                            || i / axis == 0
                            || i / axis == axis - 1
                        {
                            0.
                        } else {
                            (i % axis) as f32 * 0.125
                        }
                    })
                    .collect();
                let mut origin = patch.origin().unwrap();
                origin[1] = 20.;
                let zone = Zone {
                    schema: 7,
                    origin,
                    spawn: [origin[0] + 16., 20.1, origin[2] + 16.],
                    terrain: Some(patch),
                    ..default()
                };
                zone.validate().unwrap();
                for snap in [true, false] {
                    let result = position(&zone, Vec3::new(64.25, -30., 63.75), snap, 1.).unwrap();
                    let xz = if snap {
                        [origin[0] + 64., origin[2] + 64.]
                    } else {
                        [origin[0] + 64.25, origin[2] + 63.75]
                    };
                    assert_eq!([result[0], result[2]], xz);
                    assert_eq!(result[1], zone.surface_position(xz).unwrap()[1] + 0.1);
                    let mut editor = Editor::new(zone.clone()).unwrap();
                    editor
                        .execute(Command::Apply {
                            expected_revision: 0,
                            edits: vec![Edit::SetSpawn { position: result }],
                        })
                        .unwrap();
                    assert_eq!(editor.zone().spawn, result);
                    assert_eq!(editor.history(), (1, 0));
                    editor
                        .execute(Command::Undo {
                            expected_revision: 1,
                        })
                        .unwrap();
                    assert_eq!(editor.zone(), &zone);
                }
            }
        }
    }

    #[test]
    fn flat_surroundings_and_invalid_coordinates_use_checked_frame_bounds() {
        let zone = Zone {
            origin: [128000000000., 20., -128000000000.],
            spawn: [128000000016., 20.1, -127999999984.],
            ..default()
        };
        assert_eq!(
            position(&zone, Vec3::new(1.25, -50., 2.75), false, 1.).unwrap(),
            [128000000001.25, 20.1, -127999999997.25]
        );
        for point in [
            Vec3::NAN,
            Vec3::new(513., 0., 0.),
            Vec3::new(0., 0., -513.),
            Vec3::new(0., 4097., 0.),
        ] {
            assert!(position(&zone, point, false, 1.).is_err());
        }
    }
}
