//! Actual encrypted UDP connections and a real server tick, no mocked transport.
use std::{net::SocketAddr, time::Duration};
use theia_network::{
    client::{self, State},
    server, tls,
    wire::{Intent, Snapshot},
};
use tokio::{sync::watch, task::JoinHandle, time::timeout};

struct Client {
    input: watch::Sender<Intent>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    task: JoinHandle<theia_network::Result<()>>,
}
fn connect(address: SocketAddr, certificate: Vec<u8>) -> Client {
    connect_world(address, certificate, theia_network::terrain::GRAYBOX_ID)
}
fn connect_world(address: SocketAddr, certificate: Vec<u8>, identity: [u8; 32]) -> Client {
    let (input, inputs) = watch::channel(Intent::default());
    let (states, state) = watch::channel(State::default());
    let (stop, shutdown) = watch::channel(false);
    let task = tokio::spawn(async move {
        client::run_in_world(address, &certificate, inputs, states, shutdown, identity).await
    });
    Client {
        input,
        state,
        stop,
        task,
    }
}
async fn until(state: &mut watch::Receiver<State>, check: impl Fn(&Snapshot) -> bool) -> Snapshot {
    timeout(Duration::from_secs(5), async {
        loop {
            if let Some(snapshot) = &state.borrow_and_update().snapshot
                && check(snapshot)
            {
                return snapshot.clone();
            }
            state
                .changed()
                .await
                .expect("Client state unexpectedly closed");
        }
    })
    .await
    .expect("Snapshot condition timed out")
}

#[tokio::test]
async fn configured_server_rates_replicate_and_move_over_real_connections() {
    for hz in [20, 60] {
        let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
        let address = endpoint.local_addr().unwrap();
        let world = theia_network::simulation::World::at_rate(
            theia_core::time::TickRate::from_hz(hz).unwrap(),
        );
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(server::run_world(endpoint, shutdown, world));
        let mut client = connect(address, cert);
        until(&mut client.state, |s| {
            s.tick_hz == hz && s.players.len() == 1
        })
        .await;
        client.input.send_replace(Intent {
            jump: false,
            axes: [1000, 0],
            sprint: false,
        });
        let moved = until(&mut client.state, |s| s.players[0].xz[0] > 1.).await;
        assert_eq!(moved.tick_hz, hz);
        assert!(moved.players[0].applied_sequence > 0);
        client.stop.send(true).unwrap();
        client.task.await.unwrap().unwrap();
        stop.send(true).unwrap();
        task.await.unwrap().unwrap();
    }
}

#[tokio::test]
async fn two_clients_authority_disconnect_and_reconnect() {
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run(endpoint, shutdown));
    let mut a = connect(address, cert.clone());
    let mut b = connect(address, cert.clone());
    until(&mut a.state, |s| s.players.len() == 2).await;
    until(&mut b.state, |s| s.players.len() == 2).await;
    let a_id = a.state.borrow().id.unwrap();
    let b_id = b.state.borrow().id.unwrap();
    assert_ne!(a_id, b_id);
    a.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    b.input.send_replace(Intent {
        jump: false,
        axes: [1000, 0],
        sprint: false,
    });
    let snapshot = until(&mut a.state, |s| {
        s.players
            .iter()
            .find(|p| p.id == a_id)
            .is_some_and(|p| p.xz[1] < -2.)
    })
    .await;
    assert!(snapshot.players.iter().find(|p| p.id == b_id).unwrap().xz[0] > 1.);
    until(&mut b.state, |s| {
        s.tick >= snapshot.tick
            && s.players
                .iter()
                .find(|p| p.id == a_id)
                .is_some_and(|p| p.xz[1] < -2.)
    })
    .await;
    b.stop.send(true).unwrap();
    b.task.await.unwrap().unwrap();
    until(&mut a.state, |s| {
        s.players.len() == 1 && s.players[0].id == a_id
    })
    .await;
    let mut rejoined = connect(address, cert.clone());
    until(&mut rejoined.state, |s| s.players.len() == 2).await;
    let new_id = rejoined.state.borrow().id.unwrap();
    assert_ne!(new_id, b_id);
    until(&mut a.state, |s| s.players.iter().any(|p| p.id == new_id)).await;
    // A different otherwise valid local server certificate must not be trusted.
    let (_, wrong_cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let wrong = connect(address, wrong_cert);
    assert!(
        timeout(Duration::from_secs(6), wrong.task)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    a.stop.send(true).unwrap();
    rejoined.stop.send(true).unwrap();
    a.task.await.unwrap().unwrap();
    rejoined.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn malformed_input_closes_the_actual_connection() {
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run(endpoint, shutdown));
    let client = tls::client(address, &cert).unwrap();
    let conn = client.connect(address, "localhost").unwrap().await.unwrap();
    let welcome = conn
        .accept_uni()
        .await
        .unwrap()
        .read_to_end(theia_network::wire::WELCOME_BYTES)
        .await
        .unwrap();
    assert_eq!(welcome.len(), theia_network::wire::WELCOME_BYTES);
    conn.send_datagram(b"{malformed}".as_slice().into())
        .unwrap();
    let error = timeout(Duration::from_secs(3), conn.closed())
        .await
        .unwrap();
    assert!(matches!(
        error,
        quinn::ConnectionError::ApplicationClosed(_)
    ));
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn reordered_inputs_do_not_reverse_or_refresh_a_stale_intent() {
    use theia_network::wire::{Input, VERSION, encode};
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run(endpoint, shutdown));
    let mut observer = connect(address, cert.clone());
    let endpoint = tls::client(address, &cert).unwrap();
    let connection = timeout(
        Duration::from_secs(5),
        endpoint.connect(address, "localhost").unwrap(),
    )
    .await
    .unwrap()
    .unwrap();
    let welcome = timeout(Duration::from_secs(5), async {
        connection
            .accept_uni()
            .await
            .unwrap()
            .read_to_end(theia_network::wire::WELCOME_BYTES)
            .await
            .unwrap()
    })
    .await
    .unwrap();
    let id = u64::from_le_bytes(welcome[4..12].try_into().unwrap());
    let initial = until(&mut observer.state, |s| s.players.len() == 2).await;
    let epoch = u64::from_le_bytes(welcome[12..20].try_into().unwrap());
    connection
        .send_datagram(
            encode(&Input {
                version: VERSION,
                epoch: epoch.wrapping_add(1).max(1),
                sequence: u64::MAX,
                intent: Intent {
                    jump: false,
                    axes: [1000, 0],
                    sprint: true,
                },
            })
            .unwrap()
            .into(),
        )
        .unwrap();
    connection
        .send_datagram(
            encode(&Input {
                version: VERSION,
                epoch: u64::from_le_bytes(welcome[12..20].try_into().unwrap()),
                sequence: 50,
                intent: Intent {
                    jump: false,
                    axes: [0, 1000],
                    sprint: false,
                },
            })
            .unwrap()
            .into(),
        )
        .unwrap();
    until(&mut observer.state, |s| {
        s.players
            .iter()
            .any(|p| p.id == id && p.applied_sequence == 50)
    })
    .await;
    for sequence in [49, 50, 1] {
        connection
            .send_datagram(
                encode(&Input {
                    version: VERSION,
                    epoch: u64::from_le_bytes(welcome[12..20].try_into().unwrap()),
                    sequence,
                    intent: Intent {
                        jump: false,
                        axes: [0, -1000],
                        sprint: true,
                    },
                })
                .unwrap()
                .into(),
            )
            .unwrap();
    }
    let snapshot = until(&mut observer.state, |s| s.tick > initial.tick + 20).await;
    let stopped = snapshot
        .players
        .iter()
        .find(|p| p.id == id)
        .unwrap()
        .clone();
    assert_eq!(stopped.applied_sequence, 50);
    assert!(stopped.xz[1] > 0. && stopped.xz[1] <= 1.8);
    let later = until(&mut observer.state, |s| s.tick > snapshot.tick + 5).await;
    assert_eq!(
        later.players.iter().find(|p| p.id == id).unwrap().xz,
        stopped.xz
    );
    connection.close(0u32.into(), b"test complete");
    observer.stop.send(true).unwrap();
    observer.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn authoritative_world_stops_network_movement_at_the_fixed_door() {
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run(endpoint, shutdown));
    let mut client = connect(address, cert);
    until(&mut client.state, |s| s.players.len() == 1).await;
    client.input.send_replace(Intent {
        jump: false,
        axes: [-1000, 0],
        sprint: false,
    });
    until(&mut client.state, |s| s.players[0].xz[0] <= -6.6).await;
    client.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    let contact = until(&mut client.state, |s| s.players[0].xz[1] <= -2.53).await;
    let door = theia_network::world::PROPS[4];
    let surface = door.center[2] + door.size[2] / 2.;
    let clearance = contact.players[0].xz[1] - surface - theia_network::world::PAWN_RADIUS;
    assert!(
        (0.004..0.008).contains(&clearance),
        "Capsule clearance: {clearance}"
    );
    let later = until(&mut client.state, |s| s.tick > contact.tick + 12).await;
    assert_eq!(later.players[0].xz, contact.players[0].xz);
    assert!(later.players[0].applied_sequence > contact.players[0].applied_sequence);
    client.stop.send(true).unwrap();
    client.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn terrain_height_is_authoritative_and_mismatched_world_is_rejected() {
    use theia_network::{simulation::World, terrain::Terrain};
    let a = [-24., -7.2, -24.];
    let b = [24., 7.2, -24.];
    let c = [24., 7.2, 24.];
    let d = [-24., -7.2, 24.];
    let data=serde_json::to_vec(&serde_json::json!({"schema":1,"source_origin":[180,34,2400],"visual_sha256":vec![0;32],"triangles":[[a,c,b],[a,d,c]]})).unwrap();
    let terrain = Terrain::decode(&data).unwrap();
    let identity = terrain.identity;
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run_world(
        endpoint,
        shutdown,
        World::with_terrain(terrain),
    ));
    let wrong = connect(address, cert.clone());
    let error = timeout(Duration::from_secs(6), wrong.task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.to_string().contains("world mismatch"));
    let (input, inputs) = watch::channel(Intent::default());
    let (states, mut state) = watch::channel(State::default());
    let (client_stop, shutdown) = watch::channel(false);
    let client = tokio::spawn(async move {
        client::run_in_world(address, &cert, inputs, states, shutdown, identity).await
    });
    until(&mut state, |s| s.players.len() == 1).await;
    let id = state.borrow().id.unwrap();
    let initial = state.borrow().snapshot.as_ref().unwrap().players[0].clone();
    input.send_replace(Intent {
        jump: false,
        axes: [1000, 0],
        sprint: false,
    });
    let later = until(&mut state, |s| {
        s.players
            .iter()
            .any(|p| p.id == id && p.xz[0] > initial.xz[0] + 1.)
    })
    .await;
    let player = later.players.iter().find(|p| p.id == id).unwrap();
    let clearance = (player.y - 0.9 - player.xz[0] * 0.3 + 0.35) / 1.09_f32.sqrt() - 0.35;
    assert!(
        (0.006..0.010).contains(&clearance),
        "{player:?}; {clearance}"
    );
    assert!(player.y > initial.y + 0.3);
    client_stop.send(true).unwrap();
    client.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn terrain_solid_stops_persistent_network_input_on_uneven_ground() {
    use theia_network::{simulation::World, terrain::Terrain};
    let a = [-24., -2.4, -24.];
    let b = [24., 2.4, -24.];
    let c = [24., 2.4, 24.];
    let d = [-24., -2.4, 24.];
    let bytes = serde_json::to_vec(
        &serde_json::json!({"schema":1,"source_origin":[0,0,0],"visual_sha256":vec![0;32],
        "triangles":[[a,c,b],[a,d,c]],"solids":[{"center":[-8,3,-8],"size":[8.6,10,8.6]}]}),
    )
    .unwrap();
    let terrain = Terrain::decode(&bytes).unwrap();
    let identity = terrain.identity;
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run_world(
        endpoint,
        shutdown,
        World::with_terrain(terrain),
    ));
    let mut client = connect_world(address, cert, identity);
    until(&mut client.state, |s| s.players.len() == 1).await;
    client.input.send_replace(Intent {
        jump: false,
        axes: [-1000, 0],
        sprint: false,
    });
    until(&mut client.state, |s| s.players[0].xz[0] < -7.6).await;
    client.input.send_replace(Intent {
        jump: false,
        axes: [0, -1000],
        sprint: false,
    });
    let contact = until(&mut client.state, |s| s.players[0].xz[1] < -3.34).await;
    assert!((contact.players[0].xz[1] + 3.3445).abs() < 0.001);
    let p = &contact.players[0];
    let clearance = (p.y - 0.9 - p.xz[0] * 0.1 + 0.35) / 1.01_f32.sqrt() - 0.35;
    assert!((0.006..0.010).contains(&clearance), "{p:?}; {clearance}");
    let later = until(&mut client.state, |s| s.tick > contact.tick + 15).await;
    assert_eq!(later.players[0].xz, contact.players[0].xz);
    assert!(later.players[0].applied_sequence > contact.players[0].applied_sequence);
    client.stop.send(true).unwrap();
    client.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn full_tile_coordinates_replicate_beyond_the_old_patch_boundary() {
    use theia_network::{simulation::World, terrain::Terrain};
    let data = serde_json::to_vec(&serde_json::json!({"schema":2,
        "source_origin":[0,0,0], "visual_sha256":vec![0;32],
        "bounds":[[-100,-100],[100,100]], "triangles":[
        [[-100,-10,-100],[100,10,100],[100,10,-100]],
        [[-100,-10,-100],[-100,-10,100],[100,10,100]]]}))
    .unwrap();
    let terrain = Terrain::decode(&data).unwrap();
    let identity = terrain.identity;
    let (endpoint, cert) = tls::server("127.0.0.1:0".parse().unwrap()).unwrap();
    let address = endpoint.local_addr().unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(server::run_world(
        endpoint,
        shutdown,
        World::with_terrain(terrain),
    ));
    let mut a = connect_world(address, cert.clone(), identity);
    let mut b = connect_world(address, cert, identity);
    until(&mut a.state, |s| s.players.len() == 2).await;
    let id = a.state.borrow().id.unwrap();
    a.input.send_replace(Intent {
        jump: false,
        axes: [1000, 0],
        sprint: true,
    });
    let snapshot = until(&mut b.state, |s| {
        s.players.iter().any(|p| p.id == id && p.xz[0] > 22.)
    })
    .await;
    let p = snapshot.players.iter().find(|p| p.id == id).unwrap();
    let clearance = (p.y - 0.9 - p.xz[0] * 0.1 + 0.35) / 1.01_f32.sqrt() - 0.35;
    assert!((0.006..0.010).contains(&clearance), "{p:?}; {clearance}");
    a.stop.send(true).unwrap();
    b.stop.send(true).unwrap();
    a.task.await.unwrap().unwrap();
    b.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}
