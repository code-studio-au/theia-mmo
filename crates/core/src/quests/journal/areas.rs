//! Stage-bound logical observations. The caller establishes authoritative,
//! alive occupancy and exact world/content identity before preparing a batch.
use super::*;
use std::collections::BTreeSet;

pub const MAX_REACHED_AREA_GOALS: usize = MAX_ACTIVE_QUESTS * super::super::catalog::MAX_OBJECTIVES;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReachedAreaGoal {
    quest: ContentId,
    definition_version: u32,
    stage: ContentId,
    area: ContentId,
}
impl ReachedAreaGoal {
    pub fn quest(&self) -> &ContentId {
        &self.quest
    }
    pub fn definition_version(&self) -> u32 {
        self.definition_version
    }
    pub fn stage(&self) -> &ContentId {
        &self.stage
    }
    pub fn area(&self) -> &ContentId {
        &self.area
    }
}

/// No deserialization or public constructor: eligibility comes from a checked
/// journal. This is a logical proposal, not proof of a world observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachedAreas {
    catalog: ContentId,
    catalog_revision: u32,
    goals: Vec<ReachedAreaGoal>,
}
impl ReachedAreas {
    pub fn catalog(&self) -> &ContentId {
        &self.catalog
    }
    pub fn catalog_revision(&self) -> u32 {
        self.catalog_revision
    }
    pub fn goals(&self) -> &[ReachedAreaGoal] {
        &self.goals
    }
    pub fn len(&self) -> usize {
        self.goals.len()
    }
    pub fn is_empty(&self) -> bool {
        self.goals.is_empty()
    }
}
impl Journal {
    pub fn prepare_reached_areas(
        &self,
        catalog: &Catalog,
        inside: &BTreeSet<ContentId>,
    ) -> Result<ReachedAreas, Error> {
        self.validate(catalog)?;
        if inside.len() > super::super::areas::MAX_AREAS {
            return Err(Error::Capacity);
        }
        let mut goals = BTreeSet::new();
        for (quest, progress) in &self.quests {
            if progress.status != Status::Active {
                continue;
            }
            let definition = catalog.quest(quest).ok_or(Error::DefinitionMismatch)?;
            let stage = definition
                .stage(&progress.stage)
                .ok_or(Error::DefinitionMismatch)?;
            for objective in stage.objectives() {
                if let EventKey::ReachArea(area) = objective.event()
                    && inside.contains(area)
                    && progress.counts[objective.id()] == 0
                {
                    goals.insert(ReachedAreaGoal {
                        quest: quest.clone(),
                        definition_version: definition.version(),
                        stage: stage.id().clone(),
                        area: area.clone(),
                    });
                }
            }
        }
        if goals.len() > MAX_REACHED_AREA_GOALS {
            return Err(Error::Capacity);
        }
        Ok(ReachedAreas {
            catalog: catalog.id().clone(),
            catalog_revision: catalog.revision(),
            goals: goals.into_iter().collect(),
        })
    }

    pub fn propose_reached_areas(
        &self,
        catalog: &Catalog,
        observation: &ReachedAreas,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        if observation.catalog != *catalog.id()
            || observation.catalog_revision != catalog.revision()
            || observation.len() > MAX_REACHED_AREA_GOALS
        {
            return Err(Error::DefinitionMismatch);
        }
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
                .find(|o| o.event() == &EventKey::ReachArea(goal.area.clone()))
                .ok_or(Error::DefinitionMismatch)?;
            let Some(before) = self
                .quests
                .get(&goal.quest)
                .filter(|p| p.status == Status::Active && p.stage == goal.stage)
            else {
                continue;
            };
            let state = journal.quests.get_mut(&goal.quest).unwrap();
            // Any/at-least stages retain exactly their quorum, matching the
            // existing saved-progress invariant. Never cascade within a batch.
            if before.counts[objective.id()] != 0
                || state.counts[objective.id()] != 0
                || completed(stage, state) >= quorum(stage)
            {
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
            let definition = catalog.quest(quest).unwrap();
            let stage = definition.stage(&state.stage).unwrap();
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
