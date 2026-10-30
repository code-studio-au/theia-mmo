//! Exact item/vendor byte binding and stable latest-purchase identity. Runtime
//! must establish current NPC/range/actor authority before calling this boundary.
use crate::{Error, InventoryCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::vendors::{Catalog, Purchase};
#[derive(Clone, Debug)]
pub struct VendorCatalog {
    catalog: Arc<Catalog>,
    items: InventoryCatalog,
    hash: [u8; 32],
}
impl VendorCatalog {
    pub fn from_json(bytes: &[u8], items: &InventoryCatalog) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes, items.catalog()).map_err(Error::Invalid)?);
        let mut bound = theia_core::vendors::PROFILE_DOMAIN.to_vec();
        bound.extend(items.hash());
        bound.extend(bytes);
        Ok(Self {
            catalog,
            items: items.clone(),
            hash: digest(&SHA256, &bound).as_ref().try_into().unwrap(),
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn items(&self) -> &InventoryCatalog {
        &self.items
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub(crate) fn command_hash(&self, purchase: &Purchase) -> [u8; 32] {
        let mut bytes = b"theia-vendor-purchase-v1\0".to_vec();
        bytes.extend(self.hash);
        bytes.extend(purchase.expected_revision.to_le_bytes());
        bytes.push(purchase.offer.as_str().len() as u8);
        bytes.extend(purchase.offer.as_str().as_bytes());
        digest(&SHA256, &bytes).as_ref().try_into().unwrap()
    }
}
