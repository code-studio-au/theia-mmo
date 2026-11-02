//! Ordinary physical, standing, uncharmed player/wild-creature eligibility.
//! Independently expressed reference policy; not yet adopted by live authority.
use super::Eligibility;
use crate::damage::physical::MAX_REFERENCE_LEVEL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Player,
    UncontrolledCreature,
}
/// Checked actor category and level. Pets/charms require separate policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combatant {
    kind: Kind,
    level: u8,
}
impl Combatant {
    pub const fn new(kind: Kind, level: u8) -> Option<Self> {
        if level >= 1 && level <= MAX_REFERENCE_LEVEL {
            Some(Self { kind, level })
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttackerCapabilities {
    pub critical: bool,
    pub glancing: bool,
    pub crushing: bool,
}
/// Current authoritative facts, separate from nominal percentage chances.
/// `in_front` means the attacker is in the defender's front, not vice versa.
/// Gear usability includes broken/disarmed restrictions established by caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DefenderFacts {
    pub standing: bool,
    pub can_react: bool,
    pub casting_non_melee: bool,
    pub in_front: bool,
    pub disarmed: bool,
    pub can_dodge: bool,
    pub can_parry: bool,
    pub can_block: bool,
    pub usable_melee_weapon: bool,
    pub usable_shield: bool,
}
/// This policy does not accept sitting/lying defenders: their critical rules
/// need a separate explicit branch. Damage school is physical by this API's
/// contract; spells, ranged attacks and specials must not call it implicitly.
/// Level/skill probability formulas remain separate from these capability gates.
pub fn physical_standing_reference_v1(
    attacker: Combatant,
    defender: Combatant,
    capabilities: AttackerCapabilities,
    facts: DefenderFacts,
) -> Result<Eligibility, &'static str> {
    if !facts.standing {
        return Err("Standing physical reference does not support nonstanding defenders");
    }
    let creature = defender.kind == Kind::UncontrolledCreature;
    let reactive = facts.can_react && !facts.casting_non_melee;
    Ok(Eligibility {
        dodge: reactive && facts.can_dodge && (creature || facts.in_front),
        parry: reactive
            && !facts.disarmed
            && facts.can_parry
            && facts.in_front
            && (creature || facts.usable_melee_weapon),
        block: reactive && facts.can_block && facts.in_front && (creature || facts.usable_shield),
        glancing: capabilities.glancing
            && attacker.kind == Kind::Player
            && creature
            && defender.level > 10,
        critical: capabilities.critical,
        crushing: capabilities.crushing
            && attacker.kind == Kind::UncontrolledCreature
            && defender.kind == Kind::Player,
    })
}
