//! Bounded journal jobs on the shared critical FIFO. Mutation acceptance is
//! synchronous; dropping a response never cancels an accepted durable intent.
use super::{
    Publisher, WorkerStatus,
    inventory::{self, respond, valid_world},
};
use crate::{
    Binding, Error, ProgressionCatalog, QuestJournalApplied, QuestJournalCatalog,
    QuestJournalCommand, Result, Store,
};
use std::num::NonZeroU128;
use theia_core::quests::catalog::EventKey;
use tokio::sync::oneshot;

enum Operation {
    Ensure,
    Load,
    Event {
        id: NonZeroU128,
        event: EventKey,
    },
    Command {
        xp: ProgressionCatalog,
        command: QuestJournalCommand,
    },
}
pub(super) struct Request {
    binding: Binding,
    world: [u8; 32],
    catalog: QuestJournalCatalog,
    operation: Operation,
    reply: oneshot::Sender<Result<QuestJournalApplied>>,
}
impl Publisher {
    /// Trusted admission initialization, outside the simulation step.
    pub async fn ensure_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
    ) -> Result<QuestJournalApplied> {
        self.queue_journal(binding, world, catalog, Operation::Ensure)?
            .await
            .map_err(|_| Error::Invalid("Quest journal worker closed"))?
    }
    pub async fn load_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
    ) -> Result<QuestJournalApplied> {
        self.queue_journal(binding, world, catalog, Operation::Load)?
            .await
            .map_err(|_| Error::Invalid("Quest journal worker closed"))?
    }
    /// Caller must establish NPC/actor entitlement before acceptance and retain
    /// the character's ownership until this receipt resolves. Ok means queued,
    /// never committed; only the returned response can acknowledge SQL success.
    pub fn enqueue_quest_journal_command(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        xp: &ProgressionCatalog,
        command: QuestJournalCommand,
    ) -> Result<oneshot::Receiver<Result<QuestJournalApplied>>> {
        catalog.check_progression(xp)?;
        if command.expected_revision > i64::MAX as u64 {
            return Err(Error::QuestJournalCommand(
                theia_core::quests::journal::Error::StaleRevision,
            ));
        }
        self.queue_journal(
            binding,
            world,
            catalog,
            Operation::Command {
                xp: xp.clone(),
                command,
            },
        )
    }
    /// Trusted normalized occurrence with its original stable receipt identity.
    /// No XP kill recipe is required. Caller establishes credit eligibility.
    pub fn enqueue_quest_event(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        id: NonZeroU128,
        event: EventKey,
    ) -> Result<oneshot::Receiver<Result<QuestJournalApplied>>> {
        self.queue_journal(binding, world, catalog, Operation::Event { id, event })
    }
    /// Retained async owner path. Wait for bounded FIFO space without losing the
    /// original intent; never call from a fixed simulation step.
    pub async fn command_quest_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        xp: &ProgressionCatalog,
        command: QuestJournalCommand,
    ) -> Result<QuestJournalApplied> {
        catalog.check_progression(xp)?;
        if command.expected_revision > i64::MAX as u64 {
            return Err(Error::QuestJournalCommand(
                theia_core::quests::journal::Error::StaleRevision,
            ));
        }
        self.wait_journal(
            binding,
            world,
            catalog,
            Operation::Command {
                xp: xp.clone(),
                command,
            },
        )
        .await
    }
    pub async fn record_quest_event(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        id: NonZeroU128,
        event: EventKey,
    ) -> Result<QuestJournalApplied> {
        self.wait_journal(binding, world, catalog, Operation::Event { id, event })
            .await
    }
    async fn wait_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        operation: Operation,
    ) -> Result<QuestJournalApplied> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory
            .send(inventory::Request::QuestJournal(Request {
                binding,
                world,
                catalog: catalog.clone(),
                operation,
                reply,
            }))
            .await
            .map_err(|_| Error::Invalid("Quest journal worker closed"))?;
        answer
            .await
            .map_err(|_| Error::Invalid("Quest journal worker closed"))?
    }
    fn queue_journal(
        &self,
        binding: Binding,
        world: [u8; 32],
        catalog: &QuestJournalCatalog,
        operation: Operation,
    ) -> Result<oneshot::Receiver<Result<QuestJournalApplied>>> {
        valid_world(world)?;
        let (reply, answer) = oneshot::channel();
        self.inventory
            .try_send(inventory::Request::QuestJournal(Request {
                binding,
                world,
                catalog: catalog.clone(),
                operation,
                reply,
            }))
            .map_err(|e| match e {
                tokio::sync::mpsc::error::TrySendError::Full(_) => Error::InventoryBusy,
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    Error::Invalid("Quest journal worker closed")
                }
            })?;
        Ok(answer)
    }
}
impl Request {
    pub(super) async fn run(self, store: &mut Store, status: &mut WorkerStatus) -> Result<()> {
        // Intentionally no reply.is_closed check: accepted occurrences/actions
        // remain work even if a session disconnects before this job begins.
        let (kind, result) = match self.operation {
            Operation::Ensure => (
                0,
                store
                    .ensure_quest_journal(self.binding, self.world, &self.catalog)
                    .await,
            ),
            Operation::Load => (
                1,
                store
                    .load_quest_journal(self.binding, self.world, &self.catalog)
                    .await,
            ),
            Operation::Command { xp, command } => (
                2,
                store
                    .command_quest_journal(self.binding, self.world, &self.catalog, &xp, &command)
                    .await,
            ),
            Operation::Event { id, event } => (
                3,
                store
                    .record_quest_event(self.binding, self.world, &self.catalog, id, &event)
                    .await,
            ),
        };
        match &result {
            Ok(applied) => {
                let counter = match kind {
                    0 => &mut status.journal_ensures,
                    1 => &mut status.journal_loads,
                    2 => &mut status.journal_commands,
                    _ => &mut status.journal_events,
                };
                *counter = counter.saturating_add(1);
                status.journal_replayed = status
                    .journal_replayed
                    .saturating_add(u64::from(applied.replayed));
                status.journal_experience_units = status
                    .journal_experience_units
                    .saturating_add(u64::from(applied.experience_granted));
            }
            Err(_) => status.journal_rejected = status.journal_rejected.saturating_add(1),
        }
        respond(result, self.reply, status)
    }
}
