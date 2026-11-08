//! Grade existing terrain samples along a committed guide; no water/navigation rules.
use super::{Result, Zone, require, terrain::HeightPatch};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grade {
    pub path_id: u64,
    /// Added to guide height before converting to height above zone-origin Y.
    pub surface_offset_metres: f64,
    pub falloff_metres: f64,
    pub blend: f64,
}
impl Grade {
    pub fn apply(&self, zone: &Zone) -> Result<HeightPatch> {
        require(
            self.surface_offset_metres.is_finite() && self.surface_offset_metres.abs() <= 16.,
            "Grade offset must be finite within +/-16 metres",
        )?;
        require(
            self.falloff_metres.is_finite() && (0. ..=32.).contains(&self.falloff_metres),
            "Grade falloff must be 0..32 metres",
        )?;
        require(
            self.blend.is_finite() && (0.05..=1.).contains(&self.blend),
            "Grade blend must be 0.05..1",
        )?;
        let path = zone
            .paths
            .iter()
            .find(|p| p.id == self.path_id)
            .ok_or("Unknown guide ID for grading")?;
        path.validate(zone)?;
        let patch = zone
            .terrain
            .as_ref()
            .ok_or("Create terrain before grading a guide")?;
        patch.validate(zone)?;
        let axis = patch.axis();
        let metres = patch.sample_metres();
        let origin = patch.origin()?;
        let mut candidate = patch.clone();
        let mut changed = false;
        for z in 1..axis - 1 {
            for x in 1..axis - 1 {
                // Subtract global anchors before local arithmetic, including at far origins.
                let mut nearest = f64::INFINITY;
                let mut target = 0.;
                for segment in path.points.windows(2) {
                    let a = segment[0];
                    let b = segment[1];
                    let dx = b[0] - a[0];
                    let dz = b[2] - a[2];
                    let px = (origin[0] - a[0]) + x as f64 * metres;
                    let pz = (origin[2] - a[2]) + z as f64 * metres;
                    let t = ((px * dx + pz * dz) / (dx * dx + dz * dz)).clamp(0., 1.);
                    let distance = (px - t * dx).hypot(pz - t * dz);
                    // At exact crossings the earlier authored segment wins deterministically.
                    if distance < nearest {
                        nearest = distance;
                        target = (a[1] - zone.origin[1])
                            + (b[1] - a[1]) * t
                            + self.surface_offset_metres;
                    }
                }
                let excess = (nearest - path.width_metres / 2.).max(0.);
                let weight = if excess == 0. {
                    1.
                } else if self.falloff_metres > 0. {
                    (1. - excess / self.falloff_metres).max(0.)
                } else {
                    0.
                };
                if weight == 0. {
                    continue;
                }
                require(
                    target.is_finite() && target.abs() <= 64.,
                    "Guide grade exceeds terrain height budget",
                )?;
                let index = z * axis + x;
                let old = patch.heights[index];
                let value =
                    (f64::from(old) + (target - f64::from(old)) * self.blend * weight) as f32;
                require(
                    value.is_finite() && value.abs() <= 64.,
                    "Guide grade exceeds terrain height budget",
                )?;
                candidate.heights[index] = value;
                changed |= value != old;
            }
        }
        require(
            changed,
            "Guide grading did not change interior terrain samples",
        )?;
        candidate.validate(zone)?;
        Ok(candidate)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, Edit, Editor,
        paths::{GuidePath, PathKind},
    };
    fn fixture() -> Zone {
        Zone {
            schema: 3,
            terrain: Some(HeightPatch::flat([0, 0])),
            paths: vec![GuidePath {
                id: 1,
                name: "Slope road".into(),
                kind: PathKind::Road,
                width_metres: 8.,
                points: vec![[32., 2.03, 64.], [96., 6.03, 64.]],
            }],
            ..Zone::default()
        }
    }
    fn grade() -> Grade {
        Grade {
            path_id: 1,
            surface_offset_metres: -0.03,
            falloff_metres: 8.,
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
    fn grade_interpolates_guide_heights_width_falloff_and_one_history_entry() {
        let zone = fixture();
        let mut e = Editor::new(zone.clone()).unwrap();
        apply(&mut e, vec![Edit::GradePath { grade: grade() }]).unwrap();
        let patch = e.zone().terrain.as_ref().unwrap();
        for (x, z, height) in [
            (48., 64., 3.),
            (64., 64., 4.),
            (64., 68., 4.),
            (64., 72., 2.),
            (64., 76., 0.),
        ] {
            assert!((patch.height(x, z).unwrap().unwrap() - height).abs() < 0.00001);
        }
        assert_eq!(patch.heights[0], 0.);
        assert_eq!(patch.heights[32 * 33 + 16], 0.);
        assert_eq!(e.zone().paths, zone.paths);
        assert_eq!(e.history(), (1, 0));
        let graded = e.zone().clone();
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&graded).unwrap()).unwrap(),
            graded
        );
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        assert_eq!(e.zone(), &zone);
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        assert_eq!(e.zone(), &graded);
        let partial = Grade {
            blend: 0.5,
            ..grade()
        }
        .apply(&zone)
        .unwrap();
        assert!((partial.height(64., 64.).unwrap().unwrap() - 2.).abs() < 0.00001);
    }
    #[test]
    fn grade_failures_preserve_whole_transaction_revision_and_redo() {
        let mut e = Editor::new(fixture()).unwrap();
        apply(&mut e, vec![Edit::GradePath { grade: grade() }]).unwrap();
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        let before = e.zone().clone();
        for g in [
            Grade {
                path_id: 999,
                ..grade()
            },
            Grade {
                blend: 0.,
                ..grade()
            },
            Grade {
                falloff_metres: -1.,
                ..grade()
            },
            Grade {
                surface_offset_metres: f64::NAN,
                ..grade()
            },
        ] {
            assert!(
                apply(
                    &mut e,
                    vec![
                        Edit::SetSpawn {
                            position: [4., 0.1, 4.]
                        },
                        Edit::GradePath { grade: g }
                    ]
                )
                .is_err()
            );
            assert_eq!(e.zone(), &before);
            assert_eq!((e.revision(), e.history()), (2, (0, 1)));
        }
        let mut high = before.paths[0].clone();
        high.points.iter_mut().for_each(|p| p[1] = 80.);
        assert!(
            apply(
                &mut e,
                vec![
                    Edit::PutPath { path: high },
                    Edit::GradePath { grade: grade() }
                ]
            )
            .is_err()
        );
        assert_eq!(e.zone(), &before);
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        let graded = e.zone().clone();
        assert!(
            apply(
                &mut e,
                vec![Edit::GradePath {
                    grade: Grade {
                        falloff_metres: 0.,
                        ..grade()
                    }
                }]
            )
            .is_err()
        );
        assert_eq!(e.zone(), &graded);
        let mut no_terrain = before.clone();
        no_terrain.terrain = None;
        assert!(grade().apply(&no_terrain).is_err());
        let mut remote = before;
        remote.paths[0].points = vec![[-100., 2., -100.], [-80., 2., -100.]];
        assert!(grade().apply(&remote).is_err());
    }
    #[test]
    fn grade_signed_far_origins_and_stream_lowering_use_checked_height_frame() {
        for sign in [-1_i64, 1] {
            let chunk = [sign * 1_000_000_000, -sign * 1_000_000_000];
            let patch = HeightPatch::flat(chunk);
            let mut origin = patch.origin().unwrap();
            origin[1] = 24.;
            let zone = Zone {
                schema: 3,
                origin,
                spawn: [origin[0] + 64., 24.1, origin[2] + 64.],
                terrain: Some(patch),
                paths: vec![GuidePath {
                    id: 1,
                    name: "Stream bed".into(),
                    kind: PathKind::Stream,
                    width_metres: 8.,
                    points: vec![
                        [origin[0] + 32., 22.03, origin[2] + 64.],
                        [origin[0] + 96., 18.03, origin[2] + 64.],
                    ],
                }],
                ..Zone::default()
            };
            zone.validate().unwrap();
            let patch = grade().apply(&zone).unwrap();
            assert!(
                (patch
                    .height(origin[0] + 64., origin[2] + 64.)
                    .unwrap()
                    .unwrap()
                    + 4.)
                    .abs()
                    < 0.00001
            );
            assert_eq!(patch.heights[0], 0.);
        }
    }
}
