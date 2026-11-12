//! Editor test pawns; these are launch configuration, never gameplay NPCs.
use serde::{Deserialize, Serialize};
use theia_core::content::ContentId;

use crate::{Result, Zone, label, require};

pub const MAX_TEST_PLAYERS: usize = 2;

/// Artificial per-connection conditions, applied by the local UDP relay.
/// Delay applies in each direction; measured RTT is a separate runtime value.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConditions {
    pub delay_ms: u32,
    pub jitter_ms: u32,
    pub loss_percent: f64,
    pub seed: u32,
    /// Ordered [start_ms, duration_ms] windows relative to relay startup.
    pub outages: Vec<[u32; 2]>,
}
impl NetworkConditions {
    pub fn validate(&self) -> Result<()> {
        require(
            self.delay_ms <= 5000 && self.jitter_ms <= 5000,
            "Test delay/jitter must be within 0..5000 ms",
        )?;
        require(
            self.loss_percent.is_finite() && (0. ..=100.).contains(&self.loss_percent),
            "Test packet loss must be within 0..100 percent",
        )?;
        require(self.outages.len() <= 16, "Test outage budget exceeded")?;
        let mut previous_end = 0;
        for &[start, duration] in &self.outages {
            let end = u64::from(start) + u64::from(duration);
            require(
                start >= previous_end && (1..=60_000).contains(&duration) && end <= 3_600_000,
                "Test outages must be ordered, nonoverlapping and within one hour",
            )?;
            previous_end = end as u32;
        }
        Ok(())
    }
    pub fn added_base_rtt_ms(&self) -> u64 {
        u64::from(self.delay_ms) * 2
    }
}

fn enabled() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestPlayer {
    /// Shared authoring ID; distinct from the authenticated development slot.
    pub id: u64,
    pub name: String,
    pub client_id: u8,
    #[serde(default = "enabled")]
    pub enabled: bool,
    /// Global f64 feet in Y-up metres. Server checks support/capsule at launch.
    pub position: [f64; 3],
    /// Stable controller catalogue ID; None means normal manual input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    #[serde(default)]
    pub network: NetworkConditions,
}
impl TestPlayer {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(
            self.id > 0 && label(&self.name),
            "Invalid test player ID/name",
        )?;
        require(
            (1..=2).contains(&self.client_id),
            "Test player slot must be 1 or 2",
        )?;
        require(
            self.controller
                .as_ref()
                .is_none_or(|id| ContentId::new(id).is_some()),
            "Test controller needs a stable content ID",
        )?;
        self.network.validate()?;
        zone.check_position(self.position, 2.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor, Object, Pose, Primitive};

    fn player(position: [f64; 3]) -> TestPlayer {
        TestPlayer {
            id: 10,
            name: "Player one".into(),
            client_id: 1,
            enabled: true,
            position,
            controller: Some("test.walk-turn".into()),
            network: NetworkConditions {
                delay_ms: 80,
                jitter_ms: 20,
                seed: 7,
                ..Default::default()
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
    fn pawns_roundtrip_preserve_far_coordinates_and_use_transactional_shared_commands() {
        for sign in [-1., 0., 1.] {
            let origin = [sign * 128_000_000_000., 20., -sign * 128_000_000_000.];
            let mut editor = Editor::new(Zone {
                origin,
                spawn: origin,
                ..Default::default()
            })
            .unwrap();
            let original = editor.zone().clone();
            let position = [origin[0] + 0.125, origin[1] + 2., origin[2] - 0.25];
            apply(
                &mut editor,
                vec![Edit::PutTestPlayer {
                    player: player(position),
                }],
            )
            .unwrap();
            assert_eq!(editor.zone().schema, 12);
            assert!(!crate::starter_basin::available(editor.zone()));
            assert_eq!(editor.zone().next_id(), Some(11));
            apply(
                &mut editor,
                vec![
                    Edit::Rename {
                        id: 10,
                        name: "Delayed client".into(),
                    },
                    Edit::Translate {
                        id: 10,
                        offset: [0.25, 0., 0.],
                    },
                    Edit::Duplicate {
                        id: 10,
                        new_id: 11,
                        offset: [0., 0., 2.],
                    },
                    Edit::ProjectToSurface {
                        projection: crate::projection::Projection {
                            targets: vec![crate::projection::Target::Placement { id: 11 }],
                            offset_metres: 0.008,
                        },
                    },
                ],
            )
            .unwrap();
            let first = &editor.zone().test_players[0];
            let second = &editor.zone().test_players[1];
            assert_eq!(
                first.position,
                [origin[0] + 0.375, origin[1] + 2., origin[2] - 0.25]
            );
            assert_eq!(
                second.position,
                [origin[0] + 0.375, origin[1] + 0.008, origin[2] + 1.75]
            );
            assert_eq!((first.client_id, second.client_id), (1, 2));
            assert_eq!(first.network, second.network);
            assert_eq!(first.controller, second.controller);
            assert_eq!(
                editor.zone().authoring_name(11),
                Some("Delayed client copy")
            );
            let saved = editor.zone().clone();
            assert_eq!(
                Zone::from_json(&serde_json::to_vec(&saved).unwrap()).unwrap(),
                saved
            );
            assert!(
                apply(
                    &mut editor,
                    vec![Edit::Duplicate {
                        id: 10,
                        new_id: 12,
                        offset: [1., 0., 0.]
                    }]
                )
                .is_err()
            );
            assert_eq!(editor.zone(), &saved);
            apply(
                &mut editor,
                vec![
                    Edit::DeleteTestPlayer { id: 10 },
                    Edit::DeleteTestPlayer { id: 11 },
                ],
            )
            .unwrap();
            assert!(editor.zone().test_players.is_empty());
            editor
                .execute(Command::Undo {
                    expected_revision: editor.revision(),
                })
                .unwrap();
            assert_eq!(editor.zone(), &saved);
            editor
                .execute(Command::Undo {
                    expected_revision: editor.revision(),
                })
                .unwrap();
            editor
                .execute(Command::Undo {
                    expected_revision: editor.revision(),
                })
                .unwrap();
            assert_eq!(editor.zone(), &original);
            editor
                .execute(Command::Redo {
                    expected_revision: editor.revision(),
                })
                .unwrap();
            assert_eq!(editor.zone().test_players, vec![player(position)]);
        }
    }
    #[test]
    fn disabled_pawns_keep_unique_slots_and_all_invalid_commands_roll_back() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(
            &mut editor,
            vec![Edit::PutTestPlayer {
                player: player([0., 0., 0.]),
            }],
        )
        .unwrap();
        let before = editor.zone().clone();
        let history = editor.history();
        for mut invalid in [
            TestPlayer {
                client_id: 0,
                ..player([0.; 3])
            },
            TestPlayer {
                client_id: 3,
                ..player([0.; 3])
            },
            TestPlayer {
                enabled: false,
                ..player([0.; 3])
            },
            TestPlayer {
                client_id: 2,
                controller: Some("../controller.json".into()),
                ..player([0.; 3])
            },
            TestPlayer {
                client_id: 2,
                ..player([f64::NAN, 0., 0.])
            },
            TestPlayer {
                client_id: 2,
                ..player([511., 0., 0.])
            },
        ] {
            invalid.id = 11;
            assert!(
                apply(
                    &mut editor,
                    vec![
                        Edit::RenameZone {
                            name: "Must roll back".into()
                        },
                        Edit::PutTestPlayer { player: invalid }
                    ]
                )
                .is_err()
            );
            assert_eq!(editor.zone(), &before);
            assert_eq!(editor.history(), history);
        }
        assert!(
            apply(
                &mut editor,
                vec![Edit::Place {
                    object: Object {
                        id: 10,
                        name: "ID collision".into(),
                        primitive: Primitive::Box,
                        pose: Pose {
                            position: [2.; 3],
                            yaw_degrees: 0.,
                            size: [1.; 3]
                        },
                    }
                }]
            )
            .is_err()
        );
        assert_eq!(editor.zone(), &before);
        let mut disabled = player([0.; 3]);
        disabled.enabled = false;
        disabled.controller = None;
        apply(
            &mut editor,
            vec![Edit::PutTestPlayer {
                player: disabled.clone(),
            }],
        )
        .unwrap();
        assert_eq!(editor.zone().test_players, vec![disabled]);
    }
    #[test]
    fn old_documents_stay_unchanged_and_schema_gate_refuses_test_pawns() {
        let zone = Zone::default();
        let bytes = serde_json::to_vec(&zone).unwrap();
        assert!(
            !String::from_utf8(bytes.clone())
                .unwrap()
                .contains("test_players")
        );
        assert_eq!(Zone::from_json(&bytes).unwrap(), zone);
        let mut value = serde_json::to_value(zone).unwrap();
        value["test_players"] = serde_json::json!([player([0.; 3])]);
        assert!(Zone::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
        value["schema"] = 12.into();
        value["test_players"][0]
            .as_object_mut()
            .unwrap()
            .remove("enabled");
        value["test_players"][0]
            .as_object_mut()
            .unwrap()
            .remove("network");
        let parsed = Zone::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(parsed.test_players[0].enabled);
        assert_eq!(parsed.test_players[0].network, NetworkConditions::default());
        value["test_players"][0]["teleport"] = true.into();
        assert!(Zone::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn connection_budgets_match_local_relay_and_reject_overflowing_outages() {
        let mut conditions = NetworkConditions {
            delay_ms: 80,
            jitter_ms: 20,
            loss_percent: 2.5,
            seed: u32::MAX,
            outages: vec![[0, 1000], [1000, 60_000], [3_599_999, 1]],
        };
        conditions.validate().unwrap();
        assert_eq!(conditions.added_base_rtt_ms(), 160);
        for bad in [
            NetworkConditions {
                delay_ms: 5001,
                ..conditions.clone()
            },
            NetworkConditions {
                jitter_ms: 5001,
                ..conditions.clone()
            },
            NetworkConditions {
                loss_percent: f64::NAN,
                ..conditions.clone()
            },
            NetworkConditions {
                loss_percent: -1.,
                ..conditions.clone()
            },
            NetworkConditions {
                loss_percent: 100.1,
                ..conditions.clone()
            },
        ] {
            assert!(bad.validate().is_err());
        }
        for windows in [
            vec![[0, 0]],
            vec![[0, 60_001]],
            vec![[100, 10], [109, 1]],
            vec![[u32::MAX, u32::MAX]],
            vec![[3_600_000, 1]],
            vec![[0, 1]; 17],
        ] {
            conditions.outages = windows;
            assert!(conditions.validate().is_err());
        }
    }
}
