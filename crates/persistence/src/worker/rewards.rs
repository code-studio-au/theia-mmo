//! Reward requests share the existing bounded critical-item queue/SQL owner.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{Binding, ClaimId, Error, InventoryApplied, Result, RewardCatalog, RewardClaim, Store};
use theia_core::{content::ContentId, inventory::MAX_REVISION};
use tokio::sync::oneshot;
pub(super) enum Request {
    Offer {
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        reward: ContentId,
        reply: oneshot::Sender<Result<RewardClaim>>,
    },
    List {
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        reply: oneshot::Sender<Result<Vec<RewardClaim>>>,
    },
    Claim {
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
        reply: oneshot::Sender<Result<InventoryApplied>>,
    },
}
impl Publisher {
    /// Trusted qualifying-event adapter only. Queue acceptance is not availability.
    pub async fn offer_reward(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
    ) -> Result<RewardClaim> {
        valid_world(world)?;
        if profile.catalog().definition(reward).is_none() {
            return Err(Error::RewardMissing);
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::Offer {
                binding,
                world,
                profile: profile.clone(),
                reward: reward.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn list_reward_claims(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
    ) -> Result<Vec<RewardClaim>> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::List {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn claim_reward(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
    ) -> Result<InventoryApplied> {
        valid_world(world)?;
        if expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::Claim {
                binding,
                world,
                profile: profile.clone(),
                claim,
                expected_revision,
                reply,
            }),
            answer,
        )
        .await
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        match self {
            Self::Offer {
                binding,
                world,
                profile,
                reward,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store.offer_reward(binding, world, &profile, &reward).await;
                status.reward_offers = status
                    .reward_offers
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::List {
                binding,
                world,
                profile,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store.list_reward_claims(binding, world, &profile).await;
                status.reward_lists = status
                    .reward_lists
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::Claim {
                binding,
                world,
                profile,
                claim,
                expected_revision,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .claim_reward(binding, world, &profile, claim, expected_revision)
                    .await;
                if let Ok(applied) = &result {
                    status.reward_committed = status
                        .reward_committed
                        .saturating_add(u64::from(!applied.replayed));
                    status.reward_replayed = status
                        .reward_replayed
                        .saturating_add(u64::from(applied.replayed));
                }
                respond(result, reply, status)
            }
        }
    }
}
