//! Generation-bound durable XP owner; accepted facts drain before slot reuse.
use super::{GameAdmissions, Phase};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
use theia_core::progression::KillEvent;
use theia_network::{
    Result,
    renet_runtime::progression::{CAPACITY, Observation, Port},
};
use theia_persistence::{Binding, ProgressionCatalog, Publisher};
use tokio::sync::{mpsc, watch};
pub(super) struct Config {
    quest: Option<(
        theia_persistence::QuestCatalog,
        Arc<theia_network::quest_wire::Profile>,
    )>,
    profile: ProgressionCatalog,
    wire: Arc<theia_network::progression_wire::Profile>,
    publisher: Publisher,
}
impl Config {
    pub(super) fn new(
        profile: ProgressionCatalog,
        wire: Arc<theia_network::progression_wire::Profile>,
        publisher: Publisher,
    ) -> Result<Self> {
        if profile.hash() != wire.hash() {
            return Err("XP store/wire profile mismatch".into());
        }
        Ok(Self {
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
        if quest.hash() != wire.hash()
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
    pub(super) fn start(
        &self,
        gate: Weak<GameAdmissions>,
        slot: usize,
        generation: u64,
        binding: Binding,
        world: [u8; 32],
    ) -> Session {
        let (earned, mut receiver) = mpsc::channel::<KillEvent>(CAPACITY);
        let (publish, state) = watch::channel(Observation::default());
        let (stop, mut stopping) = watch::channel(false);
        let done = Arc::new(AtomicBool::new(false));
        let guard = Finished {
            gate: gate.clone(),
            done: done.clone(),
            handled: false,
        };
        let profile = self.profile.clone();
        let quest = self.quest.as_ref().map(|(q, _)| q.clone());
        let publisher = self.publisher.clone();
        tokio::spawn(async move {
            let mut guard = guard;
            let result: Result<()>=async {
                let authorized=|| -> Result<bool> {
                    let Some(gate)=gate.upgrade() else {return Ok(false);};
                    let state=gate.lock()?;
                    Ok(state.slots[slot].as_ref().is_some_and(|entry| entry.generation==generation && entry.binding==binding && matches!(entry.phase,Phase::Consumed|Phase::Active|Phase::Retiring)))
                };
                if !authorized()? {return Err("XP character ownership lost".into());}
                let initial=publisher.ensure_progression(binding,world,&profile).await?;
                let objective=if let Some(quest)=&quest {Some(publisher.ensure_quest(binding,world,quest).await?)} else {None};
                publish.send_modify(|s|{s.progress=Some(initial);s.quest=objective;});
                loop {
                    if *stopping.borrow() { receiver.close(); }
                    let event=if receiver.is_closed() {
                        receiver.recv().await
                    } else {
                        tokio::select! {biased;
                            changed=stopping.changed()=>{if changed.is_err() || *stopping.borrow() {receiver.close();} continue;},
                            event=receiver.recv()=>event,
                        }
                    };
                    let Some(event)=event else {break;};
                    if !authorized()? {return Err("Accepted XP fact lost ownership".into());}
                    if let Some(quest)=&quest {
                        let applied=publisher.award_progression_and_quest(binding,world,&profile,quest,&event).await?;
                        publish.send_modify(|s|{s.progress=Some(applied.experience.state);s.quest=Some(applied.quest.state);});
                    } else {
                        let applied=publisher.award_experience(binding,world,&profile,&event).await?;
                        publish.send_modify(|s|s.progress=Some(applied.state));
                    }
                }
                Ok(())
            }.await;
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
            port: Port {
                quest: self.quest.as_ref().map(|(_, wire)| wire.clone()),
                profile: self.wire.clone(),
                earned,
                state,
            },
            stop,
            done,
        }
    }
}
pub(super) struct Session {
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
