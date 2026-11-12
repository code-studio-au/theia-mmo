//! Bounded private explicit quest actions over the existing reliable channel.
use super::{
    ServerStats,
    progression::{ActionReply, Input, Port},
};
use crate::{
    Result,
    client::{State, TransportStats},
    quest_lifecycle_wire::{self as wire, Profile, Rejection},
    renet_transport::EVENT_CHANNEL,
    simulation::World,
    wire::Welcome,
};
use renet::{RenetClient, RenetServer};
use std::{sync::Arc, time::Duration};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::lifecycle::{Action, Command, Phase, State as Progress},
};
use tokio::{sync::mpsc, time::Instant};

#[derive(Default)]
pub(super) struct Peer {
    sent: Option<Progress>,
    sent_reply: Option<wire::Reply>,
    local_reply: Option<ActionReply>,
    sequence: u64,
    pending: Option<u64>,
}
impl Peer {
    pub(super) fn request(
        &mut self,
        world: &World,
        actor: u64,
        bytes: &[u8],
        port: &Port,
    ) -> Result<bool> {
        let request = wire::Request::decode(bytes)?;
        if world.entity_key(actor) != Some(request.actor) {
            return Err("Foreign quest action actor".into());
        }
        if request.sequence <= self.sequence {
            return Ok(false);
        }
        if request.sequence.saturating_sub(self.sequence) > 128 {
            return Err("Quest action sequence gap exceeds budget".into());
        }
        self.sequence = request.sequence;
        let view = port.state.borrow().clone();
        if self
            .pending
            .is_some_and(|seq| view.lifecycle_reply.is_some_and(|r| r.sequence == seq))
        {
            self.pending = None;
        }
        let rejection = if !world.quest_lifecycle_eligible(actor, request) {
            Some(Rejection::NotEntitled)
        } else if self.pending.is_some() || view.lifecycle.is_none() {
            Some(Rejection::Busy)
        } else {
            None
        };
        if let Some(reason) = rejection {
            self.refuse(
                request.sequence,
                view.lifecycle.as_ref().map_or(0, Progress::revision),
                reason,
            );
            return Ok(false);
        }
        match port.input.try_send(Input::QuestAction(request)) {
            Ok(()) => {
                self.pending = Some(request.sequence);
                Ok(true)
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.refuse(
                    request.sequence,
                    view.lifecycle.as_ref().map_or(0, Progress::revision),
                    Rejection::Busy,
                );
                Ok(false)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => Err("Quest action owner closed".into()),
        }
    }
    fn refuse(&mut self, sequence: u64, quest_revision: u64, reason: Rejection) {
        self.local_reply = Some(ActionReply {
            sequence,
            quest_revision,
            rejection: Some(reason),
        });
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn publish(
        &mut self,
        server: &mut RenetServer,
        id: u64,
        actor: EntityKey,
        profile: &Profile,
        progress: &Progress,
        reply: Option<ActionReply>,
        stats: &mut ServerStats,
    ) -> Result<bool> {
        if self
            .pending
            .is_some_and(|seq| reply.is_some_and(|r| r.sequence == seq))
        {
            self.pending = None;
        }
        let latest = [reply, self.local_reply]
            .into_iter()
            .flatten()
            .max_by_key(|r| r.sequence);
        let next_state = if self.sent.as_ref() != Some(progress) {
            Some(
                wire::State {
                    actor,
                    profile_hash: profile.hash(),
                    progress: progress.clone(),
                }
                .encode(profile)?,
            )
        } else {
            None
        };
        let next_reply = latest
            .map(|r| wire::Reply {
                actor,
                profile_hash: profile.hash(),
                sequence: r.sequence,
                quest_revision: r.quest_revision,
                rejection: r.rejection,
            })
            .filter(|r| {
                self.sent_reply
                    .as_ref()
                    .is_none_or(|old| r.sequence > old.sequence)
            });
        let reply_bytes = next_reply.map(|r| r.encode(profile)).transpose()?;
        let size =
            next_state.as_ref().map_or(0, Vec::len) + reply_bytes.as_ref().map_or(0, Vec::len);
        if size > 0 && !server.can_send_message(id, EVENT_CHANNEL, size) {
            return Ok(false);
        }
        if let Some(bytes) = next_state {
            server.send_message(id, EVENT_CHANNEL, bytes);
            self.sent = Some(progress.clone());
            stats.lifecycle_states_sent += 1;
        }
        if let Some(bytes) = reply_bytes {
            server.send_message(id, EVENT_CHANNEL, bytes);
            self.sent_reply = next_reply;
            stats.lifecycle_replies_sent += 1;
        }
        Ok(true)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Intent {
    pub expected_revision: u64,
    pub dialogue_revision: u64,
    pub npc: EntityKey,
    pub action: Action,
}
pub struct Client {
    pub profile: Arc<Profile>,
    pub actions: mpsc::Receiver<Intent>,
}
pub(super) struct ClientState {
    mode: Client,
    view: Option<wire::State>,
    reply: Option<wire::Reply>,
    sequence: u64,
    pending: Option<(u64, Instant, Intent)>,
    next_allowed: Instant,
}
pub(super) fn is_frame(bytes: &[u8]) -> bool {
    bytes.starts_with(wire::STATE_TAG) || bytes.starts_with(wire::REPLY_TAG)
}
fn actor(welcome: &Welcome) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(welcome.epoch).ok_or("Invalid quest admission epoch")?,
        entity: EntityId::new(welcome.id).ok_or("Invalid quest admission actor")?,
    })
}
impl ClientState {
    pub(super) fn new(mode: Client) -> Result<Self> {
        if mode.actions.max_capacity() > 4 {
            return Err("Quest action queue exceeds four requests".into());
        }
        Ok(Self {
            mode,
            view: None,
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
        if bytes.starts_with(wire::STATE_TAG) {
            let view = wire::State::decode(bytes, &self.mode.profile)?;
            view.validate_successor(&self.mode.profile, actor, self.view.as_ref())?;
            self.view = Some(view);
            stats.lifecycle_states_received += 1;
        } else if bytes.starts_with(wire::REPLY_TAG) {
            let reply = wire::Reply::decode(bytes, &self.mode.profile)?;
            reply.validate_successor(&self.mode.profile, actor, self.sequence, self.reply)?;
            self.reply = Some(reply);
            stats.lifecycle_replies_received += 1;
        } else {
            return Err("Unknown explicit quest observation".into());
        }
        if let Some((seq, _, intent)) = self.pending
            && let Some(reply) = self.reply.filter(|r| r.sequence == seq)
            && let Some(view) = self
                .view
                .as_ref()
                .filter(|v| v.progress.revision() >= reply.quest_revision)
        {
            if reply.rejection.is_none() {
                let phase = view
                    .progress
                    .progress(self.mode.profile.profile())
                    .map_err(|_| "Invalid committed quest phase")?
                    .phase;
                let valid = match intent.action {
                    Action::Accept => {
                        matches!(phase, Phase::Active | Phase::Ready | Phase::TurnedIn)
                    }
                    Action::TurnIn => phase == Phase::TurnedIn,
                };
                if !valid {
                    return Err("Successful quest reply lacks committed action state".into());
                }
            }
            self.pending = None;
        }
        Ok(())
    }
    pub(super) fn check(&self, elapsed: Duration, admitted: bool, current: Instant) -> Result<()> {
        if admitted && elapsed > Duration::from_secs(8) && self.view.is_none() {
            return Err("Server did not publish committed explicit quest".into());
        }
        if self.pending.is_some_and(|(_, started, _)| {
            current.saturating_duration_since(started) > Duration::from_secs(8)
        }) {
            return Err("Quest action acknowledgement timed out".into());
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
            || current < self.next_allowed
            || !client.can_send_message(EVENT_CHANNEL, wire::REQUEST_BYTES)
        {
            return Ok(false);
        }
        let Ok(intent) = self.mode.actions.try_recv() else {
            return Ok(false);
        };
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or("Quest action sequence exhausted")?;
        let request = wire::Request {
            actor: actor(welcome)?,
            sequence,
            command: Command {
                expected_revision: intent.expected_revision,
                action: intent.action,
            },
            dialogue_revision: intent.dialogue_revision,
            npc: intent.npc,
        };
        client.send_message(EVENT_CHANNEL, request.encode()?);
        self.sequence = sequence;
        self.pending = Some((sequence, current, intent));
        self.next_allowed = current + Duration::from_millis(150);
        stats.lifecycle_requests_sent += 1;
        Ok(true)
    }
    pub(super) fn copy_to(&self, state: &mut State) {
        state.quest_lifecycle = self.view.clone();
        state.quest_lifecycle_reply = self.reply;
        state.quest_lifecycle_pending = self.pending.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dialogue::{Action as Talk, Request as TalkRequest},
        renet_runtime::progression::Observation,
    };
    use tokio::sync::watch;
    const DATA: &[u8] = include_bytes!("../../../../content/gameplay/development-quest.json");
    fn profile() -> Arc<Profile> {
        Arc::new(Profile::from_json(DATA).unwrap())
    }
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn setup_client(capacity: usize) -> (ClientState, mpsc::Sender<Intent>, Welcome) {
        let (actions, requests) = mpsc::channel(capacity);
        let mode = Client {
            profile: profile(),
            actions: requests,
        };
        let welcome = Welcome {
            terrain_stream: true,
            id: 9,
            epoch: 7,
            world: [1; 32],
            origin: [0.; 3],
        };
        (ClientState::new(mode).unwrap(), actions, welcome)
    }
    fn view(
        profile: &Profile,
        status: theia_core::quests::lifecycle::Status,
        count: u32,
    ) -> wire::State {
        let revision = match status {
            theia_core::quests::lifecycle::Status::Available => 0,
            theia_core::quests::lifecycle::Status::Accepted => u64::from(count) + 1,
            theia_core::quests::lifecycle::Status::TurnedIn => u64::from(count) + 2,
        };
        wire::State {
            actor: key(9),
            profile_hash: profile.hash(),
            progress: Progress::restore(profile.profile(), revision, status, count).unwrap(),
        }
    }
    fn intent(action: Action) -> Intent {
        Intent {
            expected_revision: 0,
            dialogue_revision: 1,
            npc: key(1),
            action,
        }
    }
    #[test]
    fn client_waits_for_matching_reply_and_committed_action_state() {
        use theia_core::quests::lifecycle::Status;
        let (mut client, _, welcome) = setup_client(4);
        let p = client.mode.profile.clone();
        let mut stats = TransportStats::default();
        let available = view(&p, Status::Available, 0);
        client
            .receive(&available.encode(&p).unwrap(), &welcome, &mut stats)
            .unwrap();
        client.sequence = 1;
        client.pending = Some((1, Instant::now(), intent(Action::Accept)));
        let reply = wire::Reply {
            actor: key(9),
            profile_hash: p.hash(),
            sequence: 1,
            quest_revision: 1,
            rejection: None,
        };
        client
            .receive(&reply.encode(&p).unwrap(), &welcome, &mut stats)
            .unwrap();
        assert!(client.pending.is_some());
        assert_eq!(
            client.view.as_ref().unwrap().progress.status(),
            Status::Available
        );
        let accepted = view(&p, Status::Accepted, 0);
        client
            .receive(&accepted.encode(&p).unwrap(), &welcome, &mut stats)
            .unwrap();
        assert!(client.pending.is_none());
        let mut copy = State::default();
        client.copy_to(&mut copy);
        assert!(!copy.quest_lifecycle_pending);
        assert_eq!(copy.quest_lifecycle.unwrap(), accepted);
        assert_eq!(copy.quest_lifecycle_reply, Some(reply));
    }
    #[test]
    fn normal_refusal_acknowledges_unchanged_state_but_false_success_fails_closed() {
        use theia_core::quests::lifecycle::Status;
        let (mut client, _, welcome) = setup_client(4);
        let p = client.mode.profile.clone();
        let mut stats = TransportStats::default();
        let available = view(&p, Status::Available, 0);
        client
            .receive(&available.encode(&p).unwrap(), &welcome, &mut stats)
            .unwrap();
        client.sequence = 1;
        client.pending = Some((1, Instant::now(), intent(Action::TurnIn)));
        let refused = wire::Reply {
            actor: key(9),
            profile_hash: p.hash(),
            sequence: 1,
            quest_revision: 0,
            rejection: Some(Rejection::NotAvailable),
        };
        client
            .receive(&refused.encode(&p).unwrap(), &welcome, &mut stats)
            .unwrap();
        assert!(client.pending.is_none());
        assert_eq!(client.view.as_ref(), Some(&available));
        client.sequence = 2;
        client.pending = Some((2, Instant::now(), intent(Action::Accept)));
        let invalid = wire::Reply {
            sequence: 2,
            rejection: None,
            ..refused
        };
        assert!(
            client
                .receive(&invalid.encode(&p).unwrap(), &welcome, &mut stats)
                .is_err()
        );
    }
    #[test]
    fn client_queue_initial_state_and_pending_deadlines_are_bounded() {
        let (_, requests) = mpsc::channel(5);
        assert!(
            ClientState::new(Client {
                profile: profile(),
                actions: requests
            })
            .is_err()
        );
        let (mut client, _, _) = setup_client(4);
        let now = Instant::now();
        let elapsed = Duration::from_secs(9);
        assert!(client.check(elapsed, false, now).is_ok());
        assert!(client.check(elapsed, true, now).is_err());
        client.view = Some(view(
            &client.mode.profile,
            theia_core::quests::lifecycle::Status::Available,
            0,
        ));
        assert!(client.check(elapsed, true, now).is_ok());
        client.pending = Some((1, now, intent(Action::Accept)));
        assert!(
            client
                .check(elapsed, true, now + Duration::from_secs(7))
                .is_ok()
        );
        assert!(
            client
                .check(elapsed, true, now + Duration::from_secs(9))
                .is_err()
        );
    }
    #[test]
    fn sending_retains_painted_revisions_instead_of_rebinding_to_new_state() {
        let (mut mode, actions, welcome) = setup_client(4);
        mode.view = Some(view(
            &mode.mode.profile,
            theia_core::quests::lifecycle::Status::Accepted,
            3,
        ));
        let painted = intent(Action::Accept);
        actions.try_send(painted).unwrap();
        let mut client = RenetClient::new(super::super::connection_config());
        let mut stats = TransportStats::default();
        assert!(
            mode.send(&mut client, &welcome, Instant::now(), &mut stats)
                .unwrap()
        );
        let (_, _, captured) = mode.pending.unwrap();
        assert_eq!(captured.expected_revision, 0);
        assert_eq!(captured.dialogue_revision, 1);
        assert_eq!(captured.npc, painted.npc);
        assert_eq!(stats.lifecycle_requests_sent, 1);
        assert!(
            !mode
                .send(
                    &mut client,
                    &welcome,
                    Instant::now() + Duration::from_secs(1),
                    &mut stats
                )
                .unwrap()
        );
    }
    fn world_port() -> (
        World,
        u64,
        crate::dialogue::Guide,
        Port,
        mpsc::Receiver<Input>,
        watch::Sender<Observation>,
    ) {
        let (mut world, _) =
            crate::simulation::creatures::tests::fixture(theia_core::time::TickRate::Hz30);
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        world.enable_progression(xp.clone()).unwrap();
        let guide = world
            .spawn_development_guide(
                Arc::new(
                    crate::dialogue::Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8002., 8000.],
            )
            .unwrap();
        let p = profile();
        world.enable_quest_lifecycle(p.clone()).unwrap();
        let actor = world.join().unwrap();
        let (input, receiver) = mpsc::channel(super::super::progression::CAPACITY);
        let (publish, state) = watch::channel(Observation {
            lifecycle: Some(Progress::available(p.profile())),
            ..Observation::default()
        });
        let port = Port {
            journal: None,
            lifecycle: Some(p),
            quest: None,
            profile: xp,
            input,
            state,
        };
        (world, actor, guide, port, receiver, publish)
    }
    #[test]
    fn server_rejects_stale_guide_context_without_enqueuing_or_disconnect() {
        let (mut world, actor, guide, port, mut input, _) = world_port();
        let mut peer = Peer::default();
        let own = world.entity_key(actor).unwrap();
        let request = wire::Request {
            actor: own,
            sequence: 1,
            command: Command {
                expected_revision: 0,
                action: Action::Accept,
            },
            dialogue_revision: 1,
            npc: guide.key,
        };
        assert!(
            !peer
                .request(&world, actor, &request.encode().unwrap(), &port)
                .unwrap()
        );
        assert!(input.try_recv().is_err());
        assert_eq!(
            peer.local_reply.unwrap().rejection,
            Some(Rejection::NotEntitled)
        );
        world
            .interact(
                actor,
                &TalkRequest {
                    sequence: 1,
                    expected_revision: 0,
                    action: Talk::Open(guide.key),
                },
            )
            .unwrap();
        let fresh = wire::Request {
            sequence: 2,
            ..request
        };
        assert!(
            peer.request(&world, actor, &fresh.encode().unwrap(), &port)
                .unwrap()
        );
        assert!(matches!(input.try_recv().unwrap(),Input::QuestAction(r) if r==fresh));
        let foreign = wire::Request {
            sequence: 3,
            actor: EntityKey {
                entity: EntityId::new(99).unwrap(),
                ..own
            },
            ..request
        };
        assert!(
            peer.request(&world, actor, &foreign.encode().unwrap(), &port)
                .is_err()
        );
    }
    #[test]
    fn later_busy_refusal_cannot_be_rolled_back_by_earlier_owner_reply() {
        let (mut world, actor, guide, port, mut input, publish) = world_port();
        let mut peer = Peer::default();
        let own = world.entity_key(actor).unwrap();
        world
            .interact(
                actor,
                &TalkRequest {
                    sequence: 1,
                    expected_revision: 0,
                    action: Talk::Open(guide.key),
                },
            )
            .unwrap();
        let request = wire::Request {
            actor: own,
            sequence: 1,
            command: Command {
                expected_revision: 0,
                action: Action::Accept,
            },
            dialogue_revision: 1,
            npc: guide.key,
        };
        assert!(
            peer.request(&world, actor, &request.encode().unwrap(), &port)
                .unwrap()
        );
        input.try_recv().unwrap();
        assert!(
            !peer
                .request(
                    &world,
                    actor,
                    &wire::Request {
                        sequence: 2,
                        ..request
                    }
                    .encode()
                    .unwrap(),
                    &port
                )
                .unwrap()
        );
        assert!(input.try_recv().is_err());
        let p = port.lifecycle.as_ref().unwrap();
        let accepted = Progress::restore(
            p.profile(),
            1,
            theia_core::quests::lifecycle::Status::Accepted,
            0,
        )
        .unwrap();
        publish.send_modify(|s| {
            s.lifecycle = Some(accepted.clone());
            s.lifecycle_reply = Some(ActionReply {
                sequence: 1,
                quest_revision: 1,
                rejection: None,
            });
        });
        let mut server = RenetServer::new(super::super::connection_config());
        server.add_connection(1);
        let mut stats = ServerStats::default();
        assert!(
            peer.publish(
                &mut server,
                1,
                own,
                p,
                &accepted,
                port.state.borrow().lifecycle_reply,
                &mut stats
            )
            .unwrap()
        );
        assert_eq!(peer.sent_reply.unwrap().sequence, 2);
        assert_eq!(peer.sent_reply.unwrap().rejection, Some(Rejection::Busy));
        assert!(peer.pending.is_none());
        assert!(
            peer.publish(
                &mut server,
                1,
                own,
                p,
                &accepted,
                port.state.borrow().lifecycle_reply,
                &mut stats
            )
            .unwrap()
        );
        assert_eq!(stats.lifecycle_replies_sent, 1);
        assert!(
            !peer
                .request(
                    &world,
                    actor,
                    &wire::Request {
                        sequence: 2,
                        ..request
                    }
                    .encode()
                    .unwrap(),
                    &port
                )
                .unwrap()
        );
        assert!(input.try_recv().is_err());
    }
}

#[cfg(test)]
#[path = "quest_lifecycle/renet_tests.rs"]
mod renet_tests;
