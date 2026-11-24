//! Current backpack possession and tentative consumption, not loot history.
//! Callers establish actor ownership, exact content binding and durable COMMIT.
use super::{Bag, Catalog, Error as InventoryError, MAX_REVISION, MAX_STACK, SLOT_COUNT};
use crate::content::ContentId;
use std::collections::BTreeMap;

pub const MAX_REQUIREMENTS: usize = 8;
pub const MAX_REQUIRED_COUNT: u32 = MAX_STACK * SLOT_COUNT as u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    item: ContentId,
    count: u32,
    consume_on_turn_in: bool,
}
impl Requirement {
    pub fn new(item: ContentId, count: u32, consume_on_turn_in: bool) -> Option<Self> {
        (1..=MAX_REQUIRED_COUNT).contains(&count).then_some(Self {
            item,
            count,
            consume_on_turn_in,
        })
    }
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn consume_on_turn_in(&self) -> bool {
        self.consume_on_turn_in
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirements {
    catalog: ContentId,
    entries: BTreeMap<ContentId, Requirement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidRequirements,
    CatalogMismatch,
    Inventory(InventoryError),
    MissingItems,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holding {
    item: ContentId,
    required: u32,
    held: u32,
}
impl Holding {
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn required(&self) -> u32 {
        self.required
    }
    /// Actual total, including surplus. Equipped items and future banks are
    /// outside this explicitly backpack-only predicate.
    pub fn held(&self) -> u32 {
        self.held
    }
    pub fn displayed_count(&self) -> u32 {
        self.held.min(self.required)
    }
    pub fn satisfied(&self) -> bool {
        self.held >= self.required
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    inventory_revision: u64,
    holdings: Vec<Holding>,
}
impl Observation {
    pub fn inventory_revision(&self) -> u64 {
        self.inventory_revision
    }
    pub fn holdings(&self) -> &[Holding] {
        &self.holdings
    }
    pub fn satisfied(&self) -> bool {
        self.holdings.iter().all(Holding::satisfied)
    }
}
impl Requirements {
    pub fn new(catalog: &Catalog, requirements: Vec<Requirement>) -> Result<Self, Error> {
        if requirements.is_empty() || requirements.len() > MAX_REQUIREMENTS {
            return Err(Error::InvalidRequirements);
        }
        let mut entries = BTreeMap::new();
        for requirement in requirements {
            if entries
                .insert(requirement.item.clone(), requirement)
                .is_some()
            {
                return Err(Error::InvalidRequirements);
            }
        }
        let requirements = Self {
            catalog: catalog.id().clone(),
            entries,
        };
        requirements.validate(catalog)?;
        Ok(requirements)
    }
    pub fn entries(&self) -> impl Iterator<Item = &Requirement> {
        self.entries.values()
    }
    fn validate(&self, catalog: &Catalog) -> Result<(), Error> {
        if catalog.id() != &self.catalog {
            return Err(Error::CatalogMismatch);
        }
        for requirement in self.entries.values() {
            let definition = catalog
                .definition(&requirement.item)
                .ok_or(Error::InvalidRequirements)?;
            if requirement.count == 0
                || requirement.count > definition.maximum_stack() * SLOT_COUNT as u32
            {
                return Err(Error::InvalidRequirements);
            }
        }
        Ok(())
    }
    /// Read current committed holdings; an old observation is never an input to
    /// consumption. Pure evaluation establishes neither ownership nor durability.
    pub fn observe(&self, catalog: &Catalog, bag: &Bag) -> Result<Observation, Error> {
        self.validate(catalog)?;
        bag.validate(catalog).map_err(Error::Inventory)?;
        let mut holdings = Vec::with_capacity(self.entries.len());
        for requirement in self.entries.values() {
            let held = bag
                .slots
                .iter()
                .flatten()
                .filter(|s| s.item == requirement.item)
                .try_fold(0u32, |n, s| n.checked_add(s.count))
                .ok_or(Error::Inventory(InventoryError::InvalidState))?;
            holdings.push(Holding {
                item: requirement.item.clone(),
                required: requirement.count,
                held,
            });
        }
        Ok(Observation {
            inventory_revision: bag.revision,
            holdings,
        })
    }
    /// Tentative all-or-nothing consumption in stable slot order. Recheck the
    /// supplied current bag, including nonconsuming possession requirements.
    /// No client command, SQL, reward or quest completion is performed here.
    pub fn propose_consumption(&self, catalog: &Catalog, bag: &Bag) -> Result<Bag, Error> {
        if !self.observe(catalog, bag)?.satisfied() {
            return Err(Error::MissingItems);
        }
        if self.entries.values().all(|r| !r.consume_on_turn_in) {
            return Ok(bag.clone());
        }
        let revision = bag
            .revision
            .checked_add(1)
            .filter(|r| *r <= MAX_REVISION)
            .ok_or(Error::Inventory(InventoryError::RevisionExhausted))?;
        let mut next = bag.clone();
        for requirement in self.entries.values().filter(|r| r.consume_on_turn_in) {
            let mut remaining = requirement.count;
            for slot in &mut next.slots {
                if remaining == 0 {
                    break;
                }
                if let Some(stack) = slot.as_mut().filter(|s| s.item == requirement.item) {
                    let used = stack.count.min(remaining);
                    stack.count -= used;
                    remaining -= used;
                    if stack.count == 0 {
                        *slot = None;
                    }
                }
            }
            if remaining != 0 {
                return Err(Error::MissingItems);
            }
        }
        next.revision = revision;
        next.validate(catalog).map_err(Error::Inventory)?;
        Ok(next)
    }
}
