//! Checked nonrepeatable reward recipes and pure trusted grant proposals.
//! A proposal does not authorize a recipient, consume a claim, commit items or
//! acknowledge success. Runtime input must never supply a recipe or item list.
use crate::{
    content::ContentId,
    inventory::{self, Bag, Stack},
};
use serde::Deserialize;
use std::collections::BTreeMap;
pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const MAX_DEFINITIONS: usize = 16;
pub const MAX_GRANTS: usize = 8;
pub const MAX_GRANT_UNITS: u32 = 10_000;
pub const PROFILE_DOMAIN: &[u8] = b"theia-reward-profile-v1\0";
pub const CLAIM_DOMAIN: &[u8] = b"theia-reward-claim-v1\0";
/// Opaque server-generated persistent claim key. It conveys no recipient
/// authority; the owning character/world must still be checked by the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaimId([u8; 16]);
impl ClaimId {
    pub fn new(bytes: [u8; 16]) -> Option<Self> {
        (bytes != [0; 16]).then_some(Self(bytes))
    }
    pub fn bytes(self) -> [u8; 16] {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClaimCommand {
    pub claim: ClaimId,
    pub expected_revision: u64,
}
impl ClaimCommand {
    pub fn canonical_bytes(self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes[..16].copy_from_slice(&self.claim.bytes());
        bytes[16..].copy_from_slice(&self.expected_revision.to_le_bytes());
        bytes
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Checked policy metadata. The durable claim owner enforces consumption;
    /// calling a pure proposal repeatedly does not enforce nonrepeatability.
    OncePerCharacter,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    item_catalog: String,
    rewards: Vec<RawDefinition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefinition {
    id: String,
    name: String,
    scope: Scope,
    creature: String,
    grants: Vec<RawGrant>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGrant {
    item: String,
    count: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    item: ContentId,
    count: u32,
}
impl Grant {
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    id: ContentId,
    name: String,
    scope: Scope,
    creature: ContentId,
    item_catalog: ContentId,
    grants: Vec<Grant>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    item_catalog: ContentId,
    rewards: BTreeMap<ContentId, Definition>,
}
impl Catalog {
    pub fn from_json(bytes: &[u8], items: &inventory::Catalog) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Reward catalog exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid reward catalog")?;
        if raw.schema != 1 || !(1..=MAX_DEFINITIONS).contains(&raw.rewards.len()) {
            return Err("Unsupported reward schema/count");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid reward catalog ID")?;
        let item_catalog = ContentId::new(&raw.item_catalog)
            .filter(|id| id == items.id())
            .ok_or("Reward item catalog mismatch")?;
        let mut rewards = BTreeMap::new();
        for raw in raw.rewards {
            let id = ContentId::new(&raw.id).ok_or("Invalid reward ID")?;
            let creature = ContentId::new(&raw.creature).ok_or("Invalid qualifying creature ID")?;
            if !(1..=48).contains(&raw.name.len())
                || raw.name.trim() != raw.name
                || !raw
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b" '-".contains(&c))
                || !raw.name.bytes().any(|c| c.is_ascii_alphanumeric())
                || !(1..=MAX_GRANTS).contains(&raw.grants.len())
            {
                return Err("Invalid development reward name/grant count");
            }
            let mut grants = BTreeMap::new();
            let mut units = 0_u32;
            for raw in raw.grants {
                let item = ContentId::new(&raw.item)
                    .filter(|id| items.definition(id).is_some())
                    .ok_or("Unknown reward item")?;
                units = units
                    .checked_add(raw.count)
                    .ok_or("Reward quantity overflow")?;
                if raw.count == 0 || units > MAX_GRANT_UNITS {
                    return Err("Reward quantity exceeds budget");
                }
                if grants
                    .insert(
                        item.clone(),
                        Grant {
                            item,
                            count: raw.count,
                        },
                    )
                    .is_some()
                {
                    return Err("Duplicate reward grant item");
                }
            }
            let definition = Definition {
                id: id.clone(),
                name: raw.name,
                scope: raw.scope,
                creature,
                item_catalog: item_catalog.clone(),
                grants: grants.into_values().collect(),
            };
            if rewards.insert(id, definition).is_some() {
                return Err("Duplicate reward ID");
            }
        }
        Ok(Self {
            id,
            item_catalog,
            rewards,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn item_catalog(&self) -> &ContentId {
        &self.item_catalog
    }
    pub fn definition(&self, id: &ContentId) -> Option<&Definition> {
        self.rewards.get(id)
    }
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &Definition> {
        self.rewards.values()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    CatalogMismatch,
    ItemState(inventory::Error),
    StaleRevision,
    RevisionExhausted,
    InvalidGrant,
    NoCapacity,
}
impl Definition {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn scope(&self) -> Scope {
        self.scope
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn grants(&self) -> &[Grant] {
        &self.grants
    }
    /// First merge matching stacks in slot order, then fill empty slots. Work is
    /// bounded by MAX_GRANTS * SLOT_COUNT, not by the granted unit count. Equipment
    /// is preserved; a failure returns no partially granted state. The persistence
    /// envelope must bind both catalog byte digests and authorize the durable claim.
    pub fn propose(
        &self,
        bag: &Bag,
        items: &inventory::Catalog,
        expected_revision: u64,
    ) -> Result<Bag, Error> {
        if &self.item_catalog != items.id() {
            return Err(Error::CatalogMismatch);
        }
        bag.validate(items).map_err(Error::ItemState)?;
        if bag.revision() != expected_revision {
            return Err(Error::StaleRevision);
        }
        let revision = bag
            .revision()
            .checked_add(1)
            .filter(|r| *r <= inventory::MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let mut slots = bag.slots().clone();
        for grant in &self.grants {
            let limit = items
                .definition(&grant.item)
                .ok_or(Error::InvalidGrant)?
                .maximum_stack();
            let mut remaining = grant.count;
            for slot in &mut slots {
                if let Some(stack) = slot.as_ref().filter(|s| s.item() == &grant.item) {
                    let count = stack.count();
                    let amount = remaining.min(limit - count);
                    if amount > 0 {
                        *slot = Some(
                            Stack::new(items, grant.item.clone(), count + amount)
                                .ok_or(Error::InvalidGrant)?,
                        );
                        remaining -= amount;
                    }
                }
            }
            for slot in &mut slots {
                if remaining == 0 {
                    break;
                }
                if slot.is_none() {
                    let amount = remaining.min(limit);
                    *slot = Some(
                        Stack::new(items, grant.item.clone(), amount).ok_or(Error::InvalidGrant)?,
                    );
                    remaining -= amount;
                }
            }
            if remaining > 0 {
                return Err(Error::NoCapacity);
            }
        }
        Bag::from_parts(items, revision, slots, bag.main_hand().cloned()).map_err(Error::ItemState)
    }
}
