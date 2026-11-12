//! Owner-private expanded journals on the existing bounded reliable channel.
//! Queue acceptance never publishes success; only committed owner views do.
mod client;
use super::{ServerStats, progression::Port};
use crate::{
    Result, quest_journal::Profile, quest_journal_wire as wire, renet_transport::EVENT_CHANNEL,
    simulation::World,
};
pub use client::{Client, Intent};
pub(super) use client::{ClientState, is_frame};
use renet::RenetServer;
use std::sync::Arc;
use theia_core::identity::EntityKey;

pub(super) struct Peer {
    actor: EntityKey,
    profile: Arc<Profile>,
    router: crate::quest_journal::Router,
    sent: Option<wire::State>,
    reply: Option<wire::Reply>,
    received: u64,
}

impl Peer {
    pub(super) fn new(world: &World, actor: EntityKey) -> Result<Option<Self>> {
        let Some(profile) = world.quest_journal_profile() else {
            return Ok(None);
        };
        Ok(Some(Self {
            actor,
            profile: profile.clone(),
            router: world.quest_journal_router(actor)?,
            sent: None,
            reply: None,
            received: 0,
        }))
    }
    pub(super) fn interact(
        &mut self,
        world: &mut World,
        request: &crate::dialogue::Request,
        port: &Port,
    ) -> std::result::Result<(), crate::dialogue::Rejection> {
        self.router.interact(world, request, port).map(|_| ())
    }
    pub(super) fn kills(
        &mut self,
        world: &World,
        events: &[crate::simulation::combat::Event],
        port: &Port,
    ) -> Result<usize> {
        self.router.forward_kills(world, events, port)
    }
    pub(super) fn areas(&mut self, world: &World, port: &Port) -> Result<bool> {
        self.router.observe_areas(world, port)
    }
    pub(super) fn request(&mut self, world: &World, bytes: &[u8], port: &Port) -> Result<bool> {
        let request = wire::Request::decode(bytes, &self.profile)?;
        if request.actor != self.actor
            || world.entity_key(self.actor.entity.value()) != Some(self.actor)
        {
            return Err("Foreign journal request actor".into());
        }
        if request.sequence <= self.received {
            return Ok(false);
        }
        if request.sequence.saturating_sub(self.received) > 128 {
            return Err("Journal request sequence gap exceeds budget".into());
        }
        self.received = request.sequence;
        // An unsent reply still owns its slot. Never overwrite accepted work or
        // its acknowledgement with a second request under downstream pressure.
        if self.reply.is_some() {
            return Err("Journal reply is still awaiting publication".into());
        }
        match self.router.command(world, &request, port) {
            Ok(()) => Ok(true),
            Err(rejection) => {
                let state = port.state.borrow();
                let journal = state.journal.as_ref();
                self.reply = Some(wire::Reply {
                    actor: request.actor,
                    profile_hash: self.profile.hash(),
                    sequence: request.sequence,
                    request_id: request.command.request_id,
                    revision: journal.map_or(0, |s| s.revision),
                    quest: request.command.quest.clone(),
                    quest_revision: journal
                        .and_then(|s| s.journal.progress(&request.command.quest))
                        .map_or(0, |q| q.revision()),
                    rejection: Some(rejection),
                });
                Ok(false)
            }
        }
    }
    pub(super) fn publish(
        &mut self,
        server: &mut RenetServer,
        client: u64,
        world: &World,
        actor: EntityKey,
        port: &Port,
        stats: &mut ServerStats,
    ) -> Result<bool> {
        if self.reply.is_none() {
            self.reply = self.router.take_reply(world, port)?;
        }
        let owner = port.state.borrow();
        let Some(journal) = &owner.journal else {
            return Ok(true);
        };
        if journal.catalog != self.profile.hash() || port.journal != Some(journal.catalog) {
            return Err("Committed journal publication profile mismatch".into());
        }
        let state = wire::State {
            actor,
            profile_hash: journal.catalog,
            revision: journal.revision,
            journal: journal.journal.clone(),
        };
        if let Some(old) = &self.sent {
            state.validate_successor(&self.profile, actor, Some(old))?;
        }
        let bytes = (self.sent.as_ref() != Some(&state))
            .then(|| state.encode(&self.profile))
            .transpose()?;
        let reply = self
            .reply
            .as_ref()
            .map(|r| r.encode(&self.profile))
            .transpose()?;
        let size = bytes.as_ref().map_or(0, Vec::len) + reply.as_ref().map_or(0, Vec::len);
        if size > 0 && !server.can_send_message(client, EVENT_CHANNEL, size) {
            return Ok(false);
        }
        // State precedes its acknowledgement on the same ordered channel.
        if let Some(bytes) = bytes {
            server.send_message(client, EVENT_CHANNEL, bytes);
            self.sent = Some(state);
            stats.journal_states_sent += 1;
        }
        if let Some(bytes) = reply {
            server.send_message(client, EVENT_CHANNEL, bytes);
            self.reply = None;
            stats.journal_replies_sent += 1;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dialogue::{Action as Talk, Request as TalkRequest},
        renet_runtime::progression::{
            Input, JournalObservation, JournalOperation, JournalReply, Observation,
        },
    };
    use std::num::NonZeroU128;
    use theia_core::{
        content::ContentId,
        quests::journal::{Action, Command, Journal},
        time::TickRate,
    };
    use tokio::sync::{mpsc, watch};

    fn fixture() -> (
        World,
        EntityKey,
        Peer,
        Port,
        mpsc::Receiver<Input>,
        watch::Sender<Observation>,
    ) {
        let (mut world, _) = crate::simulation::creatures::tests::fixture(TickRate::Hz30);
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
        world
            .spawn_development_guide(
                Arc::new(
                    crate::dialogue::Profile::from_json(include_bytes!(
                        "../../../../content/quests/development-journal-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8002., 8000.],
            )
            .unwrap();
        let profile = Arc::new(
            Profile::from_json(
                include_bytes!("../../../../content/quests/development-journal.json"),
                &xp,
            )
            .unwrap(),
        );
        world
            .enable_quest_journal(profile.clone(), xp.clone())
            .unwrap();
        let actor = world.join().unwrap();
        let actor = world.entity_key(actor).unwrap();
        let peer = Peer::new(&world, actor).unwrap().unwrap();
        let (input, receiver) = mpsc::channel(super::super::progression::CAPACITY);
        let (publish, state) = watch::channel(Observation {
            progress: Some(theia_core::progression::State::new(xp.profile(), 0, 0).unwrap()),
            journal: Some(JournalObservation {
                catalog: profile.hash(),
                revision: 0,
                journal: Journal::default(),
                reply: None,
            }),
            ..Observation::default()
        });
        let port = Port {
            journal: Some(profile.hash()),
            profile: xp,
            quest: None,
            lifecycle: None,
            input,
            state,
        };
        (world, actor, peer, port, receiver, publish)
    }
    fn command(world: &World, actor: EntityKey, profile: &Profile) -> wire::Request {
        wire::Request {
            actor,
            profile_hash: profile.hash(),
            sequence: 1,
            npc: world.visible_guide(actor.entity.value()).unwrap().key,
            dialogue_revision: 1,
            command: Command {
                request_id: NonZeroU128::new(17).unwrap(),
                quest: ContentId::new("theia.dev.quest-introduction").unwrap(),
                npc: ContentId::new("theia.dev.guide").unwrap(),
                expected_revision: 0,
                action: Action::Accept,
            },
        }
    }
    #[test]
    fn publication_waits_for_commit_and_retains_reply_when_reliable_budget_is_full() {
        let (mut world, actor, mut peer, port, mut inputs, publish) = fixture();
        let request = command(&world, actor, &peer.profile);
        peer.interact(
            &mut world,
            &TalkRequest {
                sequence: 1,
                expected_revision: 0,
                action: Talk::Open(request.npc),
            },
            &port,
        )
        .unwrap();
        let mut server = RenetServer::new(super::super::connection_config());
        server.add_connection(1);
        let mut stats = ServerStats::default();
        peer.publish(&mut server, 1, &world, actor, &port, &mut stats)
            .unwrap();
        let encoded = request.encode(&peer.profile).unwrap();
        assert!(peer.request(&world, &encoded, &port).unwrap());
        assert!(
            !peer.request(&world, &encoded, &port).unwrap(),
            "Duplicate intent must not be queued twice"
        );
        let Input::Journal(queued) = inputs.try_recv().unwrap() else {
            panic!("Expected journal command")
        };
        assert!(matches!(queued.operation, JournalOperation::Command(_)));
        assert!(inputs.try_recv().is_err());
        assert!(
            peer.publish(&mut server, 1, &world, actor, &port, &mut stats)
                .unwrap()
        );
        assert_eq!(
            stats.journal_replies_sent, 0,
            "Queue acceptance cannot acknowledge success"
        );
        let accepted = Journal::default()
            .propose_accept(
                peer.profile.catalog(),
                &request.command.quest,
                &request.command.npc,
            )
            .unwrap()
            .journal;
        publish.send_modify(|s| {
            let j = s.journal.as_mut().unwrap();
            j.revision = 1;
            j.journal = accepted.clone();
            j.reply = Some(JournalReply {
                sequence: queued.sequence,
                rejection: None,
            });
        });
        server.send_message(
            1,
            EVENT_CHANNEL,
            vec![0; crate::renet_transport::CHANNEL_MEMORY_BYTES - 128],
        );
        assert!(
            !peer
                .publish(&mut server, 1, &world, actor, &port, &mut stats)
                .unwrap()
        );
        assert_eq!(stats.journal_replies_sent, 0);
        assert!(
            peer.reply.is_some(),
            "Taken committed reply must survive failed publication"
        );
        assert_eq!(peer.sent.as_ref().unwrap().revision, 0);
        // Explicit synthetic downstream recovery; normal runtime disconnects a
        // slow peer and lets the durable character owner finish retirement.
        let mut recovered = RenetServer::new(super::super::connection_config());
        recovered.add_connection(1);
        assert!(
            peer.publish(&mut recovered, 1, &world, actor, &port, &mut stats)
                .unwrap()
        );
        assert_eq!(stats.journal_states_sent, 2);
        assert_eq!(stats.journal_replies_sent, 1);
        assert_eq!(peer.sent.as_ref().unwrap().journal, accepted);
        assert!(peer.reply.is_none());
        assert!(
            peer.publish(&mut recovered, 1, &world, actor, &port, &mut stats)
                .unwrap()
        );
        assert_eq!(stats.journal_replies_sent, 1);
    }
    #[test]
    fn foreign_actor_sequence_gap_and_mismatched_owner_cannot_publish_or_queue() {
        let (world, actor, mut peer, mut port, mut inputs, publish) = fixture();
        let mut request = command(&world, actor, &peer.profile);
        request.actor.entity =
            theia_core::identity::EntityId::new(actor.entity.value() + 1).unwrap();
        assert!(
            peer.request(&world, &request.encode(&peer.profile).unwrap(), &port)
                .is_err()
        );
        request.actor = actor;
        request.sequence = 129;
        assert!(
            peer.request(&world, &request.encode(&peer.profile).unwrap(), &port)
                .is_err()
        );
        assert!(inputs.try_recv().is_err());
        // A fixed step without earned kills must not require SQL initialization.
        publish.send_modify(|s| s.journal = None);
        assert_eq!(peer.kills(&world, &[], &port).unwrap(), 0);
        publish.send_modify(|s| {
            s.journal = Some(JournalObservation {
                catalog: [9; 32],
                revision: 0,
                journal: Journal::default(),
                reply: None,
            })
        });
        let mut server = RenetServer::new(super::super::connection_config());
        server.add_connection(1);
        assert!(
            peer.publish(
                &mut server,
                1,
                &world,
                actor,
                &port,
                &mut ServerStats::default()
            )
            .is_err()
        );
        port.journal = Some([0; 32]);
        assert!(super::super::progression::Peer::new(port).is_err());
    }
}

#[cfg(test)]
#[path = "quest_journal/renet_tests.rs"]
mod renet_tests;
