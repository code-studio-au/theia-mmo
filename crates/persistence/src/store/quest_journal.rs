//! Independent event receipts and atomic multi-quest completion. Trusted Store
//! API only; no NPC, party or combat eligibility is inferred by SQL.
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
    let next = revision
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .ok_or(Error::QuestJournalData)?;
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
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        bounded(tx.execute("INSERT INTO theia.quest_journal_heads(realm_id,character_id,world_id,catalog_hash,revision,state_hash) VALUES($1,$2,$3,$4,0,$5) ON CONFLICT(realm_id,character_id) DO NOTHING",&[&self.realm.value(),&binding.character.value(),&world.as_slice(),&catalog.hash().as_slice(),&catalog.state_hash(&Journal::default())?.as_slice()])).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        bounded(tx.commit()).await?;
        Ok(applied(journal, revision, Vec::new(), false))
    }
    pub async fn load_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
    ) -> Result<QuestJournalApplied> {
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        bounded(tx.commit()).await?;
        Ok(applied(journal, revision, Vec::new(), false))
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
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let (revision, journal) = load(&tx, self.realm, binding, world, catalog).await?;
        let bytes = event_id.get().to_le_bytes();
        let hash = catalog.event_hash(event);
        if let Some(row)=bounded(tx.query_opt("SELECT event_hash,journal_revision FROM theia.quest_journal_events WHERE realm_id=$1 AND character_id=$2 AND event_id=$3",&[&self.realm.value(),&binding.character.value(),&bytes.as_slice()])).await? {
            let stored:i64=row.get(1);if row.get::<_,Vec<u8>>(0)!=hash {return Err(Error::QuestJournalConflict);}
            if stored<=0 || stored as u64>revision {return Err(Error::QuestJournalData);}
            bounded(tx.commit()).await?;return Ok(applied(journal,revision,Vec::new(),true));
        }
        let proposal = journal
            .propose_trusted_event(catalog.catalog(), event)
            .map_err(Error::QuestJournalCommand)?;
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
        bounded(tx.execute("INSERT INTO theia.quest_journal_events(realm_id,character_id,event_id,event_hash,journal_revision) VALUES($1,$2,$3,$4,$5)",&[&self.realm.value(),&binding.character.value(),&bytes.as_slice(),&hash.as_slice(),&(revision as i64)])).await?;
        bounded(tx.commit()).await?;
        Ok(applied(proposal.journal, revision, proposal.changed, false))
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
            bounded(tx.commit()).await?;let mut result=applied(journal,revision,Vec::new(),true);result.experience=experience;return Ok(result);
        }
        let proposal = match command.action {
            Action::Accept => {
                if command.expected_revision != 0 {
                    return Err(Error::QuestJournalCommand(
                        theia_core::quests::journal::Error::StaleRevision,
                    ));
                }
                journal.propose_accept(catalog.catalog(), &command.quest, &command.npc)
            }
            Action::TurnIn => journal.propose_turn_in(
                catalog.catalog(),
                &command.quest,
                &command.npc,
                command.expected_revision,
            ),
        }
        .map_err(Error::QuestJournalCommand)?;
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
        result.experience = next_experience;
        result.experience_granted = granted;
        Ok(result)
    }
}
