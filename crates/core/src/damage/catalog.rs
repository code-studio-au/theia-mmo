//! Bounded candidate attack definitions. Loading this catalogue never enables
//! damage authority, changes existing melee profiles or establishes equipment ownership.
use super::{Range, Spell, Weapon};
use crate::{abilities, content::ContentId, inventory};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_PROFILE_BYTES: usize = 32 * 1024;
pub const MAX_ATTACKS: usize = 64;
pub const MAX_WEAPONS: usize = 32;
pub const MAX_SPELLS: usize = 32;
pub const MAX_CREATURES: usize = 16;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    attacks: Vec<RawAttack>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawAttack {
    Weapon {
        id: String,
        item: String,
        minimum: u32,
        maximum: u32,
        base_speed_ms: u32,
    },
    Spell {
        id: String,
        ability: String,
        minimum: u32,
        maximum: u32,
        coefficient_bp: u32,
    },
    Creature {
        id: String,
        creature: String,
        minimum: u32,
        maximum: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attack {
    Weapon { item: ContentId, damage: Weapon },
    Spell { ability: ContentId, damage: Spell },
    Creature { creature: ContentId, damage: Range },
}
#[derive(Clone, Debug)]
pub struct Catalog {
    id: ContentId,
    attacks: BTreeMap<ContentId, Attack>,
}
fn id(value: &str) -> Result<ContentId, &'static str> {
    ContentId::new(value).ok_or("Invalid damage definition or reference ID")
}
fn valid_main_hand(items: &inventory::Catalog, id: &ContentId) -> bool {
    items
        .definition(id)
        .is_some_and(|item| item.maximum_stack() == 1 && item.main_hand_appearance().is_some())
}
fn range(minimum: u32, maximum: u32) -> Result<Range, &'static str> {
    Range::new(minimum, maximum).ok_or("Invalid starting damage range")
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Damage definition byte budget exceeded");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid damage definitions")?;
        if raw.schema != 1 || !(1..=MAX_ATTACKS).contains(&raw.attacks.len()) {
            return Err("Unsupported damage schema or attack count");
        }
        let mut attacks = BTreeMap::new();
        let mut references = BTreeSet::new();
        let mut counts = [0; 3];
        for raw in raw.attacks {
            let (key, tag, reference, attack) = match raw {
                RawAttack::Weapon {
                    id: key,
                    item,
                    minimum,
                    maximum,
                    base_speed_ms,
                } => {
                    let item = id(&item)?;
                    let damage = Weapon::new(range(minimum, maximum)?, base_speed_ms)
                        .ok_or("Invalid weapon scaling speed")?;
                    (id(&key)?, 0, item.clone(), Attack::Weapon { item, damage })
                }
                RawAttack::Spell {
                    id: key,
                    ability,
                    minimum,
                    maximum,
                    coefficient_bp,
                } => {
                    let ability = id(&ability)?;
                    let damage = Spell::new(range(minimum, maximum)?, coefficient_bp)
                        .ok_or("Invalid spell scaling coefficient")?;
                    (
                        id(&key)?,
                        1,
                        ability.clone(),
                        Attack::Spell { ability, damage },
                    )
                }
                RawAttack::Creature {
                    id: key,
                    creature,
                    minimum,
                    maximum,
                } => {
                    let creature = id(&creature)?;
                    (
                        id(&key)?,
                        2,
                        creature.clone(),
                        Attack::Creature {
                            creature,
                            damage: range(minimum, maximum)?,
                        },
                    )
                }
            };
            counts[tag] += 1;
            if counts[tag] > [MAX_WEAPONS, MAX_SPELLS, MAX_CREATURES][tag]
                || !references.insert((tag, reference))
            {
                return Err("Duplicate damage reference or category budget exceeded");
            }
            if attacks.insert(key, attack).is_some() {
                return Err("Duplicate damage attack ID");
            }
        }
        Ok(Self {
            id: id(&raw.id)?,
            attacks,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn attack(&self, id: &ContentId) -> Option<&Attack> {
        self.attacks.get(id)
    }
    /// Stable logical-ID ordering; this is not yet a network ordinal contract.
    pub fn attacks(&self) -> impl ExactSizeIterator<Item = (&ContentId, &Attack)> {
        self.attacks.iter()
    }
    /// Cross-catalogue references only. The server must still bind exact content
    /// bytes and validate committed equipment, creature incarnation and ability use.
    pub fn validate_references(
        &self,
        items: &inventory::Catalog,
        abilities: &abilities::Catalog,
        creatures: &[ContentId],
    ) -> Result<(), &'static str> {
        if creatures.len() > MAX_CREATURES {
            return Err("Known creature definition budget exceeded");
        }
        for attack in self.attacks.values() {
            match attack {
                Attack::Weapon { item, .. } => {
                    if !valid_main_hand(items, item) {
                        return Err("Damage weapon requires a nonstackable main-hand item");
                    }
                }
                Attack::Spell { ability, .. } => {
                    if abilities.ability(ability.as_str()).is_none_or(|a| {
                        a.target() != abilities::Target::HostileAlive
                            || !matches!(a.effect(), abilities::Effect::Damage { .. })
                            || a.requires_main_hand()
                                .is_some_and(|item| !valid_main_hand(items, item))
                    }) {
                        return Err("Spell damage requires a hostile direct-damage ability");
                    }
                }
                Attack::Creature { creature, .. } => {
                    if !creatures.contains(creature) {
                        return Err("Unknown damage creature reference");
                    }
                }
            }
        }
        Ok(())
    }
}
