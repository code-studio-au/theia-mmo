//! A character-bound async XP owner publishes only committed state.
use std::sync::Arc;
use theia_core::progression::{KillEvent, State};
use tokio::sync::{mpsc, watch};
pub const CAPACITY: usize = crate::simulation::progression::FACT_CAPACITY;
#[derive(Clone, Debug, Default)]
pub struct Observation {
    pub quest: Option<theia_core::quests::State>,
    pub progress: Option<State>,
    pub closed: bool,
    pub failed: bool,
}
#[derive(Clone)]
pub struct Port {
    pub quest: Option<Arc<crate::quest_wire::Profile>>,
    pub profile: Arc<crate::progression_wire::Profile>,
    /// Trusted server facts only; no client request populates this queue.
    pub earned: mpsc::Sender<KillEvent>,
    pub state: watch::Receiver<Observation>,
}
pub(super) struct Peer {
    pub port: Port,
    pub sent: Option<State>,
    pub quest_sent: Option<theia_core::quests::State>,
}
impl Peer {
    pub fn new(port: Port) -> crate::Result<Self> {
        if port.earned.max_capacity() != CAPACITY {
            return Err("Invalid XP fact queue capacity".into());
        }
        if port.quest.as_ref().is_some_and(|q| {
            port.profile
                .profile()
                .kill_experience(q.profile().creature())
                .is_none()
        }) {
            return Err("Objective/progression port mismatch".into());
        }
        Ok(Self {
            port,
            sent: None,
            quest_sent: None,
        })
    }
}
