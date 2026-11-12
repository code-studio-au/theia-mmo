//! One bounded 128..512 m authoring height patch, at most 129x129 samples.
//! Flat locked borders meet implicit flat ground; this is not runtime streaming.
use super::{Result, Zone, require};
use serde::{Deserialize, Serialize};
use theia_core::coordinates::ChunkId;
pub const AXIS: usize = 33;
pub const SAMPLE_COUNT: usize = AXIS * AXIS;
pub const TILE_METRES: f64 = 128.;
pub const SAMPLE_METRES: f64 = 4.;
pub const MAX_AXIS: usize = 129;
fn default_span() -> u8 {
    1
}
fn single_span(span: &u8) -> bool {
    *span == 1
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    #[default]
    #[serde(rename = "4m")]
    FourMetres,
    #[serde(rename = "2m")]
    TwoMetres,
}
impl Resolution {
    pub fn axis(self) -> usize {
        match self {
            Self::FourMetres => 33,
            Self::TwoMetres => 65,
        }
    }
    pub fn metres(self) -> f64 {
        match self {
            Self::FourMetres => 4.,
            Self::TwoMetres => 2.,
        }
    }
    fn is_default(&self) -> bool {
        *self == Self::FourMetres
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeightPatch {
    pub chunk: [i64; 2],
    /// Number of logical 128 m chunks along each side; one finite authoring mesh.
    #[serde(default = "default_span", skip_serializing_if = "single_span")]
    pub span_chunks: u8,
    #[serde(default, skip_serializing_if = "Resolution::is_default")]
    pub resolution: Resolution,
    /// Local height above zone-origin Y, Z-major, including both border samples.
    pub heights: Vec<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub weights: Vec<[u8; 4]>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BrushOperation {
    Raise { metres: f32 },
    Lower { metres: f32 },
    Flatten { height: f32, blend: f32 },
    Smooth { blend: f32 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brush {
    pub centre: [f64; 2],
    pub radius_metres: f64,
    pub operation: BrushOperation,
}
impl HeightPatch {
    pub fn flat(chunk: [i64; 2]) -> Self {
        Self {
            chunk,
            span_chunks: 1,
            resolution: Resolution::default(),
            heights: vec![0.; SAMPLE_COUNT],
            weights: vec![],
        }
    }
    pub fn axis(&self) -> usize {
        (self.resolution.axis() - 1) * usize::from(self.span_chunks) + 1
    }
    pub fn extent_metres(&self) -> f64 {
        TILE_METRES * f64::from(self.span_chunks)
    }
    pub fn sample_metres(&self) -> f64 {
        self.resolution.metres()
    }
    pub fn origin(&self) -> Result<[f64; 3]> {
        ChunkId {
            x: self.chunk[0],
            z: self.chunk[1],
        }
        .origin(128)
        .map(|p| p.metres())
        .ok_or("Invalid terrain chunk".into())
    }
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(
            (1..=4).contains(&self.span_chunks),
            "Terrain span must be 1..4 chunks",
        )?;
        require(
            self.span_chunks == 1 || zone.schema >= 11,
            "Expanded terrain requires zone schema 11",
        )?;
        let axis = self.axis();
        require(
            axis <= MAX_AXIS,
            "Terrain sample budget exceeded; use 4 m samples for patches wider than 256 m",
        )?;
        let count = axis * axis;
        require(
            self.resolution == Resolution::FourMetres || zone.schema >= 7,
            "Fine terrain requires zone schema 7",
        )?;
        require(
            self.heights.len() == count,
            "Terrain heights must match resolution",
        )?;
        if !self.weights.is_empty() {
            require(zone.schema >= 6, "Terrain paint requires zone schema 6")?;
            require(
                self.weights.len() == count,
                "Terrain paint must match resolution",
            )?;
            for (index, w) in self.weights.iter().enumerate() {
                require(
                    w.iter().map(|&v| u16::from(v)).sum::<u16>() == 255,
                    "Terrain weights must sum to 255",
                )?;
                if index / axis == 0
                    || index / axis == axis - 1
                    || index % axis == 0
                    || index % axis == axis - 1
                {
                    require(
                        *w == crate::painting::GRASS,
                        "Terrain colour border must remain grass",
                    )?;
                }
            }
        }
        let start = self.origin()?;
        zone.check_position([start[0], zone.origin[1], start[2]], 0.)?;
        zone.check_position(
            [
                start[0] + self.extent_metres(),
                zone.origin[1],
                start[2] + self.extent_metres(),
            ],
            0.,
        )?;
        require(
            self.heights.iter().all(|h| h.is_finite() && h.abs() <= 64.),
            "Terrain height must be finite within +/-64 metres",
        )?;
        for z in 0..axis {
            for x in 0..axis {
                if x == 0 || z == 0 || x == axis - 1 || z == axis - 1 {
                    require(
                        self.heights[z * axis + x] == 0.,
                        "Terrain border must remain flat at zero",
                    )?;
                }
            }
        }
        Ok(())
    }
    /// Exact interpolation of the retained grid's [a,c,b] / [b,c,d] triangles.
    pub fn height(&self, x: f64, z: f64) -> Result<Option<f32>> {
        let axis = self.axis();
        let metres = self.sample_metres();
        let cells = (axis - 1) as f64;
        require(x.is_finite() && z.is_finite(), "Invalid terrain query")?;
        require(self.heights.len() == axis * axis, "Invalid terrain samples")?;
        let origin = self.origin()?;
        let u = (x - origin[0]) / metres;
        let v = (z - origin[2]) / metres;
        if !(0. ..=cells).contains(&u) || !(0. ..=cells).contains(&v) {
            return Ok(None);
        }
        let ix = (u.floor() as usize).min(axis - 2);
        let iz = (v.floor() as usize).min(axis - 2);
        let u = (u - ix as f64) as f32;
        let v = (v - iz as f64) as f32;
        let a = self.heights[iz * axis + ix];
        let b = self.heights[iz * axis + ix + 1];
        let c = self.heights[(iz + 1) * axis + ix];
        let d = self.heights[(iz + 1) * axis + ix + 1];
        Ok(Some(if u + v <= 1. {
            a * (1. - u - v) + b * u + c * v
        } else {
            b * (1. - v) + c * (1. - u) + d * (u + v - 1.)
        }))
    }
    /// Potential per-vertex influence shared by sculpt, paint and authoring feedback.
    /// Locked border samples are always zero; coordinates and radius use the command bounds.
    pub fn brush_influence(&self, centre: [f64; 2], radius_metres: f64) -> Result<Vec<f32>> {
        let axis = self.axis();
        let metres = self.sample_metres();
        require(self.heights.len() == axis * axis, "Invalid terrain samples")?;
        require(
            centre.iter().all(|v| v.is_finite())
                && radius_metres.is_finite()
                && (4. ..=32.).contains(&radius_metres),
            "Brush needs finite centre and 4..32 metre radius",
        )?;
        let origin = self.origin()?;
        require(
            (origin[0]..=origin[0] + self.extent_metres()).contains(&centre[0])
                && (origin[2]..=origin[2] + self.extent_metres()).contains(&centre[1]),
            "Brush centre outside terrain patch",
        )?;
        let mut influence = vec![0.; axis * axis];
        for z in 1..axis - 1 {
            for x in 1..axis - 1 {
                let dx = (origin[0] - centre[0]) + x as f64 * metres;
                let dz = (origin[2] - centre[1]) + z as f64 * metres;
                influence[z * axis + x] = (1. - dx.hypot(dz) / radius_metres).max(0.) as f32;
            }
        }
        Ok(influence)
    }
    pub(crate) fn apply(&mut self, brush: &Brush) -> Result<()> {
        let axis = self.axis();
        let influence = self.brush_influence(brush.centre, brush.radius_metres)?;
        match brush.operation {
            BrushOperation::Raise { metres } | BrushOperation::Lower { metres } => require(
                metres.is_finite() && (0.05..=8.).contains(&metres),
                "Brush amount must be 0.05..8 metres",
            )?,
            BrushOperation::Flatten { height, blend } => require(
                height.is_finite()
                    && height.abs() <= 64.
                    && blend.is_finite()
                    && (0.05..=1.).contains(&blend),
                "Invalid flatten height/blend",
            )?,
            BrushOperation::Smooth { blend } => require(
                blend.is_finite() && (0.05..=1.).contains(&blend),
                "Invalid smooth blend",
            )?,
        }
        let previous = self.heights.clone();
        let mut candidate = previous.clone();
        let mut changed = false;
        for z in 1..axis - 1 {
            for x in 1..axis - 1 {
                let weight = influence[z * axis + x];
                if weight == 0. {
                    continue;
                }
                let index = z * axis + x;
                let old = previous[index];
                let value = match brush.operation {
                    BrushOperation::Raise { metres } => old + metres * weight,
                    BrushOperation::Lower { metres } => old - metres * weight,
                    BrushOperation::Flatten { height, blend } => {
                        old + (height - old) * blend * weight
                    }
                    BrushOperation::Smooth { blend } => {
                        let average = (previous[index - 1]
                            + previous[index + 1]
                            + previous[index - axis]
                            + previous[index + axis])
                            / 4.;
                        old + (average - old) * blend * weight
                    }
                };
                require(
                    value.is_finite() && value.abs() <= 64.,
                    "Brush would exceed terrain height budget",
                )?;
                candidate[index] = value;
                changed |= value != old;
            }
        }
        require(changed, "Brush did not change interior samples")?;
        self.heights = candidate;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn stamp(operation: BrushOperation) -> Edit {
        Edit::Brush {
            brush: Brush {
                centre: [64., 64.],
                radius_metres: 16.,
                operation,
            },
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn brush_transaction_and_all_modes_share_undo_and_schema_round_trip() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let original = editor.zone().clone();
        apply(
            &mut editor,
            vec![
                Edit::SetTerrain {
                    terrain: Some(HeightPatch::flat([0, 0])),
                },
                stamp(BrushOperation::Raise { metres: 4. }),
            ],
        )
        .unwrap();
        assert_eq!(editor.zone().schema, 2);
        assert_eq!(
            editor.zone().terrain.as_ref().unwrap().heights[16 * AXIS + 16],
            4.
        );
        let raised = editor.zone().clone();
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&raised).unwrap()).unwrap(),
            raised
        );
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(editor.zone(), &original);
        editor
            .execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone(), &raised);
        apply(
            &mut editor,
            vec![
                stamp(BrushOperation::Lower { metres: 1. }),
                stamp(BrushOperation::Flatten {
                    height: 1.,
                    blend: 0.5,
                }),
                stamp(BrushOperation::Smooth { blend: 1. }),
            ],
        )
        .unwrap();
        let terrain = editor.zone().terrain.as_ref().unwrap();
        assert!(terrain.heights[16 * AXIS + 16] < 2.);
        for z in 0..AXIS {
            for x in 0..AXIS {
                if x == 0 || x == 32 || z == 0 || z == 32 {
                    assert_eq!(terrain.heights[z * AXIS + x], 0.);
                }
            }
        }
        assert_eq!(editor.history(), (2, 0));
    }
    #[test]
    fn invalid_or_over_budget_stroke_rolls_back_every_stamp() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(
            &mut editor,
            vec![Edit::SetTerrain {
                terrain: Some(HeightPatch::flat([0, 0])),
            }],
        )
        .unwrap();
        let original = editor.zone().clone();
        let edits = vec![stamp(BrushOperation::Raise { metres: 8. }); 9];
        assert!(apply(&mut editor, edits).is_err());
        assert_eq!(editor.zone(), &original);
        assert_eq!(editor.revision(), 1);
        assert!(
            apply(
                &mut editor,
                vec![stamp(BrushOperation::Raise { metres: f32::NAN })]
            )
            .is_err()
        );
        assert!(
            apply(
                &mut editor,
                vec![Edit::Brush {
                    brush: Brush {
                        centre: [999., 0.],
                        radius_metres: 8.,
                        operation: BrushOperation::Lower { metres: 1. }
                    }
                }]
            )
            .is_err()
        );
        assert_eq!(editor.history(), (1, 0));
    }
    #[test]
    fn exact_triangle_interpolation_and_far_signed_chunk_binding() {
        for sign in [-1_i64, 1] {
            let chunk = [sign * 1_000_000, -sign * 1_000_000];
            let mut patch = HeightPatch::flat(chunk);
            let origin = patch.origin().unwrap();
            let mut zone = Zone {
                origin,
                spawn: [origin[0], 0.1, origin[2]],
                ..Zone::default()
            };
            patch.heights[17 * AXIS + 17] = 4.;
            assert_eq!(
                patch.height(origin[0] + 65., origin[2] + 65.).unwrap(),
                Some(0.)
            );
            assert_eq!(
                patch.height(origin[0] + 67., origin[2] + 67.).unwrap(),
                Some(2.)
            );
            assert_eq!(
                patch.height(origin[0] + 128., origin[2] + 128.).unwrap(),
                Some(0.)
            );
            assert_eq!(patch.height(origin[0] - 0.01, origin[2]).unwrap(), None);
            patch
                .apply(&Brush {
                    centre: [origin[0] + 64., origin[2] + 64.],
                    radius_metres: 8.,
                    operation: BrushOperation::Raise { metres: 1. },
                })
                .unwrap();
            assert_eq!(patch.heights[16 * AXIS + 16], 1.);
            zone.schema = 2;
            zone.terrain = Some(patch);
            zone.validate().unwrap();
            assert_eq!(
                Zone::from_json(&serde_json::to_vec(&zone).unwrap()).unwrap(),
                zone
            );
        }
    }
    #[test]
    fn topology_height_borders_extent_and_schema_are_validated() {
        let mut zone = Zone {
            terrain: Some(HeightPatch::flat([0, 0])),
            ..Zone::default()
        };
        assert!(zone.validate().is_err());
        zone.schema = 2;
        zone.validate().unwrap();
        zone.terrain.as_mut().unwrap().heights[0] = 1.;
        assert!(zone.validate().is_err());
        zone.terrain = Some(HeightPatch::flat([4, 0]));
        assert!(zone.validate().is_err());
        zone.terrain = Some(HeightPatch::flat([0, 0]));
        zone.terrain.as_mut().unwrap().heights.pop();
        assert!(zone.validate().is_err());
    }
}

#[cfg(test)]
mod influence_tests {
    use super::*;
    use crate::{
        Command, Edit, Editor,
        painting::{Paint, Surface},
    };
    #[test]
    fn influence_matches_actual_raise_and_quantized_paint_on_both_far_grids() {
        for resolution in [Resolution::FourMetres, Resolution::TwoMetres] {
            for chunk in [
                [0, 0],
                [1_000_000_000, -1_000_000_000],
                [-1_000_000_000, 1_000_000_000],
            ] {
                let mut patch = HeightPatch::flat(chunk);
                patch.resolution = resolution;
                patch.heights = vec![0.; patch.axis() * patch.axis()];
                let origin = patch.origin().unwrap();
                let zone = Zone {
                    schema: 7,
                    origin,
                    spawn: [origin[0] + 16., 0.1, origin[2] + 16.],
                    terrain: Some(patch.clone()),
                    ..Zone::default()
                };
                for offset in [[64., 64.], [64.125, 65.375], [0., 64.]] {
                    let centre = [origin[0] + offset[0], origin[2] + offset[1]];
                    let influence = patch.brush_influence(centre, 8.).unwrap();
                    assert!(influence.iter().all(|&w| (0. ..=1.).contains(&w)));
                    for i in 0..patch.axis() {
                        assert_eq!(influence[i], 0.);
                        assert_eq!(influence[(patch.axis() - 1) * patch.axis() + i], 0.);
                        assert_eq!(influence[i * patch.axis()], 0.);
                        assert_eq!(influence[i * patch.axis() + patch.axis() - 1], 0.);
                    }
                    let mut editor = Editor::new(zone.clone()).unwrap();
                    editor
                        .execute(Command::Apply {
                            expected_revision: 0,
                            edits: vec![Edit::Brush {
                                brush: Brush {
                                    centre,
                                    radius_metres: 8.,
                                    operation: BrushOperation::Raise { metres: 1. },
                                },
                            }],
                        })
                        .unwrap();
                    assert_eq!(editor.zone().terrain.as_ref().unwrap().heights, influence);
                    editor
                        .execute(Command::Apply {
                            expected_revision: 1,
                            edits: vec![Edit::PaintTerrain {
                                paint: Paint {
                                    centre,
                                    radius_metres: 8.,
                                    surface: Surface::Dirt,
                                    blend: 1.,
                                },
                            }],
                        })
                        .unwrap();
                    for (weight, rgba) in influence
                        .iter()
                        .zip(&editor.zone().terrain.as_ref().unwrap().weights)
                    {
                        let grass = (255. * (1. - weight)).round() as u8;
                        assert_eq!(*rgba, [grass, 255 - grass, 0, 0]);
                    }
                    assert_eq!(editor.history(), (2, 0));
                }
            }
        }
    }
    #[test]
    fn influence_has_known_falloff_and_rejects_invalid_geometry_without_mutating() {
        let patch = HeightPatch::flat([0, 0]);
        let influence = patch.brush_influence([64., 64.], 8.).unwrap();
        assert_eq!(influence[16 * 33 + 16], 1.);
        assert_eq!(influence[16 * 33 + 17], 0.5);
        assert_eq!(influence[16 * 33 + 18], 0.);
        assert!((influence[17 * 33 + 17] - (1. - 2_f32.sqrt() / 2.)).abs() < 1e-7);
        for (centre, radius) in [
            ([f64::NAN, 64.], 8.),
            ([64., f64::INFINITY], 8.),
            ([64., 64.], f64::NAN),
            ([64., 64.], 3.9),
            ([64., 64.], 32.1),
            ([-0.01, 64.], 8.),
            ([64., 128.01], 8.),
        ] {
            assert!(patch.brush_influence(centre, radius).is_err());
        }
        let mut broken = patch.clone();
        broken.heights.pop();
        assert!(broken.brush_influence([64., 64.], 8.).is_err());
        assert_eq!(patch, HeightPatch::flat([0, 0]));
    }
}

#[cfg(test)]
mod expanded_tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    #[test]
    fn expanded_footprints_resample_preserve_surface_and_refuse_oversized_fine_grids_atomically() {
        let mut patch = HeightPatch::flat([62, 62]);
        patch.span_chunks = 2;
        patch.heights = vec![0.; 65 * 65];
        patch.weights = vec![crate::painting::GRASS; 65 * 65];
        patch.heights[40 * 65 + 40] = 4.;
        patch.weights[40 * 65 + 40] = [0, 255, 0, 0];
        let zone = Zone {
            schema: 11,
            origin: [8064., 0., 8064.],
            spawn: [8064., 0.1, 8064.],
            terrain: Some(patch),
            ..Zone::default()
        };
        zone.validate().unwrap();
        let fine = crate::resampling::resample(&zone, Resolution::TwoMetres).unwrap();
        assert_eq!(fine.axis(), 129);
        assert_eq!(fine.extent_metres(), 256.);
        assert_eq!(fine.height(8096., 8096.).unwrap(), Some(4.));
        assert_eq!(fine.weights[80 * 129 + 80], [0, 255, 0, 0]);
        let mut large = HeightPatch::flat([62, 62]);
        large.span_chunks = 4;
        large.heights = vec![0.; 129 * 129];
        large.weights = vec![crate::painting::GRASS; 129 * 129];
        let zone = Zone {
            schema: 11,
            origin: [8192., 0., 8192.],
            spawn: [8192., 0.1, 8192.],
            terrain: Some(large),
            ..Zone::default()
        };
        zone.validate().unwrap();
        let patch = zone.terrain.as_ref().unwrap();
        assert!(
            patch
                .brush_influence([8400., 8400.], 8.)
                .unwrap()
                .iter()
                .any(|v| *v > 0.)
        );
        assert_eq!(patch.height(8448., 8448.).unwrap(), Some(0.));
        assert_eq!(patch.height(8448.1, 8448.).unwrap(), None);
        let mut editor = Editor::new(zone.clone()).unwrap();
        assert!(
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::ResampleTerrain {
                        resolution: Resolution::TwoMetres
                    }]
                })
                .unwrap_err()
                .contains("sample budget")
        );
        assert_eq!(editor.zone(), &zone);
        assert_eq!(editor.history(), (0, 0));
        for span in [0, 5, 255] {
            let mut invalid = zone.clone();
            invalid.terrain.as_mut().unwrap().span_chunks = span;
            assert!(invalid.validate().is_err());
        }
        let mut old = zone.clone();
        old.schema = 10;
        assert!(old.validate().is_err());
        let original = HeightPatch::flat([0, 0]);
        let encoded = serde_json::to_string(&original).unwrap();
        assert!(!encoded.contains("span_chunks"));
        assert_eq!(
            serde_json::from_str::<HeightPatch>(&encoded).unwrap(),
            original
        );
    }
}
