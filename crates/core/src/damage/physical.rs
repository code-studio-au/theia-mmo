//! Normal-contact physical reference policy. This independently expressed
//! arithmetic follows the pinned emulator reference documented in
//! docs/CLASSIC_DAMAGE_AUTHORITY.md; it does not claim all Blizzard rounding,
//! probabilities, criticals, blocks, resistance or absorption behavior.
use super::{Amount, MAX_VALUE};

pub const MAX_REFERENCE_LEVEL: u8 = 63;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub starting_exact: Amount,
    pub starting_damage: u32,
    pub effective_armour: u32,
    pub armour_prevented: u32,
    pub resolved_damage: u32,
    pub health_lost: u32,
    pub overkill: u32,
}
/// Reference-v1 boundaries: floor starting damage, apply capped armour proportion,
/// floor post-armour damage with a positive-hit minimum of one, then clip health.
/// A zero starting amount remains zero. Haste, outcome rolls and later layers are
/// outside this policy. Levels above 63 require an explicit separate extension.
pub fn normal_reference_v1(
    starting: Amount,
    attacker_level: u8,
    armour: u32,
    armour_penetration: u32,
    health: u32,
) -> std::result::Result<Resolved, &'static str> {
    if !(1..=MAX_REFERENCE_LEVEL).contains(&attacker_level)
        || armour > MAX_VALUE
        || armour_penetration > MAX_VALUE
        || health > MAX_VALUE
    {
        return Err("Physical reference inputs exceed supported bounds");
    }
    let effective_armour = armour.saturating_sub(armour_penetration);
    let starting_damage = starting.floor();
    let level_term = 400 + 85 * u32::from(attacker_level);
    let resolved_damage = if starting_damage == 0 {
        0
    } else {
        let (numerator, denominator) = if effective_armour >= 3 * level_term {
            (1, 4)
        } else {
            (level_term, level_term + effective_armour)
        };
        ((u64::from(starting_damage) * u64::from(numerator) / u64::from(denominator)) as u32).max(1)
    };
    let health_lost = resolved_damage.min(health);
    Ok(Resolved {
        starting_exact: starting,
        starting_damage,
        effective_armour,
        armour_prevented: starting_damage - resolved_damage,
        resolved_damage,
        health_lost,
        overkill: resolved_damage - health_lost,
    })
}
