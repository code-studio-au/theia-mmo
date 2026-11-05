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
    journal: Option<crate::QuestJournalCatalog>,
}
impl RewardCatalog {
    pub fn from_json(bytes: &[u8], items: &InventoryCatalog) -> Result<Self> {
        Self::parse(bytes, items, false)
    }
    /// Exact immutable binding only. Existing Store entry points remain closed.
    pub fn from_json_with_quest_journal(
        bytes: &[u8],
        items: &InventoryCatalog,
        journal: &crate::QuestJournalCatalog,
    ) -> Result<Self> {
        let mut profile = Self::parse(bytes, items, true)?;
        if !profile.catalog.has_quest_required()
            || journal.inventory().map(InventoryCatalog::hash) != Some(items.hash())
        {
            return Err(Error::Invalid(
                "Quest-only loot requires the exact inventory-bound journal",
            ));
        }
        profile
            .catalog
            .validate_quest_items(journal.catalog())
            .map_err(Error::Invalid)?;
        let mut bound = theia_core::rewards::QUEST_PROFILE_DOMAIN.to_vec();
        bound.extend(profile.hash);
        bound.extend(journal.hash());
        profile.hash = digest(&SHA256, &bound).as_ref().try_into().unwrap();
        profile.journal = Some(journal.clone());
        Ok(profile)
    }
    fn parse(bytes: &[u8], items: &InventoryCatalog, allow_quest: bool) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes, items.catalog()).map_err(Error::Invalid)?);
        if catalog.has_quest_required() && !allow_quest {
            return Err(Error::Invalid("Quest-only loot storage is not installed"));
        }
        let mut bound = theia_core::rewards::PROFILE_DOMAIN.to_vec();
        bound.extend(items.hash());
        bound.extend(bytes);
        Ok(Self {
            catalog,
            items: items.clone(),
            hash: digest(&SHA256, &bound).as_ref().try_into().unwrap(),
            journal: None,
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
    pub fn quest_journal(&self) -> Option<&crate::QuestJournalCatalog> {
        self.journal.as_ref()
    }
    pub(crate) fn check_runtime_admission(&self) -> Result<()> {
        if self.catalog.has_quest_required() {
            return Err(Error::Invalid("Quest-only loot storage is not installed"));
        }
        Ok(())
    }
}
pub use theia_core::rewards::CorpseSource;
/// Visible pending life page; historical consumed receipts stay in SQL.
pub const LIFE_CLAIM_PAGE_SIZE: usize = theia_core::rewards::MAX_VISIBLE_LIFE_CLAIMS;

#[derive(Clone, Debug, PartialEq)]
pub struct RewardClaim {
    pub id: ClaimId,
    pub reward: ContentId,
    pub consumed_revision: Option<u64>,
    pub corpse: Option<CorpseSource>,
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

#[cfg(test)]
mod tests {
    #[test]
    fn quest_only_loot_refuses_unbound_storage_profile_without_sql() {
        let items = crate::InventoryCatalog::from_json(include_bytes!(
            "../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-rewards.json"
        ))
        .unwrap();
        value["schema"] = serde_json::json!(4);
        value["rewards"][0]["scope"] = serde_json::json!("per_creature_life");
        value["rewards"][0]["quest_required"] = serde_json::json!(true);
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(
            theia_core::rewards::Catalog::from_json(&bytes, items.catalog())
                .unwrap()
                .has_quest_required()
        );
        assert!(super::RewardCatalog::from_json(&bytes, &items).is_err());
        value["rewards"][0]["quest_required"] = serde_json::json!(false);
        assert!(
            super::RewardCatalog::from_json(&serde_json::to_vec(&value).unwrap(), &items).is_ok()
        );
    }
}
