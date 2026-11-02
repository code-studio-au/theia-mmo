//! Authorized SQL boundary only. NPC entitlement and accepted work retention are
//! runtime responsibilities; no simulation step calls or awaits this store.
use super::{Store, bounded, inventory::authorize, progression::apply_experience};
use crate::{
    Binding, EarnedQuestLifecycle, Error, ProgressionCatalog, QuestLifecycleApplied,
    QuestLifecycleCatalog, RealmId, Result,
};
use theia_core::{
    progression::KillEvent,
    quests::lifecycle::{self as life, Action, Command, State, Status},
};
use tokio_postgres::{Row, Transaction};

fn status_code(status: Status) -> i16 {
    match status {
        Status::Available => 0,
        Status::Accepted => 1,
        Status::TurnedIn => 2,
    }
}
fn checked(row: &Row, world: [u8; 32], catalog: &QuestLifecycleCatalog) -> Result<State> {
    if row.get::<_, Vec<u8>>(0) != world {
        return Err(Error::WorldMismatch);
    }
    if row.get::<_, Vec<u8>>(1) != catalog.hash() {
        return Err(Error::QuestCatalogMismatch);
    }
    let revision: i64 = row.get(2);
    let status = match row.get::<_, i16>(3) {
        0 => Status::Available,
        1 => Status::Accepted,
        2 => Status::TurnedIn,
        _ => return Err(Error::QuestData),
    };
    let count: i32 = row.get(4);
    if revision < 0 || count < 0 {
        return Err(Error::QuestData);
    }
    State::restore(catalog.profile(), revision as u64, status, count as u32)
        .map_err(|_| Error::QuestData)
}
pub(super) fn unchanged(state: State, replayed: bool) -> QuestLifecycleApplied {
    QuestLifecycleApplied {
        state,
        changed: false,
        became_ready: false,
        turned_in_now: false,
        replayed,
    }
}
pub(super) async fn lock_state(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: &QuestLifecycleCatalog,
) -> Result<State> {
    let row=bounded(tx.query_opt("SELECT world_id,profile_hash,revision,status,kill_count FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()])).await?.ok_or(Error::QuestMissing)?;
    checked(&row, world, catalog)
}
pub(super) async fn update(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    before: &State,
    next: &State,
) -> Result<()> {
    let updated=bounded(tx.execute("UPDATE theia.quest_lifecycles SET revision=$1,status=$2,kill_count=$3,updated_at=clock_timestamp() WHERE realm_id=$4 AND character_id=$5 AND revision=$6",&[&(next.revision()as i64),&status_code(next.status()),&(next.count()as i32),&realm.value(),&binding.character.value(),&(before.revision()as i64)])).await?;
    if updated != 1 {
        return Err(Error::QuestData);
    }
    Ok(())
}
impl Store {
    /// Missing explicit lifecycle starts Available, never silently accepted from
    /// an existing automatic objective. Existing state is never reset.
    pub async fn ensure_quest_lifecycle(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestLifecycleCatalog,
    ) -> Result<State> {
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        bounded(tx.execute("INSERT INTO theia.quest_lifecycles(realm_id,character_id,world_id,profile_hash,revision,status,kill_count) VALUES($1,$2,$3,$4,0,0,0) ON CONFLICT(realm_id,character_id) DO NOTHING",&[&self.realm.value(),&binding.character.value(),&world.as_slice(),&catalog.hash().as_slice()])).await?;
        let state = lock_state(&tx, self.realm, binding, world, catalog).await?;
        bounded(tx.commit()).await?;
        Ok(state)
    }
    pub async fn load_quest_lifecycle(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestLifecycleCatalog,
    ) -> Result<State> {
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let state = lock_state(&tx, self.realm, binding, world, catalog).await?;
        bounded(tx.commit()).await?;
        Ok(state)
    }
    /// This initial lifecycle completes the journal only; no item/XP turn-in
    /// recipe is enabled. A future reward must commit in this same transaction.
    /// Exact action receipts survive later kills and recover without repeating
    /// acceptance/turn-in. NPC authorization happens before trusted submission.
    pub async fn command_quest_lifecycle(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestLifecycleCatalog,
        command: Command,
    ) -> Result<QuestLifecycleApplied> {
        if catalog.turn_in_xp().is_some() {
            return Err(Error::Invalid(
                "Rewarded quests require atomic experience command",
            ));
        }
        if command.expected_revision > 101 {
            return Err(Error::QuestLifecycleCommand(life::Error::StaleRevision));
        }
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let state = lock_state(&tx, self.realm, binding, world, catalog).await?;
        let hash = catalog.command_hash(command);
        if let Some(row)=bounded(tx.query_opt("SELECT command_hash,action FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND expected_revision=$3",&[&self.realm.value(),&binding.character.value(),&(command.expected_revision as i64)])).await? {
            if row.get::<_,Vec<u8>>(0)!=hash || row.get::<_,i16>(1)!=i16::from(command.canonical_bytes()[8]) {return Err(Error::QuestConflict);}
            if state.revision()<=command.expected_revision {return Err(Error::QuestData);}
            bounded(tx.commit()).await?;
            return Ok(unchanged(state,true));
        }
        let proposal = life::propose_action(catalog.profile(), &state, command)
            .map_err(Error::QuestLifecycleCommand)?;
        bounded(tx.execute("INSERT INTO theia.quest_action_receipts(realm_id,character_id,expected_revision,action,command_hash) VALUES($1,$2,$3,$4,$5)",&[&self.realm.value(),&binding.character.value(),&(command.expected_revision as i64),&i16::from(command.canonical_bytes()[8]),&hash.as_slice()])).await?;
        update(&tx, self.realm, binding, &state, &proposal.state).await?;
        bounded(tx.commit()).await?;
        Ok(QuestLifecycleApplied {
            state: proposal.state,
            changed: proposal.changed,
            became_ready: proposal.became_ready,
            turned_in_now: command.action == Action::TurnIn,
            replayed: false,
        })
    }
    /// Trusted kill fact. XP and lifecycle disposition COMMIT atomically; no
    /// client supplies a count, recipient or XP amount. Always lock XP first.
    pub async fn award_progression_and_quest_lifecycle(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        catalog: &QuestLifecycleCatalog,
        event: &KillEvent,
    ) -> Result<EarnedQuestLifecycle> {
        catalog.check_progression(xp)?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let experience = apply_experience(&tx, self.realm, binding, world, xp, event).await?;
        let state = lock_state(&tx, self.realm, binding, world, catalog).await?;
        let id = event.id.bytes();
        let receipt=bounded(tx.query_opt("SELECT character_id,profile_hash,creature_id,revision,kill_count,counted FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND world_id=$2 AND event_id=$3 FOR SHARE",&[&self.realm.value(),&world.as_slice(),&id.as_slice()])).await?;
        let quest = if let Some(row) = receipt {
            if row.get::<_, uuid::Uuid>(0) != binding.character.value()
                || row.get::<_, String>(2) != event.creature.as_str()
            {
                return Err(Error::QuestConflict);
            }
            if row.get::<_, Vec<u8>>(1) != catalog.hash() {
                return Err(Error::QuestCatalogMismatch);
            }
            let revision: i64 = row.get(3);
            let count: i32 = row.get(4);
            let counted: bool = row.get(5);
            if revision < 0
                || count < 0
                || revision as u64 > state.revision()
                || count as u32 > state.count()
                || (counted && (count == 0 || revision != i64::from(count) + 1))
            {
                return Err(Error::QuestData);
            }
            unchanged(state, true)
        } else if &event.creature == catalog.profile().creature() {
            // An event already credited to XP before lifecycle adoption is also
            // old. Record a disposition, never retrospectively count it.
            let proposed = if experience.replayed {
                unchanged(state.clone(), false)
            } else {
                let p = life::propose_kill(catalog.profile(), &state, &event.creature)
                    .map_err(Error::QuestLifecycleCommand)?;
                QuestLifecycleApplied {
                    state: p.state,
                    changed: p.changed,
                    became_ready: p.became_ready,
                    turned_in_now: false,
                    replayed: false,
                }
            };
            let inserted=bounded(tx.execute("INSERT INTO theia.quest_lifecycle_kills(realm_id,world_id,event_id,character_id,profile_hash,creature_id,revision,kill_count,counted) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(realm_id,world_id,event_id) DO NOTHING",&[&self.realm.value(),&world.as_slice(),&id.as_slice(),&binding.character.value(),&catalog.hash().as_slice(),&event.creature.as_str(),&(proposed.state.revision()as i64),&(proposed.state.count()as i32),&proposed.changed])).await?;
            if inserted != 1 {
                return Err(Error::QuestConflict);
            }
            if proposed.changed {
                update(&tx, self.realm, binding, &state, &proposed.state).await?;
            }
            proposed
        } else {
            unchanged(state, false)
        };
        bounded(tx.commit()).await?;
        Ok(EarnedQuestLifecycle { experience, quest })
    }
}
