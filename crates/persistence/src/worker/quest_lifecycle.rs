//! Explicit quest actions and trusted kills use the existing bounded FIFO queue.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{
    Binding, EarnedQuestLifecycle, ProgressionCatalog, QuestLifecycleApplied,
    QuestLifecycleCatalog, Result, Store,
};
use theia_core::{
    progression::KillEvent,
    quests::lifecycle::{Command, State},
};
use tokio::sync::oneshot;

pub(super) enum Request {
    Ensure {
        binding: Binding,
        world: [u8; 32],
        profile: QuestLifecycleCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Load {
        binding: Binding,
        world: [u8; 32],
        profile: QuestLifecycleCatalog,
        reply: oneshot::Sender<Result<State>>,
    },
    Command {
        binding: Binding,
        world: [u8; 32],
        profile: QuestLifecycleCatalog,
        command: Command,
        reply: oneshot::Sender<Result<QuestLifecycleApplied>>,
    },
    CommandExperience {
        binding: Binding,
        world: [u8; 32],
        profile: QuestLifecycleCatalog,
        xp: ProgressionCatalog,
        command: Command,
        reply: oneshot::Sender<Result<EarnedQuestLifecycle>>,
    },
    Award {
        binding: Binding,
        world: [u8; 32],
        profile: QuestLifecycleCatalog,
        xp: ProgressionCatalog,
        event: KillEvent,
        reply: oneshot::Sender<Result<EarnedQuestLifecycle>>,
    },
}
impl Publisher {
    pub async fn ensure_quest_lifecycle(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &QuestLifecycleCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::QuestLifecycle(Request::Ensure {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn load_quest_lifecycle(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &QuestLifecycleCatalog,
    ) -> Result<State> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::QuestLifecycle(Request::Load {
                binding,
                world,
                profile: profile.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
    /// Caller establishes NPC entitlement before enqueue and retains accepted
    /// work through logout. Queue acceptance is never a commit acknowledgement.
    pub async fn command_quest_lifecycle(
        &self,
        binding: Binding,
        world: [u8; 32],
        profile: &QuestLifecycleCatalog,
        command: Command,
    ) -> Result<QuestLifecycleApplied> {
        valid_world(world)?;
        if profile.turn_in_xp().is_some() {
            return Err(crate::Error::Invalid(
                "Rewarded quests require atomic experience command",
            ));
        }
        if command.expected_revision > 101 {
            return Err(crate::Error::QuestLifecycleCommand(
                theia_core::quests::lifecycle::Error::StaleRevision,
            ));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::QuestLifecycle(Request::Command {
                binding,
                world,
                profile: profile.clone(),
                command,
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn command_quest_lifecycle_with_experience(
        &self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        profile: &QuestLifecycleCatalog,
        command: Command,
    ) -> Result<EarnedQuestLifecycle> {
        valid_world(world)?;
        profile.check_progression(xp)?;
        if profile.turn_in_xp().is_none() {
            return Err(crate::Error::Invalid("Completion XP recipe is required"));
        }
        if command.expected_revision > 101 {
            return Err(crate::Error::QuestLifecycleCommand(
                theia_core::quests::lifecycle::Error::StaleRevision,
            ));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::QuestLifecycle(Request::CommandExperience {
                binding,
                world,
                profile: profile.clone(),
                xp: xp.clone(),
                command,
                reply,
            }),
            answer,
        )
        .await
    }
    pub async fn award_progression_and_quest_lifecycle(
        &self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        profile: &QuestLifecycleCatalog,
        event: &KillEvent,
    ) -> Result<EarnedQuestLifecycle> {
        valid_world(world)?;
        profile.check_progression(xp)?;
        if xp.profile().kill_experience(&event.creature).is_none() {
            return Err(crate::Error::ProgressionCommand(
                theia_core::progression::Error::UnknownCreature,
            ));
        }
        let (reply, answer) = oneshot::channel();
        self.inventory_request(
            inventory::Request::QuestLifecycle(Request::Award {
                binding,
                world,
                profile: profile.clone(),
                xp: xp.clone(),
                event: event.clone(),
                reply,
            }),
            answer,
        )
        .await
    }
}
impl WorkerStatus {
    fn record_lifecycle(&mut self, result: &Result<QuestLifecycleApplied>, action: bool) {
        if let Ok(r) = result {
            self.lifecycle_actions = self
                .lifecycle_actions
                .saturating_add(u64::from(action && r.changed));
            self.lifecycle_turn_ins = self
                .lifecycle_turn_ins
                .saturating_add(u64::from(r.turned_in_now));
            self.lifecycle_advances = self
                .lifecycle_advances
                .saturating_add(u64::from(!action && r.changed));
            self.lifecycle_replayed = self
                .lifecycle_replayed
                .saturating_add(u64::from(r.replayed));
            self.lifecycle_noops = self
                .lifecycle_noops
                .saturating_add(u64::from(!r.changed && !r.replayed));
        } else {
            self.lifecycle_rejected = self.lifecycle_rejected.saturating_add(1);
        }
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
                let result = store.ensure_quest_lifecycle(binding, world, &profile).await;
                status.lifecycle_ensures = status
                    .lifecycle_ensures
                    .saturating_add(u64::from(result.is_ok()));
                status.lifecycle_rejected = status
                    .lifecycle_rejected
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
                let result = store.load_quest_lifecycle(binding, world, &profile).await;
                status.lifecycle_loads = status
                    .lifecycle_loads
                    .saturating_add(u64::from(result.is_ok()));
                status.lifecycle_rejected = status
                    .lifecycle_rejected
                    .saturating_add(u64::from(result.is_err()));
                respond(result, reply, status)
            }
            Self::Command {
                binding,
                world,
                profile,
                command,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .command_quest_lifecycle(binding, world, &profile, command)
                    .await;
                status.record_lifecycle(&result, true);
                respond(result, reply, status)
            }
            Self::Award {
                binding,
                world,
                profile,
                xp,
                event,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .award_progression_and_quest_lifecycle(binding, world, &xp, &profile, &event)
                    .await;
                if let Ok(r) = &result {
                    status.record_experience(&r.experience);
                    status.record_lifecycle(&Ok(r.quest.clone()), false);
                } else {
                    status.lifecycle_rejected = status.lifecycle_rejected.saturating_add(1);
                    status.progression_rejected = status.progression_rejected.saturating_add(1);
                }
                respond(result, reply, status)
            }
            Self::CommandExperience {
                binding,
                world,
                profile,
                xp,
                command,
                reply,
            } => {
                if reply.is_closed() {
                    return Ok(());
                }
                let result = store
                    .command_quest_lifecycle_with_experience(binding, world, &xp, &profile, command)
                    .await;
                if let Ok(r) = &result {
                    status.record_lifecycle(&Ok(r.quest.clone()), true);
                    if command.action == theia_core::quests::lifecycle::Action::TurnIn {
                        status.record_experience(&r.experience);
                        status.lifecycle_xp_awards = status
                            .lifecycle_xp_awards
                            .saturating_add(u64::from(r.experience.granted > 0));
                        status.lifecycle_xp_units = status
                            .lifecycle_xp_units
                            .saturating_add(u64::from(r.experience.granted));
                        status.lifecycle_xp_capped = status.lifecycle_xp_capped.saturating_add(
                            u64::from(r.experience.granted == 0 && !r.experience.replayed),
                        );
                    }
                } else {
                    status.lifecycle_rejected = status.lifecycle_rejected.saturating_add(1);
                    status.progression_rejected = status.progression_rejected.saturating_add(1);
                }
                respond(result, reply, status)
            }
        }
    }
}
