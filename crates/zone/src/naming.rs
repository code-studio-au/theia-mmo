//! Narrow metadata edits retain identity, geometry and annotation definitions.
use crate::{Result, Zone};

impl Zone {
    pub fn authoring_name(&self, id: u64) -> Option<&str> {
        self.objects
            .iter()
            .map(|o| (o.id, o.name.as_str()))
            .chain(self.paths.iter().map(|p| (p.id, p.name.as_str())))
            .chain(self.markers.iter().map(|m| (m.id, m.name.as_str())))
            .chain(self.plants.iter().map(|p| (p.id, p.name.as_str())))
            .chain(self.particles.iter().map(|p| (p.id, p.name.as_str())))
            .chain(self.quest_areas.iter().map(|a| (a.id, a.name.as_str())))
            .chain(self.scenery.iter().map(|p| (p.id, p.name.as_str())))
            .chain(self.test_players.iter().map(|p| (p.id, p.name.as_str())))
            .find_map(|(key, name)| (key == id).then_some(name))
    }
}

pub(crate) fn apply(zone: &mut Zone, id: u64, name: String) -> Result<()> {
    let target = zone
        .objects
        .iter_mut()
        .map(|o| (o.id, &mut o.name))
        .chain(zone.paths.iter_mut().map(|p| (p.id, &mut p.name)))
        .chain(zone.markers.iter_mut().map(|m| (m.id, &mut m.name)))
        .chain(zone.plants.iter_mut().map(|p| (p.id, &mut p.name)))
        .chain(zone.particles.iter_mut().map(|p| (p.id, &mut p.name)))
        .chain(zone.quest_areas.iter_mut().map(|a| (a.id, &mut a.name)))
        .chain(zone.scenery.iter_mut().map(|p| (p.id, &mut p.name)))
        .chain(zone.test_players.iter_mut().map(|p| (p.id, &mut p.name)))
        .find_map(|(key, name)| (key == id).then_some(name))
        .ok_or("Unknown authoring ID to rename")?;
    *target = name;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Command, Edit, Editor, Zone};
    fn editor() -> Editor {
        let mut editor = Editor::new(Zone::default()).unwrap();
        editor
            .execute(
                serde_json::from_slice(include_bytes!(
                    "../../../content/zones/selection.commands.jsonl"
                ))
                .unwrap(),
            )
            .unwrap();
        editor
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> crate::Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn all_names_share_one_transaction_without_changing_other_data() {
        let mut editor = editor();
        let before = editor.zone().clone();
        let mut expected = before.clone();
        expected.name = "北の谷".into();
        expected.objects[0].name = "Renamed wall".into();
        expected.paths[0].name = "Renamed road".into();
        expected.markers[0].name = "Renamed note".into();
        expected.plants[0].name = "Renamed tree".into();
        apply(
            &mut editor,
            vec![
                Edit::RenameZone {
                    name: expected.name.clone(),
                },
                Edit::Rename {
                    id: 1,
                    name: expected.objects[0].name.clone(),
                },
                Edit::Rename {
                    id: 1002,
                    name: expected.paths[0].name.clone(),
                },
                Edit::Rename {
                    id: 1000,
                    name: expected.markers[0].name.clone(),
                },
                Edit::Rename {
                    id: 1001,
                    name: expected.plants[0].name.clone(),
                },
            ],
        )
        .unwrap();
        assert_eq!(editor.zone(), &expected);
        assert_eq!(editor.history(), (2, 0));
        assert_eq!(editor.zone().authoring_name(1002), Some("Renamed road"));
        assert_eq!(editor.zone().authoring_name(0), None);
        editor
            .execute(Command::Undo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone(), &before);
        editor
            .execute(Command::Redo {
                expected_revision: 3,
            })
            .unwrap();
        assert_eq!(editor.zone(), &expected);
        assert_eq!(
            serde_json::from_str::<Zone>(&serde_json::to_string(editor.zone()).unwrap()).unwrap(),
            expected
        );
    }
    #[test]
    fn invalid_names_missing_ids_and_stale_transactions_preserve_redo() {
        let mut editor = editor();
        apply(
            &mut editor,
            vec![Edit::RenameZone {
                name: "Temporary".into(),
            }],
        )
        .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 2,
            })
            .unwrap();
        let before = editor.zone().clone();
        for name in [
            "".into(),
            "  ".into(),
            "bad\nname".into(),
            "x".repeat(81),
            "é".repeat(41),
        ] {
            for edit in [
                Edit::RenameZone { name: name.clone() },
                Edit::Rename { id: 1, name },
            ] {
                assert!(
                    apply(
                        &mut editor,
                        vec![
                            Edit::Rename {
                                id: 1002,
                                name: "Must roll back".into()
                            },
                            edit
                        ]
                    )
                    .is_err()
                );
                assert_eq!(editor.zone(), &before);
                assert_eq!(editor.revision(), 3);
                assert_eq!(editor.history(), (1, 1));
            }
        }
        for id in [0, 9999] {
            assert!(
                apply(
                    &mut editor,
                    vec![Edit::Rename {
                        id,
                        name: "Unknown".into()
                    }]
                )
                .is_err()
            );
        }
        assert!(
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::RenameZone {
                        name: "Stale".into()
                    }]
                })
                .is_err()
        );
        assert_eq!(editor.zone(), &before);
        assert_eq!(editor.history(), (1, 1));
        editor
            .execute(Command::Redo {
                expected_revision: 3,
            })
            .unwrap();
        assert_eq!(editor.zone().name, "Temporary");
    }
    #[test]
    fn name_limit_counts_utf8_bytes_and_serde_keeps_commands_narrow() {
        let mut editor = editor();
        let name = "é".repeat(40);
        let command: Command = serde_json::from_value(serde_json::json!({"command":"apply","expected_revision":1,"edits":[{"op":"rename","id":1,"name":name},{"op":"rename_zone","name":"🌲".repeat(20)}]})).unwrap();
        editor.execute(command).unwrap();
        assert_eq!(editor.zone().authoring_name(1).unwrap().len(), 80);
        assert_eq!(editor.zone().name.len(), 80);
        assert!(
            serde_json::from_str::<Edit>(r#"{"op":"rename","id":1,"name":"x","position":[0,0,0]}"#)
                .is_err()
        );
    }
}
