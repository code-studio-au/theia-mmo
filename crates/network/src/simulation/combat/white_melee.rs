//! Candidate ordinary-outcome configuration. Loading it does not enable World,
//! wire or presentation behavior. Explicit actor templates are not derived gear,
//! class, progression stats or a complete Classic rule set.
use super::physical;
use crate::Result;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    content::ContentId,
    damage::{
        MAX_VALUE, Weapon,
        catalog::Attack,
        white_melee::{
            Distribution, Outcome, Rates,
            chances::{self, BaseDefences},
            eligibility::{self, AttackerCapabilities, Combatant, DefenderFacts, Kind},
            impact::{self, GlanceRange, Impact, Resolved},
            miss::{self, Weapons},
        },
    },
    hit_outcomes::Roll,
};
pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
const MAX_WEAPONS: usize = 32;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: String,
    definitions: String,
    stats: String,
    player: RawPlayer,
    creature: RawCreature,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlayer {
    defense_skill: u16,
    critical_bp: u16,
    glancing_policy: String,
    defences: Defensive,
    weapons: Vec<RawWeapon>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWeapon {
    item: String,
    skill: u16,
    hit_bonus_bp: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCreature {
    id: String,
    critical_bp: u16,
    can_crush: bool,
    defences: Defensive,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defensive {
    dodge_bp: u16,
    parry_bp: u16,
    block_bp: u16,
    block_value: u32,
}
impl Defensive {
    fn checked(&self) -> Result<BaseDefences> {
        if self.block_value > MAX_VALUE {
            return Err("Ordinary block value exceeds bounds".into());
        }
        BaseDefences::new([self.dodge_bp, self.parry_bp, self.block_bp])
            .ok_or_else(|| "Ordinary defensive chances exceed bounds".into())
    }
    fn facts(&self, state: DefenderState) -> DefenderFacts {
        DefenderFacts {
            standing: state.standing,
            can_react: state.can_react,
            casting_non_melee: state.casting_non_melee,
            in_front: state.in_front,
            disarmed: state.disarmed,
            can_dodge: self.dodge_bp > 0,
            can_parry: self.parry_bp > 0,
            can_block: self.block_bp > 0,
            usable_melee_weapon: state.usable_melee_weapon,
            usable_shield: state.usable_shield,
        }
    }
}
/// Supplied from current authoritative state after life/attack validation.
/// Creature capability and nominal percentages come from the bound profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DefenderState {
    pub standing: bool,
    pub can_react: bool,
    pub casting_non_melee: bool,
    pub in_front: bool,
    pub disarmed: bool,
    pub usable_melee_weapon: bool,
    pub usable_shield: bool,
}
/// Explicit server-owned candidate draws; never an attack command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draws {
    pub outcome: Roll,
    pub base: u32,
    pub glancing: Option<u8>,
}
#[derive(Clone, Debug)]
struct Outgoing {
    damage: Weapon,
    rates: Rates,
    glance: GlanceRange,
}
#[derive(Clone, Debug)]
pub struct Profile {
    id: ContentId,
    physical: Arc<physical::Profile>,
    weapons: BTreeMap<ContentId, Outgoing>,
    incoming: Rates,
    player: Defensive,
    creature: Defensive,
    player_critical: bool,
    creature_critical: bool,
    can_crush: bool,
    player_actor: Combatant,
    creature_actor: Combatant,
    digest: [u8; 32],
}
impl Profile {
    pub fn new(bytes: &[u8], physical: Arc<physical::Profile>) -> Result<Self> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Ordinary-melee profile exceeds byte budget".into());
        }
        let raw: Raw =
            serde_json::from_slice(bytes).map_err(|_| "Invalid ordinary-melee profile")?;
        if raw.schema != 1
            || raw.ruleset != "white_physical_reference_v1"
            || raw.definitions != physical.catalog().id().as_str()
            || raw.stats != physical.stats().id().as_str()
            || raw.creature.id != physical.creature_definition().as_str()
            || raw.player.glancing_policy != "melee_reference_v1"
            || raw.player.weapons.is_empty()
            || raw.player.weapons.len() > MAX_WEAPONS
        {
            return Err("Unsupported ordinary-melee configuration or references".into());
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid ordinary-melee profile ID")?;
        let p = physical.stats().player().physical();
        let c = physical
            .stats()
            .creature(physical.creature_definition())
            .ok_or("Missing physical creature stats")?;
        let creature_skill = u16::from(c.level()) * 5;
        let player_defences = raw.player.defences.checked()?;
        let creature_defences = raw.creature.defences.checked()?;
        let incoming_defenses = chances::player_defences_reference_v1(
            player_defences,
            creature_skill,
            raw.player.defense_skill,
        )?;
        let player_actor =
            Combatant::new(Kind::Player, p.level()).ok_or("Unsupported ordinary player level")?;
        let creature_actor = Combatant::new(Kind::UncontrolledCreature, c.level())
            .ok_or("Unsupported ordinary creature level")?;
        let player_critical = chances::higher_creature_critical_reference_v1(
            raw.player.critical_bp,
            p.level(),
            c.level(),
        )?;
        let incoming = Rates::new([
            chances::creature_miss_reference_v1(500, creature_skill, raw.player.defense_skill)?,
            incoming_defenses.dodge_bp,
            incoming_defenses.parry_bp,
            incoming_defenses.block_bp,
            0,
            chances::creature_critical_reference_v1(
                raw.creature.critical_bp,
                creature_skill,
                raw.player.defense_skill,
            )?,
            if raw.creature.can_crush {
                chances::creature_crushing_reference_v1(
                    creature_skill,
                    p.level(),
                    raw.player.defense_skill,
                )?
            } else {
                0
            },
        ])
        .ok_or("Invalid ordinary creature rates")?;
        let mut weapons = BTreeMap::new();
        for weapon in raw.player.weapons {
            let item = ContentId::new(&weapon.item).ok_or("Invalid ordinary weapon item")?;
            let (_, damage) = physical
                .weapon(&item)
                .ok_or("Unknown ordinary weapon item")?;
            let defenses = chances::creature_defences_reference_v1(
                creature_defences,
                weapon.skill,
                creature_skill,
            )?;
            let rates = Rates::new([
                miss::player_creature_research_v1(
                    c.level(),
                    weapon.skill,
                    weapon.hit_bonus_bp,
                    Weapons::Single,
                )?
                .miss_bp,
                defenses.dodge_bp,
                defenses.parry_bp,
                defenses.block_bp,
                chances::player_melee_glancing_research_v1(p.level(), weapon.skill, c.level())?,
                player_critical,
                0,
            ])
            .ok_or("Invalid ordinary player rates")?;
            let glance = GlanceRange::melee_reference_v1(weapon.skill, creature_skill)?;
            if weapons
                .insert(
                    item,
                    Outgoing {
                        damage,
                        rates,
                        glance,
                    },
                )
                .is_some()
            {
                return Err("Duplicate ordinary weapon skill binding".into());
            }
        }
        if weapons.len()
            != physical
                .catalog()
                .attacks()
                .filter(|(_, attack)| matches!(attack, Attack::Weapon { .. }))
                .count()
        {
            return Err("Ordinary profile must cover every physical weapon".into());
        }
        let mut identity = b"theia-white-melee-config-v1\0".to_vec();
        identity.extend(crate::terrain::sha256(bytes));
        identity.extend(physical.context_hash());
        let profile = Self {
            id,
            physical,
            weapons,
            incoming,
            player: raw.player.defences,
            creature: raw.creature.defences,
            player_critical: raw.player.critical_bp > 0,
            creature_critical: raw.creature.critical_bp > 0,
            can_crush: raw.creature.can_crush,
            player_actor,
            creature_actor,
            digest: crate::terrain::sha256(&identity),
        };
        profile.check_maxima()?;
        Ok(profile)
    }
    pub fn check_supported_abilities(
        &self,
        catalog: &theia_core::abilities::Catalog,
    ) -> Result<()> {
        if catalog.schema() != 7
            || catalog.abilities().iter().any(|ability| {
                let melee = ability.animation()
                    == Some(theia_core::abilities::AnimationCue::SwordAttack)
                    || ability
                        .requires_main_hand()
                        .is_some_and(|item| self.physical.weapon(item).is_some());
                melee
                    && catalog
                        .timing(ability.id().as_str(), theia_core::time::TickRate::Hz30)
                        .is_some_and(|t| t.cast_ticks > 0)
            })
        {
            return Err(
                "Ordinary outcomes require explicit rules and supported instant melee specials"
                    .into(),
            );
        }
        Ok(())
    }
    fn check_maxima(&self) -> Result<()> {
        for (item, weapon) in &self.weapons {
            self.resolve_outgoing(
                item,
                weapon.damage.range().maximum(),
                self.physical.creature_maximum_health(),
                Impact::Hit,
            )?;
            if weapon.rates.nominal()[5] > 0 {
                self.resolve_outgoing(
                    item,
                    weapon.damage.range().maximum(),
                    self.physical.creature_maximum_health(),
                    Impact::Critical,
                )?;
            }
        }
        let roll = self.physical.creature_attack().1.maximum();
        self.resolve_incoming(roll, self.physical.player_maximum_health(), Impact::Hit)?;
        if self.incoming.nominal()[5] > 0 {
            self.resolve_incoming(
                roll,
                self.physical.player_maximum_health(),
                Impact::Critical,
            )?;
        }
        if self.incoming.nominal()[6] > 0 {
            self.resolve_incoming(
                roll,
                self.physical.player_maximum_health(),
                Impact::Crushing,
            )?;
        }
        Ok(())
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn physical(&self) -> &physical::Profile {
        &self.physical
    }
    pub fn context_hash(&self) -> [u8; 32] {
        self.digest
    }
    /// Apply after physical identity, before later role wrappers.
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-white-melee-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.digest);
        crate::terrain::sha256(&bytes)
    }
    pub fn outgoing_rates(&self, item: &ContentId) -> Option<Rates> {
        self.weapons.get(item).map(|w| w.rates)
    }
    pub fn incoming_rates(&self) -> Rates {
        self.incoming
    }
    pub fn outgoing_distribution(
        &self,
        item: &ContentId,
        state: DefenderState,
    ) -> Result<Distribution> {
        let rates = self
            .outgoing_rates(item)
            .ok_or("Unknown ordinary player weapon")?;
        let eligible = eligibility::physical_standing_reference_v1(
            self.player_actor,
            self.creature_actor,
            AttackerCapabilities {
                critical: self.player_critical,
                glancing: true,
                crushing: false,
            },
            self.creature.facts(state),
        )?;
        Ok(rates.distribution(eligible))
    }
    pub fn incoming_distribution(&self, state: DefenderState) -> Result<Distribution> {
        let eligible = eligibility::physical_standing_reference_v1(
            self.creature_actor,
            self.player_actor,
            AttackerCapabilities {
                critical: self.creature_critical,
                glancing: false,
                crushing: self.can_crush,
            },
            self.player.facts(state),
        )?;
        Ok(self.incoming.distribution(eligible))
    }
    pub fn evaluate_outgoing(
        &self,
        item: &ContentId,
        state: DefenderState,
        draws: Draws,
        health: u32,
    ) -> Result<Resolved> {
        let outcome = self
            .outgoing_distribution(item, state)?
            .resolve(draws.outcome);
        let weapon = self.weapons.get(item).ok_or("Unknown ordinary weapon")?;
        let impact = selected_impact(
            outcome,
            self.creature.block_value,
            Some(weapon.glance),
            draws.glancing,
        )?;
        self.resolve_outgoing(item, draws.base, health, impact)
    }
    pub fn evaluate_incoming(
        &self,
        state: DefenderState,
        draws: Draws,
        health: u32,
    ) -> Result<Resolved> {
        let outcome = self.incoming_distribution(state)?.resolve(draws.outcome);
        let impact = selected_impact(outcome, self.player.block_value, None, draws.glancing)?;
        self.resolve_incoming(draws.base, health, impact)
    }
    fn resolve_outgoing(
        &self,
        item: &ContentId,
        roll: u32,
        health: u32,
        impact: Impact,
    ) -> Result<Resolved> {
        if !(1..=self.physical.creature_maximum_health()).contains(&health) {
            return Err("Ordinary creature health is unavailable".into());
        }
        let weapon = self.weapons.get(item).ok_or("Unknown ordinary weapon")?;
        let player = self.physical.stats().player();
        let defender = self
            .physical
            .stats()
            .creature(self.physical.creature_definition())
            .ok_or("Missing creature physical stats")?;
        Ok(impact::physical_reference_v1(
            weapon.damage.starting_damage(roll, player.attack_power())?,
            player.physical().level(),
            defender.armour(),
            player.physical().armour_penetration(),
            health,
            impact,
        )?)
    }
    fn resolve_incoming(&self, roll: u32, health: u32, impact: Impact) -> Result<Resolved> {
        if !(1..=self.physical.player_maximum_health()).contains(&health) {
            return Err("Ordinary player health is unavailable".into());
        }
        let player = self.physical.stats().player().physical();
        let attacker = self
            .physical
            .stats()
            .creature(self.physical.creature_definition())
            .ok_or("Missing creature physical stats")?;
        Ok(impact::physical_reference_v1(
            self.physical.creature_attack().1.starting_damage(roll)?,
            attacker.level(),
            player.armour(),
            attacker.armour_penetration(),
            health,
            impact,
        )?)
    }
}
fn selected_impact(
    outcome: Outcome,
    block: u32,
    range: Option<GlanceRange>,
    draw: Option<u8>,
) -> Result<Impact> {
    if outcome != Outcome::Glancing && draw.is_some() {
        return Err("Unexpected glancing draw for ordinary outcome".into());
    }
    Ok(match outcome {
        Outcome::Miss => Impact::Miss,
        Outcome::Dodge => Impact::Dodge,
        Outcome::Parry => Impact::Parry,
        Outcome::Hit => Impact::Hit,
        Outcome::Critical => Impact::Critical,
        Outcome::Crushing => Impact::Crushing,
        Outcome::Block => Impact::Block { value: block },
        Outcome::Glancing => Impact::Glancing(
            range
                .and_then(|r| draw.and_then(|value| r.pick(value)))
                .ok_or("Missing or invalid ordinary glancing draw")?,
        ),
    })
}
#[cfg(test)]
mod tests;

mod authority;
pub(super) use authority::Runtime;
pub use authority::{Counts, Fact, Metrics};
