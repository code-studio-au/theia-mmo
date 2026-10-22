//! Wall-clock qualification of the exact executable Renet bridge over real UDP.
#[path = "support/relay.rs"]
mod relay;
use relay::{Control, Profile, QUEUE_LIMIT, Relay};
use renet_netcode::{
    ConnectToken, NetcodeServerTransport, ServerAuthentication, ServerConfig, generate_random_bytes,
};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};
use theia_network::{
    client::State,
    renet_runtime::{self as runtime, ServerStats},
    renet_transport::{CHANNEL_MEMORY_BYTES, PROTOCOL_ID},
    simulation::World,
    wire::{Intent, Player},
};
use tokio::{
    sync::watch,
    task::JoinHandle,
    time::{Instant, sleep},
};

struct Authority {
    world_id: [u8; 32],
    addresses: Vec<SocketAddr>,
    controls: Vec<Arc<Mutex<Control>>>,
    tickets: Vec<Vec<u8>>,
    stop: watch::Sender<bool>,
    worker: JoinHandle<theia_network::Result<ServerStats>>,
    relays: Vec<JoinHandle<()>>,
    started: Instant,
}
impl Authority {
    fn start(profile: Profile) -> Self {
        Self::in_world(profile, World::default())
    }
    fn in_world(profile: Profile, world: World) -> Self {
        let world_id = world.identity();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let relays: Vec<_> = (0..3).map(|_| Relay::bind(profile)).collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let controls = relays.iter().map(|r| r.control.clone()).collect();
        let now = runtime::now().unwrap();
        let key = generate_random_bytes();
        let transport = NetcodeServerTransport::new(
            ServerConfig {
                current_time: now,
                max_clients: 8,
                protocol_id: PROTOCOL_ID,
                public_addresses: addresses.clone(),
                authentication: ServerAuthentication::Secure { private_key: key },
            },
            socket,
        )
        .unwrap();
        let tickets = addresses
            .iter()
            .enumerate()
            .map(|(index, address)| {
                let token = ConnectToken::generate(
                    now,
                    PROTOCOL_ID,
                    120,
                    index as u64 + 1,
                    5,
                    vec![*address],
                    None,
                    &key,
                )
                .unwrap();
                let mut bytes = Vec::new();
                token.write(&mut bytes).unwrap();
                bytes
            })
            .collect();
        let (stop, shutdown) = watch::channel(false);
        let workers = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let worker = tokio::spawn(runtime::run_server_measured(transport, shutdown, world));
        Self {
            world_id,
            addresses,
            controls,
            tickets,
            stop,
            worker,
            relays: workers,
            started: Instant::now(),
        }
    }
    fn peer(&self, index: usize) -> Peer {
        self.peer_mode(index, false)
    }
    fn peer_mode(&self, index: usize, predicted: bool) -> Peer {
        let (input, intent) = watch::channel(Intent::default());
        let (state_tx, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let address = self.addresses[index];
        let bytes = self.tickets[index].clone();
        let world_id = self.world_id;
        let (ready, readiness) = watch::channel(None);
        let (commands, command_queue) = tokio::sync::mpsc::channel(8);
        let worker = tokio::spawn(async move {
            if predicted {
                runtime::run_client_predicted(
                    address,
                    &bytes,
                    intent,
                    state_tx,
                    shutdown,
                    world_id,
                    readiness,
                    command_queue,
                )
                .await
            } else {
                runtime::run_client_with_chunks(
                    address, &bytes, intent, state_tx, shutdown, world_id, readiness,
                )
                .await
            }
        });
        Peer {
            commands: predicted.then_some(commands),
            ready,
            input,
            state,
            stop,
            worker,
        }
    }
    async fn finish(self) -> serde_json::Value {
        self.stop.send_replace(true);
        let stats = self.worker.await.unwrap().unwrap();
        for relay in self.relays {
            relay.await.unwrap();
        }
        assert!(stats.max_pending_inputs <= 3);
        assert!(stats.max_input_batch <= 32);
        assert!(stats.max_movement_queue_bytes <= CHANNEL_MEMORY_BYTES);
        assert!(stats.max_event_queue_bytes <= CHANNEL_MEMORY_BYTES);
        let seconds = self.started.elapsed().as_secs_f64();
        let relays: Vec<_> = self
            .controls
            .iter()
            .map(|c| {
                let stats = c.lock().unwrap().stats.clone();
                assert_eq!(stats.overflow, 0);
                assert!(stats.queue_high_water <= QUEUE_LIMIT);
                let bytes_per_second = stats.received_bytes.map(|n| n as f64 / seconds);
                assert!(bytes_per_second.iter().all(|n| *n < 128. * 1024.));
                serde_json::json!({"stats":stats,"udp_received_bytes_per_second":bytes_per_second})
            })
            .collect();
        serde_json::json!({"duration_seconds":seconds,"server":stats,"relays":relays})
    }
}
struct Peer {
    commands: Option<tokio::sync::mpsc::Sender<theia_network::prediction_wire::Commands>>,
    ready: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    input: watch::Sender<Intent>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    worker: JoinHandle<theia_network::Result<()>>,
}
impl Peer {
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let state = self.state.borrow().clone();
                if predicate(&state) {
                    return state;
                }
                self.state
                    .changed()
                    .await
                    .expect("Client closed before acceptance");
            }
        })
        .await
        .expect("Adverse-network acceptance timeout")
    }
    async fn finish(self) {
        self.stop.send_replace(true);
        self.worker.await.unwrap().unwrap();
    }
}
fn actor(state: &State, id: u64) -> Option<&Player> {
    state.snapshot.as_ref()?.players.iter().find(|p| p.id == id)
}
fn save(name: &str, report: &serde_json::Value) {
    println!("renet_qualification {name} {report}");
    if let Some(path) = std::env::var_os("THEIA_NETWORK_REPORT_DIR") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join(format!("{name}.json")),
            serde_json::to_vec_pretty(report).unwrap(),
        )
        .unwrap();
    }
}

#[tokio::test]
async fn real_udp_loss_delay_duplicate_reorder_and_bandwidth_profiles() {
    for profile in [
        Profile::BASELINE,
        Profile {
            name: "wifi",
            delay_ms: 40,
            jitter_ms: 60,
            loss_every: 10,
            duplicate_every: 17,
        },
        Profile {
            name: "poor",
            delay_ms: 100,
            jitter_ms: 150,
            loss_every: 5,
            duplicate_every: 11,
        },
    ] {
        let authority = Authority::start(profile);
        let mut a = authority.peer(0);
        let mut b = authority.peer(1);
        let initial = a
            .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
            .await;
        b.until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
            .await;
        let id = initial.id.unwrap();
        let start = actor(&initial, id).unwrap().xz;
        a.input.send_replace(Intent {
            jump: false,
            axes: [0, -1000],
            sprint: false,
        });
        let command = Instant::now();
        a.until(|s| actor(s, id).is_some_and(|p| p.xz[1] < start[1] - 0.1))
            .await;
        let observed_motion_ms = command.elapsed().as_secs_f64() * 1000.;
        let mut tick = 0;
        let mut ack = 0;
        let mut samples = Vec::new();
        let sampling = Instant::now();
        while sampling.elapsed() < Duration::from_secs(3) {
            let state = a.state.borrow().clone();
            let snapshot = state.snapshot.as_ref().unwrap();
            let own = actor(&state, id).unwrap();
            assert!(snapshot.tick >= tick && own.applied_sequence >= ack);
            tick = snapshot.tick;
            ack = own.applied_sequence;
            if let Some(stats) = state.transport {
                samples.push(stats);
            }
            sleep(Duration::from_millis(25)).await;
        }
        let moved = a.state.borrow().clone();
        assert!(actor(&moved, id).unwrap().xz[1] < start[1] - 5.);
        assert!(ack > 20);
        a.input.send_replace(Intent::default());
        sleep(Duration::from_millis(
            2 * (profile.delay_ms + profile.jitter_ms) + 800,
        ))
        .await;
        let stopped = actor(&a.state.borrow(), id).unwrap().xz;
        sleep(Duration::from_millis(350)).await;
        assert_eq!(actor(&a.state.borrow(), id).unwrap().xz, stopped);
        assert_eq!(actor(&b.state.borrow(), id).unwrap().xz, stopped);
        let max_queue = samples
            .iter()
            .map(|s| s.max_movement_queue_bytes)
            .max()
            .unwrap();
        assert!(max_queue > 0 && max_queue <= CHANNEL_MEMORY_BYTES);
        let mut rtts: Vec<_> = samples.iter().map(|s| s.rtt_ms).collect();
        rtts.sort_by(f64::total_cmp);
        let median = rtts[rtts.len() / 2];
        assert!(median < 3000.);
        if profile.delay_ms > 0 {
            assert!(median >= profile.delay_ms as f64);
            let control = authority.controls[0].lock().unwrap();
            assert!(
                control.stats.configured_drops > 0
                    && control.stats.duplicates > 0
                    && control.stats.reordered > 0
            );
        }
        let diagnostics = samples.last().unwrap().clone();
        a.finish().await;
        b.finish().await;
        let report = serde_json::json!({"profile":profile,"client_rtt_p50_ms":median,"client_rtt_p95_ms":rtts[rtts.len()*95/100],"observed_motion_ms":observed_motion_ms,"client":diagnostics,"runtime":authority.finish().await});
        save(profile.name, &report);
    }
}

#[tokio::test]
async fn outages_expire_inputs_disconnect_and_old_packets_cannot_move_replacement() {
    let authority = Authority::start(Profile::BASELINE);
    let mut a = authority.peer(0);
    let mut observer = authority.peer(1);
    let initial = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    observer
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    let id = initial.id.unwrap();
    a.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    observer
        .until(|s| actor(s, id).is_some_and(|p| p.xz[1] < -1.))
        .await;
    authority.controls[0].lock().unwrap().blocked = true;
    // Stop is requested while delivery is blocked. The observer must still
    // see server expiry, and no queued moving intent can race the restore.
    a.input.send_replace(Intent::default());
    sleep(Duration::from_millis(850)).await;
    let stopped = actor(&observer.state.borrow(), id).unwrap().xz;
    sleep(Duration::from_millis(1150)).await;
    assert_eq!(
        actor(&observer.state.borrow(), id).unwrap().xz,
        stopped,
        "Input must expire during two-second outage"
    );
    a.input.send_replace(Intent::default());
    authority.controls[0].lock().unwrap().blocked = false;
    a.until(|s| s.id == Some(id) && actor(s, id).is_some_and(|p| p.xz == stopped))
        .await;
    // Short interruption retained the session. Now force a transport timeout.
    a.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    observer
        .until(|s| actor(s, id).is_some_and(|p| p.xz[1] < stopped[1] - 0.5))
        .await;
    let old_packets: Vec<_> = authority.controls[0]
        .lock()
        .unwrap()
        .captures
        .iter()
        .cloned()
        .collect();
    authority.controls[0].lock().unwrap().blocked = true;
    let outage = Instant::now();
    observer
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    let timeout_seconds = outage.elapsed().as_secs_f64();
    assert!(timeout_seconds > 4. && timeout_seconds < 8.);
    assert!(a.worker.await.unwrap().is_err());
    let mut replacement = authority.peer(2);
    let fresh = replacement
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    let new_id = fresh.id.unwrap();
    assert!(new_id > id);
    let fresh_pose = actor(&fresh, new_id).unwrap().xz;
    for _ in 0..3 {
        {
            let mut control = authority.controls[2].lock().unwrap();
            control.stats.injected_old_packets += old_packets.len() as u64;
            control.inject.extend(old_packets.iter().cloned());
        }
        sleep(Duration::from_millis(100)).await;
    }
    sleep(Duration::from_millis(500)).await;
    let after = replacement.state.borrow().clone();
    assert!(
        !replacement.worker.is_finished(),
        "Old traffic disconnected replacement"
    );
    assert!(after.snapshot.as_ref().unwrap().tick > fresh.snapshot.as_ref().unwrap().tick + 10);
    assert!(
        actor(&after, new_id).unwrap().applied_sequence
            > actor(&fresh, new_id).unwrap().applied_sequence
    );
    assert_eq!(
        actor(&observer.state.borrow(), new_id).unwrap().xz,
        fresh_pose
    );
    assert_eq!(actor(&after, new_id).unwrap().xz, fresh_pose);
    assert!(actor(&after, id).is_none());
    assert!(actor(&after, new_id).unwrap().applied_sequence > 0);
    replacement.finish().await;
    observer.finish().await;
    save(
        "outages",
        &serde_json::json!({"short_outage_seconds":2,"timeout_observed_seconds":timeout_seconds,"old_packet_count":old_packets.len()*3,"replacement_stationary":true,"runtime":authority.finish().await}),
    );
}

#[test]
fn reliable_and_unreliable_sender_backlogs_remain_bounded_without_io() {
    use theia_network::renet_transport::{EVENT_CHANNEL, MOVEMENT_CHANNEL, connection_config};
    let mut client = renet::RenetClient::new(connection_config());
    for channel in [EVENT_CHANNEL, MOVEMENT_CHANNEL] {
        let mut count = 0;
        while client.can_send_message(channel, 1024) {
            client.send_message(channel, vec![1u8; 1024]);
            count += 1;
            assert!(count <= 8);
        }
        assert_eq!(count, 8);
        assert_eq!(client.channel_available_memory(channel), 0);
        assert!(!client.can_send_message(channel, 1));
    }
}

#[tokio::test]
async fn procedural_seams_and_readiness_withdrawal_survive_delayed_lost_reordered_udp() {
    use theia_core::coordinates::ChunkId;
    use theia_network::procedural;
    let profile = Profile {
        name: "procedural-wifi",
        delay_ms: 40,
        jitter_ms: 60,
        loss_every: 10,
        duplicate_every: 17,
    };
    let authority = Authority::in_world(
        profile,
        World::with_procedural([8063.5, 8000.], 1.).unwrap(),
    );
    let mut a = authority.peer(0);
    let mut b = authority.peer(1);
    let initial = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    b.until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    let id = initial.id.unwrap();
    let global = |s: &State, id| {
        let snapshot = s.snapshot.as_ref().unwrap();
        snapshot.global_position(actor(s, id).unwrap()).unwrap()
    };
    let start = global(&initial, id);
    a.input.send_replace(Intent {
        axes: [1000, 0],
        sprint: true,
        jump: false,
    });
    sleep(Duration::from_millis(700)).await;
    assert_eq!(
        global(&a.state.borrow(), id).metres()[0],
        start.metres()[0],
        "Movement preceded terrain readiness"
    );
    let ids: Vec<_> = procedural::neighbours(ChunkId { x: 62, z: 62 }, 1)
        .into_iter()
        .collect();
    a.ready.send_replace(Some(ids.clone()));
    b.ready.send_replace(Some(ids.clone()));
    a.until(|s| s.chunk_permission_confirmed(&ids)).await;
    a.until(|s| global(s, id).metres()[0] > start.metres()[0] + 8.)
        .await;
    let crossed = a.state.borrow().clone();
    assert!(procedural::owner(global(&crossed, id)).unwrap().x >= 63);
    let mut previous_tick = 0;
    let mut previous_ack = 0;
    for _ in 0..20 {
        let state = a.state.borrow().clone();
        let snapshot = state.snapshot.as_ref().unwrap();
        let own = actor(&state, id).unwrap();
        assert!(snapshot.tick >= previous_tick && own.applied_sequence >= previous_ack);
        previous_tick = snapshot.tick;
        previous_ack = own.applied_sequence;
        sleep(Duration::from_millis(25)).await;
    }
    a.ready.send_replace(Some(vec![]));
    a.until(|s| s.chunk_permission_confirmed(&[])).await;
    let withdrawn = global(&a.state.borrow(), id);
    sleep(Duration::from_millis(600)).await;
    let held = global(&a.state.borrow(), id);
    assert!((held.metres()[0] - withdrawn.metres()[0]).abs() < 0.00001);
    a.ready.send_replace(Some(ids.clone()));
    a.until(|s| s.chunk_permission_confirmed(&ids)).await;
    a.until(|s| global(s, id).metres()[0] > held.metres()[0] + 2.)
        .await;
    a.input.send_replace(Intent::default());
    sleep(Duration::from_millis(1200)).await;
    let stopped = global(&a.state.borrow(), id);
    sleep(Duration::from_millis(350)).await;
    assert_eq!(global(&a.state.borrow(), id), stopped);
    b.until(|s| actor(s, id).is_some_and(|_| global(s, id) == stopped))
        .await;
    a.finish().await;
    b.finish().await;
    let report = authority.finish().await;
    assert_eq!(report["server"]["movement_recoveries"], 0);
    assert!(report["server"]["accepted_chunk_ready"].as_u64().unwrap() >= 4);
    assert!(
        report["relays"][0]["stats"]["configured_drops"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(report["relays"][0]["stats"]["reordered"].as_u64().unwrap() > 0);
    save(
        "procedural-wifi",
        &serde_json::json!({"profile":profile,"seam_crossed":true,"withdrawal_held":true,"regrant_resumed":true,"runtime":report}),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn prediction_profile_converges_over_real_encrypted_impaired_udp_at_every_tick_rate() {
    use theia_core::{coordinates::ChunkId, time::TickRate};
    use theia_network::{prediction_driver::Driver, procedural::Patch};
    let profile = Profile {
        name: "prediction-wifi",
        delay_ms: 40,
        jitter_ms: 60,
        loss_every: 10,
        duplicate_every: 17,
    };
    let patch = Patch::build(ChunkId { x: 62, z: 62 });
    let mut reports = Vec::new();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = World::with_procedural([8062., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        let authority = Authority::in_world(profile, world);
        let mut peer = authority.peer_mode(0, true);
        peer.until(|s| s.correction.is_some()).await;
        peer.ready
            .send_replace(Some(patch.ids.iter().copied().collect()));
        peer.until(|s| {
            s.correction.is_some_and(|c| c.state.ready_sequence > 0) && s.snapshot.is_some()
        })
        .await;
        let mut driver = Driver::default();
        let started = std::time::Instant::now();
        let mut first_response = None;
        let mut start_x = None;
        let mut final_pose = None;
        let mut withheld = None;
        let mut regranted = None;
        let mut saw_airborne = false;
        let mut withdrawn = false;
        let mut granted = false;
        while started.elapsed() < Duration::from_secs(10) {
            let elapsed = started.elapsed().as_secs_f64();
            if elapsed >= 3.5 && !withdrawn {
                peer.ready.send_replace(Some(vec![]));
                withdrawn = true;
            }
            if elapsed >= 5. && !granted {
                peer.ready
                    .send_replace(Some(patch.ids.iter().copied().collect()));
                granted = true;
            }
            let state = peer.state.borrow().clone();
            assert!(state.error.is_none(), "{:?}", state.error);
            let intent = Intent {
                axes: if (0.5..7.).contains(&elapsed) {
                    [1000, 0]
                } else {
                    [0, 0]
                },
                sprint: true,
                jump: (1.2..2.2).contains(&elapsed),
            };
            if let Some(driven) = driver
                .tick(&state, intent, Some(&patch), std::time::Instant::now())
                .unwrap()
            {
                let x = driven.pose.feet.metres()[0];
                let initial = *start_x.get_or_insert(x);
                if elapsed >= 0.5 && x > initial + 0.005 && first_response.is_none() {
                    first_response = Some(elapsed - 0.5);
                }
                if !driven.pose.motion.grounded && (1.3..2.4).contains(&elapsed) {
                    saw_airborne = true;
                }
                if (4.3..4.5).contains(&elapsed) {
                    withheld = Some(x);
                }
                if (4.8..5.).contains(&elapsed) {
                    assert!(
                        (x - withheld.unwrap()).abs() < 0.003,
                        "Withdrawal failed to hold prediction"
                    );
                }
                if elapsed > 6.5 {
                    regranted = Some(x);
                }
                final_pose = Some(driven.pose);
                if let Some(packet) = driven.packet {
                    peer.commands.as_ref().unwrap().try_send(packet).unwrap();
                }
            }
            sleep(Duration::from_millis(4)).await;
        }
        let state = peer.state.borrow().clone();
        let truth = state.correction.unwrap().state;
        let final_pose = final_pose.unwrap();
        let error = final_pose
            .feet
            .metres()
            .into_iter()
            .zip(truth.pose.feet.metres())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(
            error < 0.003,
            "Stopped prediction failed to converge: {error}"
        );
        assert!(
            first_response.unwrap() < rate.period().as_secs_f64() * 2. + 0.02,
            "Local response waited for RTT"
        );
        assert!(saw_airborne);
        assert!(regranted.unwrap() > withheld.unwrap() + 1.);
        assert!(final_pose.feet.metres()[0] > 8064., "No seam crossing");
        assert!(driver.stats.predicted_steps > 80 && driver.stats.replayed_steps > 0);
        assert!(driver.stats.max_history <= 64);
        let stats = driver.stats.clone();
        peer.finish().await;
        let runtime = authority.finish().await;
        assert_eq!(runtime["server"]["movement_recoveries"], 0);
        assert!(runtime["server"]["queued_commands"].as_u64().unwrap() > 50);
        reports.push(serde_json::json!({"tick_hz":rate.hz(),"first_local_response_ms":first_response.unwrap()*1000.,"stopped_error_metres":error,"driver":stats,"runtime":runtime}));
    }
    save(
        "prediction-wifi",
        &serde_json::json!({"profile":profile,"rates":reports}),
    );
}
