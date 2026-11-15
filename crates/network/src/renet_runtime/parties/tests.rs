use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use theia_core::{character_name::CharacterName, coordinates::GlobalPosition, party::Action};

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
    world.enable_parties().unwrap();
    world
}
fn admit(world: &mut World, server: &mut RenetServer, id: u64) -> super::super::Peer {
    server.add_connection(id);
    admit_peer(
        world,
        server,
        id,
        None,
        Some(Box::new(Lease {
            live: Arc::new(AtomicBool::new(true)),
            name: Some(CharacterName::parse(if id == 1 { "Arden" } else { "Briar" }).unwrap()),
        })),
        Instant::now(),
    )
    .unwrap()
    .unwrap()
}
fn bytes(world: &World, actor: u64, action: Action) -> Vec<u8> {
    let view = world.party_view(actor).unwrap();
    party_wire::Request {
        epoch: view.owner.epoch,
        intent: theia_core::party::Request {
            sequence: view.sequence + 1,
            revision: view.revision,
            action,
        },
    }
    .encode()
    .unwrap()
}

#[test]
fn authenticated_ownership_private_results_and_bounded_publication() {
    let mut world = world();
    let mut server = RenetServer::new(connection_config());
    let one = admit(&mut world, &mut server, 1);
    let two = admit(&mut world, &mut server, 2);
    let actor = one.actor;
    let target = world.entity_key(two.actor).unwrap();
    let mut peers = BTreeMap::from([(1, one), (2, two)]);
    let mut stats = Stats::default();
    world.step();
    publish(&mut server, &mut peers, &world, &mut stats).unwrap();
    assert_eq!(stats.states_sent, 2);
    let frame = bytes(&world, actor, Action::Invite(target));
    request(peers.get_mut(&1).unwrap(), &mut world, &frame, &mut stats).unwrap();
    assert_eq!(stats.accepted, 1);
    let incoming = world
        .party_view(target.entity.value())
        .unwrap()
        .membership
        .incoming
        .unwrap();
    let wrong = bytes(&world, actor, Action::Accept(incoming.id));
    request(peers.get_mut(&1).unwrap(), &mut world, &wrong, &mut stats).unwrap();
    assert_eq!(stats.refused, 1);
    publish(&mut server, &mut peers, &world, &mut stats).unwrap();
    assert_eq!(
        peers[&1].party.sent.as_ref().unwrap().refusal,
        Some(Refusal::MissingInvitation)
    );
    assert_eq!(peers[&2].party.sent.as_ref().unwrap().refusal, None);
    assert!(
        peers[&2]
            .party
            .sent
            .as_ref()
            .unwrap()
            .view
            .membership
            .outgoing
            .is_empty()
    );
    // Replays and untrusted owner metadata cannot acknowledge another request.
    assert!(request(peers.get_mut(&1).unwrap(), &mut world, &frame, &mut stats).is_err());
    assert_eq!(stats.invalid_uploads, 1);
    // Fill reliable memory: social publication defers and preserves latest state.
    let before = stats.states_sent;
    for id in [1, 2] {
        while server.can_send_message(id, EVENT_CHANNEL, 1024) {
            server.send_message(id, EVENT_CHANNEL, vec![0; 1024]);
        }
    }
    let accepted = bytes(&world, target.entity.value(), Action::Accept(incoming.id));
    request(
        peers.get_mut(&2).unwrap(),
        &mut world,
        &accepted,
        &mut stats,
    )
    .unwrap();
    publish(&mut server, &mut peers, &world, &mut stats).unwrap();
    assert_eq!(stats.states_sent, before);
    assert_eq!(stats.deferred, 2);
    let mut drained = RenetServer::new(connection_config());
    drained.add_connection(1);
    drained.add_connection(2);
    publish(&mut drained, &mut peers, &world, &mut stats).unwrap();
    assert_eq!(stats.states_sent, before + 2);
    assert_eq!(
        peers[&1]
            .party
            .sent
            .as_ref()
            .unwrap()
            .view
            .membership
            .party
            .as_ref()
            .unwrap()
            .members
            .len(),
        2
    );
}

#[test]
fn unnamed_revoked_and_foreign_requests_never_mutate_membership() {
    let mut world = world();
    let mut server = RenetServer::new(connection_config());
    server.add_connection(9);
    assert!(
        admit_peer(&mut world, &mut server, 9, None, None, Instant::now())
            .unwrap()
            .is_none()
    );
    let mut one = admit(&mut world, &mut server, 1);
    let two = admit(&mut world, &mut server, 2);
    let mut stats = Stats::default();
    let frame = bytes(
        &world,
        one.actor,
        Action::Invite(world.entity_key(two.actor).unwrap()),
    );
    let before = world.party_view(one.actor).unwrap();
    for kind in 0..3 {
        let mut invalid = frame.clone();
        if kind == 0 {
            invalid[4..12].copy_from_slice(&world.epoch().wrapping_add(1).to_le_bytes());
        }
        if kind == 1 {
            invalid.push(0);
        }
        if kind == 2 {
            one.admission = Some(Box::new(Lease {
                live: Arc::new(AtomicBool::new(false)),
                name: one.display_name.clone(),
            }));
        }
        assert!(request(&mut one, &mut world, &invalid, &mut stats).is_err());
        assert_eq!(world.party_view(one.actor).unwrap(), before);
    }
    assert_eq!(stats.invalid_uploads, 3);
}

struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
struct Policy {
    positions: [GlobalPosition; 3],
    used: std::sync::Mutex<[bool; 3]>,
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
struct ClientPeer {
    state: watch::Receiver<State>,
    actions: mpsc::Sender<crate::party_client::Intent>,
    chat: mpsc::Sender<crate::chat_client::Compose>,
    stop: Stop,
    task: tokio::task::JoinHandle<Result<()>>,
    _input: watch::Sender<crate::wire::Intent>,
    _ready: watch::Sender<Option<Vec<theia_core::coordinates::ChunkId>>>,
    _commands: mpsc::Sender<crate::prediction_wire::Commands>,
}
impl ClientPeer {
    fn send(&self, action: Action) {
        let view = self
            .state
            .borrow()
            .party
            .as_ref()
            .unwrap()
            .state
            .view
            .clone();
        self.actions
            .try_send(crate::party_client::Intent {
                owner: view.owner,
                revision: view.revision,
                action,
            })
            .unwrap();
    }
    fn view(&self) -> crate::party_client::View {
        self.state.borrow().party.clone().unwrap()
    }
}
async fn until(peers: &[ClientPeer], condition: impl Fn() -> bool) {
    tokio::time::timeout(Duration::from_secs(8), async {
        while !condition() {
            for peer in peers {
                assert!(
                    !peer.task.is_finished(),
                    "Client worker exited before condition"
                );
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Party worker condition timed out");
}

async fn worker_flow(with_chat: bool) {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    let classic_source = world.identity();
    world.enable_classic().unwrap();
    let chat_source = world.identity();
    let area = theia_core::chat::Area::new(1).unwrap();
    if with_chat {
        world.enable_local_chat(area).unwrap();
    }
    let source = world.identity();
    world.enable_parties().unwrap();
    let identity = world.identity();
    let patch = world
        .procedural()
        .unwrap()
        .patch(GlobalPosition::new([8060., 0., 8000.]).unwrap())
        .unwrap();
    let positions = std::array::from_fn(|index| {
        let p = patch
            .try_spawn(8060. + index as f64 * 3., 8000.)
            .unwrap()
            .metres();
        GlobalPosition::new([p[0], p[1] + f64::from(crate::world::PAWN_Y), p[2]]).unwrap()
    });
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = secure_issuer(socket, vec![address]).unwrap();
    let (server_stop, shutdown) = watch::channel(false);
    let server_stop = Stop(server_stop);
    let server = tokio::spawn(run_server_with_admissions(
        transport,
        shutdown,
        world,
        Arc::new(Policy {
            positions,
            used: std::sync::Mutex::new([false; 3]),
        }),
    ));
    let mut peers = Vec::new();
    for id in 1..=3 {
        let mut data = [0; 256];
        data[0] = id as u8;
        let ticket = issuer.issue(id, Some(&data), 30).unwrap().bytes().to_vec();
        let (input, inputs) = watch::channel(crate::wire::Intent::default());
        let (ready, readiness) = watch::channel(Some(
            crate::procedural::neighbours(theia_core::coordinates::ChunkId { x: 62, z: 62 }, 1)
                .into_iter()
                .collect(),
        ));
        let (commands, command_queue) = mpsc::channel(8);
        let (publish, state) = watch::channel(State::default());
        let (actions, outgoing) = mpsc::channel(4);
        let (chat, chats) = mpsc::channel(4);
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            run_client_gameplay(
                address,
                &ticket,
                inputs,
                publish,
                shutdown,
                identity,
                GameplayClient {
                    party: Some(crate::party_client::Client::new(source, outgoing).unwrap()),
                    chat: with_chat.then(|| {
                        crate::chat_client::Client::new(chat_source, area, chats).unwrap()
                    }),
                    classic_source: Some(classic_source),
                    avatar_life: None,
                    public_attacks: None,
                    dodge: None,
                    player_life: None,
                    journal: None,
                    lifecycle: None,
                    dialogue: None,
                    ready: readiness,
                    commands: Some(command_queue),
                    choices: None,
                    combat: None,
                    quest: None,
                    progression: None,
                    inventory: None,
                    statuses: None,
                },
            )
            .await
        });
        peers.push(ClientPeer {
            state,
            actions,
            chat,
            stop: Stop(stop),
            task,
            _input: input,
            _ready: ready,
            _commands: commands,
        });
    }
    until(&peers, || {
        peers.iter().all(|p| {
            p.state
                .borrow()
                .party
                .as_ref()
                .is_some_and(|v| v.state.roster.len() == 3)
                && p.state.borrow().chunk_ack.is_some()
        })
    })
    .await;
    let keys: Vec<_> = peers.iter().map(|p| p.view().state.view.owner).collect();
    // Exercise the actual local predictor, not just transport snapshots. A
    // social profile changes the admitted world hash without changing movement.
    let mut predictor = crate::prediction_driver::Driver::with_classic_social(
        classic_source,
        false,
        with_chat.then_some(area),
        true,
    );
    let initial = peers[0].state.borrow().correction.unwrap().state.pose.feet;
    let patch = crate::procedural::Patch::build(crate::procedural::owner(initial).unwrap());
    let facing = theia_core::facing::Facing::from_units(16384);
    let intent = crate::wire::Intent {
        facing: Some(facing),
        axes: facing.world_axes([1000, 0]).unwrap(),
        ..Default::default()
    };
    let mut unbound = crate::prediction_driver::Driver::with_classic(classic_source, false);
    assert!(
        unbound
            .tick(
                &peers[0].state.borrow(),
                intent,
                Some(&patch),
                std::time::Instant::now()
            )
            .is_err()
    );
    let began = Instant::now();
    while began.elapsed() < Duration::from_millis(800) {
        let state = peers[0].state.borrow().clone();
        if let Some(driven) = predictor
            .tick(&state, intent, Some(&patch), std::time::Instant::now())
            .unwrap()
            && let Some(commands) = driven.packet
        {
            peers[0]._commands.send(commands).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
    until(&peers, || {
        let correction = peers[0].state.borrow().correction.unwrap().state;
        correction.pose.facing == Some(facing)
            && correction
                .pose
                .feet
                .metres()
                .into_iter()
                .zip(initial.metres())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                > 0.25
    })
    .await;
    assert!(predictor.stats.predicted_steps > 0 && predictor.stats.corrections > 0);
    assert_eq!(
        peers[0]
            .view()
            .state
            .roster
            .iter()
            .map(|p| p.name.as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        ["Arden", "Briar", "Cedar"].into_iter().collect()
    );
    peers[0].send(Action::Invite(keys[1]));
    until(&peers, || {
        peers[1].view().state.view.membership.incoming.is_some() && !peers[0].view().pending
    })
    .await;
    assert!(peers[2].view().state.view.membership.incoming.is_none());
    let invite = peers[1]
        .view()
        .state
        .view
        .membership
        .incoming
        .as_ref()
        .unwrap()
        .id;
    peers[1].send(Action::Accept(invite));
    until(&peers, || {
        peers[..2]
            .iter()
            .all(|p| p.view().state.view.membership.party.is_some())
            && !peers[1].view().pending
    })
    .await;
    assert!(peers[2].view().state.view.membership.party.is_none());
    peers[1].send(Action::Invite(keys[2]));
    until(&peers, || {
        peers[1].view().submitted == 2 && !peers[1].view().pending
    })
    .await;
    assert_eq!(peers[1].view().state.refusal, Some(Refusal::NotLeader));
    peers[0].send(Action::Invite(keys[2]));
    until(&peers, || {
        peers[2].view().state.view.membership.incoming.is_some()
    })
    .await;
    let invite = peers[2]
        .view()
        .state
        .view
        .membership
        .incoming
        .as_ref()
        .unwrap()
        .id;
    peers[2].send(Action::Accept(invite));
    until(&peers, || {
        peers.iter().all(|p| {
            p.view()
                .state
                .view
                .membership
                .party
                .as_ref()
                .is_some_and(|g| g.members.len() == 3)
        })
    })
    .await;
    peers[0].send(Action::Promote(keys[1]));
    until(&peers, || {
        peers.iter().all(|p| {
            p.view()
                .state
                .view
                .membership
                .party
                .as_ref()
                .unwrap()
                .leader
                == keys[1]
        })
    })
    .await;
    if with_chat {
        peers[0]
            .chat
            .try_send(crate::chat_client::Compose {
                channel: theia_core::chat::Channel::Say,
                text: theia_core::chat::Text::parse("Party ready").unwrap(),
            })
            .unwrap();
        until(&peers, || {
            peers.iter().all(|p| {
                p.state
                    .borrow()
                    .chat
                    .as_ref()
                    .is_some_and(|v| v.inbox.history().len() == 1)
            })
        })
        .await;
    }
    let departed = peers.remove(1);
    departed.stop.0.send_replace(true);
    departed.task.await.unwrap().unwrap();
    until(&peers, || {
        peers.iter().all(|p| {
            p.view().state.roster.len() == 2
                && p.view()
                    .state
                    .view
                    .membership
                    .party
                    .as_ref()
                    .is_some_and(|g| g.members.len() == 2 && g.leader == keys[0])
        })
    })
    .await;
    peers[0].send(Action::Leave);
    until(&peers, || {
        peers
            .iter()
            .all(|p| p.view().state.view.membership.party.is_none())
    })
    .await;
    let before = peers[0].state.borrow().snapshot.as_ref().unwrap().tick;
    until(&peers, || {
        peers[0].state.borrow().snapshot.as_ref().unwrap().tick > before + 2
    })
    .await;
    for peer in peers {
        assert!(!peer.view().pending && !peer.view().timed_out);
        assert!(
            peer.state
                .borrow()
                .transport
                .as_ref()
                .unwrap()
                .party_states_received
                >= 5
        );
        peer.stop.0.send_replace(true);
        peer.task.await.unwrap().unwrap();
    }
    server_stop.0.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.party.accepted, 6);
    assert_eq!(stats.party.refused, 1);
    assert_eq!(stats.party.invalid_uploads, 0);
    assert_eq!(stats.max_players, 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_classic_workers_form_manage_and_leave_a_party() {
    worker_flow(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_party_and_chat_profiles_coexist_with_classic_movement() {
    worker_flow(true).await;
}

#[test]
fn worker_bounds_admission_wait_timeout_queued_clicks_and_late_acknowledgements() {
    let mut world = world();
    let mut server = RenetServer::new(connection_config());
    let peer = admit(&mut world, &mut server, 1);
    world.step();
    let owner = world.entity_key(peer.actor).unwrap();
    let source = [4; 32];
    let expected = party_wire::profile_identity(source);
    let initial = party_wire::State {
        world: expected,
        tick: world.completed_tick(),
        view: world.party_view(peer.actor).unwrap(),
        refusal: None,
        roster: vec![party_wire::NamedActor {
            key: owner,
            name: peer.display_name.unwrap(),
        }],
    };
    let (send, outgoing) = mpsc::channel(4);
    let mut driver = Driver::new(
        crate::party_client::Client::new(source, outgoing).unwrap(),
        expected,
    )
    .unwrap();
    let intent = crate::party_client::Intent {
        owner,
        revision: initial.view.revision,
        action: Action::Leave,
    };
    let welcome = Welcome {
        world: expected,
        epoch: owner.epoch.value(),
        id: owner.entity.value(),
        origin: [0.; 3],
        terrain_stream: true,
    };
    let now = Instant::now();
    let (state, observed) = watch::channel(State::default());
    let mut stats = TransportStats::default();
    let mut transport = RenetClient::new(connection_config());
    transport.set_connected();
    send.try_send(intent.clone()).unwrap();
    driver.admit(&welcome, now).unwrap();
    driver
        .tick(&mut transport, now, &state, &mut stats)
        .unwrap();
    assert_eq!(stats.party_requests_sent, 0);
    assert!(
        driver
            .tick(
                &mut transport,
                now + Duration::from_secs(8),
                &state,
                &mut stats
            )
            .is_err()
    );
    driver
        .receive(&initial.encode().unwrap(), &state, &mut stats)
        .unwrap();
    send.try_send(intent.clone()).unwrap();
    driver
        .tick(&mut transport, now, &state, &mut stats)
        .unwrap();
    assert!(observed.borrow().party.as_ref().unwrap().pending);
    send.try_send(intent.clone()).unwrap();
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(5),
            &state,
            &mut stats,
        )
        .unwrap();
    assert!(observed.borrow().party.as_ref().unwrap().timed_out);
    assert!(!observed.borrow().party.as_ref().unwrap().pending);
    assert!(transport.is_connected());
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(6),
            &state,
            &mut stats,
        )
        .unwrap();
    assert_eq!(stats.party_requests_sent, 1); // queued click was discarded
    send.try_send(intent).unwrap();
    driver
        .tick(
            &mut transport,
            now + Duration::from_secs(6),
            &state,
            &mut stats,
        )
        .unwrap();
    let mut reply = initial.clone();
    reply.view.sequence = 1;
    reply.refusal = Some(Refusal::NotMember);
    driver
        .receive(&reply.encode().unwrap(), &state, &mut stats)
        .unwrap();
    assert!(observed.borrow().party.as_ref().unwrap().pending);
    reply.view.sequence = 2;
    driver
        .receive(&reply.encode().unwrap(), &state, &mut stats)
        .unwrap();
    assert!(!observed.borrow().party.as_ref().unwrap().pending);
    assert!(!observed.borrow().party.as_ref().unwrap().timed_out);
    let before = driver.view().unwrap().state;
    reply.view.sequence = 3;
    assert!(
        driver
            .receive(&reply.encode().unwrap(), &state, &mut stats)
            .is_err()
    );
    assert_eq!(driver.view().unwrap().state, before);
    assert_eq!(stats.party_timeouts, 1);
    assert_eq!(stats.party_requests_sent, 2);
    let (_, large) = mpsc::channel(5);
    assert!(crate::party_client::Client::new(source, large).is_err());
    let (_, outgoing) = mpsc::channel(4);
    assert!(
        Driver::new(
            crate::party_client::Client::new(source, outgoing).unwrap(),
            [9; 32]
        )
        .is_err()
    );
}
