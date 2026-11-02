//! Bounded immutable development ability data. No combat execution or I/O.
pub mod classic;
use crate::{content::ContentId, time::TickRate};
use serde::Deserialize;
use std::{collections::BTreeSet, time::Duration};

pub const MAX_CATALOG_BYTES: usize = 64 * 1024;
pub const MAX_ABILITIES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    HostileAlive,
}
/// Presentation hint only; never defines damage, movement or hit timing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationCue {
    SwordAttack,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Damage { amount: u32 },
    ApplyStatus { status: ContentId },
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawEffect {
    Damage { amount: u32 },
    ApplyStatus { status: String },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    global_cooldown_ms: u32,
    queue_window_ms: u32,
    abilities: Vec<RawAbility>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAbility {
    id: String,
    target: Target,
    range_metres: f64,
    cast_ms: u32,
    cooldown_ms: u32,
    #[serde(default = "enabled")]
    cast_while_moving: bool,
    #[serde(default = "enabled")]
    uses_global_cooldown: bool,
    #[serde(default)]
    mana_cost: Option<u32>,
    #[serde(default, deserialize_with = "required_item")]
    requires_main_hand: Option<String>,
    #[serde(default)]
    animation: Option<AnimationCue>,
    #[serde(default, deserialize_with = "hit_table")]
    hit_table: Option<crate::hit_outcomes::Table>,
    #[serde(default, deserialize_with = "projectile")]
    projectile: Option<crate::projectiles::Profile>,
    #[serde(default, deserialize_with = "combat_rules")]
    combat: Option<classic::Rules>,
    effect: RawEffect,
}
fn combat_rules<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<classic::Rules>, D::Error> {
    classic::Rules::deserialize(d).map(Some)
}
fn hit_table<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<crate::hit_outcomes::Table>, D::Error> {
    crate::hit_outcomes::Table::deserialize(d).map(Some)
}
fn projectile<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<crate::projectiles::Profile>, D::Error> {
    crate::projectiles::Profile::deserialize(d).map(Some)
}
fn required_item<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}
fn enabled() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ability {
    id: ContentId,
    target: Target,
    range_metres: f64,
    cast_ms: u32,
    cooldown_ms: u32,
    cast_while_moving: bool,
    uses_global_cooldown: bool,
    mana_cost: u32,
    requires_main_hand: Option<ContentId>,
    animation: Option<AnimationCue>,
    hit_table: Option<crate::hit_outcomes::Table>,
    projectile: Option<crate::projectiles::Profile>,
    combat: Option<classic::Rules>,
    effect: Effect,
}
impl Ability {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn target(&self) -> Target {
        self.target
    }
    pub fn combat_rules(&self) -> Option<classic::Rules> {
        self.combat
    }
    pub fn range_metres(&self) -> f64 {
        self.range_metres
    }
    pub fn cast_while_moving(&self) -> bool {
        self.cast_while_moving
    }
    pub fn uses_global_cooldown(&self) -> bool {
        self.uses_global_cooldown
    }
    pub fn mana_cost(&self) -> u32 {
        self.mana_cost
    }
    pub fn requires_main_hand(&self) -> Option<&ContentId> {
        self.requires_main_hand.as_ref()
    }
    pub fn animation(&self) -> Option<AnimationCue> {
        self.animation
    }
    pub fn hit_table(&self) -> Option<crate::hit_outcomes::Table> {
        self.hit_table
    }
    pub fn projectile(&self) -> Option<crate::projectiles::Profile> {
        self.projectile
    }
    pub fn effect(&self) -> Effect {
        self.effect.clone()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    schema: u32,
    abilities: Vec<Ability>,
    global_cooldown_ms: u32,
    queue_window_ms: u32,
}
/// Durations only; this does not acknowledge, queue or execute any cast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub cast_ticks: u64,
    pub cooldown_ticks: u64,
    pub global_cooldown_ticks: u64,
    pub queue_window_ticks: u64,
}
fn ticks(ms: u32, rate: TickRate) -> u64 {
    rate.ticks_for(Duration::from_millis(u64::from(ms)))
        .expect("Validated millisecond budgets fit every qualified tick rate")
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Ability catalog exceeds byte budget".into());
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid ability catalog: {e}"))?;
        if !matches!(raw.schema, 1..=7) {
            return Err("Unsupported ability schema".into());
        }
        if raw.abilities.is_empty() || raw.abilities.len() > MAX_ABILITIES {
            return Err("Ability count outside prototype budget".into());
        }
        if raw.global_cooldown_ms > 10_000 || raw.queue_window_ms > 1_000 {
            return Err("Global cooldown/queue timing exceeds prototype budget".into());
        }
        let mut names = BTreeSet::new();
        let mut abilities = Vec::with_capacity(raw.abilities.len());
        for ability in raw.abilities {
            let mana_cost = match (raw.schema, ability.mana_cost) {
                (1, None) => 0,
                (2..=7, Some(cost)) if cost <= crate::resources::MAX_VALUE => cost,
                _ => {
                    return Err(
                        "Ability mana cost is missing, unsupported or outside bounds".into(),
                    );
                }
            };
            let id = ContentId::new(&ability.id).ok_or("Invalid stable ability ID")?;
            if raw.schema < 4
                && (ability.requires_main_hand.is_some() || ability.animation.is_some())
            {
                return Err(
                    "Equipment requirements and animation cues require ability schema 4".into(),
                );
            }
            let requires_main_hand = ability
                .requires_main_hand
                .as_deref()
                .map(|id| ContentId::new(id).ok_or("Invalid required main-hand item ID"))
                .transpose()?;
            if ability.animation == Some(AnimationCue::SwordAttack) && requires_main_hand.is_none()
            {
                return Err("Sword attack presentation requires an explicit main-hand item".into());
            }
            if !names.insert(ability.id) {
                return Err("Duplicate ability ID".into());
            }
            if !ability.range_metres.is_finite()
                || ability.range_metres <= 0.
                || ability.range_metres > 128.
            {
                return Err("Ability range outside prototype budget".into());
            }
            if (raw.schema == 7) != ability.combat.is_some() {
                return Err("Explicit combat rules require ability schema 7".into());
            }
            if let Some(rules) = ability.combat {
                rules.validate(ability.range_metres)?;
            }
            if ability.cast_ms > 60_000 || ability.cooldown_ms > 60_000 {
                return Err("Ability timing exceeds prototype budget".into());
            }
            let effect = match ability.effect {
                RawEffect::Damage { amount } if amount == 0 || amount > 1_000_000 => {
                    return Err("Ability damage outside prototype budget".into());
                }
                RawEffect::Damage { amount } => Effect::Damage { amount },
                RawEffect::ApplyStatus { status } if raw.schema >= 3 => Effect::ApplyStatus {
                    status: ContentId::new(&status).ok_or("Invalid status reference")?,
                },
                RawEffect::ApplyStatus { .. } => {
                    return Err("Status effects require ability schema 3".into());
                }
            };
            if ability.hit_table.is_some()
                && (raw.schema < 5 || !matches!(effect, Effect::Damage { .. }))
            {
                return Err("Hit tables require schema 5 direct damage abilities".into());
            }
            if let Some(profile) = ability.projectile {
                if raw.schema < 6 || !matches!(effect, Effect::Damage { .. }) {
                    return Err("Projectiles require schema 6 direct damage abilities".into());
                }
                if profile.maximum_distance_metres() < ability.range_metres
                    || profile.speed_metres_per_second() * profile.lifetime().as_secs_f64()
                        < ability.range_metres
                {
                    return Err("Projectile travel cannot cover its declared launch range".into());
                }
            }
            abilities.push(Ability {
                id,
                target: ability.target,
                range_metres: ability.range_metres,
                cast_ms: ability.cast_ms,
                cooldown_ms: ability.cooldown_ms,
                cast_while_moving: ability.cast_while_moving,
                uses_global_cooldown: ability.uses_global_cooldown,
                mana_cost,
                requires_main_hand,
                animation: ability.animation,
                hit_table: ability.hit_table,
                projectile: ability.projectile,
                combat: ability.combat,
                effect,
            });
        }
        Ok(Self {
            schema: raw.schema,
            abilities,
            global_cooldown_ms: raw.global_cooldown_ms,
            queue_window_ms: raw.queue_window_ms,
        })
    }
    pub fn schema(&self) -> u32 {
        self.schema
    }
    pub fn abilities(&self) -> &[Ability] {
        &self.abilities
    }
    /// Cross-profile startup validation, independent of an actor's live gear.
    pub fn validate_equipment(
        &self,
        items: Option<&crate::inventory::Catalog>,
    ) -> Result<(), String> {
        for ability in &self.abilities {
            if let Some(required) = ability.requires_main_hand()
                && !items.is_some_and(|items| {
                    items.equipment_enabled()
                        && items
                            .definition(required)
                            .is_some_and(|item| item.main_hand_appearance().is_some())
                })
            {
                return Err("Ability requires an undeclared or unequippable main-hand item".into());
            }
        }
        Ok(())
    }
    pub fn ability(&self, id: &str) -> Option<&Ability> {
        self.abilities.iter().find(|a| a.id.as_str() == id)
    }
    pub fn timing(&self, id: &str, rate: TickRate) -> Option<Timing> {
        let ability = self.ability(id)?;
        Some(Timing {
            cast_ticks: ticks(ability.cast_ms, rate),
            cooldown_ticks: ticks(ability.cooldown_ms, rate),
            global_cooldown_ticks: if ability.uses_global_cooldown {
                ticks(self.global_cooldown_ms, rate)
            } else {
                0
            },
            queue_window_ticks: ticks(self.queue_window_ms, rate),
        })
    }
}
