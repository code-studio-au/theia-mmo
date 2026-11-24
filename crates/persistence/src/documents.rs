//! Exact known-item document content and validated original open intents.
use crate::{Error, InventoryCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
/// An original intent, not authority. The selected profile and saved slot must
/// still match under the authorized transaction's backpack lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentOpenCommand {
    pub request_id: std::num::NonZeroU128,
    pub profile_hash: [u8; 32],
    pub document: theia_core::content::ContentId,
    pub expected_revision: u64,
    pub slot: u8,
}
#[derive(Clone, Debug)]
pub struct DocumentCatalog {
    catalog: Arc<theia_core::documents::Catalog>,
    hash: [u8; 32],
    inventory: InventoryCatalog,
}
impl DocumentCatalog {
    pub fn from_json(bytes: &[u8], inventory: &InventoryCatalog) -> Result<Self> {
        let catalog = Arc::new(
            theia_core::documents::Catalog::from_json(bytes, inventory.catalog())
                .map_err(Error::Invalid)?,
        );
        let mut bound = theia_core::documents::PROFILE_DOMAIN.to_vec();
        bound.extend(inventory.hash());
        bound.extend(bytes);
        Ok(Self {
            catalog,
            hash: digest(&SHA256, &bound).as_ref().try_into().unwrap(),
            inventory: inventory.clone(),
        })
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn inventory(&self) -> &InventoryCatalog {
        &self.inventory
    }
    pub fn catalog(&self) -> &theia_core::documents::Catalog {
        &self.catalog
    }
    pub(crate) fn validate_open(&self, command: &DocumentOpenCommand) -> Result<()> {
        use theia_core::documents::OpenError;
        if command.profile_hash != self.hash() {
            return Err(Error::DocumentProfileMismatch);
        }
        if command.expected_revision > theia_core::inventory::MAX_REVISION {
            return Err(Error::DocumentCommand(OpenError::StaleRevision));
        }
        if usize::from(command.slot) >= theia_core::inventory::SLOT_COUNT {
            return Err(Error::DocumentCommand(OpenError::InvalidSlot));
        }
        if self.catalog.document(&command.document).is_none() {
            return Err(Error::DocumentCommand(OpenError::UnknownDocument));
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod fixtures;
