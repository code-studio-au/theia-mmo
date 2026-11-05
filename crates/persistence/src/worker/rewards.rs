//! Reward requests share the existing bounded critical-item queue/SQL owner.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{Binding, ClaimId, Error, InventoryApplied, Result, RewardCatalog, RewardClaim, Store};
use theia_core::{
    content::ContentId, coordinates::GlobalPosition, inventory::MAX_REVISION,
    progression::KillEvent,
};
use tokio::sync::oneshot;
pub(super) enum Request {
    Offer {
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        reward: ContentId,
        source: Option<(KillEvent, GlobalPosition)>,
        reply: oneshot::Sender<Result<RewardClaim>>,
    },
    Roll {
        quest_loot: bool,
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        reward: ContentId,
        event: KillEvent,
        position: GlobalPosition,
        reply: oneshot::Sender<Result<Option<RewardClaim>>>,
    },
    List {
        quest_loot: bool,
        position: Option<GlobalPosition>,
        binding: Binding,
        world: [u8; 32],
        profile: RewardCatalog,
        reply: oneshot::Sender<Result<Vec<RewardClaim>>>,
    },
    Claim {
        quest_loot: bool,
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
        profile.check_runtime_admission()?;
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
                source: None,
                reply,
            }),
            answer,
        )
        .await
    }
    /// Accepted trusted loot offers survive response loss and drain on shutdown.
    pub async fn offer_reward_for_kill(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        event: &KillEvent,
        position: GlobalPosition,
    ) -> Result<RewardClaim> {
        valid_world(world)?;
        profile.check_runtime_admission()?;
        profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?
            .occurrence(event)
            .map_err(Error::RewardCommand)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::Offer {
                binding,
                world,
                profile: profile.clone(),
                reward: reward.clone(),
                source: Some((event.clone(), position)),
                reply,
            }),
            answer,
        )
        .await
    }
    /// Accepted roll jobs retain empty receipts and drain after response loss.
    pub async fn roll_reward_for_kill(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        event: &KillEvent,
        position: GlobalPosition,
    ) -> Result<Option<RewardClaim>> {
        valid_world(world)?;
        profile.check_runtime_admission()?;
        profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?
            .occurrence(event)
            .map_err(Error::RewardCommand)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::Roll {
                quest_loot: false,
                binding,
                world,
                profile: profile.clone(),
                reward: reward.clone(),
                event: event.clone(),
                position,
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
        profile.check_runtime_admission()?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::List {
                quest_loot: false,
                position: None,
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    /// Trusted server-pose interest, independent from pickup permission.
    pub async fn list_reward_claims_nearby(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        position: GlobalPosition,
    ) -> Result<Vec<RewardClaim>> {
        valid_world(world)?;
        profile.check_runtime_admission()?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::List {
                quest_loot: false,
                position: Some(position),
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
        profile.check_runtime_admission()?;
        if expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::Claim {
                quest_loot: false,
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
impl Publisher {
    fn queue_quest_loot(&self, request: Request) -> Result<()> {
        self.inventory
            .try_send(inventory::Request::Reward(request))
            .map_err(|error| match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => Error::InventoryBusy,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    Error::Invalid("Quest loot worker closed")
                }
            })
    }
    fn quest_loot_profile(profile: &RewardCatalog) -> Result<()> {
        if profile.quest_journal().is_none() || !profile.catalog().has_quest_required() {
            return Err(Error::Invalid(
                "Quest loot requires the bound conditional profile",
            ));
        }
        Ok(())
    }
    async fn wait_quest_loot<T: Send>(answer: oneshot::Receiver<Result<T>>) -> Result<T> {
        tokio::time::timeout(std::time::Duration::from_secs(5), answer)
            .await
            .map_err(|_| Error::Timeout)?
            .map_err(|_| Error::Invalid("Quest loot worker closed"))?
    }
    /// Accepted generation retains its first rights/ticket even after reply loss.
    pub fn enqueue_quest_loot_generation(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        event: &KillEvent,
        position: GlobalPosition,
    ) -> Result<oneshot::Receiver<Result<Option<RewardClaim>>>> {
        valid_world(world)?;
        Self::quest_loot_profile(profile)?;
        profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?
            .occurrence(event)
            .map_err(Error::RewardCommand)?;
        let (reply, answer) = oneshot::channel();
        self.queue_quest_loot(Request::Roll {
            quest_loot: true,
            binding,
            world,
            profile: profile.clone(),
            reward: reward.clone(),
            event: event.clone(),
            position,
            reply,
        })?;
        Ok(answer)
    }
    pub async fn generate_quest_loot_for_kill(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        event: &KillEvent,
        position: GlobalPosition,
    ) -> Result<Option<RewardClaim>> {
        valid_world(world)?;
        Self::quest_loot_profile(profile)?;
        profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?
            .occurrence(event)
            .map_err(Error::RewardCommand)?;
        let permit = self
            .inventory
            .reserve()
            .await
            .map_err(|_| Error::Invalid("Quest loot worker closed"))?;
        let (reply, answer) = oneshot::channel();
        permit.send(inventory::Request::Reward(Request::Roll {
            quest_loot: true,
            binding,
            world,
            profile: profile.clone(),
            reward: reward.clone(),
            event: event.clone(),
            position,
            reply,
        }));
        Self::wait_quest_loot(answer).await
    }
    pub async fn list_quest_reward_claims_nearby(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        position: GlobalPosition,
    ) -> Result<Vec<RewardClaim>> {
        valid_world(world)?;
        Self::quest_loot_profile(profile)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Reward(Request::List {
                quest_loot: true,
                position: Some(position),
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    /// An accepted pickup drains even if the caller no longer awaits its receipt.
    pub fn enqueue_quest_loot_pickup(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
    ) -> Result<oneshot::Receiver<Result<InventoryApplied>>> {
        valid_world(world)?;
        Self::quest_loot_profile(profile)?;
        if expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let (reply, answer) = oneshot::channel();
        self.queue_quest_loot(Request::Claim {
            quest_loot: true,
            binding,
            world,
            profile: profile.clone(),
            claim,
            expected_revision,
            reply,
        })?;
        Ok(answer)
    }
    pub async fn pickup_quest_reward(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
    ) -> Result<InventoryApplied> {
        valid_world(world)?;
        Self::quest_loot_profile(profile)?;
        if expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let permit = self
            .inventory
            .reserve()
            .await
            .map_err(|_| Error::Invalid("Quest loot worker closed"))?;
        let (reply, answer) = oneshot::channel();
        permit.send(inventory::Request::Reward(Request::Claim {
            quest_loot: true,
            binding,
            world,
            profile: profile.clone(),
            claim,
            expected_revision,
            reply,
        }));
        Self::wait_quest_loot(answer).await
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
                source,
                reply,
            } => {
                // Availability is critical retained work once accepted, even if
                // its caller stops waiting before this job reaches the store.
                let result = match source {
                    Some((event, position)) => {
                        store
                            .offer_reward_for_kill(
                                binding, world, &profile, &reward, &event, position,
                            )
                            .await
                    }
                    None => store.offer_reward(binding, world, &profile, &reward).await,
                };
                status.reward_offers = status
                    .reward_offers
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::Roll {
                quest_loot,
                binding,
                world,
                profile,
                reward,
                event,
                position,
                reply,
            } => {
                let result = if quest_loot {
                    store
                        .generate_quest_loot_for_kill(
                            binding, world, &profile, &reward, &event, position,
                        )
                        .await
                } else {
                    store
                        .roll_reward_for_kill(binding, world, &profile, &reward, &event, position)
                        .await
                };
                status.reward_offers = status
                    .reward_offers
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::List {
                quest_loot,
                position,
                binding,
                world,
                profile,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = match (quest_loot, position) {
                    (true, Some(position)) => {
                        store
                            .list_quest_reward_claims_nearby(binding, world, &profile, position)
                            .await
                    }
                    (true, None) => {
                        Err(Error::Invalid("Quest loot listing requires a server pose"))
                    }
                    (false, Some(position)) => {
                        store
                            .list_reward_claims_nearby(binding, world, &profile, position)
                            .await
                    }
                    (false, None) => store.list_reward_claims(binding, world, &profile).await,
                };
                status.reward_lists = status
                    .reward_lists
                    .saturating_add(u64::from(result.is_ok()));
                respond(result, reply, status)
            }
            Self::Claim {
                quest_loot,
                binding,
                world,
                profile,
                claim,
                expected_revision,
                reply,
            } => {
                if reply.is_closed() && !quest_loot {
                    return Ok(());
                }
                let result = if quest_loot {
                    store
                        .pickup_quest_reward(binding, world, &profile, claim, expected_revision)
                        .await
                } else {
                    store
                        .claim_reward(binding, world, &profile, claim, expected_revision)
                        .await
                };
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
