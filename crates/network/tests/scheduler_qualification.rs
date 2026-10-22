//! Wall-clock qualification of the executable authority, not a synthetic step loop.
use renet::RenetClient;
use renet_netcode::NetcodeClientTransport;
use std::{net::UdpSocket, path::PathBuf, time::Duration};
use theia_core::time::TickRate;
use theia_network::{
    renet_runtime::{self as runtime, ServerStats},
    renet_transport::*,
    simulation::World,
    terrain::GRAYBOX_ID,
    wire::*,
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    time::Instant,
};

type Stall = (Duration, oneshot::Sender<()>);
struct Authority {
    directory: PathBuf,
    stop: watch::Sender<bool>,
    stall: mpsc::Sender<Stall>,
    worker: Option<std::thread::JoinHandle<ServerStats>>,
}
impl Drop for Authority {
    fn drop(&mut self) {
        self.stop.send_replace(true);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
struct Peer {
    client: RenetClient,
    transport: NetcodeClientTransport,
    welcome: Option<Welcome>,
    snapshot: Option<Snapshot>,
    sequence: u64,
    sent: u64,
    snapshots: u64,
    next_input: Instant,
}
struct Load {
    peers: Vec<Peer>,
    last: Instant,
    rate: TickRate,
}
impl Load {
    // Four messages per 1/30 s: two new intents, one replay and one wrong epoch.
    // Deliberate bursts exercise coalescing without exceeding the 180/s guard.
    async fn pump(&mut self, duration: Duration, axes: Option<[i16; 2]>, flood: bool) {
        let end = Instant::now() + duration;
        while Instant::now() < end {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let now = Instant::now();
            let delta = now - self.last;
            self.last = now;
            for (index, peer) in self.peers.iter_mut().enumerate() {
                if peer.client.is_disconnected() {
                    assert!(
                        flood && index == 0,
                        "Only the budget violator may disconnect"
                    );
                    continue;
                }
                peer.client.update(delta);
                if let Err(error) = peer.transport.update(delta, &mut peer.client) {
                    assert!(
                        flood && index == 0 && peer.transport.disconnect_reason().is_some(),
                        "{error}"
                    );
                    peer.client.disconnect_due_to_transport();
                    continue;
                }
                if peer.transport.disconnect_reason().is_some() {
                    assert!(flood && index == 0);
                    peer.client.disconnect_due_to_transport();
                    continue;
                }
                if peer.client.is_disconnected() {
                    assert!(
                        flood && index == 0,
                        "Only the budget violator may disconnect"
                    );
                    continue;
                }
                if let Some(bytes) = peer.client.receive_message(EVENT_CHANNEL) {
                    assert!(peer.welcome.is_none());
                    peer.welcome = Some(Welcome::decode(&bytes, GRAYBOX_ID).unwrap());
                }
                while let Some(bytes) = peer.client.receive_message(MOVEMENT_CHANNEL) {
                    let snapshot = Snapshot::decode(&bytes).unwrap();
                    let Some(welcome) = &peer.welcome else {
                        continue;
                    };
                    assert!(welcome.accepts(&snapshot));
                    if let Some(old) = &peer.snapshot {
                        if snapshot.tick <= old.tick {
                            continue;
                        }
                        let a = old.players.iter().find(|p| p.id == welcome.id).unwrap();
                        let b = snapshot
                            .players
                            .iter()
                            .find(|p| p.id == welcome.id)
                            .unwrap();
                        assert!(b.applied_sequence >= a.applied_sequence);
                        let distance =
                            ((b.xz[0] - a.xz[0]).powi(2) + (b.xz[1] - a.xz[1]).powi(2)).sqrt();
                        assert!(
                            distance
                                <= 5. * (snapshot.tick - old.tick) as f32 * self.rate.seconds()
                                    + 0.002,
                            "Traffic must not increase simulation speed: {distance}"
                        );
                    }
                    peer.snapshots += 1;
                    peer.snapshot = Some(snapshot);
                }
                if let (Some(welcome), Some(axes)) = (&peer.welcome, axes)
                    && now >= peer.next_input
                    && peer.client.is_connected()
                {
                    peer.next_input = now + Duration::from_secs_f64(1. / 30.);
                    for slot in 0..if flood && index == 0 { 64 } else { 4 } {
                        if slot % 4 == 0 || slot % 4 == 3 {
                            peer.sequence += 1;
                        }
                        let input = Input {
                            version: VERSION,
                            epoch: if slot % 4 == 1 {
                                welcome.epoch ^ 1
                            } else {
                                welcome.epoch
                            },
                            sequence: if slot % 4 == 1 {
                                u64::MAX
                            } else {
                                peer.sequence
                            },
                            intent: Intent {
                                jump: false,
                                axes,
                                sprint: false,
                            },
                        };
                        peer.client
                            .send_message(MOVEMENT_CHANNEL, encode(&input).unwrap());
                        peer.sent += 1;
                    }
                }
                if let Err(error) = peer.transport.send_packets(&mut peer.client) {
                    assert!(
                        flood && index == 0 && peer.client.is_disconnected(),
                        "{error}"
                    );
                }
            }
        }
    }
    fn positions(&self) -> Vec<[f32; 2]> {
        self.peers
            .iter()
            .map(|p| {
                p.snapshot
                    .as_ref()
                    .unwrap()
                    .players
                    .iter()
                    .find(|a| a.id == p.welcome.as_ref().unwrap().id)
                    .unwrap()
                    .xz
            })
            .collect()
    }
}

#[tokio::test]
async fn eight_authenticated_clients_rates_expiry_stall_recovery_and_input_budget() {
    let mut reports = Vec::new();
    for hz in [20, 30, 60] {
        let rate = TickRate::from_hz(hz).unwrap();
        let directory =
            std::env::temp_dir().join(format!("theia-scheduler-{}-{hz}", std::process::id()));
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let transport = runtime::issue_local(socket, &directory).unwrap();
        let (stop, shutdown) = watch::channel(false);
        let (stall, mut requests) = mpsc::channel::<Stall>(1);
        // A dedicated OS thread permits the clients to keep sending while the
        // authority executor is deliberately stalled. No production stall hook.
        let worker = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    tokio::spawn(async move {
                        while let Some((duration, started)) = requests.recv().await {
                            let _ = started.send(());
                            std::thread::sleep(duration);
                        }
                    });
                    runtime::run_server_measured(transport, shutdown, World::at_rate(rate))
                        .await
                        .unwrap()
                })
        });
        let mut authority = Authority {
            directory,
            stop,
            stall,
            worker: Some(worker),
        };
        let mut load = Load {
            peers: Vec::new(),
            last: Instant::now(),
            rate,
        };
        for id in 1..=MAX_PLAYERS {
            let bytes =
                std::fs::read(authority.directory.join(format!("client-{id}.token"))).unwrap();
            let token = runtime::read_ticket(&bytes, address).unwrap();
            load.peers.push(Peer {
                client: RenetClient::new(connection_config()),
                transport: secure_client(
                    UdpSocket::bind("127.0.0.1:0").unwrap(),
                    runtime::now().unwrap(),
                    token,
                )
                .unwrap(),
                welcome: None,
                snapshot: None,
                sequence: 0,
                sent: 0,
                snapshots: 0,
                next_input: Instant::now(),
            });
        }
        let admission_deadline = Instant::now() + Duration::from_secs(5);
        while !load.peers.iter().all(|p| {
            p.snapshot
                .as_ref()
                .is_some_and(|s| s.players.len() == MAX_PLAYERS)
        }) {
            assert!(
                Instant::now() < admission_deadline,
                "Eight-peer admission timed out"
            );
            load.pump(Duration::from_millis(50), None, false).await;
        }
        load.pump(Duration::from_millis(500), None, false).await;
        for peer in &load.peers {
            assert_eq!(peer.snapshot.as_ref().unwrap().players.len(), MAX_PLAYERS);
            assert_eq!(peer.snapshot.as_ref().unwrap().tick_hz, hz);
        }
        let initial = load.positions();
        let baseline_tick = load.peers[1].snapshot.as_ref().unwrap().tick;
        let baseline_start = Instant::now();
        load.pump(Duration::from_secs(3), Some([0, -1000]), false)
            .await;
        let baseline_seconds = baseline_start.elapsed().as_secs_f64();
        let baseline_steps = load.peers[1].snapshot.as_ref().unwrap().tick - baseline_tick;
        assert!(
            (baseline_steps as f64 - baseline_seconds * f64::from(hz)).abs()
                < f64::from(hz) * 0.2 + 2.
        );
        for (a, b) in initial.iter().zip(load.positions()) {
            assert!(a[1] - b[1] > 8.);
        }
        load.pump(Duration::from_millis(650), None, false).await;
        let stopped = load.positions();
        load.pump(Duration::from_millis(200), None, false).await;
        assert_eq!(
            stopped,
            load.positions(),
            "Inputs must expire in simulation time"
        );
        load.pump(Duration::from_millis(400), Some([0, 1000]), false)
            .await;
        let before = load.peers[1].snapshot.clone().unwrap();
        let (started, started_rx) = oneshot::channel();
        authority
            .stall
            .send((Duration::from_millis(250), started))
            .await
            .unwrap();
        started_rx.await.unwrap();
        let recovery_start = Instant::now();
        load.pump(Duration::from_millis(900), Some([0, 1000]), false)
            .await;
        let recovery_seconds = recovery_start.elapsed().as_secs_f64();
        let recovered = load.peers[1].snapshot.as_ref().unwrap();
        let recovery_steps = recovered.tick - before.tick;
        assert!(
            (recovery_steps as f64) < recovery_seconds * f64::from(hz) - f64::from(hz) * 0.12,
            "Skip policy must not replay the entire stalled wall time"
        );
        for peer in &load.peers {
            let p = peer
                .snapshot
                .as_ref()
                .unwrap()
                .players
                .iter()
                .find(|p| p.id == peer.welcome.as_ref().unwrap().id)
                .unwrap();
            let old = before.players.iter().find(|old| old.id == p.id).unwrap();
            assert!(p.applied_sequence > old.applied_sequence);
            assert!(p.xz[1] > old.xz[1] + 1.);
        }
        // An authenticated sender over the input budget is removed; the other
        // seven keep advancing. It cannot buy extra simulation steps.
        load.pump(Duration::from_millis(700), Some([0, 1000]), true)
            .await;
        assert!(!load.peers[0].client.is_connected());
        assert_eq!(
            load.peers[1].snapshot.as_ref().unwrap().players.len(),
            MAX_PLAYERS - 1
        );
        authority.stop.send_replace(true);
        let stats = authority.worker.take().unwrap().join().unwrap();
        assert_eq!(stats.max_players, MAX_PLAYERS);
        assert!(stats.max_pending_inputs <= MAX_PLAYERS);
        assert!(stats.max_input_batch <= 32);
        assert!(stats.max_movement_queue_bytes <= CHANNEL_MEMORY_BYTES);
        assert!(stats.max_event_queue_bytes <= CHANNEL_MEMORY_BYTES);
        assert!(stats.received_inputs > stats.applied_inputs * 2);
        assert!(stats.skipped_intervals >= u64::from(hz / 8));
        assert!(stats.simulation.missed_deadlines > 0);
        assert!(stats.simulation.lateness_max_us > 150_000.);
        for peer in &mut load.peers {
            peer.transport.disconnect();
        }
        reports.push(serde_json::json!({
            "tick_hz":hz, "actors":MAX_PLAYERS, "transport":"secure Renet real loopback UDP",
            "baseline_seconds":baseline_seconds, "baseline_steps":baseline_steps,
            "stall_ms":250, "recovery_seconds":recovery_seconds, "recovery_steps":recovery_steps,
            "client_sent_inputs":load.peers.iter().map(|p|p.sent).sum::<u64>(),
            "client_snapshots":load.peers.iter().map(|p|p.snapshots).sum::<u64>(),
            "stats":stats,
        }));
    }
    let report = serde_json::json!({"debug_assertions":cfg!(debug_assertions), "os":std::env::consts::OS, "arch":std::env::consts::ARCH, "profiles":reports});
    println!(
        "scheduler_qualification {}",
        serde_json::to_string(&report).unwrap()
    );
    if let Ok(directory) = std::env::var("THEIA_SCHEDULER_REPORT_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            PathBuf::from(directory).join("scheduler.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
