//! Trusted earned-event persistence. No gameplay credit is inferred here, and
//! neither queueing nor an uncommitted proposal is an XP-success observation.
use super::{Store, bounded, inventory::authorize};
use crate::{Binding, Error, ProgressionApplied, ProgressionCatalog, Result};
use theia_core::progression::{KillEvent, MAX_KILL_EXPERIENCE, State};
use tokio_postgres::Row;
fn checked(row: &Row, world: [u8; 32], catalog: &ProgressionCatalog) -> Result<State> {
    if row.get::<_, Vec<u8>>(0) != world {
        return Err(Error::WorldMismatch);
    }
    if row.get::<_, Vec<u8>>(1) != catalog.hash() {
        return Err(Error::ProgressionCatalogMismatch);
    }
    let revision: i64 = row.get(2);
    let experience: i32 = row.get(3);
    if revision < 0 || experience < 0 {
        return Err(Error::ProgressionData);
    }
    State::new(catalog.profile(), revision as u64, experience as u32)
        .map_err(|_| Error::ProgressionData)
}
impl Store {
    /// Missing state starts at zero. Existing XP is never silently reseeded.
    pub async fn ensure_progression(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &ProgressionCatalog,
    ) -> Result<State> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        bounded(transaction.execute("INSERT INTO theia.progression(realm_id,character_id,world_id,profile_hash,revision,experience) VALUES($1,$2,$3,$4,0,0) ON CONFLICT(realm_id,character_id) DO NOTHING",&[&self.realm.value(),&binding.character.value(),&world.as_slice(),&catalog.hash().as_slice()])).await?;
        let row=bounded(transaction.query_one("SELECT world_id,profile_hash,revision,experience FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR SHARE",&[&self.realm.value(),&binding.character.value()])).await?;
        let state = checked(&row, world, catalog)?;
        bounded(transaction.commit()).await?;
        Ok(state)
    }
    pub async fn load_progression(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &ProgressionCatalog,
    ) -> Result<State> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let row=bounded(transaction.query_opt("SELECT world_id,profile_hash,revision,experience FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR SHARE",&[&self.realm.value(),&binding.character.value()])).await?.ok_or(Error::ProgressionMissing)?;
        let state = checked(&row, world, catalog)?;
        bounded(transaction.commit()).await?;
        Ok(state)
    }
    /// Victim life is unique across recipients in this realm/world's initial
    /// last-hit policy. A later legitimate kill never makes an old receipt new.
    pub async fn award_experience(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &ProgressionCatalog,
        event: &KillEvent,
    ) -> Result<ProgressionApplied> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let result =
            apply_experience(&transaction, self.realm, binding, world, catalog, event).await?;
        bounded(transaction.commit()).await?;
        Ok(result)
    }
}
/// Caller owns authorization and COMMIT, and always locks XP before objectives.
pub(super) async fn apply_experience(
    transaction: &tokio_postgres::Transaction<'_>,
    realm: crate::RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: &ProgressionCatalog,
    event: &KillEvent,
) -> Result<ProgressionApplied> {
    if catalog.profile().kill_experience(&event.creature).is_none() {
        return Err(Error::ProgressionCommand(
            theia_core::progression::Error::UnknownCreature,
        ));
    }
    let row=bounded(transaction.query_opt("SELECT world_id,profile_hash,revision,experience FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()])).await?.ok_or(Error::ProgressionMissing)?;
    let state = checked(&row, world, catalog)?;
    let id = event.id.bytes();
    if let Some(receipt)=bounded(transaction.query_opt("SELECT character_id,profile_hash,creature_id,granted,revision,experience FROM theia.experience_receipts WHERE realm_id=$1 AND world_id=$2 AND event_id=$3 FOR SHARE",&[&realm.value(),&world.as_slice(),&id.as_slice()])).await? {
            if receipt.get::<_,uuid::Uuid>(0)!=binding.character.value() || receipt.get::<_,String>(2)!=event.creature.as_str() {return Err(Error::ProgressionConflict);}
            if receipt.get::<_,Vec<u8>>(1)!=catalog.hash() {return Err(Error::ProgressionCatalogMismatch);}
            let granted:i32=receipt.get(3);let revision:i64=receipt.get(4);let total:i32=receipt.get(5);
            if !(1..=MAX_KILL_EXPERIENCE as i32).contains(&granted) || revision<=0 || revision as u64>state.revision() || total<granted || total as u32>state.total() || (revision as u64==state.revision() && total as u32!=state.total()) {return Err(Error::ProgressionData);}
            return Ok(ProgressionApplied {state,granted:0,replayed:true});
        }
    let proposal = catalog
        .profile()
        .propose_kill(&state, &event.creature)
        .map_err(Error::ProgressionCommand)?;
    if proposal.granted == 0 {
        return Ok(ProgressionApplied {
            state,
            granted: 0,
            replayed: false,
        });
    }
    let changed=bounded(transaction.execute("INSERT INTO theia.experience_receipts(realm_id,world_id,event_id,character_id,profile_hash,creature_id,granted,revision,experience) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(realm_id,world_id,event_id) DO NOTHING",&[&realm.value(),&world.as_slice(),&id.as_slice(),&binding.character.value(),&catalog.hash().as_slice(),&event.creature.as_str(),&(proposal.granted as i32),&(proposal.state.revision()as i64),&(proposal.state.total()as i32)])).await?;
    if changed != 1 {
        return Err(Error::ProgressionConflict);
    }
    let changed=bounded(transaction.execute("UPDATE theia.progression SET revision=$1,experience=$2,updated_at=clock_timestamp() WHERE realm_id=$3 AND character_id=$4 AND revision=$5",&[&(proposal.state.revision()as i64),&(proposal.state.total()as i32),&realm.value(),&binding.character.value(),&(state.revision()as i64)])).await?;
    if changed != 1 {
        return Err(Error::ProgressionData);
    }
    Ok(ProgressionApplied {
        state: proposal.state,
        granted: proposal.granted,
        replayed: false,
    })
}
