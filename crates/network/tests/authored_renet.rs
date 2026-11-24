//! Actual exported Synty starter world and encrypted peers, without a renderer.
//! Explicit CPU chunk readiness is test scaffolding, not native-client acceptance.
use std::{
    net::UdpSocket,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::{coordinates::ChunkId, facing::Facing};
use theia_network::{
    authored_world::Store,
    client::State,
    prediction_driver::Driver,
    procedural::{Runtime, Source},
    renet_runtime::{self as net, GameplayClient},
    simulation::World,
    wire::Intent,
};
use tokio::sync::{mpsc, watch};

struct Cleanup {
    stop: watch::Sender<bool>,
    tickets: PathBuf,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        self.stop.send_replace(true);
        let _ = std::fs::remove_dir_all(&self.tickets);
    }
}
struct Peer {
    state: watch::Receiver<State>,
    ready: watch::Sender<Option<Vec<ChunkId>>>,
    commands: mpsc::Sender<theia_network::prediction_wire::Commands>,
    _input: watch::Sender<Intent>,
    _choice: Option<watch::Sender<Option<theia_network::targeting::Choice>>>,
    _casts: Option<mpsc::Sender<net::CastChoice>>,
    _events: Option<mpsc::Receiver<theia_network::combat_wire::Batch>>,
    dialogue: Option<mpsc::Sender<net::dialogue::Intent>>,
    driver: Driver,
    terrain: Runtime,
    initial: [f64; 3],
    task: tokio::task::JoinHandle<theia_network::Result<()>>,
}
fn mode(
    ready: net::ChunkReadiness,
    commands: mpsc::Receiver<theia_network::prediction_wire::Commands>,
    source: [u8; 32],
) -> GameplayClient {
    GameplayClient {
        party: None,
        chat: None,
        avatar_life: None,
        public_attacks: None,
        journal: None,
        classic_source: Some(source),
        dodge: None,
        player_life: None,
        lifecycle: None,
        dialogue: None,
        ready,
        commands: Some(commands),
        choices: None,
        combat: None,
        quest: None,
        progression: None,
        inventory: None,
        statuses: None,
    }
}
async fn drive(peers: &mut [Peer], duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {
        for peer in peers.iter_mut() {
            let state = peer.state.borrow().clone();
            assert!(state.error.is_none(), "{:?}", state.error);
            let focus = state
                .correction
                .expect("Admitted correction")
                .state
                .pose
                .feet;
            peer.terrain.poll(&[focus]);
            assert!(peer.terrain.error.is_none());
            let patch = peer.terrain.patch(focus).unwrap();
            if let Some(output) = peer
                .driver
                .tick(
                    &state,
                    Intent {
                        axes: [0, 1000],
                        facing: Some(Facing::default()),
                        ..Default::default()
                    },
                    Some(patch),
                    Instant::now(),
                )
                .unwrap()
                && let Some(packet) = output.packet
            {
                peer.commands.send(packet).await.unwrap();
            }
        }
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires THEIA_AUTHORED_WORLD local exported starter world with a clear +Z approach"]
async fn authored_classic_two_encrypted_peers_hold_until_ready_walk_and_reject_stale_world() {
    run_authored(false, false, false, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires THEIA_AUTHORED_WORLD valley and THEIA_AUTHORED_NAV west-grove package"]
async fn authored_valley_patrol_and_resource_combat_replicate_to_two_classic_peers() {
    run_authored(true, false, false, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires THEIA_AUTHORED_WORLD valley and THEIA_AUTHORED_NAV west-grove package"]
async fn authored_valley_full_xyz_guide_opens_and_closes_on_walkaway_for_two_encrypted_peers() {
    run_authored(true, true, false, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires THEIA_AUTHORED_WORLD bound village contact and THEIA_AUTHORED_NAV package"]
async fn authored_valley_exported_contact_binds_profile_and_placement_for_two_encrypted_peers() {
    run_authored(true, true, true, false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires THEIA_AUTHORED_WORLD local valley with clear village spawns"]
async fn authored_test_players_keep_four_client_spawns_in_reverse_arrival_and_refuse_unconfigured_peer()
 {
    run_authored(false, false, false, true).await;
}

async fn run_authored(patrol: bool, with_guide: bool, from_world: bool, test_players: bool) {
    let root =
        PathBuf::from(std::env::var_os("THEIA_AUTHORED_WORLD").expect("Authored starter export"));
    let store = Arc::new(Store::open(&root).unwrap());
    let source = store.identity();
    let [x, y, z] = store.spawn(0).unwrap().metres();
    let spawn = [x, z];
    let mut world = World::with_terrain_source(spawn, 1., Source::Authored(store)).unwrap();
    world.enable_prediction().unwrap();
    let catalog_bytes =
        include_bytes!("../../../content/gameplay/development-resource-abilities.json");
    let catalog = Arc::new(theia_core::abilities::Catalog::from_json(catalog_bytes).unwrap());
    let patrol_home = if patrol {
        use theia_network::{navigation, simulation::creatures};
        let path = PathBuf::from(std::env::var_os("THEIA_AUTHORED_NAV").unwrap());
        let nav = navigation::Store::open(&path, source).unwrap();
        let profile = creatures::Profile::from_json(include_bytes!(
            "../../../content/gameplay/starter-valley-wolf-patrol.json"
        ))
        .unwrap();
        let prepared =
            creatures::Prepared::load(profile, &nav, &world.procedural().unwrap().source).unwrap();
        world.enable_patrol(prepared).unwrap();
        world
            .enable_resource_combat(
                catalog_bytes,
                theia_network::simulation::combat::DEVELOPMENT_HEALTH,
                theia_network::simulation::combat::DEVELOPMENT_RESPAWN_MS,
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
            )
            .unwrap();
        Some(world.patrol_snapshot().unwrap().home)
    } else {
        None
    };
    let dialogue = with_guide.then(|| {
        let bytes = if from_world {
            theia_network::gameplay_bundle::snapshot(&root.join("gameplay/gameplay.manifest.json"))
                .unwrap()
                .profile("dialogue")
                .unwrap()
                .to_vec()
        } else {
            include_bytes!("../../../content/gameplay/development-dialogue.json").to_vec()
        };
        Arc::new(theia_network::dialogue::Profile::from_json(&bytes).unwrap())
    });
    let mut guide_requested_feet = None;
    let guide = dialogue.as_ref().map(|profile| {
        use theia_network::dialogue::Placement;
        let placement = if from_world {
            Placement::resolve("authored", &world.procedural().unwrap().source, profile).unwrap()
        } else {
            Placement::Feet(theia_core::coordinates::GlobalPosition::new([x + 2., y, z]).unwrap())
        };
        let Placement::Feet(feet) = placement else {
            panic!("Expected explicit authored feet")
        };
        guide_requested_feet = Some(feet.metres());
        world
            .spawn_placed_guide(profile.clone(), placement)
            .unwrap()
    });
    let classic_source = world.identity();
    world.enable_classic().unwrap();
    let identity = world.identity();
    if !patrol && !with_guide {
        assert_eq!(identity, theia_network::classic::profile_identity(source));
    }
    let prepared = if test_players {
        let bytes = serde_json::to_vec_pretty(
            &serde_json::json!({"schema":1,"world_identity":source,"players":[
                {"client_id":1,"feet":[x,y,z]}, {"client_id":2,"feet":[x-2.,y,z]},
                {"client_id":3,"feet":[x-4.,y,z]}, {"client_id":4,"feet":[x-6.,y,z]}
            ]}),
        )
        .unwrap();
        if let Some(path) = std::env::var_os("THEIA_TEST_PLAYERS_PROFILE") {
            std::fs::write(path, &bytes).unwrap();
        }
        Some(
            net::test_players::Plan::from_json(&bytes, &world)
                .unwrap()
                .prepare(&world)
                .unwrap(),
        )
    } else {
        None
    };
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let tickets = std::env::temp_dir().join(format!(
        "theia-authored-net-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let transport = net::issue_local(socket, &tickets).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let cleanup = Cleanup { stop, tickets };
    let server_shutdown = shutdown.clone();
    let server = tokio::spawn(async move {
        if let Some(prepared) = prepared {
            net::run_server_with_test_players(transport, server_shutdown, world, prepared).await
        } else {
            net::run_server_measured(transport, server_shutdown, world).await
        }
    });
    let mut peers = Vec::new();
    for i in if test_players {
        vec![4, 3, 2, 1]
    } else {
        vec![1, 2]
    } {
        let client_store = Arc::new(Store::open(&root).unwrap());
        assert_eq!(client_store.identity(), source);
        let terrain = Runtime::with_source(spawn, 1., Source::Authored(client_store)).unwrap();
        let ticket = std::fs::read(cleanup.tickets.join(format!("client-{i}.token"))).unwrap();
        let (input, inputs) = watch::channel(Intent::default());
        let (states, mut state) = watch::channel(State::default());
        let (ready, readiness) = watch::channel(None);
        let (commands, queue) = mpsc::channel(8);
        let mut mode = mode(readiness, queue, classic_source);
        let (choice, choices) = watch::channel(None);
        let (casts, requests) = mpsc::channel(8);
        let (events, event_rx) = mpsc::channel(16);
        let (dialogue_actions, dialogue_requests) = mpsc::channel(4);
        if let Some(profile) = &dialogue {
            mode.dialogue = Some(net::dialogue::Client {
                profile: profile.clone(),
                actions: dialogue_requests,
            });
        }
        if patrol {
            mode.choices = Some(choices);
            mode.combat = Some(net::CombatActions {
                melee: None,
                catalog: catalog.clone(),
                casts: requests,
                events,
            });
        }
        let shutdown = shutdown.clone();
        let task = tokio::spawn(async move {
            let result = net::run_client_gameplay(
                address,
                &ticket,
                inputs,
                states.clone(),
                shutdown,
                identity,
                mode,
            )
            .await;
            if let Err(error) = &result {
                states.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        tokio::time::timeout(Duration::from_secs(8), async {
            while state.borrow().correction.is_none() {
                assert!(state.borrow().error.is_none(), "{:?}", state.borrow().error);
                state.changed().await.unwrap();
            }
        })
        .await
        .expect("Authored admission timeout");
        let initial = state.borrow().correction.unwrap().state.pose.feet.metres();
        assert!(
            (initial[0]
                - (spawn[0]
                    - if test_players {
                        f64::from(i - 1) * 2.
                    } else {
                        0.
                    }))
            .abs()
                < 0.001
        );
        assert!((initial[2] - spawn[1]).abs() < 0.001);
        peers.push(Peer {
            state,
            ready,
            commands,
            _input: input,
            _choice: patrol.then_some(choice),
            _casts: patrol.then_some(casts),
            _events: patrol.then_some(event_rx),
            dialogue: with_guide.then_some(dialogue_actions),
            driver: Driver::with_classic(classic_source, false),
            terrain,
            initial,
            task,
        });
    }
    drive(&mut peers, Duration::from_millis(900)).await;
    if let Some(guide) = guide {
        for peer in &peers {
            let revision = {
                let state = peer.state.borrow();
                assert_eq!(state.guide.as_ref().unwrap().guide, Some(guide));
                state.dialogue.as_ref().unwrap().revision
            };
            peer.dialogue
                .as_ref()
                .unwrap()
                .send(net::dialogue::Intent {
                    expected_revision: revision,
                    action: theia_network::dialogue::Action::Open(guide.key),
                })
                .await
                .unwrap();
        }
        drive(&mut peers, Duration::from_millis(400)).await;
        for peer in &peers {
            let state = peer.state.borrow();
            assert_eq!(
                state
                    .dialogue
                    .as_ref()
                    .unwrap()
                    .open
                    .as_ref()
                    .unwrap()
                    .guide,
                guide.key
            );
            assert!(state.dialogue_reply.as_ref().unwrap().rejection.is_none());
        }
    }
    for peer in &peers {
        let state = peer.state.borrow();
        let correction = state.correction.unwrap().state;
        assert_eq!(correction.ready_sequence, 0);
        let p = correction.pose.feet.metres();
        assert!((p[0] - peer.initial[0]).abs() < 0.01 && (p[2] - peer.initial[2]).abs() < 0.01);
        assert_eq!(
            state.snapshot.as_ref().unwrap().players.len(),
            if test_players { 4 } else { 2 }
        );
        // Headless harness publishes collision readiness explicitly. The real
        // client uses its separate scene/material/GPU readiness gate.
        let patch = peer.terrain.patch(correction.pose.feet).unwrap();
        peer.ready
            .send_replace(Some(patch.ids.iter().copied().collect()));
    }
    drive(&mut peers, Duration::from_millis(2600)).await;
    for peer in &peers {
        let state = peer.state.borrow();
        let correction = state.correction.unwrap().state;
        assert!(correction.ready_sequence > 0);
        assert!(correction.pose.feet.metres()[2] > peer.initial[2] + 1.);
        assert!((correction.pose.feet.metres()[0] - peer.initial[0]).abs() < 0.02);
        assert!(peer.driver.stats.predicted_steps > 0 && peer.driver.stats.corrections > 0);
        assert_eq!(
            state.snapshot.as_ref().unwrap().players.len(),
            if test_players { 4 } else { 2 }
        );
        if with_guide {
            assert!(
                state.dialogue.as_ref().unwrap().open.is_none(),
                "Walkaway must close dialogue"
            );
        }
        if let Some(home) = patrol_home {
            let targeting = state
                .targeting
                .as_ref()
                .expect("Replicated patrol targeting");
            assert_eq!(targeting.targets.len(), 1);
            let position = targeting.targets[0].position.metres();
            assert!(
                (position[0] - home[0]).hypot(position[2] - home[2]) > 1.,
                "Patrol failed to move"
            );
            assert!(
                state.combat.is_some(),
                "Resource combat state must replicate"
            );
        }
    }
    let ticket = std::fs::read(cleanup.tickets.join(if test_players {
        "client-5.token"
    } else {
        "client-3.token"
    }))
    .unwrap();
    let (_input, input) = watch::channel(Intent::default());
    let (states, rejected_state) = watch::channel(State::default());
    let (_ready, ready) = watch::channel(None);
    let (_commands, queue) = mpsc::channel(8);
    let mut stale = source;
    if !test_players {
        stale[0] ^= 1;
    }
    let error = tokio::time::timeout(
        Duration::from_secs(8),
        net::run_client_gameplay(
            address,
            &ticket,
            input,
            states,
            shutdown,
            theia_network::classic::profile_identity(stale),
            mode(ready, queue, stale),
        ),
    )
    .await
    .unwrap()
    .unwrap_err();
    if test_players {
        assert!(rejected_state.borrow().correction.is_none());
    } else {
        assert!(error.to_string().contains("world mismatch"), "{error}");
    }
    cleanup.stop.send_replace(true);
    for peer in peers {
        peer.task.await.unwrap().unwrap();
    }
    let stats = server.await.unwrap().unwrap();
    if test_players {
        assert_eq!(stats.test_player_admissions, 4);
        assert_eq!(stats.test_player_refusals, 1);
        assert_eq!(stats.persistent_spawn_admissions, 0);
        assert_eq!(stats.rejected_inputs, 0);
    }
    if with_guide {
        assert_eq!(stats.dialogue_requests_accepted, 2);
        assert_eq!(stats.dialogue_requests_rejected, 0);
    }
    println!(
        "{}",
        serde_json::json!({"passed":true,"clients":2,"patrol":patrol,"guide":with_guide,
        "guide_requested_feet":guide_requested_feet,"authored_contact":from_world,"test_player_spawns":test_players,"reverse_arrival":test_players,"world":identity,"server":stats,
        "scope":"Actual authored Synty collision, encrypted loopback admission and Classic driver; explicit headless readiness, no native-client/quest/persistence claim"})
    );
}
