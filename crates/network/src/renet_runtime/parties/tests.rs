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
    casts: mpsc::Sender<CastChoice>,
    selection: watch::Sender<Option<crate::targeting::Choice>>,
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

async fn paced_request(peers: &[ClientPeer], index: usize, action: Action) -> Option<Refusal> {
    tokio::time::sleep(Duration::from_millis(600)).await;
    let sequence = peers[index].view().state.view.sequence;
    peers[index].send(action);
    until(peers, || {
        peers[index].view().state.view.sequence > sequence && !peers[index].view().pending
    })
    .await;
    peers[index].view().state.refusal
}
fn marks(peer: &ClientPeer) -> Vec<theia_core::party::marks::Mark> {
    peer.view()
        .state
        .view
        .membership
        .party
        .as_ref()
        .map_or_else(Vec::new, |p| p.marks.clone())
}

async fn select_member(
    peers: &[ClientPeer],
    index: usize,
    target: Option<EntityKey>,
    accepted: bool,
) -> crate::targeting_wire::State {
    let sequence = peers[index]
        .state
        .borrow()
        .targeting
        .as_ref()
        .unwrap()
        .request_sequence;
    peers[index]
        .selection
        .send_replace(Some(crate::targeting::Choice {
            target: target.map(|key| key.entity),
        }));
    until(peers, || {
        peers[index]
            .state
            .borrow()
            .targeting
            .as_ref()
            .is_some_and(|v| v.request_sequence > sequence)
    })
    .await;
    let result = peers[index].state.borrow().targeting.clone().unwrap();
    assert_eq!(result.accepted, accepted);
    result
}

async fn worker_flow(with_chat: bool) {
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    // The combined chat fixture also carries real authoritative resource pools.
    let abilities =
        include_bytes!("../../../../../content/gameplay/development-resource-abilities.json");
    if with_chat {
        world.spawn_development_target([8072., 8000.]).unwrap();
        world
            .enable_resource_combat(
                abilities,
                100,
                5000,
                include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
            )
            .unwrap();
    }
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
        let (cast_sender, casts) = mpsc::channel(4);
        let (selection, choices) = watch::channel(None);
        let task = tokio::spawn(async move {
            let (events, _events) = mpsc::channel(8);
            let combat = with_chat.then(|| CombatActions {
                melee: None,
                catalog: Arc::new(theia_core::abilities::Catalog::from_json(abilities).unwrap()),
                casts,
                events,
            });
            let result = run_client_gameplay(
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
                    choices: with_chat.then_some(choices),
                    combat,
                    quest: None,
                    progression: None,
                    inventory: None,
                    statuses: None,
                },
            )
            .await;
            assert!(result.is_ok(), "{result:?}");
            result
        });
        peers.push(ClientPeer {
            state,
            actions,
            chat,
            casts: cast_sender,
            selection,
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
    assert!(peers[2].view().state.member_vitals.is_empty());
    for peer in &peers[..2] {
        let values = peer.view().state.member_vitals.clone();
        assert_eq!(values.iter().map(|v| v.key).collect::<Vec<_>>(), keys[..2]);
        for value in values {
            assert_eq!(
                value.vitals,
                with_chat.then(|| theia_core::resources::Vitals::new(100, 100, 100, 100).unwrap())
            );
        }
    }
    use theia_core::party::marks::{Mark, Symbol};
    let assign = |target, symbol| Action::Mark {
        target,
        symbol: Some(symbol),
    };
    tokio::time::sleep(Duration::from_millis(600)).await;
    let initial_poses: Vec<_> = peers
        .iter()
        .map(|p| p.state.borrow().correction.unwrap().state.pose)
        .collect();
    assert_eq!(
        paced_request(&peers, 0, assign(keys[1], Symbol::Skull)).await,
        None
    );
    until(&peers, || {
        marks(&peers[1])
            == vec![Mark {
                target: keys[1],
                symbol: Symbol::Skull,
            }]
    })
    .await;
    assert!(marks(&peers[2]).is_empty());
    assert_eq!(
        paced_request(&peers, 1, assign(keys[0], Symbol::Star)).await,
        Some(Refusal::NotLeader)
    );
    assert_eq!(
        paced_request(&peers, 2, assign(keys[1], Symbol::Star)).await,
        Some(Refusal::NotMember)
    );
    assert_eq!(
        paced_request(&peers, 0, assign(keys[0], Symbol::Skull)).await,
        None
    );
    until(&peers, || {
        marks(&peers[1])
            == vec![Mark {
                target: keys[0],
                symbol: Symbol::Skull,
            }]
    })
    .await;
    assert_eq!(
        paced_request(&peers, 0, assign(keys[1], Symbol::Star)).await,
        None
    );
    assert_eq!(
        paced_request(
            &peers,
            0,
            Action::Mark {
                target: keys[0],
                symbol: None
            }
        )
        .await,
        None
    );
    until(&peers, || {
        marks(&peers[1])
            == vec![Mark {
                target: keys[1],
                symbol: Symbol::Star,
            }]
    })
    .await;
    assert_eq!(paced_request(&peers, 0, Action::ClearMarks).await, None);
    until(&peers, || marks(&peers[1]).is_empty()).await;
    assert_eq!(
        paced_request(&peers, 0, assign(keys[1], Symbol::Star)).await,
        None
    );
    until(&peers, || marks(&peers[0]) == marks(&peers[1])).await;
    assert_eq!(
        peers
            .iter()
            .map(|p| p.state.borrow().correction.unwrap().state.pose)
            .collect::<Vec<_>>(),
        initial_poses
    );
    assert!(marks(&peers[2]).is_empty());
    if with_chat {
        until(&peers, || {
            peers.iter().all(|p| p.state.borrow().targeting.is_some())
        })
        .await;
        let facing = peers[0]
            .state
            .borrow()
            .correction
            .unwrap()
            .state
            .pose
            .facing;
        let friendly = select_member(&peers, 0, Some(keys[1]), true).await;
        assert_eq!(friendly.selected_member, Some(keys[1].entity));
        assert!(friendly.selected.is_none());
        assert!(friendly.targets.iter().all(|t| !keys.contains(&t.key)));
        assert_eq!(
            peers[0]
                .state
                .borrow()
                .correction
                .unwrap()
                .state
                .pose
                .facing,
            facing
        );
        assert!(
            peers[1]
                .state
                .borrow()
                .targeting
                .as_ref()
                .unwrap()
                .selected_member
                .is_none()
        );
        let refused = select_member(&peers, 2, Some(keys[1]), false).await;
        assert!(refused.selected.is_none() && refused.selected_member.is_none());
        let enemy = friendly.targets[0].key;
        assert_eq!(
            paced_request(&peers, 0, assign(enemy, Symbol::Skull)).await,
            None
        );
        until(&peers, || {
            marks(&peers[1])
                .iter()
                .any(|m| m.target == enemy && m.symbol == Symbol::Skull)
        })
        .await;
        assert!(marks(&peers[2]).is_empty());
        assert_eq!(
            peers[0]
                .state
                .borrow()
                .targeting
                .as_ref()
                .unwrap()
                .selected_member,
            Some(keys[1].entity)
        );
        assert_eq!(
            paced_request(
                &peers,
                0,
                Action::Mark {
                    target: enemy,
                    symbol: None
                }
            )
            .await,
            None
        );
        let hostile = select_member(&peers, 0, Some(enemy), true).await;
        assert_eq!(hostile.selected, Some(enemy.entity));
        assert!(hostile.selected_member.is_none());
        let cleared = select_member(&peers, 0, None, true).await;
        assert!(cleared.selected.is_none() && cleared.selected_member.is_none());
        assert_eq!(
            select_member(&peers, 0, Some(keys[1]), true)
                .await
                .selected_member,
            Some(keys[1].entity)
        );
        // A malicious hostile spell against the friendly selection remains a
        // server refusal, without resource cost or damage to the party member.
        peers[0]
            .casts
            .try_send(CastChoice {
                ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target: keys[1].entity,
            })
            .unwrap();
        until(&peers, || {
            peers[0]
                .state
                .borrow()
                .combat
                .as_ref()
                .unwrap()
                .reply
                .is_some()
        })
        .await;
        assert_eq!(
            peers[0]
                .state
                .borrow()
                .combat
                .as_ref()
                .unwrap()
                .reply
                .unwrap()
                .outcome,
            crate::simulation::combat::Outcome::Rejected(
                crate::simulation::combat::Reason::UnknownTarget
            )
        );
        assert_eq!(
            peers[0]
                .state
                .borrow()
                .combat
                .as_ref()
                .unwrap()
                .vitals
                .unwrap()
                .mana(),
            100
        );
        assert_eq!(
            peers[1].view().state.member_vitals[1]
                .vitals
                .unwrap()
                .health(),
            100
        );
        let revision = peers[1].view().state.view.revision;
        let target = peers[0].state.borrow().combat.as_ref().unwrap().targets[0].entity;
        peers[0]
            .casts
            .try_send(CastChoice {
                ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target,
            })
            .unwrap();
        until(&peers, || {
            peers[1].view().state.member_vitals[0]
                .vitals
                .unwrap()
                .mana()
                < 100
                && peers[0]
                    .state
                    .borrow()
                    .combat
                    .as_ref()
                    .unwrap()
                    .vitals
                    .unwrap()
                    .mana()
                    < 100
        })
        .await;
        assert_eq!(peers[1].view().state.view.revision, revision);
        assert!(peers[2].view().state.member_vitals.is_empty());
        assert!(
            peers[0]
                .state
                .borrow()
                .combat
                .as_ref()
                .unwrap()
                .vitals
                .unwrap()
                .mana()
                < 100
        );
    }
    let submitted = peers[1].view().submitted;
    peers[1].send(Action::Invite(keys[2]));
    until(&peers, || {
        peers[1].view().submitted > submitted && !peers[1].view().pending
    })
    .await;
    assert_eq!(peers[1].view().state.refusal, Some(Refusal::NotLeader));
    assert_eq!(
        paced_request(&peers, 0, Action::Invite(keys[2])).await,
        None
    );
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
    assert_eq!(
        paced_request(&peers, 0, Action::Promote(keys[1])).await,
        None
    );
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
    assert!(peers.iter().all(|p| marks(p)
        == vec![Mark {
            target: keys[1],
            symbol: Symbol::Star
        }]));
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
    for peer in &peers {
        assert!(marks(peer).is_empty(), "Departed member kept a mark");
        assert_eq!(
            peer.view()
                .state
                .member_vitals
                .iter()
                .map(|v| v.key)
                .collect::<Vec<_>>(),
            vec![keys[0], keys[2]]
        );
    }
    if with_chat {
        until(&peers, || {
            peers[0]
                .state
                .borrow()
                .targeting
                .as_ref()
                .unwrap()
                .selected_member
                .is_none()
        })
        .await;
        assert_eq!(
            select_member(&peers, 0, Some(keys[2]), true)
                .await
                .selected_member,
            Some(keys[2].entity)
        );
    }
    assert_eq!(paced_request(&peers, 0, Action::Leave).await, None);
    until(&peers, || {
        peers
            .iter()
            .all(|p| p.view().state.view.membership.party.is_none())
    })
    .await;
    assert!(
        peers
            .iter()
            .all(|p| p.view().state.member_vitals.is_empty())
    );
    if with_chat {
        until(&peers, || {
            peers[0]
                .state
                .borrow()
                .targeting
                .as_ref()
                .unwrap()
                .selected_member
                .is_none()
        })
        .await;
        assert!(
            select_member(&peers, 0, Some(keys[2]), false)
                .await
                .selected_member
                .is_none()
        );
    }
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
    assert_eq!(stats.party.accepted, if with_chat { 14 } else { 12 });
    assert_eq!(stats.party.refused, 3);
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
        member_vitals: Vec::new(),
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

#[test]
fn changed_vitals_coalesce_under_backpressure_without_membership_revision_changes() {
    let mut world = world();
    world.spawn_development_target([8065., 8000.]).unwrap();
    world
        .enable_resource_combat(
            include_bytes!("../../../../../content/gameplay/development-resource-abilities.json"),
            100,
            5000,
            include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
        )
        .unwrap();
    let mut server = RenetServer::new(connection_config());
    let one = admit(&mut world, &mut server, 1);
    let two = admit(&mut world, &mut server, 2);
    let actor = one.actor;
    let recipient = two.actor;
    let target = world.entity_key(recipient).unwrap();
    let mut peers = BTreeMap::from([(1, one), (2, two)]);
    let mut stats = Stats::default();
    let invite = bytes(&world, actor, Action::Invite(target));
    request(peers.get_mut(&1).unwrap(), &mut world, &invite, &mut stats).unwrap();
    let id = world
        .party_view(recipient)
        .unwrap()
        .membership
        .incoming
        .unwrap()
        .id;
    let accept = bytes(&world, recipient, Action::Accept(id));
    request(peers.get_mut(&2).unwrap(), &mut world, &accept, &mut stats).unwrap();
    world.step();
    publish(&mut server, &mut peers, &world, &mut stats).unwrap();
    let prior = peers[&2].party.sent.clone().unwrap();
    let patch = world
        .procedural()
        .unwrap()
        .patch(world.global_position(actor).unwrap())
        .unwrap();
    assert!(world.client_ready(
        actor,
        &crate::wire::ChunkReady {
            version: crate::wire::VERSION,
            epoch: world.epoch(),
            sequence: 1,
            chunks: patch.ids.iter().map(|id| [id.x, id.z]).collect(),
        }
    ));
    let creature = world.combat_snapshot(actor).unwrap().targets[0].key;
    let result = world.cast(
        actor,
        crate::simulation::combat::Request {
            epoch: world.epoch(),
            sequence: 1,
            ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target: creature,
        },
    );
    assert!(matches!(
        result.reply.outcome,
        crate::simulation::combat::Outcome::Started { .. }
    ));
    for id in [1, 2] {
        while server.can_send_message(id, EVENT_CHANNEL, 1024) {
            server.send_message(id, EVENT_CHANNEL, vec![0; 1024]);
        }
    }
    for _ in 0..30 {
        world.step();
        publish(&mut server, &mut peers, &world, &mut stats).unwrap();
    }
    assert_eq!(peers[&2].party.sent.as_ref().unwrap(), &prior);
    assert!(stats.deferred > 0);
    let mut drained = RenetServer::new(connection_config());
    drained.add_connection(1);
    drained.add_connection(2);
    publish(&mut drained, &mut peers, &world, &mut stats).unwrap();
    let latest = peers[&2].party.sent.as_ref().unwrap();
    assert_eq!(latest.view, prior.view);
    assert_eq!(latest.member_vitals[0].vitals, world.combat_vitals(actor));
    assert!(latest.member_vitals[0].vitals.unwrap().mana() < 100);
    let sent = stats.states_sent;
    publish(&mut drained, &mut peers, &world, &mut stats).unwrap();
    assert_eq!(stats.states_sent, sent);
}
