use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use theia_core::{
    character_name::CharacterName,
    chat::{Area, Channel, Request as Intent, Text},
    coordinates::GlobalPosition,
};
struct Lease {
    live: Arc<AtomicBool>,
    name: Option<CharacterName>,
}
impl AdmissionLease for Lease {
    fn live(&self) -> Result<bool> {
        Ok(self.live.load(Ordering::Acquire))
    }
    fn record(&self, _: EntityKey, _: GlobalPosition, _: u64) -> Result<()> {
        Ok(())
    }
    fn display_name(&self) -> Result<Option<CharacterName>> {
        Ok(self.name.clone())
    }
}
fn world() -> World {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    world.enable_local_chat(Area::new(1).unwrap()).unwrap();
    world
}
fn bytes(world: &World, sequence: u64) -> Vec<u8> {
    crate::chat_wire::Request {
        epoch: theia_core::identity::WorldEpoch::new(world.epoch()).unwrap(),
        intent: Intent {
            sequence,
            channel: Channel::Say,
            text: Text::parse("Hello nearby").unwrap(),
        },
    }
    .encode()
    .unwrap()
}
#[test]
fn optional_profile_binds_compatibility_and_trusted_lease_revocation_purges_queued_chat() {
    let mut baseline = World::with_procedural([8060., 8000.], 1.).unwrap();
    baseline.enable_prediction().unwrap();
    let original = baseline.identity();
    assert!(Bridge::new(&baseline).unwrap().is_none());
    baseline.enable_local_chat(Area::new(1).unwrap()).unwrap();
    assert_eq!(
        baseline.identity(),
        crate::chat::profile_identity(original, Area::new(1).unwrap())
    );
    assert_ne!(baseline.identity(), original);
    assert!(baseline.enable_local_chat(Area::new(2).unwrap()).is_err());
    let mut world = world();
    let mut server = RenetServer::new(connection_config());
    let live = Arc::new(AtomicBool::new(true));
    server.add_connection(1);
    let peer = admit_peer(
        &mut world,
        &mut server,
        1,
        None,
        Some(Box::new(Lease {
            live: live.clone(),
            name: Some(CharacterName::parse("Arden").unwrap()),
        })),
        Instant::now(),
    )
    .unwrap()
    .unwrap();
    let mut peers = BTreeMap::from([(1, peer)]);
    let mut bridge = Bridge::new(&world).unwrap().unwrap();
    world.step();
    bridge.sync(&peers, &world).unwrap();
    let mut stats = Stats::default();
    bridge
        .request(&peers[&1], &world, &bytes(&world, 1), &mut stats)
        .unwrap();
    assert_eq!(stats.accepted, 1);
    assert_eq!(stats.queued, 1);
    live.store(false, Ordering::Release);
    bridge.sync(&peers, &world).unwrap();
    bridge.flush(&mut server, &peers, &mut stats).unwrap();
    assert_eq!(stats.sent, 0);
    assert!(bridge.actors.is_empty());
    assert!(world.enable_local_chat(Area::new(1).unwrap()).is_err());
    peers.clear();
}

struct Policy {
    used: std::sync::Mutex<[bool; 3]>,
    positions: [GlobalPosition; 3],
}
impl AccountAdmissions for Policy {
    fn consume(self: Arc<Self>, id: u64, data: &[u8; 256]) -> Result<Option<PlayerAdmission>> {
        if !(1..=3).contains(&id) || data[0] != id as u8 {
            return Ok(None);
        }
        let index = id as usize - 1;
        let mut used = self.used.lock().unwrap();
        if used[index] {
            return Ok(None);
        }
        used[index] = true;
        Ok(Some(PlayerAdmission {
            position: self.positions[index],
            lease: Box::new(Lease {
                live: Arc::new(AtomicBool::new(true)),
                name: Some(CharacterName::parse(["Arden", "Briar", "Cedar"][index]).unwrap()),
            }),
        }))
    }
    fn retained_positions(&self) -> Result<[Option<GlobalPosition>; MAX_PLAYERS]> {
        let mut positions = [None; MAX_PLAYERS];
        for (to, from) in positions.iter_mut().zip(self.positions) {
            *to = Some(from);
        }
        Ok(positions)
    }
}
struct Raw {
    client: RenetClient,
    transport: renet_netcode::NetcodeClientTransport,
    last: Instant,
    confirmed: bool,
    welcome: Option<Welcome>,
    messages: Vec<crate::chat_wire::Delivery>,
    snapshots: usize,
    world: [u8; 32],
}
impl Raw {
    fn new(address: SocketAddr, ticket: &[u8], world: [u8; 32]) -> Self {
        Self {
            client: RenetClient::new(connection_config()),
            transport: secure_client(
                UdpSocket::bind("127.0.0.1:0").unwrap(),
                now().unwrap(),
                read_ticket(ticket, address).unwrap(),
            )
            .unwrap(),
            last: Instant::now(),
            confirmed: false,
            welcome: None,
            messages: Vec::new(),
            snapshots: 0,
            world,
        }
    }
    fn poll(&mut self) {
        let current = Instant::now();
        let delta = current.saturating_duration_since(self.last);
        self.last = current;
        self.client.update(delta);
        self.transport.update(delta, &mut self.client).unwrap();
        confirm_connection(&mut self.client, &mut self.confirmed).unwrap();
        while let Some(bytes) = self.client.receive_message(EVENT_CHANNEL) {
            let Some(welcome) = self.welcome.as_ref() else {
                self.welcome = Some(Welcome::decode(&bytes, self.world).unwrap());
                continue;
            };
            if bytes.starts_with(crate::chat_wire::DELIVERY_TAG) {
                let message = crate::chat_wire::Delivery::decode(&bytes).unwrap();
                message
                    .validate_context(
                        theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),
                        self.world,
                    )
                    .unwrap();
                self.messages.push(message);
            } else if bytes.starts_with(crate::player_identity_wire::STATE_TAG) {
                crate::player_identity_wire::State::decode(&bytes).unwrap();
            } else {
                panic!("Unexpected headless chat fixture event tag");
            }
        }
        while let Some(bytes) = self.client.receive_message(MOVEMENT_CHANNEL) {
            if Snapshot::decode(&bytes).is_ok() {
                self.snapshots += 1;
            } else {
                crate::prediction_wire::Corrected::decode(&bytes).unwrap();
            }
        }
        self.transport.send_packets(&mut self.client).unwrap();
    }
    fn send(&mut self, sequence: u64, channel: Channel) {
        let request = crate::chat_wire::Request {
            epoch: theia_core::identity::WorldEpoch::new(self.welcome.as_ref().unwrap().epoch)
                .unwrap(),
            intent: Intent {
                sequence,
                channel,
                text: Text::parse("Hello nearby").unwrap(),
            },
        };
        self.client
            .send_message(EVENT_CHANNEL, request.encode().unwrap());
    }
}
async fn until(peers: &mut [Raw], condition: impl Fn(&[Raw]) -> bool) {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            for peer in &mut *peers {
                peer.poll();
            }
            if condition(peers) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Encrypted chat fixture condition timed out");
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_named_chat_routes_local_and_area_with_replay_and_flood_refusals() {
    let world = world();
    let mut positions = [GlobalPosition::new([8060., 0., 8000.]).unwrap(); 3];
    for (index, x) in [8060., 8068., 8135.].into_iter().enumerate() {
        let patch = world
            .procedural()
            .unwrap()
            .patch(GlobalPosition::new([x, 0., 8000.]).unwrap())
            .unwrap();
        let mut position = patch.try_spawn(x, 8000.).unwrap().metres();
        position[1] += f64::from(crate::world::PAWN_Y);
        positions[index] = GlobalPosition::new(position).unwrap();
    }
    let policy = Arc::new(Policy {
        used: std::sync::Mutex::new([false; 3]),
        positions,
    });
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = secure_issuer(socket, vec![address]).unwrap();
    let mut peers: Vec<_> = (1..=3)
        .map(|id| {
            let mut data = [0; 256];
            data[0] = id as u8;
            let issued = issuer.issue(id, Some(&data), 30).unwrap();
            Raw::new(address, issued.bytes(), identity)
        })
        .collect();
    let (stop, shutdown) = watch::channel(false);
    let stop = Stop(stop);
    let server = tokio::spawn(run_server_with_admissions(
        transport, shutdown, world, policy,
    ));
    until(&mut peers, |p| {
        p.iter().all(|p| p.welcome.is_some() && p.snapshots > 1)
    })
    .await;
    peers[0].send(1, Channel::Say);
    peers[0].send(2, Channel::Area);
    peers[0].send(3, Channel::Say);
    peers[0].send(4, Channel::Say);
    peers[0].send(1, Channel::Say);
    until(&mut peers, |p| {
        p[0].messages.len() == 3 && p[1].messages.len() == 3 && p[2].messages.len() == 1
    })
    .await;
    let until_time = Instant::now() + Duration::from_millis(150);
    while Instant::now() < until_time {
        for peer in &mut peers {
            peer.poll();
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(
        peers.iter().map(|p| p.messages.len()).collect::<Vec<_>>(),
        vec![3, 3, 1]
    );
    assert_eq!(peers[2].messages[0].channel, Channel::Area);
    for peer in &peers {
        for message in &peer.messages {
            assert_eq!(message.name.as_str(), "Arden");
            assert_eq!(
                message.sender.entity.value(),
                peers[0].welcome.as_ref().unwrap().id
            );
        }
    }
    stop.0.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.local_chat.accepted, 3);
    assert_eq!(stats.local_chat.refused, 2);
    assert_eq!(stats.local_chat.queued, 7);
    assert_eq!(stats.local_chat.sent, 7);
    assert_eq!(stats.local_chat.malformed_uploads, 0);
    assert!(stats.simulation.samples > 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_disabled_malformed_and_forged_chat_uploads_fail_closed() {
    for case in 0..3 {
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        world.enable_prediction().unwrap();
        if case != 0 {
            world.enable_local_chat(Area::new(1).unwrap()).unwrap();
        }
        let position = world
            .procedural()
            .unwrap()
            .patch(GlobalPosition::new([8060., 0., 8000.]).unwrap())
            .unwrap()
            .try_spawn(8060., 8000.)
            .unwrap()
            .metres();
        let position = GlobalPosition::new([
            position[0],
            position[1] + f64::from(crate::world::PAWN_Y),
            position[2],
        ])
        .unwrap();
        let identity = world.identity();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let (transport, issuer) = secure_issuer(socket, vec![address]).unwrap();
        let mut data = [0; 256];
        data[0] = 1;
        let ticket = issuer.issue(1, Some(&data), 30).unwrap();
        let mut peers = [Raw::new(address, ticket.bytes(), identity)];
        let (stop, shutdown) = watch::channel(false);
        let stop = Stop(stop);
        let policy = Arc::new(Policy {
            used: std::sync::Mutex::new([false; 3]),
            positions: [position; 3],
        });
        let server = tokio::spawn(run_server_with_admissions(
            transport, shutdown, world, policy,
        ));
        until(&mut peers, |p| p[0].welcome.is_some() && p[0].snapshots > 1).await;
        match case {
            0 => peers[0].send(1, Channel::Say),
            1 => peers[0]
                .client
                .send_message(EVENT_CHANNEL, crate::chat_wire::REQUEST_TAG.as_slice()),
            _ => peers[0]
                .client
                .send_message(EVENT_CHANNEL, crate::chat_wire::DELIVERY_TAG.as_slice()),
        }
        peers[0]
            .transport
            .send_packets(&mut peers[0].client)
            .unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let peer = &mut peers[0];
                let delta = Duration::from_millis(5);
                peer.client.update(delta);
                if peer.transport.update(delta, &mut peer.client).is_err()
                    || peer.client.is_disconnected()
                {
                    break;
                }
                if peer.transport.send_packets(&mut peer.client).is_err() {
                    break;
                }
                tokio::time::sleep(delta).await;
            }
        })
        .await
        .expect("Invalid chat peer was not disconnected");
        stop.0.send_replace(true);
        let stats = server.await.unwrap().unwrap();
        assert_eq!(stats.local_chat.accepted, 0);
        assert_eq!(stats.local_chat.sent, 0);
        match case {
            0 => assert_eq!(stats.local_chat.disabled_uploads, 1),
            1 => assert_eq!(stats.local_chat.malformed_uploads, 1),
            _ => assert_eq!(stats.local_chat.forged_deliveries, 1),
        }
    }
}
