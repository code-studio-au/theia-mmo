//! Actual encrypted production runtime and actual World health. Development
//! tickets only: no OIDC, SQL health durability or native presentation claim.
use super::*;
use crate::test_relay as relay;
use crate::{
    client::State, combat_wire::Batch, renet_runtime as net, simulation::creatures, wire::Intent,
};
use serde_json::{Value, json};
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};
use theia_core::{abilities::Catalog, coordinates::ChunkId};
use tokio::{sync::watch, task::JoinHandle};
const STATS: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-character-stats.json");
const CREATURE: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-creature-patrol.json");
const POLICY: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-creature-retaliation.json");
const RESPAWN: &[u8] =
    include_bytes!("../../../../../content/gameplay/development-player-respawn.json");
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
fn fixture(
    rate: TickRate,
    damage: u32,
) -> (World, EntityKey, Arc<Catalog>, Arc<Profile>, Vec<ChunkId>) {
    fixture_mode(rate, damage, false)
}
fn fixture_mode(
    rate: TickRate,
    damage: u32,
    pursuit: bool,
) -> (World, EntityKey, Arc<Catalog>, Arc<Profile>, Vec<ChunkId>) {
    let mut creature: Value = serde_json::from_slice(CREATURE).unwrap();
    creature["speed_metres_per_second"] = json!(0.1);
    if pursuit {
        creature["aggro"] = json!({"perception_metres":12.,"leash_metres":20.,"chase_speed":5.,"return_speed":5.,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
    }
    let creature = serde_json::to_vec(&creature).unwrap();
    let spawn = if pursuit {
        [8000., 8000.]
    } else {
        [8003.5, 8000.]
    };
    let (mut world, npc) = creatures::tests::fixture_profile_at(rate, spawn, &creature);
    world.enable_prediction().unwrap();
    let mut abilities: Value = serde_json::from_slice(include_bytes!(
        "../../../../../content/gameplay/development-abilities.json"
    ))
    .unwrap();
    for ability in abilities["abilities"].as_array_mut().unwrap() {
        ability["cast_ms"] = json!(0);
        ability["cooldown_ms"] = json!(0);
        ability["uses_global_cooldown"] = json!(false);
        ability["effect"]["amount"] = json!(1);
    }
    let abilities = serde_json::to_vec(&abilities).unwrap();
    world
        .enable_resource_combat(&abilities, 100, 300, STATS)
        .unwrap();
    let mut policy: Value = serde_json::from_slice(POLICY).unwrap();
    policy["damage"] = json!(damage);
    policy["windup_ms"] = json!(100);
    policy["recovery_ms"] = json!(100);
    let mut respawn: Value = serde_json::from_slice(RESPAWN).unwrap();
    respawn["delay_ms"] = json!(600);
    let profile = Arc::new(
        Profile::from_json(
            STATS,
            &creature,
            &serde_json::to_vec(&policy).unwrap(),
            &serde_json::to_vec(&respawn).unwrap(),
        )
        .unwrap(),
    );
    world.enable_player_life(profile.clone()).unwrap();
    let chunks = world
        .procedural()
        .unwrap()
        .patch(theia_core::coordinates::GlobalPosition::new([8004., 0., 8000.]).unwrap())
        .unwrap()
        .ids
        .iter()
        .copied()
        .collect();
    (
        world,
        npc,
        Arc::new(Catalog::from_json(&abilities).unwrap()),
        profile,
        chunks,
    )
}
struct Peer {
    state: watch::Receiver<State>,
    casts: mpsc::Sender<net::CastChoice>,
    commands: mpsc::Sender<crate::prediction_wire::Commands>,
    life: mpsc::Receiver<Observation>,
    combat: mpsc::Receiver<Batch>,
    seen: Vec<Observation>,
    stop: watch::Sender<bool>,
    task: JoinHandle<Result<()>>,
    _ready: watch::Sender<Option<Vec<ChunkId>>>,
    _input: watch::Sender<Intent>,
    _choice: watch::Sender<Option<crate::targeting::Choice>>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_two_peers_observe_passive_pursuit_melee_leash_and_patrol_return() {
    let (world, npc, catalog, profile, chunks) = fixture_mode(TickRate::Hz30, 1, true);
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let a_token = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
    let b_token = issuer.issue(2, None, 60).unwrap().bytes().to_vec();
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(
        address,
        a_token,
        identity,
        profile.clone(),
        catalog.clone(),
        chunks.clone(),
    );
    let attacked = a
        .until(|s, _| {
            s.player_life
                .as_ref()
                .is_some_and(|p| p.current().is_some_and(|v| v.health < 100))
        })
        .await;
    let a_key = attacked
        .player_life
        .as_ref()
        .unwrap()
        .current()
        .unwrap()
        .key;
    let mut b = Peer::start(address, b_token, identity, profile, catalog, chunks);
    b.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
        .await;
    let patch = crate::procedural::Patch::build(ChunkId { x: 62, z: 62 });
    let mut drivers = [
        crate::prediction_driver::Driver::with_player_life(),
        crate::prediction_driver::Driver::with_player_life(),
    ];
    let started = std::time::Instant::now();
    let mut extents = [[f64::INFINITY, f64::NEG_INFINITY]; 2];
    while started.elapsed() < Duration::from_secs(11) {
        a.drain();
        b.drain();
        assert!(a.seen.iter().all(|v| v.key == a_key));
        assert!(b.seen.iter().all(|v| v.health == 100 && v.windup.is_none()));
        a.seen.clear();
        b.seen.clear();
        let seconds = started.elapsed().as_secs_f32();
        for (index, peer) in [&mut a, &mut b].into_iter().enumerate() {
            let state = peer.state.borrow().clone();
            assert!(state.error.is_none(), "{:?}", state.error);
            if let Some(target) = state
                .targeting
                .as_ref()
                .and_then(|s| s.targets.iter().find(|t| t.key == npc))
            {
                extents[index][0] = extents[index][0].min(target.position.metres()[0]);
                extents[index][1] = extents[index][1].max(target.position.metres()[0]);
            }
            let axes = if index == 0 && seconds < 5. {
                [-1000, 0]
            } else if index == 1 && seconds < 4. {
                [0, 1000]
            } else {
                [0, 0]
            };
            if let Some(driven) = drivers[index]
                .tick(
                    &state,
                    Intent {
                        axes,
                        ..Default::default()
                    },
                    Some(&patch),
                    std::time::Instant::now(),
                )
                .unwrap()
                && let Some(packet) = driven.packet
            {
                peer.commands.send(packet).await.unwrap();
            }
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    for [min, max] in extents {
        assert!(
            max - min > 10.,
            "NPC did not replicate its chase/return: {min}..{max}"
        );
    }
    a.finish().await.unwrap();
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    let metrics = stats.creature_pursuit.unwrap();
    assert_eq!(metrics.acquisitions, 1);
    assert_eq!(metrics.returns, 1);
    assert_eq!(metrics.arrivals, 1);
    assert!(metrics.published > 0 && metrics.chase_ticks > 0 && metrics.attack_ticks > 0);
    assert!(stats.player_life.unwrap().damage > 0);
    assert_eq!(stats.max_players, 2);
    assert!(matches!(
        stats.patrol.unwrap().mode,
        creatures::Mode::Patrol | creatures::Mode::Idle
    ));
}
impl Peer {
    fn start(
        address: SocketAddr,
        ticket: Vec<u8>,
        world: [u8; 32],
        profile: Arc<Profile>,
        catalog: Arc<Catalog>,
        chunks: Vec<ChunkId>,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(Some(chunks));
        let (choice, choices) = watch::channel(None);
        let (casts, requests) = mpsc::channel(8);
        let (events, combat) = mpsc::channel(16);
        let (life_events, life) = mpsc::channel(PRESENTATION_CAPACITY);
        let (commands, command_rx) = mpsc::channel(8);
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            let result = net::run_client_gameplay(
                address,
                &ticket,
                inputs,
                states.clone(),
                shutdown,
                world,
                net::GameplayClient {
                    classic_source: None,
                    dodge: None,
                    player_life: Some(Client {
                        membership: None,
                        profile,
                        events: life_events,
                    }),
                    lifecycle: None,
                    dialogue: None,
                    ready: readiness,
                    commands: Some(command_rx),
                    choices: Some(choices),
                    combat: Some(net::CombatActions {
                        melee: None,
                        catalog,
                        casts: requests,
                        events,
                    }),
                    quest: None,
                    progression: None,
                    inventory: None,
                    statuses: None,
                },
            )
            .await;
            if let Err(error) = &result {
                states.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            state,
            casts,
            commands,
            life,
            combat,
            seen: Vec::new(),
            stop,
            task,
            _ready: ready,
            _input: input,
            _choice: choice,
        }
    }
    fn drain(&mut self) {
        while let Ok(value) = self.life.try_recv() {
            self.seen.push(value);
            assert!(self.seen.len() <= 256);
        }
        while self.combat.try_recv().is_ok() {}
    }
    async fn until(&mut self, predicate: impl Fn(&State, &[Observation]) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                self.drain();
                let state = self.state.borrow().clone();
                assert!(state.error.is_none(), "{:?}", state.error);
                if predicate(&state, &self.seen) {
                    return state;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap_or_else(|_| {
            let s = self.state.borrow();
            panic!(
                "Encrypted life timed out: tick={:?}, life={:?}, reply={:?}, error={:?}, events={}",
                s.snapshot.as_ref().map(|v| v.tick),
                s.player_life.as_ref().and_then(|v| v.current()).copied(),
                s.combat.as_ref().and_then(|v| v.reply),
                s.error,
                self.seen.len()
            )
        })
    }
    async fn cast(&self, npc: EntityKey) {
        self.casts
            .send(net::CastChoice {
                ability: theia_core::content::ContentId::new("theia.dev.arcane-bolt").unwrap(),
                target: npc.entity,
            })
            .await
            .unwrap();
    }
    async fn finish(self) -> Result<()> {
        self.stop.send_replace(true);
        tokio::time::timeout(Duration::from_secs(3), self.task)
            .await
            .unwrap()
            .unwrap()
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_impaired_peers_receive_only_own_damage_death_and_matching_respawn_corrections_at_all_rates()
 {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let (world, npc, catalog, profile, chunks) = fixture(rate, 40);
        let identity = world.identity();
        let relays: Vec<_> = (0..2)
            .map(|_| {
                relay::Relay::bind(relay::Profile {
                    name: "life-impaired",
                    delay_ms: 20,
                    jitter_ms: 30,
                    loss_every: 11,
                    duplicate_every: 17,
                })
            })
            .collect();
        let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let (transport, issuer) = net::secure_issuer(socket, addresses.clone()).unwrap();
        let a_token = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
        let b_token = issuer.issue(2, None, 60).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let _guard = Stop(stop.clone());
        let workers: Vec<_> = relays
            .into_iter()
            .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
            .collect();
        let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
        let mut a = Peer::start(
            addresses[0],
            a_token,
            identity,
            profile.clone(),
            catalog.clone(),
            chunks.clone(),
        );
        let mut b = Peer::start(addresses[1], b_token, identity, profile, catalog, chunks);
        let initial = a
            .until(|s, _| {
                s.player_life.is_some()
                    && s.chunk_ack.is_some()
                    && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
            })
            .await;
        b.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
            .await;
        let a_key = initial.player_life.as_ref().unwrap().current().unwrap().key;
        a.cast(npc).await;
        let dead = a
            .until(|s, _| {
                s.player_life
                    .as_ref()
                    .is_some_and(|p| p.current().is_some_and(|v| v.health == 0))
            })
            .await;
        let mut driver = crate::prediction_driver::Driver::with_player_life();
        let held = driver
            .tick(
                &dead,
                Intent {
                    axes: [1000, 0],
                    ..Intent::default()
                },
                None,
                std::time::Instant::now(),
            )
            .unwrap()
            .unwrap();
        assert!(driver.life_held() && held.packet.is_none());
        assert_eq!(driver.stats.predicted_steps, 0);
        assert!(
            !dead
                .player_life
                .as_ref()
                .unwrap()
                .movement_allowed(&dead.correction.unwrap().state)
        );
        let passive = b.state.borrow().clone();
        assert_eq!(
            passive
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .health,
            100
        );
        b.drain();
        assert!(b.seen.iter().all(|v| v.windup.is_none() && v.health == 100));
        let fresh = a
            .until(|s, _| {
                s.player_life.as_ref().is_some_and(|p| {
                    p.current()
                        .is_some_and(|v| v.key.generation.get() == 2 && v.health == 100)
                        && s.correction.is_some_and(|c| p.movement_allowed(&c.state))
                })
            })
            .await;
        assert_eq!(
            fresh
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .movement_stream,
            3
        );
        assert!(a.seen.iter().any(|v| v.windup.is_some()));
        assert!(a.seen.iter().all(|v| v.key.actor == a_key.actor));
        let patch = crate::procedural::Patch::build(ChunkId { x: 62, z: 62 });
        let start_x = fresh.correction.unwrap().state.pose.feet.metres()[0];
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                a.drain();
                b.drain();
                let state = a.state.borrow().clone();
                assert!(state.error.is_none(), "{:?}", state.error);
                if state
                    .correction
                    .is_some_and(|c| c.state.pose.feet.metres()[0] > start_x + 0.1)
                {
                    break;
                }
                if let Some(driven) = driver
                    .tick(
                        &state,
                        Intent {
                            axes: [1000, 0],
                            ..Intent::default()
                        },
                        Some(&patch),
                        std::time::Instant::now(),
                    )
                    .unwrap()
                    && let Some(packet) = driven.packet
                {
                    assert_eq!(packet.stream_epoch, 3);
                    a.commands.send(packet).await.unwrap();
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("Respawn prediction did not move its encrypted authority");
        assert!(!driver.life_held() && driver.stats.predicted_steps > 0);
        b.cast(npc).await;
        b.until(|s, _| {
            s.player_life
                .as_ref()
                .is_some_and(|p| p.current().is_some_and(|v| v.health == 0))
        })
        .await;
        a.drain();
        assert_eq!(
            a.state
                .borrow()
                .player_life
                .as_ref()
                .unwrap()
                .current()
                .unwrap()
                .health,
            100
        );
        b.until(|s, _| {
            s.player_life.as_ref().is_some_and(|p| {
                p.current()
                    .is_some_and(|v| v.key.generation.get() == 2 && v.health == 100)
            })
        })
        .await;
        a.finish().await.unwrap();
        b.finish().await.unwrap();
        stop.send_replace(true);
        let stats = tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        for worker in workers {
            worker.await.unwrap();
        }
        assert_eq!(stats.max_players, 2);
        assert!(stats.life_states_sent >= 12);
        assert_eq!(stats.life_max_frame_bytes, wire::MAX_FRAME_BYTES);
        let life = stats.player_life.unwrap();
        assert_eq!((life.deaths, life.respawns), (2, 2));
        assert_eq!(life.damage, 200);
        assert_eq!(stats.life_backpressure_disconnects, 0);
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stalled_life_presentation_disconnects_without_dropping_transitions_and_the_other_peer_keeps_playing()
 {
    let (world, npc, catalog, profile, chunks) = fixture(TickRate::Hz30, 1);
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(
        address,
        issuer.issue(1, None, 60).unwrap().bytes().to_vec(),
        identity,
        profile.clone(),
        catalog.clone(),
        chunks.clone(),
    );
    let mut b = Peer::start(
        address,
        issuer.issue(2, None, 60).unwrap().bytes().to_vec(),
        identity,
        profile,
        catalog,
        chunks,
    );
    a.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
        .await;
    b.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
        .await;
    a.cast(npc).await;
    // Keep A's life receiver alive but stop draining it. Combat has only the
    // initial spell events, so this specifically fills the life queue.
    tokio::time::timeout(Duration::from_secs(8), async {
        while !a.task.is_finished() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let error = a.finish().await.unwrap_err().to_string();
    assert!(error.contains("Life presentation queue full"), "{error}");
    b.cast(npc).await;
    b.until(|s, _| {
        s.player_life
            .as_ref()
            .is_some_and(|p| p.current().is_some_and(|v| v.health < 100))
    })
    .await;
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert!(stats.player_life.unwrap().strikes > 0);
}
#[test]
fn full_server_reliable_queue_unit_retires_the_life_peer_instead_of_marking_an_unsent_frame_delivered()
 {
    let (mut world, _, _, _, _) = fixture(TickRate::Hz30, 40);
    let mut server = RenetServer::new(crate::renet_transport::connection_config());
    server.add_connection(1);
    let peer = super::super::admit_peer(
        &mut world,
        &mut server,
        1,
        None,
        None,
        tokio::time::Instant::now(),
    )
    .unwrap()
    .unwrap();
    let actor = peer.actor;
    let mut peers = BTreeMap::from([(1, peer)]);
    let available = server.channel_available_memory(1, EVENT_CHANNEL);
    assert!(available > 0);
    server.send_message(1, EVENT_CHANNEL, vec![0; available]);
    let mut stats = ServerStats::default();
    publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
    assert!(peers.is_empty());
    assert!(world.player_life_snapshot(actor).is_none());
    assert_eq!(
        (stats.life_states_sent, stats.life_backpressure_disconnects),
        (0, 1)
    );
}

async fn forge_server_state(
    address: SocketAddr,
    ticket: Vec<u8>,
    world: [u8; 32],
    bytes: Vec<u8>,
) -> Welcome {
    let token = net::read_ticket(&ticket, address).unwrap();
    let mut transport = crate::renet_transport::secure_client(
        UdpSocket::bind("127.0.0.1:0").unwrap(),
        net::now().unwrap(),
        token,
    )
    .unwrap();
    let mut client = renet::RenetClient::new(crate::renet_transport::connection_config());
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut welcome = None;
        let mut sent = false;
        let mut last = tokio::time::Instant::now();
        loop {
            let now = tokio::time::Instant::now();
            let delta = now.duration_since(last);
            last = now;
            client.update(delta);
            if transport.update(delta, &mut client).is_err() || client.is_disconnected() {
                assert!(sent, "Rogue disconnected before its state was sent");
                return welcome.unwrap();
            }
            while let Some(packet) = client.receive_message(EVENT_CHANNEL) {
                if welcome.is_none() {
                    welcome = Some(Welcome::decode(&packet, world).unwrap());
                }
            }
            if welcome.is_some() && !sent {
                client.send_message(EVENT_CHANNEL, bytes.clone());
                sent = true;
            }
            if transport.send_packets(&mut client).is_err() {
                assert!(sent, "Rogue disconnected before its state was sent");
                return welcome.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("Server did not reject the forged life direction")
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn old_life_input_cannot_move_a_respawned_peer_and_a_client_cannot_publish_another_players_health()
 {
    let (world, npc, catalog, profile, chunks) = fixture(TickRate::Hz30, 40);
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(
        address,
        issuer.issue(1, None, 60).unwrap().bytes().to_vec(),
        identity,
        profile.clone(),
        catalog.clone(),
        chunks.clone(),
    );
    let mut b = Peer::start(
        address,
        issuer.issue(2, None, 60).unwrap().bytes().to_vec(),
        identity,
        profile.clone(),
        catalog,
        chunks,
    );
    a.until(|s, _| {
        s.player_life.is_some()
            && s.chunk_ack.is_some()
            && s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
    })
    .await;
    b.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
        .await;
    a.cast(npc).await;
    a.until(|s, _| {
        s.player_life
            .as_ref()
            .is_some_and(|p| p.current().is_some_and(|v| v.health == 0))
    })
    .await;
    let fresh = a
        .until(|s, _| {
            s.player_life.as_ref().is_some_and(|p| {
                p.current().is_some_and(|v| v.key.generation.get() == 2)
                    && s.correction.is_some_and(|c| p.movement_allowed(&c.state))
            })
        })
        .await;
    let pose = fresh.correction.unwrap().state;
    let old = crate::prediction_wire::Commands {
        epoch: pose.actor.epoch.value(),
        stream_epoch: pose.input.stream_epoch - 1,
        ready_sequence: fresh.chunk_report.unwrap().sequence,
        commands: vec![crate::commands::Command {
            sequence: 1,
            tick: pose.input.tick + 4,
            intent: Intent {
                axes: [1000, 0],
                ..Default::default()
            },
        }],
    };
    a.commands.send(old.clone()).await.unwrap();
    let unchanged = a
        .until(|s, _| {
            s.correction
                .is_some_and(|c| c.state.input.tick >= pose.input.tick + 6)
        })
        .await;
    let correction = unchanged.correction.unwrap().state;
    assert_eq!(correction.input.applied_sequence, 0);
    assert!((correction.pose.feet.metres()[0] - pose.pose.feet.metres()[0]).abs() < 0.001);
    let new = crate::prediction_wire::Commands {
        stream_epoch: correction.input.stream_epoch,
        commands: vec![crate::commands::Command {
            tick: correction.input.tick + 4,
            ..old.commands[0]
        }],
        ..old
    };
    a.commands.send(new).await.unwrap();
    let moved = a
        .until(|s, _| {
            s.correction
                .is_some_and(|c| c.state.input.applied_sequence == 1)
        })
        .await;
    assert!(moved.correction.unwrap().state.pose.feet.metres()[0] > pose.pose.feet.metres()[0]);
    let mut fake = *b
        .state
        .borrow()
        .player_life
        .as_ref()
        .unwrap()
        .current()
        .unwrap();
    fake.health = 1;
    fake.tick += 1;
    let forged = wire::encode(&profile, TickRate::Hz30, &fake).unwrap();
    let rogue = forge_server_state(
        address,
        issuer.issue(3, None, 60).unwrap().bytes().to_vec(),
        identity,
        forged,
    )
    .await;
    assert_ne!(rogue.id, fake.key.actor.entity.value());
    let intact = b
        .until(|s, _| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.players.len() == 2 && s.tick > fake.tick + 2)
        })
        .await;
    assert_eq!(
        intact
            .player_life
            .as_ref()
            .unwrap()
            .current()
            .unwrap()
            .health,
        100
    );
    assert!(intact.error.is_none());
    a.finish().await.unwrap();
    b.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.life_states_rejected, 1);
    assert!(stats.received_inputs >= 2);
    assert_eq!(stats.queued_commands, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mismatched_life_profile_fails_before_publication_while_the_other_peer_keeps_playing() {
    let (world, npc, catalog, profile, chunks) = fixture(TickRate::Hz30, 40);
    let (_, _, _, incompatible, _) = fixture(TickRate::Hz30, 41);
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    // Even a caller that supplies the correct world hash cannot bypass the
    // independent exact-byte life profile binding.
    let mut bad = Peer::start(
        address,
        issuer.issue(1, None, 60).unwrap().bytes().to_vec(),
        identity,
        incompatible,
        catalog.clone(),
        chunks.clone(),
    );
    let mut good = Peer::start(
        address,
        issuer.issue(2, None, 60).unwrap().bytes().to_vec(),
        identity,
        profile,
        catalog,
        chunks,
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        while !bad.task.is_finished() {
            good.drain();
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    assert!(bad.state.borrow().player_life.is_none());
    assert!(bad.life.try_recv().is_err());
    assert_eq!(
        bad.finish().await.unwrap_err().to_string(),
        "Invalid player-life observation"
    );
    good.until(|s, _| s.player_life.is_some() && s.chunk_ack.is_some())
        .await;
    good.cast(npc).await;
    good.until(|s, _| {
        s.player_life
            .as_ref()
            .is_some_and(|p| p.current().is_some_and(|v| v.health < 100))
    })
    .await;
    good.finish().await.unwrap();
    stop.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert!(stats.player_life.unwrap().damage >= 40);
    assert!(stats.life_states_sent > 0);
}
