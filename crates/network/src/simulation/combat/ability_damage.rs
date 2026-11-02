//! Explicit candidate ability starting/outcome/impact profile. No World, wire,
//! RNG or client adoption is enabled by loading this profile.
use super::{physical, white_melee};
use crate::Result;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    abilities,
    content::ContentId,
    damage::{
        self, Amount, MAX_VALUE, Range, ability_impact,
        ability_starting::catalog::{Catalog, Recipe},
        melee_special, spell,
        white_melee::eligibility::DefenderFacts,
    },
    hit_outcomes::Roll,
};
pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
mod authority;
pub(in crate::simulation::combat) use authority::Runtime;
pub use authority::{Fact, Metrics};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: Ruleset,
    starting: String,
    ordinary: String,
    spell_bonus_damage: u32,
    abilities: Vec<RawPolicy>,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Ruleset {
    AbilityDamageReferenceFixtureV1,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawPolicy {
    MeleeSpecial {
        ability: String,
        miss_bp: u16,
        dodge_bp: u16,
        parry_bp: u16,
        critical_bp: u16,
        can_crit: bool,
    },
    ArcaneZeroResistance {
        ability: String,
        hit_bonus_bp: u16,
        critical_bp: u16,
        can_crit: bool,
    },
}
#[derive(Clone, Debug)]
enum Policy {
    Melee {
        chances: melee_special::Chances,
        dodge: bool,
        parry: bool,
        critical: bool,
    },
    Arcane {
        chances: spell::Chances,
        critical: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draws {
    pub base: u32,
    pub landing: Roll,
    pub critical: Option<Roll>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    MeleeSpecial(melee_special::Outcome),
    Arcane(spell::Outcome),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub outcome: Outcome,
    pub damage: ability_impact::Resolved,
}
#[derive(Clone, Debug)]
pub struct Profile {
    id: ContentId,
    white: Arc<white_melee::Profile>,
    recipes: Catalog,
    catalog: abilities::Catalog,
    policies: BTreeMap<ContentId, Policy>,
    spell_bonus: u32,
    digest: [u8; 32],
}
fn id(value: &str) -> Result<ContentId> {
    ContentId::new(value).ok_or_else(|| "Invalid ability-damage reference ID".into())
}
impl Profile {
    /// All three byte inputs and the entire ordinary/physical context are bound.
    /// Equal-level, zero-resistance arcane and unblocked physical specials only.
    pub fn new(
        starting: &[u8],
        policy: &[u8],
        ability_data: &[u8],
        white: Arc<white_melee::Profile>,
    ) -> Result<Self> {
        if policy.len() > MAX_PROFILE_BYTES || ability_data.len() > abilities::MAX_CATALOG_BYTES {
            return Err("Ability-damage input byte budget exceeded".into());
        }
        let raw: Raw =
            serde_json::from_slice(policy).map_err(|_| "Invalid ability-damage policy")?;
        let Ruleset::AbilityDamageReferenceFixtureV1 = raw.ruleset;
        let ability_catalog = abilities::Catalog::from_json(ability_data)?;
        white.check_supported_abilities(&ability_catalog)?;
        let recipes = Catalog::from_json(starting, white.physical().catalog(), &ability_catalog)?;
        if raw.schema != 1
            || id(&raw.starting)? != *recipes.id()
            || id(&raw.ordinary)? != *white.id()
            || raw.spell_bonus_damage > MAX_VALUE
            || raw.abilities.len() != recipes.recipes().len()
        {
            return Err(
                "Unsupported ability-damage schema, references, count or spell bonus".into(),
            );
        }
        let physical = white.physical();
        let attacker = physical.stats().player().physical();
        let defender = physical
            .stats()
            .creature(physical.creature_definition())
            .ok_or("Unknown ability-damage creature stats")?;
        // Positive block capability needs a separately qualified full/partial
        // block policy, not a zero block amount disguised as support.
        if physical.catalog().attacks().any(|(_,a)|matches!(a,damage::catalog::Attack::Weapon{item,..} if white.outgoing_rates(item).is_none_or(|r|r.nominal()[3]>0))) {
            return Err("Initial ability-damage profile requires an unblocked creature".into());
        }
        let mut policies = BTreeMap::new();
        for raw_policy in raw.abilities {
            let (ability, value) = match raw_policy {
                RawPolicy::MeleeSpecial {
                    ability,
                    miss_bp,
                    dodge_bp,
                    parry_bp,
                    critical_bp,
                    can_crit,
                } => {
                    let key = id(&ability)?;
                    if !matches!(recipes.recipe(&key), Some(Recipe::WeaponSpecial { .. })) {
                        return Err(
                            "Physical special policy requires its matching starting recipe".into(),
                        );
                    }
                    let Some(Recipe::WeaponSpecial { item, .. }) = recipes.recipe(&key) else {
                        return Err("Missing special recipe".into());
                    };
                    let nominal = white
                        .outgoing_rates(item)
                        .ok_or("Unknown special weapon context")?
                        .nominal();
                    if (dodge_bp > 0 && nominal[1] == 0) || (parry_bp > 0 && nominal[2] == 0) {
                        return Err(
                            "Special cannot grant undeclared creature defensive capability".into(),
                        );
                    }
                    let a = ability_catalog
                        .ability(&ability)
                        .ok_or("Unknown special ability")?;
                    if a.combat_rules().and_then(|r| r.arc()).is_none() || a.range_metres() > 3.0 {
                        return Err("Initial special requires explicit close-range facing".into());
                    }
                    let chances =
                        melee_special::Chances::new(miss_bp, dodge_bp, parry_bp, critical_bp)
                            .ok_or("Invalid effective special rates")?;
                    (
                        key,
                        Policy::Melee {
                            chances,
                            dodge: dodge_bp > 0,
                            parry: parry_bp > 0,
                            critical: can_crit,
                        },
                    )
                }
                RawPolicy::ArcaneZeroResistance {
                    ability,
                    hit_bonus_bp,
                    critical_bp,
                    can_crit,
                } => {
                    let key = id(&ability)?;
                    if !matches!(recipes.recipe(&key), Some(Recipe::Spell { .. }))
                        || attacker.level() != defender.level()
                    {
                        return Err("Zero-resistance arcane requires its spell recipe and equal-level creature".into());
                    }
                    let landing = spell::PveHit::reference_v1(
                        attacker.level(),
                        defender.level(),
                        hit_bonus_bp,
                    )?;
                    let chances = spell::Chances::new(landing, critical_bp)
                        .ok_or("Invalid effective spell critical chance")?;
                    (
                        key,
                        Policy::Arcane {
                            chances,
                            critical: can_crit,
                        },
                    )
                }
            };
            if policies.insert(ability, value).is_some() {
                return Err("Duplicate ability-damage policy".into());
            }
        }
        if recipes
            .recipes()
            .any(|(key, _)| !policies.contains_key(key))
        {
            return Err("Missing ability-damage policy".into());
        }
        let mut identity = b"theia-ability-damage-config-v1\0".to_vec();
        identity.extend(white.context_hash());
        for bytes in [starting, policy, ability_data] {
            identity.extend(crate::terrain::sha256(bytes));
        }
        let value = Self {
            id: id(&raw.id)?,
            white,
            recipes,
            catalog: ability_catalog,
            policies,
            spell_bonus: raw.spell_bonus_damage,
            digest: crate::terrain::sha256(&identity),
        };
        value.check_maxima()?;
        Ok(value)
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn physical(&self) -> &physical::Profile {
        self.white.physical()
    }
    pub fn ordinary(&self) -> &white_melee::Profile {
        &self.white
    }
    pub fn context_hash(&self) -> [u8; 32] {
        self.digest
    }
    /// Apply after ordinary identity and before later gameplay role wrappers.
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-ability-damage-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.digest);
        crate::terrain::sha256(&bytes)
    }
    pub fn recipes(&self) -> &Catalog {
        &self.recipes
    }
    pub fn catalog(&self) -> &abilities::Catalog {
        &self.catalog
    }
    pub fn range(&self, ability: &ContentId) -> Result<Range> {
        Ok(self
            .recipes
            .recipe(ability)
            .ok_or("Unknown ability-damage recipe")?
            .range())
    }
    fn starting(&self, recipe: &Recipe, base: u32) -> Result<Amount> {
        Ok(match recipe {
            Recipe::WeaponSpecial { damage, .. } => {
                damage
                    .starting_damage(base, self.physical().stats().player().attack_power())?
                    .starting
            }
            Recipe::Spell { damage } => damage.starting_damage(base, self.spell_bonus)?,
        })
    }
    fn eligibility(
        state: white_melee::DefenderState,
        dodge: bool,
        parry: bool,
        critical: bool,
    ) -> Result<melee_special::Eligibility> {
        Ok(melee_special::creature_eligibility_reference_v1(
            DefenderFacts {
                standing: state.standing,
                can_react: state.can_react,
                casting_non_melee: state.casting_non_melee,
                in_front: state.in_front,
                disarmed: state.disarmed,
                can_dodge: dodge,
                can_parry: parry,
                can_block: false,
                usable_melee_weapon: state.usable_melee_weapon,
                usable_shield: state.usable_shield,
            },
            critical,
        )?)
    }
    pub fn needs_critical_roll(
        &self,
        ability: &ContentId,
        state: white_melee::DefenderState,
        landing: Roll,
    ) -> Result<bool> {
        Ok(
            match self
                .policies
                .get(ability)
                .ok_or("Unknown ability-damage policy")?
            {
                Policy::Melee {
                    chances,
                    dodge,
                    parry,
                    critical,
                } => chances.needs_critical_roll(
                    landing,
                    Self::eligibility(state, *dodge, *parry, *critical)?,
                ),
                Policy::Arcane { chances, critical } => {
                    chances.needs_critical_roll(landing, *critical)
                }
            },
        )
    }
    pub fn evaluate(
        &self,
        ability: &ContentId,
        state: white_melee::DefenderState,
        draws: Draws,
        health: u32,
    ) -> Result<Resolved> {
        if !(1..=self.physical().creature_maximum_health()).contains(&health) {
            return Err("Ability-damage victim health is unavailable".into());
        }
        let recipe = self
            .recipes
            .recipe(ability)
            .ok_or("Unknown ability-damage recipe")?;
        let starting = self.starting(recipe, draws.base)?;
        let player = self.physical().stats().player().physical();
        let defender = self
            .physical()
            .stats()
            .creature(self.physical().creature_definition())
            .ok_or("Unknown ability-damage creature stats")?;
        Ok(
            match self
                .policies
                .get(ability)
                .ok_or("Unknown ability-damage policy")?
            {
                Policy::Melee {
                    chances,
                    dodge,
                    parry,
                    critical,
                } => {
                    let outcome = chances.resolve(
                        draws.landing,
                        draws.critical,
                        Self::eligibility(state, *dodge, *parry, *critical)?,
                    )?;
                    Resolved {
                        outcome: Outcome::MeleeSpecial(outcome),
                        damage: ability_impact::physical_special_reference_v1(
                            starting,
                            outcome,
                            player.level(),
                            defender.armour(),
                            player.armour_penetration(),
                            health,
                        )?,
                    }
                }
                Policy::Arcane { chances, critical } => {
                    let outcome = chances.resolve(draws.landing, draws.critical, *critical)?;
                    Resolved {
                        outcome: Outcome::Arcane(outcome),
                        damage: ability_impact::magic_zero_resistance_reference_v1(
                            starting, outcome, health,
                        )?,
                    }
                }
            },
        )
    }
    fn check_maxima(&self) -> Result<()> {
        let player = self.physical().stats().player().physical();
        let defender = self
            .physical()
            .stats()
            .creature(self.physical().creature_definition())
            .ok_or("Missing ability-damage creature")?;
        for (key, recipe) in self.recipes.recipes() {
            let starting = self.starting(recipe, recipe.range().maximum())?;
            match self
                .policies
                .get(key)
                .ok_or("Missing ability-damage policy")?
            {
                Policy::Melee { critical, .. } => {
                    ability_impact::physical_special_reference_v1(
                        starting,
                        if *critical {
                            melee_special::Outcome::Critical
                        } else {
                            melee_special::Outcome::Hit
                        },
                        player.level(),
                        defender.armour(),
                        player.armour_penetration(),
                        self.physical().creature_maximum_health(),
                    )?;
                }
                Policy::Arcane { critical, .. } => {
                    ability_impact::magic_zero_resistance_reference_v1(
                        starting,
                        if *critical {
                            spell::Outcome::Critical
                        } else {
                            spell::Outcome::Hit
                        },
                        self.physical().creature_maximum_health(),
                    )?;
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests;
