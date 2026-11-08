//! Nonblocking road/stream authoring guides. These are neither navigation links
//! nor water volumes, terrain carving or an authoritative route/quest system.
use super::{Result, Zone, label, require};
use serde::{Deserialize, Serialize};
pub const MAX_PATHS: usize = 128;
pub const MAX_POINTS: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathKind {
    Road,
    Stream,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuidePath {
    pub id: u64,
    pub name: String,
    pub kind: PathKind,
    pub width_metres: f64,
    /// Global f64 metres, explicit authored surface height at each point.
    pub points: Vec<[f64; 3]>,
}
impl GuidePath {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(self.id > 0 && label(&self.name), "Invalid path ID/name")?;
        require(
            self.width_metres.is_finite() && (0.25..=32.).contains(&self.width_metres),
            "Path width must be 0.25..32 metres",
        )?;
        require(
            (2..=MAX_POINTS).contains(&self.points.len()),
            "Path needs 2..64 control points",
        )?;
        for &point in &self.points {
            zone.check_position(point, self.width_metres / 2.)?;
        }
        for segment in self.points.windows(2) {
            let delta = std::array::from_fn::<_, 3, _>(|i| segment[1][i] - segment[0][i]);
            require(
                delta[0].hypot(delta[2]) >= 0.25
                    && delta.iter().map(|v| v * v).sum::<f64>().sqrt() <= 256.,
                "Path segments need >=0.25 m horizontal span and <=256 m length",
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn path(id: u64) -> GuidePath {
        GuidePath {
            id,
            name: "Road guide".into(),
            kind: PathKind::Road,
            width_metres: 4.,
            points: vec![[0., 0.03, 0.], [8., 0.03, 0.], [8., 0.03, 8.]],
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn paths_share_atomic_history_schema_and_stable_authoring_ids() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(&mut editor, vec![Edit::PutPath { path: path(100) }]).unwrap();
        let placed = editor.zone().clone();
        assert_eq!(placed.schema, 3);
        assert_eq!(placed.next_id(), Some(101));
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&placed).unwrap()).unwrap(),
            placed
        );
        let mut changed = path(100);
        changed.kind = PathKind::Stream;
        changed.width_metres = 2.;
        apply(
            &mut editor,
            vec![Edit::PutPath {
                path: changed.clone(),
            }],
        )
        .unwrap();
        assert_eq!(editor.zone().paths[0], changed);
        editor
            .execute(Command::Undo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone(), &placed);
        apply(&mut editor, vec![Edit::DeletePath { id: 100 }]).unwrap();
        assert!(editor.zone().paths.is_empty());
        editor
            .execute(Command::Undo {
                expected_revision: 4,
            })
            .unwrap();
        assert_eq!(editor.zone(), &placed);
    }
    #[test]
    fn malformed_paths_and_cross_kind_ids_refuse_without_history_changes() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(&mut editor, vec![Edit::PutPath { path: path(1) }]).unwrap();
        let before = editor.zone().clone();
        let mut candidate = path(2);
        candidate.points[1] = candidate.points[0];
        assert!(apply(&mut editor, vec![Edit::PutPath { path: candidate }]).is_err());
        let mut bad = path(2);
        bad.width_metres = f64::NAN;
        assert!(apply(&mut editor, vec![Edit::PutPath { path: bad }]).is_err());
        let mut bad = path(2);
        bad.points[1] = [511., 0., 0.];
        assert!(apply(&mut editor, vec![Edit::PutPath { path: bad }]).is_err());
        let mut bad = path(2);
        bad.points = vec![[0., 0., 0.]; MAX_POINTS + 1];
        assert!(apply(&mut editor, vec![Edit::PutPath { path: bad }]).is_err());
        let object = crate::Object {
            id: 1,
            name: "ID collision".into(),
            primitive: crate::Primitive::Box,
            pose: crate::Pose {
                position: [0., 0.5, 0.],
                yaw_degrees: 0.,
                size: [1.; 3],
            },
        };
        assert!(apply(&mut editor, vec![Edit::Place { object }]).is_err());
        assert!(apply(&mut editor, vec![Edit::DeletePath { id: 99 }]).is_err());
        assert_eq!(editor.zone(), &before);
        assert_eq!(editor.history(), (1, 0));
        assert_eq!(editor.revision(), 1);
    }
    #[test]
    fn signed_far_path_centimetres_preserve_precision_and_old_schemas_refuse_guides() {
        for sign in [-1., 1.] {
            let origin = [sign * 1e9, 0., -sign * 1e9];
            let mut guide = path(1);
            for point in &mut guide.points {
                point[0] += origin[0] + 0.01;
                point[2] += origin[2];
            }
            let mut zone = Zone {
                origin,
                spawn: [origin[0], 0.1, origin[2]],
                paths: vec![guide],
                ..Zone::default()
            };
            assert!(zone.validate().is_err());
            zone.schema = 3;
            zone.validate().unwrap();
            assert!((zone.local(zone.paths[0].points[0]).unwrap()[0] - 0.01).abs() < 0.000001);
            assert_eq!(
                Zone::from_json(&serde_json::to_vec(&zone).unwrap()).unwrap(),
                zone
            );
        }
    }
    #[test]
    fn combined_document_budget_refuses_unsavable_atomic_path_batch() {
        let origin = [500_000_000_000.123_4; 3];
        let objects = (1..=1024)
            .map(|id| crate::Object {
                id,
                name: "x".repeat(80),
                primitive: crate::Primitive::Box,
                pose: crate::Pose {
                    position: origin,
                    yaw_degrees: 0.,
                    size: [1.; 3],
                },
            })
            .collect();
        let zone = Zone {
            origin,
            spawn: origin,
            objects,
            ..Zone::default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        let edits = (1025..1153)
            .map(|id| Edit::PutPath {
                path: GuidePath {
                    id,
                    name: "p".repeat(80),
                    kind: PathKind::Road,
                    width_metres: 2.,
                    points: (0..64)
                        .map(|i| [origin[0] + i as f64 / 3., origin[1], origin[2]])
                        .collect(),
                },
            })
            .collect();
        assert!(
            apply(&mut editor, edits)
                .unwrap_err()
                .contains("byte budget")
        );
        assert_eq!(editor.zone(), &zone);
        assert_eq!(editor.revision(), 0);
        assert_eq!(editor.history(), (0, 0));
    }
}
