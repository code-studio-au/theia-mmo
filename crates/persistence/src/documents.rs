//! Exact known-item document content; no saved open or read receipt is installed.
use crate::{Error, InventoryCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
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
}

#[cfg(test)]
pub(crate) mod fixtures;
