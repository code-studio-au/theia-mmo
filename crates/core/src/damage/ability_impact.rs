//! Candidate ability amount boundaries. Special crit precedes armour; magic
//! explicitly supports zero resistance only. No eligibility, RNG or HP mutation.
use super::{Amount, MAX_VALUE, melee_special, physical, spell};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub starting_exact: Amount,
    pub starting_damage: u32,
    pub critical_bonus: u32,
    pub outcome_damage: u32,
    pub armour_prevented: u32,
    pub resolved_damage: u32,
    pub health_lost: u32,
    pub overkill: u32,
}
fn result(
    starting: Amount,
    outcome_damage: u32,
    armour_prevented: u32,
    health: u32,
    critical: bool,
) -> Resolved {
    let resolved_damage = outcome_damage - armour_prevented;
    let health_lost = resolved_damage.min(health);
    Resolved {
        starting_exact: starting,
        starting_damage: starting.floor(),
        critical_bonus: if critical {
            outcome_damage - starting.floor()
        } else {
            0
        },
        outcome_damage,
        armour_prevented,
        resolved_damage,
        health_lost,
        overkill: resolved_damage - health_lost,
    }
}
/// Reference-v1: floor starting, 2x selected critical, then existing checked
/// physical armour/minimum policy and HP clipping. Avoidance has no mitigation.
/// Block, absorption, mixed schools and ignore-armour specials are unsupported.
pub fn physical_special_reference_v1(
    starting: Amount,
    outcome: melee_special::Outcome,
    attacker_level: u8,
    armour: u32,
    penetration: u32,
    health: u32,
) -> Result<Resolved, &'static str> {
    // Validate the context even when this attempt is avoided.
    physical::normal_reference_v1(
        Amount::fixed(0)?,
        attacker_level,
        armour,
        penetration,
        health,
    )?;
    let critical = outcome == melee_special::Outcome::Critical;
    let amount = match outcome {
        melee_special::Outcome::Hit => starting.floor(),
        melee_special::Outcome::Critical => starting.floor() * 2,
        _ => 0,
    };
    let mitigated = physical::normal_reference_v1(
        Amount::fixed(amount)?,
        attacker_level,
        armour,
        penetration,
        health,
    )?;
    Ok(result(
        starting,
        amount,
        mitigated.armour_prevented,
        health,
        critical,
    ))
}
/// Reference-v1 with no school resistance/absorption: floor starting, selected
/// 1.5x magic critical with integer floor, HP clipping. Do not use for an actor
/// with resistance, binary effects, vulnerability or crit-modifying auras.
pub fn magic_zero_resistance_reference_v1(
    starting: Amount,
    outcome: spell::Outcome,
    health: u32,
) -> Result<Resolved, &'static str> {
    if health > MAX_VALUE {
        return Err("Magic victim health exceeds numerical bounds");
    }
    let critical = outcome == spell::Outcome::Critical;
    let amount = match outcome {
        spell::Outcome::Miss => 0,
        spell::Outcome::Hit => starting.floor(),
        spell::Outcome::Critical => starting.floor() * 3 / 2,
    };
    Amount::fixed(amount)?;
    Ok(result(starting, amount, 0, health, critical))
}
