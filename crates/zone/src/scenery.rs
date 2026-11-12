//! Bounded authoring references to static scenery assets, independent of the renderer.
//!
//! The asset is an unresolved catalogue key, not a filesystem path. Placement
//! alone does not imply asset packaging, runtime spawning or gameplay authority.
use serde::{Deserialize, Serialize};
use theia_core::content::ContentId;

use super::{Result, Zone, label, require};

pub const MAX_SCENERY_PLACEMENTS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneryPlacement {
    pub id: u64,
    pub name: String,
    /// Stable unresolved catalogue key, never an operating-system path.
    pub asset: String,
    /// Global f64 anchor in Y-up metres.
    pub position: [f64; 3],
    pub yaw_degrees: f64,
    /// Dimensionless uniform scale; the asset's visual bounds are not known here.
    pub scale: f64,
}

impl SceneryPlacement {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(self.id > 0 && label(&self.name), "Invalid scenery ID/name")?;
        require(
            ContentId::new(&self.asset).is_some(),
            "Scenery asset needs a valid stable content ID",
        )?;
        require(
            self.yaw_degrees.is_finite() && self.yaw_degrees.abs() <= 360_000.,
            "Invalid scenery yaw",
        )?;
        require(
            self.scale.is_finite() && (0.01..=64.).contains(&self.scale),
            "Scenery scale must be 0.01..64",
        )?;
        // Only the anchor is bounded; model bounds depend on the resolved asset.
        zone.check_position(self.position, 0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    fn placement() -> SceneryPlacement {
        SceneryPlacement {
            id: 4,
            name: "Forest archway".into(),
            asset: "synty.forest.archway".into(),
            position: [0.125, 0., -0.25],
            yaw_degrees: 45.,
            scale: 1.,
        }
    }
    #[test]
    fn model_references_roundtrip_share_ids_history_and_exact_far_offsets() {
        let origin = [128_000_000_000., 0., -128_000_000_000.];
        let zone = Zone {
            origin,
            spawn: origin,
            ..Zone::default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        let mut scenery = placement();
        scenery.position = std::array::from_fn(|i| origin[i] + scenery.position[i]);
        apply(
            &mut editor,
            vec![Edit::PutScenery {
                scenery: scenery.clone(),
            }],
        )
        .unwrap();
        assert_eq!(editor.zone().schema, 10);
        assert_eq!(editor.zone().next_id(), Some(5));
        apply(
            &mut editor,
            vec![
                Edit::Rename {
                    id: 4,
                    name: "Entry arch".into(),
                },
                Edit::Translate {
                    id: 4,
                    offset: [0.25, 0., 0.],
                },
                Edit::Duplicate {
                    id: 4,
                    new_id: 5,
                    offset: [0., 0., 1.],
                },
            ],
        )
        .unwrap();
        assert_eq!(editor.zone().scenery[1].asset, scenery.asset);
        assert_eq!(
            editor.zone().scenery[1].position,
            [origin[0] + 0.375, 0., origin[2] + 0.75]
        );
        assert_eq!(editor.zone().authoring_name(5), Some("Entry arch copy"));
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(editor.zone()).unwrap()).unwrap(),
            *editor.zone()
        );
        editor
            .execute(Command::Undo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone().scenery, vec![scenery.clone()]);
        let before = editor.zone().clone();
        scenery.asset = "../asset.glb".into();
        assert!(apply(&mut editor, vec![Edit::PutScenery { scenery }]).is_err());
        assert_eq!(editor.zone(), &before);
        assert_eq!(editor.history(), (1, 1));
        editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        assert_eq!(editor.zone(), &zone);
    }
    #[test]
    fn malformed_placements_and_namespace_collisions_refuse_atomically() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        for (scale, yaw, x) in [(0., 0., 0.), (1., f64::NAN, 0.), (1., 0., 513.)] {
            let mut scenery = placement();
            scenery.scale = scale;
            scenery.yaw_degrees = yaw;
            scenery.position[0] = x;
            assert!(apply(&mut editor, vec![Edit::PutScenery { scenery }]).is_err());
            assert_eq!(editor.revision(), 0);
        }
        apply(
            &mut editor,
            vec![Edit::PutScenery {
                scenery: placement(),
            }],
        )
        .unwrap();
        let before = editor.zone().clone();
        assert!(
            apply(
                &mut editor,
                vec![Edit::PutParticle {
                    particle: crate::particles::ParticlePlacement {
                        id: 4,
                        name: "Collision".into(),
                        effect: "vfx.test".into(),
                        position: [0.; 3],
                        yaw_degrees: 0.,
                        scale: 1.,
                    }
                }]
            )
            .is_err()
        );
        assert_eq!(editor.zone(), &before);
        apply(&mut editor, vec![Edit::DeleteScenery { id: 4 }]).unwrap();
        assert!(editor.zone().scenery.is_empty());
    }
}
