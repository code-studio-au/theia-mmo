//! Editable 512 m starter valley. Gameplay contacts and navigation are separate.
use crate::{
    Result, Zone, require,
    terrain::{HeightPatch, Resolution},
};
fn smooth(value: f64) -> f64 {
    let t = value.clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub(crate) fn apply(zone: &mut Zone) -> Result<()> {
    require(
        crate::starter_basin::available(zone),
        "Create a new blank zone before adding starter valley terrain",
    )?;
    let chunk = theia_core::coordinates::GlobalPosition::new(zone.origin)
        .and_then(|p| p.chunk(128))
        .ok_or("Invalid valley origin")?;
    // Centre on the containing logical chunk boundary. The local frame stays
    // exact even when the zone origin has a fractional, far-world offset.
    let mut patch = HeightPatch {
        chunk: [chunk.x - 2, chunk.z - 2],
        span_chunks: 4,
        resolution: Resolution::FourMetres,
        heights: vec![0.; 129 * 129],
        weights: vec![crate::painting::GRASS; 129 * 129],
    };
    let start = patch.origin()?;
    for z in 1..128 {
        for x in 1..128 {
            let dx = x as f64 * 4. - 256.;
            let dz = z as f64 * 4. - 256.;
            let radius = dx.hypot(dz);
            let angle = dx.atan2(dz);
            let inner = 154.
                + 22. * (2. * angle + 0.3).cos()
                + 10. * (3. * angle).sin()
                + 7. * (5. * angle + 0.7).sin();
            let peak = 32.
                + 24. * (3. * angle + 0.4).cos().max(0.).powi(4)
                + 4. * (dx / 8.).sin() * (dz / 11.).cos();
            let ridge =
                peak * smooth((radius - inner) / 24.) * smooth((inner + 65. - radius) / 34.);
            let pass_x = 5. * ((dz - 100.) / 40.).sin();
            let pass = if dz > 100. {
                smooth(((dx - pass_x).abs() - 10.) / 8.)
            } else {
                1.
            };
            let hills = (1.8 + 1.4 * (dx / 28.).sin() * (dz / 35.).cos())
                * smooth((radius - 42.) / 32.)
                * (1. - smooth((radius - 135.) / 24.));
            // Village plaza and four broad roads stay level; the ridge still
            // crosses the western/eastern/northern routes to contain the zone.
            let road = if dz >= 0. {
                (dx - pass_x).abs()
            } else {
                (dx - 12. * (dz / 38.).sin()).abs()
            };
            let road = road.min((dz - 10. * (dx / 34.).sin()).abs());
            let hill_mask = smooth((road - 4.) / 5.);
            let height = ridge * pass + hills * hill_mask;
            let index = z * 129 + x;
            patch.heights[index] = height as f32;
            let rock = smooth((height - 8.) / 14.);
            let trail_end = if dz > 100. && (dx - pass_x).abs() < 9. {
                1.
            } else {
                1. - smooth((radius - 125.) / 25.)
            };
            let dirt = (1. - smooth((road - 3.) / 4.)) * trail_end;
            let dirt = dirt.max(1. - smooth((radius - 16.) / 8.)) * (1. - smooth(height / 3.));
            let stone = (rock * 255.).round() as u8;
            let dirt = (dirt * f64::from(255 - stone)).round() as u8;
            patch.weights[index] = [255 - stone - dirt, dirt, stone, 0];
        }
    }
    zone.schema = zone.schema.max(11);
    zone.spawn = [start[0] + 256., zone.origin[1] + 0.1, start[2] + 256.];
    zone.terrain = Some(patch);
    zone.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    #[test]
    fn valley_is_bounded_editable_and_one_undo_at_local_and_far_origins() {
        for origin in [
            [8192., 0., 8192.],
            [128_000_000_000.125, 20., -128_000_000_000.25],
        ] {
            let zone = Zone {
                origin,
                spawn: origin,
                ..Zone::default()
            };
            let mut editor = Editor::new(zone.clone()).unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::CreateStarterValley],
                })
                .unwrap();
            let result = editor.zone().clone();
            let patch = result.terrain.as_ref().unwrap();
            assert_eq!(patch.extent_metres(), 512.);
            assert_eq!(patch.axis(), 129);
            assert!(patch.heights.iter().copied().fold(0., f32::max) > 45.);
            assert!(
                serde_json::to_vec_pretty(&result).unwrap().len()
                    < crate::MAX_DOCUMENT_BYTES as usize
            );
            let start = patch.origin().unwrap();
            for dz in (0..=256).step_by(4) {
                let pass_x = if dz > 100 {
                    5. * ((dz as f64 - 100.) / 40.).sin()
                } else {
                    0.
                };
                assert!(
                    patch
                        .height(start[0] + 256. + pass_x, start[2] + 256. + dz as f64)
                        .unwrap()
                        .unwrap()
                        < 0.05
                );
            }
            let p = [start[0] + 256., start[2] + 256.];
            editor
                .execute(Command::Apply {
                    expected_revision: 1,
                    edits: vec![Edit::Brush {
                        brush: crate::terrain::Brush {
                            centre: p,
                            radius_metres: 12.,
                            operation: crate::terrain::BrushOperation::Raise { metres: 2. },
                        },
                    }],
                })
                .unwrap();
            assert!(
                editor
                    .zone()
                    .terrain
                    .as_ref()
                    .unwrap()
                    .height(p[0], p[1])
                    .unwrap()
                    .unwrap()
                    > 1.
            );
            editor
                .execute(Command::Undo {
                    expected_revision: 2,
                })
                .unwrap();
            assert_eq!(editor.zone(), &result);
            editor
                .execute(Command::Undo {
                    expected_revision: 3,
                })
                .unwrap();
            assert_eq!(editor.zone(), &zone);
            editor
                .execute(Command::Redo {
                    expected_revision: 4,
                })
                .unwrap();
            assert_eq!(editor.zone(), &result);
        }
    }
}
