//! Pure final-stage possession reconciliation and tentative item turn-in.
//! The caller supplies current authoritative inventory and commits all results.
use super::*;
use crate::inventory::{Bag, Catalog as ItemCatalog};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PossessionTurnIn {
    pub quest: Proposal,
    pub bag: Bag,
}
impl Journal {
    /// Recalculate active/ready final-stage quantities from current backpack
    /// holdings. Earlier historical facts and completed quests never regress.
    pub fn propose_inventory(
        &self,
        catalog: &Catalog,
        items: &ItemCatalog,
        bag: &Bag,
    ) -> Result<Proposal, Error> {
        self.validate(catalog)?;
        catalog
            .validate_possession_items(items)
            .map_err(|_| Error::DefinitionMismatch)?;
        bag.validate(items).map_err(|_| Error::InvalidProgress)?;
        let mut journal = self.clone();
        let mut changed = Vec::new();
        for (id, before) in &self.quests {
            if before.status == Status::Completed {
                continue;
            }
            let definition = catalog.quest(id).ok_or(Error::DefinitionMismatch)?;
            let stage = definition
                .stage(&before.stage)
                .ok_or(Error::DefinitionMismatch)?;
            if !stage
                .objectives()
                .iter()
                .any(|o| matches!(o.event(), EventKey::PossessItem(_)))
            {
                continue;
            }
            let requirements = definition
                .possession_requirements(items)
                .map_err(|_| Error::DefinitionMismatch)?
                .ok_or(Error::DefinitionMismatch)?;
            let observation = requirements
                .observe(items, bag)
                .map_err(|_| Error::InvalidProgress)?;
            let state = journal.quests.get_mut(id).unwrap();
            let mut modified = false;
            for objective in stage.objectives() {
                if let EventKey::PossessItem(item) = objective.event() {
                    let holding = observation
                        .holdings()
                        .iter()
                        .find(|h| h.item() == item)
                        .ok_or(Error::DefinitionMismatch)?;
                    let next = holding.displayed_count();
                    let count = state.counts.get_mut(objective.id()).unwrap();
                    if *count != next {
                        *count = next;
                        modified = true;
                        state.revision = state
                            .revision
                            .checked_add(1)
                            .filter(|v| *v <= super::super::MAX_REVISION)
                            .ok_or(Error::RevisionExhausted)?;
                    }
                }
            }
            if modified {
                state.status = if completed(stage, state) == quorum(stage) {
                    Status::Ready
                } else {
                    Status::Active
                };
                changed.push(id.clone());
            }
        }
        journal.validate(catalog)?;
        Ok(Proposal {
            journal,
            changed,
            reward: None,
        })
    }
    /// Recheck current holdings, then prepare consumption and ordinary once-only
    /// completion together. Neither returned value may be acknowledged alone.
    pub fn propose_possession_turn_in(
        &self,
        catalog: &Catalog,
        items: &ItemCatalog,
        bag: &Bag,
        quest: &ContentId,
        npc: &ContentId,
        expected_revision: u64,
    ) -> Result<PossessionTurnIn, Error> {
        self.validate(catalog)?;
        let definition = catalog.quest(quest).ok_or(Error::UnknownQuest)?;
        let requirements = definition
            .possession_requirements(items)
            .map_err(|_| Error::DefinitionMismatch)?
            .ok_or(Error::InvalidProgress)?;
        // Do not trust cached Ready, even when the caller failed to reconcile
        // a previous removal. A fresh count must match this logical projection.
        let fresh = self.propose_inventory(catalog, items, bag)?;
        if fresh.journal.progress(quest) != self.progress(quest) {
            return Err(Error::NotReady);
        }
        if !requirements
            .observe(items, bag)
            .map_err(|_| Error::InvalidProgress)?
            .satisfied()
        {
            return Err(Error::NotReady);
        }
        let proposal = self.turn_in_checked(catalog, quest, npc, expected_revision, true)?;
        let next = requirements
            .propose_consumption(items, bag)
            .map_err(|error| match error {
                crate::inventory::requirements::Error::Inventory(
                    crate::inventory::Error::RevisionExhausted,
                ) => Error::RevisionExhausted,
                crate::inventory::requirements::Error::MissingItems => Error::NotReady,
                _ => Error::InvalidProgress,
            })?;
        // Consuming this quest's materials may invalidate another ready quest.
        // Completed delivery counts remain historical proof and are untouched.
        let after = proposal.journal.propose_inventory(catalog, items, &next)?;
        let changed = proposal
            .changed
            .into_iter()
            .chain(after.changed)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(PossessionTurnIn {
            quest: Proposal {
                journal: after.journal,
                changed,
                reward: proposal.reward,
            },
            bag: next,
        })
    }
}
