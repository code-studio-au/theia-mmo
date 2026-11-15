//! Checked reward recipes and pure trusted grant proposals.
//! A proposal does not authorize a recipient, consume a claim, commit items or
//! acknowledge success. Runtime input must never supply a recipe or item list.
use crate::{
    content::ContentId,
    inventory::{self, Bag, Stack},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const MAX_DEFINITIONS: usize = 16;
/// Bounded visible page; uncollected loot outside it remains durable.
pub const MAX_VISIBLE_LIFE_CLAIMS: usize = 16;
/// Initial personal-corpse presentation radius; pickup has its own authority check.
pub const LOOT_INTEREST_METRES: f64 = 64.;
/// Immutable source location, not proof of death or recipient entitlement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorpseSource {
    pub life: crate::progression::KillId,
    pub position: crate::coordinates::GlobalPosition,
}
pub const MAX_GRANTS: usize = 8;
pub const MAX_GRANT_UNITS: u32 = 10_000;
pub const PROFILE_DOMAIN: &[u8] = b"theia-reward-profile-v1\0";
pub const QUEST_PROFILE_DOMAIN: &[u8] = b"theia-quest-loot-profile-v1\0";
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
    /// One durable entitlement per recipient and authoritative victim life.
    /// Schemas 2–4 admit this policy; storage/runtime support is separate.
    PerCreatureLife,
}
/// Scope discriminator for a trusted kill selector. This conveys no recipient
/// authorization, proof of death, durable offer or item grant by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occurrence {
    OncePerCharacter,
    CreatureLife(crate::progression::KillId),
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
    #[serde(default)]
    creature: Option<String>,
    ground: Option<RawGround>,
    grants: Vec<RawGrant>,
    chance_basis_points: Option<u16>,
    drop_group: Option<u8>,
    quest_required: Option<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGround {
    position: [f64; 3],
    quest: String,
    area: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Ground {
    pub position: crate::coordinates::GlobalPosition,
    pub quest: ContentId,
    pub area: String,
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
    pub(crate) fn checked(items: &inventory::Catalog, item: ContentId, count: u32) -> Option<Self> {
        (items.definition(&item).is_some() && (1..=MAX_GRANT_UNITS).contains(&count))
            .then_some(Self { item, count })
    }
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    id: ContentId,
    name: String,
    scope: Scope,
    creature: ContentId,
    item_catalog: ContentId,
    grants: Vec<Grant>,
    chance_basis_points: u16,
    drop_group: u8,
    roll_policy: bool,
    quest_required: bool,
    ground: Option<Ground>,
}
#[derive(Clone, Debug, PartialEq)]
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
        if !matches!(raw.schema, 1..=5) || !(1..=MAX_DEFINITIONS).contains(&raw.rewards.len()) {
            return Err("Unsupported reward schema/count");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid reward catalog ID")?;
        let item_catalog = ContentId::new(&raw.item_catalog)
            .filter(|id| id == items.id())
            .ok_or("Reward item catalog mismatch")?;
        let schema = raw.schema;
        let mut rewards = BTreeMap::new();
        for raw in raw.rewards {
            if schema == 1 && raw.scope == Scope::PerCreatureLife {
                return Err("Per-creature-life rewards require schema 2");
            }
            if schema < 3 && (raw.chance_basis_points.is_some() || raw.drop_group.is_some()) {
                return Err("Drop chances require schema 3");
            }
            let chance_basis_points = raw.chance_basis_points.unwrap_or(10_000);
            let drop_group = raw.drop_group.unwrap_or(0);
            if schema < 4 && raw.quest_required.is_some() {
                return Err("Quest-required metadata requires schema 4");
            }
            let quest_required = raw.quest_required.unwrap_or(false);
            if raw.ground.is_some()
                && (schema < 5
                    || raw.creature.is_some()
                    || raw.scope != Scope::OncePerCharacter
                    || !quest_required
                    || raw.grants.len() != 1
                    || raw.grants[0].count != 1
                    || raw.chance_basis_points.is_some()
                    || raw.drop_group.is_some())
            {
                return Err(
                    "Ground items require schema 5, one quest item and once-character scope",
                );
            }
            let ground = raw
                .ground
                .map(|g| {
                    let position = crate::coordinates::GlobalPosition::new(g.position)
                        .ok_or("Invalid ground position")?;
                    let quest = ContentId::new(&g.quest).ok_or("Invalid ground quest")?;
                    if g.area.is_empty()
                        || g.area.len() > 96
                        || g.area.trim() != g.area
                        || g.area.chars().any(char::is_control)
                    {
                        return Err("Invalid ground area");
                    }
                    Ok(Ground {
                        position,
                        quest,
                        area: g.area,
                    })
                })
                .transpose()?;
            if quest_required
                && ((raw.scope != Scope::PerCreatureLife && ground.is_none())
                    || raw.grants.len() != 1
                    || drop_group != 0)
            {
                return Err("Quest-required loot requires one independent per-life item grant");
            }
            if chance_basis_points > 10_000
                || ((raw.chance_basis_points.is_some() || raw.drop_group.is_some())
                    && raw.scope != Scope::PerCreatureLife)
            {
                return Err("Invalid per-life drop chance/scope");
            }
            let id = ContentId::new(&raw.id).ok_or("Invalid reward ID")?;
            let creature = if ground.is_some() {
                id.clone()
            } else {
                ContentId::new(
                    raw.creature
                        .as_deref()
                        .ok_or("Missing qualifying creature")?,
                )
                .ok_or("Invalid qualifying creature ID")?
            };
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
                chance_basis_points,
                drop_group,
                roll_policy: schema >= 3 && raw.scope == Scope::PerCreatureLife,
                quest_required,
                ground,
            };
            if rewards.insert(id, definition).is_some() {
                return Err("Duplicate reward ID");
            }
        }
        let mut groups = BTreeMap::new();
        for definition in rewards.values().filter(|d| d.drop_group != 0) {
            let total = groups
                .entry((definition.creature.clone(), definition.drop_group))
                .or_insert(0_u32);
            *total += u32::from(definition.chance_basis_points);
            if *total > 10_000 {
                return Err("Drop group chances exceed 100 percent");
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
    /// Metadata only. Runtime must bind generation rights and current need.
    pub fn has_quest_required(&self) -> bool {
        self.rewards.values().any(|d| d.quest_required)
    }
    /// Cross-reference metadata only; does not inspect an accepted journal or bag.
    pub fn validate_quest_items(
        &self,
        quests: &crate::quests::catalog::Catalog,
    ) -> Result<(), &'static str> {
        for reward in self.definitions().filter(|d| d.ground.is_some()) {
            let ground = reward.ground.as_ref().unwrap();
            let item = reward.grants[0].item();
            let owners: Vec<_> = quests
                .quests()
                .filter(|q| {
                    q.stages().flat_map(|s| s.objectives()).any(|o| {
                        o.event() == &crate::quests::catalog::EventKey::PossessItem(item.clone())
                            && o.required() == 1
                            && o.consume_on_turn_in()
                    })
                })
                .collect();
            if owners.len() != 1 || owners[0].id() != &ground.quest {
                return Err("Ground pickup needs one exact item delivery quest");
            }
        }
        let required: BTreeSet<_> = quests
            .quests()
            .flat_map(|q| q.stages())
            .flat_map(|s| s.objectives())
            .filter_map(|o| match o.event() {
                crate::quests::catalog::EventKey::PossessItem(item) => Some(item),
                _ => None,
            })
            .collect();
        if self
            .definitions()
            .filter_map(Definition::quest_item)
            .any(|item| !required.contains(item))
        {
            return Err("Quest-only item has no possession objective");
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    CatalogMismatch,
    CreatureMismatch,
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
    /// Independent percent chance when group is zero; otherwise an exclusive
    /// interval in the creature's group. One basis point is 0.01 percent.
    pub fn chance_basis_points(&self) -> u16 {
        self.chance_basis_points
    }
    pub fn drop_group(&self) -> u8 {
        self.drop_group
    }
    pub fn requires_roll(&self) -> bool {
        self.roll_policy
    }
    /// The one conditional grant item, not proof of acceptance or entitlement.
    /// Chance classification stays independent of quest/inventory observations.
    pub fn quest_item(&self) -> Option<&ContentId> {
        self.quest_required.then(|| self.grants[0].item())
    }
    /// Pure classification of a trusted uniform ticket in 0..10_000. It creates
    /// no authority, entropy or durable receipt. Groups share ONE stored ticket;
    /// sorted stable IDs define disjoint intervals and unused chance means no loot.
    pub fn wins_roll(&self, catalog: &Catalog, ticket: u16) -> bool {
        if ticket >= 10_000 || catalog.definition(&self.id) != Some(self) {
            return false;
        }
        let lower: u32 = if self.drop_group == 0 {
            0
        } else {
            catalog
                .definitions()
                .filter(|d| {
                    d.creature == self.creature && d.drop_group == self.drop_group && d.id < self.id
                })
                .map(|d| u32::from(d.chance_basis_points))
                .sum()
        };
        (lower..lower + u32::from(self.chance_basis_points)).contains(&u32::from(ticket))
    }
    pub fn scope(&self) -> Scope {
        self.scope
    }
    pub fn ground(&self) -> Option<&Ground> {
        self.ground.as_ref()
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn grants(&self) -> &[Grant] {
        &self.grants
    }
    /// Check only the configured creature selector and scope. The authoritative
    /// caller must establish the fatal event and recipient before creating an
    /// offer; storage must bind that character and deduplicate this occurrence.
    pub fn occurrence(&self, event: &crate::progression::KillEvent) -> Result<Occurrence, Error> {
        if self.ground.is_some() || event.creature != self.creature {
            return Err(Error::CreatureMismatch);
        }
        Ok(match self.scope {
            Scope::OncePerCharacter => Occurrence::OncePerCharacter,
            Scope::PerCreatureLife => Occurrence::CreatureLife(event.id),
        })
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
        propose_grants(&self.grants, bag, items, expected_revision)
    }
}

/// Shared bounded fixed grants; caller owns recipe binding and entitlement.
pub(crate) fn propose_grants(
    grants: &[Grant],
    bag: &Bag,
    items: &inventory::Catalog,
    expected_revision: u64,
) -> Result<Bag, Error> {
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
    for grant in grants {
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
                *slot =
                    Some(Stack::new(items, grant.item.clone(), amount).ok_or(Error::InvalidGrant)?);
                remaining -= amount;
            }
        }
        if remaining > 0 {
            return Err(Error::NoCapacity);
        }
    }
    Bag::from_parts_with_copper(
        items,
        revision,
        slots,
        bag.main_hand().cloned(),
        bag.copper(),
    )
    .map_err(Error::ItemState)
}
