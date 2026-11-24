//! Original-stage logical read observations. These do not authorize an item open;
//! a durable owner must validate current ownership and exact profiles separately.
use super::*;
use crate::documents::Catalog as Documents;
use std::collections::BTreeSet;
pub const MAX_DOCUMENT_READ_GOALS: usize = MAX_ACTIVE_QUESTS;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentReadGoal {
    quest: ContentId,
    definition_version: u32,
    stage: ContentId,
}
impl DocumentReadGoal {
    pub fn quest(&self) -> &ContentId {
        &self.quest
    }
    pub fn definition_version(&self) -> u32 {
        self.definition_version
    }
    pub fn stage(&self) -> &ContentId {
        &self.stage
    }
}
/// No deserialization/public constructor: retain the original eligible stages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentRead {
    catalog: ContentId,
    catalog_revision: u32,
    documents: ContentId,
    documents_revision: u32,
    document: ContentId,
    goals: Vec<DocumentReadGoal>,
}
impl DocumentRead {
    pub fn catalog(&self) -> &ContentId {
        &self.catalog
    }
    pub fn catalog_revision(&self) -> u32 {
        self.catalog_revision
    }
    pub fn documents(&self) -> &ContentId {
        &self.documents
    }
    pub fn documents_revision(&self) -> u32 {
        self.documents_revision
    }
    pub fn document(&self) -> &ContentId {
        &self.document
    }
    pub fn goals(&self) -> &[DocumentReadGoal] {
        &self.goals
    }
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }
    pub fn len(&self) -> usize {
        self.goals.len()
    }
}
impl Journal {
    pub fn prepare_document_read(
        &self,
        catalog: &Catalog,
        documents: &Documents,
        document: &ContentId,
    ) -> Result<DocumentRead, Error> {
        self.validate(catalog)?;
        documents
            .validate_quests(catalog)
            .map_err(|_| Error::DefinitionMismatch)?;
        if documents.document(document).is_none() {
            return Err(Error::DefinitionMismatch);
        }
        let mut goals = Vec::new();
        let event = EventKey::ReadDocument(document.clone());
        for (quest, state) in &self.quests {
            if state.status != Status::Active {
                continue;
            }
            let definition = catalog.quest(quest).ok_or(Error::DefinitionMismatch)?;
            let stage = definition
                .stage(&state.stage)
                .ok_or(Error::DefinitionMismatch)?;
            if stage
                .objectives()
                .iter()
                .any(|o| o.event() == &event && state.counts[o.id()] == 0)
            {
                goals.push(DocumentReadGoal {
                    quest: quest.clone(),
                    definition_version: definition.version(),
                    stage: stage.id().clone(),
                });
            }
        }
        if goals.len() > MAX_DOCUMENT_READ_GOALS {
            return Err(Error::Capacity);
        }
        Ok(DocumentRead {
            catalog: catalog.id().clone(),
            catalog_revision: catalog.revision(),
            documents: documents.id().clone(),
            documents_revision: documents.revision(),
            document: document.clone(),
            goals,
        })
    }
    pub fn propose_document_read(
        &self,
        catalog: &Catalog,
        documents: &Documents,
        observation: &DocumentRead,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        documents
            .validate_quests(catalog)
            .map_err(|_| Error::DefinitionMismatch)?;
        if observation.catalog != *catalog.id()
            || observation.catalog_revision != catalog.revision()
            || observation.documents != *documents.id()
            || observation.documents_revision != documents.revision()
            || documents.document(&observation.document).is_none()
            || observation.len() > MAX_DOCUMENT_READ_GOALS
        {
            return Err(Error::DefinitionMismatch);
        }
        let event = EventKey::ReadDocument(observation.document.clone());
        let mut journal = self.clone();
        let mut changed = BTreeSet::new();
        for goal in &observation.goals {
            let definition = catalog
                .quest(&goal.quest)
                .ok_or(Error::DefinitionMismatch)?;
            if definition.version() != goal.definition_version {
                return Err(Error::DefinitionMismatch);
            }
            let stage = definition
                .stage(&goal.stage)
                .ok_or(Error::DefinitionMismatch)?;
            let objective = stage
                .objectives()
                .iter()
                .find(|o| o.event() == &event)
                .ok_or(Error::DefinitionMismatch)?;
            let Some(state) = journal
                .quests
                .get_mut(&goal.quest)
                .filter(|p| p.status == Status::Active && p.stage == goal.stage)
            else {
                continue;
            };
            if state.counts[objective.id()] != 0 || completed(stage, state) >= quorum(stage) {
                continue;
            }
            *state.counts.get_mut(objective.id()).unwrap() = 1;
            state.revision = state
                .revision
                .checked_add(1)
                .filter(|v| *v <= super::super::MAX_REVISION)
                .ok_or(Error::RevisionExhausted)?;
            changed.insert(goal.quest.clone());
        }
        for quest in &changed {
            let state = journal.quests.get_mut(quest).unwrap();
            let stage = catalog.quest(quest).unwrap().stage(&state.stage).unwrap();
            if completed(stage, state) == quorum(stage) {
                if let Some(next) = stage.next() {
                    state.stage = next.clone();
                } else {
                    state.status = Status::Ready;
                }
            }
        }
        journal.validate(catalog)?;
        Ok(Proposal {
            journal,
            changed: changed.into_iter().collect(),
            reward: None,
        })
    }
}
