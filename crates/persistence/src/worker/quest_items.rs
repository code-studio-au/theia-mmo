//! Item-bound journal jobs retain accepted work on the existing critical FIFO.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{
    Binding, ClaimId, InventoryApplied, InventoryCatalog, QuestJournalCatalog, Result,
    RewardCatalog, Store, VendorCatalog,
};
use theia_core::{inventory::ItemCommand, vendors::Purchase};
use tokio::sync::oneshot;
enum Operation {
    Items {
        profile: InventoryCatalog,
        command: ItemCommand,
    },
    Claim {
        profile: RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
    },
    Purchase {
        profile: VendorCatalog,
        purchase: Purchase,
    },
}
pub(super) struct Request {
    binding: Binding,
    world: [u8; 32],
    journal: QuestJournalCatalog,
    operation: Operation,
    reply: oneshot::Sender<Result<InventoryApplied>>,
}
impl Publisher {
    pub fn enqueue_items_with_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: ItemCommand,
        journal: &QuestJournalCatalog,
    ) -> Result<oneshot::Receiver<Result<InventoryApplied>>> {
        self.queue_quest_items(
            binding,
            world,
            journal,
            Operation::Items {
                profile: profile.clone(),
                command,
            },
        )
    }
    pub async fn mutate_items_with_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: ItemCommand,
        journal: &QuestJournalCatalog,
    ) -> Result<InventoryApplied> {
        self.wait_quest_items(
            binding,
            world,
            journal,
            Operation::Items {
                profile: profile.clone(),
                command,
            },
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn claim_reward_with_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
        journal: &QuestJournalCatalog,
    ) -> Result<InventoryApplied> {
        self.wait_quest_items(
            binding,
            world,
            journal,
            Operation::Claim {
                profile: profile.clone(),
                claim,
                expected_revision,
            },
        )
        .await
    }
    pub async fn purchase_vendor_with_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &VendorCatalog,
        purchase: &Purchase,
        journal: &QuestJournalCatalog,
    ) -> Result<InventoryApplied> {
        self.wait_quest_items(
            binding,
            world,
            journal,
            Operation::Purchase {
                profile: profile.clone(),
                purchase: purchase.clone(),
            },
        )
        .await
    }
    fn queue_quest_items(
        &self,
        binding: Binding,
        world: [u8; 32],
        journal: &QuestJournalCatalog,
        operation: Operation,
    ) -> Result<oneshot::Receiver<Result<InventoryApplied>>> {
        valid_world(world)?;
        journal.check_inventory_binding()?;
        let (reply, answer) = oneshot::channel();
        self.inventory
            .try_send(inventory::Request::QuestItems(Request {
                binding,
                world,
                journal: journal.clone(),
                operation,
                reply,
            }))
            .map_err(|error| match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => crate::Error::InventoryBusy,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    crate::Error::Invalid("Quest item worker closed")
                }
            })?;
        Ok(answer)
    }
    async fn wait_quest_items(
        &self,
        binding: Binding,
        world: [u8; 32],
        journal: &QuestJournalCatalog,
        operation: Operation,
    ) -> Result<InventoryApplied> {
        valid_world(world)?;
        journal.check_inventory_binding()?;
        let (reply, answer) = oneshot::channel();
        self.inventory
            .send(inventory::Request::QuestItems(Request {
                binding,
                world,
                journal: journal.clone(),
                operation,
                reply,
            }))
            .await
            .map_err(|_| crate::Error::Invalid("Quest item worker closed"))?;
        answer
            .await
            .map_err(|_| crate::Error::Invalid("Quest item worker closed"))?
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        // Accepted jobs survive response loss and drain before ownership release.
        let result = match self.operation {
            Operation::Items { profile, command } => {
                store
                    .mutate_items_with_quest_journal(
                        self.binding,
                        self.world,
                        &profile,
                        command,
                        &self.journal,
                    )
                    .await
            }
            Operation::Claim {
                profile,
                claim,
                expected_revision,
            } => {
                store
                    .claim_reward_with_quest_journal(
                        self.binding,
                        self.world,
                        &profile,
                        claim,
                        expected_revision,
                        &self.journal,
                    )
                    .await
            }
            Operation::Purchase { profile, purchase } => {
                store
                    .purchase_vendor_with_quest_journal(
                        self.binding,
                        self.world,
                        &profile,
                        &purchase,
                        &self.journal,
                    )
                    .await
            }
        };
        if let Ok(applied) = &result {
            status.inventory_committed = status
                .inventory_committed
                .saturating_add(u64::from(!applied.replayed));
            status.inventory_replayed = status
                .inventory_replayed
                .saturating_add(u64::from(applied.replayed));
        }
        respond(result, self.reply, status)
    }
}
