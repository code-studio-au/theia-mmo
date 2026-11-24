//! One bounded original command in flight; observations remain server-owned.
use crate::{
    Result,
    client::{State, TransportStats},
    quest_journal::Profile,
    quest_journal_wire as wire,
    renet_transport::EVENT_CHANNEL,
    wire::Welcome,
};
use renet::RenetClient;
use std::{sync::Arc, time::Duration};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::journal::{Action, Command, Status},
};
use tokio::{sync::mpsc, time::Instant};

#[derive(Clone, Debug)]
pub struct Intent {
    pub npc: EntityKey,
    pub dialogue_revision: u64,
    /// Keep this original request ID when retrying a lost result after rejoin.
    pub command: Command,
}

pub struct Client {
    pub profile: Arc<Profile>,
    pub actions: mpsc::Receiver<Intent>,
}
pub(crate) struct ClientState {
    mode: Client,
    view: Option<wire::State>,
    reply: Option<wire::Reply>,
    pending: Option<(wire::Request, Instant)>,
    sequence: u64,
    next_allowed: Instant,
}
fn actor(welcome: &Welcome) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(welcome.epoch).ok_or("Invalid journal epoch")?,
        entity: EntityId::new(welcome.id).ok_or("Invalid journal actor")?,
    })
}
pub(crate) fn is_frame(bytes: &[u8]) -> bool {
    bytes.starts_with(wire::STATE_TAG) || bytes.starts_with(wire::REPLY_TAG)
}
impl ClientState {
    pub(crate) fn new(mode: Client) -> Result<Self> {
        if mode.actions.max_capacity() > 4 {
            return Err("Journal action queue exceeds four requests".into());
        }
        Ok(Self {
            mode,
            view: None,
            reply: None,
            pending: None,
            sequence: 0,
            next_allowed: Instant::now(),
        })
    }
    pub(crate) fn receive(
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
            stats.journal_states_received += 1;
            return Ok(());
        }
        let reply = wire::Reply::decode(bytes, &self.mode.profile)?;
        if self.reply.as_ref() == Some(&reply) {
            return Ok(());
        }
        let (request, _) = self.pending.as_ref().ok_or("Unsolicited journal reply")?;
        if reply.actor != actor
            || self
                .reply
                .as_ref()
                .is_some_and(|old| reply.sequence <= old.sequence)
        {
            return Err("Foreign or regressed journal reply".into());
        }
        reply.validate_request(&self.mode.profile, request)?;
        let view = self.view.as_ref().ok_or("Journal reply preceded state")?;
        if view.revision < reply.revision
            || view
                .journal
                .progress(&reply.quest)
                .map_or(0, |q| q.revision())
                < reply.quest_revision
        {
            return Err("Journal reply preceded its committed progress".into());
        }
        if reply.rejection.is_none() {
            let status = view
                .journal
                .progress(&reply.quest)
                .ok_or("Successful journal reply lacks quest")?
                .status();
            if request.command.action == Action::TurnIn && status != Status::Completed {
                return Err("Successful turn-in lacks completed quest".into());
            }
        }
        self.reply = Some(reply);
        self.pending = None;
        stats.journal_replies_received += 1;
        Ok(())
    }
    pub(crate) fn check(&self, elapsed: Duration, admitted: bool, current: Instant) -> Result<()> {
        if admitted && elapsed > Duration::from_secs(8) && self.view.is_none() {
            return Err("Server did not publish committed journal".into());
        }
        if self.pending.as_ref().is_some_and(|(_, start)| {
            current.saturating_duration_since(*start) > Duration::from_secs(8)
        }) {
            return Err("Journal acknowledgement timed out".into());
        }
        Ok(())
    }
    pub(crate) fn send(
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
            .ok_or("Journal sequence exhausted")?;
        let request = wire::Request {
            actor: actor(welcome)?,
            profile_hash: self.mode.profile.hash(),
            sequence,
            npc: intent.npc,
            dialogue_revision: intent.dialogue_revision,
            command: intent.command,
        };
        client.send_message(EVENT_CHANNEL, request.encode(&self.mode.profile)?);
        self.pending = Some((request, current));
        self.sequence = sequence;
        self.next_allowed = current + Duration::from_millis(150);
        stats.journal_requests_sent += 1;
        Ok(true)
    }
    pub(crate) fn copy_to(&self, state: &mut State) {
        state.quest_journal = self.view.clone();
        state.quest_journal_reply = self.reply.clone();
        state.quest_journal_pending = self.pending.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU128;
    use theia_core::{content::ContentId, quests::journal::Journal};
    fn setup() -> (ClientState, mpsc::Sender<Intent>, Welcome) {
        let xp = crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let profile = Arc::new(
            Profile::from_json(
                include_bytes!("../../../../../content/quests/development-journal.json"),
                &xp,
            )
            .unwrap(),
        );
        let (send, actions) = mpsc::channel(4);
        (
            ClientState::new(Client { profile, actions }).unwrap(),
            send,
            Welcome {
                terrain_stream: true,
                id: 9,
                epoch: 7,
                world: [1; 32],
                origin: [0.; 3],
            },
        )
    }
    fn request(client: &ClientState, welcome: &Welcome) -> wire::Request {
        wire::Request {
            actor: actor(welcome).unwrap(),
            profile_hash: client.mode.profile.hash(),
            sequence: 1,
            npc: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(1).unwrap(),
            },
            dialogue_revision: 1,
            command: Command {
                request_id: NonZeroU128::new(31).unwrap(),
                quest: ContentId::new("theia.dev.quest-introduction").unwrap(),
                npc: ContentId::new("theia.dev.guide").unwrap(),
                expected_revision: 0,
                action: Action::Accept,
            },
        }
    }
    fn view(client: &ClientState, welcome: &Welcome, journal: Journal, revision: u64) -> Vec<u8> {
        wire::State {
            actor: actor(welcome).unwrap(),
            profile_hash: client.mode.profile.hash(),
            revision,
            journal,
        }
        .encode(&client.mode.profile)
        .unwrap()
    }
    #[test]
    fn original_intent_stays_pending_until_matching_reply_and_prior_committed_state() {
        let (mut mode, send, welcome) = setup();
        let request = request(&mode, &welcome);
        send.try_send(Intent {
            npc: request.npc,
            dialogue_revision: 1,
            command: request.command.clone(),
        })
        .unwrap();
        let mut transport = RenetClient::new(crate::renet_transport::connection_config());
        let mut stats = TransportStats::default();
        assert!(
            !mode
                .send(&mut transport, &welcome, Instant::now(), &mut stats)
                .unwrap()
        );
        mode.receive(
            &view(&mode, &welcome, Journal::default(), 0),
            &welcome,
            &mut stats,
        )
        .unwrap();
        assert!(
            mode.send(&mut transport, &welcome, Instant::now(), &mut stats)
                .unwrap()
        );
        assert_eq!(
            mode.pending.as_ref().unwrap().0.command.request_id,
            request.command.request_id
        );
        assert!(
            !mode
                .send(
                    &mut transport,
                    &welcome,
                    Instant::now() + Duration::from_secs(1),
                    &mut stats
                )
                .unwrap()
        );
        let accepted = Journal::default()
            .propose_accept(
                mode.mode.profile.catalog(),
                &request.command.quest,
                &request.command.npc,
            )
            .unwrap()
            .journal;
        let reply = wire::Reply {
            actor: request.actor,
            profile_hash: request.profile_hash,
            sequence: 1,
            request_id: request.command.request_id,
            revision: 1,
            quest: request.command.quest.clone(),
            quest_revision: 1,
            rejection: None,
        };
        assert!(
            mode.receive(
                &reply.encode(&mode.mode.profile).unwrap(),
                &welcome,
                &mut stats
            )
            .is_err(),
            "An acknowledgement cannot invent missing committed state"
        );
        assert!(mode.pending.is_some());
        mode.receive(&view(&mode, &welcome, accepted, 1), &welcome, &mut stats)
            .unwrap();
        assert!(
            mode.pending.is_some(),
            "A snapshot alone does not identify the accepted intent"
        );
        let mut wrong = reply.clone();
        wrong.request_id = NonZeroU128::new(32).unwrap();
        assert!(
            mode.receive(
                &wrong.encode(&mode.mode.profile).unwrap(),
                &welcome,
                &mut stats
            )
            .is_err()
        );
        mode.receive(
            &reply.encode(&mode.mode.profile).unwrap(),
            &welcome,
            &mut stats,
        )
        .unwrap();
        assert!(mode.pending.is_none());
        mode.receive(
            &reply.encode(&mode.mode.profile).unwrap(),
            &welcome,
            &mut stats,
        )
        .unwrap();
        assert_eq!(stats.journal_replies_received, 1);
    }
    #[test]
    fn normal_refusal_clears_only_its_original_pending_intent_without_progress() {
        let (mut mode, _, welcome) = setup();
        let request = request(&mode, &welcome);
        let mut stats = TransportStats::default();
        mode.receive(
            &view(&mode, &welcome, Journal::default(), 0),
            &welcome,
            &mut stats,
        )
        .unwrap();
        mode.pending = Some((request.clone(), Instant::now()));
        let reply = wire::Reply {
            actor: request.actor,
            profile_hash: request.profile_hash,
            sequence: 1,
            request_id: request.command.request_id,
            revision: 0,
            quest: request.command.quest.clone(),
            quest_revision: 0,
            rejection: Some(wire::Rejection::NotEntitled),
        };
        mode.receive(
            &reply.encode(&mode.mode.profile).unwrap(),
            &welcome,
            &mut stats,
        )
        .unwrap();
        assert!(mode.pending.is_none());
        assert_eq!(mode.view.as_ref().unwrap().revision, 0);
        let mut state = State::default();
        mode.copy_to(&mut state);
        assert!(!state.quest_journal_pending);
        assert_eq!(state.quest_journal_reply, Some(reply));
        let (actions, requests) = mpsc::channel(5);
        drop(actions);
        assert!(
            ClientState::new(Client {
                profile: mode.mode.profile.clone(),
                actions: requests
            })
            .is_err()
        );
    }
}
