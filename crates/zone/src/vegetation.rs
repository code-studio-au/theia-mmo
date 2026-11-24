//! Bounded greybox vegetation. Static visual proxies, independent of collision.
use super::{Result, Zone, label, require};
use serde::{Deserialize, Serialize};
pub const MAX_PLANTS: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlantKind {
    Tree,
    Shrub,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plant {
    pub id: u64,
    pub name: String,
    pub kind: PlantKind,
    pub position: [f64; 3],
    pub yaw_degrees: f64,
    /// Dimensionless uniform scale of the fixed metre-sized greybox proxy.
    pub scale: f64,
}
impl Plant {
    pub fn validate(&self, zone: &Zone) -> Result<()> {
        require(self.id > 0 && label(&self.name), "Invalid plant ID/name")?;
        require(
            self.scale.is_finite() && (0.25..=8.).contains(&self.scale),
            "Plant scale must be 0.25..8",
        )?;
        require(
            self.yaw_degrees.is_finite() && self.yaw_degrees.abs() <= 360000.,
            "Invalid plant yaw",
        )?;
        zone.check_position(
            self.position,
            self.scale * if self.kind == PlantKind::Tree { 5. } else { 2. },
        )
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scatter {
    pub first_id: u64,
    pub seed: u64,
    pub kind: PlantKind,
    pub centre: [f64; 2],
    pub radius_metres: f64,
    pub count: u32,
}
impl Scatter {
    pub fn generate(&self, zone: &Zone) -> Result<Vec<Plant>> {
        require(
            self.first_id > 0 && (1..=64).contains(&self.count),
            "Scatter needs a positive ID and 1..64 plants",
        )?;
        require(
            zone.plants.len() + self.count as usize <= MAX_PLANTS,
            "Vegetation budget exceeded",
        )?;
        require(
            self.radius_metres.is_finite() && (1.0..=32.0).contains(&self.radius_metres),
            "Scatter radius must be 1..32 metres",
        )?;
        zone.check_position(
            [self.centre[0], zone.origin[1], self.centre[1]],
            self.radius_metres + 6.,
        )?;
        let mut state = self.seed;
        // Explicit SplitMix64 arithmetic plus rejection sampling in the unit disk.
        // No global RNG, trig, iteration-order dependence or per-frame spawning.
        let mut random = || {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut v = state;
            v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
            (((v ^ (v >> 31)) >> 11) as f64) * (1. / ((1_u64 << 53) as f64))
        };
        let mut plants = Vec::new();
        for _ in 0..1024 {
            let x = random() * 2. - 1.;
            let z = random() * 2. - 1.;
            if x * x + z * z > 1. {
                continue;
            }
            let id = self
                .first_id
                .checked_add(plants.len() as u64)
                .ok_or("Plant IDs exhausted")?;
            let position = zone.surface_position([
                self.centre[0] + x * self.radius_metres,
                self.centre[1] + z * self.radius_metres,
            ])?;
            let plant = Plant {
                id,
                name: format!("{:?} {id}", self.kind),
                kind: self.kind,
                position,
                yaw_degrees: random() * 360.,
                scale: 0.8 + random() * 0.4,
            };
            plant.validate(zone)?;
            plants.push(plant);
            if plants.len() == self.count as usize {
                return Ok(plants);
            }
        }
        Err("Scatter sampling budget exhausted".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Command, Edit, Editor};
    fn brush() -> Scatter {
        Scatter {
            first_id: 100,
            seed: 42,
            kind: PlantKind::Tree,
            centre: [64., 64.],
            radius_metres: 8.,
            count: 16,
        }
    }
    fn apply(e: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        e.execute(Command::Apply {
            expected_revision: e.revision(),
            edits,
        })
    }
    #[test]
    fn deterministic_scatter_samples_exact_surface_and_is_one_history_entry() {
        let mut zone = Zone {
            schema: 2,
            ..Zone::default()
        };
        let mut patch = crate::terrain::HeightPatch::flat([0, 0]);
        for z in 1..32 {
            for x in 1..32 {
                patch.heights[z * 33 + x] = (x + z) as f32 * 0.1;
            }
        }
        zone.terrain = Some(patch);
        let b = brush();
        let plants = b.generate(&zone).unwrap();
        assert_eq!(plants, b.generate(&zone).unwrap());
        let mut other = b.clone();
        other.seed += 1;
        assert_ne!(plants, other.generate(&zone).unwrap());
        for p in &plants {
            assert_eq!(
                p.position,
                zone.surface_position([p.position[0], p.position[2]])
                    .unwrap()
            );
            assert!((p.position[0] - 64.).hypot(p.position[2] - 64.) <= 8.);
        }
        let mut e = Editor::new(zone.clone()).unwrap();
        apply(&mut e, vec![Edit::ScatterVegetation { scatter: b }]).unwrap();
        assert_eq!(e.zone().plants, plants);
        assert_eq!(e.zone().schema, 5);
        assert_eq!(e.history(), (1, 0));
        e.execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
        assert_eq!(e.zone(), &zone);
        e.execute(Command::Redo {
            expected_revision: 2,
        })
        .unwrap();
        assert_eq!(e.zone().plants, plants);
        assert_eq!(
            Zone::from_json(&serde_json::to_vec(e.zone()).unwrap()).unwrap(),
            *e.zone()
        );
    }
    #[test]
    fn vegetation_refusals_preserve_transaction_revision_and_redo() {
        let mut e = Editor::new(Zone::default()).unwrap();
        apply(&mut e, vec![Edit::ScatterVegetation { scatter: brush() }]).unwrap();
        let mut changed = e.zone().plants[0].clone();
        changed.scale = 2.;
        apply(&mut e, vec![Edit::PutPlant { plant: changed }]).unwrap();
        e.execute(Command::Undo {
            expected_revision: 2,
        })
        .unwrap();
        let before = e.zone().clone();
        let mut invalid = brush();
        invalid.first_id = 200;
        invalid.count = 65;
        let mut collision = brush();
        collision.first_id = 99; // second generated ID already exists
        let mut exhausted = brush();
        exhausted.first_id = u64::MAX;
        let mut outside = brush();
        outside.centre = [510., 0.];
        for b in [invalid, collision, exhausted, outside] {
            assert!(
                apply(
                    &mut e,
                    vec![
                        Edit::SetSpawn {
                            position: [4., 0.1, 4.]
                        },
                        Edit::ScatterVegetation { scatter: b }
                    ]
                )
                .is_err()
            );
            assert_eq!(e.zone(), &before);
            assert_eq!((e.revision(), e.history()), (3, (1, 1)));
        }
        let mut old = before.clone();
        old.schema = 4;
        assert!(old.validate().is_err());
        let mut p = before.plants[0].clone();
        p.scale = f64::NAN;
        assert!(apply(&mut e, vec![Edit::PutPlant { plant: p }]).is_err());
        assert_eq!(e.zone(), &before);
        e.execute(Command::Redo {
            expected_revision: 3,
        })
        .unwrap();
        assert_eq!(e.zone().plants[0].scale, 2.);
    }
    #[test]
    fn plant_cap_and_far_signed_centimetres_are_checked() {
        let origin = [-128_000_000_000., 0., 128_000_000_000.];
        let zone = Zone {
            origin,
            spawn: origin.map(|v| v + 0.1),
            ..Zone::default()
        };
        let mut e = Editor::new(zone).unwrap();
        let p = Plant {
            id: 1,
            name: "Shrub".into(),
            kind: PlantKind::Shrub,
            position: [origin[0] + 0.01, 0., origin[2] + 0.02],
            yaw_degrees: 0.,
            scale: 1.,
        };
        apply(&mut e, vec![Edit::PutPlant { plant: p.clone() }]).unwrap();
        assert!((e.zone().local(p.position).unwrap()[0] - 0.01).abs() < 0.00001);
        apply(&mut e, vec![Edit::DeletePlant { id: 1 }]).unwrap();
        for i in 0..8 {
            let b = Scatter {
                first_id: 1 + i * 64,
                seed: i,
                centre: [origin[0], origin[2]],
                count: 64,
                ..brush()
            };
            apply(&mut e, vec![Edit::ScatterVegetation { scatter: b }]).unwrap();
        }
        assert_eq!(e.zone().plants.len(), MAX_PLANTS);
        let before = e.zone().clone();
        let mut p = p;
        p.id = 1000;
        assert!(apply(&mut e, vec![Edit::PutPlant { plant: p }]).is_err());
        assert_eq!(e.zone(), &before);
    }
}
