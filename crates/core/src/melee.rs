//! Immutable melee weapon fixtures and independent swing timing.
//! These primitives do not select targets, validate hits or authorize damage.
pub mod timer;
use crate::{content::ContentId, facing::Arc, inventory, time::TickRate};
use serde::Deserialize;
use std::{collections::BTreeMap, time::Duration};
pub const MAX_PROFILE_BYTES: usize = 32 * 1024;
pub const MAX_WEAPONS: usize = 32;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    weapons: Vec<RawWeapon>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWeapon {
    item: String,
    period_ms: u32,
    damage: u32,
    reach_metres: f64,
    frontal_arc_degrees: f64,
}
#[derive(Clone, Debug)]
pub struct Weapon {
    item: ContentId,
    period_ms: u32,
    damage: u32,
    reach_metres: f64,
    arc: Arc,
}
impl Weapon {
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn period_ms(&self) -> u32 {
        self.period_ms
    }
    pub fn period_ticks(&self, rate: TickRate) -> u64 {
        rate.ticks_for(Duration::from_millis(u64::from(self.period_ms)))
            .unwrap()
    }
    pub fn damage(&self) -> u32 {
        self.damage
    }
    pub fn reach_metres(&self) -> f64 {
        self.reach_metres
    }
    pub fn arc(&self) -> Arc {
        self.arc
    }
}
#[derive(Clone, Debug)]
pub struct Catalog {
    id: ContentId,
    weapons: BTreeMap<ContentId, Weapon>,
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Melee weapon profile exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid melee weapon profile")?;
        if raw.schema != 1 || !(1..=MAX_WEAPONS).contains(&raw.weapons.len()) {
            return Err("Invalid melee weapon schema/count");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid melee profile identity")?;
        let mut weapons = BTreeMap::new();
        for raw in raw.weapons {
            let item = ContentId::new(&raw.item).ok_or("Invalid melee weapon item")?;
            if !(100..=10_000).contains(&raw.period_ms)
                || !(1..=1_000_000).contains(&raw.damage)
                || !raw.reach_metres.is_finite()
                || !(0.1..=10.).contains(&raw.reach_metres)
            {
                return Err("Melee weapon outside prototype bounds");
            }
            let weapon = Weapon {
                item: item.clone(),
                period_ms: raw.period_ms,
                damage: raw.damage,
                reach_metres: raw.reach_metres,
                arc: Arc::from_degrees(raw.frontal_arc_degrees).ok_or("Invalid melee arc")?,
            };
            if weapons.insert(item, weapon).is_some() {
                return Err("Duplicate melee weapon item");
            }
        }
        Ok(Self { id, weapons })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn weapon(&self, item: &ContentId) -> Option<&Weapon> {
        self.weapons.get(item)
    }
    /// Stable content-ID ordering for catalog-bound wire ordinals.
    pub fn weapons(&self) -> impl ExactSizeIterator<Item = &Weapon> {
        self.weapons.values()
    }
    /// Validate immutable item references, not a player's equipment ownership.
    pub fn validate_items(&self, items: &inventory::Catalog) -> Result<(), &'static str> {
        for item in self.weapons.keys() {
            if items
                .definition(item)
                .is_none_or(|d| d.maximum_stack() != 1 || d.main_hand_appearance().is_none())
            {
                return Err("Melee weapon requires a known nonstackable main-hand item");
            }
        }
        Ok(())
    }
}
