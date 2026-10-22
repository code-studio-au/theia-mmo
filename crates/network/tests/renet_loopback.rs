//! Real encrypted UDP proof for the target transport; no mocked connections.
//! The authority steps in simulated time here; this is not a scheduler benchmark.
use renet::{RenetClient, RenetServer, ServerEvent};
use renet_netcode::{
    ConnectToken, NetcodeClientTransport, NetcodeServerTransport, generate_random_bytes,
};
use std::{
    collections::BTreeMap,
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
};
use theia_network::{
    renet_transport::{self as transport, EVENT_CHANNEL, MOVEMENT_CHANNEL, PROTOCOL_ID},
    simulation::World,
    wire::{Input, Intent, Snapshot, VERSION, encode},
};

struct Peer {
    client: RenetClient,
    transport: NetcodeClientTransport,
    snapshot: Option<Snapshot>,
}
fn peer(address: SocketAddr, key: &[u8; 32], id: u64) -> Peer {
    let now = Duration::from_secs(1000);
    let token =
        ConnectToken::generate(now, PROTOCOL_ID, 30, id, 5, vec![address], None, key).unwrap();
    Peer {
        client: RenetClient::new(transport::connection_config()),
        transport: transport::secure_client(UdpSocket::bind("127.0.0.1:0").unwrap(), now, token)
            .unwrap(),
        snapshot: None,
    }
}

struct Authority {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: World,
    players: BTreeMap<u64, u64>,
}
impl Authority {
    fn pump(&mut self, peers: &mut [&mut Peer]) {
        let delta = Duration::from_secs_f64(1. / 30.);
        for peer in peers.iter_mut() {
            peer.client.update(delta);
            peer.transport.update(delta, &mut peer.client).unwrap();
            peer.transport.send_packets(&mut peer.client).unwrap();
        }
        self.server.update(delta);
        self.transport.update(delta, &mut self.server).unwrap();
        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    self.players.insert(client_id, self.world.join().unwrap());
                    self.server
                        .send_message(client_id, EVENT_CHANNEL, b"welcome".to_vec());
                }
                ServerEvent::ClientDisconnected { client_id, .. } => {
                    if let Some(id) = self.players.remove(&client_id) {
                        self.world.leave(id);
                    }
                }
            }
        }
        for client_id in self.server.clients_id() {
            while let Some(bytes) = self.server.receive_message(client_id, MOVEMENT_CHANNEL) {
                match Input::decode(&bytes) {
                    Ok(input) => {
                        self.world.input(self.players[&client_id], input);
                    }
                    Err(_) => self.server.disconnect(client_id),
                }
            }
        }
        self.server
            .broadcast_message(MOVEMENT_CHANNEL, encode(&self.world.step()).unwrap());
        self.transport.send_packets(&mut self.server);
        for peer in peers.iter_mut() {
            peer.transport
                .update(Duration::ZERO, &mut peer.client)
                .unwrap();
            while let Some(bytes) = peer.client.receive_message(MOVEMENT_CHANNEL) {
                peer.snapshot = Some(Snapshot::decode(&bytes).unwrap());
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn secure_renet_two_clients_authority_and_disconnect() {
    let key = generate_random_bytes();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let mut authority = Authority {
        server: RenetServer::new(transport::connection_config()),
        transport: transport::secure_server(socket, Duration::from_secs(1000), key).unwrap(),
        world: World::default(),
        players: BTreeMap::new(),
    };
    let mut a = peer(address, &key, 101);
    let mut b = peer(address, &key, 102);
    for _ in 0..200 {
        authority.pump(&mut [&mut a, &mut b]);
        if a.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
            && b.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        {
            break;
        }
    }
    assert_eq!(authority.players.len(), 2);
    assert_eq!(a.snapshot.as_ref().unwrap().players.len(), 2);
    assert_eq!(b.snapshot.as_ref().unwrap().players.len(), 2);
    let a_id = authority.players[&101];
    assert_ne!(a_id, authority.players[&102]);
    assert_eq!(
        a.client.receive_message(EVENT_CHANNEL).unwrap().as_ref(),
        b"welcome"
    );
    for sequence in 1..=40 {
        a.client.send_message(
            MOVEMENT_CHANNEL,
            encode(&Input {
                version: VERSION,
                epoch: authority.world.epoch(),
                sequence,
                intent: Intent {
                    jump: false,
                    axes: [0, -1000],
                    sprint: false,
                },
            })
            .unwrap(),
        );
        authority.pump(&mut [&mut a, &mut b]);
    }
    let snapshot = b.snapshot.as_ref().unwrap();
    let moved = snapshot.players.iter().find(|p| p.id == a_id).unwrap();
    assert!(moved.xz[1] < -3.);
    assert!(moved.applied_sequence > 0);
    a.transport.disconnect();
    for _ in 0..100 {
        authority.pump(&mut [&mut b]);
        if b.snapshot.as_ref().is_some_and(|s| s.players.len() == 1) {
            break;
        }
    }
    assert_eq!(b.snapshot.as_ref().unwrap().players.len(), 1);
    assert!(!authority.players.contains_key(&101));
}

#[test]
fn wrong_issuer_cannot_join_and_channel_backlogs_are_bounded() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let mut authority = Authority {
        server: RenetServer::new(transport::connection_config()),
        transport: transport::secure_server(
            socket,
            Duration::from_secs(1000),
            generate_random_bytes(),
        )
        .unwrap(),
        world: World::default(),
        players: BTreeMap::new(),
    };
    let mut attacker = peer(address, &generate_random_bytes(), 999);
    // Before the token's local timeout: rejected handshakes never create actors.
    for _ in 0..90 {
        authority.pump(&mut [&mut attacker]);
    }
    assert!(authority.players.is_empty());
    assert!(!attacker.client.is_connected());
    assert!(
        !attacker
            .client
            .can_send_message(EVENT_CHANNEL, transport::CHANNEL_MEMORY_BYTES + 1)
    );
}
