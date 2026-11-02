//! Candidate ordinary-melee one-roll arithmetic. Rates and eligibility are
//! supplied by authority; this module derives no level/skill/equipment rules.
//! The reference ordering and evidence limits are in CLASSIC_ATTACK_TABLES.md.
//! It is not a spell/special-attack table and is not enabled by loading a profile.
pub mod chances;
pub mod eligibility;
pub mod impact;
pub mod miss;

use crate::hit_outcomes::{BASIS_POINTS, Roll};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Miss,
    Dodge,
    Parry,
    Block,
    Glancing,
    Critical,
    Crushing,
    Hit,
}
const ORDER: [Outcome; 8] = [
    Outcome::Miss,
    Outcome::Dodge,
    Outcome::Parry,
    Outcome::Block,
    Outcome::Glancing,
    Outcome::Critical,
    Outcome::Crushing,
    Outcome::Hit,
];
/// Independently bounded nominal rates. Totals may exceed 100%: earlier eligible
/// outcomes consume the finite roll domain before later outcomes and normal hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rates {
    values: [u16; 7],
}
impl Rates {
    /// Order: miss, dodge, parry, block, glancing, critical, crushing.
    pub const fn new(values: [u16; 7]) -> Option<Self> {
        let mut i = 0;
        while i < values.len() {
            if values[i] > BASIS_POINTS {
                return None;
            }
            i += 1;
        }
        Some(Self { values })
    }
    pub const fn nominal(self) -> [u16; 7] {
        self.values
    }
    pub fn distribution(self, eligibility: Eligibility) -> Distribution {
        let enabled = [
            true,
            eligibility.dodge,
            eligibility.parry,
            eligibility.block,
            eligibility.glancing,
            eligibility.critical,
            eligibility.crushing,
        ];
        let mut remaining = BASIS_POINTS;
        let mut weights = [0; 8];
        for (i, value) in self.values.into_iter().enumerate() {
            if enabled[i] {
                weights[i] = value.min(remaining);
                remaining -= weights[i];
            }
        }
        weights[7] = remaining;
        Distribution { weights }
    }
}
/// Trusted, already evaluated category/defender facts. Callers must check facing,
/// capabilities, equipped weapon/shield, state, actor types and level/skill rules.
/// An ineligible outcome consumes no interval; it is not rerolled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Eligibility {
    pub dodge: bool,
    pub parry: bool,
    pub block: bool,
    pub glancing: bool,
    pub critical: bool,
    pub crushing: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Distribution {
    weights: [u16; 8],
}
impl Distribution {
    /// Effective basis-point weights in the documented outcome order, summing
    /// to exactly 10,000 even when nominal rates exceed the roll domain.
    pub const fn weights(self) -> [u16; 8] {
        self.weights
    }
    pub fn resolve(self, roll: Roll) -> Outcome {
        let mut upper = 0;
        for (outcome, weight) in ORDER.into_iter().zip(self.weights) {
            upper += weight;
            if roll.value() < upper {
                return outcome;
            }
        }
        unreachable!("Checked roll and complete distribution")
    }
}
