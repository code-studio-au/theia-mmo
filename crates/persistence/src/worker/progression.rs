//! Earned XP uses the existing bounded critical-character queue and SQL owner.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{Binding, Error, ProgressionApplied, ProgressionCatalog, Result, Store};
use theia_core::progression::{KillEvent, State};
use tokio::sync::oneshot;
pub(super) enum Request {
    Ensure {
        binding: Binding,
        world: [u8; 32],
        profile: ProgressionCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Load {
        binding: Binding,
        world: [u8; 32],
        profile: ProgressionCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Award {
        binding: Binding,
        world: [u8; 32],
        profile: ProgressionCatalog,
        event: KillEvent,
        reply: oneshot::Sender<Result<ProgressionApplied>>,
    },
}
impl Publisher {
    pub async fn ensure_progression(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &ProgressionCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Progression(Request::Ensure {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn load_progression(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &ProgressionCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Progression(Request::Load {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    /// Trusted earned event only. No simulation or client awaits this RPC;
    /// character retirement must retain accepted work until it finishes.
    pub async fn award_experience(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &ProgressionCatalog,
        event: &KillEvent,
    ) -> Result<ProgressionApplied> {
        valid_world(world)?;
        if profile.profile().kill_experience(&event.creature).is_none() {
            return Err(Error::ProgressionCommand(
                theia_core::progression::Error::UnknownCreature,
            ));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Progression(Request::Award {
                binding,
                world,
                profile: profile.clone(),
                event: event.clone(),
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
            Self::Ensure {
                binding,
                world,
                profile,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store.ensure_progression(binding, world, &profile).await;
                status.progression_ensures = status
                    .progression_ensures
                    .saturating_add(u64::from(result.is_ok()));
                status.progression_rejected = status
                    .progression_rejected
                    .saturating_add(u64::from(result.is_err()));
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
                let result = store.load_progression(binding, world, &profile).await;
                status.progression_loads = status
                    .progression_loads
                    .saturating_add(u64::from(result.is_ok()));
                status.progression_rejected = status
                    .progression_rejected
                    .saturating_add(u64::from(result.is_err()));
                respond(result, reply, status)
            }
            Self::Award {
                binding,
                world,
                profile,
                event,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .award_experience(binding, world, &profile, &event)
                    .await;
                if let Ok(applied) = &result {
                    status.record_experience(applied);
                }
                status.progression_rejected = status
                    .progression_rejected
                    .saturating_add(u64::from(result.is_err()));
                respond(result, reply, status)
            }
        }
    }
}

impl WorkerStatus {
    pub(super) fn record_experience(&mut self, applied: &ProgressionApplied) {
        self.experience_committed = self
            .experience_committed
            .saturating_add(u64::from(applied.granted > 0));
        self.experience_replayed = self
            .experience_replayed
            .saturating_add(u64::from(applied.replayed));
        self.experience_capped = self
            .experience_capped
            .saturating_add(u64::from(applied.granted == 0 && !applied.replayed));
    }
}
