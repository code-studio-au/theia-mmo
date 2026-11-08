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

/// One outstanding chat submission, independent of gameplay progress/transport.
/// A delayed reply times out locally without disconnecting or pausing the world.
pub(super) struct Driver {
    world: [u8; 32],
    client: crate::chat_client::Client,
    view: Option<crate::chat_client::View>,
    pending: Option<(u64, Instant)>,
}
impl Driver {
    pub(super) fn new(client: crate::chat_client::Client, expected: [u8; 32]) -> Result<Self> {
        if crate::chat::profile_identity(client.source_world, client.area) != expected {
            return Err("Unbound chat transport profile".into());
        }
        Ok(Self {
            world: expected,
            client,
            view: None,
            pending: None,
        })
    }
    fn discard_queued(&mut self) {
        for _ in 0..4 {
            if self.client.outgoing.try_recv().is_err() {
                break;
            }
        }
    }
    pub(super) fn admit(&mut self, welcome: &Welcome, state: &watch::Sender<State>) -> Result<()> {
        if welcome.world != self.world {
            return Err("Chat admission changed world profile".into());
        }
        let owner = EntityKey {
            epoch: theia_core::identity::WorldEpoch::new(welcome.epoch)
                .ok_or("Invalid chat admission epoch")?,
            entity: theia_core::identity::EntityId::new(welcome.id)
                .ok_or("Invalid chat admission actor")?,
        };
        let inbox = crate::chat_client::Inbox::new(owner, welcome.world)?;
        self.view = Some(crate::chat_client::View {
            inbox: std::sync::Arc::new(inbox),
            submitted: 0,
            pending: false,
            timed_out: false,
            exhausted: false,
        });
        self.pending = None;
        self.discard_queued();
        state.send_modify(|s| s.chat = self.view());
        Ok(())
    }
    pub(super) fn view(&self) -> Option<crate::chat_client::View> {
        self.view.clone()
    }
    pub(super) fn receive(
        &mut self,
        bytes: &[u8],
        state: &watch::Sender<State>,
        stats: &mut TransportStats,
    ) -> Result<()> {
        let view = self
            .view
            .as_mut()
            .ok_or("Chat observation before admission")?;
        if bytes.starts_with(crate::chat_wire::DELIVERY_TAG) {
            let message = crate::chat_wire::Delivery::decode(bytes)?;
            std::sync::Arc::make_mut(&mut view.inbox).receive(message)?;
            stats.chat_deliveries_received = stats.chat_deliveries_received.saturating_add(1);
        } else {
            let reply = crate::chat_wire::Reply::decode(bytes)?;
            if reply.sequence > view.submitted {
                return Err("Unsolicited chat submission reply".into());
            }
            let sequence = reply.sequence;
            std::sync::Arc::make_mut(&mut view.inbox).receive_reply(reply)?;
            if self.pending.is_some_and(|(pending, _)| pending == sequence) {
                self.pending = None;
                view.pending = false;
                view.timed_out = false;
            }
            stats.chat_replies_received = stats.chat_replies_received.saturating_add(1);
        }
        state.send_modify(|s| {
            s.chat = self.view();
            s.transport = Some(stats.clone());
        });
        Ok(())
    }
    pub(super) fn tick(
        &mut self,
        transport: &mut RenetClient,
        current: Instant,
        state: &watch::Sender<State>,
        stats: &mut TransportStats,
    ) -> Result<()> {
        if self.pending.is_some_and(|(_, sent)| {
            current.saturating_duration_since(sent) >= Duration::from_secs(5)
        }) {
            self.pending = None;
            if let Some(view) = &mut self.view {
                view.pending = false;
                view.timed_out = true;
            }
            self.discard_queued();
            stats.chat_timeouts = stats.chat_timeouts.saturating_add(1);
            state.send_modify(|s| {
                s.chat = self.view();
                s.transport = Some(stats.clone());
            });
            return Ok(());
        }
        if self.pending.is_some()
            || !transport.is_connected()
            || !transport.can_send_message(
                EVENT_CHANNEL,
                crate::chat_wire::MAX_REQUEST_BYTES + GAMEPLAY_RESERVE_BYTES,
            )
        {
            return Ok(());
        }
        let Some(view) = self.view.as_mut() else {
            return Ok(());
        };
        if view.exhausted {
            self.discard_queued();
            return Ok(());
        }
        let Ok(compose) = self.client.outgoing.try_recv() else {
            return Ok(());
        };
        let Some(sequence) = view.submitted.checked_add(1) else {
            view.exhausted = true;
            self.discard_queued();
            state.send_modify(|s| s.chat = self.view());
            return Ok(());
        };
        let owner = state
            .borrow()
            .welcome
            .as_ref()
            .map(|w| w.epoch)
            .and_then(theia_core::identity::WorldEpoch::new)
            .ok_or("Chat without admission")?;
        let request = crate::chat_wire::Request {
            epoch: owner,
            intent: theia_core::chat::Request {
                sequence,
                channel: compose.channel,
                text: compose.text,
            },
        };
        transport.send_message(EVENT_CHANNEL, request.encode()?);
        view.submitted = sequence;
        view.pending = true;
        view.timed_out = false;
        self.pending = Some((sequence, current));
        stats.chat_requests_sent = stats.chat_requests_sent.saturating_add(1);
        state.send_modify(|s| {
            s.chat = self.view();
            s.transport = Some(stats.clone());
        });
        Ok(())
    }
}

#[cfg(test)]
#[path = "local_chat/worker_tests.rs"]
mod worker_tests;
