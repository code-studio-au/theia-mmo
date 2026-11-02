//! Pure multi-quest proposals. The durable owner authorizes characters/world
//! facts, deduplicates original event receipts and commits rewards before ack.
use super::catalog::{Catalog, Completion, Definition, EventKey, MAX_QUESTS, Stage};
use crate::content::ContentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_ACTIVE_QUESTS: usize = 16;
pub const MAX_JOURNAL_BYTES: usize = 65536;
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Ready,
    Completed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    version: u32,
    revision: u64,
    status: Status,
    stage: ContentId,
    counts: BTreeMap<ContentId, u32>,
}
impl Progress {
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn status(&self) -> Status {
        self.status
    }
    pub fn stage(&self) -> &ContentId {
        &self.stage
    }
    pub fn count(&self, objective: &ContentId) -> Option<u32> {
        self.counts.get(objective).copied()
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Journal {
    quests: BTreeMap<ContentId, Progress>,
}
/// Untrusted persistence envelope; `restore_quests` checks metadata and bytes
/// together before accepting it as logical progress. It grants no ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestSnapshot {
    pub quest: ContentId,
    pub definition_version: u32,
    pub revision: u64,
    pub status: Status,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionReward {
    pub quest: ContentId,
    pub definition_version: u32,
    pub completed_revision: u64,
    pub experience: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub journal: Journal,
    pub changed: Vec<ContentId>,
    pub reward: Option<CompletionReward>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidSnapshot,
    DefinitionMismatch,
    InvalidProgress,
    UnknownQuest,
    AlreadyAccepted,
    Prerequisite,
    Capacity,
    WrongNpc,
    StaleRevision,
    NotReady,
    RevisionExhausted,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedJournal {
    schema: u32,
    catalog: String,
    catalog_revision: u32,
    quests: Vec<SavedQuest>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedQuest {
    quest: String,
    definition_version: u32,
    revision: u64,
    status: Status,
    stage: String,
    objectives: Vec<SavedCount>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedCount {
    objective: String,
    count: u32,
}
fn content(raw: &str) -> Result<ContentId, Error> {
    ContentId::new(raw).ok_or(Error::InvalidSnapshot)
}
fn quorum(stage: &Stage) -> usize {
    match stage.completion() {
        Completion::All => stage.objectives().len(),
        Completion::Any => 1,
        Completion::AtLeast { count } => count as usize,
    }
}
fn completed(stage: &Stage, state: &Progress) -> usize {
    stage
        .objectives()
        .iter()
        .filter(|o| state.counts.get(o.id()) == Some(&o.required()))
        .count()
}
fn validate_progress(definition: &Definition, state: &Progress) -> Result<(), Error> {
    if state.version != definition.version() {
        return Err(Error::DefinitionMismatch);
    }
    let objectives: Vec<_> = definition.stages().flat_map(|s| s.objectives()).collect();
    if state.counts.len() != objectives.len()
        || objectives
            .iter()
            .any(|o| state.counts.get(o.id()).is_none_or(|c| *c > o.required()))
    {
        return Err(Error::InvalidProgress);
    }
    let total: u64 = state.counts.values().map(|n| u64::from(*n)).sum();
    let expected = 1 + total + u64::from(state.status == Status::Completed);
    if state.revision != expected || state.revision > super::MAX_REVISION {
        return Err(Error::InvalidProgress);
    }
    let mut current = Some(definition.start_stage());
    let mut seen_current = false;
    while let Some(id) = current {
        let stage = definition.stage(id).ok_or(Error::DefinitionMismatch)?;
        if id == &state.stage {
            seen_current = true;
            match state.status {
                Status::Active if completed(stage, state) < quorum(stage) => {}
                Status::Ready | Status::Completed
                    if stage.next().is_none() && completed(stage, state) == quorum(stage) => {}
                _ => return Err(Error::InvalidProgress),
            }
        } else if !seen_current {
            if completed(stage, state) != quorum(stage) {
                return Err(Error::InvalidProgress);
            }
        } else if stage.objectives().iter().any(|o| state.counts[o.id()] != 0) {
            return Err(Error::InvalidProgress);
        }
        current = stage.next();
    }
    if !seen_current {
        return Err(Error::InvalidProgress);
    }
    Ok(())
}
impl Journal {
    pub fn quest_snapshots(&self, catalog: &Catalog) -> Result<Vec<QuestSnapshot>, Error> {
        self.validate(catalog)?;
        self.quests
            .iter()
            .map(|(id, q)| {
                let saved = SavedQuest {
                    quest: id.as_str().to_owned(),
                    definition_version: q.version,
                    revision: q.revision,
                    status: q.status,
                    stage: q.stage.as_str().to_owned(),
                    objectives: q
                        .counts
                        .iter()
                        .map(|(id, count)| SavedCount {
                            objective: id.as_str().to_owned(),
                            count: *count,
                        })
                        .collect(),
                };
                let bytes = serde_json::to_vec(&saved).map_err(|_| Error::InvalidSnapshot)?;
                Ok(QuestSnapshot {
                    quest: id.clone(),
                    definition_version: q.version,
                    revision: q.revision,
                    status: q.status,
                    bytes,
                })
            })
            .collect()
    }
    pub fn restore_quests(catalog: &Catalog, rows: &[QuestSnapshot]) -> Result<Self, Error> {
        if rows.len() > MAX_QUESTS
            || rows
                .iter()
                .try_fold(0usize, |n, r| n.checked_add(r.bytes.len()))
                .is_none_or(|n| n > MAX_JOURNAL_BYTES)
        {
            return Err(Error::InvalidSnapshot);
        }
        let mut quests = Vec::new();
        for row in rows {
            let saved: SavedQuest =
                serde_json::from_slice(&row.bytes).map_err(|_| Error::InvalidSnapshot)?;
            if saved.quest != row.quest.as_str()
                || saved.definition_version != row.definition_version
                || saved.revision != row.revision
                || saved.status != row.status
            {
                return Err(Error::InvalidSnapshot);
            }
            quests.push(saved);
        }
        let saved = SavedJournal {
            schema: 1,
            catalog: catalog.id().as_str().to_owned(),
            catalog_revision: catalog.revision(),
            quests,
        };
        Self::restore(
            catalog,
            &serde_json::to_vec(&saved).map_err(|_| Error::InvalidSnapshot)?,
        )
    }
    pub fn progress(&self, quest: &ContentId) -> Option<&Progress> {
        self.quests.get(quest)
    }
    pub fn quests(&self) -> impl Iterator<Item = (&ContentId, &Progress)> {
        self.quests.iter()
    }
    pub fn validate(&self, catalog: &Catalog) -> Result<(), Error> {
        if self.quests.len() > MAX_QUESTS
            || self
                .quests
                .values()
                .filter(|q| q.status != Status::Completed)
                .count()
                > MAX_ACTIVE_QUESTS
        {
            return Err(Error::Capacity);
        }
        for (quest, state) in &self.quests {
            validate_progress(
                catalog.quest(quest).ok_or(Error::DefinitionMismatch)?,
                state,
            )?;
        }
        // Even restored active/completed quests must have their prerequisites
        // completed in this same character journal; no free unlock on restore.
        for quest in self.quests.keys() {
            for prerequisite in catalog.quest(quest).unwrap().prerequisites() {
                if !self
                    .quests
                    .get(prerequisite)
                    .is_some_and(|p| p.status == Status::Completed)
                {
                    return Err(Error::Prerequisite);
                }
            }
        }
        Ok(())
    }
    pub fn snapshot(&self, catalog: &Catalog) -> Result<Vec<u8>, Error> {
        self.validate(catalog)?;
        let saved = SavedJournal {
            schema: 1,
            catalog: catalog.id().as_str().to_owned(),
            catalog_revision: catalog.revision(),
            quests: self
                .quests
                .iter()
                .map(|(id, q)| SavedQuest {
                    quest: id.as_str().to_owned(),
                    definition_version: q.version,
                    revision: q.revision,
                    status: q.status,
                    stage: q.stage.as_str().to_owned(),
                    objectives: q
                        .counts
                        .iter()
                        .map(|(id, count)| SavedCount {
                            objective: id.as_str().to_owned(),
                            count: *count,
                        })
                        .collect(),
                })
                .collect(),
        };
        let bytes = serde_json::to_vec(&saved).map_err(|_| Error::InvalidSnapshot)?;
        if bytes.len() > MAX_JOURNAL_BYTES {
            return Err(Error::Capacity);
        }
        Ok(bytes)
    }
    /// Logical restore only. The owner must first verify the exact definition
    /// digest and stable character binding; versions alone do not authenticate it.
    pub fn restore(catalog: &Catalog, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_JOURNAL_BYTES {
            return Err(Error::InvalidSnapshot);
        }
        let saved: SavedJournal =
            serde_json::from_slice(bytes).map_err(|_| Error::InvalidSnapshot)?;
        if saved.schema != 1
            || saved.catalog != catalog.id().as_str()
            || saved.catalog_revision != catalog.revision()
        {
            return Err(Error::DefinitionMismatch);
        }
        if saved.quests.len() > MAX_QUESTS {
            return Err(Error::Capacity);
        }
        let mut quests = BTreeMap::new();
        for quest in saved.quests {
            let mut counts = BTreeMap::new();
            for objective in quest.objectives {
                if counts
                    .insert(content(&objective.objective)?, objective.count)
                    .is_some()
                {
                    return Err(Error::InvalidSnapshot);
                }
            }
            let progress = Progress {
                version: quest.definition_version,
                revision: quest.revision,
                status: quest.status,
                stage: content(&quest.stage)?,
                counts,
            };
            if quests.insert(content(&quest.quest)?, progress).is_some() {
                return Err(Error::InvalidSnapshot);
            }
        }
        let journal = Self { quests };
        journal.validate(catalog)?;
        Ok(journal)
    }
    pub fn propose_accept(
        &self,
        catalog: &Catalog,
        quest: &ContentId,
        npc: &ContentId,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        let definition = catalog.quest(quest).ok_or(Error::UnknownQuest)?;
        if definition.giver() != npc {
            return Err(Error::WrongNpc);
        }
        if self.quests.contains_key(quest) {
            return Err(Error::AlreadyAccepted);
        }
        if self
            .quests
            .values()
            .filter(|q| q.status != Status::Completed)
            .count()
            >= MAX_ACTIVE_QUESTS
        {
            return Err(Error::Capacity);
        }
        if definition.prerequisites().iter().any(|id| {
            !self
                .quests
                .get(id)
                .is_some_and(|q| q.status == Status::Completed)
        }) {
            return Err(Error::Prerequisite);
        }
        let counts = definition
            .stages()
            .flat_map(|s| s.objectives())
            .map(|o| (o.id().clone(), 0))
            .collect();
        let mut journal = self.clone();
        journal.quests.insert(
            quest.clone(),
            Progress {
                version: definition.version(),
                revision: 1,
                status: Status::Active,
                stage: definition.start_stage().clone(),
                counts,
            },
        );
        Ok(Proposal {
            journal,
            changed: vec![quest.clone()],
            reward: None,
        })
    }
    /// One normalized already-authorized occurrence. Event receipt deduplication
    /// is outside this pure proposal; replaying it as a new occurrence can count
    /// again. Match against the ORIGINAL stage, so one event never cascades.
    pub fn propose_trusted_event(
        &self,
        catalog: &Catalog,
        event: &EventKey,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        let mut journal = self.clone();
        let mut changed = Vec::new();
        for reference in catalog.matching(event) {
            let Some(before) = self
                .quests
                .get(&reference.quest)
                .filter(|q| q.status == Status::Active && q.stage == reference.stage)
            else {
                continue;
            };
            let definition = catalog
                .quest(&reference.quest)
                .ok_or(Error::DefinitionMismatch)?;
            let stage = definition
                .stage(&reference.stage)
                .ok_or(Error::DefinitionMismatch)?;
            let objective = stage
                .objectives()
                .iter()
                .find(|o| o.id() == &reference.objective)
                .ok_or(Error::DefinitionMismatch)?;
            if before.counts[objective.id()] >= objective.required() {
                continue;
            }
            let state = journal.quests.get_mut(&reference.quest).unwrap();
            *state.counts.get_mut(objective.id()).unwrap() += 1;
            state.revision = state
                .revision
                .checked_add(1)
                .filter(|v| *v <= super::MAX_REVISION)
                .ok_or(Error::RevisionExhausted)?;
            if stage
                .completion()
                .satisfied(completed(stage, state), stage.objectives().len())
            {
                if let Some(next) = stage.next() {
                    state.stage = next.clone();
                } else {
                    state.status = Status::Ready;
                }
            }
            changed.push(reference.quest.clone());
        }
        journal.validate(catalog)?;
        Ok(Proposal {
            journal,
            changed,
            reward: None,
        })
    }
    pub fn propose_turn_in(
        &self,
        catalog: &Catalog,
        quest: &ContentId,
        npc: &ContentId,
        expected_revision: u64,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        let definition = catalog.quest(quest).ok_or(Error::UnknownQuest)?;
        if definition.turn_in() != npc {
            return Err(Error::WrongNpc);
        }
        let before = self.quests.get(quest).ok_or(Error::NotReady)?;
        if before.revision != expected_revision {
            return Err(Error::StaleRevision);
        }
        if before.status != Status::Ready {
            return Err(Error::NotReady);
        }
        let mut journal = self.clone();
        let state = journal.quests.get_mut(quest).unwrap();
        state.revision = state
            .revision
            .checked_add(1)
            .filter(|v| *v <= super::MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        state.status = Status::Completed;
        let reward = CompletionReward {
            quest: quest.clone(),
            definition_version: definition.version(),
            completed_revision: state.revision,
            experience: definition.experience(),
        };
        Ok(Proposal {
            journal,
            changed: vec![quest.clone()],
            reward: Some(reward),
        })
    }
}
