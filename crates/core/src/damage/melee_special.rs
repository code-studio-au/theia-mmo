//! Unblocked main-hand physical specials against standing uncontrolled creatures.
//! Separate from ordinary swings: avoided attacks stop before conditional crit.
//! No glancing/crushing/deflect or inferred block handling. No RNG or authority.
use crate::{
    damage::white_melee::eligibility::DefenderFacts,
    hit_outcomes::{BASIS_POINTS, Roll},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eligibility {
    pub dodge: bool,
    pub parry: bool,
    pub critical: bool,
}
/// Reference first scope excludes blocking creatures, nonstanding states and
/// ability-specific avoidance bypasses. Callers must validate those categories.
/// `in_front` is the attacker in the defender's front, not attacker attack arc.
pub fn creature_eligibility_reference_v1(
    facts: DefenderFacts,
    can_crit: bool,
) -> Result<Eligibility, &'static str> {
    if !facts.standing || facts.can_block {
        return Err("Unblocked standing creature special scope required");
    }
    let reactive = facts.can_react && !facts.casting_non_melee;
    Ok(Eligibility {
        dodge: reactive && facts.can_dodge,
        parry: reactive && facts.can_parry && facts.in_front && !facts.disarmed,
        critical: can_crit,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Miss,
    Dodge,
    Parry,
    Hit,
    Critical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chances {
    avoidance: [u16; 3],
    critical_bp: u16,
}
impl Chances {
    /// Supplied effective rates. Nominal avoidance may exceed 100% in total;
    /// ordered eligible miss/dodge/parry buckets consume at most 10,000 values.
    /// Critical is a separate conditional probability, never a later bucket.
    pub const fn new(miss: u16, dodge: u16, parry: u16, critical: u16) -> Option<Self> {
        if miss <= BASIS_POINTS
            && dodge <= BASIS_POINTS
            && parry <= BASIS_POINTS
            && critical <= BASIS_POINTS
        {
            Some(Self {
                avoidance: [miss, dodge, parry],
                critical_bp: critical,
            })
        } else {
            None
        }
    }
    pub fn weights(self, eligible: Eligibility) -> [u16; 4] {
        let active = [true, eligible.dodge, eligible.parry];
        let mut left = BASIS_POINTS;
        let mut weights = [0; 4];
        for i in 0..3 {
            if active[i] {
                weights[i] = self.avoidance[i].min(left);
                left -= weights[i];
            }
        }
        weights[3] = left;
        weights
    }
    fn landing(self, roll: Roll, eligible: Eligibility) -> Outcome {
        let weights = self.weights(eligible);
        let mut end = 0;
        for (i, outcome) in [Outcome::Miss, Outcome::Dodge, Outcome::Parry]
            .into_iter()
            .enumerate()
        {
            end += weights[i];
            if roll.value() < end {
                return outcome;
            }
        }
        Outcome::Hit
    }
    pub fn needs_critical_roll(self, landing: Roll, eligible: Eligibility) -> bool {
        self.landing(landing, eligible) == Outcome::Hit && eligible.critical && self.critical_bp > 0
    }
    pub fn resolve(
        self,
        landing: Roll,
        critical: Option<Roll>,
        eligible: Eligibility,
    ) -> Result<Outcome, &'static str> {
        if self.needs_critical_roll(landing, eligible) != critical.is_some() {
            return Err("Unexpected or missing conditional melee-special critical draw");
        }
        let outcome = self.landing(landing, eligible);
        Ok(if critical.is_some_and(|r| r.value() < self.critical_bp) {
            Outcome::Critical
        } else {
            outcome
        })
    }
}
