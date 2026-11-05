//! Generation-bound durable progression owner. Accepted actions and earned facts
//! share one FIFO and drain before retirement releases the character slot.
use super::{GameAdmissions, Phase};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
use theia_core::{identity::EntityKey, progression::KillEvent};
use theia_network::{
    Result,
    quest_lifecycle_wire::{self as life, Rejection},
    renet_runtime::progression::{ActionReply, CAPACITY, Input, Observation, Port},
};
use theia_persistence::{Binding, Error, ProgressionCatalog, Publisher};
use tokio::sync::{mpsc, watch};

pub(super) struct Config {
    journal: Option<theia_persistence::QuestJournalCatalog>,
    lifecycle: Option<(theia_persistence::QuestLifecycleCatalog, Arc<life::Profile>)>,
    quest: Option<(
        theia_persistence::QuestCatalog,
        Arc<theia_network::quest_wire::Profile>,
    )>,
    profile: ProgressionCatalog,
    wire: Arc<theia_network::progression_wire::Profile>,
    publisher: Publisher,
}
impl Config {
    pub(super) fn admission_profiles(&self) -> ([u8; 32], Option<[u8; 32]>, Option<[u8; 32]>) {
        (
            self.profile.hash(),
            self.quest.as_ref().map(|(profile, _)| profile.hash()),
            self.lifecycle.as_ref().map(|(profile, _)| profile.hash()),
        )
    }

    pub(super) fn new(
        profile: ProgressionCatalog,
        wire: Arc<theia_network::progression_wire::Profile>,
        publisher: Publisher,
    ) -> Result<Self> {
        if profile.hash() != wire.hash() {
            return Err("XP store/wire profile mismatch".into());
        }
        Ok(Self {
            journal: None,
            lifecycle: None,
            quest: None,
            profile,
            wire,
            publisher,
        })
    }
    pub(super) fn set_quest(
        &mut self,
        quest: theia_persistence::QuestCatalog,
        wire: Arc<theia_network::quest_wire::Profile>,
    ) -> Result<()> {
        if self.quest.is_some()
            || self.journal.is_some()
            || self.lifecycle.is_some()
            || quest.hash() != wire.hash()
            || self
                .profile
                .profile()
                .kill_experience(quest.profile().creature())
                .is_none()
        {
            return Err("Objective store/wire/progression profile mismatch".into());
        }
        self.quest = Some((quest, wire));
        Ok(())
    }
    pub(super) fn set_lifecycle(
        &mut self,
        quest: theia_persistence::QuestLifecycleCatalog,
        wire: Arc<life::Profile>,
    ) -> Result<()> {
        if self.quest.is_some()
            || self.journal.is_some()
            || self.lifecycle.is_some()
            || quest.hash() != wire.hash()
            || self
                .profile
                .profile()
                .kill_experience(quest.profile().creature())
                .is_none()
        {
            return Err("Explicit quest store/wire/progression profile mismatch".into());
        }
        wire.check_progression(&self.wire)?;
        self.lifecycle = Some((quest, wire));
        Ok(())
    }
    pub(super) fn set_journal(
        &mut self,
        journal: theia_persistence::QuestJournalCatalog,
    ) -> Result<()> {
        if self.quest.is_some() || self.lifecycle.is_some() || self.journal.is_some() {
            return Err("Expanded journals require exclusive quest mode".into());
        }
        journal.check_progression(&self.profile)?;
        self.journal = Some(journal);
        Ok(())
    }
    pub(super) fn journal_hash(&self) -> Option<[u8; 32]> {
        self.journal.as_ref().map(|j| j.hash())
    }
    pub(super) fn start(
        &self,
        gate: Weak<GameAdmissions>,
        slot: usize,
        generation: u64,
        binding: Binding,
        world: [u8; 32],
        item_state: Option<watch::Receiver<theia_network::renet_runtime::inventory::Observation>>,
    ) -> Session {
        let (input, receiver) = mpsc::channel::<Input>(CAPACITY);
        let (publish, state) = watch::channel(Observation::default());
        let (stop, stopping) = watch::channel(false);
        let done = Arc::new(AtomicBool::new(false));
        let guard = Finished {
            gate: gate.clone(),
            done: done.clone(),
            handled: false,
        };
        let owner = Owner {
            gate,
            slot,
            generation,
            binding,
            world,
            profile: self.profile.clone(),
            journal: self.journal.clone(),
            quest: self.quest.as_ref().map(|q| q.0.clone()),
            lifecycle: self.lifecycle.as_ref().map(|q| q.0.clone()),
            publisher: self.publisher.clone(),
            sequence: 0,
            item_state,
        };
        let publication = publish.clone();
        tokio::spawn(async move {
            let mut guard = guard;
            let result = owner.run(receiver, stopping, &publish).await;
            if result.is_err()
                && let Some(gate) = guard.gate.upgrade()
            {
                gate.fail_closed();
            }
            publish.send_modify(|s| {
                s.failed = result.is_err();
                s.closed = true;
            });
            guard.handled = true;
        });
        Session {
            publish: publication,
            port: Port {
                journal: self.journal.as_ref().map(|q| q.hash()),
                lifecycle: self.lifecycle.as_ref().map(|q| q.1.clone()),
                quest: self.quest.as_ref().map(|q| q.1.clone()),
                profile: self.wire.clone(),
                input,
                state,
            },
            stop,
            done,
        }
    }
}
struct Owner {
    item_state: Option<watch::Receiver<theia_network::renet_runtime::inventory::Observation>>,
    gate: Weak<GameAdmissions>,
    slot: usize,
    generation: u64,
    binding: Binding,
    world: [u8; 32],
    profile: ProgressionCatalog,
    journal: Option<theia_persistence::QuestJournalCatalog>,
    quest: Option<theia_persistence::QuestCatalog>,
    lifecycle: Option<theia_persistence::QuestLifecycleCatalog>,
    publisher: Publisher,
    sequence: u64,
}
impl Owner {
    fn authorized(&self, actor: Option<EntityKey>) -> Result<bool> {
        let Some(gate) = self.gate.upgrade() else {
            return Ok(false);
        };
        let state = gate.lock()?;
        Ok(state.slots[self.slot].as_ref().is_some_and(|entry| {
            entry.generation == self.generation
                && entry.binding == self.binding
                && matches!(
                    entry.phase,
                    Phase::Consumed | Phase::Active | Phase::Retiring
                )
                && actor.is_none_or(|actor| entry.actor == Some(actor))
        }))
    }
    async fn run(
        mut self,
        mut receiver: mpsc::Receiver<Input>,
        mut stopping: watch::Receiver<bool>,
        publish: &watch::Sender<Observation>,
    ) -> Result<()> {
        if !self.authorized(None)? {
            return Err("Progression character ownership lost".into());
        }
        if self
            .journal
            .as_ref()
            .is_some_and(|j| j.inventory().is_some())
        {
            let state = self
                .item_state
                .as_mut()
                .ok_or("Item journal has no inventory initialization signal")?;
            let ready = state
                .wait_for(|s| s.bag.is_some() || s.failed || s.closed)
                .await
                .map_err(|_| "Inventory initialization closed")?;
            if ready.bag.is_none() || ready.failed {
                return Err("Inventory initialization failed".into());
            }
        }
        let initial = self
            .publisher
            .ensure_progression(self.binding, self.world, &self.profile)
            .await?;
        let objective = if let Some(quest) = &self.quest {
            Some(
                self.publisher
                    .ensure_quest(self.binding, self.world, quest)
                    .await?,
            )
        } else {
            None
        };
        let lifecycle = if let Some(quest) = &self.lifecycle {
            Some(
                self.publisher
                    .ensure_quest_lifecycle(self.binding, self.world, quest)
                    .await?,
            )
        } else {
            None
        };
        let journal = if let Some(catalog) = &self.journal {
            Some(
                self.publisher
                    .ensure_quest_journal(self.binding, self.world, catalog)
                    .await?,
            )
        } else {
            None
        };
        publish.send_modify(|s| {
            if s.progress
                .as_ref()
                .is_none_or(|xp| xp.revision() <= initial.revision())
            {
                s.progress = Some(initial);
            }
            s.quest = objective;
            s.lifecycle = lifecycle;
        });
        if let Some(saved) = journal {
            let catalog = self.journal.as_ref().unwrap();
            if catalog.inventory().is_some() {
                self.publish_items(saved, None)?;
            } else {
                let mut merged = Ok(());
                publish.send_modify(|s| {
                    merged = super::quest_items::merge_journal(s, catalog.hash(), saved, None)
                });
                merged?;
            }
        }
        loop {
            if *stopping.borrow() {
                receiver.close();
            }
            let input = if receiver.is_closed() {
                receiver.recv().await
            } else {
                tokio::select! {biased;
                    changed=stopping.changed()=>{if changed.is_err()||*stopping.borrow(){receiver.close();}continue;},
                    input=receiver.recv()=>input,
                }
            };
            let Some(input) = input else {
                break;
            };
            if !self.authorized(None)? {
                return Err("Accepted progression input lost ownership".into());
            }
            match input {
                Input::Journal(request) => self.journal(request, publish).await?,
                Input::Kill(event) => self.award(event, publish).await?,
                Input::QuestAction(request) => self.command(request, publish).await?,
            }
        }
        Ok(())
    }
    async fn journal(
        &mut self,
        request: theia_network::renet_runtime::progression::JournalInput,
        publish: &watch::Sender<Observation>,
    ) -> Result<()> {
        use theia_network::renet_runtime::progression::{
            JournalOperation, JournalRejection, JournalReply,
        };
        let catalog = self
            .journal
            .as_ref()
            .ok_or("Expanded journal owner is not configured")?;
        if request.catalog != catalog.hash()
            || !self.authorized(Some(request.actor))?
            || request.sequence <= self.sequence
            || request.sequence.saturating_sub(self.sequence) > 128
        {
            return Err("Accepted journal input lost actor/catalogue/sequence binding".into());
        }
        self.sequence = request.sequence;
        let command = matches!(&request.operation, JournalOperation::Command(_));
        // The bounded character owner retains this original intent while it
        // waits for SQL queue space and COMMIT. Simulation never awaits it.
        let result = match request.operation {
            JournalOperation::Command(command) => {
                self.publisher
                    .command_quest_journal(
                        self.binding,
                        self.world,
                        catalog,
                        &self.profile,
                        command,
                    )
                    .await
            }
            JournalOperation::Kill(event) => {
                self.publisher
                    .award_progression_and_quest_journal(
                        self.binding,
                        self.world,
                        &self.profile,
                        catalog,
                        &event,
                    )
                    .await
            }
            JournalOperation::Areas { id, observation } => {
                self.publisher
                    .record_reached_areas(self.binding, self.world, catalog, id, observation)
                    .await
            }
            JournalOperation::Event { id, event } => {
                if matches!(
                    event,
                    theia_core::quests::catalog::EventKey::Kill(_)
                        | theia_core::quests::catalog::EventKey::ReachArea(_)
                ) {
                    return Err(
                        "Journal kill/area requires its original typed authority input".into(),
                    );
                }
                self.publisher
                    .record_quest_event(self.binding, self.world, catalog, id, event)
                    .await
            }
        };
        match result {
            Ok(applied) => {
                let reply = command.then_some(JournalReply {
                    sequence: request.sequence,
                    rejection: None,
                });
                if catalog.inventory().is_some() {
                    self.publish_items(applied, reply)?;
                } else {
                    let mut merged = Ok(());
                    publish.send_modify(|s| {
                        merged =
                            super::quest_items::merge_journal(s, catalog.hash(), applied, reply)
                    });
                    merged?;
                }
            }
            Err(error) => {
                // Trusted occurrence conflicts/rule failures are authority errors,
                // not replies to a player command. Preserve command acknowledgements.
                if !command {
                    return Err(error.into());
                }
                let rejection = match error {
                    Error::QuestJournalCommand(rule) => JournalRejection::Rule(rule),
                    Error::QuestJournalConflict => JournalRejection::Conflict,
                    other => return Err(other.into()),
                };
                if catalog.inventory().is_some() {
                    let saved = self
                        .publisher
                        .load_quest_journal(self.binding, self.world, catalog)
                        .await?;
                    self.publish_items(
                        saved,
                        Some(JournalReply {
                            sequence: request.sequence,
                            rejection: Some(rejection),
                        }),
                    )?;
                    return Ok(());
                }
                publish.send_modify(|s| {
                    s.journal.as_mut().expect("Initialized journal owner").reply =
                        Some(JournalReply {
                            sequence: request.sequence,
                            rejection: Some(rejection),
                        })
                });
            }
        }
        Ok(())
    }
    fn publish_items(
        &self,
        applied: theia_persistence::QuestJournalApplied,
        reply: Option<theia_network::renet_runtime::progression::JournalReply>,
    ) -> Result<()> {
        let catalog = self.journal.as_ref().ok_or("Item journal not configured")?;
        self.gate
            .upgrade()
            .ok_or("Item journal gate closed")?
            .publish_quest_items(
                self.slot,
                self.generation,
                self.binding,
                catalog.hash(),
                applied,
                reply,
            )
    }
    async fn award(&self, event: KillEvent, publish: &watch::Sender<Observation>) -> Result<()> {
        // Expanded journal kills must retain their shared producer sequence and
        // combined transaction; refuse the legacy XP-only path in this mode.
        if self.journal.is_some() {
            return Err("Expanded journal requires sequenced atomic kill input".into());
        }
        if let Some(quest) = &self.lifecycle {
            let applied = self
                .publisher
                .award_progression_and_quest_lifecycle(
                    self.binding,
                    self.world,
                    &self.profile,
                    quest,
                    &event,
                )
                .await?;
            publish.send_modify(|s| {
                s.progress = Some(applied.experience.state);
                s.lifecycle = Some(applied.quest.state);
            });
        } else if let Some(quest) = &self.quest {
            let applied = self
                .publisher
                .award_progression_and_quest(self.binding, self.world, &self.profile, quest, &event)
                .await?;
            publish.send_modify(|s| {
                s.progress = Some(applied.experience.state);
                s.quest = Some(applied.quest.state);
            });
        } else {
            let applied = self
                .publisher
                .award_experience(self.binding, self.world, &self.profile, &event)
                .await?;
            publish.send_modify(|s| s.progress = Some(applied.state));
        }
        Ok(())
    }
    async fn command(
        &mut self,
        request: life::Request,
        publish: &watch::Sender<Observation>,
    ) -> Result<()> {
        // The world checked NPC entitlement before queue acceptance. Only the
        // retained generation/actor is rechecked now; close/walkaway/logout must
        // not erase an accepted intent or change its painted revision.
        if !self.authorized(Some(request.actor))?
            || request.sequence <= self.sequence
            || request.sequence.saturating_sub(self.sequence) > 128
        {
            return Err("Accepted quest action lost actor/sequence binding".into());
        }
        self.sequence = request.sequence;
        let quest = self
            .lifecycle
            .as_ref()
            .ok_or("Quest action requires explicit lifecycle owner")?;
        let result = if quest.turn_in_xp().is_some() {
            self.publisher
                .command_quest_lifecycle_with_experience(
                    self.binding,
                    self.world,
                    &self.profile,
                    quest,
                    request.command,
                )
                .await
                .map(|r| (r.quest, Some(r.experience.state)))
        } else {
            self.publisher
                .command_quest_lifecycle(self.binding, self.world, quest, request.command)
                .await
                .map(|r| (r, None))
        };
        match result {
            Ok((applied, experience)) => publish.send_modify(|s| {
                if let Some(experience) = experience {
                    s.progress = Some(experience);
                }
                s.lifecycle_reply = Some(ActionReply {
                    sequence: request.sequence,
                    quest_revision: applied.state.revision(),
                    rejection: None,
                });
                s.lifecycle = Some(applied.state);
            }),
            Err(error) => {
                let rejection = normal_refusal(error)?;
                publish.send_modify(|s| {
                    s.lifecycle_reply = Some(ActionReply {
                        sequence: request.sequence,
                        quest_revision: s
                            .lifecycle
                            .as_ref()
                            .expect("Initialized quest owner")
                            .revision(),
                        rejection: Some(rejection),
                    })
                });
            }
        }
        Ok(())
    }
}
fn normal_refusal(error: Error) -> Result<Rejection> {
    use theia_core::quests::lifecycle::Error as E;
    Ok(match error {
        Error::QuestLifecycleCommand(E::StaleRevision) | Error::QuestConflict => {
            Rejection::StaleRevision
        }
        Error::QuestLifecycleCommand(E::AlreadyAccepted | E::NotAccepted) => {
            Rejection::NotAvailable
        }
        Error::QuestLifecycleCommand(E::NotReady) => Rejection::NotReady,
        Error::QuestLifecycleCommand(E::AlreadyTurnedIn) => Rejection::AlreadyTurnedIn,
        Error::InventoryBusy => Rejection::Busy,
        other => return Err(other.into()),
    })
}
pub(super) struct Session {
    pub(super) publish: watch::Sender<Observation>,
    pub(super) port: Port,
    stop: watch::Sender<bool>,
    done: Arc<AtomicBool>,
}
impl Session {
    pub(super) fn stop(&self) {
        self.stop.send_replace(true);
    }
    pub(super) fn finished(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Finished {
    gate: Weak<GameAdmissions>,
    done: Arc<AtomicBool>,
    handled: bool,
}
impl Drop for Finished {
    fn drop(&mut self) {
        if !self.handled
            && let Some(gate) = self.gate.upgrade()
        {
            gate.fail_closed();
        }
        self.done.store(true, Ordering::Release);
    }
}
