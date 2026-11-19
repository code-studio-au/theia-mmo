//! Independent event receipts and atomic multi-quest completion. Trusted Store
//! API only; no NPC, party or combat eligibility is inferred by SQL.
mod items;
use super::{Store, bounded, inventory::authorize, progression};
use crate::{
    Binding, Error, ProgressionCatalog, QuestJournalAction as Action, QuestJournalApplied,
    QuestJournalCatalog, QuestJournalCommand, RealmId, Result,
};
use std::num::NonZeroU128;
use theia_core::{
    content::ContentId,
    quests::{
        catalog::{EventKey, MAX_QUESTS},
        journal::{Journal, QuestSnapshot, Status},
    },
};
use tokio_postgres::Transaction;
fn code(status: Status) -> i16 {
    match status {
        Status::Active => 0,
        Status::Ready => 1,
        Status::Completed => 2,
    }
}
async fn load(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: &QuestJournalCatalog,
) -> Result<(u64, Journal)> {
    let row=bounded(tx.query_opt("SELECT world_id,catalog_hash,revision,state_hash FROM theia.quest_journal_heads WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&binding.character.value()])).await?.ok_or(Error::QuestJournalMissing)?;
    if row.get::<_, Vec<u8>>(0) != world {
        return Err(Error::WorldMismatch);
    }
    if row.get::<_, Vec<u8>>(1) != catalog.hash() {
        return Err(Error::QuestJournalCatalogMismatch);
    }
    let revision: i64 = row.get(2);
    if revision < 0 {
        return Err(Error::QuestJournalData);
    }
    let rows=bounded(tx.query("SELECT quest_id,definition_version,revision,status,state FROM theia.quest_journal_entries WHERE realm_id=$1 AND character_id=$2 ORDER BY quest_id LIMIT 33",&[&realm.value(),&binding.character.value()])).await?;
    if rows.len() > MAX_QUESTS {
        return Err(Error::QuestJournalData);
    }
    let mut snapshots = Vec::new();
    for row in rows {
        let quest = ContentId::new(&row.get::<_, String>(0)).ok_or(Error::QuestJournalData)?;
        let version: i64 = row.get(1);
        let quest_revision: i64 = row.get(2);
        if quest_revision <= 0 || quest_revision > revision {
            return Err(Error::QuestJournalData);
        }
        let status = match row.get::<_, i16>(3) {
            0 => Status::Active,
            1 => Status::Ready,
            2 => Status::Completed,
            _ => return Err(Error::QuestJournalData),
        };
        snapshots.push(QuestSnapshot {
            quest,
            definition_version: u32::try_from(version).map_err(|_| Error::QuestJournalData)?,
            revision: quest_revision as u64,
            status,
            bytes: row.get(4),
        });
    }
    let journal = Journal::restore_quests(catalog.catalog(), &snapshots)
        .map_err(|_| Error::QuestJournalData)?;
    if row.get::<_, Vec<u8>>(3) != catalog.state_hash(&journal)? {
        return Err(Error::QuestJournalData);
    }
    Ok((revision as u64, journal))
}
async fn save(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    catalog: &QuestJournalCatalog,
    revision: u64,
    journal: &Journal,
    changed: &[ContentId],
) -> Result<u64> {
    let next = journal
        .next_head_revision(revision)
        .map_err(|_| Error::QuestJournalData)?;
    let snapshots = journal
        .quest_snapshots(catalog.catalog())
        .map_err(|_| Error::QuestJournalData)?;
    for quest in changed {
        let q = snapshots
            .iter()
            .find(|q| &q.quest == quest)
            .ok_or(Error::QuestJournalData)?;
        let affected=bounded(tx.execute("INSERT INTO theia.quest_journal_entries(realm_id,character_id,quest_id,definition_version,revision,status,state) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(realm_id,character_id,quest_id) DO UPDATE SET definition_version=EXCLUDED.definition_version,revision=EXCLUDED.revision,status=EXCLUDED.status,state=EXCLUDED.state",&[&realm.value(),&binding.character.value(),&q.quest.as_str(),&i64::from(q.definition_version),&(q.revision as i64),&code(q.status),&q.bytes])).await?;
        if affected != 1 {
            return Err(Error::QuestJournalData);
        }
    }
    let affected=bounded(tx.execute("UPDATE theia.quest_journal_heads SET revision=$1,state_hash=$5 WHERE realm_id=$2 AND character_id=$3 AND revision=$4",&[&(next as i64),&realm.value(),&binding.character.value(),&(revision as i64),&catalog.state_hash(journal)?.as_slice()])).await?;
    if affected != 1 {
        return Err(Error::QuestJournalData);
    }
    Ok(next)
}
fn applied(
    journal: Journal,
    revision: u64,
    changed: Vec<ContentId>,
    replayed: bool,
) -> QuestJournalApplied {
    QuestJournalApplied {
        journal,
        revision,
        changed,
        replayed,
        inventory: None,
        experience: None,
        experience_granted: 0,
    }
}
impl Store {
    pub async fn ensure_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let bag = items::lock(&tx, self.realm, binding, catalog).await?;
        bounded(tx.execute("INSERT INTO theia.quest_journal_heads(realm_id,character_id,world_id,catalog_hash,revision,state_hash) VALUES($1,$2,$3,$4,0,$5) ON CONFLICT(realm_id,character_id) DO NOTHING",&[&self.realm.value(),&binding.character.value(),&world.as_slice(),&catalog.hash().as_slice(),&catalog.state_hash(&Journal::default())?.as_slice()])).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        let result = items::refresh(
            &tx, self.realm, binding, catalog, revision, journal, bag, false,
        )
        .await?;
        bounded(tx.commit()).await?;
        Ok(result)
    }
    pub async fn load_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let bag = items::lock(&tx, self.realm, binding, catalog).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        let result = items::refresh(
            &tx, self.realm, binding, catalog, revision, journal, bag, false,
        )
        .await?;
        bounded(tx.commit()).await?;
        Ok(result)
    }
    /// The same original occurrence may be offered for each legitimately eligible
    /// character. Each receipt covers ALL affected quests, including a pre-accept
    /// no-op. Quest progress does not depend on a matching XP kill recipe.
    pub async fn record_quest_event(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        event_id: NonZeroU128,
        event: &EventKey,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let result = apply_event(
            &tx,
            self.realm,
            binding,
            world,
            catalog,
            event_id,
            |journal| journal.propose_trusted_event(catalog.catalog(), event),
            catalog.event_hash(event),
        )
        .await?;
        bounded(tx.commit()).await?;
        Ok(result)
    }
    /// Already-authorized live occupancy, sealed to original eligible stages.
    /// The receipt and every changed quest commit together; no XP is granted.
    pub async fn record_reached_areas(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        event_id: NonZeroU128,
        observation: &theia_core::quests::journal::ReachedAreas,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        catalog.check_reached_areas(observation)?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let result = apply_event(
            &tx,
            self.realm,
            binding,
            world,
            catalog,
            event_id,
            |journal| journal.propose_reached_areas(catalog.catalog(), observation),
            catalog.reached_areas_hash(observation),
        )
        .await?;
        bounded(tx.commit()).await?;
        Ok(result)
    }
    /// One trusted victim life: optional configured XP and every affected quest
    /// share authorization/COMMIT. No XP recipe is required for quest-only kills.
    pub async fn award_progression_and_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        catalog: &QuestJournalCatalog,
        event: &theia_core::progression::KillEvent,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        catalog.check_progression(xp)?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let configured = xp.profile().kill_experience(&event.creature).is_some();
        // All combined transactions acquire XP before journal state.
        if configured {
            progression::lock_state(&tx, self.realm, binding, world, xp).await?;
        }
        let mut result = apply_event(
            &tx,
            self.realm,
            binding,
            world,
            catalog,
            QuestJournalCatalog::kill_event_id(event),
            |journal| {
                journal.propose_trusted_event(
                    catalog.catalog(),
                    &EventKey::Kill(event.creature.clone()),
                )
            },
            catalog.kill_hash(event),
        )
        .await?;
        if configured {
            let earned =
                progression::apply_experience(&tx, self.realm, binding, world, xp, event).await?;
            // Paired replay cannot mint XP, and a legacy XP-only receipt cannot
            // retrospectively credit a newly accepted quest. Any proposal above
            // remains uncommitted and rolls back when this check refuses.
            if (result.replayed && earned.granted != 0) || (!result.replayed && earned.replayed) {
                return Err(Error::QuestJournalConflict);
            }
            result.experience = Some(earned.state);
            result.experience_granted = earned.granted;
        }
        bounded(tx.commit()).await?;
        Ok(result)
    }
    /// Call only after current NPC/session authorization. XP is locked before
    /// the journal; completion, state and exact request receipt COMMIT together.
    pub async fn command_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        xp: &ProgressionCatalog,
        command: &QuestJournalCommand,
    ) -> Result<QuestJournalApplied> {
        catalog.check_inventory_binding()?;
        catalog.check_progression(xp)?;
        if command.expected_revision > i64::MAX as u64 {
            return Err(Error::QuestJournalData);
        }
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let experience = if command.action == Action::TurnIn {
            Some(progression::lock_state(&tx, self.realm, binding, world, xp).await?)
        } else {
            None
        };
        let bag = items::lock(&tx, self.realm, binding, catalog).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        let id = command.request_id.get().to_le_bytes();
        let hash = catalog.command_hash(command);
        if let Some(row)=bounded(tx.query_opt("SELECT request_hash,quest_id,action,journal_revision,xp_granted,xp_revision,xp_total FROM theia.quest_journal_actions WHERE realm_id=$1 AND character_id=$2 AND request_id=$3",&[&self.realm.value(),&binding.character.value(),&id.as_slice()])).await? {
            if row.get::<_,Vec<u8>>(0)!=hash || row.get::<_,String>(1)!=command.quest.as_str() || row.get::<_,i16>(2)!=command.action.code() {return Err(Error::QuestJournalConflict);}
            let stored:i64=row.get(3);if stored<=0 || stored as u64>revision {return Err(Error::QuestJournalData);}
            if journal.progress(&command.quest).is_none_or(|q|q.revision()<=command.expected_revision) {return Err(Error::QuestJournalData);}
            let granted:i32=row.get(4);let xp_revision:Option<i64>=row.get(5);let total:Option<i32>=row.get(6);
            if let Some(experience)=&experience {
                let (Some(xp_revision),Some(total))=(xp_revision,total) else {return Err(Error::QuestJournalData);};
                let definition=catalog.catalog().quest(&command.quest).ok_or(Error::QuestJournalData)?;
                if granted<0 || granted as u32>definition.experience() || total<granted || total as u32>experience.total() || xp_revision<0 || xp_revision as u64>experience.revision() || (xp_revision as u64==experience.revision() && total as u32!=experience.total()) {return Err(Error::QuestJournalData);}
                let before=(total-granted) as u32;
                if granted as u32!=definition.experience().min(xp.profile().maximum_experience()-before) || journal.progress(&command.quest).is_none_or(|q|q.status()!=Status::Completed) {return Err(Error::QuestJournalData);}
            } else if granted!=0 || xp_revision.is_some() || total.is_some() {return Err(Error::QuestJournalData);}
            let mut result=items::refresh(&tx,self.realm,binding,catalog,revision,journal,bag,true).await?;
            bounded(tx.commit()).await?;result.experience=experience;return Ok(result);
        }
        let mut next_bag = bag.clone();
        let proposal = match command.action {
            Action::Accept => {
                if command.expected_revision != 0 {
                    return Err(Error::QuestJournalCommand(
                        theia_core::quests::journal::Error::StaleRevision,
                    ));
                }
                journal.propose_accept(catalog.catalog(), &command.quest, &command.npc)
            }
            Action::TurnIn
                if catalog
                    .catalog()
                    .quest(&command.quest)
                    .is_some_and(|q| q.has_possession_objectives()) =>
            {
                let items = catalog.inventory().ok_or(Error::QuestJournalData)?;
                let before = bag.as_ref().ok_or(Error::InventoryMissing)?;
                journal
                    .propose_possession_turn_in(
                        catalog.catalog(),
                        items.catalog(),
                        before,
                        &command.quest,
                        &command.npc,
                        command.expected_revision,
                    )
                    .map(|result| {
                        next_bag = Some(result.bag);
                        result.quest
                    })
            }
            Action::TurnIn => journal.propose_turn_in(
                catalog.catalog(),
                &command.quest,
                &command.npc,
                command.expected_revision,
            ),
        }
        .map_err(Error::QuestJournalCommand)?;
        let proposal = items::reconcile(catalog, next_bag.as_ref(), proposal)?;
        let (next_experience, granted) = if let Some(before) = &experience {
            let reward = proposal.reward.as_ref().ok_or(Error::QuestJournalData)?;
            let p = if reward.experience == 0 {
                None
            } else {
                Some(
                    xp.profile()
                        .propose_grant(before, reward.experience)
                        .map_err(Error::ProgressionCommand)?,
                )
            };
            if let Some(p) = p {
                if p.granted > 0 {
                    progression::update_state(&tx, self.realm, binding, before, &p.state).await?;
                }
                (Some(p.state), p.granted)
            } else {
                (Some(before.clone()), 0)
            }
        } else {
            (None, 0)
        };
        if let (Some(before), Some(after)) = (&bag, &next_bag) {
            items::consume(&tx, self.realm, binding, before, after).await?;
        }
        let revision = save(
            &tx,
            self.realm,
            binding,
            catalog,
            revision,
            &proposal.journal,
            &proposal.changed,
        )
        .await?;
        let xp_revision = next_experience.as_ref().map(|x| x.revision() as i64);
        let total = next_experience.as_ref().map(|x| x.total() as i32);
        bounded(tx.execute("INSERT INTO theia.quest_journal_actions(realm_id,character_id,request_id,request_hash,quest_id,action,journal_revision,xp_granted,xp_revision,xp_total) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",&[&self.realm.value(),&binding.character.value(),&id.as_slice(),&hash.as_slice(),&command.quest.as_str(),&command.action.code(),&(revision as i64),&(granted as i32),&xp_revision,&total])).await?;
        bounded(tx.commit()).await?;
        let mut result = applied(proposal.journal, revision, proposal.changed, false);
        result.inventory = next_bag;
        result.experience = next_experience;
        result.experience_granted = granted;
        Ok(result)
    }
}

#[allow(clippy::too_many_arguments)]
async fn apply_event(
    tx: &tokio_postgres::Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: &QuestJournalCatalog,
    event_id: NonZeroU128,
    propose: impl FnOnce(
        &Journal,
    ) -> std::result::Result<
        theia_core::quests::journal::Proposal,
        theia_core::quests::journal::Error,
    >,
    hash: [u8; 32],
) -> Result<QuestJournalApplied> {
    let bag = items::lock(tx, realm, binding, catalog).await?;
    let (revision, journal) = load(tx, realm, binding, world, catalog).await?;
    let bytes = event_id.get().to_le_bytes();
    if let Some(row)=bounded(tx.query_opt("SELECT event_hash,journal_revision FROM theia.quest_journal_events WHERE realm_id=$1 AND character_id=$2 AND event_id=$3",&[&realm.value(),&binding.character.value(),&bytes.as_slice()])).await? {
            let stored:i64=row.get(1);if row.get::<_,Vec<u8>>(0)!=hash {return Err(Error::QuestJournalConflict);}
            if stored<=0 || stored as u64>revision {return Err(Error::QuestJournalData);}
            return items::refresh(tx,realm,binding,catalog,revision,journal,bag,true).await;
        }
    let proposal = items::reconcile(
        catalog,
        bag.as_ref(),
        propose(&journal).map_err(Error::QuestJournalCommand)?,
    )?;
    let revision = save(
        tx,
        realm,
        binding,
        catalog,
        revision,
        &proposal.journal,
        &proposal.changed,
    )
    .await?;
    bounded(tx.execute("INSERT INTO theia.quest_journal_events(realm_id,character_id,event_id,event_hash,journal_revision) VALUES($1,$2,$3,$4,$5)",&[&realm.value(),&binding.character.value(),&bytes.as_slice(),&hash.as_slice(),&(revision as i64)])).await?;
    let mut result = applied(proposal.journal, revision, proposal.changed, false);
    result.inventory = bag;
    Ok(result)
}

/// Called with the backpack already locked, before the enclosing item COMMIT.
#[allow(clippy::too_many_arguments)]
pub(super) async fn reconcile_inventory(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
    catalog: Option<&QuestJournalCatalog>,
    profile: &crate::InventoryCatalog,
    bag: &theia_core::inventory::Bag,
    replayed: bool,
) -> Result<Option<Box<QuestJournalApplied>>> {
    let Some(catalog) = catalog else {
        return Ok(None);
    };
    catalog.check_inventory_binding()?;
    if catalog
        .inventory()
        .is_none_or(|items| items.hash() != profile.hash())
    {
        return Err(Error::InventoryCatalogMismatch);
    }
    let (revision, journal) = load(tx, realm, binding, world, catalog).await?;
    Ok(Some(Box::new(
        items::refresh(
            tx,
            realm,
            binding,
            catalog,
            revision,
            journal,
            Some(bag.clone()),
            replayed,
        )
        .await?,
    )))
}
