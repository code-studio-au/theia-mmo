//! Authenticated party control and coalesced private observations. Membership
//! stays in World; no persistence, blocking work or unbounded social queue.
use super::*;
use crate::party_wire;
use theia_core::{identity::EntityKey, party::Refusal};

const GAMEPLAY_RESERVE: usize = CHANNEL_MEMORY_BYTES / 2;

#[derive(Default)]
pub(super) struct Peer {
    sent: Option<party_wire::State>,
    refusal: Option<Refusal>,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub accepted: u64,
    pub refused: u64,
    pub invalid_uploads: u64,
    pub states_sent: u64,
    pub deferred: u64,
}
pub(super) fn request(
    peer: &mut super::Peer,
    world: &mut World,
    bytes: &[u8],
    stats: &mut Stats,
) -> Result<()> {
    let result = (|| {
        if !world.parties_enabled()
            || peer.display_name.is_none()
            || peer
                .admission
                .as_ref()
                .is_none_or(|lease| !lease.live().unwrap_or(false))
        {
            return Err("Party request requires a live named admission".into());
        }
        let request = party_wire::Request::decode(bytes)?;
        if request.epoch.value() != world.epoch() {
            return Err("Foreign party request epoch".into());
        }
        match world.party_request(peer.actor, request.intent) {
            Ok(()) => {
                peer.party.refusal = None;
                stats.accepted += 1;
            }
            Err(Refusal::Context | Refusal::Sequence | Refusal::Exhausted) => {
                return Err("Invalid party request context or sequence".into());
            }
            Err(reason) => {
                peer.party.refusal = Some(reason);
                stats.refused += 1;
            }
        }
        Ok(())
    })();
    if result.is_err() {
        stats.invalid_uploads += 1;
    }
    result
}
pub(super) fn publish(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, super::Peer>,
    world: &World,
    stats: &mut Stats,
) -> Result<()> {
    if !world.parties_enabled() || world.completed_tick() == 0 {
        return Ok(());
    }
    let mut roster = peers
        .values()
        .map(|peer| {
            Ok(party_wire::NamedActor {
                key: world
                    .entity_key(peer.actor)
                    .ok_or("Party roster actor missing")?,
                name: peer
                    .display_name
                    .clone()
                    .ok_or("Party roster name missing")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    roster.sort_by_key(|actor| actor.key.entity);
    let identity = world.identity();
    for (&id, peer) in peers.iter_mut() {
        let state = party_wire::State {
            world: identity,
            tick: world.completed_tick(),
            view: world.party_view(peer.actor).ok_or("Party owner missing")?,
            refusal: peer.party.refusal,
            roster: roster.clone(),
        };
        if peer
            .party
            .sent
            .as_ref()
            .is_some_and(|old| state.same_view(old))
        {
            continue;
        }
        let bytes = state.encode()?;
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len() + GAMEPLAY_RESERVE) {
            stats.deferred += 1;
            continue;
        }
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.party.sent = Some(state);
        stats.states_sent += 1;
    }
    Ok(())
}

pub(super) struct Driver {
    world: [u8; 32],
    client: crate::party_client::Client,
    owner: Option<EntityKey>,
    admitted: Option<Instant>,
    view: Option<crate::party_client::View>,
    pending: Option<(u64, Instant)>,
}
impl Driver {
    pub(super) fn new(client: crate::party_client::Client, expected: [u8; 32]) -> Result<Self> {
        if party_wire::profile_identity(client.source_world) != expected {
            return Err("Unbound party transport profile".into());
        }
        Ok(Self {
            world: expected,
            client,
            owner: None,
            admitted: None,
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
    pub(super) fn admit(&mut self, welcome: &Welcome, now: Instant) -> Result<()> {
        if welcome.world != self.world {
            return Err("Party admission changed world".into());
        }
        self.owner = Some(EntityKey {
            epoch: theia_core::identity::WorldEpoch::new(welcome.epoch)
                .ok_or("Invalid party owner epoch")?,
            entity: theia_core::identity::EntityId::new(welcome.id).ok_or("Invalid party owner")?,
        });
        self.admitted = Some(now);
        self.view = None;
        self.pending = None;
        self.discard_queued();
        Ok(())
    }
    pub(super) fn view(&self) -> Option<crate::party_client::View> {
        self.view.clone()
    }
    pub(super) fn receive(
        &mut self,
        bytes: &[u8],
        state: &watch::Sender<State>,
        stats: &mut TransportStats,
    ) -> Result<()> {
        let incoming = party_wire::State::decode(bytes)?;
        let owner = self.owner.ok_or("Party observation before admission")?;
        incoming.validate_successor(
            owner,
            self.world,
            self.view.as_ref().map(|v| v.state.as_ref()),
        )?;
        let submitted = self.view.as_ref().map_or(0, |v| v.submitted);
        if incoming.view.sequence > submitted {
            return Err("Unsolicited party acknowledgement".into());
        }
        let acknowledged = self
            .pending
            .is_some_and(|(sequence, _)| incoming.view.sequence == sequence);
        if acknowledged {
            self.pending = None;
        }
        let timed_out = !acknowledged
            && self.view.as_ref().is_some_and(|v| v.timed_out)
            && incoming.view.sequence < submitted;
        self.view = Some(crate::party_client::View {
            state: std::sync::Arc::new(incoming),
            submitted,
            pending: self.pending.is_some(),
            timed_out,
        });
        stats.party_states_received += 1;
        state.send_modify(|s| {
            s.party = self.view();
            s.transport = Some(stats.clone());
        });
        Ok(())
    }
    pub(super) fn tick(
        &mut self,
        transport: &mut RenetClient,
        now: Instant,
        state: &watch::Sender<State>,
        stats: &mut TransportStats,
    ) -> Result<()> {
        if self.view.is_none()
            && self
                .admitted
                .is_some_and(|at| now.saturating_duration_since(at) >= Duration::from_secs(8))
        {
            return Err("Server did not publish party state".into());
        }
        if self
            .pending
            .is_some_and(|(_, at)| now.saturating_duration_since(at) >= Duration::from_secs(5))
        {
            self.pending = None;
            if let Some(view) = &mut self.view {
                view.pending = false;
                view.timed_out = true;
            }
            self.discard_queued();
            stats.party_timeouts += 1;
            state.send_modify(|s| {
                s.party = self.view();
                s.transport = Some(stats.clone());
            });
            return Ok(());
        }
        if self.pending.is_some()
            || !transport.is_connected()
            || !transport
                .can_send_message(EVENT_CHANNEL, party_wire::REQUEST_BYTES + GAMEPLAY_RESERVE)
        {
            return Ok(());
        }
        let Some(view) = self.view.as_mut() else {
            return Ok(());
        };
        let Ok(intent) = self.client.outgoing.try_recv() else {
            return Ok(());
        };
        if Some(intent.owner) != self.owner {
            return Err("Queued party click from another incarnation".into());
        }
        let sequence = view
            .submitted
            .checked_add(1)
            .ok_or("Party sequence exhausted")?;
        let request = party_wire::Request {
            epoch: intent.owner.epoch,
            intent: theia_core::party::Request {
                sequence,
                revision: intent.revision,
                action: intent.action,
            },
        };
        transport.send_message(EVENT_CHANNEL, request.encode()?);
        view.submitted = sequence;
        view.pending = true;
        view.timed_out = false;
        self.pending = Some((sequence, now));
        stats.party_requests_sent += 1;
        state.send_modify(|s| {
            s.party = self.view();
            s.transport = Some(stats.clone());
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests;
