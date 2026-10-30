//! One shared-row cost/output/receipt transaction. No schema or historical
//! purchase ledger is added; preserve the existing latest-item-receipt contract.
use super::{
    Store, bounded,
    inventory::{authorize, checked},
};
use crate::{Binding, Error, InventoryApplied, Result, VendorCatalog};
use theia_core::{inventory::MAX_REVISION, vendors::Purchase};
impl Store {
    /// Trusted live purchase adapter only. NPC/range entitlement belongs outside
    /// SQL. One item intent may be outstanding; exact latest retry is recoverable,
    /// conflicting/superseded revisions reject. Only COMMIT permits success.
    pub async fn purchase_vendor(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &VendorCatalog,
        purchase: &Purchase,
    ) -> Result<InventoryApplied> {
        if purchase.expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        if profile.catalog().offer(&purchase.offer).is_none() {
            return Err(Error::VendorCommand(
                theia_core::vendors::Error::UnknownOffer,
            ));
        }
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let row=bounded(transaction.query_opt("SELECT catalog_hash,revision,state,last_command FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&self.realm.value(),&binding.character.value()])).await?.ok_or(Error::InventoryMissing)?;
        let bag = checked(&row, profile.items())?;
        let hash = profile.command_hash(purchase);
        let receipt: Option<Vec<u8>> = row.get(3);
        if purchase.expected_revision.checked_add(1) == Some(bag.revision()) {
            if receipt.as_deref() != Some(hash.as_slice()) {
                return Err(Error::InventoryConflict);
            }
            bounded(transaction.commit()).await?;
            return Ok(InventoryApplied {
                bag,
                replayed: true,
            });
        }
        if purchase.expected_revision != bag.revision() {
            return Err(Error::InventoryStale);
        }
        let next = profile
            .catalog()
            .propose(&bag, profile.items().catalog(), purchase)
            .map_err(Error::VendorCommand)?;
        let bytes = next.to_json();
        let updated=bounded(transaction.execute("UPDATE theia.backpacks SET revision=$1,state=$2,last_command=$3,updated_at=clock_timestamp() WHERE realm_id=$4 AND character_id=$5 AND revision=$6",&[&(next.revision() as i64),&bytes,&hash.as_slice(),&self.realm.value(),&binding.character.value(),&(bag.revision() as i64)])).await?;
        if updated != 1 {
            return Err(Error::InventoryData);
        }
        bounded(transaction.commit()).await?;
        Ok(InventoryApplied {
            bag: next,
            replayed: false,
        })
    }
}
