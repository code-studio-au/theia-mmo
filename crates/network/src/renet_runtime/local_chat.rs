//! Opt-in low-priority named chat. No SQL, provider, renderer or awaited I/O.
use super::*;
use theia_core::{chat::Speaker, identity::EntityKey};
// Chat itself may never occupy the final half of the shared reliable queue.
// This supplements publishing order; critical events can arrive on the next step.
const GAMEPLAY_RESERVE_BYTES: usize = CHANNEL_MEMORY_BYTES / 2;
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub accepted: u64,
    pub replies_sent: u64,
    pub replies_deferred: u64,
    pub replies_dropped: u64,
    pub refused: u64,
    pub unnamed_uploads: u64,
    pub queued: u64,
    pub dropped_full: u64,
    pub sent: u64,
    pub deferred: u64,
    pub disabled_uploads: u64,
    pub malformed_uploads: u64,
    pub forged_deliveries: u64,
}
pub(super) struct Bridge {
    router: crate::chat::Router,
    actors: BTreeMap<u64, EntityKey>,
    replies: BTreeMap<u64, std::collections::VecDeque<crate::chat_wire::Reply>>,
}
impl Bridge {
    pub(super) fn new(world: &World) -> Result<Option<Self>> {
        if world.local_chat_area().is_none() {
            return Ok(None);
        }
        let epoch = theia_core::identity::WorldEpoch::new(world.epoch())
            .ok_or("Invalid chat world epoch")?;
        Ok(Some(Self {
            router: crate::chat::Router::new(world.identity(), epoch, world.tick_rate())?,
            actors: BTreeMap::new(),
            replies: BTreeMap::new(),
        }))
    }
    pub(super) fn sync(&mut self, peers: &BTreeMap<u64, Peer>, world: &World) -> Result<()> {
        let mut active = BTreeMap::new();
        for peer in peers.values() {
            let Some(lease) = &peer.admission else {
                continue;
            };
            if !lease.live()? {
                continue;
            }
            let Some(name) = &peer.display_name else {
                continue;
            };
            let key = world.entity_key(peer.actor).ok_or("Chat actor missing")?;
            let position = world
                .global_position(peer.actor)
                .ok_or("Chat actor pose missing")?;
            active.insert(peer.actor, (key, name.clone(), position));
        }
        self.actors.retain(|actor, key| {
            if active
                .get(actor)
                .is_none_or(|(current, _, _)| current != key)
            {
                self.router.retire(*key);
                false
            } else {
                true
            }
        });
        self.replies
            .retain(|actor, _| self.actors.contains_key(actor));
        for (actor, (key, name, position)) in active {
            if !self.actors.contains_key(&actor) {
                self.router.admit(Speaker {
                    key,
                    name,
                    position,
                    area: world.local_chat_area(),
                })?;
                self.actors.insert(actor, key);
            } else {
                self.router.update(key, position, world.local_chat_area());
            }
        }
        Ok(())
    }
    pub(super) fn request(
        &mut self,
        peer: &Peer,
        world: &World,
        bytes: &[u8],
        stats: &mut Stats,
    ) -> Result<()> {
        let request = crate::chat_wire::Request::decode(bytes)
            .inspect_err(|_| stats.malformed_uploads += 1)?;
        let Some(key) = self.actors.get(&peer.actor).copied() else {
            stats.unnamed_uploads += 1;
            return Ok(());
        };
        let sequence = request.intent.sequence;
        let result = self.router.submit(key, request, world.completed_tick());
        use crate::chat_wire::Outcome as O;
        use theia_core::chat::Refusal as R;
        let outcome = match result {
            Ok(()) => O::Accepted,
            Err(
                crate::chat::Rejected::Admission
                | crate::chat::Rejected::Exhausted
                | crate::chat::Rejected::Policy(R::Context),
            ) => O::Unavailable,
            Err(crate::chat::Rejected::Policy(reason)) => match reason {
                R::Sequence => O::Sequence,
                R::Time => O::Time,
                R::Muted => O::Muted,
                R::NoArea => O::NoArea,
                R::RateLimited => O::RateLimited,
                R::Context => unreachable!(),
            },
        };
        let queue = self
            .replies
            .entry(peer.actor)
            .or_insert_with(|| std::collections::VecDeque::with_capacity(4));
        if queue.len() < 4 {
            queue.push_back(crate::chat_wire::Reply {
                world: world.identity(),
                owner: key,
                sequence,
                tick: world.completed_tick(),
                outcome,
            });
        } else {
            stats.replies_dropped = stats.replies_dropped.saturating_add(1);
        }
        self.observe(stats);
        Ok(())
    }
    fn observe(&self, stats: &mut Stats) {
        let counts = self.router.stats();
        stats.accepted = counts.accepted;
        stats.refused = counts.refused;
        stats.queued = counts.queued;
        stats.dropped_full = counts.dropped_full;
        stats.sent = counts.sent;
        stats.deferred = counts.deferred;
    }
    pub(super) fn flush(
        &mut self,
        server: &mut RenetServer,
        peers: &BTreeMap<u64, Peer>,
        stats: &mut Stats,
    ) -> Result<()> {
        let recipients: BTreeMap<_, _> = peers
            .iter()
            .map(|(transport, peer)| (peer.actor, *transport))
            .collect();
        for (actor, queue) in &mut self.replies {
            let Some(reply) = queue.front() else {
                continue;
            };
            let Some(&id) = recipients.get(actor) else {
                continue;
            };
            let bytes = reply.encode()?;
            if server.can_send_message(id, EVENT_CHANNEL, bytes.len() + GAMEPLAY_RESERVE_BYTES) {
                server.send_message(id, EVENT_CHANNEL, bytes);
                queue.pop_front();
                stats.replies_sent = stats.replies_sent.saturating_add(1);
            } else {
                stats.replies_deferred = stats.replies_deferred.saturating_add(1);
            }
        }
        self.router.flush(|key, bytes| {
            let Some(&id) = recipients.get(&key.entity.value()) else {
                return false;
            };
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len() + GAMEPLAY_RESERVE_BYTES) {
                return false;
            }
            server.send_message(id, EVENT_CHANNEL, bytes.to_vec());
            true
        })?;
        self.observe(stats);
        Ok(())
    }
}
#[cfg(test)]
mod tests;
