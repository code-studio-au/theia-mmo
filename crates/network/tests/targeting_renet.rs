//! Actual production encrypted runtime through deterministic impaired loopback UDP.
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
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use theia_network::{
    client::State, renet_runtime as net, simulation::World, targeting::Choice, wire::Intent,
};
use tokio::{sync::watch, task::JoinHandle};
static NEXT: AtomicU64 = AtomicU64::new(0);
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
    choice: watch::Sender<Option<Choice>>,
    commands: Option<tokio::sync::mpsc::Sender<theia_network::prediction_wire::Commands>>,
    stop: watch::Sender<bool>,
    _input: watch::Sender<Intent>,
    _ready: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    task: JoinHandle<theia_network::Result<()>>,
}
impl Peer {
    fn start(address: SocketAddr, ticket: Vec<u8>, world: [u8; 32], targeted: bool) -> Self {
        Self::mode(address, ticket, world, targeted, false)
    }
    fn mode(
        address: SocketAddr,
        ticket: Vec<u8>,
        world: [u8; 32],
        targeted: bool,
        predicted: bool,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(None);
        let (choice, choices) = watch::channel(None);
        let (commands, command_queue) = tokio::sync::mpsc::channel(8);
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            let result = if targeted {
                net::run_client_targeted(
                    address,
                    &ticket,
                    inputs,
                    states.clone(),
                    shutdown,
                    world,
                    net::TargetingClient {
                        ready: readiness,
                        commands: predicted.then_some(command_queue),
                        choices,
                    },
                )
                .await
            } else {
                net::run_client_with_chunks(
                    address,
                    &ticket,
                    inputs,
                    states.clone(),
                    shutdown,
                    world,
                    readiness,
                )
                .await
            };
            if let Err(error) = &result {
                states.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            state,
            choice,
            commands: predicted.then_some(commands),
            stop,
            _input: input,
            _ready: ready,
            task,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = self.state.borrow().clone();
                if predicate(&state) {
                    return state;
                }
                assert!(state.error.is_none(), "{:?}", state.error);
                self.state.changed().await.unwrap();
            }
        })
        .await
        .expect("Targeting acceptance timeout")
    }
    async fn finish(self) -> theia_network::Result<()> {
        self.stop.send_replace(true);
        self.task.await.unwrap()
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_impaired_two_client_targeting_clear_reject_profile_and_fresh_session() {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    let first = world.spawn_development_target([8068., 8000.]).unwrap();
    let second = world.spawn_development_target([8072., 8000.]).unwrap();
    let identity = world.identity();
    let profile = Profile {
        name: "targeting-wifi",
        delay_ms: 40,
        jitter_ms: 60,
        loss_every: 10,
        duplicate_every: 17,
    };
    let relays: Vec<_> = (0..4).map(|_| Relay::bind(profile)).collect();
    let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
    let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
    let tickets = Tickets(std::env::temp_dir().join(format!(
        "theia-targeting-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
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
    let (control, requests) = net::world_control();
    let server = tokio::spawn(net::run_server_with_world_control(
        transport, shutdown, world, requests,
    ));
    let mut a = Peer::start(addresses[0], token(1), identity, true);
    let mut b = Peer::start(addresses[1], token(2), identity, true);
    let initial = a
        .until(|s| {
            s.targeting.as_ref().is_some_and(|t| t.targets.len() == 2)
                && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        })
        .await;
    b.until(|s| s.targeting.is_some()).await;
    let a_id = initial.id.unwrap();
    let b_id = b.state.borrow().id.unwrap();
    assert_ne!(a_id, b_id);
    assert_eq!(initial.targeting.unwrap().selected, None);
    a.choice.send_replace(Some(Choice {
        target: Some(first.entity),
    }));
    a.until(|s| {
        s.targeting.as_ref().is_some_and(|t| {
            t.request_sequence == 1 && t.accepted && t.selected == Some(first.entity)
        })
    })
    .await;
    assert_eq!(b.state.borrow().targeting.as_ref().unwrap().selected, None);
    a.choice.send_replace(Some(Choice {
        target: Some(first.entity),
    }));
    a.until(|s| {
        s.targeting.as_ref().is_some_and(|t| {
            t.request_sequence == 2 && t.accepted && t.selected == Some(first.entity)
        })
    })
    .await;
    b.choice.send_replace(Some(Choice {
        target: Some(second.entity),
    }));
    b.until(|s| {
        s.targeting.as_ref().is_some_and(|t| {
            t.request_sequence == 1 && t.accepted && t.selected == Some(second.entity)
        })
    })
    .await;
    assert_eq!(
        a.state.borrow().targeting.as_ref().unwrap().selected,
        Some(first.entity)
    );
    a.choice.send_replace(Some(Choice { target: None }));
    a.until(|s| {
        s.targeting
            .as_ref()
            .is_some_and(|t| t.request_sequence == 3 && t.accepted && t.selected.is_none())
    })
    .await;
    // Existing player IDs must not masquerade as development NPC targets.
    a.choice.send_replace(Some(Choice {
        target: Some(theia_core::identity::EntityId::new(b_id).unwrap()),
    }));
    a.until(|s| {
        s.targeting
            .as_ref()
            .is_some_and(|t| t.request_sequence == 4 && !t.accepted && t.selected.is_none())
    })
    .await;
    a.choice.send_replace(Some(Choice {
        target: Some(first.entity),
    }));
    a.until(|s| {
        s.targeting.as_ref().is_some_and(|t| {
            t.request_sequence == 5 && t.accepted && t.selected == Some(first.entity)
        })
    })
    .await;
    let received = a
        .state
        .borrow()
        .transport
        .as_ref()
        .unwrap()
        .target_states_received;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        a.state
            .borrow()
            .transport
            .as_ref()
            .unwrap()
            .target_states_received,
        received,
        "Static targets should not retransmit every simulation tick"
    );
    let mut mismatch = Peer::start(
        addresses[2],
        token(3),
        theia_network::procedural::IDENTITY,
        false,
    );
    mismatch.until(|s| s.error.is_some()).await;
    assert!(mismatch.state.borrow().id.is_none());
    assert!(mismatch.finish().await.is_err());
    a.finish().await.unwrap();
    b.until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    let mut fresh = Peer::start(addresses[3], token(4), identity, true);
    let replacement = fresh
        .until(|s| {
            s.targeting.is_some() && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        })
        .await;
    assert_ne!(replacement.id, Some(a_id));
    assert_eq!(replacement.targeting.unwrap().selected, None);
    assert_eq!(
        b.state.borrow().targeting.as_ref().unwrap().selected,
        Some(second.entity)
    );
    fresh.choice.send_replace(Some(Choice {
        target: Some(first.entity),
    }));
    fresh
        .until(|s| {
            s.targeting.as_ref().is_some_and(|t| {
                t.request_sequence == 1 && t.accepted && t.selected == Some(first.entity)
            })
        })
        .await;
    assert!(control.remove_target(first).await.unwrap());
    fresh
        .until(|s| {
            s.targeting
                .as_ref()
                .is_some_and(|t| t.selected.is_none() && t.targets.len() == 1)
        })
        .await;
    assert_eq!(
        b.state.borrow().targeting.as_ref().unwrap().selected,
        Some(second.entity)
    );
    // Repeating the same local ID after server-side removal is still a fresh
    // request and must receive a negative acknowledgement, not disappear locally.
    fresh.choice.send_replace(Some(Choice {
        target: Some(first.entity),
    }));
    fresh
        .until(|s| {
            s.targeting
                .as_ref()
                .is_some_and(|t| t.request_sequence == 2 && !t.accepted && t.selected.is_none())
        })
        .await;
    assert!(control.remove_target(second).await.unwrap());
    b.until(|s| {
        s.targeting
            .as_ref()
            .is_some_and(|t| t.targets.is_empty() && t.selected.is_none())
    })
    .await;
    fresh
        .until(|s| s.targeting.as_ref().is_some_and(|t| t.targets.is_empty()))
        .await;
    fresh.finish().await.unwrap();
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    for worker in workers {
        worker.await.unwrap();
    }
    drop(guard);
    assert_eq!(stats.target_selections_accepted, 6);
    assert_eq!(stats.target_selections_rejected, 2);
    assert!(stats.target_states_sent < 20);
    assert!(stats.max_event_queue_bytes <= 8192);
    let relay_stats: Vec<_> = controls
        .iter()
        .map(|c| {
            let s = c.lock().unwrap().stats.clone();
            assert_eq!(s.overflow, 0);
            s
        })
        .collect();
    assert!(
        relay_stats
            .iter()
            .any(|s| s.configured_drops > 0 && s.duplicates > 0 && s.reordered > 0)
    );
    drop(tickets);
    let report = serde_json::json!({"passed":true,"profile":profile,"world":identity,"server":stats,"relays":relay_stats,
        "scope":"Two actual encrypted targeting clients, baseline-profile rejection and fresh-session replacement over impaired loopback; actual trusted-owner removal with selected-state clearing; no GPU, transported teleport, combat or capacity claim"});
    if let Some(root) = std::env::var_os("THEIA_NETWORK_REPORT_DIR") {
        let root = PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("targeting-wifi.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}

struct RawPeer {
    client: renet::RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
    last: std::time::Instant,
    identity: [u8; 32],
    welcome: Option<theia_network::wire::Welcome>,
    targeting: Option<theia_network::targeting_wire::State>,
    disconnected: bool,
}
impl RawPeer {
    fn start(address: SocketAddr, ticket: &[u8], identity: [u8; 32]) -> Self {
        let token = net::read_ticket(ticket, address).unwrap();
        let transport = theia_network::renet_transport::secure_client(
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            net::now().unwrap(),
            token,
        )
        .unwrap();
        Self {
            client: renet::RenetClient::new(theia_network::renet_transport::connection_config()),
            transport,
            last: std::time::Instant::now(),
            identity,
            welcome: None,
            targeting: None,
            disconnected: false,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&Self) -> bool) {
        use theia_network::renet_transport::{EVENT_CHANNEL, MOVEMENT_CHANNEL};
        tokio::time::timeout(Duration::from_secs(5), async {
            while !predicate(self) {
                let now = std::time::Instant::now();
                let delta = now.duration_since(self.last);
                self.last = now;
                self.client.update(delta);
                if self.transport.update(delta, &mut self.client).is_err() {
                    self.disconnected = true;
                    assert!(predicate(self), "Unexpected raw test-peer disconnect");
                    break;
                }
                while let Some(bytes) = self.client.receive_message(EVENT_CHANNEL) {
                    if self.welcome.is_none() {
                        self.welcome = Some(
                            theia_network::wire::Welcome::decode(&bytes, self.identity).unwrap(),
                        );
                    } else {
                        self.targeting =
                            Some(theia_network::targeting_wire::State::decode(&bytes).unwrap());
                    }
                }
                while self.client.receive_message(MOVEMENT_CHANNEL).is_some() {}
                if self.transport.send_packets(&mut self.client).is_err() {
                    self.disconnected = true;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("Raw target-control acceptance timeout");
    }
    fn select(
        &mut self,
        epoch: u64,
        sequence: u64,
        target: Option<theia_core::identity::EntityId>,
    ) {
        let bytes = theia_network::targeting_wire::Selection {
            epoch,
            sequence,
            target,
        }
        .encode()
        .unwrap();
        self.client
            .send_message(theia_network::renet_transport::EVENT_CHANNEL, bytes);
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn real_encrypted_target_controls_reject_foreign_huge_duplicate_and_malformed_requests() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    let target = world.spawn_development_target([8008., 8000.]).unwrap();
    let identity = world.identity();
    let tickets = Tickets(std::env::temp_dir().join(format!(
        "theia-targeting-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &tickets.0).unwrap();
    let ticket = fs::read(tickets.0.join("client-1.token")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut peer = RawPeer::start(address, &ticket, identity);
    peer.until(|p| p.targeting.is_some()).await;
    let welcome = peer.welcome.clone().unwrap();
    let epoch = welcome.epoch;
    peer.select(if epoch == 7 { 8 } else { 7 }, 1, Some(target.entity));
    peer.select(epoch, u64::MAX, Some(target.entity));
    peer.select(epoch, 1, Some(target.entity));
    peer.until(|p| {
        p.targeting.as_ref().is_some_and(|s| {
            s.request_sequence == 1 && s.accepted && s.selected == Some(target.entity)
        })
    })
    .await;
    peer.select(epoch, 1, None);
    peer.select(epoch, 2, None);
    peer.until(|p| {
        p.targeting
            .as_ref()
            .is_some_and(|s| s.request_sequence == 2 && s.accepted && s.selected.is_none())
    })
    .await;
    peer.select(
        epoch,
        3,
        Some(theia_core::identity::EntityId::new(welcome.id).unwrap()),
    );
    peer.until(|p| {
        p.targeting
            .as_ref()
            .is_some_and(|s| s.request_sequence == 3 && !s.accepted && s.selected.is_none())
    })
    .await;
    peer.select(epoch, 4, Some(target.entity));
    peer.until(|p| {
        p.targeting.as_ref().is_some_and(|s| {
            s.request_sequence == 4 && s.accepted && s.selected == Some(target.entity)
        })
    })
    .await;
    let mut malformed = theia_network::targeting_wire::Selection {
        epoch,
        sequence: 5,
        target: None,
    }
    .encode()
    .unwrap();
    malformed.pop();
    peer.client
        .send_message(theia_network::renet_transport::EVENT_CHANNEL, malformed);
    peer.until(|p| p.disconnected).await;
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    drop(guard);
    drop(tickets);
    assert_eq!(stats.target_selections_accepted, 3);
    assert_eq!(stats.target_selections_rejected, 5);
    assert!(stats.max_event_queue_bytes <= 8192);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_saved_target_profile_preserves_prediction_and_nonblocking_traversal() {
    use std::sync::Arc;
    use theia_network::{
        prediction_driver::Driver,
        procedural::{Patch, Source},
    };
    let fixture = saved_fixture::Fixture::new();
    let store = Arc::new(fixture.open());
    let source = Source::Saved(store.clone());
    let mut world = World::with_terrain_source([8000., 8000.], 1., source.clone()).unwrap();
    world.enable_prediction().unwrap();
    let target = world.spawn_development_target([8008., 8000.]).unwrap();
    let identity = world.identity();
    let patch = Patch::load(theia_core::coordinates::ChunkId { x: 62, z: 62 }, &source).unwrap();
    let tickets = Tickets(std::env::temp_dir().join(format!(
        "theia-targeting-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &tickets.0).unwrap();
    let ticket = fs::read(tickets.0.join("client-1.token")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut peer = Peer::mode(address, ticket, identity, true, true);
    peer._ready
        .send_replace(Some(patch.ids.iter().copied().collect()));
    peer.until(|s| {
        s.targeting.is_some() && s.correction.is_some_and(|c| c.state.ready_sequence > 0)
    })
    .await;
    peer.choice.send_replace(Some(Choice {
        target: Some(target.entity),
    }));
    peer.until(|s| {
        s.targeting
            .as_ref()
            .is_some_and(|s| s.accepted && s.selected == Some(target.entity))
    })
    .await;
    let mut driver = Driver::default();
    let started = std::time::Instant::now();
    let mut final_pose = None;
    while started.elapsed() < Duration::from_millis(2600) {
        let state = peer.state.borrow().clone();
        assert!(state.error.is_none(), "{:?}", state.error);
        assert_eq!(
            state.targeting.as_ref().unwrap().selected,
            Some(target.entity),
            "Movement snapshot lost target state"
        );
        let intent = Intent {
            axes: if started.elapsed() < Duration::from_millis(1300) {
                [1000, 0]
            } else {
                [0, 0]
            },
            sprint: true,
            jump: false,
        };
        if let Some(driven) = driver
            .tick(&state, intent, Some(&patch), std::time::Instant::now())
            .unwrap()
        {
            final_pose = Some(driven.pose);
            if let Some(packet) = driven.packet {
                peer.commands.as_ref().unwrap().try_send(packet).unwrap();
            }
        }
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
    let state = peer.state.borrow().clone();
    let truth = state.correction.unwrap().state.pose.feet.metres();
    let predicted = final_pose.unwrap().feet.metres();
    let error = predicted
        .into_iter()
        .zip(truth)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(truth[0] > 8010., "Target blocked authoritative traversal");
    assert!(error < 0.003);
    assert!(
        driver.stats.max_history <= 64
            && driver.stats.predicted_steps > 50
            && driver.stats.queue_drops == 0
    );
    peer.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    drop(guard);
    drop(tickets);
    assert_eq!(stats.target_selections_accepted, 1);
    assert_eq!(stats.movement_recoveries, 0);
    assert!(stats.queued_commands > 50);
    assert!(store.stats().tile_reads >= 9);
}
