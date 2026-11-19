//! A character-bound async XP owner publishes only committed state.
use std::sync::Arc;
use theia_core::progression::{KillEvent, State};
use tokio::sync::{mpsc, watch};
pub const CAPACITY: usize = crate::simulation::progression::FACT_CAPACITY;
/// One authority-producer FIFO orders accepted NPC actions against kill facts.
/// Action construction alone establishes no entitlement; the runtime must check
/// the current guide/session/range before enqueueing it.
#[derive(Clone, Debug)]
pub enum Input {
    Kill(KillEvent),
    QuestAction(crate::quest_lifecycle_wire::Request),
    Journal(JournalInput),
}
/// Trusted authority-only interface; no network decoder admits these directly.
#[derive(Clone, Debug)]
pub struct JournalInput {
    pub actor: theia_core::identity::EntityKey,
    pub catalog: [u8; 32],
    pub sequence: u64,
    pub operation: JournalOperation,
}
#[derive(Clone, Debug)]
pub enum JournalOperation {
    /// Original authority victim life, committed with optional XP in one transaction.
    Kill(KillEvent),
    /// Already-authorized occupancy sealed to original eligible quest stages.
    Areas {
        id: std::num::NonZeroU128,
        observation: theia_core::quests::journal::ReachedAreas,
    },
    Command(theia_core::quests::journal::Command),
    Event {
        id: std::num::NonZeroU128,
        event: theia_core::quests::catalog::EventKey,
    },
}
#[derive(Clone, Debug)]
pub struct JournalObservation {
    pub catalog: [u8; 32],
    pub revision: u64,
    pub journal: theia_core::quests::journal::Journal,
    pub reply: Option<JournalReply>,
}
#[derive(Clone, Debug)]
pub struct JournalReply {
    pub sequence: u64,
    pub rejection: Option<JournalRejection>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRejection {
    Rule(theia_core::quests::journal::Error),
    Conflict,
    Busy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionReply {
    pub sequence: u64,
    pub quest_revision: u64,
    pub rejection: Option<crate::quest_lifecycle_wire::Rejection>,
}
#[derive(Clone, Debug, Default)]
pub struct Observation {
    pub journal: Option<JournalObservation>,
    pub lifecycle: Option<theia_core::quests::lifecycle::State>,
    pub lifecycle_reply: Option<ActionReply>,
    pub quest: Option<theia_core::quests::State>,
    pub progress: Option<State>,
    pub closed: bool,
    pub failed: bool,
}
#[derive(Clone)]
pub struct Port {
    pub journal: Option<[u8; 32]>,
    pub lifecycle: Option<Arc<crate::quest_lifecycle_wire::Profile>>,
    pub quest: Option<Arc<crate::quest_wire::Profile>>,
    pub profile: Arc<crate::progression_wire::Profile>,
    /// Trusted kills and already authorized NPC actions, never client counters.
    pub input: mpsc::Sender<Input>,
    pub state: watch::Receiver<Observation>,
}
pub(super) struct Peer {
    pub journal: Option<super::quest_journal::Peer>,
    pub lifecycle: Option<super::quest_lifecycle::Peer>,
    pub port: Port,
    pub sent: Option<State>,
    pub quest_sent: Option<theia_core::quests::State>,
}
impl Peer {
    pub fn new(port: Port) -> crate::Result<Self> {
        if port.journal.is_some_and(|hash| hash == [0; 32])
            || (port.journal.is_some() && (port.quest.is_some() || port.lifecycle.is_some()))
        {
            return Err("Invalid or conflicting expanded journal port".into());
        }
        if port.input.max_capacity() != CAPACITY {
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
        if (port.quest.is_some() && port.lifecycle.is_some())
            || port.lifecycle.as_ref().is_some_and(|q| {
                port.profile
                    .profile()
                    .kill_experience(q.profile().creature())
                    .is_none()
            })
        {
            return Err("Explicit quest/progression port mismatch".into());
        }
        if let Some(profile) = &port.lifecycle {
            profile.check_progression(&port.profile)?;
        }
        Ok(Self {
            journal: None,
            lifecycle: port
                .lifecycle
                .as_ref()
                .map(|_| super::quest_lifecycle::Peer::default()),
            port,
            sent: None,
            quest_sent: None,
        })
    }
}
