//! Bounded owner-side pose snapshots. No database or persistent-ID dependency.
use super::{MAX_PLAYERS, Peer, World};
use std::collections::BTreeMap;
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
use tokio::sync::watch;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorCheckpoint {
    pub actor: EntityKey,
    pub position: GlobalPosition,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AuthorityCheckpoint {
    pub epoch: u64,
    pub tick: u64,
    /// Index is authenticated local issuer client ID minus one, not actor ID.
    pub slots: [Option<ActorCheckpoint>; MAX_PLAYERS],
}
pub struct CheckpointBridge {
    restored: [Option<GlobalPosition>; MAX_PLAYERS],
    retained: [Option<ActorCheckpoint>; MAX_PLAYERS],
    latest: watch::Sender<Option<AuthorityCheckpoint>>,
}
impl CheckpointBridge {
    pub fn channel(
        restored: [Option<GlobalPosition>; MAX_PLAYERS],
    ) -> (Self, watch::Receiver<Option<AuthorityCheckpoint>>) {
        let (latest, receiver) = watch::channel(None);
        (
            Self {
                restored,
                retained: [None; MAX_PLAYERS],
                latest,
            },
            receiver,
        )
    }
    pub(super) fn pending(&self) -> Vec<GlobalPosition> {
        self.restored.iter().flatten().copied().collect()
    }
    pub(super) fn position(&self, client: u64) -> Option<GlobalPosition> {
        self.restored[(client - 1) as usize]
    }
    pub(super) fn admitted(&mut self, client: u64) {
        self.restored[(client - 1) as usize] = None;
    }
    pub(super) fn capture(&mut self, world: &World, peers: &BTreeMap<u64, Peer>) -> bool {
        for (&client, peer) in peers {
            if let (Some(actor), Some(position)) = (
                world.entity_key(peer.actor),
                world.global_position(peer.actor),
            ) {
                self.retained[(client - 1) as usize] = Some(ActorCheckpoint { actor, position });
            }
        }
        let snapshot = AuthorityCheckpoint {
            epoch: world.epoch(),
            tick: world.completed_tick(),
            slots: self.retained,
        };
        self.latest.send_if_modified(|old| {
            if old.as_ref() == Some(&snapshot) {
                false
            } else {
                *old = Some(snapshot);
                true
            }
        })
    }
}
