//! Pure all-or-nothing fixed reward turn-in; no persistence or entitlement.
use super::Catalog;
use crate::{
    content::ContentId,
    inventory::Bag,
    quests::journal::{self, Journal, PossessionTurnIn},
    rewards,
};
#[derive(Clone, Copy, Debug)]
pub struct TurnIn<'a> {
    pub quest: &'a ContentId,
    pub npc: &'a ContentId,
    pub quest_revision: u64,
    pub inventory_revision: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnInError {
    NoReward,
    Quest(journal::Error),
    Inventory(rewards::Error),
}
impl Catalog {
    /// Prepare completion, required-item consumption and fixed grants together.
    /// Only final holdings reconcile other quests. The original action receipt,
    /// XP grant and all returned states must be committed by an authority.
    pub fn propose_turn_in(
        &self,
        journal: &Journal,
        bag: &Bag,
        intent: TurnIn<'_>,
    ) -> Result<PossessionTurnIn, TurnInError> {
        let grants = self.grants(intent.quest).ok_or(TurnInError::NoReward)?;
        bag.validate(&self.items)
            .map_err(|e| TurnInError::Inventory(rewards::Error::ItemState(e)))?;
        if bag.revision() != intent.inventory_revision {
            return Err(TurnInError::Inventory(rewards::Error::StaleRevision));
        }
        let definition = self
            .quests
            .quest(intent.quest)
            .ok_or(TurnInError::NoReward)?;
        let prepared = if definition.has_possession_objectives() {
            journal
                .prepare_possession_turn_in(
                    &self.quests,
                    &self.items,
                    bag,
                    intent.quest,
                    intent.npc,
                    intent.quest_revision,
                )
                .map_err(TurnInError::Quest)?
        } else {
            PossessionTurnIn {
                quest: journal
                    .propose_turn_in(
                        &self.quests,
                        intent.quest,
                        intent.npc,
                        intent.quest_revision,
                    )
                    .map_err(TurnInError::Quest)?,
                bag: bag.clone(),
            }
        };
        // Consumption and grants share one mutation of the original revision.
        // This local candidate is not published or accepted as saved inventory.
        let candidate = Bag::from_parts(
            &self.items,
            bag.revision(),
            prepared.bag.slots().clone(),
            prepared.bag.main_hand().cloned(),
        )
        .map_err(|e| TurnInError::Inventory(rewards::Error::ItemState(e)))?;
        let next =
            rewards::propose_grants(grants, &candidate, &self.items, intent.inventory_revision)
                .map_err(TurnInError::Inventory)?;
        let after = prepared
            .quest
            .journal
            .propose_inventory(&self.quests, &self.items, &next)
            .map_err(TurnInError::Quest)?;
        let changed = prepared
            .quest
            .changed
            .into_iter()
            .chain(after.changed)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(PossessionTurnIn {
            quest: journal::Proposal {
                journal: after.journal,
                changed,
                reward: prepared.quest.reward,
            },
            bag: next,
        })
    }
}
#[cfg(test)]
#[path = "proposal_tests.rs"]
mod tests;
