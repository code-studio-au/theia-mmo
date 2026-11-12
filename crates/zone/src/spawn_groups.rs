//! Saved population authoring drafts. These do not create runtime NPCs; export
//! must resolve definitions, navigation/collision candidates and population caps.
use super::{Result, Zone, label, require};
use serde::{Deserialize, Serialize};
use theia_core::{content::ContentId, spawn_volume::PolygonVolume};

pub const MAX_GROUPS: usize = 16;
pub const MAX_ALIVE_PER_GROUP: u16 = 64;
pub const MAX_REQUESTED_ALIVE: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnGroup {
    pub id: u64,
    pub name: String,
    pub enabled: bool,
    /// Logical gameplay definition, not a path or a live entity ID.
    pub creature: String,
    /// Optional logical appearance binding, independent of combat/clearance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub volume: PolygonVolume,
    pub target_alive: u16,
    pub respawn_seconds: u32,
    pub seed: u64,
}
impl SpawnGroup {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(
            self.id > 0 && label(&self.name),
            "Invalid spawn group ID/name",
        )?;
        require(
            ContentId::new(&self.creature).is_some(),
            "Spawn group needs a logical creature definition",
        )?;
        require(
            self.model
                .as_ref()
                .is_none_or(|id| ContentId::new(id).is_some()),
            "Spawn model needs a logical asset ID",
        )?;
        require(
            (1..=MAX_ALIVE_PER_GROUP).contains(&self.target_alive),
            "Spawn count must be 1..64",
        )?;
        require(
            (1..=86400).contains(&self.respawn_seconds),
            "Respawn delay must be 1..86400 seconds",
        )?;
        for &[x, z] in self.volume.polygon() {
            zone.check_position([x, self.volume.min_y(), z], 0.)?;
            zone.check_position([x, self.volume.max_y(), z], 0.)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn group(origin: [f64; 3]) -> SpawnGroup {
        SpawnGroup {
            id: 10,
            name: "Western wolves".into(),
            enabled: true,
            creature: "theia.dev.training-creature".into(),
            model: Some("theia.test.wolf".into()),
            volume: PolygonVolume::new(
                vec![[0., 0.], [8., 0.], [8., 3.], [3., 3.], [3., 8.], [0., 8.]],
                -2.,
                8.,
            )
            .unwrap()
            .translated(origin)
            .unwrap(),
            target_alive: 5,
            respawn_seconds: 30,
            seed: 42,
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn groups_round_trip_and_share_far_translation_copy_name_and_atomic_history() {
        for origin in [[0.; 3], [128_000_000_000.125, 20.25, -128_000_000_000.5]] {
            let initial = Zone {
                origin,
                spawn: [origin[0], origin[1] + 0.1, origin[2]],
                ..Zone::default()
            };
            let mut editor = Editor::new(initial.clone()).unwrap();
            let original = group(origin);
            apply(
                &mut editor,
                vec![Edit::PutSpawnGroup {
                    group: original.clone(),
                }],
            )
            .unwrap();
            assert_eq!(editor.zone().schema, 14);
            assert_eq!(editor.zone().next_id(), Some(11));
            assert_eq!(editor.zone().authoring_name(10), Some("Western wolves"));
            assert_eq!(
                Zone::from_json(&serde_json::to_vec(editor.zone()).unwrap()).unwrap(),
                *editor.zone()
            );
            let offset = [0.125, 0.25, -0.5];
            apply(
                &mut editor,
                vec![
                    Edit::Translate { id: 10, offset },
                    Edit::Rename {
                        id: 10,
                        name: "Grove wolves".into(),
                    },
                    Edit::Duplicate {
                        id: 10,
                        new_id: 11,
                        offset,
                    },
                ],
            )
            .unwrap();
            let first = &editor.zone().spawn_groups[0];
            assert_eq!(first.volume, original.volume.translated(offset).unwrap());
            assert_eq!(first.creature, original.creature);
            assert_eq!(first.model, original.model);
            assert_eq!(first.target_alive, 5);
            assert_eq!(first.respawn_seconds, 30);
            assert_eq!(first.seed, 42);
            let second = &editor.zone().spawn_groups[1];
            assert_eq!(second.volume, first.volume.translated(offset).unwrap());
            assert_eq!(second.name, "Grove wolves copy");
            editor
                .execute(Command::Undo {
                    expected_revision: 2,
                })
                .unwrap();
            assert_eq!(editor.zone().spawn_groups, vec![original.clone()]);
            editor
                .execute(Command::Undo {
                    expected_revision: 3,
                })
                .unwrap();
            assert_eq!(editor.zone(), &initial);
            editor
                .execute(Command::Redo {
                    expected_revision: 4,
                })
                .unwrap();
            assert_eq!(editor.zone().spawn_groups, vec![original]);
            apply(&mut editor, vec![Edit::DeleteSpawnGroup { id: 10 }]).unwrap();
            assert!(editor.zone().spawn_groups.is_empty());
        }
    }
    #[test]
    fn invalid_group_edits_schema_ids_and_population_budgets_fail_atomically() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let original = group([0.; 3]);
        apply(
            &mut editor,
            vec![Edit::PutSpawnGroup {
                group: original.clone(),
            }],
        )
        .unwrap();
        let before = editor.zone().clone();
        let mut old = before.clone();
        old.schema = 13;
        assert!(old.validate().is_err());
        for (case, mut invalid) in [
            original.clone(),
            original.clone(),
            original.clone(),
            original.clone(),
            original.clone(),
        ]
        .into_iter()
        .enumerate()
        {
            match case {
                0 => invalid.target_alive = 0,
                1 => invalid.target_alive = 65,
                2 => invalid.respawn_seconds = 0,
                3 => invalid.model = Some("../wolf.glb".into()),
                _ => invalid.volume = invalid.volume.translated([600., 0., 0.]).unwrap(),
            }
            assert!(
                apply(
                    &mut editor,
                    vec![
                        Edit::RenameZone {
                            name: "must roll back".into()
                        },
                        Edit::PutSpawnGroup { group: invalid }
                    ]
                )
                .is_err()
            );
            assert_eq!(editor.zone(), &before);
            assert_eq!(editor.history(), (1, 0));
        }
        assert!(
            apply(
                &mut editor,
                vec![Edit::Duplicate {
                    id: 10,
                    new_id: 10,
                    offset: [0.; 3]
                }]
            )
            .is_err()
        );
        let mut excessive = before.clone();
        excessive.spawn_groups = (1..=5)
            .map(|id| {
                let mut g = original.clone();
                g.id = id;
                g.target_alive = 64;
                g
            })
            .collect();
        assert!(
            excessive
                .validate()
                .unwrap_err()
                .contains("population budget")
        );
        excessive
            .spawn_groups
            .iter_mut()
            .for_each(|g| g.enabled = false);
        excessive.validate().unwrap();
        assert_eq!(editor.zone(), &before);
    }
}
