//! Candidate player-to-creature ordinary-melee miss policy, based on original
//! Classic research. Interpolations/low-level rules are research estimates;
//! primary confirmed examples and limits are recorded in CLASSIC_ATTACK_TABLES.md.
//! No live table, player skill training, gear derivation or randomness is enabled.
use crate::{damage::physical::MAX_REFERENCE_LEVEL, hit_outcomes::BASIS_POINTS};

pub const MAX_WEAPON_SKILL: u16 = 1000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapons {
    Single,
    Dual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub defense_skill: u16,
    pub skill_deficit: i32,
    /// Signed intermediate before the final probability clamp. Preserving it
    /// lets skill reductions offset the dual penalty instead of flooring early.
    pub raw_base_bp: i32,
    /// Normal-weapon base probability, provided separately for inspection.
    pub base_miss_bp: u16,
    pub dual_penalty_bp: u16,
    pub hit_suppression_bp: u16,
    pub effective_hit_bonus_bp: u16,
    pub miss_bp: u16,
}
/// Ordinary player weapon attacks against an uncontrolled creature, with
/// creature defense modeled as five times its level. PvP, pets, specials,
/// elemental autoattacks and aura interactions require separate policies.
/// Skill includes explicit authoritative bonuses; no current/max skill is guessed.
/// Bonus bounds are checked before calculation, not silently clamped.
pub fn player_creature_research_v1(
    creature_level: u8,
    weapon_skill: u16,
    hit_bonus_bp: u16,
    weapons: Weapons,
) -> Result<Resolved, &'static str> {
    if !(1..=MAX_REFERENCE_LEVEL).contains(&creature_level)
        || weapon_skill > MAX_WEAPON_SKILL
        || hit_bonus_bp > BASIS_POINTS
    {
        return Err("Ordinary player/creature miss inputs exceed research policy bounds");
    }
    let defense_skill = u16::from(creature_level) * 5;
    let skill_deficit = i32::from(defense_skill) - i32::from(weapon_skill);
    let rate = if skill_deficit > 10 { 20 } else { 10 };
    let base = 500 + skill_deficit * rate;
    // Every base rate is a multiple of ten, so this adjustment is exact in
    // basis points. No implicit fractional-probability rounding occurs here.
    let base = if creature_level < 10 {
        base * i32::from(creature_level) / 10
    } else {
        base
    };
    let suppression = (skill_deficit - 10).max(0) * 20;
    let effective_hit = i32::from(hit_bonus_bp).saturating_sub(suppression).max(0);
    let dual = if weapons == Weapons::Dual { 1900 } else { 0 };
    let miss = (base + dual - effective_hit).clamp(0, i32::from(BASIS_POINTS));
    // Checked level/skill/rate domains keep each intermediate below u16::MAX.
    Ok(Resolved {
        defense_skill,
        skill_deficit,
        raw_base_bp: base,
        base_miss_bp: base.clamp(0, i32::from(BASIS_POINTS)) as u16,
        dual_penalty_bp: dual as u16,
        hit_suppression_bp: suppression as u16,
        effective_hit_bonus_bp: effective_hit as u16,
        miss_bp: miss as u16,
    })
}
