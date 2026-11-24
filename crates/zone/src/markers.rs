//! Authoring annotations, separate from runtime NPCs and quest definitions.
use super::{Result, Zone, label, require};
use serde::{Deserialize, Serialize};
pub const MAX_MARKERS: usize = 256;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerKind {
    Npc,
    Quest,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marker {
    pub id: u64,
    pub name: String,
    pub kind: MarkerKind,
    /// Global f64 anchor, Y-up metres. NPC anchors represent feet.
    pub position: [f64; 3],
    pub yaw_degrees: f64,
    /// Authoring footprint only; not a hitbox or a gameplay trigger.
    pub radius_metres: f64,
    /// Optional unresolved catalogue key. Never a filesystem path or live entity ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
}
impl Marker {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(self.id > 0 && label(&self.name), "Invalid marker ID/name")?;
        require(
            self.yaw_degrees.is_finite() && self.yaw_degrees.abs() <= 360000.,
            "Invalid marker yaw",
        )?;
        require(
            self.radius_metres.is_finite() && (0.25..=64.).contains(&self.radius_metres),
            "Marker radius must be 0.25..64 metres",
        )?;
        require(
            self.definition.as_ref().is_none_or(|s| {
                !s.is_empty()
                    && s.len() <= 96
                    && s.bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            }),
            "Marker definition needs a 1..96 byte ASCII catalogue key",
        )?;
        zone.check_position(self.position, self.radius_metres.max(2.))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn marker(id: u64) -> Marker {
        Marker {
            id,
            name: "Guide NPC".into(),
            kind: MarkerKind::Npc,
            position: [3., 0., 4.],
            yaw_degrees: 15.,
            radius_metres: 1.,
            definition: Some("human.guide".into()),
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn markers_share_schema_history_ids_and_checked_far_frames() {
        let origin = [128_000_000_000., -4., -128_000_000_000.];
        let mut zone = Zone {
            origin,
            spawn: origin.map(|v| v + 0.1),
            ..Zone::default()
        };
        let mut m = marker(500);
        m.position = [origin[0] + 0.01, origin[1], origin[2] + 0.02];
        zone.schema = 4;
        zone.markers.push(m.clone());
        let mut e = Editor::new(zone.clone()).unwrap();
        assert_eq!(e.zone().next_id(), Some(501));
        assert!((e.zone().local(m.position).unwrap()[0] - 0.01).abs() < 0.00001);
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&zone).unwrap()).unwrap(),
            zone
        );
        m.kind = MarkerKind::Quest;
        m.radius_metres = 8.;
        apply(&mut e, vec![Edit::PutMarker { marker: m.clone() }]).unwrap();
        assert_eq!(e.zone().markers, [m]);
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        assert_eq!(e.zone(), &zone);
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        apply(&mut e, vec![Edit::DeleteMarker { id: 500 }]).unwrap();
        assert!(e.zone().markers.is_empty());
        e.execute(Command::Undo {
            expected_revision: 4,
        })
        .unwrap();
        assert_eq!(e.zone().markers.len(), 1);
        // Adding earlier-version features must never downgrade schema 4.
        apply(
            &mut e,
            vec![Edit::PutPath {
                path: crate::paths::GuidePath {
                    id: 501,
                    name: "Road".into(),
                    kind: crate::paths::PathKind::Road,
                    width_metres: 2.,
                    points: vec![origin, [origin[0] + 4., origin[1], origin[2]]],
                },
            }],
        )
        .unwrap();
        assert_eq!(e.zone().schema, 4);
    }
    #[test]
    fn invalid_markers_roll_back_whole_transactions_and_preserve_redo() {
        let mut e = Editor::new(Zone::default()).unwrap();
        apply(&mut e, vec![Edit::PutMarker { marker: marker(1) }]).unwrap();
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        let mut cases = vec![];
        let mut m = marker(2);
        m.radius_metres = 0.;
        cases.push(m);
        let mut m = marker(2);
        m.position[0] = 512.;
        cases.push(m);
        let mut m = marker(2);
        m.yaw_degrees = f64::NAN;
        cases.push(m);
        let mut m = marker(2);
        m.definition = Some("../unsafe".into());
        cases.push(m);
        let mut m = marker(2);
        m.definition = Some(String::new());
        cases.push(m);
        let mut m = marker(2);
        m.name = "control\n".into();
        cases.push(m);
        for m in cases {
            let before = e.zone().clone();
            assert!(
                apply(
                    &mut e,
                    vec![
                        Edit::SetSpawn {
                            position: [8., 0.1, 8.]
                        },
                        Edit::PutMarker { marker: m }
                    ]
                )
                .is_err()
            );
            assert_eq!(e.zone(), &before);
            assert_eq!(e.revision(), 2);
            assert_eq!(e.history(), (0, 1));
        }
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        let before = e.zone().clone();
        assert!(
            apply(
                &mut e,
                vec![Edit::Place {
                    object: crate::Object {
                        id: 1,
                        name: "Box".into(),
                        primitive: crate::Primitive::Box,
                        pose: crate::Pose {
                            position: [0., 0.5, 0.],
                            size: [1.; 3],
                            yaw_degrees: 0.
                        }
                    }
                }]
            )
            .is_err()
        );
        assert_eq!(e.zone(), &before);
        assert!(apply(&mut e, vec![Edit::DeleteMarker { id: 999 }]).is_err());
        let mut old = before;
        old.schema = 3;
        assert!(old.validate().is_err());
    }
    #[test]
    fn marker_count_is_bounded_before_history_mutation() {
        let mut e = Editor::new(Zone::default()).unwrap();
        for start in [1, 129] {
            apply(
                &mut e,
                (start..start + 128)
                    .map(|id| Edit::PutMarker { marker: marker(id) })
                    .collect(),
            )
            .unwrap();
        }
        assert_eq!(e.zone().markers.len(), MAX_MARKERS);
        let before = e.zone().clone();
        assert!(
            apply(
                &mut e,
                vec![Edit::PutMarker {
                    marker: marker(257)
                }]
            )
            .is_err()
        );
        assert_eq!(e.zone(), &before);
        assert_eq!((e.revision(), e.history()), (2, (2, 0)));
    }
}
