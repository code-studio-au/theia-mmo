//! Quantized terrain colour weights; no gameplay surface/friction rules.
use super::{Result, require, terrain::HeightPatch};
use serde::{Deserialize, Serialize};
pub const GRASS: [u8; 4] = [255, 0, 0, 0];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Grass,
    Dirt,
    Rock,
    Sand,
}
impl Surface {
    pub fn index(self) -> usize {
        match self {
            Self::Grass => 0,
            Self::Dirt => 1,
            Self::Rock => 2,
            Self::Sand => 3,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paint {
    pub centre: [f64; 2],
    pub radius_metres: f64,
    pub surface: Surface,
    pub blend: f32,
}
impl Paint {
    pub(crate) fn apply(&self, patch: &mut HeightPatch) -> Result<()> {
        let axis = patch.axis();
        let count = axis * axis;
        require(
            self.radius_metres.is_finite() && (4. ..=32.).contains(&self.radius_metres),
            "Paint radius must be 4..32 metres",
        )?;
        let influence = patch.brush_influence(self.centre, self.radius_metres)?;
        require(
            self.blend.is_finite() && (0.05..=1.).contains(&self.blend),
            "Paint blend must be 0.05..1",
        )?;
        let mut candidate = if patch.weights.is_empty() {
            vec![GRASS; count]
        } else {
            patch.weights.clone()
        };
        require(
            candidate.len() == count,
            "Paint weights must match resolution",
        )?;
        require(
            candidate
                .iter()
                .all(|w| w.iter().map(|&v| u16::from(v)).sum::<u16>() == 255),
            "Terrain weights must sum to 255",
        )?;
        let mut changed = false;
        for z in 1..axis - 1 {
            for x in 1..axis - 1 {
                let amount = influence[z * axis + x] * self.blend;
                let index = z * axis + x;
                let old = candidate[index];
                let mut next = old;
                let target = self.surface.index();
                let mut others = 0_u16;
                for (i, w) in next.iter_mut().enumerate() {
                    if i != target {
                        *w = (f32::from(old[i]) * (1. - amount)).round() as u8;
                        others += u16::from(*w);
                    }
                }
                next[target] = (255 - others) as u8;
                changed |= next != old;
                candidate[index] = next;
            }
        }
        require(changed, "Paint did not change interior samples")?;
        patch.weights = candidate;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor, Zone};
    fn brush(surface: Surface) -> Paint {
        Paint {
            centre: [64., 64.],
            radius_metres: 16.,
            surface,
            blend: 1.,
        }
    }
    fn apply(e: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        e.execute(Command::Apply {
            expected_revision: e.revision(),
            edits,
        })
    }
    #[test]
    fn paint_weights_are_normalized_bordered_and_share_grouped_history() {
        let original = Zone {
            schema: 2,
            terrain: Some(HeightPatch::flat([0, 0])),
            ..Zone::default()
        };
        let mut e = Editor::new(original.clone()).unwrap();
        apply(
            &mut e,
            vec![
                Edit::PaintTerrain {
                    paint: brush(Surface::Dirt),
                },
                Edit::PaintTerrain {
                    paint: Paint {
                        blend: 0.5,
                        ..brush(Surface::Rock)
                    },
                },
            ],
        )
        .unwrap();
        assert_eq!(e.zone().schema, 6);
        let patch = e.zone().terrain.as_ref().unwrap();
        assert_eq!(patch.heights, original.terrain.as_ref().unwrap().heights);
        assert_eq!(patch.weights[16 * 33 + 16], [0, 128, 127, 0]);
        for (i, w) in patch.weights.iter().enumerate() {
            assert_eq!(w.iter().map(|&v| u16::from(v)).sum::<u16>(), 255);
            if i / 33 == 0 || i / 33 == 32 || i % 33 == 0 || i % 33 == 32 {
                assert_eq!(*w, GRASS);
            }
        }
        let painted = e.zone().clone();
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&painted).unwrap()).unwrap(),
            painted
        );
        assert_eq!(e.history(), (1, 0));
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        assert_eq!(e.zone(), &original);
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        assert_eq!(e.zone(), &painted);
    }
    #[test]
    fn paint_invalid_data_and_noops_preserve_document_revision_and_redo() {
        let mut e = Editor::new(Zone {
            schema: 2,
            terrain: Some(HeightPatch::flat([0, 0])),
            ..Zone::default()
        })
        .unwrap();
        assert!(
            apply(
                &mut e,
                vec![Edit::PaintTerrain {
                    paint: brush(Surface::Grass)
                }]
            )
            .is_err()
        );
        apply(
            &mut e,
            vec![Edit::PaintTerrain {
                paint: brush(Surface::Sand),
            }],
        )
        .unwrap();
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        let before = e.zone().clone();
        for paint in [
            Paint {
                radius_metres: 0.,
                ..brush(Surface::Dirt)
            },
            Paint {
                blend: f32::NAN,
                ..brush(Surface::Dirt)
            },
            Paint {
                centre: [999., 0.],
                ..brush(Surface::Dirt)
            },
        ] {
            assert!(
                apply(
                    &mut e,
                    vec![
                        Edit::SetSpawn {
                            position: [4., 0.1, 4.]
                        },
                        Edit::PaintTerrain { paint }
                    ]
                )
                .is_err()
            );
            assert_eq!(e.zone(), &before);
            assert_eq!((e.revision(), e.history()), (2, (0, 1)));
        }
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        let stable = e.zone().clone();
        let mut invalid = stable.terrain.clone().unwrap();
        invalid.weights[16 * 33 + 16] = [255; 4];
        assert!(
            apply(
                &mut e,
                vec![
                    Edit::SetTerrain {
                        terrain: Some(invalid)
                    },
                    Edit::PaintTerrain {
                        paint: brush(Surface::Dirt)
                    }
                ]
            )
            .is_err()
        );
        assert_eq!(e.zone(), &stable);
        let mut malformed = e.zone().clone();
        malformed.terrain.as_mut().unwrap().weights[5] = [0; 4];
        assert!(malformed.validate().is_err());
        let mut old = e.zone().clone();
        old.schema = 5;
        assert!(old.validate().is_err());
        let mut border = e.zone().clone();
        border.terrain.as_mut().unwrap().weights[0] = [0, 255, 0, 0];
        assert!(border.validate().is_err());
        let mut truncated = e.zone().clone();
        truncated.terrain.as_mut().unwrap().weights.pop();
        assert!(truncated.validate().is_err());
    }
    #[test]
    fn paint_far_signed_frames_and_all_channels_keep_exact_integer_weight_sum() {
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
            for surface in [Surface::Dirt, Surface::Rock, Surface::Sand, Surface::Grass] {
                apply(
                    &mut e,
                    vec![Edit::PaintTerrain {
                        paint: Paint {
                            centre: [origin[0] + 64., origin[2] + 64.],
                            surface,
                            blend: 0.37,
                            ..brush(surface)
                        },
                    }],
                )
                .unwrap();
                for w in &e.zone().terrain.as_ref().unwrap().weights {
                    assert_eq!(w.iter().map(|&v| u16::from(v)).sum::<u16>(), 255);
                }
            }
        }
    }
}
