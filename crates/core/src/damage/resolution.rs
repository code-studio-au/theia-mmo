//! Provisional numerical ordering, explicitly separate from Classic eligibility,
//! probability tables and live state. This is not an authoritative attack API.
use super::{Amount, BASIS_POINTS, MAX_VALUE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale(u32);
/// Numerical engineering bound, separate from spell coefficient definitions.
pub const MAX_SCALE_BP: u32 = 100_000;
impl Scale {
    pub const fn new(basis_points: u32) -> Option<Self> {
        if basis_points <= MAX_SCALE_BP {
            Some(Self(basis_points))
        } else {
            None
        }
    }
}
/// The authority must establish outcome eligibility separately. No universal
/// miss/dodge/parry/critical/glancing/crushing table is inferred from these inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Impact {
    Avoided,
    Contact(Scale),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reduction {
    Armour,
    Resistance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Defences {
    reduction: Reduction,
    reduction_bp: u16,
    block: u32,
    absorption: u32,
}
impl Defences {
    pub fn physical(armour_bp: u16, block: u32, absorption: u32) -> Option<Self> {
        Self::new(Reduction::Armour, armour_bp, block, absorption)
    }
    pub fn magical(resistance_bp: u16, absorption: u32) -> Option<Self> {
        Self::new(Reduction::Resistance, resistance_bp, 0, absorption)
    }
    fn new(reduction: Reduction, reduction_bp: u16, block: u32, absorption: u32) -> Option<Self> {
        (u32::from(reduction_bp) <= BASIS_POINTS && block <= MAX_VALUE && absorption <= MAX_VALUE)
            .then_some(Self {
                reduction,
                reduction_bp,
                block,
                absorption,
            })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Floor,
    NearestHalfUp,
}
/// Exact prevention facts and separately rounded health quantities. Constructing
/// or observing this value does not apply health, spend a shield or emit events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakdown {
    pub starting: Amount,
    pub after_outcome: Amount,
    pub armour_prevented: Amount,
    pub resistance_prevented: Amount,
    pub blocked: Amount,
    pub absorbed: Amount,
    pub resolved: Amount,
    pub rounded_damage: u32,
    pub health_lost: u32,
    pub overkill: u32,
}
/// Fixture-v1 order: supplied impact scale -> supplied armour/resistance reduction
/// -> flat block -> flat absorption -> named integer rounding -> health clipping.
/// Percentages, outcome scale and ordering require ruleset evidence before live
/// adoption. Absorption usage remains exact; no integer shield-pool rule is implied.
pub fn resolve_fixture_v1(
    starting: Amount,
    impact: Impact,
    defences: Defences,
    health: u32,
    rounding: Rounding,
) -> Result<Breakdown, &'static str> {
    if health > MAX_VALUE {
        return Err("Health exceeds numerical bounds");
    }
    let zero = Amount::fixed(0)?;
    let after_outcome = match impact {
        Impact::Avoided => zero,
        Impact::Contact(scale) => starting.scale_bp(scale.0)?,
    };
    let after_reduction =
        after_outcome.scale_bp(BASIS_POINTS - u32::from(defences.reduction_bp))?;
    let prevented = after_outcome.difference(after_reduction)?;
    let (armour_prevented, resistance_prevented) = match defences.reduction {
        Reduction::Armour => (prevented, zero),
        Reduction::Resistance => (zero, prevented),
    };
    let (after_block, blocked) = after_reduction.prevent_flat(defences.block)?;
    let (resolved, absorbed) = after_block.prevent_flat(defences.absorption)?;
    let rounded_damage = match rounding {
        Rounding::Floor => resolved.floor(),
        Rounding::NearestHalfUp => resolved.nearest_half_up(),
    };
    let health_lost = rounded_damage.min(health);
    Ok(Breakdown {
        starting,
        after_outcome,
        armour_prevented,
        resistance_prevented,
        blocked,
        absorbed,
        resolved,
        rounded_damage,
        health_lost,
        overkill: rounded_damage - health_lost,
    })
}
