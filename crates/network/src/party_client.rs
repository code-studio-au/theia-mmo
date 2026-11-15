//! Bounded render-independent party controls and observations.
use crate::{Result, party_wire};
use theia_core::{identity::EntityKey, party::Action};

#[derive(Clone, Debug)]
pub struct View {
    pub state: std::sync::Arc<party_wire::State>,
    pub submitted: u64,
    pub pending: bool,
    pub timed_out: bool,
}
/// Retain the identity and revision actually painted by the UI; the worker
/// must never rebind an old click to a newer invitation or party membership.
#[derive(Clone, Debug)]
pub struct Intent {
    pub owner: EntityKey,
    pub revision: u64,
    pub action: Action,
}
pub struct Client {
    pub(crate) source_world: [u8; 32],
    pub(crate) outgoing: tokio::sync::mpsc::Receiver<Intent>,
}
impl Client {
    pub fn new(
        source_world: [u8; 32],
        outgoing: tokio::sync::mpsc::Receiver<Intent>,
    ) -> Result<Self> {
        if source_world == [0; 32] || outgoing.max_capacity() > 4 {
            return Err("Invalid party source or command capacity".into());
        }
        Ok(Self {
            source_world,
            outgoing,
        })
    }
}
