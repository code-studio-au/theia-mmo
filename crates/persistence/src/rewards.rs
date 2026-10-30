//! Immutable, byte-bound reward profile. Offers/claims are trusted authority
//! work; a recipe does not establish a kill or authorize its recipient.
use crate::{ClaimId, Error, InventoryCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::{content::ContentId, rewards::Catalog};
#[derive(Clone, Debug)]
pub struct RewardCatalog {
    catalog: Arc<Catalog>,
    items: InventoryCatalog,
    hash: [u8; 32],
}
impl RewardCatalog {
    pub fn from_json(bytes: &[u8], items: &InventoryCatalog) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes, items.catalog()).map_err(Error::Invalid)?);
        let mut bound = theia_core::rewards::PROFILE_DOMAIN.to_vec();
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
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardClaim {
    pub id: ClaimId,
    pub reward: ContentId,
    pub consumed_revision: Option<u64>,
}
pub(crate) fn command_hash(claim: ClaimId, expected_revision: u64) -> [u8; 32] {
    let mut bytes = theia_core::rewards::CLAIM_DOMAIN.to_vec();
    bytes.extend(
        theia_core::rewards::ClaimCommand {
            claim,
            expected_revision,
        }
        .canonical_bytes(),
    );
    digest(&SHA256, &bytes).as_ref().try_into().unwrap()
}
