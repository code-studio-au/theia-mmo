//! Bounded immutable development ability data. No combat execution or I/O.
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
    effect: RawEffect,
}
fn enabled() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct Ability {
    id: ContentId,
    target: Target,
    range_metres: f64,
    cast_ms: u32,
    cooldown_ms: u32,
    cast_while_moving: bool,
    uses_global_cooldown: bool,
    mana_cost: u32,
    effect: Effect,
}
impl Ability {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn target(&self) -> Target {
        self.target
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
    pub fn effect(&self) -> Effect {
        self.effect.clone()
    }
}
#[derive(Clone, Debug)]
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
        if !matches!(raw.schema, 1..=3) {
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
                (2 | 3, Some(cost)) if cost <= crate::resources::MAX_VALUE => cost,
                _ => {
                    return Err(
                        "Ability mana cost is missing, unsupported or outside bounds".into(),
                    );
                }
            };
            let id = ContentId::new(&ability.id).ok_or("Invalid stable ability ID")?;
            if !names.insert(ability.id) {
                return Err("Duplicate ability ID".into());
            }
            if !ability.range_metres.is_finite()
                || ability.range_metres <= 0.
                || ability.range_metres > 128.
            {
                return Err("Ability range outside prototype budget".into());
            }
            if ability.cast_ms > 60_000 || ability.cooldown_ms > 60_000 {
                return Err("Ability timing exceeds prototype budget".into());
            }
            let effect = match ability.effect {
                RawEffect::Damage { amount } if amount == 0 || amount > 1_000_000 => {
                    return Err("Ability damage outside prototype budget".into());
                }
                RawEffect::Damage { amount } => Effect::Damage { amount },
                RawEffect::ApplyStatus { status } if raw.schema == 3 => Effect::ApplyStatus {
                    status: ContentId::new(&status).ok_or("Invalid status reference")?,
                },
                RawEffect::ApplyStatus { .. } => {
                    return Err("Status effects require ability schema 3".into());
                }
            };
            abilities.push(Ability {
                id,
                target: ability.target,
                range_metres: ability.range_metres,
                cast_ms: ability.cast_ms,
                cooldown_ms: ability.cooldown_ms,
                cast_while_moving: ability.cast_while_moving,
                uses_global_cooldown: ability.uses_global_cooldown,
                mana_cost,
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
