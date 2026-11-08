//! Explicit terrain grounding; props do not support other placements.
use crate::{MAX_EDITS, Result, Zone, require};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Placement { id: u64 },
    GuidePoint { id: u64, index: usize },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub targets: Vec<Target>,
    pub offset_metres: f64,
}
impl Projection {
    pub(crate) fn apply(&self, zone: &mut Zone) -> Result<()> {
        // A previous edit in the same candidate may have supplied malformed terrain.
        zone.validate()?;
        require(
            !self.targets.is_empty() && self.targets.len() <= MAX_EDITS,
            "Grounding needs 1..128 targets",
        )?;
        require(
            self.offset_metres.is_finite() && self.offset_metres.abs() <= 16.,
            "Grounding offset must be within +/-16 metres",
        )?;
        let mut seen = BTreeSet::new();
        let mut changed = false;
        for target in &self.targets {
            require(seen.insert(target), "Duplicate grounding target")?;
            let (position, half_height) = match *target {
                Target::Placement { id } => {
                    if let Some(o) = zone.objects.iter().find(|o| o.id == id) {
                        (o.pose.position, o.pose.size[1] / 2.)
                    } else if let Some(p) = zone.plants.iter().find(|p| p.id == id) {
                        (p.position, 0.)
                    } else if let Some(m) = zone.markers.iter().find(|m| m.id == id) {
                        (m.position, 0.)
                    } else {
                        return Err("Unknown placement ID".into());
                    }
                }
                Target::GuidePoint { id, index } => (
                    *zone
                        .paths
                        .iter()
                        .find(|p| p.id == id)
                        .and_then(|p| p.points.get(index))
                        .ok_or("Unknown guide point")?,
                    0.,
                ),
            };
            let mut next = position;
            next[1] = zone.surface_position([position[0], position[2]])?[1]
                + half_height
                + self.offset_metres;
            changed |= next != position;
            match *target {
                Target::Placement { id } => {
                    if let Some(o) = zone.objects.iter_mut().find(|o| o.id == id) {
                        o.pose.position = next;
                    } else if let Some(p) = zone.plants.iter_mut().find(|p| p.id == id) {
                        p.position = next;
                    } else if let Some(m) = zone.markers.iter_mut().find(|m| m.id == id) {
                        m.position = next;
                    }
                }
                Target::GuidePoint { id, index } => {
                    zone.paths
                        .iter_mut()
                        .find(|p| p.id == id)
                        .ok_or("Unknown guide ID")?
                        .points[index] = next;
                }
            }
        }
        require(changed, "Selection already grounded")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Command, Edit, Editor, Object, Pose, Primitive,
        markers::{Marker, MarkerKind},
        paths::{GuidePath, PathKind},
        terrain::{Brush, BrushOperation, HeightPatch},
        vegetation::{Plant, PlantKind},
    };
    fn fixture(sign: i64) -> Zone {
        let terrain = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
        let origin = terrain.origin().unwrap();
        let point = [origin[0] + 64., 20., origin[2] + 64.];
        let mut zone = Zone {
            schema: 5,
            origin,
            terrain: Some(terrain),
            spawn: origin.map(|v| v + 0.1),
            objects: vec![Object {
                id: 1,
                name: "Box".into(),
                primitive: Primitive::Box,
                pose: Pose {
                    position: point,
                    yaw_degrees: 45.,
                    size: [2., 4., 2.],
                },
            }],
            plants: vec![Plant {
                id: 2,
                name: "Tree".into(),
                kind: PlantKind::Tree,
                position: point,
                yaw_degrees: 0.,
                scale: 1.,
            }],
            markers: vec![Marker {
                id: 3,
                name: "NPC".into(),
                kind: MarkerKind::Npc,
                position: point,
                yaw_degrees: 0.,
                radius_metres: 1.,
                definition: None,
            }],
            paths: vec![GuidePath {
                id: 4,
                name: "Road".into(),
                kind: PathKind::Road,
                width_metres: 2.,
                points: vec![point, [point[0] + 8., 20., point[2]]],
            }],
            ..Zone::default()
        };
        zone.terrain
            .as_mut()
            .unwrap()
            .apply(&Brush {
                centre: [point[0], point[2]],
                radius_metres: 16.,
                operation: BrushOperation::Raise { metres: 4. },
            })
            .unwrap();
        zone.validate().unwrap();
        zone
    }
    fn command(revision: u64, targets: Vec<Target>, offset_metres: f64) -> Command {
        Command::Apply {
            expected_revision: revision,
            edits: vec![Edit::ProjectToSurface {
                projection: Projection {
                    targets,
                    offset_metres,
                },
            }],
        }
    }
    #[test]
    fn grouped_grounding_uses_centres_and_feet_in_signed_far_frames_with_one_undo() {
        for sign in [-1, 0, 1] {
            let original = fixture(sign);
            let mut e = Editor::new(original.clone()).unwrap();
            let targets = vec![
                Target::Placement { id: 1 },
                Target::Placement { id: 2 },
                Target::Placement { id: 3 },
                Target::GuidePoint { id: 4, index: 0 },
            ];
            e.execute(command(0, targets.clone(), 0.25)).unwrap();
            let grounded = e.zone().clone();
            assert_eq!(grounded.objects[0].pose.position[1], 6.25);
            assert_eq!(grounded.plants[0].position[1], 4.25);
            assert_eq!(grounded.markers[0].position[1], 4.25);
            assert_eq!(grounded.paths[0].points[0][1], 4.25);
            assert_eq!(grounded.paths[0].points[1], original.paths[0].points[1]);
            let mut expected = original.clone();
            expected.objects[0].pose.position[1] = 6.25;
            expected.plants[0].position[1] = 4.25;
            expected.markers[0].position[1] = 4.25;
            expected.paths[0].points[0][1] = 4.25;
            assert_eq!(grounded, expected);
            assert_eq!(e.history(), (1, 0));
            assert!(e.execute(command(1, targets, 0.25)).is_err());
            assert_eq!(e.revision(), 1);
            e.execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
            assert_eq!(e.zone(), &original);
            e.execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
            assert_eq!(e.zone(), &grounded);
        }
    }
    #[test]
    fn invalid_group_preserves_document_revision_and_redo() {
        let mut e = Editor::new(fixture(0)).unwrap();
        e.execute(command(0, vec![Target::Placement { id: 1 }], 0.))
            .unwrap();
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        let original = e.zone().clone();
        for (targets, offset) in [
            (vec![], 0.),
            (vec![Target::Placement { id: 1 }; 129], 0.),
            (
                vec![Target::Placement { id: 1 }, Target::Placement { id: 999 }],
                0.,
            ),
            (vec![Target::Placement { id: 1 }; 2], 0.),
            (
                vec![Target::GuidePoint {
                    id: 4,
                    index: usize::MAX,
                }],
                0.,
            ),
            (vec![Target::Placement { id: 4 }], 0.),
            (vec![Target::Placement { id: 1 }], f64::NAN),
            (vec![Target::Placement { id: 1 }], 16.1),
        ] {
            assert!(e.execute(command(2, targets, offset)).is_err());
            assert_eq!(e.zone(), &original);
            assert_eq!(e.revision(), 2);
            assert_eq!(e.history(), (0, 1));
        }
        let mut broken = original.terrain.clone().unwrap();
        broken.heights.clear();
        assert!(
            e.execute(Command::Apply {
                expected_revision: 2,
                edits: vec![
                    Edit::SetTerrain {
                        terrain: Some(broken)
                    },
                    Edit::ProjectToSurface {
                        projection: Projection {
                            targets: vec![Target::Placement { id: 1 }],
                            offset_metres: 0.
                        }
                    }
                ]
            })
            .is_err()
        );
        assert_eq!(e.zone(), &original);
        assert_eq!(e.history(), (0, 1));
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
    }
    #[test]
    fn off_grid_anchor_samples_triangles_without_changing_xz_or_shape() {
        let mut zone = fixture(0);
        zone.objects[0].pose.position = [65., 20., 65.];
        let support = zone.surface_position([65., 65.]).unwrap()[1];
        assert!(support > 3. && support < 4.);
        let original = zone.objects[0].clone();
        let mut e = Editor::new(zone).unwrap();
        e.execute(command(0, vec![Target::Placement { id: 1 }], 0.))
            .unwrap();
        let mut expected = original;
        expected.pose.position[1] = support + 2.;
        assert_eq!(e.zone().objects[0], expected);
    }
    #[test]
    fn flat_fallback_and_structured_command_round_trip() {
        let mut zone = fixture(0);
        zone.terrain = None;
        let mut e = Editor::new(zone).unwrap();
        let c = command(0, vec![Target::Placement { id: 1 }], -0.5);
        let bytes = serde_json::to_vec(&c).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("project_to_surface"));
        e.execute(serde_json::from_slice(&bytes).unwrap()).unwrap();
        assert_eq!(e.zone().objects[0].pose.position[1], 1.5);
    }
}
