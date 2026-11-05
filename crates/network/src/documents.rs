//! Exact document identity and structural intent resolution; no open authority.
use crate::{Result, renet_runtime::inventory::Profile as Items};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct Profile {
    catalog: Arc<theia_core::documents::Catalog>,
    hash: [u8; 32],
    inventory: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8], items: &Items) -> Result<Self> {
        let catalog = Arc::new(theia_core::documents::Catalog::from_json(
            bytes,
            items.catalog(),
        )?);
        let mut bound = theia_core::documents::PROFILE_DOMAIN.to_vec();
        bound.extend(items.hash());
        bound.extend(bytes);
        Ok(Self {
            catalog,
            hash: crate::terrain::sha256(&bound),
            inventory: items.hash(),
        })
    }
    pub fn resolve(
        &self,
        command: theia_core::documents::OpenCommand,
    ) -> Result<&theia_core::documents::Document> {
        if command.expected_revision > theia_core::inventory::MAX_REVISION
            || usize::from(command.slot) >= theia_core::inventory::SLOT_COUNT
        {
            return Err("Invalid document slot or revision".into());
        }
        self.catalog
            .documents()
            .nth(usize::from(command.document))
            .ok_or_else(|| "Unknown document ordinal".into())
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn inventory_hash(&self) -> [u8; 32] {
        self.inventory
    }
    pub fn catalog(&self) -> &theia_core::documents::Catalog {
        &self.catalog
    }
}
