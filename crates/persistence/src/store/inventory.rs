//! Async durable item boundary; never called from a simulation step.
use super::{Store, bounded};
use crate::{
    Binding, Error, InventoryApplied, InventoryCatalog, RealmId, Result, inventory::intent_hash,
};
use theia_core::inventory::{Bag, Command, EquipmentCommand, ItemCommand as ItemIntent};
use tokio_postgres::{Row, Transaction};

pub(super) async fn authorize(
    transaction: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
) -> Result<()> {
    if world == [0; 32] {
        return Err(Error::Invalid("World content identity must be nonzero"));
    }
    let row = bounded(transaction.query_opt(
        "SELECT owner_id,world_id FROM theia.characters WHERE realm_id=$1 AND character_id=$2 FOR SHARE",
        &[&realm.value(), &binding.character.value()],
    )).await?.ok_or(Error::CharacterMissing)?;
    if row.get::<_, uuid::Uuid>(0) != binding.owner.value() {
        return Err(Error::Ownership);
    }
    if row.get::<_, Vec<u8>>(1) != world {
        return Err(Error::WorldMismatch);
    }
    Ok(())
}
pub(super) fn checked(row: &Row, profile: &InventoryCatalog) -> Result<Bag> {
    if row.get::<_, Vec<u8>>(0) != profile.hash() {
        return Err(Error::InventoryCatalogMismatch);
    }
    let revision = row.get::<_, i64>(1);
    let bytes: Vec<u8> = row.get(2);
    let bag = Bag::from_json(profile.catalog(), &bytes).map_err(|_| Error::InventoryData)?;
    if revision < 0 || bag.revision() != revision as u64 {
        return Err(Error::InventoryData);
    }
    Ok(bag)
}
impl Store {
    /// Trusted authority initialization only. A seed must be checked revision 0;
    /// existing contents are never reseeded. Success is returned after COMMIT.
    pub async fn ensure_inventory(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        seed: &Bag,
    ) -> Result<Bag> {
        let bytes = seed.to_json();
        if seed.revision() != 0 || Bag::from_json(profile.catalog(), &bytes).is_err() {
            return Err(Error::Invalid("Invalid authority inventory seed"));
        }
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        bounded(transaction.execute(
            "INSERT INTO theia.backpacks(realm_id,character_id,catalog_hash,revision,state) VALUES($1,$2,$3,0,$4) ON CONFLICT(realm_id,character_id) DO NOTHING",
            &[&self.realm.value(), &binding.character.value(), &profile.hash().as_slice(), &bytes],
        )).await?;
        let row = bounded(transaction.query_one(
            "SELECT catalog_hash,revision,state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&self.realm.value(), &binding.character.value()],
        )).await?;
        let bag = checked(&row, profile)?;
        bounded(transaction.commit()).await?;
        Ok(bag)
    }
    pub async fn load_inventory(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
    ) -> Result<Bag> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let row = bounded(transaction.query_opt(
            "SELECT catalog_hash,revision,state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2",
            &[&self.realm.value(), &binding.character.value()],
        )).await?.ok_or(Error::InventoryMissing)?;
        let bag = checked(&row, profile)?;
        bounded(transaction.commit()).await?;
        Ok(bag)
    }
    /// Latest receipt only: an exact retry of the most recently committed intent
    /// returns its result. Different input at that revision conflicts. Older or
    /// future revisions fail without mutation; callers reload current contents.
    /// The UI/protocol must allow only one outstanding intent per character.
    pub async fn mutate_inventory(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: Command,
    ) -> Result<InventoryApplied> {
        self.mutate_items(binding, world, profile, ItemIntent::Backpack(command))
            .await
    }
    pub async fn mutate_equipment(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: EquipmentCommand,
    ) -> Result<InventoryApplied> {
        self.mutate_items(binding, world, profile, ItemIntent::Equipment(command))
            .await
    }
    pub(crate) async fn mutate_items(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: ItemIntent,
    ) -> Result<InventoryApplied> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let row = bounded(transaction.query_opt(
            "SELECT catalog_hash,revision,state,last_command FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&self.realm.value(), &binding.character.value()],
        )).await?.ok_or(Error::InventoryMissing)?;
        let bag = checked(&row, profile)?;
        let hash = intent_hash(command)?;
        let receipt: Option<Vec<u8>> = row.get(3);
        if command.expected_revision().checked_add(1) == Some(bag.revision()) {
            if receipt.as_deref() != Some(hash.as_slice()) {
                return Err(Error::InventoryConflict);
            }
            bounded(transaction.commit()).await?;
            return Ok(InventoryApplied {
                bag,
                replayed: true,
            });
        }
        if command.expected_revision() != bag.revision() {
            return Err(Error::InventoryStale);
        }
        let next = command
            .propose(&bag, profile.catalog())
            .map_err(Error::InventoryCommand)?;
        let bytes = next.to_json();
        let count = bounded(transaction.execute(
            "UPDATE theia.backpacks SET revision=$1,state=$2,last_command=$3,updated_at=clock_timestamp() WHERE realm_id=$4 AND character_id=$5 AND revision=$6",
            &[&(next.revision() as i64), &bytes, &hash.as_slice(), &self.realm.value(), &binding.character.value(), &(bag.revision() as i64)],
        )).await?;
        if count != 1 {
            return Err(Error::InventoryData);
        }
        bounded(transaction.commit()).await?;
        Ok(InventoryApplied {
            bag: next,
            replayed: false,
        })
    }
}
