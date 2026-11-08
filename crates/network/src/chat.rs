//! Bounded host-side chat routing prerequisite. Runtime admission and transport
//! integration must bind every caller key to its authenticated connection.
use crate::{
    Result,
    chat_wire::{Delivery, Request},
};
use std::collections::{BTreeMap, VecDeque};
use theia_core::{
    chat::{Gate, Listener, Refusal, Speaker},
    coordinates::GlobalPosition,
    identity::{EntityKey, WorldEpoch},
    time::TickRate,
};
pub const MAX_PEERS: usize = crate::wire::MAX_PLAYERS;
pub const OUTBOX_MESSAGES: usize = 16;
struct Peer {
    speaker: Speaker,
    gate: Gate,
    muted: bool,
    blocked: Vec<EntityKey>,
    outbox: VecDeque<Delivery>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub accepted: u64,
    pub refused: u64,
    pub queued: u64,
    pub dropped_full: u64,
    pub sent: u64,
    pub deferred: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    Admission,
    Policy(Refusal),
}
pub struct Router {
    world: [u8; 32],
    epoch: WorldEpoch,
    rate: TickRate,
    peers: BTreeMap<u64, Peer>,
    stats: Stats,
}
impl Router {
    pub fn new(world: [u8; 32], epoch: WorldEpoch, rate: TickRate) -> Result<Self> {
        if world == [0; 32] {
            return Err("Chat requires a world identity".into());
        }
        Ok(Self {
            world,
            epoch,
            rate,
            peers: BTreeMap::new(),
            stats: Stats::default(),
        })
    }
    /// Trusted admission only. Duplicate registration never resets a gate or
    /// overwrites queued messages. Retire an old incarnation before reusing IDs.
    pub fn admit(&mut self, speaker: Speaker) -> Result<()> {
        let id = speaker.key.entity.value();
        if speaker.key.epoch != self.epoch
            || self.peers.len() >= MAX_PEERS
            || self.peers.contains_key(&id)
        {
            return Err("Invalid or full chat admission".into());
        }
        let gate = Gate::new(speaker.key, self.rate);
        self.peers.insert(
            id,
            Peer {
                speaker,
                gate,
                muted: false,
                blocked: Vec::new(),
                outbox: VecDeque::with_capacity(OUTBOX_MESSAGES),
            },
        );
        Ok(())
    }
    fn peer_mut(&mut self, key: EntityKey) -> Option<&mut Peer> {
        self.peers
            .get_mut(&key.entity.value())
            .filter(|p| p.speaker.key == key)
    }
    /// Supply current checked authoritative pose/area before routing each host
    /// step. Chat never mutates movement or chooses area membership itself.
    pub fn update(
        &mut self,
        key: EntityKey,
        position: GlobalPosition,
        area: Option<theia_core::chat::Area>,
    ) -> bool {
        let Some(peer) = self.peer_mut(key) else {
            return false;
        };
        peer.speaker.position = position;
        peer.speaker.area = area;
        true
    }
    /// Host moderation result. The router does not implement a public mute API.
    pub fn mute(&mut self, key: EntityKey, muted: bool) -> bool {
        let Some(peer) = self.peer_mut(key) else {
            return false;
        };
        peer.muted = muted;
        true
    }
    pub fn block(&mut self, listener: EntityKey, sender: EntityKey, blocked: bool) -> bool {
        if sender.epoch != self.epoch {
            return false;
        }
        let Some(peer) = self.peer_mut(listener) else {
            return false;
        };
        if blocked {
            if !peer.blocked.contains(&sender) {
                if peer.blocked.len() >= MAX_PEERS {
                    return false;
                }
                peer.blocked.push(sender);
            }
            // A newly blocked sender's unsent text must not leak later.
            peer.outbox.retain(|message| message.sender != sender);
        } else {
            peer.blocked.retain(|key| *key != sender);
        }
        true
    }
    pub fn retire(&mut self, key: EntityKey) -> bool {
        if self
            .peers
            .get(&key.entity.value())
            .is_none_or(|p| p.speaker.key != key)
        {
            return false;
        }
        self.peers.remove(&key.entity.value());
        true
    }
    pub fn submit(
        &mut self,
        authenticated: EntityKey,
        request: Request,
        tick: u64,
    ) -> std::result::Result<(), Rejected> {
        let result = self.route(authenticated, request, tick);
        match result {
            Ok(()) => self.stats.accepted = self.stats.accepted.saturating_add(1),
            Err(_) => self.stats.refused = self.stats.refused.saturating_add(1),
        }
        result
    }
    fn route(
        &mut self,
        authenticated: EntityKey,
        request: Request,
        tick: u64,
    ) -> std::result::Result<(), Rejected> {
        if request.epoch != self.epoch {
            return Err(Rejected::Admission);
        }
        let peer = self.peer_mut(authenticated).ok_or(Rejected::Admission)?;
        let message = peer
            .gate
            .submit(peer.speaker.clone(), request.intent, tick, peer.muted)
            .map_err(Rejected::Policy)?;
        for peer in self.peers.values_mut() {
            if !message.reaches(Listener {
                key: peer.speaker.key,
                position: peer.speaker.position,
                area: peer.speaker.area,
                blocks_sender: peer.blocked.contains(&authenticated),
            }) {
                continue;
            }
            if peer.outbox.len() >= OUTBOX_MESSAGES {
                self.stats.dropped_full = self.stats.dropped_full.saturating_add(1);
                continue;
            }
            peer.outbox.push_back(Delivery {
                world: self.world,
                sender: message.speaker.key,
                sequence: message.sequence,
                tick: message.tick,
                channel: message.channel,
                name: message.speaker.name.clone(),
                text: message.text.clone(),
            });
            self.stats.queued = self.stats.queued.saturating_add(1);
        }
        Ok(())
    }
    /// At most one bounded frame per peer/call. A false transport capacity result
    /// retains the message, never blocks/awaits or disconnects gameplay peers.
    /// Call after gameplay-priority publishing, not inside simulation work.
    pub fn flush(&mut self, mut send: impl FnMut(EntityKey, &[u8]) -> bool) -> Result<()> {
        for peer in self.peers.values_mut() {
            let Some(message) = peer.outbox.front() else {
                continue;
            };
            let bytes = message.encode()?;
            if send(peer.speaker.key, &bytes) {
                peer.outbox.pop_front();
                self.stats.sent = self.stats.sent.saturating_add(1);
            } else {
                self.stats.deferred = self.stats.deferred.saturating_add(1);
            }
        }
        Ok(())
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
}
#[cfg(test)]
mod tests;
