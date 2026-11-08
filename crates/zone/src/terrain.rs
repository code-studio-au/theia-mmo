//! One 128 m, 33x33 height patch. Flat locked borders meet the implicit flat
//! ground. Full multi-tile seams/export/streaming are separate authoring slices.
use super::{Result, Zone, require};
use serde::{Deserialize, Serialize};
use theia_core::coordinates::ChunkId;
pub const AXIS: usize = 33;
pub const SAMPLE_COUNT: usize = AXIS * AXIS;
pub const TILE_METRES: f64 = 128.;
pub const SAMPLE_METRES: f64 = 4.;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeightPatch {
    pub chunk: [i64; 2],
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
            heights: vec![0.; SAMPLE_COUNT],
            weights: vec![],
        }
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
            self.heights.len() == SAMPLE_COUNT,
            "Terrain needs exactly 33x33 heights",
        )?;
        if !self.weights.is_empty() {
            require(zone.schema >= 6, "Terrain paint requires zone schema 6")?;
            require(
                self.weights.len() == SAMPLE_COUNT,
                "Terrain paint needs exactly 33x33 weights",
            )?;
            for (index, w) in self.weights.iter().enumerate() {
                require(
                    w.iter().map(|&v| u16::from(v)).sum::<u16>() == 255,
                    "Terrain weights must sum to 255",
                )?;
                if index / AXIS == 0
                    || index / AXIS == AXIS - 1
                    || index % AXIS == 0
                    || index % AXIS == AXIS - 1
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
                start[0] + TILE_METRES,
                zone.origin[1],
                start[2] + TILE_METRES,
            ],
            0.,
        )?;
        require(
            self.heights.iter().all(|h| h.is_finite() && h.abs() <= 64.),
            "Terrain height must be finite within +/-64 metres",
        )?;
        for z in 0..AXIS {
            for x in 0..AXIS {
                if x == 0 || z == 0 || x == AXIS - 1 || z == AXIS - 1 {
                    require(
                        self.heights[z * AXIS + x] == 0.,
                        "Terrain border must remain flat at zero",
                    )?;
                }
            }
        }
        Ok(())
    }
    /// Exact interpolation of the retained grid's [a,c,b] / [b,c,d] triangles.
    pub fn height(&self, x: f64, z: f64) -> Result<Option<f32>> {
        require(x.is_finite() && z.is_finite(), "Invalid terrain query")?;
        require(
            self.heights.len() == SAMPLE_COUNT,
            "Invalid terrain samples",
        )?;
        let origin = self.origin()?;
        let u = (x - origin[0]) / SAMPLE_METRES;
        let v = (z - origin[2]) / SAMPLE_METRES;
        if !(0. ..=32.).contains(&u) || !(0. ..=32.).contains(&v) {
            return Ok(None);
        }
        let ix = (u.floor() as usize).min(31);
        let iz = (v.floor() as usize).min(31);
        let u = (u - ix as f64) as f32;
        let v = (v - iz as f64) as f32;
        let a = self.heights[iz * AXIS + ix];
        let b = self.heights[iz * AXIS + ix + 1];
        let c = self.heights[(iz + 1) * AXIS + ix];
        let d = self.heights[(iz + 1) * AXIS + ix + 1];
        Ok(Some(if u + v <= 1. {
            a * (1. - u - v) + b * u + c * v
        } else {
            b * (1. - v) + c * (1. - u) + d * (u + v - 1.)
        }))
    }
    pub(crate) fn apply(&mut self, brush: &Brush) -> Result<()> {
        require(
            self.heights.len() == SAMPLE_COUNT,
            "Invalid terrain samples",
        )?;
        require(
            brush.centre.iter().all(|v| v.is_finite())
                && brush.radius_metres.is_finite()
                && (4. ..=32.).contains(&brush.radius_metres),
            "Brush needs finite centre and 4..32 metre radius",
        )?;
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
        let origin = self.origin()?;
        require(
            (origin[0]..=origin[0] + TILE_METRES).contains(&brush.centre[0])
                && (origin[2]..=origin[2] + TILE_METRES).contains(&brush.centre[1]),
            "Brush centre outside terrain patch",
        )?;
        let previous = self.heights.clone();
        let mut candidate = previous.clone();
        let mut changed = false;
        for z in 1..AXIS - 1 {
            for x in 1..AXIS - 1 {
                let dx = (origin[0] - brush.centre[0]) + x as f64 * SAMPLE_METRES;
                let dz = (origin[2] - brush.centre[1]) + z as f64 * SAMPLE_METRES;
                let weight = (1. - dx.hypot(dz) / brush.radius_metres).max(0.) as f32;
                if weight == 0. {
                    continue;
                }
                let index = z * AXIS + x;
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
                            + previous[index - AXIS]
                            + previous[index + AXIS])
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
