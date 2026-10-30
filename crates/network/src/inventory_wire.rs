//! Inventory profile 1. Structural validation only: the runtime must bind actor
//! keys to its admitted peer and the store must acknowledge durable outcomes.
use crate::Result;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{
        Bag, Catalog, Command, EquipmentCommand, EquipmentOperation, ItemCommand, MAX_REVISION,
        MAX_STACK, Operation, SLOT_COUNT, Stack,
    },
};
pub const REQUEST_TAG: &[u8; 4] = b"IR01";
pub const STATE_TAG: &[u8; 4] = b"IS01";
pub const REQUEST_BYTES: usize = 43;
pub const STATE_BYTES: usize = 117;
pub const ITEM_REQUEST_TAG: &[u8; 4] = b"IR02";
pub const ITEM_STATE_TAG: &[u8; 4] = b"IS02";
pub const ITEM_STATE_BYTES: usize = STATE_BYTES + 3;
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn key(bytes: &[u8]) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(word(bytes, 4)).ok_or("Invalid inventory epoch")?,
        entity: EntityId::new(word(bytes, 12)).ok_or("Invalid inventory actor")?,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: Command,
}
impl Request {
    fn valid(self) -> bool {
        let (from, to, amount) = match self.command.operation {
            Operation::Move { from, to } => (from, to, 0),
            Operation::Split { from, to, amount } => (from, to, amount),
        };
        self.sequence > 0
            && self.command.expected_revision <= MAX_REVISION
            && usize::from(from) < SLOT_COUNT
            && usize::from(to) < SLOT_COUNT
            && amount <= MAX_STACK
    }
    pub fn encode(self) -> Result<Vec<u8>> {
        if !self.valid() {
            return Err("Invalid inventory request fields".into());
        }
        let mut bytes = REQUEST_TAG.to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.command.expected_revision.to_le_bytes());
        match self.command.operation {
            Operation::Move { from, to } => {
                bytes.extend([0, from, to]);
                bytes.extend(0_u32.to_le_bytes());
            }
            Operation::Split { from, to, amount } => {
                bytes.extend([1, from, to]);
                bytes.extend(amount.to_le_bytes());
            }
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid inventory request frame".into());
        }
        let from = bytes[37];
        let to = bytes[38];
        let amount = u32::from_le_bytes(bytes[39..43].try_into().unwrap());
        let operation = match bytes[36] {
            0 if amount == 0 => Operation::Move { from, to },
            1 => Operation::Split { from, to, amount },
            _ => return Err("Invalid inventory operation ordinal".into()),
        };
        let value = Self {
            actor: key(bytes)?,
            sequence: word(bytes, 20),
            command: Command {
                expected_revision: word(bytes, 28),
                operation,
            },
        };
        if !value.valid() {
            return Err("Invalid inventory request fields".into());
        }
        Ok(value)
    }
}
/// Profile 2 carries ordinary backpack and equipment intents in one ordered
/// stream. Unused fields have one canonical representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemRequest {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: ItemCommand,
}
impl ItemRequest {
    pub fn encode(self) -> Result<Vec<u8>> {
        if let ItemCommand::Backpack(command) = self.command {
            let mut bytes = Request {
                actor: self.actor,
                sequence: self.sequence,
                command,
            }
            .encode()?;
            bytes[..4].copy_from_slice(ITEM_REQUEST_TAG);
            return Ok(bytes);
        }
        let ItemCommand::Equipment(command) = self.command else {
            return Err("Reward claims require their explicit request profile".into());
        };
        let (code, from, to) = match command.operation {
            EquipmentOperation::Equip { from } => (2, from, 0),
            EquipmentOperation::Unequip { to } => (3, 0, to),
        };
        if self.sequence == 0
            || command.expected_revision > MAX_REVISION
            || usize::from(from) >= SLOT_COUNT
            || usize::from(to) >= SLOT_COUNT
        {
            return Err("Invalid equipment request fields".into());
        }
        let mut bytes = ITEM_REQUEST_TAG.to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(command.expected_revision.to_le_bytes());
        bytes.extend([code, from, to]);
        bytes.extend(0_u32.to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(ITEM_REQUEST_TAG) {
            return Err("Invalid equipment-capable request frame".into());
        }
        if bytes[36] <= 1 {
            let mut legacy = bytes.to_vec();
            legacy[..4].copy_from_slice(REQUEST_TAG);
            let old = Request::decode(&legacy)?;
            return Ok(Self {
                actor: old.actor,
                sequence: old.sequence,
                command: old.command.into(),
            });
        }
        let operation = match bytes[36] {
            2 if bytes[38] == 0 => EquipmentOperation::Equip { from: bytes[37] },
            3 if bytes[37] == 0 => EquipmentOperation::Unequip { to: bytes[38] },
            _ => return Err("Invalid equipment operation or reserved field".into()),
        };
        if bytes[39..43] != [0; 4] {
            return Err("Nonzero equipment reserved amount".into());
        }
        let request = Self {
            actor: key(bytes)?,
            sequence: word(bytes, 20),
            command: EquipmentCommand {
                expected_revision: word(bytes, 28),
                operation,
            }
            .into(),
        };
        request.encode()?;
        Ok(request)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    Stale = 0,
    Conflict = 1,
    SameSlot = 2,
    EmptySource = 3,
    Full = 4,
    Occupied = 5,
    Amount = 6,
    Exhausted = 7,
    Busy = 8,
    Slot = 9,
    EquipmentDisabled = 10,
    NotEquippable = 11,
    EmptyHand = 12,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Applied,
    Replayed,
    Rejected(Reason),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub sequence: u64,
    pub outcome: Outcome,
}
fn outcome(code: u8, equipment: bool) -> Result<Outcome> {
    Ok(match code {
        1 => Outcome::Applied,
        2 => Outcome::Replayed,
        3 => Outcome::Rejected(Reason::Stale),
        4 => Outcome::Rejected(Reason::Conflict),
        5 => Outcome::Rejected(Reason::SameSlot),
        6 => Outcome::Rejected(Reason::EmptySource),
        7 => Outcome::Rejected(Reason::Full),
        8 => Outcome::Rejected(Reason::Occupied),
        9 => Outcome::Rejected(Reason::Amount),
        10 => Outcome::Rejected(Reason::Exhausted),
        11 => Outcome::Rejected(Reason::Busy),
        12 => Outcome::Rejected(Reason::Slot),
        13 if equipment => Outcome::Rejected(Reason::EquipmentDisabled),
        14 if equipment => Outcome::Rejected(Reason::NotEquippable),
        15 if equipment => Outcome::Rejected(Reason::EmptyHand),
        _ => return Err("Invalid inventory reply ordinal".into()),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub actor: EntityKey,
    pub catalog_hash: [u8; 32],
    pub bag: Bag,
    pub reply: Option<Reply>,
}
impl State {
    pub fn encode(&self, catalog: &Catalog, expected_hash: [u8; 32]) -> Result<Vec<u8>> {
        if catalog.equipment_enabled() != self.bag.equipment_enabled()
            || expected_hash == [0; 32]
            || self.catalog_hash != expected_hash
            || self.bag.catalog() != catalog.id()
            || self.bag.revision() > MAX_REVISION
            || (self.bag.revision() == 0
                && self
                    .reply
                    .is_some_and(|r| matches!(r.outcome, Outcome::Applied | Outcome::Replayed)))
        {
            return Err("Inventory state/catalog mismatch".into());
        }
        if let Some(hand) = self.bag.main_hand()
            && (hand.count() != 1
                || catalog
                    .definition(hand.item())
                    .is_none_or(|d| d.main_hand_appearance().is_none()))
        {
            return Err("Invalid equipped item".into());
        }
        let mut bytes = if catalog.equipment_enabled() {
            ITEM_STATE_TAG
        } else {
            STATE_TAG
        }
        .to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.catalog_hash);
        bytes.extend(self.bag.revision().to_le_bytes());
        bytes.extend(self.reply.map_or(0, |r| r.sequence).to_le_bytes());
        bytes.push(match self.reply {
            Some(reply) if reply.sequence == 0 => {
                return Err("Invalid inventory reply sequence".into());
            }
            Some(Reply {
                outcome: Outcome::Applied,
                ..
            }) => 1,
            Some(Reply {
                outcome: Outcome::Replayed,
                ..
            }) => 2,
            Some(Reply {
                outcome: Outcome::Rejected(reason),
                ..
            }) => {
                if !catalog.equipment_enabled() && reason as u8 > Reason::Slot as u8 {
                    return Err("Equipment reason requires profile 2".into());
                }
                3 + reason as u8
            }
            None => 0,
        });
        let definitions: Vec<_> = catalog.definitions().collect();
        let mut slots: Vec<_> = self.bag.slots().iter().map(Option::as_ref).collect();
        if catalog.equipment_enabled() {
            slots.push(self.bag.main_hand());
        }
        for slot in slots {
            if let Some(stack) = slot {
                let ordinal = definitions
                    .iter()
                    .position(|d| d.id() == stack.item())
                    .ok_or("Unknown inventory item")?;
                if stack.count() == 0 || stack.count() > definitions[ordinal].maximum_stack() {
                    return Err("Inventory stack exceeds checked definition".into());
                }
                bytes.push(ordinal as u8);
                bytes.extend((stack.count() as u16).to_le_bytes());
            } else {
                bytes.push(255);
                bytes.extend(0_u16.to_le_bytes());
            }
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog, expected_hash: [u8; 32]) -> Result<Self> {
        let (tag, size) = if catalog.equipment_enabled() {
            (ITEM_STATE_TAG, ITEM_STATE_BYTES)
        } else {
            (STATE_TAG, STATE_BYTES)
        };
        if bytes.len() != size || !bytes.starts_with(tag) {
            return Err("Invalid inventory state frame".into());
        }
        let actor = key(bytes)?;
        let catalog_hash: [u8; 32] = bytes[20..52].try_into().unwrap();
        if expected_hash == [0; 32] || catalog_hash != expected_hash {
            return Err("Inventory catalog byte identity mismatch".into());
        }
        let sequence = word(bytes, 60);
        let reply = match (sequence, bytes[68]) {
            (0, 0) => None,
            (0, _) | (_, 0) => return Err("Invalid inventory reply sequence/flag".into()),
            (sequence, code) => Some(Reply {
                sequence,
                outcome: outcome(code, catalog.equipment_enabled())?,
            }),
        };
        let definitions: Vec<_> = catalog.definitions().collect();
        let mut slots = std::array::from_fn(|_| None);
        let mut main_hand = None;
        for (i, data) in bytes[69..].as_chunks::<3>().0.iter().enumerate() {
            let ordinal = data[0];
            let count = u32::from(u16::from_le_bytes([data[1], data[2]]));
            if ordinal == 255 && count == 0 {
                continue;
            }
            let definition = definitions
                .get(usize::from(ordinal))
                .ok_or("Unknown inventory item ordinal")?;
            let stack = Some(
                Stack::new(catalog, definition.id().clone(), count)
                    .ok_or("Invalid inventory stack count")?,
            );
            if i < SLOT_COUNT {
                slots[i] = stack;
            } else {
                main_hand = stack;
            }
        }
        let bag = Bag::from_parts(catalog, word(bytes, 52), slots, main_hand)
            .map_err(|_| "Invalid inventory state revision/contents")?;
        if bag.revision() == 0
            && reply.is_some_and(|r| matches!(r.outcome, Outcome::Applied | Outcome::Replayed))
        {
            return Err("Inventory success requires a committed revision".into());
        }
        Ok(Self {
            actor,
            catalog_hash,
            bag,
            reply,
        })
    }
}
