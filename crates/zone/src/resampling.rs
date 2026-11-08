//! Finite 4 m / 2 m authoring grids; no runtime LOD or streaming.
use crate::{
    Result, Zone,
    painting::GRASS,
    require,
    terrain::{HeightPatch, Resolution},
};
pub fn resample(zone: &Zone, resolution: Resolution) -> Result<HeightPatch> {
    let previous = zone
        .terrain
        .as_ref()
        .ok_or("Create terrain before changing resolution")?;
    previous.validate(zone)?;
    require(
        previous.resolution != resolution,
        "Terrain already uses that resolution",
    )?;
    let origin = previous.origin()?;
    let axis = resolution.axis();
    let old_axis = previous.axis();
    let mut next = previous.clone();
    next.resolution = resolution;
    next.heights = Vec::with_capacity(axis * axis);
    next.weights = vec![];
    for z in 0..axis {
        for x in 0..axis {
            let px = origin[0] + x as f64 * resolution.metres();
            let pz = origin[2] + z as f64 * resolution.metres();
            next.heights
                .push(previous.height(px, pz)?.ok_or("Resample outside patch")?);
            if !previous.weights.is_empty() {
                let u = x as f64 * resolution.metres() / previous.sample_metres();
                let v = z as f64 * resolution.metres() / previous.sample_metres();
                let ix = (u.floor() as usize).min(old_axis - 2);
                let iz = (v.floor() as usize).min(old_axis - 2);
                let u = u - ix as f64;
                let v = v - iz as f64;
                let a = iz * old_axis + ix;
                let b = a + 1;
                let c = a + old_axis;
                let d = c + 1;
                let (indices, weights) = if u + v <= 1. {
                    ([a, b, c], [1. - u - v, u, v])
                } else {
                    ([b, c, d], [1. - v, 1. - u, u + v - 1.])
                };
                let values = std::array::from_fn::<_, 4, _>(|channel| {
                    (0..3)
                        .map(|i| f64::from(previous.weights[indices[i]][channel]) * weights[i])
                        .sum::<f64>()
                });
                let mut row = values.map(|v| v.floor() as u8);
                let sum = row.iter().map(|&v| u16::from(v)).sum::<u16>();
                require(sum <= 255 && 255 - sum <= 4, "Invalid resampled weight sum")?;
                let mut order = [0, 1, 2, 3];
                order.sort_by(|&a, &b| {
                    (values[b] - values[b].floor())
                        .total_cmp(&(values[a] - values[a].floor()))
                        .then(a.cmp(&b))
                });
                for &index in order.iter().take((255 - sum) as usize) {
                    row[index] += 1;
                }
                next.weights.push(row);
            }
        }
    }
    // Borders interpolate only retained zero/grass border samples.
    if !next.weights.is_empty() {
        for i in 0..axis * axis {
            if i / axis == 0 || i / axis == axis - 1 || i % axis == 0 || i % axis == axis - 1 {
                require(next.weights[i] == GRASS, "Resample changed grass border")?;
            }
        }
    }
    next.validate(zone)?;
    Ok(next)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, Edit, Editor,
        painting::{Paint, Surface},
        terrain::{Brush, BrushOperation},
    };
    #[test]
    fn finer_grid_preserves_height_surface_and_normalized_colour_round_trip() {
        let mut e = Editor::new(Zone {
            schema: 2,
            terrain: Some(HeightPatch::flat([0, 0])),
            ..Zone::default()
        })
        .unwrap();
        e.execute(Command::Apply {
            expected_revision: 0,
            edits: vec![
                Edit::Brush {
                    brush: Brush {
                        centre: [64., 64.],
                        radius_metres: 16.,
                        operation: BrushOperation::Raise { metres: 4. },
                    },
                },
                Edit::PaintTerrain {
                    paint: Paint {
                        centre: [64., 64.],
                        radius_metres: 16.,
                        surface: Surface::Dirt,
                        blend: 0.73,
                    },
                },
            ],
        })
        .unwrap();
        let original = e.zone().clone();
        e.execute(Command::Apply {
            expected_revision: 1,
            edits: vec![Edit::ResampleTerrain {
                resolution: Resolution::TwoMetres,
            }],
        })
        .unwrap();
        let fine = e.zone().clone();
        let patch = fine.terrain.as_ref().unwrap();
        assert_eq!(fine.schema, 7);
        assert_eq!(patch.heights.len(), 65 * 65);
        assert_eq!(patch.weights.len(), 65 * 65);
        for z in 0..128 {
            for x in 0..128 {
                let px = x as f64 + 0.37;
                let pz = z as f64 + 0.23;
                assert!(
                    (patch.height(px, pz).unwrap().unwrap()
                        - original
                            .terrain
                            .as_ref()
                            .unwrap()
                            .height(px, pz)
                            .unwrap()
                            .unwrap())
                    .abs()
                        < 0.00001
                );
            }
        }
        for row in &patch.weights {
            assert_eq!(row.iter().map(|&v| u16::from(v)).sum::<u16>(), 255);
        }
        e.execute(Command::Undo {
            expected_revision: 2,
        })
        .unwrap();
        assert_eq!(e.zone(), &original);
        e.execute(Command::Redo {
            expected_revision: 3,
        })
        .unwrap();
        assert_eq!(e.zone(), &fine);
        e.execute(Command::Apply {
            expected_revision: 4,
            edits: vec![Edit::ResampleTerrain {
                resolution: Resolution::FourMetres,
            }],
        })
        .unwrap();
        assert_eq!(e.zone().terrain, original.terrain);
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&fine).unwrap()).unwrap(),
            fine
        );
    }
    #[test]
    fn resolution_failures_preserve_history_and_far_signed_refinement_is_checked() {
        for sign in [-1_i64, 1] {
            let patch = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
            let origin = patch.origin().unwrap();
            let mut e = Editor::new(Zone {
                schema: 2,
                origin,
                spawn: origin.map(|v| v + 0.1),
                terrain: Some(patch),
                ..Zone::default()
            })
            .unwrap();
            assert!(
                e.execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::ResampleTerrain {
                        resolution: Resolution::FourMetres
                    }]
                })
                .is_err()
            );
            assert_eq!(e.revision(), 0);
            e.execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::ResampleTerrain {
                    resolution: Resolution::TwoMetres,
                }],
            })
            .unwrap();
            let fine = e.zone().clone();
            let mut old = fine.clone();
            old.schema = 6;
            assert!(old.validate().is_err());
            let mut invalid = fine.terrain.clone().unwrap();
            invalid.heights.pop();
            assert!(
                e.execute(Command::Apply {
                    expected_revision: 1,
                    edits: vec![
                        Edit::SetTerrain {
                            terrain: Some(invalid)
                        },
                        Edit::ResampleTerrain {
                            resolution: Resolution::FourMetres
                        }
                    ]
                })
                .is_err()
            );
            assert_eq!(e.zone(), &fine);
            let bytes=br#"{"command":"apply","expected_revision":1,"edits":[{"op":"resample_terrain","resolution":"1m"}]}"#;
            assert!(serde_json::from_slice::<Command>(bytes).is_err());
        }
    }
}
