//! Checked backpack proposals. A successful proposal is not a durable commit or
//! permission to acknowledge success; the asynchronous authority must commit it.
use crate::content::ContentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SLOT_COUNT: usize = 16;
pub const MAX_DEFINITIONS: usize = 64;
pub const MAX_STACK: u32 = 10_000;
pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const MAX_BAG_BYTES: usize = 4096;
/// Fits PostgreSQL BIGINT without lossy casts. Exhaustion must fail closed.
pub const MAX_REVISION: u64 = i64::MAX as u64;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    items: Vec<RawDefinition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefinition {
    id: String,
    name: String,
    maximum_stack: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEquipmentCatalog {
    schema: u32,
    id: String,
    items: Vec<RawEquipmentDefinition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEquipmentDefinition {
    id: String,
    name: String,
    maximum_stack: u32,
    main_hand_appearance: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    id: ContentId,
    name: String,
    maximum_stack: u32,
    main_hand_appearance: Option<ContentId>,
}
impl Definition {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn maximum_stack(&self) -> u32 {
        self.maximum_stack
    }
    pub fn main_hand_appearance(&self) -> Option<&ContentId> {
        self.main_hand_appearance.as_ref()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    items: BTreeMap<ContentId, Definition>,
    equipment_enabled: bool,
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Item catalog exceeds byte budget");
        }
        #[derive(Deserialize)]
        struct Version {
            schema: u32,
        }
        let version: Version = serde_json::from_slice(bytes).map_err(|_| "Invalid item catalog")?;
        let (name, definitions, equipment_enabled) = match version.schema {
            1 => {
                let raw: RawCatalog =
                    serde_json::from_slice(bytes).map_err(|_| "Invalid item catalog")?;
                debug_assert_eq!(raw.schema, 1);
                (
                    raw.id,
                    raw.items.into_iter().map(|d| (d, None)).collect::<Vec<_>>(),
                    false,
                )
            }
            2 => {
                let raw: RawEquipmentCatalog =
                    serde_json::from_slice(bytes).map_err(|_| "Invalid equipment item catalog")?;
                debug_assert_eq!(raw.schema, 2);
                (
                    raw.id,
                    raw.items
                        .into_iter()
                        .map(|d| {
                            (
                                RawDefinition {
                                    id: d.id,
                                    name: d.name,
                                    maximum_stack: d.maximum_stack,
                                },
                                d.main_hand_appearance,
                            )
                        })
                        .collect(),
                    true,
                )
            }
            _ => return Err("Unsupported item catalog schema or count"),
        };
        if !(1..=MAX_DEFINITIONS).contains(&definitions.len()) {
            return Err("Unsupported item catalog schema or count");
        }
        let id = ContentId::new(&name).ok_or("Invalid item catalog ID")?;
        let mut items = BTreeMap::new();
        for (item, appearance) in definitions {
            let id = ContentId::new(&item.id).ok_or("Invalid item ID")?;
            if !(1..=MAX_STACK).contains(&item.maximum_stack)
                || !(1..=48).contains(&item.name.len())
                || item.name.trim() != item.name
                || !item
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b" '-".contains(&c))
                || !item.name.bytes().any(|c| c.is_ascii_alphanumeric())
            {
                return Err("Invalid development item name or stack limit");
            }
            let definition = Definition {
                id: id.clone(),
                name: item.name,
                maximum_stack: item.maximum_stack,
                main_hand_appearance: match appearance {
                    Some(name) if item.maximum_stack == 1 => {
                        Some(ContentId::new(&name).ok_or("Invalid main-hand appearance ID")?)
                    }
                    Some(_) => return Err("Equippable items must be nonstackable"),
                    None => None,
                },
            };
            if items.insert(id, definition).is_some() {
                return Err("Duplicate item ID");
            }
        }
        if equipment_enabled && !items.values().any(|d| d.main_hand_appearance.is_some()) {
            return Err("Equipment catalog requires at least one equippable item");
        }
        Ok(Self {
            id,
            items,
            equipment_enabled,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn definition(&self, id: &ContentId) -> Option<&Definition> {
        self.items.get(id)
    }
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &Definition> {
        self.items.values()
    }
    pub fn equipment_enabled(&self) -> bool {
        self.equipment_enabled
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack {
    item: ContentId,
    count: u32,
}
impl Stack {
    pub fn new(catalog: &Catalog, item: ContentId, count: u32) -> Option<Self> {
        let definition = catalog.definition(&item)?;
        (count > 0 && count <= definition.maximum_stack).then_some(Self { item, count })
    }
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bag {
    catalog: ContentId,
    revision: u64,
    slots: [Option<Stack>; SLOT_COUNT],
    equipment_enabled: bool,
    main_hand: Option<Stack>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidState,
    CatalogMismatch,
    StaleRevision,
    RevisionExhausted,
    InvalidSlot,
    SameSlot,
    EmptySource,
    DestinationFull,
    DestinationOccupied,
    InvalidAmount,
    EquipmentDisabled,
    NotEquippable,
    EmptyHand,
    ClaimAuthorityRequired,
    VendorAuthorityRequired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Move into an empty slot, merge up to the limit, or swap different items.
    Move { from: u8, to: u8 },
    /// Move a strictly partial, positive amount into an empty slot.
    Split { from: u8, to: u8, amount: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub expected_revision: u64,
    pub operation: Operation,
}
/// Separate from IR01 backpack commands. Network admission must explicitly
/// support the equipment profile before exposing these proposals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentOperation {
    Equip { from: u8 },
    Unequip { to: u8 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EquipmentCommand {
    pub expected_revision: u64,
    pub operation: EquipmentOperation,
}
/// One ordered stream and revision covers all transfers of owned items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemCommand {
    Backpack(Command),
    Equipment(EquipmentCommand),
    RewardClaim(crate::rewards::ClaimCommand),
    Vendor(crate::vendors::Command),
}
impl From<Command> for ItemCommand {
    fn from(value: Command) -> Self {
        Self::Backpack(value)
    }
}
impl From<EquipmentCommand> for ItemCommand {
    fn from(value: EquipmentCommand) -> Self {
        Self::Equipment(value)
    }
}
impl From<crate::rewards::ClaimCommand> for ItemCommand {
    fn from(value: crate::rewards::ClaimCommand) -> Self {
        Self::RewardClaim(value)
    }
}
impl From<crate::vendors::Command> for ItemCommand {
    fn from(value: crate::vendors::Command) -> Self {
        Self::Vendor(value)
    }
}
impl ItemCommand {
    pub fn expected_revision(self) -> u64 {
        match self {
            Self::Backpack(c) => c.expected_revision,
            Self::Equipment(c) => c.expected_revision,
            Self::RewardClaim(c) => c.expected_revision,
            Self::Vendor(c) => c.expected_revision,
        }
    }
    pub fn propose(self, bag: &Bag, catalog: &Catalog) -> Result<Bag, Error> {
        match self {
            Self::Backpack(c) => bag.propose(catalog, c),
            Self::Equipment(c) => bag.propose_equipment(catalog, c),
            Self::RewardClaim(_) => Err(Error::ClaimAuthorityRequired),
            Self::Vendor(_) => Err(Error::VendorAuthorityRequired),
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawStack {
    item: String,
    count: u32,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawBag {
    schema: u32,
    catalog: String,
    revision: u64,
    slots: [Option<RawStack>; SLOT_COUNT],
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawEquipmentBag {
    schema: u32,
    catalog: String,
    revision: u64,
    slots: [Option<RawStack>; SLOT_COUNT],
    main_hand: RawHand,
}
/// Required field, nullable value. An absent Option field would deserialize as
/// None and could silently erase equipped contents from incomplete stored data.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum RawHand {
    Empty(()),
    Item(RawStack),
}
impl Bag {
    pub fn empty(catalog: &Catalog) -> Self {
        Self {
            catalog: catalog.id.clone(),
            revision: 0,
            slots: std::array::from_fn(|_| None),
            equipment_enabled: catalog.equipment_enabled,
            main_hand: None,
        }
    }
    pub fn from_slots(
        catalog: &Catalog,
        revision: u64,
        slots: [Option<Stack>; SLOT_COUNT],
    ) -> Result<Self, Error> {
        Self::from_parts(catalog, revision, slots, None)
    }
    pub fn from_parts(
        catalog: &Catalog,
        revision: u64,
        slots: [Option<Stack>; SLOT_COUNT],
        main_hand: Option<Stack>,
    ) -> Result<Self, Error> {
        let bag = Self {
            catalog: catalog.id.clone(),
            revision,
            slots,
            equipment_enabled: catalog.equipment_enabled,
            main_hand,
        };
        bag.validate(catalog)?;
        Ok(bag)
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn catalog(&self) -> &ContentId {
        &self.catalog
    }
    pub fn slots(&self) -> &[Option<Stack>; SLOT_COUNT] {
        &self.slots
    }
    pub fn main_hand(&self) -> Option<&Stack> {
        self.main_hand.as_ref()
    }
    pub fn equipment_enabled(&self) -> bool {
        self.equipment_enabled
    }
    pub(crate) fn validate(&self, catalog: &Catalog) -> Result<(), Error> {
        if self.catalog != catalog.id || self.equipment_enabled != catalog.equipment_enabled {
            return Err(Error::CatalogMismatch);
        }
        if self.revision > MAX_REVISION
            || self.slots.iter().flatten().any(|s| {
                catalog
                    .definition(&s.item)
                    .is_none_or(|d| s.count == 0 || s.count > d.maximum_stack)
            })
        {
            return Err(Error::InvalidState);
        }
        if let Some(item) = &self.main_hand
            && (!self.equipment_enabled
                || item.count != 1
                || catalog
                    .definition(&item.item)
                    .is_none_or(|d| d.main_hand_appearance.is_none() || d.maximum_stack != 1))
        {
            return Err(Error::InvalidState);
        }
        Ok(())
    }
    /// Atomic pure transfer using the same revision as backpack changes. A
    /// proposal is not a commit; publish it only after the durable boundary.
    pub fn propose_equipment(
        &self,
        catalog: &Catalog,
        command: EquipmentCommand,
    ) -> Result<Self, Error> {
        self.validate(catalog)?;
        if !self.equipment_enabled {
            return Err(Error::EquipmentDisabled);
        }
        if command.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let revision = self
            .revision
            .checked_add(1)
            .filter(|r| *r <= MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let mut next = self.clone();
        match command.operation {
            EquipmentOperation::Equip { from } => {
                let source = next
                    .slots
                    .get_mut(usize::from(from))
                    .ok_or(Error::InvalidSlot)?;
                let item = source.as_ref().ok_or(Error::EmptySource)?;
                if catalog
                    .definition(&item.item)
                    .is_none_or(|d| d.main_hand_appearance.is_none())
                {
                    return Err(Error::NotEquippable);
                }
                std::mem::swap(source, &mut next.main_hand);
            }
            EquipmentOperation::Unequip { to } => {
                let destination = next
                    .slots
                    .get_mut(usize::from(to))
                    .ok_or(Error::InvalidSlot)?;
                if next.main_hand.is_none() {
                    return Err(Error::EmptyHand);
                }
                if destination.is_some() {
                    return Err(Error::DestinationOccupied);
                }
                *destination = next.main_hand.take();
            }
        }
        next.revision = revision;
        next.validate(catalog)?;
        Ok(next)
    }
    /// Pure tentative result. Does not mutate published state, perform I/O,
    /// authorize a character, deduplicate durable requests or commit anything.
    pub fn propose(&self, catalog: &Catalog, command: Command) -> Result<Self, Error> {
        self.validate(catalog)?;
        if command.expected_revision != self.revision {
            return Err(Error::StaleRevision);
        }
        let revision = self
            .revision
            .checked_add(1)
            .filter(|r| *r <= MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let (from, to) = match command.operation {
            Operation::Move { from, to } | Operation::Split { from, to, .. } => {
                (usize::from(from), usize::from(to))
            }
        };
        if from >= SLOT_COUNT || to >= SLOT_COUNT {
            return Err(Error::InvalidSlot);
        }
        if from == to {
            return Err(Error::SameSlot);
        }
        let source = self.slots[from].as_ref().ok_or(Error::EmptySource)?;
        let mut next = self.clone();
        match command.operation {
            Operation::Move { .. } => match &self.slots[to] {
                Some(destination) if destination.item == source.item => {
                    let limit = catalog.definition(&source.item).unwrap().maximum_stack;
                    let amount = source.count.min(limit - destination.count);
                    if amount == 0 {
                        return Err(Error::DestinationFull);
                    }
                    next.slots[to].as_mut().unwrap().count += amount;
                    next.slots[from] = (source.count > amount).then(|| Stack {
                        item: source.item.clone(),
                        count: source.count - amount,
                    });
                }
                _ => next.slots.swap(from, to),
            },
            Operation::Split { amount, .. } => {
                if amount == 0 || amount >= source.count {
                    return Err(Error::InvalidAmount);
                }
                if self.slots[to].is_some() {
                    return Err(Error::DestinationOccupied);
                }
                next.slots[from].as_mut().unwrap().count -= amount;
                next.slots[to] = Some(Stack {
                    item: source.item.clone(),
                    count: amount,
                });
            }
        }
        next.revision = revision;
        Ok(next)
    }
    pub fn from_json(catalog: &Catalog, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_BAG_BYTES {
            return Err(Error::InvalidState);
        }
        let (name, revision, raw_slots, raw_hand) = if catalog.equipment_enabled {
            let raw: RawEquipmentBag =
                serde_json::from_slice(bytes).map_err(|_| Error::InvalidState)?;
            if raw.schema != 2 {
                return Err(Error::InvalidState);
            }
            (
                raw.catalog,
                raw.revision,
                raw.slots,
                match raw.main_hand {
                    RawHand::Empty(()) => None,
                    RawHand::Item(item) => Some(item),
                },
            )
        } else {
            let raw: RawBag = serde_json::from_slice(bytes).map_err(|_| Error::InvalidState)?;
            if raw.schema != 1 {
                return Err(Error::InvalidState);
            }
            (raw.catalog, raw.revision, raw.slots, None)
        };
        if name != catalog.id.as_str() {
            return Err(Error::CatalogMismatch);
        }
        let mut slots = std::array::from_fn(|_| None);
        for (slot, raw) in slots.iter_mut().zip(raw_slots) {
            if let Some(raw) = raw {
                *slot = Some(
                    ContentId::new(&raw.item)
                        .and_then(|id| Stack::new(catalog, id, raw.count))
                        .ok_or(Error::InvalidState)?,
                );
            }
        }
        let main_hand = raw_hand
            .map(|raw| {
                ContentId::new(&raw.item)
                    .and_then(|id| Stack::new(catalog, id, raw.count))
                    .ok_or(Error::InvalidState)
            })
            .transpose()?;
        Self::from_parts(catalog, revision, slots, main_hand)
    }
    /// The persistence envelope must separately bind the checked catalog's byte
    /// digest. A logical catalog ID alone does not prove identical definitions.
    pub fn to_json(&self) -> Vec<u8> {
        if self.equipment_enabled {
            let raw = RawEquipmentBag {
                schema: 2,
                catalog: self.catalog.as_str().to_owned(),
                revision: self.revision,
                slots: std::array::from_fn(|i| {
                    self.slots[i].as_ref().map(|s| RawStack {
                        item: s.item.as_str().to_owned(),
                        count: s.count,
                    })
                }),
                main_hand: self.main_hand.as_ref().map_or(RawHand::Empty(()), |s| {
                    RawHand::Item(RawStack {
                        item: s.item.as_str().to_owned(),
                        count: s.count,
                    })
                }),
            };
            return serde_json::to_vec(&raw).expect("Checked equipped backpack is serializable");
        }
        let raw = RawBag {
            schema: 1,
            catalog: self.catalog.as_str().to_owned(),
            revision: self.revision,
            slots: std::array::from_fn(|i| {
                self.slots[i].as_ref().map(|s| RawStack {
                    item: s.item.as_str().to_owned(),
                    count: s.count,
                })
            }),
        };
        // Every component is checked, bounded and JSON-serializable by construction.
        serde_json::to_vec(&raw).expect("Checked backpack is serializable")
    }
}
