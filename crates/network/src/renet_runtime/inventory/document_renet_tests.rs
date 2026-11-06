//! Actual encrypted runtime with held synthetic owner publications. Real SQL and
//! identity lifetime are qualified separately; this is not native reader proof.
use super::*;
use crate::{client::State, renet_runtime as net, test_relay as relay};
use net::{AccountAdmissions, AdmissionLease, PlayerAdmission};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Mutex,
    time::Duration,
};
use theia_core::{
    coordinates::{ChunkId, GlobalPosition},
    documents::OpenCommand,
    identity::EntityKey,
    inventory::{Command, Operation, Stack},
    quests::journal::Journal,
    time::TickRate,
};
const ITEMS: &[u8] = include_bytes!("../../../../../content/gameplay/development-items.json");
const NOTES:&[u8]=br#"{"schema":1,"id":"documents","revision":1,"item_catalog":"theia.dev.backpack-items","documents":[{"id":"note","item":"theia.dev.training-stone","title":"Orders","pages":["Read the orders."]}]}"#;
struct Lease {
    items: Port,
    progression: net::progression::Port,
}
impl AdmissionLease for Lease {
    fn live(&self) -> crate::Result<bool> {
        Ok(true)
    }
    fn record(&self, _: EntityKey, _: GlobalPosition, _: u64) -> crate::Result<()> {
        Ok(())
    }
    fn inventory(&self) -> crate::Result<Option<Port>> {
        Ok(Some(self.items.clone()))
    }
    fn progression(&self) -> crate::Result<Option<net::progression::Port>> {
        Ok(Some(self.progression.clone()))
    }
}
struct Policy {
    items: [Port; 2],
    progression: [net::progression::Port; 2],
    position: GlobalPosition,
    used: Mutex<[bool; 2]>,
}
impl AccountAdmissions for Policy {
    fn consume(
        self: Arc<Self>,
        client: u64,
        data: &[u8; 256],
    ) -> crate::Result<Option<PlayerAdmission>> {
        if !(1..=2).contains(&client) || data[0] != client as u8 {
            return Ok(None);
        }
        let i = client as usize - 1;
        let mut used = self.used.lock().unwrap();
        if used[i] {
            return Ok(None);
        }
        used[i] = true;
        Ok(Some(PlayerAdmission {
            position: self.position,
            lease: Box::new(Lease {
                items: self.items[i].clone(),
                progression: self.progression[i].clone(),
            }),
        }))
    }
    fn retained_positions(&self) -> crate::Result<[Option<GlobalPosition>; 8]> {
        let mut positions = [None; 8];
        positions[0] = Some(self.position);
        positions[1] = Some(self.position);
        Ok(positions)
    }
}
struct PeerClient {
    state: watch::Receiver<State>,
    commands: mpsc::Sender<ItemCommand>,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<crate::Result<()>>,
    _keepalive: KeepAlive,
}
struct KeepAlive {
    _movement: watch::Sender<crate::wire::Intent>,
    _ready: watch::Sender<Option<Vec<ChunkId>>>,
    _actions: mpsc::Sender<net::quest_journal::Intent>,
    _talk: mpsc::Sender<net::dialogue::Intent>,
    _prediction: mpsc::Sender<crate::prediction_wire::Commands>,
}
struct Profiles {
    items: Profile,
    journal: Arc<crate::quest_journal::Profile>,
    xp: Arc<crate::progression_wire::Profile>,
    talk: Arc<crate::dialogue::Profile>,
}
impl PeerClient {
    fn start(
        address: SocketAddr,
        ticket: Vec<u8>,
        world: [u8; 32],
        profiles: &Profiles,
        chunks: Vec<ChunkId>,
    ) -> Self {
        let (movement, input) = watch::channel(crate::wire::Intent::default());
        let (ready, readiness) = watch::channel(Some(chunks));
        let (commands, receiver) = mpsc::channel(1);
        let (prediction, prediction_commands) = mpsc::channel(8);
        let (actions, quests) = mpsc::channel(4);
        let (talk, talks) = mpsc::channel(4);
        let (publish, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let items = profiles.items.clone();
        let journal = profiles.journal.clone();
        let xp = profiles.xp.clone();
        let profile = profiles.talk.clone();
        let task = tokio::spawn(async move {
            let result = net::run_client_gameplay(
                address,
                &ticket,
                input,
                publish.clone(),
                shutdown,
                world,
                net::GameplayClient {
                    avatar_life: None,
                    public_attacks: None,
                    classic_source: None,
                    dodge: None,
                    player_life: None,
                    lifecycle: None,
                    journal: Some(net::quest_journal::Client {
                        profile: journal,
                        actions: quests,
                    }),
                    dialogue: Some(net::dialogue::Client {
                        profile,
                        actions: talks,
                    }),
                    ready: readiness,
                    commands: Some(prediction_commands),
                    choices: None,
                    combat: None,
                    quest: None,
                    progression: Some(xp),
                    inventory: Some(Client {
                        profile: items,
                        commands: receiver,
                        pickups: None,
                    }),
                    statuses: None,
                },
            )
            .await;
            if let Err(error) = &result {
                publish.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            state,
            commands,
            stop,
            task,
            _keepalive: KeepAlive {
                _movement: movement,
                _ready: ready,
                _actions: actions,
                _talk: talk,
                _prediction: prediction,
            },
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let state = self.state.borrow().clone();
                assert!(state.error.is_none(), "{:?}", state.error);
                if predicate(&state) {
                    break state;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .expect("Encrypted document condition timed out")
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
fn command(request: u128, revision: u64) -> OpenCommand {
    OpenCommand {
        request_id: std::num::NonZeroU128::new(request).unwrap(),
        expected_revision: revision,
        slot: 0,
        document: 0,
    }
}
fn profiles() -> Profiles {
    let items = Profile::from_json(ITEMS)
        .unwrap()
        .with_documents(NOTES)
        .unwrap();
    let xp = Arc::new(
        crate::progression_wire::Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    );
    let bytes=serde_json::to_vec(&serde_json::json!({"schema":4,"id":"catalog","revision":1,"quests":[{"id":"quest","version":1,"title":"Orders","description":"Read orders","prerequisites":[],"giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"first","stages":[{"id":"first","completion":{"kind":"all"},"next":null,"objectives":[{"id":"read","kind":"read_document","label":"Read","document":"note"}]}]}]})).unwrap();
    let journal = Arc::new(
        crate::quest_journal::Profile::from_json_with_documents(
            &bytes,
            None,
            &xp,
            items.documents().unwrap(),
        )
        .unwrap(),
    );
    let talk = Arc::new(
        crate::dialogue::Profile::from_json(include_bytes!(
            "../../../../../content/quests/development-journal-dialogue.json"
        ))
        .unwrap(),
    );
    Profiles {
        items,
        journal,
        xp,
        talk,
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_impaired_document_approval_shares_order_and_preserves_owner_privacy_at_all_rates()
 {
    use crate::inventory_wire::{Outcome, Reason, Reply};
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let profiles = profiles();
        let mut world = crate::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        world
            .spawn_development_guide(profiles.talk.clone(), [8004., 8000.])
            .unwrap();
        world
            .enable_document_quest_journal(
                profiles.journal.clone(),
                profiles.xp.clone(),
                &profiles.items,
            )
            .unwrap();
        world.enable_prediction().unwrap();
        let identity = world.identity();
        let position = world.development_player_spawn(1).unwrap();
        let chunks = world
            .procedural()
            .unwrap()
            .patch(position)
            .unwrap()
            .ids
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(
            profiles.items.catalog(),
            theia_core::content::ContentId::new("theia.dev.training-stone").unwrap(),
            1,
        );
        let bag = Bag::from_slots(profiles.items.catalog(), 0, slots).unwrap();
        let mut publications = Vec::new();
        let mut receivers = Vec::new();
        let mut ports = Vec::new();
        let mut progress_ports = Vec::new();
        let mut keepalive = Vec::new();
        let mut progress_inputs = Vec::new();
        for _ in 0..2 {
            let (requests, receiver) = mpsc::channel(1);
            let (publish, state) = watch::channel(Observation {
                bag: Some(bag.clone()),
                ..Default::default()
            });
            publications.push(publish);
            receivers.push(receiver);
            ports.push(Port {
                documents: Some(Arc::new(profiles.items.documents().unwrap().clone())),
                vendor: None,
                rewards: None,
                opportunities: None,
                catalog: Arc::new(profiles.items.catalog().clone()),
                hash: profiles.items.hash(),
                requests,
                state,
            });
            let (input, receiver) = mpsc::channel(net::progression::CAPACITY);
            let (publish, state) = watch::channel(net::progression::Observation {
                progress: Some(
                    theia_core::progression::State::new(profiles.xp.profile(), 0, 0).unwrap(),
                ),
                journal: Some(net::progression::JournalObservation {
                    catalog: profiles.journal.hash(),
                    revision: 0,
                    journal: Journal::default(),
                    reply: None,
                }),
                ..Default::default()
            });
            keepalive.push(publish);
            progress_inputs.push(receiver);
            progress_ports.push(net::progression::Port {
                journal: Some(profiles.journal.hash()),
                lifecycle: None,
                quest: None,
                profile: profiles.xp.clone(),
                input,
                state,
            });
        }
        let policy = Arc::new(Policy {
            items: ports.try_into().unwrap_or_else(|_| panic!("ports")),
            progression: progress_ports
                .try_into()
                .unwrap_or_else(|_| panic!("progress")),
            position,
            used: Mutex::new([false; 2]),
        });
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "documents-impaired",
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
        let tokens: Vec<_> = (1..=2)
            .map(|id| {
                let mut data = [0; 256];
                data[0] = id as u8;
                issuer.issue(id, Some(&data), 120).unwrap().bytes().to_vec()
            })
            .collect();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_with_admissions(
            transport, shutdown, world, policy,
        ));
        let mut a = PeerClient::start(
            addresses[0],
            tokens[0].clone(),
            identity,
            &profiles,
            chunks.clone(),
        );
        let mut b = PeerClient::start(addresses[1], tokens[1].clone(), identity, &profiles, chunks);
        for client in [&mut a, &mut b] {
            client
                .until(|s| {
                    s.inventory.is_some()
                        && s.quest_journal.is_some()
                        && s.progression.is_some()
                        && s.chunk_ack.is_some()
                })
                .await;
        }
        a.commands.try_send(command(100, 0).into()).unwrap();
        let first = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(first.command, command(100, 0).into());
        assert_eq!(first.sequence, 1);
        a.until(|s| s.inventory_pending).await;
        assert!(a.state.borrow().approved_document.is_none());
        assert!(b.state.borrow().approved_document.is_none());
        a.commands
            .try_send(
                Command {
                    expected_revision: 0,
                    operation: Operation::Move { from: 0, to: 1 },
                }
                .into(),
            )
            .unwrap();
        assert!(receivers[0].try_recv().is_err());
        publications[0].send_modify(|s| {
            s.reply = Some(Reply {
                sequence: 1,
                outcome: Outcome::Applied,
            })
        });
        a.until(|s| {
            s.approved_document
                .is_some_and(|d| d.command == command(100, 0))
                && s.inventory.as_ref().unwrap().bag.revision() == 0
        })
        .await;
        let second = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second.sequence, 2);
        let removed = second
            .command
            .propose(&bag, profiles.items.catalog())
            .unwrap();
        publications[0].send_modify(|s| {
            s.bag = Some(removed.clone());
            s.reply = Some(Reply {
                sequence: 2,
                outcome: Outcome::Applied,
            });
        });
        a.until(|s| {
            s.inventory
                .as_ref()
                .is_some_and(|v| v.reply.is_some_and(|r| r.sequence == 2))
                && !s.inventory_pending
        })
        .await;
        assert_eq!(b.state.borrow().inventory.as_ref().unwrap().bag, bag);
        assert!(b.state.borrow().inventory.as_ref().unwrap().reply.is_none());
        assert!(b.state.borrow().approved_document.is_none());
        a.commands.try_send(command(101, 1).into()).unwrap();
        let refused = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(refused.command, command(101, 1).into());
        publications[0].send_modify(|s| {
            s.reply = Some(Reply {
                sequence: 3,
                outcome: Outcome::Rejected(Reason::EmptySource),
            })
        });
        a.until(|s| {
            s.inventory
                .as_ref()
                .is_some_and(|v| v.reply.is_some_and(|r| r.sequence == 3))
                && !s.inventory_pending
        })
        .await;
        assert!(a.state.borrow().approved_document.is_none());
        a.commands.try_send(command(100, 0).into()).unwrap();
        let retry = tokio::time::timeout(Duration::from_secs(4), receivers[0].recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retry.command, first.command);
        publications[0].send_modify(|s| {
            s.reply = Some(Reply {
                sequence: 4,
                outcome: Outcome::Replayed,
            })
        });
        a.until(|s| {
            s.approved_document
                .is_some_and(|d| d.command == command(100, 0))
                && !s.inventory_pending
        })
        .await;
        assert_eq!(a.state.borrow().inventory.as_ref().unwrap().bag, removed);
        assert!(receivers[0].try_recv().is_err());
        assert!(receivers[1].try_recv().is_err());
        for input in &mut progress_inputs {
            assert!(input.try_recv().is_err());
        }
        a.stop.send_replace(true);
        b.stop.send_replace(true);
        a.task.await.unwrap().unwrap();
        b.task.await.unwrap().unwrap();
        stop.send_replace(true);
        server.await.unwrap().unwrap();
        for worker in workers {
            worker.await.unwrap();
        }
        for control in controls {
            let c = control.lock().unwrap();
            assert!(c.stats.queue_high_water <= relay::QUEUE_LIMIT && c.stats.overflow == 0);
            assert!(c.stats.configured_drops > 0 && c.stats.duplicates > 0);
        }
    }
}

#[path = "item_reward_renet_tests.rs"]
mod item_rewards;

#[test]
fn document_only_journal_can_bind_search_metadata_without_a_reach_objective() {
    let p = profiles();
    let q = serde_json::to_vec(&serde_json::json!({"schema":4,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Orders","description":"Read orders","prerequisites":[],
        "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"first","stages":[{"id":"first","completion":{"kind":"all"},"next":null,
        "objectives":[{"id":"read","kind":"read_document","label":"Read","document":"note"}]}]}]})).unwrap();
    let mut a = serde_json::json!({"schema":1,"id":"map.areas","revision":1,"areas":[{
        "id":"map.search","name":"Orders search","min":[7990.,-64.,7990.],"max":[8010.,64.,8010.],"map_objectives":["read"]
    }]});
    let bytes = serde_json::to_vec(&a).unwrap();
    let bound = crate::quest_journal::Profile::from_json_with_documents(
        &q,
        Some(&bytes),
        &p.xp,
        p.items.documents().unwrap(),
    )
    .unwrap();
    assert_ne!(bound.hash(), p.journal.hash());
    assert!(bound.area_catalog().is_some());
    a["areas"][0]["map_objectives"] = serde_json::json!(["unknown"]);
    assert!(
        crate::quest_journal::Profile::from_json_with_documents(
            &q,
            Some(&serde_json::to_vec(&a).unwrap()),
            &p.xp,
            p.items.documents().unwrap()
        )
        .is_err()
    );
    a["areas"][0]["map_objectives"] = serde_json::json!([]);
    assert!(
        crate::quest_journal::Profile::from_json_with_documents(
            &q,
            Some(&serde_json::to_vec(&a).unwrap()),
            &p.xp,
            p.items.documents().unwrap()
        )
        .is_err()
    );
}
