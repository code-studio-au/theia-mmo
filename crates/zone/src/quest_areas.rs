//! Explicit gameplay area drafts, separate from decorative quest markers.
use crate::{Result, Zone, require};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use theia_core::{content::ContentId, quests::areas};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestArea {
    pub id: u64,
    pub definition: String,
    pub name: String,
    /// Inclusive axis-aligned bounds in global f64 metres.
    pub min: [f64; 3],
    pub max: [f64; 3],
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub map_objectives: Vec<String>,
}
impl QuestArea {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(
            self.id > 0 && ContentId::new(&self.definition).is_some(),
            "Invalid quest area ID",
        )?;
        require(
            crate::label(&self.name) && self.name.trim() == self.name,
            "Invalid quest area name",
        )?;
        zone.check_position(self.min, 0.)?;
        zone.check_position(self.max, 0.)?;
        require(
            (0..3).all(|axis| {
                (areas::MIN_EXTENT_METRES..=areas::MAX_EXTENT_METRES)
                    .contains(&(self.max[axis] - self.min[axis]))
            }),
            "Invalid quest area extent",
        )?;
        require(
            self.map_objectives.len() <= areas::MAX_MAP_OBJECTIVES_PER_AREA,
            "Quest area objective link budget exceeded",
        )?;
        let mut ids = BTreeSet::new();
        require(
            self.map_objectives
                .iter()
                .all(|id| ContentId::new(id).is_some() && ids.insert(id)),
            "Invalid or duplicate quest area objective link",
        )
    }
}

/// Produce an unpublished runtime-format draft. Quest references are checked
/// separately against the intended executable quest catalog before publication.
pub fn draft_catalog(zone: &Zone, id: &str, revision: u32) -> Result<Vec<u8>> {
    zone.validate()?;
    let entries: Vec<_> = zone
        .quest_areas
        .iter()
        .map(|area| {
            serde_json::json!({
                "id":area.definition,"name":area.name,"min":area.min,"max":area.max,
                "map_objectives":area.map_objectives
            })
        })
        .collect();
    let bytes = serde_json::to_vec_pretty(
        &serde_json::json!({"schema":1,"id":id,"revision":revision,"areas":entries}),
    )
    .map_err(|e| e.to_string())?;
    areas::Catalog::from_json(&bytes).map_err(str::to_owned)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn area(origin: [f64; 3]) -> QuestArea {
        QuestArea {
            id: 40,
            definition: "theia.area.overlook".into(),
            name: "Overlook".into(),
            min: std::array::from_fn(|i| origin[i] + 0.125),
            max: std::array::from_fn(|i| origin[i] + 8.125),
            map_objectives: vec!["theia.quest.survey.reach".into()],
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn area_history_far_coordinates_and_runtime_draft_preserve_geometry() {
        let origin = [128_000_000_000., 4., -128_000_000_000.];
        let mut editor = Editor::new(Zone {
            origin,
            spawn: origin,
            ..Zone::default()
        })
        .unwrap();
        apply(&mut editor, vec![Edit::PutQuestArea { area: area(origin) }]).unwrap();
        assert_eq!(editor.zone().schema, 9);
        assert_eq!(editor.zone().next_id(), Some(41));
        let initial = editor.zone().clone();
        apply(
            &mut editor,
            vec![
                Edit::Translate {
                    id: 40,
                    offset: [0.25, 1., -0.5],
                },
                Edit::Rename {
                    id: 40,
                    name: "Western overlook".into(),
                },
            ],
        )
        .unwrap();
        let bytes = draft_catalog(editor.zone(), "theia.areas.starter", 1).unwrap();
        let runtime = areas::Catalog::from_json(&bytes).unwrap();
        let runtime_area = runtime.areas().next().unwrap();
        assert_eq!(
            runtime_area.min().metres(),
            editor.zone().quest_areas[0].min
        );
        assert_eq!(
            runtime_area.max().metres(),
            editor.zone().quest_areas[0].max
        );
        assert_eq!(runtime_area.map_objectives().len(), 1);
        let encoded = serde_json::to_vec(editor.zone()).unwrap();
        assert_eq!(Zone::from_json(&encoded).unwrap(), *editor.zone());
        editor
            .execute(Command::Undo {
                expected_revision: editor.revision(),
            })
            .unwrap();
        assert_eq!(editor.zone(), &initial);
        apply(&mut editor, vec![Edit::DeleteQuestArea { id: 40 }]).unwrap();
        assert!(editor.zone().quest_areas.is_empty());
        editor
            .execute(Command::Undo {
                expected_revision: editor.revision(),
            })
            .unwrap();
        assert_eq!(editor.zone(), &initial);
    }
    #[test]
    fn invalid_area_transactions_preserve_revision_document_and_redo() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(
            &mut editor,
            vec![Edit::PutQuestArea {
                area: area([0.; 3]),
            }],
        )
        .unwrap();
        apply(
            &mut editor,
            vec![Edit::Rename {
                id: 40,
                name: "Changed".into(),
            }],
        )
        .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: editor.revision(),
            })
            .unwrap();
        let before = editor.zone().clone();
        let revision = editor.revision();
        let mut bad = area([0.; 3]);
        bad.max[1] = bad.min[1];
        let mut duplicate = area([0.; 3]);
        duplicate.id = 41;
        let mut outside = area([0.; 3]);
        outside.max[0] = 513.;
        let mut links = area([0.; 3]);
        links.map_objectives.push(links.map_objectives[0].clone());
        for candidate in [bad, duplicate, outside, links] {
            assert!(
                apply(
                    &mut editor,
                    vec![
                        Edit::RenameZone {
                            name: "Must roll back".into()
                        },
                        Edit::PutQuestArea { area: candidate }
                    ]
                )
                .is_err()
            );
            assert_eq!(editor.zone(), &before);
            assert_eq!(editor.revision(), revision);
        }
        // Copying a gameplay area needs an explicit new definition, not an alias.
        assert!(
            apply(
                &mut editor,
                vec![Edit::Duplicate {
                    id: 40,
                    new_id: 41,
                    offset: [10., 0., 0.]
                }]
            )
            .is_err()
        );
        editor
            .execute(Command::Redo {
                expected_revision: revision,
            })
            .unwrap();
        assert_eq!(editor.zone().quest_areas[0].name, "Changed");
        let mut legacy = before;
        legacy.schema = 8;
        assert!(legacy.validate().is_err());
        assert!(draft_catalog(editor.zone(), "theia.area.overlook", 1).is_err());
    }
}
