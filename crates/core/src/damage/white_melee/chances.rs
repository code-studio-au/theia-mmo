//! Candidate ordinary-melee chance arithmetic. Evidence and scope limits are
//! in CLASSIC_ATTACK_TABLES.md. This module neither establishes eligibility
//! nor derives stats, class, equipment, auras or random rolls.
use super::miss::MAX_WEAPON_SKILL;
use crate::{damage::physical::MAX_REFERENCE_LEVEL, hit_outcomes::BASIS_POINTS};
fn bounded(value: i32) -> u16 {
    value.clamp(0, i32::from(BASIS_POINTS)) as u16
}
fn skills(weapon: u16, defense: u16) -> Result<i32, &'static str> {
    if weapon > MAX_WEAPON_SKILL || defense > MAX_WEAPON_SKILL {
        return Err("Ordinary chance skill inputs exceed reference bounds");
    }
    Ok(i32::from(defense) - i32::from(weapon))
}
fn level(value: u8) -> Result<i32, &'static str> {
    if !(1..=MAX_REFERENCE_LEVEL).contains(&value) {
        return Err("Ordinary chance level exceeds reference bounds");
    }
    Ok(i32::from(value))
}
fn base(value: u16) -> Result<i32, &'static str> {
    if value > BASIS_POINTS {
        return Err("Ordinary base chance exceeds reference bounds");
    }
    Ok(i32::from(value))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseDefences {
    values: [u16; 3],
}
impl BaseDefences {
    /// Explicit nominal dodge, parry and block before skill adjustment.
    pub fn new(values: [u16; 3]) -> Option<Self> {
        values
            .iter()
            .all(|value| *value <= BASIS_POINTS)
            .then_some(Self { values })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Defences {
    pub dodge_bp: u16,
    pub parry_bp: u16,
    pub block_bp: u16,
}
fn adjusted(base: u16, difference: i32, factor: i32) -> u16 {
    // A zero base is an explicitly absent chance; skill does not create a
    // learned defensive capability. Eligibility is still checked separately.
    if base == 0 {
        0
    } else {
        bounded(i32::from(base) + difference * factor)
    }
}
/// Player defender reference: each defensive chance has a 4-bp skill factor.
pub fn player_defences_reference_v1(
    chances: BaseDefences,
    attacker_skill: u16,
    defense_skill: u16,
) -> Result<Defences, &'static str> {
    let difference = skills(attacker_skill, defense_skill)?;
    Ok(Defences {
        dodge_bp: adjusted(chances.values[0], difference, 4),
        parry_bp: adjusted(chances.values[1], difference, 4),
        block_bp: adjusted(chances.values[2], difference, 4),
    })
}
/// Uncontrolled creature reference. A positive defense deficit affects dodge
/// and parry differently; it does not grant extra block. This is not a shield
/// or parry capability default for every creature.
pub fn creature_defences_reference_v1(
    chances: BaseDefences,
    attacker_skill: u16,
    defense_skill: u16,
) -> Result<Defences, &'static str> {
    let difference = skills(attacker_skill, defense_skill)?;
    let (dodge, parry, block) = if difference > 10 {
        (10, 60, 0)
    } else if difference > 0 {
        (10, 10, 0)
    } else {
        (4, 4, 4)
    };
    Ok(Defences {
        dodge_bp: adjusted(chances.values[0], difference, dodge),
        parry_bp: adjusted(chances.values[1], difference, parry),
        block_bp: adjusted(chances.values[2], difference, block),
    })
}
/// Original measured melee-class glancing shape; current skill is capped by
/// attacker level for chance, independently of damage-reduction calculations.
/// The eligibility policy separately excludes creature levels ten and lower.
pub fn player_melee_glancing_research_v1(
    attacker_level: u8,
    weapon_skill: u16,
    creature_level: u8,
) -> Result<u16, &'static str> {
    let cap = level(attacker_level)? * 5;
    let defense = level(creature_level)? * 5;
    skills(weapon_skill, defense as u16)?;
    Ok(bounded(
        1000 + (defense - i32::from(weapon_skill).min(cap)) * 200,
    ))
}
/// Wild creature-to-player crushing reference. Defense bonuses above the
/// defender's level cap do not raise the cap used for this deficit calculation.
/// Capability and actor-category eligibility remain separate.
pub fn creature_crushing_reference_v1(
    attacker_skill: u16,
    defender_level: u8,
    defense_skill: u16,
) -> Result<u16, &'static str> {
    let cap = level(defender_level)? * 5;
    skills(attacker_skill, defense_skill)?;
    let deficit = i32::from(attacker_skill) - i32::from(defense_skill).min(cap);
    Ok(if deficit < 15 {
        0
    } else {
        bounded(1500 + (deficit - 15) * 200)
    })
}
/// Isolated level suppression confirmed for same/higher creature targets.
/// This does not include the separate +3 aura modifier or class/gear derivation.
/// Lower-level targets need another explicit policy instead of guessed gains.
pub fn higher_creature_critical_reference_v1(
    base_critical_bp: u16,
    attacker_level: u8,
    creature_level: u8,
) -> Result<u16, &'static str> {
    let chance = base(base_critical_bp)?;
    let gap = level(creature_level)? - level(attacker_level)?;
    if gap < 0 {
        return Err("Lower-creature critical adjustment is outside this reference policy");
    }
    Ok(bounded(chance - gap * 100))
}
/// Wild creature ordinary critical reference against a player, before any
/// explicit additional modifiers. A zero supplied base remains zero.
pub fn creature_critical_reference_v1(
    base_critical_bp: u16,
    attacker_skill: u16,
    player_defense: u16,
) -> Result<u16, &'static str> {
    let chance = base(base_critical_bp)?;
    let difference = skills(attacker_skill, player_defense)?;
    Ok(if chance == 0 {
        0
    } else {
        bounded(chance - difference * 4)
    })
}

/// Wild creature ordinary miss reference against a player, before explicit
/// additional hit/avoidance modifiers. Distinct from player-to-creature research.
pub fn creature_miss_reference_v1(
    base_miss_bp: u16,
    attacker_skill: u16,
    player_defense: u16,
) -> Result<u16, &'static str> {
    let chance = base(base_miss_bp)?;
    let difference = skills(attacker_skill, player_defense)?;
    Ok(if chance == 0 {
        0
    } else {
        bounded(chance + difference * 4)
    })
}
