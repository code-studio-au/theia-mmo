//! Checked fixed barter and pure atomic exchange proposals. No NPC proximity,
//! ownership, durable receipt, commit or purchase acknowledgement is established.
use crate::{
    content::ContentId,
    inventory::{self, Bag, Stack},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CATALOG_BYTES: usize = 16 * 1024;
pub const MAX_OFFERS: usize = 8;
pub const MAX_TRADE_UNITS: u32 = 1000;
pub const PROFILE_DOMAIN: &[u8] = b"theia-vendor-barter-profile-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    npc: String,
    item_catalog: String,
    offers: Vec<RawOffer>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOffer {
    id: String,
    name: String,
    cost: RawAmount,
    grant: RawAmount,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAmount {
    item: String,
    count: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amount {
    item: ContentId,
    count: u32,
}
impl Amount {
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    id: ContentId,
    name: String,
    cost: Amount,
    grant: Amount,
}
impl Offer {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn cost(&self) -> &Amount {
        &self.cost
    }
    pub fn grant(&self) -> &Amount {
        &self.grant
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    npc: ContentId,
    items: inventory::Catalog,
    offers: BTreeMap<ContentId, Offer>,
}
fn define(
    value: &str,
    ids: &mut BTreeSet<ContentId>,
    items: &inventory::Catalog,
) -> Result<ContentId, &'static str> {
    let id = ContentId::new(value).ok_or("Invalid vendor definition ID")?;
    if !ids.insert(id.clone()) || items.definition(&id).is_some() {
        return Err("Vendor definition IDs alias");
    }
    Ok(id)
}
fn amount(raw: RawAmount, items: &inventory::Catalog, cost: bool) -> Result<Amount, &'static str> {
    let item = ContentId::new(&raw.item).ok_or("Invalid barter item ID")?;
    let definition = items.definition(&item).ok_or("Unknown barter item")?;
    if raw.count == 0
        || raw.count > MAX_TRADE_UNITS
        || raw.count > inventory::SLOT_COUNT as u32 * definition.maximum_stack()
    {
        return Err("Barter quantity exceeds bounded backpack capacity");
    }
    if cost && definition.main_hand_appearance().is_some() {
        return Err("Initial barter cannot consume equippable costs");
    }
    Ok(Amount {
        item,
        count: raw.count,
    })
}
impl Catalog {
    pub fn from_json(bytes: &[u8], items: &inventory::Catalog) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Vendor profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid vendor profile")?;
        if raw.schema != 1 || !(1..=MAX_OFFERS).contains(&raw.offers.len()) {
            return Err("Invalid vendor schema/offer count");
        }
        let item_catalog = ContentId::new(&raw.item_catalog)
            .filter(|id| id == items.id())
            .ok_or("Vendor item catalog mismatch")?;
        let mut ids = BTreeSet::from([item_catalog]);
        let id = define(&raw.id, &mut ids, items)?;
        let npc = define(&raw.npc, &mut ids, items)?;
        let mut offers = BTreeMap::new();
        for raw in raw.offers {
            let id = define(&raw.id, &mut ids, items)?;
            if raw.name.is_empty()
                || raw.name.len() > 64
                || raw.name.trim() != raw.name
                || raw.name.chars().any(char::is_control)
            {
                return Err("Invalid barter offer name");
            }
            let cost = amount(raw.cost, items, true)?;
            let grant = amount(raw.grant, items, false)?;
            if cost.item == grant.item {
                return Err("Initial barter cost/output items must differ");
            }
            offers.insert(
                id.clone(),
                Offer {
                    id,
                    name: raw.name,
                    cost,
                    grant,
                },
            );
        }
        Ok(Self {
            id,
            npc,
            items: items.clone(),
            offers,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn npc(&self) -> &ContentId {
        &self.npc
    }
    pub fn item_catalog(&self) -> &ContentId {
        self.items.id()
    }
    pub fn offer(&self, id: &ContentId) -> Option<&Offer> {
        self.offers.get(id)
    }
    pub fn offers(&self) -> impl ExactSizeIterator<Item = &Offer> {
        self.offers.values()
    }
    /// Bind checked definitions, not merely their logical name. Exact source
    /// byte hashes and live purchase authorization belong to the outer boundary.
    pub fn propose(
        &self,
        bag: &Bag,
        items: &inventory::Catalog,
        purchase: &Purchase,
    ) -> Result<Bag, Error> {
        if items != &self.items || bag.catalog() != items.id() {
            return Err(Error::CatalogMismatch);
        }
        bag.validate(items).map_err(Error::ItemState)?;
        if purchase.expected_revision != bag.revision() {
            return Err(Error::StaleRevision);
        }
        let offer = self
            .offers
            .get(&purchase.offer)
            .ok_or(Error::UnknownOffer)?;
        let revision = bag
            .revision()
            .checked_add(1)
            .filter(|r| *r <= inventory::MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let mut slots = bag.slots().clone();
        let mut remaining = offer.cost.count;
        for slot in &mut slots {
            if let Some(stack) = slot.as_ref().filter(|s| s.item() == &offer.cost.item) {
                let take = remaining.min(stack.count());
                let count = stack.count() - take;
                *slot = if count == 0 {
                    None
                } else {
                    Some(
                        Stack::new(items, offer.cost.item.clone(), count)
                            .ok_or(Error::InvalidRecipe)?,
                    )
                };
                remaining -= take;
            }
        }
        if remaining > 0 {
            return Err(Error::InsufficientCost);
        }
        // Reviewed reward policy (rewards.rs): merge in slot order, then fill
        // empties. All work stays within three scans of sixteen slots.
        let limit = items
            .definition(&offer.grant.item)
            .ok_or(Error::InvalidRecipe)?
            .maximum_stack();
        let mut remaining = offer.grant.count;
        for slot in &mut slots {
            if let Some(stack) = slot.as_ref().filter(|s| s.item() == &offer.grant.item) {
                let count = stack.count();
                let add = remaining.min(limit - count);
                if add > 0 {
                    *slot = Some(
                        Stack::new(items, offer.grant.item.clone(), count + add)
                            .ok_or(Error::InvalidRecipe)?,
                    );
                    remaining -= add;
                }
            }
        }
        for slot in &mut slots {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                let add = remaining.min(limit);
                *slot = Some(
                    Stack::new(items, offer.grant.item.clone(), add).ok_or(Error::InvalidRecipe)?,
                );
                remaining -= add;
            }
        }
        if remaining > 0 {
            return Err(Error::NoCapacity);
        }
        Bag::from_parts(items, revision, slots, bag.main_hand().cloned()).map_err(Error::ItemState)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Purchase {
    pub expected_revision: u64,
    pub offer: ContentId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    CatalogMismatch,
    ItemState(inventory::Error),
    StaleRevision,
    UnknownOffer,
    RevisionExhausted,
    InsufficientCost,
    NoCapacity,
    InvalidRecipe,
}

/// Byte-profile-bound offer index and the observed live interaction context.
/// This is an intent, never a price, recipe or authorization to change items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub expected_revision: u64,
    pub dialogue_revision: u64,
    pub npc: crate::identity::EntityKey,
    pub offer: u8,
}
