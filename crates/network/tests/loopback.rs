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
    let (input, inputs) = watch::channel(Intent::default());
    let (states, state) = watch::channel(State::default());
    let (stop, shutdown) = watch::channel(false);
    let task =
        tokio::spawn(
            async move { client::run(address, &certificate, inputs, states, shutdown).await },
        );
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
        axes: [0, -1000],
        sprint: false,
    });
    b.input.send_replace(Intent {
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
        .read_to_end(12)
        .await
        .unwrap();
    assert_eq!(welcome.len(), 12);
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
            .read_to_end(12)
            .await
            .unwrap()
    })
    .await
    .unwrap();
    let id = u64::from_le_bytes(welcome[4..].try_into().unwrap());
    let initial = until(&mut observer.state, |s| s.players.len() == 2).await;
    connection
        .send_datagram(
            encode(&Input {
                version: VERSION,
                sequence: 50,
                intent: Intent {
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
                    sequence,
                    intent: Intent {
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
        axes: [-1000, 0],
        sprint: false,
    });
    until(&mut client.state, |s| s.players[0].xz[0] <= -6.6).await;
    client.input.send_replace(Intent {
        axes: [0, -1000],
        sprint: false,
    });
    let contact = until(&mut client.state, |s| s.players[0].xz[1] <= -4.53).await;
    assert!((contact.players[0].xz[1] + 4.54).abs() < 0.0001);
    let later = until(&mut client.state, |s| s.tick > contact.tick + 12).await;
    assert_eq!(later.players[0].xz, contact.players[0].xz);
    assert!(later.players[0].applied_sequence > contact.players[0].applied_sequence);
    client.stop.send(true).unwrap();
    client.task.await.unwrap().unwrap();
    stop.send(true).unwrap();
    server.await.unwrap().unwrap();
}
