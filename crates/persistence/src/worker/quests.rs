//! Objective/XP bundles use the existing bounded critical worker, never a tick.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{Binding, EarnedProgression, ProgressionCatalog, QuestCatalog, Result, Store};
use theia_core::{progression::KillEvent, quests::State};
use tokio::sync::oneshot;
pub(super) enum Request {
    Ensure {
        binding: Binding,
        world: [u8; 32],
        profile: QuestCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Load {
        binding: Binding,
        world: [u8; 32],
        profile: QuestCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Award {
        binding: Binding,
        world: [u8; 32],
        xp: ProgressionCatalog,
        quest: QuestCatalog,
        event: KillEvent,
        reply: oneshot::Sender<Result<EarnedProgression>>,
    },
}
impl Publisher {
    pub async fn ensure_quest(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &QuestCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Quest(Request::Ensure {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn load_quest(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &QuestCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Quest(Request::Load {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    /// Trusted earned fact only. Both effects commit before this RPC returns;
    /// retirement must retain accepted executing/queued work through completion.
    pub async fn award_progression_and_quest(
        &self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        quest: &QuestCatalog,
        event: &KillEvent,
    ) -> Result<EarnedProgression> {
        valid_world(world)?;
        quest.check_progression(xp)?;
        if xp.profile().kill_experience(&event.creature).is_none() {
            return Err(crate::Error::ProgressionCommand(
                theia_core::progression::Error::UnknownCreature,
            ));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::Quest(Request::Award {
                binding,
                world,
                xp: xp.clone(),
                quest: quest.clone(),
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
                let result = store.ensure_quest(binding, world, &profile).await;
                status.quest_ensures = status
                    .quest_ensures
                    .saturating_add(u64::from(result.is_ok()));
                status.quest_rejected = status
                    .quest_rejected
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
                let result = store.load_quest(binding, world, &profile).await;
                status.quest_loads = status.quest_loads.saturating_add(u64::from(result.is_ok()));
                status.quest_rejected = status
                    .quest_rejected
                    .saturating_add(u64::from(result.is_err()));
                respond(result, reply, status)
            }
            Self::Award {
                binding,
                world,
                xp,
                quest,
                event,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .award_progression_and_quest(binding, world, &xp, &quest, &event)
                    .await;
                if let Ok(applied) = &result {
                    status.record_experience(&applied.experience);
                    status.quest_advances = status
                        .quest_advances
                        .saturating_add(u64::from(applied.quest.advanced));
                    status.quest_completions = status
                        .quest_completions
                        .saturating_add(u64::from(applied.quest.completed_now));
                    status.quest_replayed = status
                        .quest_replayed
                        .saturating_add(u64::from(applied.quest.replayed));
                    status.quest_noops = status.quest_noops.saturating_add(u64::from(
                        !applied.quest.advanced && !applied.quest.replayed,
                    ));
                }
                status.quest_rejected = status
                    .quest_rejected
                    .saturating_add(u64::from(result.is_err()));
                status.progression_rejected = status
                    .progression_rejected
                    .saturating_add(u64::from(result.is_err()));
                respond(result, reply, status)
            }
        }
    }
}
