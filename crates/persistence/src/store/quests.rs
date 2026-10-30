//! Optional objective and XP credit share one authorized transaction/COMMIT.
use super::{Store, bounded, inventory::authorize, progression::apply_experience};
use crate::{
    Binding, EarnedProgression, Error, ProgressionCatalog, QuestApplied, QuestCatalog, Result,
};
use theia_core::{progression::KillEvent, quests::State};
use tokio_postgres::{Row, Transaction};
fn checked(row: &Row, world: [u8; 32], catalog: &QuestCatalog) -> Result<State> {
    if row.get::<_, Vec<u8>>(0) != world {
        return Err(Error::WorldMismatch);
    }
    if row.get::<_, Vec<u8>>(1) != catalog.hash() {
        return Err(Error::QuestCatalogMismatch);
    }
    let revision: i64 = row.get(2);
    let count: i32 = row.get(3);
    if revision < 0 || count < 0 {
        return Err(Error::QuestData);
    }
    State::new(catalog.profile(), revision as u64, count as u32).map_err(|_| Error::QuestData)
}
impl Store {
    /// Initializes only a missing opt-in development fixture. No automatic
    /// production acceptance/retake policy and no reseeding existing progress.
    pub async fn ensure_quest(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestCatalog,
    ) -> Result<State> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        bounded(transaction.execute("INSERT INTO theia.quest_objectives(realm_id,character_id,world_id,profile_hash,revision,kill_count) VALUES($1,$2,$3,$4,0,0) ON CONFLICT(realm_id,character_id) DO NOTHING",&[&self.realm.value(),&binding.character.value(),&world.as_slice(),&catalog.hash().as_slice()])).await?;
        let row=bounded(transaction.query_one("SELECT world_id,profile_hash,revision,kill_count FROM theia.quest_objectives WHERE realm_id=$1 AND character_id=$2 FOR SHARE",&[&self.realm.value(),&binding.character.value()])).await?;
        let state = checked(&row, world, catalog)?;
        bounded(transaction.commit()).await?;
        Ok(state)
    }
    pub async fn load_quest(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestCatalog,
    ) -> Result<State> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let row=bounded(transaction.query_opt("SELECT world_id,profile_hash,revision,kill_count FROM theia.quest_objectives WHERE realm_id=$1 AND character_id=$2 FOR SHARE",&[&self.realm.value(),&binding.character.value()])).await?.ok_or(Error::QuestMissing)?;
        let state = checked(&row, world, catalog)?;
        bounded(transaction.commit()).await?;
        Ok(state)
    }
    /// Trusted current-life fact only. The runtime establishes credit; no client
    /// can select a recipient, objective count, XP amount or victim receipt ID.
    pub async fn award_progression_and_quest(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        quest: &QuestCatalog,
        event: &KillEvent,
    ) -> Result<EarnedProgression> {
        quest.check_progression(xp)?;
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        let experience =
            apply_experience(&transaction, self.realm, binding, world, xp, event).await?;
        let quest = apply_quest(&transaction, self.realm, binding, world, quest, event).await?;
        bounded(transaction.commit()).await?;
        Ok(EarnedProgression { experience, quest })
    }
}
async fn apply_quest(
    transaction: &Transaction<'_>,
    realm: crate::RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: &QuestCatalog,
    event: &KillEvent,
) -> Result<QuestApplied> {
    let row=bounded(transaction.query_opt("SELECT world_id,profile_hash,revision,kill_count FROM theia.quest_objectives WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()])).await?.ok_or(Error::QuestMissing)?;
    let state = checked(&row, world, catalog)?;
    let id = event.id.bytes();
    if let Some(receipt)=bounded(transaction.query_opt("SELECT character_id,profile_hash,creature_id,revision,kill_count FROM theia.quest_kill_receipts WHERE realm_id=$1 AND world_id=$2 AND event_id=$3 FOR SHARE",&[&realm.value(),&world.as_slice(),&id.as_slice()])).await? {
        if receipt.get::<_,uuid::Uuid>(0)!=binding.character.value() || receipt.get::<_,String>(2)!=event.creature.as_str() {return Err(Error::QuestConflict);}
        if receipt.get::<_,Vec<u8>>(1)!=catalog.hash() {return Err(Error::QuestCatalogMismatch);}
        let revision:i64=receipt.get(3);let count:i32=receipt.get(4);
        if revision<=0 || revision as u64>state.revision() || count<=0 || count as u32>state.count() || count as u32>catalog.profile().required() || (revision as u64==state.revision() && count as u32!=state.count()) {return Err(Error::QuestData);}
        return Ok(QuestApplied {state,advanced:false,completed_now:false,replayed:true});
    }
    let proposal = catalog
        .profile()
        .propose_kill(&state, &event.creature)
        .map_err(Error::QuestCommand)?;
    if !proposal.advanced {
        return Ok(QuestApplied {
            state,
            advanced: false,
            completed_now: false,
            replayed: false,
        });
    }
    let inserted=bounded(transaction.execute("INSERT INTO theia.quest_kill_receipts(realm_id,world_id,event_id,character_id,profile_hash,creature_id,revision,kill_count) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(realm_id,world_id,event_id) DO NOTHING",&[&realm.value(),&world.as_slice(),&id.as_slice(),&binding.character.value(),&catalog.hash().as_slice(),&event.creature.as_str(),&(proposal.state.revision()as i64),&(proposal.state.count()as i32)])).await?;
    if inserted != 1 {
        return Err(Error::QuestConflict);
    }
    let updated=bounded(transaction.execute("UPDATE theia.quest_objectives SET revision=$1,kill_count=$2,updated_at=clock_timestamp() WHERE realm_id=$3 AND character_id=$4 AND revision=$5",&[&(proposal.state.revision()as i64),&(proposal.state.count()as i32),&realm.value(),&binding.character.value(),&(state.revision()as i64)])).await?;
    if updated != 1 {
        return Err(Error::QuestData);
    }
    Ok(QuestApplied {
        state: proposal.state,
        advanced: true,
        completed_now: proposal.completed_now,
        replayed: false,
    })
}
