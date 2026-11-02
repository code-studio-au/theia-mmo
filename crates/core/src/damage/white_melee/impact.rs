//! Candidate ordinary physical outcome amounts. No health mutation, random
//! source, shield pool or live adoption. Reference order and limits are recorded
//! in CLASSIC_ATTACK_TABLES.md; exact Blizzard float behavior is not claimed.
use super::{Outcome, miss::MAX_WEAPON_SKILL};
use crate::damage::{Amount, MAX_VALUE, physical};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlancePercent(u8);
impl GlancePercent {
    pub const fn new(value: u8) -> Option<Self> {
        if value >= 1 && value <= 100 {
            Some(Self(value))
        } else {
            None
        }
    }
    pub const fn value(self) -> u8 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlanceRange {
    minimum: u8,
    maximum: u8,
}
impl GlanceRange {
    /// Melee-class reference range, using uncapped skill for damage (unlike
    /// chance). A skill above defense has no reduction. Casters need a separate
    /// policy; these percentages are explicit integer quantization.
    pub fn melee_reference_v1(weapon_skill: u16, defense_skill: u16) -> Result<Self, &'static str> {
        if weapon_skill > MAX_WEAPON_SKILL || defense_skill > MAX_WEAPON_SKILL {
            return Err("Glancing damage skills exceed reference bounds");
        }
        let gap = i32::from(defense_skill) - i32::from(weapon_skill);
        if gap < 0 {
            return Ok(Self {
                minimum: 100,
                maximum: 100,
            });
        }
        let high = (120 - gap * 3).clamp(20, 99);
        let low = (130 - gap * 5).clamp(1, 91.min(high));
        Ok(Self {
            minimum: low as u8,
            maximum: high as u8,
        })
    }
    pub const fn minimum(self) -> u8 {
        self.minimum
    }
    pub const fn maximum(self) -> u8 {
        self.maximum
    }
    /// Validate a supplied server-owned inclusive percent draw. This is not RNG.
    pub const fn pick(self, roll: u8) -> Option<GlancePercent> {
        if roll >= self.minimum && roll <= self.maximum {
            GlancePercent::new(roll)
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    Miss,
    Dodge,
    Parry,
    Hit,
    Critical,
    Crushing,
    Block { value: u32 },
    Glancing(GlancePercent),
}
impl Impact {
    pub const fn outcome(self) -> Outcome {
        match self {
            Self::Miss => Outcome::Miss,
            Self::Dodge => Outcome::Dodge,
            Self::Parry => Outcome::Parry,
            Self::Hit => Outcome::Hit,
            Self::Critical => Outcome::Critical,
            Self::Crushing => Outcome::Crushing,
            Self::Block { .. } => Outcome::Block,
            Self::Glancing(_) => Outcome::Glancing,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Armour {
    pub effective: u32,
    pub prevented: u32,
    pub remaining: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub starting_exact: Amount,
    pub starting_damage: u32,
    pub outcome: Outcome,
    /// Absent for avoidance, which consumes no mitigation or shield resources.
    pub armour: Option<Armour>,
    pub outcome_damage: u32,
    pub blocked: u32,
    pub resolved_damage: u32,
    pub health_lost: u32,
    pub overkill: u32,
}
/// Reference normal physical floor/minimum, then selected outcome: 2x critical,
/// 1.5x crushing with integer flooring, supplied whole-percent glancing, or flat
/// block. Finally clip health; do not reapply a positive minimum after block or
/// glancing. Eligibility and roll selection must already be established.
/// Critical talents/auras, resistance and absorption are outside this policy.
pub fn physical_reference_v1(
    starting: Amount,
    attacker_level: u8,
    armour: u32,
    penetration: u32,
    health: u32,
    impact: Impact,
) -> Result<Resolved, &'static str> {
    let normal =
        physical::normal_reference_v1(starting, attacker_level, armour, penetration, health)?;
    if matches!(impact,Impact::Block{value} if value>MAX_VALUE) {
        return Err("Ordinary block amount exceeds reference bounds");
    }
    let avoided = matches!(impact, Impact::Miss | Impact::Dodge | Impact::Parry);
    let after_armour = u64::from(normal.resolved_damage);
    let outcome_damage = match impact {
        Impact::Miss | Impact::Dodge | Impact::Parry => 0,
        Impact::Critical => after_armour * 2,
        Impact::Crushing => after_armour * 3 / 2,
        Impact::Glancing(percent) => after_armour * u64::from(percent.value()) / 100,
        Impact::Hit | Impact::Block { .. } => after_armour,
    };
    if outcome_damage > u64::from(MAX_VALUE) {
        return Err("Ordinary outcome damage exceeds numerical bounds");
    }
    let outcome_damage = outcome_damage as u32;
    let blocked = match impact {
        Impact::Block { value } => value.min(outcome_damage),
        _ => 0,
    };
    let resolved_damage = outcome_damage - blocked;
    let health_lost = resolved_damage.min(health);
    Ok(Resolved {
        starting_exact: starting,
        starting_damage: normal.starting_damage,
        outcome: impact.outcome(),
        armour: (!avoided).then_some(Armour {
            effective: normal.effective_armour,
            prevented: normal.armour_prevented,
            remaining: normal.resolved_damage,
        }),
        outcome_damage,
        blocked,
        resolved_damage,
        health_lost,
        overkill: resolved_damage - health_lost,
    })
}
