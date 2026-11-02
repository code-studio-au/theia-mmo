//! Actual encrypted runtime with a synthetic admission/in-memory commit adapter.
//! Real OIDC/SQL retirement is qualified separately; no native presentation claim.
use super::*;
use crate::{
    dialogue::{Action as Talk, Profile as TalkProfile},
    renet_runtime::{self as net, AccountAdmissions, AdmissionLease, PlayerAdmission},
    wire::Intent as Movement,
};
use std::{
    collections::BTreeSet,
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    progression,
    quests::lifecycle,
};
use tokio::{sync::watch, task::JoinHandle};

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
    quest: Arc<Profile>,
) -> (Port, JoinHandle<()>) {
    let (input, mut inputs) = mpsc::channel(net::progression::CAPACITY);
    let (publish, state) = watch::channel(net::progression::Observation::default());
    let port = Port {
        profile: xp.clone(),
        quest: None,
        lifecycle: Some(quest.clone()),
        input,
        state,
    };
    let task = tokio::spawn(async move {
        let mut experience = progression::State::new(xp.profile(), 0, 0).unwrap();
        let mut progress = Progress::available(quest.profile());
        let mut seen = BTreeSet::new();
        let mut reply = None;
        publish.send_modify(|s| {
            s.progress = Some(experience.clone());
            s.lifecycle = Some(progress.clone());
        });
        while let Some(input) = inputs.recv().await {
            match input {
                Input::Journal(_) => {
                    panic!("Legacy fixture cannot receive expanded journal inputs")
                }
                Input::Kill(event) => {
                    if !seen.insert(event.id.bytes()) {
                        continue;
                    }
                    experience = xp
                        .profile()
                        .propose_kill(&experience, &event.creature)
                        .unwrap()
                        .state;
                    progress = lifecycle::propose_kill(quest.profile(), &progress, &event.creature)
                        .unwrap()
                        .state;
                }
                Input::QuestAction(request) => {
                    let rejection = match lifecycle::propose_action(
                        quest.profile(),
                        &progress,
                        request.command,
                    ) {
                        Ok(next) => {
                            progress = next.state;
                            None
                        }
                        Err(lifecycle::Error::NotAccepted | lifecycle::Error::AlreadyAccepted) => {
                            Some(Rejection::NotAvailable)
                        }
                        Err(lifecycle::Error::NotReady) => Some(Rejection::NotReady),
                        Err(lifecycle::Error::AlreadyTurnedIn) => Some(Rejection::AlreadyTurnedIn),
                        Err(lifecycle::Error::StaleRevision) => Some(Rejection::StaleRevision),
                        Err(error) => panic!("Unexpected synthetic lifecycle error: {error:?}"),
                    };
                    reply = Some(ActionReply {
                        sequence: request.sequence,
                        quest_revision: progress.revision(),
                        rejection,
                    });
                }
            }
            publish.send_modify(|s| {
                s.progress = Some(experience.clone());
                s.lifecycle = Some(progress.clone());
                s.lifecycle_reply = reply;
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
                    classic_source: None,
                    dodge: None,
                    player_life: None,
                    lifecycle: Some(Client {
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
    async fn action(
        &mut self,
        sequence: u64,
        revision: u64,
        dialogue_revision: u64,
        npc: EntityKey,
        action: Action,
    ) -> State {
        self.quest
            .try_send(Intent {
                expected_revision: revision,
                dialogue_revision,
                npc,
                action,
            })
            .unwrap();
        self.until(|s| {
            s.quest_lifecycle_reply
                .is_some_and(|r| r.sequence == sequence)
                && !s.quest_lifecycle_pending
        })
        .await
    }
    async fn kill(&mut self, expected_count: u32, preaccept: bool) {
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
        self.casts
            .try_send(net::CastChoice {
                ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target,
            })
            .unwrap();
        self.until(|s| {
            s.quest_lifecycle
                .as_ref()
                .is_some_and(|v| v.progress.count() == expected_count)
                && s.progression
                    .as_ref()
                    .is_some_and(|p| p.progress.total() == 25)
                && (!preaccept
                    || s.combat
                        .as_ref()
                        .is_some_and(|c| c.targets.iter().any(|t| t.current == 0)))
        })
        .await;
    }
    async fn close(self) {
        self.stop.send_replace(true);
        self.task.await.unwrap().unwrap();
        self.events.await.unwrap();
    }
}
#[tokio::test]
async fn encrypted_two_player_quests_keep_private_progress_and_refuse_stale_guide_actions() {
    let (mut world, _) =
        crate::simulation::creatures::tests::fixture(theia_core::time::TickRate::Hz30);
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
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    raw["level_thresholds"] = serde_json::json!([0, 25]);
    let xp = Arc::new(
        crate::progression_wire::Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),
    );
    world.enable_progression(xp.clone()).unwrap();
    let talk = Arc::new(
        TalkProfile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-dialogue.json"
        ))
        .unwrap(),
    );
    let guide = world
        .spawn_development_guide(talk.clone(), [8002., 8000.])
        .unwrap();
    let profile = Arc::new(
        Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-quest.json"
        ))
        .unwrap(),
    );
    world.enable_quest_lifecycle(profile.clone()).unwrap();
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
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let mut one = [0; 256];
    one[0] = 1;
    let mut two = [0; 256];
    two[0] = 2;
    let token_a = issuer.issue(1, Some(&one), 120).unwrap().bytes().to_vec();
    let token_b = issuer.issue(2, Some(&two), 120).unwrap().bytes().to_vec();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(net::run_server_with_admissions(
        transport, shutdown, world, policy,
    ));
    let mut a = ClientPeer::start(
        address,
        token_a,
        identity,
        profile.clone(),
        xp.clone(),
        talk.clone(),
        catalog.clone(),
        chunks.clone(),
    );
    let mut b = ClientPeer::start(
        address,
        token_b,
        identity,
        profile.clone(),
        xp,
        talk,
        catalog,
        chunks,
    );
    for client in [&mut a, &mut b] {
        client
            .until(|s| {
                s.quest_lifecycle.is_some() && s.progression.is_some() && s.dialogue.is_some()
            })
            .await;
    }
    a.kill(0, true).await;
    assert_eq!(
        a.state
            .borrow()
            .quest_lifecycle
            .as_ref()
            .unwrap()
            .progress
            .status(),
        lifecycle::Status::Available
    );
    a.talk(0, Talk::Open(guide.key)).await;
    a.talk(1, Talk::Close).await;
    let refused = a.action(1, 0, 1, guide.key, Action::Accept).await;
    assert_eq!(
        refused.quest_lifecycle_reply.unwrap().rejection,
        Some(Rejection::NotEntitled)
    );
    assert_eq!(refused.quest_lifecycle.unwrap().progress.revision(), 0);
    a.talk(2, Talk::Open(guide.key)).await;
    let accepted = a.action(2, 0, 3, guide.key, Action::Accept).await;
    assert_eq!(accepted.quest_lifecycle_reply.unwrap().rejection, None);
    assert_eq!(accepted.quest_lifecycle.unwrap().progress.revision(), 1);
    for count in 1..=3 {
        a.kill(count, false).await;
    }
    let done = a.action(3, 4, 3, guide.key, Action::TurnIn).await;
    assert_eq!(
        done.quest_lifecycle.unwrap().progress.status(),
        lifecycle::Status::TurnedIn
    );
    assert_eq!(
        b.state
            .borrow()
            .quest_lifecycle
            .as_ref()
            .unwrap()
            .progress
            .revision(),
        0
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
    b.talk(0, Talk::Open(guide.key)).await;
    let early = b.action(1, 0, 1, guide.key, Action::TurnIn).await;
    assert_eq!(
        early.quest_lifecycle_reply.unwrap().rejection,
        Some(Rejection::NotAvailable)
    );
    b.action(2, 0, 1, guide.key, Action::Accept).await;
    for count in 1..=3 {
        b.kill(count, false).await;
    }
    let done = b.action(3, 4, 1, guide.key, Action::TurnIn).await;
    assert_eq!(
        done.quest_lifecycle.unwrap().progress.status(),
        lifecycle::Status::TurnedIn
    );
    assert_eq!(
        a.state
            .borrow()
            .quest_lifecycle
            .as_ref()
            .unwrap()
            .progress
            .revision(),
        5
    );
    assert_eq!(
        a.state
            .borrow()
            .transport
            .as_ref()
            .unwrap()
            .lifecycle_requests_sent,
        3
    );
    assert_eq!(
        b.state
            .borrow()
            .transport
            .as_ref()
            .unwrap()
            .lifecycle_requests_sent,
        3
    );
    a.close().await;
    b.close().await;
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    owner_a.abort();
    owner_b.abort();
    let _ = owner_a.await;
    let _ = owner_b.await;
    assert_eq!(stats.experience_opportunities_forwarded, 7);
    assert_eq!(stats.lifecycle_requests_forwarded, 5);
    assert_eq!(stats.lifecycle_requests_ignored, 1);
    assert_eq!(stats.lifecycle_replies_sent, 6);
    assert!(stats.lifecycle_states_sent >= 12);
}
