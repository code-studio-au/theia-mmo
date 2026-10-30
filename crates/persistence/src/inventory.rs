//! Immutable byte-bound item configuration and committed inventory responses.
use crate::{Error, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::inventory::{Bag, Catalog, Command, EquipmentOperation, ItemCommand, Operation};
#[derive(Clone, Debug)]
pub struct InventoryCatalog {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
}
impl InventoryCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            catalog: Arc::new(Catalog::from_json(bytes).map_err(Error::Invalid)?),
            hash: digest(&SHA256, bytes).as_ref().try_into().unwrap(),
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryApplied {
    pub bag: Bag,
    pub replayed: bool,
}
/// Exact versioned intent fingerprint. Neither unordered JSON nor a client
/// supplied contents snapshot determines durable operation identity.
pub(crate) fn command_hash(command: Command) -> [u8; 32] {
    let mut bytes = b"theia-backpack-command-v1\0".to_vec();
    bytes.extend(command.expected_revision.to_le_bytes());
    match command.operation {
        Operation::Move { from, to } => bytes.extend([0, from, to]),
        Operation::Split { from, to, amount } => {
            bytes.extend([1, from, to]);
            bytes.extend(amount.to_le_bytes());
        }
    }
    digest(&SHA256, &bytes).as_ref().try_into().unwrap()
}
/// Both kinds mutate the same serialized item state and revision/receipt row.
pub(crate) fn intent_hash(command: ItemCommand) -> crate::Result<[u8; 32]> {
    Ok(match command {
        ItemCommand::Backpack(c) => command_hash(c),
        ItemCommand::Equipment(c) => {
            let mut bytes = b"theia-equipment-command-v1\0".to_vec();
            bytes.extend(c.expected_revision.to_le_bytes());
            match c.operation {
                EquipmentOperation::Equip { from } => bytes.extend([0, from]),
                EquipmentOperation::Unequip { to } => bytes.extend([1, to]),
            }
            digest(&SHA256, &bytes).as_ref().try_into().unwrap()
        }
        ItemCommand::RewardClaim(c) => crate::rewards::command_hash(c.claim, c.expected_revision),
        ItemCommand::Vendor(_) => {
            return Err(crate::Error::InventoryCommand(
                theia_core::inventory::Error::VendorAuthorityRequired,
            ));
        }
    })
}
