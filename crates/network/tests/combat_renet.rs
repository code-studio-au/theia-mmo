//! Actual encrypted production runtime; no mocked health or connections.
#[path = "support/relay.rs"]
#[allow(dead_code)]
mod relay;
#[path = "support/saved_fixture.rs"]
#[allow(dead_code)]
mod saved_fixture;
use relay::{Profile, Relay};
use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use theia_core::{abilities::Catalog, coordinates::ChunkId, identity::EntityKey};
use theia_network::{
    client::State,
    combat_wire::Batch,
    renet_runtime as net,
    simulation::{
        World,
        combat::{Event, Outcome, Reason},
    },
    targeting::Choice,
    wire::Intent,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-abilities.json");
struct Tickets(PathBuf);
impl Drop for Tickets {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
struct Peer {
    state: watch::Receiver<State>,
    input: watch::Sender<Intent>,
    choice: watch::Sender<Option<Choice>>,
    casts: mpsc::Sender<net::CastChoice>,
    events: mpsc::Receiver<Batch>,
    seen: Vec<Event>,
    stop: watch::Sender<bool>,
    task: JoinHandle<theia_network::Result<()>>,
    _ready: watch::Sender<Option<Vec<ChunkId>>>,
    catalog: Arc<Catalog>,
}
impl Peer {
    fn start(
        address: SocketAddr,
        ticket: Vec<u8>,
        identity: [u8; 32],
        chunks: Vec<ChunkId>,
        catalog: Arc<Catalog>,
        event_capacity: usize,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(Some(chunks));
        let (choice, choices) = watch::channel(None);
        let (casts, requests) = mpsc::channel(8);
        let (events, event_rx) = mpsc::channel(event_capacity);
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let data = catalog.clone();
        let task = tokio::spawn(async move {
            let result = net::run_client_combat(
                address,
                &ticket,
                inputs,
                states.clone(),
                shutdown,
                identity,
                net::CombatClient {
                    targeting: net::TargetingClient {
                        ready: readiness,
                        commands: None,
                        choices,
                    },
                    catalog: data,
                    casts: requests,
                    events,
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
            input,
            choice,
            casts,
            events: event_rx,
            seen: vec![],
            stop,
            task,
            _ready: ready,
            catalog,
        }
    }
    fn drain(&mut self) {
        while let Ok(batch) = self.events.try_recv() {
            self.seen.extend(batch.events);
            assert!(self.seen.len() <= 256);
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State, &[Event]) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                self.drain();
                let state = self.state.borrow().clone();
                if predicate(&state, &self.seen) {
                    return state;
                }
                assert!(state.error.is_none(), "{:?}", state.error);
                self.state.changed().await.unwrap();
            }
        })
        .await
        .expect("Combat client observation timed out")
    }
    async fn cast(&self, target: EntityKey) {
        self.casts
            .send(net::CastChoice {
                ability: self.catalog.abilities()[0].id().clone(),
                target: target.entity,
            })
            .await
            .unwrap();
    }
    async fn finish(self) -> theia_network::Result<()> {
        self.stop.send_replace(true);
        tokio::time::timeout(Duration::from_secs(3), self.task)
            .await
            .unwrap()
            .unwrap()
    }
}
fn tickets() -> Tickets {
    Tickets(std::env::temp_dir().join(format!(
        "theia-combat-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )))
}
fn world(bytes: &[u8], health: u32, respawn_ms: u32) -> (World, EntityKey, Vec<ChunkId>) {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    let target = world.spawn_development_target([8068., 8000.]).unwrap();
    world.enable_combat(bytes, health, respawn_ms).unwrap();
    let chunks = theia_network::procedural::neighbours(ChunkId { x: 62, z: 62 }, 1)
        .into_iter()
        .collect();
    (world, target, chunks)
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn saved_impaired_clients_share_mobile_damage_death_respawn_and_fresh_session_state() {
    let saved = saved_fixture::Fixture::new();
    let source = theia_network::procedural::Source::Saved(Arc::new(saved.open()));
    let base = theia_network::targeting::profile_identity(source.identity());
    let mut world = World::with_terrain_source([8060., 8000.], 1., source).unwrap();
    let target = world.spawn_development_target([8068., 8000.]).unwrap();
    world.enable_combat(DATA, 20, 500).unwrap();
    let identity = world.identity();
    let chunks: Vec<_> = theia_network::procedural::neighbours(ChunkId { x: 62, z: 62 }, 1)
        .into_iter()
        .collect();
    let catalog = Arc::new(Catalog::from_json(DATA).unwrap());
    let profile = Profile {
        name: "combat-wifi",
        delay_ms: 40,
        jitter_ms: 60,
        loss_every: 10,
        duplicate_every: 17,
    };
    let relays: Vec<_> = (0..5).map(|_| Relay::bind(profile)).collect();
    let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
    let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
    let tickets = tickets();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let transport = net::issue_local_advertised(socket, &tickets.0, addresses.clone()).unwrap();
    let token = |id| fs::read(tickets.0.join(format!("client-{id}.token"))).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let workers: Vec<_> = relays
        .into_iter()
        .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
        .collect();
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(
        addresses[0],
        token(1),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    let mut b = Peer::start(
        addresses[1],
        token(2),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    let initial = a
        .until(|s, _| {
            s.combat.is_some()
                && s.chunk_ack.is_some()
                && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        })
        .await;
    b.until(|s, _| s.combat.is_some() && s.chunk_ack.is_some())
        .await;
    a.choice.send_replace(Some(Choice {
        target: Some(target.entity),
    }));
    b.choice.send_replace(Some(Choice {
        target: Some(target.entity),
    }));
    a.until(|s, _| {
        s.targeting
            .as_ref()
            .is_some_and(|t| t.selected == Some(target.entity))
    })
    .await;
    b.until(|s, _| {
        s.targeting
            .as_ref()
            .is_some_and(|t| t.selected == Some(target.entity))
    })
    .await;
    let a_id = initial.id.unwrap();
    let initial_x = initial
        .snapshot
        .as_ref()
        .unwrap()
        .players
        .iter()
        .find(|p| p.id == a_id)
        .map(|p| {
            initial
                .snapshot
                .as_ref()
                .unwrap()
                .global_position(p)
                .unwrap()
                .metres()[0]
        })
        .unwrap();
    a.input.send_replace(Intent {
        axes: [1000, 0],
        ..Intent::default()
    });
    a.cast(target).await;
    b.cast(target).await;
    a.until(|s, _| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply
                .is_some_and(|r| matches!(r.outcome, Outcome::Started { .. }))
        })
    })
    .await;
    a.until(|_, events| events.iter().any(|e| matches!(e, Event::Died { .. })))
        .await;
    b.until(|_, events| events.iter().any(|e| matches!(e, Event::Died { .. })))
        .await;
    a.input.send_replace(Intent::default());
    let after = a
        .until(|s, _| {
            s.combat.as_ref().is_some_and(|c| {
                c.targets
                    .first()
                    .is_some_and(|h| h.entity != target.entity && h.current == 20)
            })
        })
        .await;
    let snapshot = after.snapshot.as_ref().unwrap();
    let x = snapshot
        .global_position(snapshot.players.iter().find(|p| p.id == a_id).unwrap())
        .unwrap()
        .metres()[0];
    assert!(
        x > initial_x + 3.,
        "Actual authority must move while casting"
    );
    let next = EntityKey {
        epoch: target.epoch,
        entity: after.combat.as_ref().unwrap().targets[0].entity,
    };
    b.until(|s, _| {
        s.combat.as_ref().is_some_and(|c| {
            c.targets
                .first()
                .is_some_and(|h| h.entity == next.entity && h.current == 20)
        })
    })
    .await;
    for peer in [&mut a, &mut b] {
        peer.until(|_,events|events.iter().any(|e|matches!(e,Event::Respawned {previous,target:new,..} if *previous==target && *new==next))).await;
        assert_eq!(
            peer.seen
                .iter()
                .filter(|e| matches!(e,Event::Damage {target:t,..} if *t==target))
                .count(),
            1
        );
        assert!(
            peer.state
                .borrow()
                .targeting
                .as_ref()
                .unwrap()
                .selected
                .is_none()
        );
    }
    a.cast(target).await;
    a.until(|s, _| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply.is_some_and(|r| {
                r.sequence == 2 && r.outcome == Outcome::Rejected(Reason::UnknownTarget)
            })
        })
    })
    .await;
    let started = after.combat.as_ref().unwrap().reply.unwrap();
    let end = match started.outcome {
        Outcome::Started { completes } => completes + 30,
        _ => panic!("Expected cast"),
    };
    a.until(|s, _| s.snapshot.as_ref().is_some_and(|s| s.tick >= end))
        .await;
    a.cast(next).await;
    a.until(|s, _| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply
                .is_some_and(|r| r.sequence == 3 && matches!(r.outcome, Outcome::Started { .. }))
        })
    })
    .await;
    a.finish().await.unwrap();
    b.until(|s, _| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(
        b.state.borrow().combat.as_ref().unwrap().targets[0].current,
        20,
        "Disconnected caster must not hit replacement"
    );
    let mut fresh = Peer::start(
        addresses[2],
        token(3),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    let state = fresh
        .until(|s, _| s.combat.is_some() && s.chunk_ack.is_some())
        .await;
    let state = state.combat.unwrap();
    assert!(state.reply.is_none() && state.cast.is_none() && state.cooldowns.is_empty());
    // The previous combat profile must fail at admission, before reading CE02.
    let mut config = b"theia-combat-profile-v1\0".to_vec();
    config.extend(theia_network::terrain::sha256(DATA));
    config.extend(20u32.to_le_bytes());
    config.extend(500u32.to_le_bytes());
    let mut prior = b"theia-combat-world-v1\0".to_vec();
    prior.extend(base);
    prior.extend(theia_network::terrain::sha256(&config));
    let prior = theia_network::terrain::sha256(&prior);
    let mut old = Peer::start(
        addresses[4],
        token(5),
        prior,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    old.until(|s, _| s.error.is_some()).await;
    assert!(old.state.borrow().id.is_none());
    assert!(old.finish().await.is_err());
    // Old targeting profile cannot silently join this application profile.
    let (inputs, input) = watch::channel(Intent::default());
    let (states, state) = watch::channel(State::default());
    let (_mismatch_stop, shutdown) = watch::channel(false);
    let (_, ready) = watch::channel(None);
    let (_, choices) = watch::channel(None);
    let mismatch = net::run_client_targeted(
        addresses[3],
        &token(4),
        input,
        states,
        shutdown,
        base,
        net::TargetingClient {
            ready,
            commands: None,
            choices,
        },
    )
    .await;
    assert!(mismatch.is_err());
    assert!(state.borrow().id.is_none());
    drop(inputs);
    fresh.finish().await.unwrap();
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    for worker in workers {
        worker.await.unwrap();
    }
    drop(guard);
    assert_eq!(stats.cast_requests_accepted, 3);
    assert_eq!(stats.cast_requests_rejected, 1);
    assert_eq!(stats.combat_backpressure_disconnects, 0);
    assert!(stats.max_event_queue_bytes <= 8192);
    for c in &controls {
        assert_eq!(c.lock().unwrap().stats.overflow, 0);
    }
    for c in &controls[..2] {
        let c = c.lock().unwrap();
        assert!(c.stats.configured_drops > 0 && c.stats.reordered > 0);
    }
    let report = serde_json::json!({"passed":true,"profile":profile,"server":stats,"old_target":target.entity.value(),"respawn_target":next.entity.value(),"scope":"Actual encrypted saved-terrain mobile casting and shared damage/death/respawn under impairment; fresh session and targeting-profile mismatch; no GPU or capacity claim"});
    if let Some(root) = std::env::var_os("THEIA_NETWORK_REPORT_DIR") {
        let root = PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("combat-wifi.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stalled_presentation_disconnects_without_silently_dropping_damage_for_a_live_client() {
    let (world, target, chunks) = world(DATA, 100, 500);
    let identity = world.identity();
    let catalog = Arc::new(Catalog::from_json(DATA).unwrap());
    let tickets = tickets();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &tickets.0).unwrap();
    let token = |id| fs::read(tickets.0.join(format!("client-{id}.token"))).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let a = Peer::start(
        address,
        token(1),
        identity,
        chunks.clone(),
        catalog.clone(),
        1,
    );
    let mut b = Peer::start(address, token(2), identity, chunks, catalog, 16);
    b.until(|s, _| {
        s.combat.is_some()
            && s.chunk_ack.is_some()
            && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
    })
    .await;
    // Do not drain a.events. The single Started batch fills the bounded queue;
    // the subsequent authoritative Damage must close that client.
    a.cast(target).await;
    b.until(|_, events| {
        events
            .iter()
            .any(|e| matches!(e, Event::Damage { remaining: 80, .. }))
    })
    .await;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if a.state.borrow().error.is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        a.state
            .borrow()
            .error
            .as_ref()
            .unwrap()
            .contains("presentation queue full")
    );
    assert!(a.finish().await.is_err());
    b.until(|s, _| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    assert_eq!(
        b.state.borrow().combat.as_ref().unwrap().targets[0].current,
        80
    );
    assert_eq!(
        b.seen
            .iter()
            .filter(|e| matches!(e, Event::Damage { remaining: 80, .. }))
            .count(),
        1
    );
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    drop(guard);
    assert_eq!(stats.cast_requests_accepted, 1);
    assert_eq!(stats.combat_backpressure_disconnects, 0);
    assert!(stats.max_event_queue_bytes <= 8192);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn blocked_downstream_reliable_budget_disconnects_only_that_peer_without_silent_event_loss() {
    let mut data: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    data["abilities"][0]["cast_ms"] = 0.into();
    data["abilities"][0]["cooldown_ms"] = 60_000.into();
    data["abilities"][0]["uses_global_cooldown"] = false.into();
    // Accumulate real, bounded cooldown entries quickly enough to exercise the
    // send budget before the unchanged five-second netcode timeout wins.
    data["abilities"] = serde_json::Value::Array(
        (0..64)
            .map(|i| {
                let mut ability = data["abilities"][0].clone();
                ability["id"] = format!("theia.dev.probe-{i:02}").into();
                ability
            })
            .collect(),
    );
    let data = serde_json::to_vec(&data).unwrap();
    let catalog = Arc::new(Catalog::from_json(&data).unwrap());
    let (world, target, chunks) = world(&data, 100_000, 500);
    let identity = world.identity();
    let tickets = tickets();
    let relays: Vec<_> = (0..2).map(|_| Relay::bind(Profile::BASELINE)).collect();
    let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
    let control = relays[0].control.clone();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let transport = net::issue_local_advertised(socket, &tickets.0, addresses.clone()).unwrap();
    let token = |id| fs::read(tickets.0.join(format!("client-{id}.token"))).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let workers: Vec<_> = relays
        .into_iter()
        .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
        .collect();
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(
        addresses[0],
        token(1),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    let mut b = Peer::start(
        addresses[1],
        token(2),
        identity,
        chunks,
        catalog.clone(),
        16,
    );
    a.until(|s, _| s.combat.is_some() && s.chunk_ack.is_some())
        .await;
    b.until(|s, _| {
        s.combat.is_some()
            && s.chunk_ack.is_some()
            && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
    })
    .await;
    control.lock().unwrap().block_server_to_client = true;
    let casts = a.casts.clone();
    let producer = tokio::spawn(async move {
        for index in 0..64 {
            if casts
                .send(net::CastChoice {
                    ability: catalog.abilities()[index].id().clone(),
                    target: target.entity,
                })
                .await
                .is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(160)).await;
        }
    });
    let state = b
        .until(|s, events| {
            s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1)
                && events.iter().any(|e| matches!(e, Event::Damage { .. }))
        })
        .await;
    assert!(state.combat.unwrap().targets[0].current < 100_000);
    producer.abort();
    let _ = producer.await;
    let _ = a.finish().await;
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    for worker in workers {
        worker.await.unwrap();
    }
    drop(guard);
    assert_eq!(stats.combat_backpressure_disconnects, 1);
    assert!(stats.max_event_queue_bytes <= 8192);
    assert!(stats.cast_requests_accepted > 10);
    let control = control.lock().unwrap();
    assert_eq!(control.stats.overflow, 0);
    assert!(control.stats.outage_drops > 0);
}

struct Raw {
    client: renet::RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
    catalog: Arc<Catalog>,
    identity: [u8; 32],
    last: std::time::Instant,
    welcome: Option<theia_network::wire::Welcome>,
    state: Option<theia_network::combat_wire::State>,
    ready: bool,
    disconnected: bool,
}
impl Raw {
    fn new(address: SocketAddr, ticket: &[u8], identity: [u8; 32], catalog: Arc<Catalog>) -> Self {
        let token = net::read_ticket(ticket, address).unwrap();
        Self {
            client: renet::RenetClient::new(theia_network::renet_transport::connection_config()),
            transport: theia_network::renet_transport::secure_client(
                UdpSocket::bind("127.0.0.1:0").unwrap(),
                net::now().unwrap(),
                token,
            )
            .unwrap(),
            catalog,
            identity,
            last: std::time::Instant::now(),
            welcome: None,
            state: None,
            ready: false,
            disconnected: false,
        }
    }
    fn send(&mut self, bytes: Vec<u8>) {
        self.client
            .send_message(theia_network::renet_transport::EVENT_CHANNEL, bytes);
    }
    fn cast(&mut self, epoch: u64, sequence: u64, target: theia_core::identity::EntityId) {
        self.send(
            theia_network::combat_wire::Request {
                epoch,
                sequence,
                ability: 0,
                target,
            }
            .encode()
            .unwrap(),
        );
    }
    async fn until(&mut self, predicate: impl Fn(&Self) -> bool) {
        tokio::time::timeout(Duration::from_secs(5), async {
            while !predicate(self) {
                let now = std::time::Instant::now();
                let delta = now.duration_since(self.last);
                self.last = now;
                self.client.update(delta);
                if self.transport.update(delta, &mut self.client).is_err() {
                    self.disconnected = true;
                    assert!(predicate(self), "Unexpected raw combat disconnect");
                    break;
                }
                while let Some(bytes) = self
                    .client
                    .receive_message(theia_network::renet_transport::EVENT_CHANNEL)
                {
                    if self.welcome.is_none() {
                        self.welcome = Some(
                            theia_network::wire::Welcome::decode(&bytes, self.identity).unwrap(),
                        );
                    } else if bytes.starts_with(theia_network::combat_wire::STATE_TAG) {
                        self.state = Some(
                            theia_network::combat_wire::State::decode(&bytes, &self.catalog)
                                .unwrap(),
                        );
                    } else if bytes.starts_with(theia_network::combat_wire::EVENTS_TAG) {
                        theia_network::combat_wire::Batch::decode(&bytes, &self.catalog).unwrap();
                    } else if bytes.starts_with(theia_network::targeting_wire::STATE_TAG) {
                        theia_network::targeting_wire::State::decode(&bytes).unwrap();
                    } else if matches!(
                        theia_network::wire::ChunkEvent::decode(&bytes).unwrap(),
                        theia_network::wire::ChunkEvent::Ack(_)
                    ) {
                        self.ready = true;
                    }
                }
                while self
                    .client
                    .receive_message(theia_network::renet_transport::MOVEMENT_CHANNEL)
                    .is_some()
                {}
                if let Err(error) = self.transport.send_packets(&mut self.client) {
                    self.disconnected = true;
                    assert!(predicate(self), "Unexpected raw send disconnect: {error}");
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("Raw combat observation timed out");
    }
    async fn admit(&mut self, chunks: &[ChunkId]) {
        self.until(|s| s.welcome.is_some()).await;
        let ready = theia_network::wire::ChunkReady {
            version: theia_network::wire::VERSION,
            epoch: self.welcome.as_ref().unwrap().epoch,
            sequence: 1,
            chunks: chunks.iter().map(|id| [id.x, id.z]).collect(),
        };
        self.send(theia_network::wire::encode(&ready).unwrap());
        self.until(|s| s.ready && s.state.is_some()).await;
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_encrypted_foreign_stale_forged_and_flooded_casts_fail_closed() {
    let (world, target, chunks) = world(DATA, 100, 500);
    let identity = world.identity();
    let catalog = Arc::new(Catalog::from_json(DATA).unwrap());
    let tickets = tickets();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &tickets.0).unwrap();
    let token = |id| fs::read(tickets.0.join(format!("client-{id}.token"))).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut raw = Raw::new(address, &token(1), identity, catalog.clone());
    raw.admit(&chunks).await;
    let mut observer = Peer::start(
        address,
        token(2),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
    );
    observer
        .until(|s, _| s.combat.is_some() && s.chunk_ack.is_some())
        .await;
    let epoch = target.epoch.value();
    raw.cast(if epoch == 1 { 2 } else { 1 }, 1, target.entity);
    raw.cast(epoch, u64::MAX, target.entity);
    raw.cast(epoch, 1, target.entity);
    raw.until(|s| {
        s.state.as_ref().is_some_and(|s| {
            s.reply
                .is_some_and(|r| r.sequence == 1 && matches!(r.outcome, Outcome::Started { .. }))
        })
    })
    .await;
    raw.cast(epoch, 1, target.entity);
    raw.cast(
        epoch,
        2,
        theia_core::identity::EntityId::new(raw.welcome.as_ref().unwrap().id).unwrap(),
    );
    raw.cast(epoch, 3, target.entity);
    raw.until(|s| {
        s.state.as_ref().is_some_and(|s| {
            s.reply.is_some_and(|r| {
                r.sequence == 3 && r.outcome == Outcome::Rejected(Reason::Unavailable)
            })
        })
    })
    .await;
    // The schema has no caster field. Appending a forged actor ID is malformed,
    // closes the connection and immediately retires its unfinished cast.
    let mut forged = theia_network::combat_wire::Request {
        epoch,
        sequence: 4,
        ability: 0,
        target: target.entity,
    }
    .encode()
    .unwrap();
    forged.extend(9999u64.to_le_bytes());
    raw.send(forged);
    raw.until(|s| s.disconnected).await;
    let mut flooded = Raw::new(address, &token(3), identity, catalog);
    flooded.admit(&chunks).await;
    for sequence in 1..=11 {
        flooded.cast(epoch, sequence, target.entity);
    }
    flooded.until(|s| s.disconnected).await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let state = observer
        .until(|s, _| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    assert_eq!(state.combat.unwrap().targets[0].current, 100);
    assert!(
        !observer
            .seen
            .iter()
            .any(|e| matches!(e, Event::Damage { .. }))
    );
    observer.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    drop(guard);
    assert_eq!(stats.cast_requests_accepted, 2);
    assert_eq!(stats.cast_requests_ignored, 3);
    assert!(stats.cast_requests_rejected >= 13);
    assert!(stats.max_control_batch <= 8 && stats.max_event_queue_bytes <= 8192);
}
