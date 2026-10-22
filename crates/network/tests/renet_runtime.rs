#[path = "support/chunk_fixture.rs"]
mod chunk_fixture;
// Real-time executable bridge tests, using actual authenticated UDP sockets.
use std::{
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use theia_network::{
    client::State, renet_runtime as runtime, simulation::World, terrain::GRAYBOX_ID, wire::Intent,
};
use tokio::{sync::watch, task::JoinHandle};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Authority {
    address: SocketAddr,
    directory: PathBuf,
    stop: watch::Sender<bool>,
    worker: JoinHandle<theia_network::Result<runtime::ServerStats>>,
}
impl Authority {
    fn start() -> Self {
        Self::start_world(World::default())
    }
    fn start_world(world: World) -> Self {
        Self::start_managed(world, None)
    }
    fn start_managed(world: World, chunks: Option<runtime::ChunkRequests>) -> Self {
        Self::start_policy(world, chunks, false)
    }
    fn start_policy(world: World, chunks: Option<runtime::ChunkRequests>, stream: bool) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "theia-renet-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let transport = runtime::issue_local(socket, &directory).unwrap();
        let (stop, shutdown) = watch::channel(false);
        let worker = tokio::spawn(async move {
            if stream {
                return runtime::run_server_streaming(transport, shutdown, world).await;
            }
            match chunks {
                Some(requests) => {
                    runtime::run_server_with_chunks(transport, shutdown, world, requests).await
                }
                None => runtime::run_server_measured(transport, shutdown, world).await,
            }
        });
        Self {
            address,
            directory,
            stop,
            worker,
        }
    }
    fn peer(&self, ticket: u64, world: [u8; 32]) -> Peer {
        let bytes = std::fs::read(self.directory.join(format!("client-{ticket}.token"))).unwrap();
        let (input, intent) = watch::channel(Intent::default());
        let (chunks, ready) = watch::channel(None);
        let (state_tx, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let address = self.address;
        let worker = tokio::spawn(async move {
            runtime::run_client_with_chunks(
                address, &bytes, intent, state_tx, shutdown, world, ready,
            )
            .await
        });
        Peer {
            input,
            chunks,
            state,
            stop,
            worker,
        }
    }
    async fn finish(self) -> runtime::ServerStats {
        self.stop.send_replace(true);
        let stats = self.worker.await.unwrap().unwrap();
        std::fs::remove_dir_all(self.directory).unwrap();
        stats
    }
}
struct Peer {
    input: watch::Sender<Intent>,
    chunks: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    worker: JoinHandle<theia_network::Result<()>>,
}
impl Peer {
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let value = self.state.borrow().clone();
                if predicate(&value) {
                    return value;
                }
                self.state
                    .changed()
                    .await
                    .expect("Client exited before acceptance");
            }
        })
        .await
        .expect("Acceptance timeout")
    }
    async fn finish(self) {
        self.stop.send_replace(true);
        self.worker.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn bridge_two_clients_acknowledgements_leave_and_fresh_ticket_reconnect() {
    let server = Authority::start();
    let mut a = server.peer(1, GRAYBOX_ID);
    let mut b = server.peer(2, GRAYBOX_ID);
    let initial = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    b.until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    assert_ne!(initial.id, b.state.borrow().id);
    let id = initial.id.unwrap();
    let start = initial
        .snapshot
        .unwrap()
        .players
        .into_iter()
        .find(|p| p.id == id)
        .unwrap()
        .xz;
    a.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    let moved = b
        .until(|s| {
            s.snapshot.as_ref().is_some_and(|s| {
                s.players
                    .iter()
                    .any(|p| p.id == id && p.xz[1] < start[1] - 1.)
            })
        })
        .await;
    assert!(
        moved
            .snapshot
            .unwrap()
            .players
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .applied_sequence
            > 0
    );
    a.input.send_replace(Intent::default());
    a.finish().await;
    b.until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
        .await;
    let mut reconnected = server.peer(3, GRAYBOX_ID);
    let state = reconnected
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    assert!(state.id.unwrap() > id);
    // Reuse of the first ticket cannot create a second application session.
    let replay = server.peer(1, GRAYBOX_ID);
    assert!(
        tokio::time::timeout(Duration::from_secs(8), replay.worker)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    reconnected.finish().await;
    b.finish().await;
    server.finish().await;
}

#[tokio::test]
async fn world_mismatch_and_invalid_intent_fail_closed() {
    let server = Authority::start();
    let wrong_world = server.peer(1, [99; 32]);
    let error = wrong_world.worker.await.unwrap().unwrap_err().to_string();
    assert!(error.contains("world mismatch"), "{error}");
    let mut peer = server.peer(2, GRAYBOX_ID);
    peer.until(|s| s.id.is_some()).await;
    peer.input.send_replace(Intent {
        jump: false,
        axes: [1001, 0],
        sprint: false,
    });
    assert!(
        peer.worker
            .await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("Invalid local intent")
    );
    server.finish().await;
}

#[tokio::test]
async fn local_ticket_validation_permissions_and_exclusive_creation() {
    let server = Authority::start();
    let path = server.directory.join("client-1.token");
    let bytes = std::fs::read(&path).unwrap();
    assert!(runtime::read_ticket(&bytes, server.address).is_ok());
    assert!(runtime::read_ticket(&bytes[..bytes.len() - 1], server.address).is_err());
    assert!(runtime::read_ticket(&bytes, "127.0.0.1:1".parse().unwrap()).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(runtime::read_ticket(&trailing, server.address).is_err());
    let mut token = runtime::read_ticket(&bytes, server.address).unwrap();
    token.expire_timestamp = 0;
    let mut expired = Vec::new();
    token.write(&mut expired).unwrap();
    assert!(runtime::read_ticket(&expired, server.address).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    assert!(runtime::issue_local(socket, &server.directory).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    server.finish().await;
}

#[tokio::test]
async fn distant_worlds_rebase_live_replication_and_project_continuous_local_motion() {
    use theia_core::coordinates::GlobalPosition;
    for sign in [-1., 1.] {
        let origin = [sign * 1e9 + 0.01, sign * 1e9 + 0.02, sign * 1e9 - 0.03];
        let mut world = World::default();
        assert!(world.set_origin(GlobalPosition::new(origin).unwrap()));
        assert!(world.set_replication_span(2.));
        let server = Authority::start_world(world);
        let mut a = server.peer(1, GRAYBOX_ID);
        let mut observer = server.peer(2, GRAYBOX_ID);
        let initial = a
            .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
            .await;
        assert_eq!(initial.welcome.as_ref().unwrap().origin, origin);
        a.input.send_replace(Intent {
            jump: false,
            axes: [0, -1000],
            sprint: false,
        });
        let state = a.until(|s| s.wire_rebases >= 2).await;
        let local = state.local_view().unwrap();
        let local = local.snapshot.unwrap();
        let own = local
            .players
            .iter()
            .find(|p| Some(p.id) == state.id)
            .unwrap();
        assert!(own.xz[1] < -3.9 && own.xz[1] > -9.);
        let raw = state.snapshot.as_ref().unwrap();
        let actor = raw.players.iter().find(|p| Some(p.id) == state.id).unwrap();
        let global = raw.global_position(actor).unwrap().metres();
        for (axis, expected) in [own.xz[0], own.y, own.xz[1]].into_iter().enumerate() {
            assert!((global[axis] - origin[axis] - f64::from(expected)).abs() < 0.000002);
        }
        let mut previous = local;
        for _ in 0..12 {
            a.state.changed().await.unwrap();
            let current = a.state.borrow().local_view().unwrap().snapshot.unwrap();
            let before = previous.players.iter().find(|p| p.id == actor.id).unwrap();
            let after = current.players.iter().find(|p| p.id == actor.id).unwrap();
            let allowance = (current.tick - previous.tick) as f32 * 5. / 30. + 0.00002;
            assert!((after.xz[1] - before.xz[1]).abs() <= allowance);
            previous = current;
        }
        observer.until(|s| s.wire_rebases > 0).await;
        a.finish().await;
        observer.finish().await;
        server.finish().await;
    }
}

#[tokio::test]
async fn wrong_epoch_does_not_poison_current_sequence_in_executable_runtime() {
    use theia_network::{
        renet_transport as transport,
        wire::{Input, VERSION, Welcome, encode},
    };
    let server = Authority::start();
    let mut observer = server.peer(1, GRAYBOX_ID);
    observer.until(|s| s.id.is_some()).await;
    let token = runtime::read_ticket(
        &std::fs::read(server.directory.join("client-2.token")).unwrap(),
        server.address,
    )
    .unwrap();
    let mut socket = transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        runtime::now().unwrap(),
        token,
    )
    .unwrap();
    let mut client = renet::RenetClient::new(transport::connection_config());
    let mut welcome = None;
    for _ in 0..200 {
        client.update(Duration::from_millis(10));
        socket
            .update(Duration::from_millis(10), &mut client)
            .unwrap();
        socket.send_packets(&mut client).unwrap();
        if let Some(bytes) = client.receive_message(transport::EVENT_CHANNEL) {
            welcome = Some(Welcome::decode(&bytes, GRAYBOX_ID).unwrap());
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let welcome = welcome.expect("Raw client admission");
    for (epoch, sequence) in [
        (welcome.epoch.wrapping_add(1).max(1), u64::MAX),
        (welcome.epoch, 1),
    ] {
        client.send_message(
            transport::MOVEMENT_CHANNEL,
            encode(&Input {
                version: VERSION,
                epoch,
                sequence,
                intent: Intent::default(),
            })
            .unwrap(),
        );
    }
    socket.send_packets(&mut client).unwrap();
    observer
        .until(|s| {
            s.snapshot.as_ref().is_some_and(|s| {
                s.players
                    .iter()
                    .any(|p| p.id == welcome.id && p.applied_sequence == 1)
            })
        })
        .await;
    socket.disconnect();
    observer.finish().await;
    server.finish().await;
}

#[tokio::test]
async fn authoritative_jump_is_replicated_and_held_input_requires_release_at_all_rates() {
    use theia_core::time::TickRate;
    for hz in [20, 30, 60] {
        let authority = Authority::start_world(World::at_rate(TickRate::from_hz(hz).unwrap()));
        let mut a = authority.peer(1, GRAYBOX_ID);
        let mut b = authority.peer(2, GRAYBOX_ID);
        let initial = a
            .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
            .await;
        let id = initial.id.unwrap();
        let initial_y = initial
            .snapshot
            .unwrap()
            .players
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .y;
        a.input.send_replace(Intent {
            jump: true,
            ..Intent::default()
        });
        b.until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.players.iter().any(|p| p.id == id && p.y > initial_y + 1.))
        })
        .await;
        let landed = b
            .until(|s| {
                s.snapshot.as_ref().is_some_and(|s| {
                    s.players
                        .iter()
                        .any(|p| p.id == id && (p.y - initial_y).abs() < 0.002)
                })
            })
            .await;
        let tick = landed.snapshot.unwrap().tick;
        b.until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.tick > tick + u64::from(hz / 2))
        })
        .await;
        let snapshot = b.state.borrow().snapshot.clone().unwrap();
        let own = snapshot.players.iter().find(|p| p.id == id).unwrap();
        assert!(
            (own.y - initial_y).abs() < 0.002,
            "Held jump repeated at {hz} Hz"
        );
        let sequence = own.applied_sequence;
        a.input.send_replace(Intent::default());
        b.until(|s| {
            s.snapshot.as_ref().is_some_and(|s| {
                s.players
                    .iter()
                    .any(|p| p.id == id && p.applied_sequence > sequence + 1)
            })
        })
        .await;
        a.input.send_replace(Intent {
            jump: true,
            ..Intent::default()
        });
        b.until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.players.iter().any(|p| p.id == id && p.y > initial_y + 1.))
        })
        .await;
        a.finish().await;
        b.finish().await;
        authority.finish().await;
    }
}

#[tokio::test]
async fn real_chunk_authority_loads_off_tick_and_publishes_against_two_live_peers() {
    let fixture = chunk_fixture::Fixture::new(7_812_500, false, false);
    let world = fixture.world(&[-1, 0], 126.);
    let identity = world.identity();
    let frame = world.chunks().unwrap().frame();
    let (control, requests) = runtime::chunk_control();
    let server = Authority::start_managed(world, Some(requests));
    let mut a = server.peer(1, identity);
    let mut b = server.peer(2, identity);
    let initial = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2))
        .await;
    let id = initial.id.unwrap();
    a.chunks
        .send_replace(Some(vec![fixture.id(-1), fixture.id(0)]));
    b.chunks
        .send_replace(Some(vec![fixture.id(-1), fixture.id(0)]));

    let position = |state: &State| {
        let snapshot = state.snapshot.as_ref()?;
        let player = snapshot.players.iter().find(|p| p.id == id)?;
        frame
            .to_local(snapshot.global_position(player)?)
            .map(|p| p.metres())
    };
    a.input.send_replace(Intent {
        axes: [1000, 0],
        jump: false,
        sprint: false,
    });
    a.until(|s| position(s).is_some_and(|p| p[0] > 127.4)).await;
    let at_edge = a
        .until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.tick > initial.snapshot.as_ref().unwrap().tick + 30)
        })
        .await;
    assert!(position(&at_edge).unwrap()[0] < 127.646);
    let sequence = at_edge
        .snapshot
        .as_ref()
        .unwrap()
        .players
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .applied_sequence;
    let tick = at_edge.snapshot.as_ref().unwrap().tick;
    a.input.send_replace(Intent {
        axes: [1000, 0],
        jump: true,
        sprint: false,
    });
    a.until(|s| position(s).is_some_and(|p| p[1] > 1.3)).await;
    // The actual runtime requests a plan, loads on spawn_blocking and validates
    // publication while both authenticated UDP clients remain connected.
    assert_eq!(
        control
            .replace(&[fixture.id(0), fixture.id(1)])
            .await
            .unwrap(),
        2
    );
    // Concurrent management requests cannot start a second load or silently
    // queue unbounded work. The current set is a valid no-op candidate.
    let ids = [fixture.id(0), fixture.id(1)];
    let (first, second) = tokio::join!(control.replace(&ids), control.replace(&ids));
    assert_eq!(first.unwrap(), 2);
    assert!(
        second
            .unwrap_err()
            .to_string()
            .contains("already in flight")
    );
    // Server collision is loaded, but this client's renderer has not reported
    // the new neighbor yet. Simulation/acks and gravity must keep advancing.
    b.input.send_replace(Intent {
        axes: [1000, 0],
        jump: false,
        sprint: false,
    });
    let held_tick = a.state.borrow().snapshot.as_ref().unwrap().tick;
    let held = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.tick > held_tick + 10))
        .await;
    assert!(position(&held).unwrap()[0] < 127.646);
    a.chunks
        .send_replace(Some(vec![fixture.id(0), fixture.id(1)]));
    let moved = b.until(|s| position(s).is_some_and(|p| p[0] > 129.)).await;
    let snapshot = moved.snapshot.as_ref().unwrap();
    let player = snapshot.players.iter().find(|p| p.id == id).unwrap();
    assert!(player.applied_sequence > sequence && snapshot.tick > tick);
    assert_eq!(snapshot.players.len(), 2);
    let other = b.state.borrow().id.unwrap();
    let other_pose = |s: &State| {
        let snap = s.snapshot.as_ref()?;
        let p = snap.players.iter().find(|p| p.id == other)?;
        frame.to_local(snap.global_position(p)?).map(|p| p.metres())
    };
    assert!(
        other_pose(&moved).unwrap()[0] < 127.646,
        "One ready client must not release another"
    );
    b.chunks
        .send_replace(Some(vec![fixture.id(0), fixture.id(1)]));
    b.until(|s| other_pose(s).is_some_and(|p| p[0] > 129.))
        .await;
    a.input.send_replace(Intent::default());
    b.input.send_replace(Intent::default());
    // Even with no actor standing at the initial location, recovery/fresh-join
    // support must remain resident. A rejection keeps the session alive.
    assert!(
        control
            .replace(&[fixture.id(1)])
            .await
            .unwrap_err()
            .to_string()
            .contains("spawn")
    );
    std::fs::write(
        fixture
            .root
            .join(format!("chunk_{}_{}", fixture.id(-1).x, fixture.id(-1).z))
            .join("terrain.json"),
        b"corrupt",
    )
    .unwrap();
    assert!(
        control
            .replace(&[fixture.id(-1), fixture.id(0)])
            .await
            .is_err()
    );
    let after = a
        .until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.tick > snapshot.tick + 8)
        })
        .await;
    assert_eq!(after.id, Some(id));
    a.finish().await;
    b.finish().await;
    let stats = server.finish().await;
    assert_eq!(stats.chunk_generation, 2);
    assert_eq!(stats.chunk_rejections, 1);
    assert!(stats.accepted_chunk_ready >= 4);
    assert!(stats.chunk_unready_steps > 0);
    assert!(stats.max_control_batch <= 8);
    assert!(stats.chunk_blocked_steps > 0);
    assert_eq!(stats.movement_recoveries, 0);
    println!(
        "chunk_authority_summary {}",
        serde_json::to_string(&stats).unwrap()
    );
}

#[tokio::test]
async fn acknowledged_chunk_withdrawal_holds_only_its_actor_and_regrant_needs_a_new_ack() {
    let fixture = chunk_fixture::Fixture::new(0, false, false);
    let world = fixture.world(&[-1, 0], 16.);
    let identity = world.identity();
    let frame = world.chunks().unwrap().frame();
    let server = Authority::start_world(world);
    let mut a = server.peer(1, identity);
    let mut b = server.peer(2, identity);
    let ids = vec![fixture.id(-1), fixture.id(0)];
    a.chunks.send_replace(Some(ids.clone()));
    b.chunks.send_replace(Some(ids.clone()));
    let admitted = a.until(|s| s.chunk_permission_confirmed(&ids)).await;
    b.until(|s| s.chunk_permission_confirmed(&ids)).await;
    let id = admitted.id.unwrap();
    let other = b.state.borrow().id.unwrap();
    let position = |state: &State, actor| {
        let snapshot = state.snapshot.as_ref().unwrap();
        let player = snapshot.players.iter().find(|p| p.id == actor).unwrap();
        frame
            .to_local(snapshot.global_position(player).unwrap())
            .unwrap()
            .metres()
    };
    for peer in [&a, &b] {
        peer.input.send_replace(Intent {
            axes: [1000, 0],
            jump: false,
            sprint: false,
        });
    }
    a.until(|s| s.snapshot.is_some() && position(s, id)[0] > 17.)
        .await;
    a.chunks.send_replace(Some(vec![]));
    let withdrawn = a.until(|s| s.chunk_permission_confirmed(&[])).await;
    assert_eq!(withdrawn.chunk_ack.as_ref().unwrap().sequence, 2);
    let tick = withdrawn.snapshot.as_ref().unwrap().tick;
    let held = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.tick > tick + 12))
        .await;
    assert!((position(&held, id)[0] - position(&withdrawn, id)[0]).abs() < 0.00001);
    assert!(position(&held, other)[0] > position(&withdrawn, other)[0] + 1.5);
    let applied = |s: &State| {
        s.snapshot
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .applied_sequence
    };
    assert!(
        applied(&held) > applied(&withdrawn),
        "Withdrawal must not stop input acknowledgements"
    );
    a.chunks.send_replace(Some(ids.clone()));
    let regranted = a
        .until(|s| {
            s.chunk_permission_confirmed(&ids) && position(s, id)[0] > position(&held, id)[0] + 0.5
        })
        .await;
    assert_eq!(regranted.chunk_ack.as_ref().unwrap().sequence, 3);
    assert!(regranted.chunk_ack.as_ref().unwrap().tick > tick);
    assert!(
        regranted
            .transport
            .as_ref()
            .unwrap()
            .chunk_ready_acks_received
            >= 3
    );
    a.finish().await;
    b.finish().await;
    let stats = server.finish().await;
    assert_eq!(stats.accepted_chunk_ready, 4);
    assert_eq!(stats.chunk_ready_acks_sent, 4);
    assert_eq!(stats.movement_recoveries, 0);
}

#[tokio::test]
async fn automatic_chunk_lookahead_offers_real_publication_but_waits_for_each_peer_readiness() {
    let f = chunk_fixture::Fixture::new(0, false, false);
    let world = f.world(&[-1, 0], 126.);
    let identity = world.identity();
    let server = Authority::start_policy(world, None, true);
    let mut a = server.peer(1, identity);
    let b = server.peer(2, identity);
    a.chunks.send_replace(Some(vec![f.id(-1), f.id(0)]));
    b.chunks.send_replace(Some(vec![f.id(-1), f.id(0)]));
    let initial = a
        .until(|s| s.chunk_permission_confirmed(&[f.id(-1), f.id(0)]))
        .await;
    assert_eq!(initial.chunk_offer.as_ref().unwrap().generation, 1);
    let id = initial.id.unwrap();
    // A peer first moves within the current cell. This must not retire a spare
    // package and force a delayed client to prepare an unnecessary generation.
    b.input.send_replace(Intent {
        axes: [0, 1000],
        ..Intent::default()
    });
    let initial_tick = initial.snapshot.as_ref().unwrap().tick;
    let resident = a
        .until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|snap| snap.tick > initial_tick + 15)
        })
        .await;
    assert_eq!(resident.chunk_offer.as_ref().unwrap().generation, 1);
    assert_eq!(
        resident.chunk_offer.as_ref().unwrap().chunks,
        vec![[-1, 0], [0, 0]]
    );
    b.input.send_replace(Intent::default());
    a.input.send_replace(Intent {
        axes: [1000, 0],
        jump: false,
        sprint: false,
    });
    let offered = a
        .until(|s| s.chunk_offer.as_ref().is_some_and(|o| o.generation == 2))
        .await;
    assert_eq!(
        offered.chunk_offer.as_ref().unwrap().chunks,
        vec![[0, 0], [1, 0]]
    );
    let tick = offered.snapshot.as_ref().unwrap().tick;
    let held = a
        .until(|s| s.snapshot.as_ref().is_some_and(|s| s.tick > tick + 15))
        .await;
    assert!(
        held.snapshot
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.id == id)
            .unwrap()
            .xz[0]
            < 127.646
    );
    a.chunks.send_replace(Some(vec![f.id(0), f.id(1)]));
    a.until(|s| {
        s.chunk_permission_confirmed(&[f.id(0), f.id(1)])
            && s.snapshot.as_ref().is_some_and(|snapshot| {
                snapshot
                    .players
                    .iter()
                    .any(|p| p.id == id && snapshot.global_position(p).unwrap().metres()[0] > 129.)
            })
    })
    .await;
    a.finish().await;
    b.finish().await;
    let stats = server.finish().await;
    assert_eq!(stats.chunk_generation, 2);
    assert!(stats.chunk_offers_sent >= 4);
    assert_eq!(stats.movement_recoveries, 0);
}

#[tokio::test]
async fn initial_only_chunk_sessions_reject_different_loaded_sets_for_the_same_dataset() {
    let fixture = chunk_fixture::Fixture::new(0, false, false);
    let world = fixture.world(&[-1, 0], 16.);
    let wrong = fixture.world(&[0, 1], 16.).identity();
    assert_ne!(world.identity(), wrong);
    let server = Authority::start_world(world);
    let peer = server.peer(1, wrong);
    let error = tokio::time::timeout(Duration::from_secs(8), peer.worker)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err()
        .to_string();
    assert!(error.contains("world mismatch"), "{error}");
    server.finish().await;
}

#[tokio::test]
async fn readiness_replay_unknown_ids_and_malformed_actor_addressing_are_rejected_over_real_udp() {
    use theia_network::{
        renet_transport as transport,
        wire::{ChunkEvent, ChunkReady, Input, VERSION, Welcome, encode},
    };
    let fixture = chunk_fixture::Fixture::new(0, false, false);
    let world = fixture.world(&[0, 1], 126.);
    let identity = world.identity();
    let server = Authority::start_world(world);
    let mut observer = server.peer(1, identity);
    observer.until(|s| s.id.is_some()).await;
    let token = runtime::read_ticket(
        &std::fs::read(server.directory.join("client-2.token")).unwrap(),
        server.address,
    )
    .unwrap();
    let mut socket = transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        runtime::now().unwrap(),
        token,
    )
    .unwrap();
    let mut client = renet::RenetClient::new(transport::connection_config());
    let mut welcome = None;
    for _ in 0..200 {
        client.update(Duration::from_millis(10));
        socket
            .update(Duration::from_millis(10), &mut client)
            .unwrap();
        socket.send_packets(&mut client).unwrap();
        if let Some(bytes) = client.receive_message(transport::EVENT_CHANNEL) {
            welcome = Some(Welcome::decode(&bytes, identity).unwrap());
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let welcome = welcome.expect("Raw chunk client admission");
    let report = |epoch, sequence, chunks| ChunkReady {
        version: VERSION,
        epoch,
        sequence,
        chunks,
    };
    for message in [
        report(
            welcome.epoch.wrapping_add(1).max(1),
            u64::MAX,
            vec![[0, 0], [1, 0]],
        ),
        report(welcome.epoch, u64::MAX, vec![[99, 0]]),
    ] {
        client.send_message(transport::EVENT_CHANNEL, encode(&message).unwrap());
    }
    for sequence in 1..=13 {
        client.send_message(
            transport::EVENT_CHANNEL,
            encode(&report(welcome.epoch, sequence, vec![[0, 0], [1, 0]])).unwrap(),
        );
    }
    client.send_message(
        transport::EVENT_CHANNEL,
        encode(&report(welcome.epoch, 1, vec![])).unwrap(),
    );
    client.send_message(
        transport::MOVEMENT_CHANNEL,
        encode(&Input {
            version: VERSION,
            epoch: welcome.epoch,
            sequence: 1,
            intent: Intent {
                axes: [1000, 0],
                jump: false,
                sprint: false,
            },
        })
        .unwrap(),
    );
    socket.send_packets(&mut client).unwrap();
    observer
        .until(|s| {
            s.snapshot.as_ref().is_some_and(|s| {
                s.players
                    .iter()
                    .any(|p| p.id == welcome.id && p.applied_sequence == 1 && p.xz[0] > 126.3)
            })
        })
        .await;
    let mut acknowledged = None;
    for _ in 0..200 {
        client.update(Duration::from_millis(10));
        socket
            .update(Duration::from_millis(10), &mut client)
            .unwrap();
        socket.send_packets(&mut client).unwrap();
        while let Some(bytes) = client.receive_message(transport::EVENT_CHANNEL) {
            let ChunkEvent::Ack(ack) = ChunkEvent::decode(&bytes).unwrap() else {
                continue;
            };
            assert_eq!(ack.epoch, welcome.epoch);
            assert!(ack.sequence <= 13);
            if ack.sequence == 13 {
                acknowledged = Some(ack);
            }
        }
        if acknowledged.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let acknowledged = acknowledged.expect("Latest valid burst report must be acknowledged");
    assert!(acknowledged.matches(&report(welcome.epoch, 13, vec![[0, 0], [1, 0]])));
    observer
        .until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.tick >= acknowledged.tick)
        })
        .await;
    let mut malformed =
        serde_json::to_value(report(welcome.epoch, 14, vec![[0, 0], [1, 0]])).unwrap();
    malformed["actor"] = serde_json::json!(observer.state.borrow().id.unwrap());
    client.send_message(
        transport::EVENT_CHANNEL,
        serde_json::to_vec(&malformed).unwrap(),
    );
    // Pump the raw authenticated peer so reliable delivery/acknowledgements are
    // real, rather than depending on one unacknowledged datagram.
    for _ in 0..50 {
        client.update(Duration::from_millis(10));
        let _ = socket.update(Duration::from_millis(10), &mut client);
        let _ = socket.send_packets(&mut client);
        if observer
            .state
            .borrow()
            .snapshot
            .as_ref()
            .is_some_and(|s| s.players.iter().all(|p| p.id != welcome.id))
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    observer
        .until(|s| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.players.iter().all(|p| p.id != welcome.id))
        })
        .await;
    observer.finish().await;
    let stats = server.finish().await;
    assert_eq!(stats.accepted_chunk_ready, 13);
    assert_eq!(stats.rejected_chunk_ready, 4);
    assert!((1..=13).contains(&stats.chunk_ready_acks_sent));
    assert!((1..=8).contains(&stats.max_control_batch));
    assert_eq!(stats.movement_recoveries, 0);
    println!(
        "readiness_replay_summary {}",
        serde_json::to_string(&stats).unwrap()
    );
}

#[path = "support/wide_chunk_fixture.rs"]
mod wide_fixture;
#[tokio::test]
async fn full_capacity_offer_ready_ack_and_withdrawal_use_real_encrypted_transport() {
    use theia_core::coordinates::GlobalPosition;
    use theia_network::{
        chunks::{Index, MAX_RESIDENT, Residency},
        wire::VERSION,
    };
    let fixture = wide_fixture::WideFixture::new(-7_812_500);
    let ids = fixture.ids[..MAX_RESIDENT].to_vec();
    let mut residency = Residency::new(Index::load(&fixture.root).unwrap(), MAX_RESIDENT).unwrap();
    residency.replace(&ids, &[]).unwrap();
    let start = GlobalPosition::new([-1e9 + 16., 0., -1e9 + 16.]).unwrap();
    let world = World::with_residency(residency, &[start], -18.).unwrap();
    let identity = world.identity();
    let server = Authority::start_world(world);
    let mut peer = server.peer(1, identity);
    peer.chunks.send_replace(Some(ids.clone()));
    let admitted = peer
        .until(|state| state.chunk_permission_confirmed(&ids))
        .await;
    let offer = admitted.chunk_offer.as_ref().unwrap();
    assert_eq!(offer.version, VERSION);
    assert_eq!(offer.chunks.len(), MAX_RESIDENT);
    assert_eq!(
        admitted.chunk_ack.as_ref().unwrap().chunks.len(),
        MAX_RESIDENT
    );
    let actor = admitted.id.unwrap();
    peer.input.send_replace(Intent {
        axes: [1000, 0],
        jump: false,
        sprint: false,
    });
    peer.until(|state| {
        state.snapshot.as_ref().is_some_and(|s| {
            s.players
                .iter()
                .find(|p| p.id == actor)
                .and_then(|p| s.global_position(p))
                .is_some_and(|p| p.metres()[0] > start.metres()[0] + 1.)
        })
    })
    .await;
    peer.chunks.send_replace(Some(vec![]));
    peer.until(|state| state.chunk_permission_confirmed(&[]))
        .await;
    peer.chunks.send_replace(Some(ids.clone()));
    peer.until(|state| state.chunk_permission_confirmed(&ids))
        .await;
    peer.finish().await;
    let stats = server.finish().await;
    assert_eq!(stats.movement_recoveries, 0);
    assert!(stats.chunk_offers_sent > 0);
    println!(
        "full_capacity_secure_udp_pass {}",
        serde_json::to_string(&stats).unwrap()
    );
}
