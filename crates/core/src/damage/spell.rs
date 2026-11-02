//! Isolated player-to-uncontrolled-creature spell landing reference. No binary
//! resistance, school mitigation, reflection, damage, RNG or authority implied.
use super::physical::MAX_REFERENCE_LEVEL;
use crate::hit_outcomes::{BASIS_POINTS, Roll};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PveHit {
    pub level_difference: i16,
    pub raw_base_miss_bp: i32,
    pub bonus_hit_bp: u16,
    miss_bp: u16,
}
impl PveHit {
    /// Ordinary, non-always-hit magic: base 4%, +1% per level through +2,
    /// then +11% per additional NPC level. Spell-hit bonus applies before the
    /// [1%,100%] miss clamp. No PvP, pets, auras or expansion extension inferred.
    pub fn reference_v1(
        caster_level: u8,
        creature_level: u8,
        bonus_hit_bp: u16,
    ) -> Result<Self, &'static str> {
        if !(1..=MAX_REFERENCE_LEVEL).contains(&caster_level)
            || !(1..=MAX_REFERENCE_LEVEL).contains(&creature_level)
            || bonus_hit_bp > BASIS_POINTS
        {
            return Err("Spell landing inputs exceed supported PvE reference domain");
        }
        let level_difference = i16::from(creature_level) - i16::from(caster_level);
        let gap = i32::from(level_difference);
        let raw_base_miss_bp = if gap <= 2 {
            400 + 100 * gap
        } else {
            600 + 1100 * (gap - 2)
        };
        let miss_bp =
            (raw_base_miss_bp - i32::from(bonus_hit_bp)).clamp(100, i32::from(BASIS_POINTS)) as u16;
        Ok(Self {
            level_difference,
            raw_base_miss_bp,
            bonus_hit_bp,
            miss_bp,
        })
    }
    pub const fn miss_bp(self) -> u16 {
        self.miss_bp
    }
    pub const fn hit_bp(self) -> u16 {
        BASIS_POINTS - self.miss_bp
    }
}
/// Base landing failure is distinct from later school resistance. Classic may
/// present both as "resist"; retaining their causes prevents mitigation guesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Miss,
    Hit,
    Critical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chances {
    landing: PveHit,
    critical_bp: u16,
}
impl Chances {
    /// The authority supplies the effective spell critical chance separately;
    /// it is not weapon crit, a white-melee interval or a level-derived default.
    pub const fn new(landing: PveHit, critical_bp: u16) -> Option<Self> {
        if critical_bp <= BASIS_POINTS {
            Some(Self {
                landing,
                critical_bp,
            })
        } else {
            None
        }
    }
    pub fn needs_critical_roll(self, landing: Roll, can_crit: bool) -> bool {
        landing.value() >= self.landing.miss_bp && can_crit && self.critical_bp > 0
    }
    /// A second independent server-owned draw is required only for a landed,
    /// eligible spell with positive critical chance. Extra/missing draws fail;
    /// this checks the contract, not the entropy or ownership of supplied rolls.
    pub fn resolve(
        self,
        landing: Roll,
        critical: Option<Roll>,
        can_crit: bool,
    ) -> Result<Outcome, &'static str> {
        if self.needs_critical_roll(landing, can_crit) != critical.is_some() {
            return Err("Unexpected or missing conditional spell critical draw");
        }
        if landing.value() < self.landing.miss_bp {
            return Ok(Outcome::Miss);
        }
        Ok(if critical.is_some_and(|r| r.value() < self.critical_bp) {
            Outcome::Critical
        } else {
            Outcome::Hit
        })
    }
}
