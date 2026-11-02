//! Explicit static development templates, not client-provided stats or derived
//! gear/class progression. Exact-byte authority/world binding remains a caller task.
use super::{
    MAX_VALUE,
    catalog::{Attack, Catalog},
    physical::MAX_REFERENCE_LEVEL,
};
use crate::content::ContentId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
pub const MAX_CREATURES: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ruleset {
    PhysicalReferenceV1,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPhysical {
    level: u8,
    armour: u32,
    armour_penetration: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawPhysical")]
pub struct Physical {
    level: u8,
    armour: u32,
    armour_penetration: u32,
}
impl TryFrom<RawPhysical> for Physical {
    type Error = &'static str;
    fn try_from(raw: RawPhysical) -> Result<Self, Self::Error> {
        Self::new(raw.level, raw.armour, raw.armour_penetration)
            .ok_or("Physical stats exceed reference domain")
    }
}
impl Physical {
    pub fn new(level: u8, armour: u32, armour_penetration: u32) -> Option<Self> {
        ((1..=MAX_REFERENCE_LEVEL).contains(&level)
            && armour <= MAX_VALUE
            && armour_penetration <= MAX_VALUE)
            .then_some(Self {
                level,
                armour,
                armour_penetration,
            })
    }
    pub fn level(self) -> u8 {
        self.level
    }
    pub fn armour(self) -> u32 {
        self.armour
    }
    pub fn armour_penetration(self) -> u32 {
        self.armour_penetration
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlayer {
    physical: Physical,
    attack_power: u32,
    spell_bonus_damage: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawPlayer")]
pub struct Player {
    physical: Physical,
    attack_power: u32,
    spell_bonus_damage: u32,
}
impl TryFrom<RawPlayer> for Player {
    type Error = &'static str;
    fn try_from(raw: RawPlayer) -> Result<Self, Self::Error> {
        if raw.attack_power > MAX_VALUE || raw.spell_bonus_damage > MAX_VALUE {
            return Err("Damage stats exceed numerical bounds");
        }
        Ok(Self {
            physical: raw.physical,
            attack_power: raw.attack_power,
            spell_bonus_damage: raw.spell_bonus_damage,
        })
    }
}
impl Player {
    pub fn physical(self) -> Physical {
        self.physical
    }
    pub fn attack_power(self) -> u32 {
        self.attack_power
    }
    pub fn spell_bonus_damage(self) -> u32 {
        self.spell_bonus_damage
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCreature {
    creature: String,
    physical: Physical,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: Ruleset,
    player: Player,
    creatures: Vec<RawCreature>,
}
#[derive(Clone, Debug)]
pub struct Profile {
    id: ContentId,
    ruleset: Ruleset,
    player: Player,
    creatures: BTreeMap<ContentId, Physical>,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Damage stats byte budget exceeded");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid damage stats profile")?;
        if raw.schema != 1 || raw.creatures.len() > MAX_CREATURES {
            return Err("Unsupported damage stats schema/count");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid damage stats ID")?;
        let mut creatures = BTreeMap::new();
        for raw in raw.creatures {
            let id = ContentId::new(&raw.creature).ok_or("Invalid damage stats creature ID")?;
            if creatures.insert(id, raw.physical).is_some() {
                return Err("Duplicate damage stats creature");
            }
        }
        Ok(Self {
            id,
            ruleset: raw.ruleset,
            player: raw.player,
            creatures,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn ruleset(&self) -> Ruleset {
        self.ruleset
    }
    pub fn player(&self) -> Player {
        self.player
    }
    pub fn creature(&self, id: &ContentId) -> Option<Physical> {
        self.creatures.get(id).copied()
    }
    pub fn creatures(&self) -> impl ExactSizeIterator<Item = (&ContentId, &Physical)> {
        self.creatures.iter()
    }
    /// Check combined starting magnitudes and exact creature-definition coverage.
    /// Known world creature/item/ability references and committed equipment still
    /// require their existing authority checks; this does not establish ownership.
    pub fn validate_definitions(&self, catalog: &Catalog) -> Result<(), &'static str> {
        let mut used = BTreeSet::new();
        for (_, attack) in catalog.attacks() {
            match attack {
                Attack::Weapon { damage, .. } => {
                    damage.starting_damage(damage.range.maximum, self.player.attack_power)?;
                }
                Attack::Spell { damage, .. } => {
                    damage.starting_damage(damage.range.maximum, self.player.spell_bonus_damage)?;
                }
                Attack::Creature { creature, .. } => {
                    if !self.creatures.contains_key(creature) {
                        return Err("Creature damage is missing stats");
                    }
                    used.insert(creature);
                }
            }
        }
        if used.len() != self.creatures.len() {
            return Err("Damage stats include an unused creature");
        }
        Ok(())
    }
}
