//! Bounded authoring references to particle effects, independent of the renderer.
//!
//! The effect is an unresolved catalogue key, not a filesystem path. Placement
//! alone does not imply asset packaging, runtime spawning or gameplay authority.
use serde::{Deserialize, Serialize};
use theia_core::content::ContentId;

use super::{Result, Zone, label, require};

pub const MAX_PARTICLE_PLACEMENTS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticlePlacement {
    pub id: u64,
    pub name: String,
    /// Stable unresolved catalogue key, never an operating-system path.
    pub effect: String,
    /// Global f64 anchor in Y-up metres.
    pub position: [f64; 3],
    pub yaw_degrees: f64,
    /// Dimensionless uniform scale; the effect's visual bounds are not known here.
    pub scale: f64,
}

impl ParticlePlacement {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(self.id > 0 && label(&self.name), "Invalid particle ID/name")?;
        require(
            ContentId::new(&self.effect).is_some(),
            "Particle effect needs a valid stable content ID",
        )?;
        require(
            self.yaw_degrees.is_finite() && self.yaw_degrees.abs() <= 360_000.,
            "Invalid particle yaw",
        )?;
        require(
            self.scale.is_finite() && (0.01..=64.).contains(&self.scale),
            "Particle scale must be 0.01..64",
        )?;
        // Only the anchor is bounded; emitter bounds depend on the effect asset.
        zone.check_position(self.position, 0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};

    fn placement(id: u64, position: [f64; 3]) -> ParticlePlacement {
        ParticlePlacement {
            id,
            name: "Campfire".into(),
            effect: "vfx.campfire".into(),
            position,
            yaw_degrees: 25.,
            scale: 1.,
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }

    #[test]
    fn placement_roundtrips_and_undoes_in_a_checked_far_frame() {
        let origin = [128_000_000_000., -4., -128_000_000_000.];
        let zone = Zone {
            origin,
            spawn: origin,
            ..Zone::default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        let particle = placement(7, [origin[0] + 0.01, origin[1], origin[2] + 0.02]);
        apply(
            &mut editor,
            vec![Edit::PutParticle {
                particle: particle.clone(),
            }],
        )
        .unwrap();
        assert_eq!(editor.zone().schema, 8);
        assert_eq!(editor.zone().next_id(), Some(8));
        assert!((editor.zone().local(particle.position).unwrap()[0] - 0.01).abs() < 0.00001);
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(editor.zone()).unwrap()).unwrap(),
            *editor.zone()
        );
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(editor.zone(), &zone);
        editor
            .execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
        apply(&mut editor, vec![Edit::DeleteParticle { id: 7 }]).unwrap();
        assert!(editor.zone().particles.is_empty());
    }

    #[test]
    fn invalid_placements_and_duplicate_ids_roll_back_atomically() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let mut cases = Vec::new();
        for effect in [
            "",
            "../secret",
            "/tmp/effect",
            "bad\\path",
            "a:b",
            "World",
            "world..tile",
        ] {
            let mut particle = placement(1, [0.; 3]);
            particle.effect = effect.into();
            cases.push(particle);
        }
        for (position, yaw, scale) in [
            ([f64::NAN, 0., 0.], 0., 1.),
            ([513., 0., 0.], 0., 1.),
            ([0.; 3], f64::INFINITY, 1.),
            ([0.; 3], 0., 0.),
        ] {
            let mut particle = placement(1, position);
            particle.yaw_degrees = yaw;
            particle.scale = scale;
            cases.push(particle);
        }
        for particle in cases {
            assert!(
                apply(
                    &mut editor,
                    vec![
                        Edit::SetSpawn {
                            position: [1., 0., 1.]
                        },
                        Edit::PutParticle { particle }
                    ]
                )
                .is_err()
            );
            assert_eq!(editor.zone(), &Zone::default());
            assert_eq!(editor.revision(), 0);
        }
        let mut zone = Zone {
            schema: 8,
            ..Zone::default()
        };
        zone.particles = vec![placement(1, [0.; 3]), placement(1, [1.; 3])];
        assert!(zone.validate().is_err());
        zone.schema = 7;
        zone.particles.pop();
        assert!(zone.validate().is_err());
        zone.schema = 8;
        zone.objects.push(crate::Object {
            id: 1,
            name: "Box".into(),
            primitive: crate::Primitive::Box,
            pose: crate::Pose {
                position: [0.; 3],
                yaw_degrees: 0.,
                size: [1.; 3],
            },
        });
        assert!(zone.validate().is_err());
    }

    #[test]
    fn older_documents_default_to_no_particle_placements() {
        let zone = Zone::default();
        let value = serde_json::to_value(&zone).unwrap();
        assert!(value.get("particles").is_none());
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(&value).unwrap()).unwrap(),
            zone
        );
    }
}
