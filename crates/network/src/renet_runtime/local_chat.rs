//! Opt-in low-priority named chat. No SQL, provider, renderer or awaited I/O.
use super::*;
use theia_core::{chat::Speaker, identity::EntityKey};
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub accepted: u64,
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
        let _ = self.router.submit(key, request, world.completed_tick());
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
        self.router.flush(|key, bytes| {
            let Some(&id) = recipients.get(&key.entity.value()) else {
                return false;
            };
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
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
