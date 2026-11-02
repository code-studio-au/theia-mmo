//! Bounded candidate recipes, separate from ordinary weapon/creature definitions.
//! Parsing does not activate ability damage or replace existing fixed effects.
use super::{ApSpeed, WeaponSpecial};
use crate::{
    abilities,
    content::ContentId,
    damage::{self, Amount, Range, Spell, stats},
};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
pub const MAX_RECIPES: usize = 32;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: Ruleset,
    weapon_definitions: String,
    recipes: Vec<RawRecipe>,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Ruleset {
    AbilityStartingFixtureV1,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawSpeed {
    WeaponBase {},
    Normalized { milliseconds: u32 },
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawRecipe {
    WeaponSpecial {
        ability: String,
        weapon_attack: String,
        ap_speed: RawSpeed,
        weapon_percent_bp: u32,
        flat_bonus: u32,
    },
    Spell {
        ability: String,
        minimum: u32,
        maximum: u32,
        coefficient_bp: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recipe {
    WeaponSpecial {
        weapon_attack: ContentId,
        item: ContentId,
        damage: WeaponSpecial,
    },
    Spell {
        damage: Spell,
    },
}
impl Recipe {
    pub fn range(&self) -> Range {
        match self {
            Self::WeaponSpecial { damage, .. } => damage.weapon().range(),
            Self::Spell { damage } => damage.range(),
        }
    }
    /// Uses only the supplied immutable server-template stats. This is neither
    /// a trusted client request nor a live gear/class-derived stat calculation.
    pub fn starting_damage(
        &self,
        roll: u32,
        player: stats::Player,
    ) -> Result<Amount, &'static str> {
        match self {
            Self::WeaponSpecial { damage, .. } => Ok(damage
                .starting_damage(roll, player.attack_power())?
                .starting),
            Self::Spell { damage } => damage.starting_damage(roll, player.spell_bonus_damage()),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Catalog {
    id: ContentId,
    recipes: BTreeMap<ContentId, Recipe>,
}
fn id(s: &str) -> Result<ContentId, &'static str> {
    ContentId::new(s).ok_or("Invalid ability starting-damage ID")
}
impl Catalog {
    pub fn from_json(
        bytes: &[u8],
        weapons: &damage::catalog::Catalog,
        abilities: &abilities::Catalog,
    ) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Ability starting-damage byte budget exceeded");
        }
        let raw: Raw =
            serde_json::from_slice(bytes).map_err(|_| "Invalid ability starting-damage recipes")?;
        let Ruleset::AbilityStartingFixtureV1 = raw.ruleset;
        if raw.schema != 1
            || abilities.schema() != 7
            || id(&raw.weapon_definitions)? != *weapons.id()
            || !(1..=MAX_RECIPES).contains(&raw.recipes.len())
        {
            return Err("Unsupported ability starting-damage schema, context or count");
        }
        let mut recipes = BTreeMap::new();
        for entry in raw.recipes {
            let (ability, recipe) = match entry {
                RawRecipe::WeaponSpecial {
                    ability,
                    weapon_attack,
                    ap_speed,
                    weapon_percent_bp,
                    flat_bonus,
                } => {
                    let weapon_attack = id(&weapon_attack)?;
                    let Some(damage::catalog::Attack::Weapon { item, damage }) =
                        weapons.attack(&weapon_attack)
                    else {
                        return Err("Special requires a known weapon attack");
                    };
                    let a = abilities
                        .ability(&ability)
                        .ok_or("Unknown special ability")?;
                    if a.requires_main_hand() != Some(item) || a.projectile().is_some() {
                        return Err(
                            "Initial main-hand special requires its exact item and direct delivery",
                        );
                    }
                    let speed = match ap_speed {
                        RawSpeed::WeaponBase {} => ApSpeed::weapon_base(),
                        RawSpeed::Normalized { milliseconds } => ApSpeed::normalized(milliseconds)
                            .ok_or("Invalid normalized AP speed")?,
                    };
                    let damage = WeaponSpecial::new(*damage, speed, weapon_percent_bp, flat_bonus)
                        .ok_or("Invalid special starting-damage scaling")?;
                    (
                        ability,
                        Recipe::WeaponSpecial {
                            weapon_attack,
                            item: item.clone(),
                            damage,
                        },
                    )
                }
                RawRecipe::Spell {
                    ability,
                    minimum,
                    maximum,
                    coefficient_bp,
                } => {
                    let a = abilities.ability(&ability).ok_or("Unknown spell ability")?;
                    if weapons.attacks().any(|(_, attack)| matches!(attack, damage::catalog::Attack::Weapon { item, .. } if a.requires_main_hand() == Some(item))) {
                        return Err("Known physical weapon special cannot silently use a spell recipe");
                    }
                    let range = Range::new(minimum, maximum).ok_or("Invalid spell base range")?;
                    let damage =
                        Spell::new(range, coefficient_bp).ok_or("Invalid spell coefficient")?;
                    (ability, Recipe::Spell { damage })
                }
            };
            let a = abilities
                .ability(&ability)
                .ok_or("Unknown starting-damage ability")?;
            if a.target() != abilities::Target::HostileAlive
                || !matches!(a.effect(), abilities::Effect::Damage { .. })
            {
                return Err("Starting-damage recipes require hostile direct damage");
            }
            if recipes.insert(id(&ability)?, recipe).is_some() {
                return Err("Duplicate starting-damage ability");
            }
        }
        if abilities
            .abilities()
            .iter()
            .filter(|a| matches!(a.effect(), abilities::Effect::Damage { .. }))
            .any(|a| !recipes.contains_key(a.id()))
        {
            return Err("Every direct-damage ability requires an explicit recipe");
        }
        Ok(Self {
            id: id(&raw.id)?,
            recipes,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn recipe(&self, ability: &ContentId) -> Option<&Recipe> {
        self.recipes.get(ability)
    }
    pub fn recipes(&self) -> impl ExactSizeIterator<Item = (&ContentId, &Recipe)> {
        self.recipes.iter()
    }
    /// Check maximal combined values before a caller opts into authority. This
    /// does not bind exact bytes, authorize equipment or validate live victims.
    pub fn validate_stats(&self, player: stats::Player) -> Result<(), &'static str> {
        for recipe in self.recipes.values() {
            recipe.starting_damage(recipe.range().maximum(), player)?;
        }
        Ok(())
    }
}
