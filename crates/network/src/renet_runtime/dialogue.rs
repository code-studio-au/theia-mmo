//! Bounded ephemeral informational interaction over the existing reliable channel.
use super::{Peer as RuntimePeer, ServerStats, retire_peers};
use crate::{
    Result,
    client::{State, TransportStats},
    dialogue::{Action, Profile, Request, View},
    dialogue_wire::{self as wire, Presence, Reply},
    renet_transport::EVENT_CHANNEL,
    simulation::World,
    wire::Welcome,
};
use renet::{RenetClient, RenetServer};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
use tokio::{sync::mpsc, time::Instant};

#[derive(Default)]
pub(super) struct Peer {
    view: Option<View>,
    presence: Option<Presence>,
    sent_reply: Option<Reply>,
    pending_reply: Option<Reply>,
}
impl Peer {
    pub(super) fn request(
        &mut self,
        world: &mut World,
        actor: u64,
        bytes: &[u8],
        stats: &mut ServerStats,
    ) -> Result<()> {
        let request = wire::decode_request(bytes, world.epoch()).inspect_err(|_| {
            stats.dialogue_requests_rejected += 1;
        })?;
        let result = world.interact(actor, &request);
        if result.is_ok() {
            stats.dialogue_requests_accepted += 1;
        } else {
            stats.dialogue_requests_rejected += 1;
        }
        if result == Err(crate::dialogue::Rejection::RateLimited) {
            return Err("Dialogue request budget exceeded".into());
        }
        let view = world.dialogue_view(actor).ok_or("Dialogue actor missing")?;
        if self
            .sent_reply
            .as_ref()
            .is_none_or(|old| request.sequence > old.sequence)
            && self
                .pending_reply
                .as_ref()
                .is_none_or(|old| request.sequence > old.sequence)
        {
            self.pending_reply = Some(Reply {
                actor: view.actor,
                profile_hash: view.profile_hash,
                sequence: request.sequence,
                view_revision: view.revision,
                rejection: result.err(),
            });
        }
        Ok(())
    }
}
pub(super) fn publish(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, RuntimePeer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let Some(profile) = world.dialogue_profile().cloned() else {
        return Ok(());
    };
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let Some(dialogue) = &mut peer.dialogue else {
            continue;
        };
        let view = world
            .dialogue_view(peer.actor)
            .ok_or("Dialogue peer missing actor")?;
        let presence = Presence {
            actor: view.actor,
            profile_hash: profile.hash(),
            guide: world.visible_guide(peer.actor),
        };
        let next_view = if dialogue.view.as_ref() != Some(&view) {
            Some(wire::encode_view(&view, &profile)?)
        } else {
            None
        };
        let next_presence = if dialogue.presence.as_ref() != Some(&presence) {
            Some(wire::encode_presence(&presence, &profile)?)
        } else {
            None
        };
        let next_reply = dialogue
            .pending_reply
            .as_ref()
            .map(|reply| wire::encode_reply(reply, &profile))
            .transpose()?;
        let size = next_view.as_ref().map_or(0, Vec::len)
            + next_presence.as_ref().map_or(0, Vec::len)
            + next_reply.as_ref().map_or(0, Vec::len);
        if size > 0 && !server.can_send_message(id, EVENT_CHANNEL, size) {
            slow.push(id);
            continue;
        }
        // One ordered channel: publish the authoritative view before its reply.
        if let Some(bytes) = next_view {
            server.send_message(id, EVENT_CHANNEL, bytes);
            dialogue.view = Some(view);
            stats.dialogue_views_sent += 1;
        }
        if let Some(bytes) = next_presence {
            server.send_message(id, EVENT_CHANNEL, bytes);
            dialogue.presence = Some(presence);
            stats.guide_presences_sent += 1;
        }
        if let Some(bytes) = next_reply {
            server.send_message(id, EVENT_CHANNEL, bytes);
            dialogue.sent_reply = dialogue.pending_reply.take();
            stats.dialogue_replies_sent += 1;
        }
    }
    stats.dialogue_backpressure_disconnects += slow.len() as u64;
    retire_peers(server, peers, world, &slow)
}

#[derive(Clone, Debug)]
pub struct Intent {
    pub expected_revision: u64,
    pub action: Action,
}
pub struct Client {
    pub profile: Arc<Profile>,
    pub actions: mpsc::Receiver<Intent>,
}
pub(super) struct ClientState {
    mode: Client,
    view: Option<View>,
    presence: Option<Presence>,
    known_guide: Option<crate::dialogue::Guide>,
    reply: Option<Reply>,
    sequence: u64,
    pending: Option<(u64, Instant)>,
    next_allowed: Instant,
}
pub(super) fn is_frame(bytes: &[u8]) -> bool {
    [wire::VIEW_TAG, wire::GUIDE_TAG, wire::REPLY_TAG]
        .into_iter()
        .any(|tag| bytes.starts_with(tag))
}
fn actor(welcome: &Welcome) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(welcome.epoch).ok_or("Invalid dialogue admission epoch")?,
        entity: EntityId::new(welcome.id).ok_or("Invalid dialogue admission actor")?,
    })
}
impl ClientState {
    pub(super) fn new(mode: Client) -> Result<Self> {
        if mode.actions.max_capacity() > 4 {
            return Err("Dialogue action queue exceeds four requests".into());
        }
        Ok(Self {
            mode,
            view: None,
            presence: None,
            known_guide: None,
            reply: None,
            sequence: 0,
            pending: None,
            next_allowed: Instant::now(),
        })
    }
    pub(super) fn receive(
        &mut self,
        bytes: &[u8],
        welcome: &Welcome,
        stats: &mut TransportStats,
    ) -> Result<()> {
        let actor = actor(welcome)?;
        if bytes.starts_with(wire::VIEW_TAG) {
            let view = wire::decode_view(bytes, &self.mode.profile)?;
            wire::validate_successor(&view, actor, self.view.as_ref())?;
            if let (Some(open), Some(guide)) = (&view.open, self.known_guide)
                && open.guide != guide.key
            {
                return Err("Dialogue view changed guide incarnation".into());
            }
            self.view = Some(view);
            stats.dialogue_views_received += 1;
        } else if bytes.starts_with(wire::GUIDE_TAG) {
            let presence = wire::decode_presence(bytes, &self.mode.profile, actor)?;
            if let Some(guide) = presence.guide {
                if self.known_guide.is_some_and(|old| old != guide) {
                    return Err("Guide changed position/incarnation within profile".into());
                }
                self.known_guide = Some(guide);
            }
            self.presence = Some(presence);
            stats.guide_presences_received += 1;
        } else if bytes.starts_with(wire::REPLY_TAG) {
            let reply = wire::decode_reply(bytes, &self.mode.profile)?;
            wire::validate_reply(&reply, actor, self.sequence, self.reply.as_ref())?;
            self.reply = Some(reply);
            stats.dialogue_replies_received += 1;
        } else {
            return Err("Unknown dialogue observation".into());
        }
        if let Some((sequence, _)) = self.pending
            && self.reply.as_ref().is_some_and(|reply| {
                reply.sequence == sequence
                    && self
                        .view
                        .as_ref()
                        .is_some_and(|view| view.revision >= reply.view_revision)
            })
        {
            self.pending = None;
        }
        Ok(())
    }
    pub(super) fn check(&self, elapsed: Duration, admitted: bool, current: Instant) -> Result<()> {
        if admitted
            && elapsed > Duration::from_secs(8)
            && (self.view.is_none() || self.presence.is_none())
        {
            return Err("Server did not publish initial dialogue state".into());
        }
        if self.pending.is_some_and(|(_, started)| {
            current.saturating_duration_since(started) > Duration::from_secs(8)
        }) {
            return Err("Dialogue request acknowledgement timed out".into());
        }
        Ok(())
    }
    pub(super) fn send(
        &mut self,
        client: &mut RenetClient,
        welcome: &Welcome,
        current: Instant,
        stats: &mut TransportStats,
    ) -> Result<bool> {
        if self.pending.is_some()
            || self.view.is_none()
            || self.presence.is_none()
            || current < self.next_allowed
            || !client.can_send_message(EVENT_CHANNEL, wire::MAX_REQUEST_BYTES)
        {
            return Ok(false);
        }
        let Ok(intent) = self.mode.actions.try_recv() else {
            return Ok(false);
        };
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("Dialogue request sequence exhausted")?;
        let request = Request {
            sequence,
            expected_revision: intent.expected_revision,
            action: intent.action,
        };
        client.send_message(
            EVENT_CHANNEL,
            wire::encode_request(welcome.epoch, &request)?,
        );
        self.sequence = sequence;
        self.pending = Some((sequence, current));
        self.next_allowed = current + Duration::from_millis(150);
        stats.dialogue_requests_sent += 1;
        Ok(true)
    }
    pub(super) fn copy_to(&self, state: &mut State) {
        state.dialogue = self.view.clone();
        state.guide = self.presence.clone();
        state.dialogue_reply = self.reply.clone();
        state.dialogue_pending = self.pending.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::Open;
    fn setup(capacity: usize) -> (Client, Welcome) {
        let profile = Arc::new(
            Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-dialogue.json"
            ))
            .unwrap(),
        );
        let (_, actions) = mpsc::channel(capacity);
        (
            Client { profile, actions },
            Welcome {
                terrain_stream: true,
                id: 2,
                epoch: 7,
                world: [1; 32],
                origin: [0.; 3],
            },
        )
    }
    #[test]
    fn pending_action_requires_its_reply_and_the_referenced_authoritative_view() {
        let (mode, welcome) = setup(4);
        let profile = mode.profile.clone();
        let mut state = ClientState::new(mode).unwrap();
        let actor = actor(&welcome).unwrap();
        let guide = EntityKey {
            entity: EntityId::new(1).unwrap(),
            ..actor
        };
        let closed = View {
            actor,
            profile_hash: profile.hash(),
            revision: 0,
            open: None,
        };
        let reply = Reply {
            actor,
            profile_hash: profile.hash(),
            sequence: 1,
            view_revision: 1,
            rejection: None,
        };
        let mut stats = TransportStats::default();
        state
            .receive(
                &wire::encode_view(&closed, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        state.sequence = 1;
        state.pending = Some((1, Instant::now()));
        state
            .receive(
                &wire::encode_reply(&reply, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        assert!(
            state.pending.is_some(),
            "Reply alone must not predict the new node"
        );
        let opened = View {
            revision: 1,
            open: Some(Open {
                guide,
                node: profile.profile().start().id().clone(),
            }),
            ..closed
        };
        state
            .receive(
                &wire::encode_view(&opened, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        assert!(state.pending.is_none());
        state.sequence = 2;
        state.pending = Some((2, Instant::now()));
        let rejected = Reply {
            sequence: 2,
            rejection: Some(crate::dialogue::Rejection::InvalidAction),
            ..reply
        };
        state
            .receive(
                &wire::encode_reply(&rejected, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        assert!(
            state.pending.is_none(),
            "Rejected choice can acknowledge unchanged view"
        );
        let mut view = State::default();
        state.copy_to(&mut view);
        assert_eq!(view.dialogue, Some(opened));
        assert_eq!(view.dialogue_reply, Some(rejected));
        assert!(!view.dialogue_pending);
    }
    #[test]
    fn client_queue_initial_state_and_pending_deadlines_remain_bounded() {
        let (mode, _) = setup(5);
        assert!(ClientState::new(mode).is_err());
        let (mode, welcome) = setup(4);
        let profile = mode.profile.clone();
        let mut state = ClientState::new(mode).unwrap();
        let now = Instant::now();
        let elapsed = Duration::from_secs(9);
        assert!(state.check(elapsed, false, now).is_ok());
        assert!(state.check(elapsed, true, now).is_err());
        let actor = actor(&welcome).unwrap();
        let mut stats = TransportStats::default();
        let closed = View {
            actor,
            profile_hash: profile.hash(),
            revision: 0,
            open: None,
        };
        state
            .receive(
                &wire::encode_view(&closed, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        assert!(
            state.check(elapsed, true, now).is_err(),
            "Presence frame is independently required"
        );
        let presence = Presence {
            actor,
            profile_hash: profile.hash(),
            guide: None,
        };
        state
            .receive(
                &wire::encode_presence(&presence, &profile).unwrap(),
                &welcome,
                &mut stats,
            )
            .unwrap();
        assert!(state.check(elapsed, true, now).is_ok());
        state.pending = Some((1, now));
        assert!(
            state
                .check(elapsed, true, now + Duration::from_secs(9))
                .is_err()
        );
        assert!(
            state
                .check(elapsed, true, now + Duration::from_secs(7))
                .is_ok()
        );
    }
}
