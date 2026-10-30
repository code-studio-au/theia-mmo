//! Bounded critical item requests, separate from latest-only pose checkpoints.
use super::{Publisher, WorkerStatus};
use crate::{Binding, Error, InventoryApplied, InventoryCatalog, Result, Store};
use theia_core::inventory::{
    Bag, Command, EquipmentCommand, ItemCommand as ItemIntent, MAX_REVISION,
};
use tokio::sync::oneshot;
pub(super) enum Request {
    Vendor(super::vendors::Request),
    Progression(super::progression::Request),
    Quest(super::quests::Request),
    Reward(super::rewards::Request),
    Ensure {
        binding: Binding,
        world: [u8; 32],
        profile: InventoryCatalog,
        seed: Box<Bag>,
        reply: oneshot::Sender<Result<Bag>>,
    },
    Load {
        binding: Binding,
        world: [u8; 32],
        profile: InventoryCatalog,
        reply: oneshot::Sender<Result<Bag>>,
    },
    Mutate {
        binding: Binding,
        world: [u8; 32],
        profile: InventoryCatalog,
        command: ItemIntent,
        reply: oneshot::Sender<Result<InventoryApplied>>,
    },
}
impl Publisher {
    /// Trusted authority seed, never a client contents upload. Completion is a
    /// database acknowledgement; queue acceptance is not observable as success.
    pub async fn ensure_inventory(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        seed: &Bag,
    ) -> Result<Bag> {
        valid_world(world)?;
        if seed.revision() != 0 || Bag::from_json(profile.catalog(), &seed.to_json()).is_err() {
            return Err(Error::Invalid("Invalid authority inventory seed"));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            Request::Ensure {
                binding,
                world,
                profile: profile.clone(),
                seed: Box::new(seed.clone()),
                reply,
            },
            answer,
        )
        .await
    }
    pub async fn load_inventory(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
    ) -> Result<Bag> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            Request::Load {
                binding,
                world,
                profile: profile.clone(),
                reply,
            },
            answer,
        )
        .await
    }
    pub async fn mutate_inventory(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: Command,
    ) -> Result<InventoryApplied> {
        self.mutate_items(binding, world, profile, ItemIntent::Backpack(command))
            .await
    }
    pub async fn mutate_equipment(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: EquipmentCommand,
    ) -> Result<InventoryApplied> {
        self.mutate_items(binding, world, profile, ItemIntent::Equipment(command))
            .await
    }
    async fn mutate_items(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &InventoryCatalog,
        command: ItemIntent,
    ) -> Result<InventoryApplied> {
        valid_world(world)?;
        if command.expected_revision() > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            Request::Mutate {
                binding,
                world,
                profile: profile.clone(),
                command,
                reply,
            },
            answer,
        )
        .await
    }
    pub(super) async fn inventory_request<T: Send>(
        &self,
        request: Request,
        answer: oneshot::Receiver<Result<T>>,
    ) -> Result<T> {
        self.inventory
            .try_send(request)
            .map_err(|error| match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => Error::InventoryBusy,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    Error::Invalid("Inventory worker closed")
                }
            })?;
        tokio::time::timeout(std::time::Duration::from_secs(5), answer)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Invalid("Inventory worker closed"))?
    }
}
pub(super) fn valid_world(world: [u8; 32]) -> Result<()> {
    if world == [0; 32] {
        Err(Error::Invalid("World content identity must be nonzero"))
    } else {
        Ok(())
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        match self {
            Self::Vendor(request) => request.run(store, status).await,
            Self::Progression(request) => request.run(store, status).await,
            Self::Quest(request) => request.run(store, status).await,
            Self::Reward(request) => request.run(store, status).await,
            Self::Ensure {
                binding,
                world,
                profile,
                seed,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .ensure_inventory(binding, world, &profile, &seed)
                    .await;
                status.inventory_ensures = status
                    .inventory_ensures
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::Load {
                binding,
                world,
                profile,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store.load_inventory(binding, world, &profile).await;
                status.inventory_loads = status
                    .inventory_loads
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::Mutate {
                binding,
                world,
                profile,
                command,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store.mutate_items(binding, world, &profile, command).await;
                if let Ok(applied) = &result {
                    status.inventory_committed = status
                        .inventory_committed
                        .saturating_add(u64::from(!applied.replayed));
                    status.inventory_replayed = status
                        .inventory_replayed
                        .saturating_add(u64::from(applied.replayed));
                }
                respond(result, reply, status)
            }
        }
    }
}
pub(super) fn respond<T>(
    result: Result<T>,
    reply: oneshot::Sender<Result<T>>,
    status: &mut WorkerStatus,
) -> Result<()> {
    status.processed_inventory_requests = status.processed_inventory_requests.saturating_add(1);
    let fatal = match &result {
        Ok(_) => false,
        Err(Error::VendorCommand(
            theia_core::vendors::Error::InsufficientCost
            | theia_core::vendors::Error::NoCapacity
            | theia_core::vendors::Error::UnknownOffer
            | theia_core::vendors::Error::StaleRevision
            | theia_core::vendors::Error::RevisionExhausted,
        )) => {
            status.inventory_rejected = status.inventory_rejected.saturating_add(1);
            false
        }
        Err(
            Error::RewardMissing
            | Error::RewardConsumed
            | Error::RewardCommand(
                theia_core::rewards::Error::NoCapacity
                | theia_core::rewards::Error::StaleRevision
                | theia_core::rewards::Error::RevisionExhausted,
            ),
        ) => {
            status.inventory_rejected = status.inventory_rejected.saturating_add(1);
            status.reward_rejected = status.reward_rejected.saturating_add(1);
            false
        }
        Err(
            Error::ProgressionMissing
            | Error::ProgressionConflict
            | Error::ProgressionCommand(_)
            | Error::QuestMissing
            | Error::QuestConflict
            | Error::QuestCommand(_),
        ) => {
            status.inventory_rejected = status.inventory_rejected.saturating_add(1);
            false
        }
        Err(
            Error::Ownership
            | Error::WorldMismatch
            | Error::CharacterMissing
            | Error::InventoryMissing
            | Error::InventoryStale
            | Error::InventoryConflict
            | Error::InventoryCommand(_),
        ) => {
            status.inventory_rejected = status.inventory_rejected.saturating_add(1);
            false
        }
        Err(error) => {
            status.failed = true;
            status.error = Some(error.to_string());
            true
        }
    };
    let _ = reply.send(result);
    if fatal {
        Err(Error::Invalid("Persistent inventory operation failed"))
    } else {
        Ok(())
    }
}
