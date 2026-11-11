//! Narrow f64 offsets preserve authoring identity and all non-position metadata.
use crate::{Result, ZONE_EXTENT, Zone, require};

pub(crate) fn apply(zone: &mut Zone, id: u64, offset: [f64; 3]) -> Result<()> {
    require(
        offset
            .iter()
            .all(|v| v.is_finite() && v.abs() <= 2. * ZONE_EXTENT),
        "Translation offset must be finite and within +/-1024 metres",
    )?;
    let shift = |position: [f64; 3]| std::array::from_fn(|i| position[i] + offset[i]);
    if let Some(object) = zone.objects.iter_mut().find(|o| o.id == id) {
        object.pose.position = shift(object.pose.position);
    } else if let Some(path) = zone.paths.iter_mut().find(|p| p.id == id) {
        for point in &mut path.points {
            *point = shift(*point);
        }
    } else if let Some(marker) = zone.markers.iter_mut().find(|m| m.id == id) {
        marker.position = shift(marker.position);
    } else if let Some(plant) = zone.plants.iter_mut().find(|p| p.id == id) {
        plant.position = shift(plant.position);
    } else if let Some(particle) = zone.particles.iter_mut().find(|p| p.id == id) {
        particle.position = shift(particle.position);
    } else if let Some(area) = zone.quest_areas.iter_mut().find(|a| a.id == id) {
        area.min = shift(area.min);
        area.max = shift(area.max);
    } else {
        return Err("Unknown authoring ID to translate".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn seed(origin: [f64; 3]) -> Zone {
        let mut editor = Editor::new(Zone::default()).unwrap();
        editor
            .execute(
                serde_json::from_slice(include_bytes!(
                    "../../../content/zones/selection.commands.jsonl"
                ))
                .unwrap(),
            )
            .unwrap();
        let mut zone = editor.zone().clone();
        let shift = |p: [f64; 3]| std::array::from_fn(|i| p[i] + origin[i]);
        zone.origin = shift(zone.origin);
        zone.spawn = shift(zone.spawn);
        for object in &mut zone.objects {
            object.pose.position = shift(object.pose.position);
        }
        for path in &mut zone.paths {
            for p in &mut path.points {
                *p = shift(*p);
            }
        }
        for marker in &mut zone.markers {
            marker.position = shift(marker.position);
            marker.definition = Some("quest.authoring-note".into());
        }
        for plant in &mut zone.plants {
            plant.position = shift(plant.position);
            plant.yaw_degrees = 37.5;
        }
        zone.validate().unwrap();
        zone
    }
    fn command(revision: u64, edits: Vec<Edit>) -> Command {
        Command::Apply {
            expected_revision: revision,
            edits,
        }
    }
    #[test]
    fn mixed_offsets_equal_legacy_edits_preserve_metadata_and_undo_once() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let original = seed(origin);
            let offset = [0.125, 0.25, -0.5];
            let shift = |p: [f64; 3]| std::array::from_fn(|i| p[i] + offset[i]);
            let mut edits = vec![];
            for object in &original.objects {
                let mut pose = object.pose.clone();
                pose.position = shift(pose.position);
                edits.push(Edit::Transform {
                    id: object.id,
                    pose,
                });
            }
            for path in &original.paths {
                let mut path = path.clone();
                path.points = path.points.into_iter().map(shift).collect();
                edits.push(Edit::PutPath { path });
            }
            for marker in &original.markers {
                let mut marker = marker.clone();
                marker.position = shift(marker.position);
                edits.push(Edit::PutMarker { marker });
            }
            for plant in &original.plants {
                let mut plant = plant.clone();
                plant.position = shift(plant.position);
                edits.push(Edit::PutPlant { plant });
            }
            let mut legacy = Editor::new(original.clone()).unwrap();
            legacy.execute(command(0, edits)).unwrap();
            let mut editor = Editor::new(original.clone()).unwrap();
            let edits = [1, 2, 3, 1000, 1001, 1002]
                .into_iter()
                .map(|id| Edit::Translate { id, offset })
                .collect();
            let encoded = serde_json::to_vec(&command(0, edits)).unwrap();
            editor
                .execute(serde_json::from_slice(&encoded).unwrap())
                .unwrap();
            assert_eq!(editor.zone(), legacy.zone());
            assert_eq!(editor.zone().origin, original.origin);
            assert_eq!(editor.zone().spawn, original.spawn);
            assert_eq!(editor.zone().next_id(), original.next_id());
            assert_eq!(editor.history(), (1, 0));
            let changed = editor.zone().clone();
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
            assert_eq!(editor.zone(), &changed);
        }
    }
    #[test]
    fn invalid_offsets_targets_and_stale_batches_preserve_source_and_redo() {
        let original = seed([128_000_000_000.125, 20.25, -128_000_000_000.5]);
        let mut editor = Editor::new(original.clone()).unwrap();
        editor
            .execute(command(
                0,
                vec![Edit::Translate {
                    id: 1,
                    offset: [0.125, 0., 0.],
                }],
            ))
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let history = editor.history();
        for edits in [
            vec![Edit::Translate {
                id: 1,
                offset: [f64::NAN, 0., 0.],
            }],
            vec![Edit::Translate {
                id: 1,
                offset: [f64::INFINITY, 0., 0.],
            }],
            vec![Edit::Translate {
                id: 1,
                offset: [1025., 0., 0.],
            }],
            vec![Edit::Translate {
                id: 1,
                offset: [1024., 0., 0.],
            }],
            vec![
                Edit::Translate {
                    id: 1,
                    offset: [0.125, 0., 0.],
                },
                Edit::Translate {
                    id: 9999,
                    offset: [1., 0., 0.],
                },
            ],
            vec![Edit::Translate {
                id: 0,
                offset: [1., 0., 0.],
            }],
        ] {
            assert!(editor.execute(command(2, edits)).is_err());
            assert_eq!(editor.zone(), &original);
            assert_eq!(editor.revision(), 2);
            assert_eq!(editor.history(), history);
        }
        assert!(
            editor
                .execute(command(
                    1,
                    vec![Edit::Translate {
                        id: 1,
                        offset: [1., 0., 0.]
                    }]
                ))
                .is_err()
        );
        assert_eq!(editor.zone(), &original);
        assert_eq!(editor.history(), history);
        editor
            .execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
        assert_ne!(editor.zone(), &original);
    }
    #[test]
    fn mailbox_translation_retry_and_shared_undo_do_not_double_apply() {
        use crate::mailbox::{Mailbox, Request};
        let directory = std::env::temp_dir().join(format!(
            "theia-translate-mailbox-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut mailbox = Mailbox::open(&directory).unwrap();
        let original = seed([128_000_000_000.125, 20.25, -128_000_000_000.5]);
        let mut editor = Editor::new(original.clone()).unwrap();
        let request = Request {
            session: mailbox.session().into(),
            request_id: 1,
            command: command(
                0,
                vec![Edit::Translate {
                    id: 1001,
                    offset: [0.125, 0., -0.5],
                }],
            ),
        };
        let bytes = serde_json::to_vec(&request).unwrap();
        let submit = |bytes: &[u8]| std::fs::write(directory.join("request.json"), bytes).unwrap();
        submit(&bytes);
        let receipt = mailbox.poll(&mut editor, false).unwrap().unwrap();
        assert!(receipt.accepted);
        assert_eq!(receipt.revision, 1);
        let changed = editor.zone().clone();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        submit(&bytes);
        assert_eq!(mailbox.poll(&mut editor, false).unwrap().unwrap(), receipt);
        assert_eq!(editor.zone(), &original);
        assert_eq!(editor.revision(), 2);
        let redo = Request {
            request_id: 2,
            command: Command::Redo {
                expected_revision: 2,
            },
            ..request.clone()
        };
        submit(&serde_json::to_vec(&redo).unwrap());
        assert!(mailbox.poll(&mut editor, false).unwrap().unwrap().accepted);
        assert_eq!(editor.zone(), &changed);
        assert_eq!(editor.history(), (1, 0));
        let busy = Request {
            request_id: 3,
            command: command(
                3,
                vec![Edit::Translate {
                    id: 1000,
                    offset: [1., 0., 0.],
                }],
            ),
            ..request
        };
        submit(&serde_json::to_vec(&busy).unwrap());
        let refused = mailbox.poll(&mut editor, true).unwrap().unwrap();
        assert!(!refused.accepted);
        assert_eq!(editor.zone(), &changed);
        assert_eq!(editor.revision(), 3);
        let published: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("state.json")).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_value::<Zone>(published["document"].clone()).unwrap(),
            changed
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn json_offsets_are_strict_and_transaction_budget_stays_bounded() {
        for invalid in [
            r#"{"op":"translate","id":1,"offset":[1,2]}"#,
            r#"{"op":"translate","id":1,"offset":[1,2,3],"position":[1,2,3]}"#,
        ] {
            assert!(serde_json::from_str::<Edit>(invalid).is_err());
        }
        let original = seed([0.; 3]);
        let mut editor = Editor::new(original.clone()).unwrap();
        assert!(
            editor
                .execute(command(
                    0,
                    vec![
                        Edit::Translate {
                            id: 1,
                            offset: [0.; 3]
                        };
                        crate::MAX_EDITS + 1
                    ]
                ))
                .is_err()
        );
        assert_eq!(editor.zone(), &original);
        assert_eq!(editor.history(), (0, 0));
        assert_eq!(editor.revision(), 0);
    }
}
