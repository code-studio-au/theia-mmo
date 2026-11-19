//! Real encrypted two-peer runtime; explicit in-memory commit/identity fixture.
//! OIDC/SQL retirement is tested separately. No native or normal-launch claim.
use super::*;
use crate::{
    client::State,
    dialogue::{Action as Talk, Profile as TalkProfile},
    renet_runtime::{self as net, AccountAdmissions, AdmissionLease, PlayerAdmission},
    test_relay as relay,
    wire::Intent as Movement,
};
use net::progression::{
    Input, JournalObservation, JournalOperation, JournalRejection, JournalReply, Port,
};
use std::{
    collections::BTreeSet,
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
    time::Duration,
};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    identity::EntityKey,
    progression,
    quests::{
        catalog::EventKey,
        journal::{Action, Command, Error as Rule, Journal, Status},
    },
    time::TickRate,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
fn id(s: &str) -> theia_core::content::ContentId {
    theia_core::content::ContentId::new(s).unwrap()
}
struct Lease(Port);
impl AdmissionLease for Lease {
    fn progression(&self) -> Result<Option<Port>> {
        Ok(Some(self.0.clone()))
    }
    fn live(&self) -> Result<bool> {
        Ok(true)
    }
    fn record(&self, _: EntityKey, _: GlobalPosition, _: u64) -> Result<()> {
        Ok(())
    }
}
struct Policy {
    ports: [Port; 2],
    position: GlobalPosition,
    used: Mutex<[bool; 2]>,
}
impl AccountAdmissions for Policy {
    fn consume(self: Arc<Self>, client: u64, data: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        if !(1..=2).contains(&client) || data[0] != client as u8 {
            return Ok(None);
        }
        let index = client as usize - 1;
        let mut used = self.used.lock().unwrap();
        if used[index] {
            return Ok(None);
        }
        used[index] = true;
        Ok(Some(PlayerAdmission {
            position: self.position,
            lease: Box::new(Lease(self.ports[index].clone())),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; 8]> {
        let mut p = [None; 8];
        p[0] = Some(self.position);
        p[1] = Some(self.position);
        Ok(p)
    }
}
fn synthetic_owner(
    xp: Arc<crate::progression_wire::Profile>,
    profile: Arc<Profile>,
) -> (Port, JoinHandle<()>) {
    let (input, mut inputs) = mpsc::channel(net::progression::CAPACITY);
    let (publish, state) = watch::channel(net::progression::Observation::default());
    let port = Port {
        journal: Some(profile.hash()),
        profile: xp.clone(),
        quest: None,
        lifecycle: None,
        input,
        state,
    };
    let task = tokio::spawn(async move {
        let mut experience = progression::State::new(xp.profile(), 0, 0).unwrap();
        let mut journal = Journal::default();
        let mut revision = 0;
        let mut sequence = 0;
        let mut reply = None;
        let mut seen = BTreeSet::new();
        publish.send_modify(|s| {
            s.progress = Some(experience.clone());
            s.journal = Some(JournalObservation {
                catalog: profile.hash(),
                revision,
                journal: journal.clone(),
                reply: None,
            });
        });
        while let Some(input) = inputs.recv().await {
            let Input::Journal(input) = input else {
                panic!("Expanded mode must not split legacy XP/quest input")
            };
            assert_eq!(input.catalog, profile.hash());
            assert!(input.sequence > sequence);
            sequence = input.sequence;
            match input.operation {
                JournalOperation::Command(command) => {
                    let current = journal.progress(&command.quest).map_or(0, |q| q.revision());
                    let proposal = if current != command.expected_revision {
                        Err(Rule::StaleRevision)
                    } else {
                        match command.action {
                            Action::Accept => journal.propose_accept(
                                profile.catalog(),
                                &command.quest,
                                &command.npc,
                            ),
                            Action::TurnIn => journal.propose_turn_in(
                                profile.catalog(),
                                &command.quest,
                                &command.npc,
                                command.expected_revision,
                            ),
                        }
                    };
                    let rejection = match proposal {
                        Ok(proposal) => {
                            journal = proposal.journal;
                            revision += 1;
                            if let Some(reward) = proposal.reward
                                && reward.experience > 0
                            {
                                experience = xp
                                    .profile()
                                    .propose_grant(&experience, reward.experience)
                                    .unwrap()
                                    .state;
                            }
                            None
                        }
                        Err(rule) => Some(JournalRejection::Rule(rule)),
                    };
                    reply = Some(JournalReply {
                        sequence,
                        rejection,
                    });
                }
                JournalOperation::Event { id, event } => {
                    assert!(!matches!(event, EventKey::Kill(_)));
                    if seen.insert((0, id.get().to_le_bytes())) {
                        journal = journal
                            .propose_trusted_event(profile.catalog(), &event)
                            .unwrap()
                            .journal;
                        revision += 1;
                    }
                }
                JournalOperation::Areas { id, observation } => {
                    if seen.insert((2, id.get().to_le_bytes())) {
                        journal = journal
                            .propose_reached_areas(profile.catalog(), &observation)
                            .unwrap()
                            .journal;
                        revision = journal.next_head_revision(revision).unwrap();
                    }
                }
                JournalOperation::Kill(event) => {
                    if seen.insert((1, event.id.bytes())) {
                        journal = journal
                            .propose_trusted_event(
                                profile.catalog(),
                                &EventKey::Kill(event.creature.clone()),
                            )
                            .unwrap()
                            .journal;
                        revision += 1;
                        if xp.profile().kill_experience(&event.creature).is_some() {
                            experience = xp
                                .profile()
                                .propose_kill(&experience, &event.creature)
                                .unwrap()
                                .state;
                        }
                    }
                }
            }
            publish.send_modify(|s| {
                s.progress = Some(experience.clone());
                s.journal = Some(JournalObservation {
                    catalog: profile.hash(),
                    revision,
                    journal: journal.clone(),
                    reply: reply.clone(),
                });
            });
        }
    });
    (port, task)
}
struct ClientPeer {
    state: watch::Receiver<State>,
    talk: mpsc::Sender<net::dialogue::Intent>,
    quest: mpsc::Sender<Intent>,
    casts: mpsc::Sender<net::CastChoice>,
    stop: watch::Sender<bool>,
    task: JoinHandle<Result<()>>,
    events: JoinHandle<()>,
    _commands: mpsc::Sender<crate::prediction_wire::Commands>,
    _movement: watch::Sender<Movement>,
    _ready: watch::Sender<Option<Vec<ChunkId>>>,
    _choices: watch::Sender<Option<crate::targeting::Choice>>,
}
impl ClientPeer {
    #[allow(clippy::too_many_arguments)]
    fn start(
        address: SocketAddr,
        ticket: Vec<u8>,
        identity: [u8; 32],
        profile: Arc<Profile>,
        xp: Arc<crate::progression_wire::Profile>,
        talk_profile: Arc<TalkProfile>,
        catalog: Arc<theia_core::abilities::Catalog>,
        chunks: Vec<ChunkId>,
    ) -> Self {
        let (commands, command_queue) = mpsc::channel(8);
        let (movement, input) = watch::channel(Movement::default());
        let (ready, readiness) = watch::channel(Some(chunks));
        let (choices, selected) = watch::channel(None);
        let (talk, talks) = mpsc::channel(4);
        let (quest, quests) = mpsc::channel(4);
        let (casts, requests) = mpsc::channel(8);
        let (events, mut batches) = mpsc::channel(16);
        let drain = tokio::spawn(async move { while batches.recv().await.is_some() {} });
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            let result = net::run_client_gameplay(
                address,
                &ticket,
                input,
                states.clone(),
                shutdown,
                identity,
                net::GameplayClient {
                    avatar_life: None,
                    public_attacks: None,
                    classic_source: None,
                    dodge: None,
                    player_life: None,
                    lifecycle: None,
                    journal: Some(Client {
                        profile,
                        actions: quests,
                    }),
                    dialogue: Some(net::dialogue::Client {
                        profile: talk_profile,
                        actions: talks,
                    }),
                    ready: readiness,
                    commands: Some(command_queue),
                    choices: Some(selected),
                    combat: Some(net::CombatActions {
                        melee: None,
                        catalog,
                        casts: requests,
                        events,
                    }),
                    quest: None,
                    progression: Some(xp),
                    inventory: None,
                    statuses: None,
                },
            )
            .await;
            if let Err(error) = &result {
                states.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            state,
            talk,
            quest,
            casts,
            stop,
            task,
            events: drain,
            _commands: commands,
            _movement: movement,
            _ready: ready,
            _choices: choices,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let state = self.state.borrow().clone();
                assert!(state.error.is_none(), "{:?}", state.error);
                if predicate(&state) {
                    return state;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .expect("Encrypted quest condition timed out")
    }
    async fn talk(&mut self, revision: u64, action: Talk) {
        self.talk
            .try_send(net::dialogue::Intent {
                expected_revision: revision,
                action,
            })
            .unwrap();
        self.until(|s| {
            s.dialogue
                .as_ref()
                .is_some_and(|v| v.revision == revision + 1)
                && !s.dialogue_pending
        })
        .await;
    }
    async fn action(&mut self, sequence: u64, intent: Intent) -> State {
        self.quest.try_send(intent).unwrap();
        self.until(|s| {
            s.quest_journal_reply
                .as_ref()
                .is_some_and(|r| r.sequence == sequence)
                && !s.quest_journal_pending
        })
        .await
    }
    async fn ask(&mut self, guide: EntityKey, name: &str) {
        let view = self.state.borrow().dialogue.clone().unwrap();
        if view.open.is_some() {
            self.talk(view.revision, Talk::Close).await;
        }
        let revision = self.state.borrow().dialogue.as_ref().unwrap().revision;
        self.talk(revision, Talk::Open(guide)).await;
        let revision = self.state.borrow().dialogue.as_ref().unwrap().revision;
        self.talk(
            revision,
            Talk::Choose(id(&format!("theia.dev.interaction.{name}"))),
        )
        .await;
    }
    async fn kill(&mut self, expected_xp: u32, expected_count: Option<u32>) {
        let state = self
            .until(|s| {
                s.combat
                    .as_ref()
                    .is_some_and(|c| c.targets.iter().any(|t| t.current > 0))
            })
            .await;
        let target = state
            .combat
            .unwrap()
            .targets
            .into_iter()
            .find(|t| t.current > 0)
            .unwrap()
            .entity;
        let revision = self.state.borrow().quest_journal.as_ref().unwrap().revision;
        self.casts
            .try_send(net::CastChoice {
                ability: id("theia.dev.arcane-bolt"),
                target,
            })
            .unwrap();
        self.until(|s| {
            s.quest_journal.as_ref().is_some_and(|j| {
                j.revision > revision
                    && expected_count.is_none_or(|n| {
                        j.journal
                            .progress(&id("theia.dev.quest-follow-up"))
                            .is_some_and(|q| {
                                q.count(&id("theia.dev.quest-follow-up.objective.wolves"))
                                    == Some(n)
                            })
                    })
            }) && s
                .progression
                .as_ref()
                .is_some_and(|p| p.progress.total() == expected_xp)
        })
        .await;
    }
    async fn close(self) {
        self.stop.send_replace(true);
        self.task.await.unwrap().unwrap();
        self.events.await.unwrap();
    }
}
fn intent(
    sequence: u64,
    quest: &str,
    revision: u64,
    dialogue_revision: u64,
    npc: EntityKey,
    action: Action,
) -> Intent {
    Intent {
        npc,
        dialogue_revision,
        command: Command {
            request_id: std::num::NonZeroU128::new(u128::from(sequence)).unwrap(),
            quest: id(quest),
            npc: id("theia.dev.guide"),
            expected_revision: revision,
            action,
        },
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
#[tokio::test]
async fn encrypted_impaired_two_peer_journals_preserve_stages_optional_xp_and_private_replies_at_all_rates()
 {
    for (rate, no_xp) in [
        (TickRate::Hz20, false),
        (TickRate::Hz30, false),
        (TickRate::Hz60, false),
        (TickRate::Hz30, true),
    ] {
        let (mut world, _) = crate::simulation::creatures::tests::fixture(rate);
        let mut abilities: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        abilities["global_cooldown_ms"] = 0.into();
        abilities["abilities"][0]["cast_ms"] = 0.into();
        abilities["abilities"][0]["cooldown_ms"] = 0.into();
        abilities["abilities"][0]["effect"]["amount"] = 100.into();
        let bytes = serde_json::to_vec(&abilities).unwrap();
        world.enable_combat(&bytes, 100, 1000).unwrap();
        let catalog = Arc::new(theia_core::abilities::Catalog::from_json(&bytes).unwrap());
        let mut xp_bytes =
            include_bytes!("../../../../../content/gameplay/development-progression.json").to_vec();
        if no_xp {
            let mut raw: serde_json::Value = serde_json::from_slice(&xp_bytes).unwrap();
            raw["kill_rewards"][0]["creature"] = "theia.dev.other-region-creature".into();
            xp_bytes = serde_json::to_vec(&raw).unwrap();
        }
        let xp = Arc::new(crate::progression_wire::Profile::from_json(&xp_bytes).unwrap());
        let kill_xp = if no_xp { 0 } else { 25 };
        let talk = Arc::new(
            TalkProfile::from_json(include_bytes!(
                "../../../../../content/quests/development-journal-dialogue.json"
            ))
            .unwrap(),
        );
        let guide = world
            .spawn_development_guide(talk.clone(), [8002., 8000.])
            .unwrap();
        let profile = Arc::new(
            Profile::from_json(
                include_bytes!("../../../../../content/quests/development-journal.json"),
                &xp,
            )
            .unwrap(),
        );
        world
            .enable_quest_journal(profile.clone(), xp.clone())
            .unwrap();
        world.enable_prediction().unwrap();
        let identity = world.identity();
        let chunks = world
            .procedural()
            .unwrap()
            .patch(guide.position)
            .unwrap()
            .ids
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let (a, owner_a) = synthetic_owner(xp.clone(), profile.clone());
        let (b, owner_b) = synthetic_owner(xp.clone(), profile.clone());
        let policy = Arc::new(Policy {
            ports: [a, b],
            position: guide.position,
            used: Mutex::new([false; 2]),
        });
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "journal-impaired",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let mut one = [0; 256];
        one[0] = 1;
        let mut two = [0; 256];
        two[0] = 2;
        let token_a = issuer.issue(1, Some(&one), 120).unwrap().bytes().to_vec();
        let token_b = issuer.issue(2, Some(&two), 120).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport, shutdown, world, policy,
        ));
        let mut a = ClientPeer::start(
            addresses[0],
            token_a,
            identity,
            profile.clone(),
            xp.clone(),
            talk.clone(),
            catalog.clone(),
            chunks.clone(),
        );
        let mut b = ClientPeer::start(
            addresses[1],
            token_b,
            identity,
            profile,
            xp,
            talk,
            catalog,
            chunks,
        );
        for client in [&mut a, &mut b] {
            client
                .until(|s| {
                    s.quest_journal.is_some()
                        && s.progression.is_some()
                        && s.dialogue.is_some()
                        && s.chunk_ack.is_some()
                })
                .await;
        }
        a.kill(kill_xp, None).await;
        assert!(
            a.state
                .borrow()
                .quest_journal
                .as_ref()
                .unwrap()
                .journal
                .quests()
                .next()
                .is_none()
        );
        a.talk(0, Talk::Open(guide.key)).await;
        let first = "theia.dev.quest-introduction";
        let second = "theia.dev.quest-follow-up";
        a.action(1, intent(1, first, 0, 1, guide.key, Action::Accept))
            .await;
        b.talk(0, Talk::Open(guide.key)).await;
        let refused = b
            .action(1, intent(1, second, 0, 1, guide.key, Action::Accept))
            .await;
        assert_eq!(
            refused.quest_journal_reply.unwrap().rejection,
            Some(wire::Rejection::NotAvailable)
        );
        for name in ["road", "supplies", "report"] {
            a.ask(guide.key, name).await;
        }
        a.until(|s| {
            s.quest_journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id(first))
                .is_some_and(|q| q.status() == Status::Ready)
        })
        .await;
        let dialogue_revision = a.state.borrow().dialogue.as_ref().unwrap().revision;
        let completed = a
            .action(
                2,
                intent(2, first, 4, dialogue_revision, guide.key, Action::TurnIn),
            )
            .await;
        assert_eq!(completed.quest_journal_reply.unwrap().rejection, None);
        assert_eq!(
            completed.progression.unwrap().progress.total(),
            kill_xp + 25
        );
        a.action(
            3,
            intent(3, second, 0, dialogue_revision, guide.key, Action::Accept),
        )
        .await;
        for n in 1..=3 {
            a.kill(kill_xp + 25 + kill_xp * n, Some(n)).await;
        }
        a.ask(guide.key, "training").await;
        a.until(|s| {
            s.quest_journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&id(second))
                .is_some_and(|q| q.status() == Status::Ready)
        })
        .await;
        let dialogue_revision = a.state.borrow().dialogue.as_ref().unwrap().revision;
        let completed = a
            .action(
                4,
                intent(4, second, 5, dialogue_revision, guide.key, Action::TurnIn),
            )
            .await;
        assert_eq!(completed.quest_journal_reply.unwrap().rejection, None);
        assert_eq!(
            completed.progression.unwrap().progress.total(),
            50 + 4 * kill_xp
        );
        let journal = completed.quest_journal.unwrap();
        assert_eq!(journal.revision, 12);
        assert!(
            journal
                .journal
                .quests()
                .all(|(_, q)| q.status() == Status::Completed)
        );
        let private = b.state.borrow().clone();
        assert_eq!(private.progression.unwrap().progress.total(), 0);
        assert_eq!(private.quest_journal.unwrap().revision, 0);
        assert_eq!(
            a.state
                .borrow()
                .transport
                .as_ref()
                .unwrap()
                .journal_requests_sent,
            4
        );
        assert_eq!(
            b.state
                .borrow()
                .transport
                .as_ref()
                .unwrap()
                .journal_requests_sent,
            1
        );
        a.close().await;
        b.close().await;
        stop.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.journal_kills_forwarded, 4);
        assert_eq!(stats.journal_replies_sent, 5);
        assert_eq!(stats.journal_requests_forwarded, 5);
        for worker in workers {
            worker.await.unwrap();
        }
        for control in controls {
            let stats = &control.lock().unwrap().stats;
            assert!(stats.configured_drops > 0 && stats.duplicates > 0 && stats.reordered > 0);
            assert_eq!(stats.overflow, 0);
        }
        owner_a.await.unwrap();
        owner_b.await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_area_occupancy_advances_each_stage_privately_without_client_events_at_all_rates()
{
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (mut world, _) = crate::simulation::creatures::tests::fixture(rate);
        let mut abilities: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-abilities.json"
        ))
        .unwrap();
        abilities["global_cooldown_ms"] = 0.into();
        abilities["abilities"][0]["cast_ms"] = 0.into();
        abilities["abilities"][0]["cooldown_ms"] = 0.into();
        abilities["abilities"][0]["effect"]["amount"] = 100.into();
        let bytes = serde_json::to_vec(&abilities).unwrap();
        world.enable_combat(&bytes, 100, 1000).unwrap();
        let catalog = Arc::new(theia_core::abilities::Catalog::from_json(&bytes).unwrap());
        let xp_bytes =
            include_bytes!("../../../../../content/gameplay/development-progression.json").to_vec();
        let xp = Arc::new(crate::progression_wire::Profile::from_json(&xp_bytes).unwrap());
        let talk = Arc::new(
            TalkProfile::from_json(include_bytes!(
                "../../../../../content/quests/development-journal-dialogue.json"
            ))
            .unwrap(),
        );
        let guide = world
            .spawn_development_guide(talk.clone(), [8002., 8000.])
            .unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/quests/development-journal.json"
        ))
        .unwrap();
        raw["schema"] = 2.into();
        raw["quests"].as_array_mut().unwrap().truncate(1);
        for (index, stage) in raw["quests"][0]["stages"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            stage["objectives"] = serde_json::json!([{"kind":"reach_area",
                "id":format!("theia.dev.area-goal.{index}"),"label":"Reach","area":"theia.dev.area.test"}]);
        }
        let areas = serde_json::json!({"schema":1,"id":"theia.dev.areas","revision":1,"areas":[{
            "id":"theia.dev.area.test","name":"Test","min":[7990.,-64.,7990.],"max":[8010.,64.,8010.]}]});
        let mut areas = areas;
        let mut overlap = areas["areas"][0].clone();
        overlap["id"] = "theia.dev.area.overlap".into();
        areas["areas"].as_array_mut().unwrap().push(overlap);
        raw["quests"][0]["stages"][0]["objectives"]
            .as_array_mut()
            .unwrap()
            .push(
                serde_json::json!({"kind":"reach_area","id":"theia.dev.area-goal.overlap",
                "label":"Reach overlap","area":"theia.dev.area.overlap"}),
            );
        let profile = Arc::new(
            Profile::from_json_with_areas(
                &serde_json::to_vec(&raw).unwrap(),
                &serde_json::to_vec(&areas).unwrap(),
                &xp,
            )
            .unwrap(),
        );
        world.enable_prediction().unwrap();
        world
            .enable_quest_journal(profile.clone(), xp.clone())
            .unwrap();
        let identity = world.identity();
        let chunks = world
            .procedural()
            .unwrap()
            .patch(guide.position)
            .unwrap()
            .ids
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let (a, owner_a) = synthetic_owner(xp.clone(), profile.clone());
        let (b, owner_b) = synthetic_owner(xp.clone(), profile.clone());
        let policy = Arc::new(Policy {
            ports: [a, b],
            position: guide.position,
            used: Mutex::new([false; 2]),
        });
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "journal-impaired",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let mut one = [0; 256];
        one[0] = 1;
        let mut two = [0; 256];
        two[0] = 2;
        let token_a = issuer.issue(1, Some(&one), 120).unwrap().bytes().to_vec();
        let token_b = issuer.issue(2, Some(&two), 120).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport, shutdown, world, policy,
        ));
        let mut a = ClientPeer::start(
            addresses[0],
            token_a,
            identity,
            profile.clone(),
            xp.clone(),
            talk.clone(),
            catalog.clone(),
            chunks.clone(),
        );
        let mut b = ClientPeer::start(
            addresses[1],
            token_b,
            identity,
            profile,
            xp,
            talk,
            catalog,
            chunks,
        );
        for client in [&mut a, &mut b] {
            client
                .until(|s| {
                    s.quest_journal.is_some()
                        && s.progression.is_some()
                        && s.dialogue.is_some()
                        && s.chunk_ack.is_some()
                })
                .await;
        }

        let first = "theia.dev.quest-introduction";
        a.talk(0, Talk::Open(guide.key)).await;
        a.action(1, intent(1, first, 0, 1, guide.key, Action::Accept))
            .await;
        let ready = a
            .until(|s| {
                s.quest_journal
                    .as_ref()
                    .unwrap()
                    .journal
                    .progress(&id(first))
                    .is_some_and(|p| p.status() == Status::Ready)
            })
            .await;
        assert_eq!(
            ready
                .quest_journal
                .unwrap()
                .journal
                .progress(&id(first))
                .unwrap()
                .revision(),
            4
        );
        assert_eq!(
            a.state
                .borrow()
                .progression
                .as_ref()
                .unwrap()
                .progress
                .total(),
            0
        );
        assert_eq!(b.state.borrow().quest_journal.as_ref().unwrap().revision, 0);
        let done = a
            .action(2, intent(2, first, 4, 1, guide.key, Action::TurnIn))
            .await;
        assert_eq!(done.quest_journal_reply.unwrap().rejection, None);
        assert_eq!(done.progression.unwrap().progress.total(), 25);
        assert_eq!(
            done.quest_journal
                .unwrap()
                .journal
                .progress(&id(first))
                .unwrap()
                .status(),
            Status::Completed
        );
        assert_eq!(
            b.state
                .borrow()
                .progression
                .as_ref()
                .unwrap()
                .progress
                .total(),
            0
        );
        a.close().await;
        b.close().await;
        stop.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.journal_area_batches_forwarded, 2);
        assert_eq!(stats.journal_kills_forwarded, 0);
        assert_eq!(stats.journal_requests_forwarded, 2);
        for worker in workers {
            worker.await.unwrap();
        }
        for control in controls {
            assert_eq!(control.lock().unwrap().stats.overflow, 0);
        }
        owner_a.await.unwrap();
        owner_b.await.unwrap();
    }
}
