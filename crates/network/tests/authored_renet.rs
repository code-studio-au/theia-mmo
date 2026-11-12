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
    let root =
        PathBuf::from(std::env::var_os("THEIA_AUTHORED_WORLD").expect("Authored starter export"));
    let store = Arc::new(Store::open(&root).unwrap());
    let source = store.identity();
    let [x, _, z] = store.spawn(0).unwrap().metres();
    let spawn = [x, z];
    let mut world = World::with_terrain_source(spawn, 1., Source::Authored(store)).unwrap();
    world.enable_prediction().unwrap();
    world.enable_classic().unwrap();
    let identity = world.identity();
    assert_eq!(identity, theia_network::classic::profile_identity(source));
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
    let server = tokio::spawn(net::run_server_measured(transport, shutdown.clone(), world));
    let mut peers = Vec::new();
    for i in 1..=2 {
        let client_store = Arc::new(Store::open(&root).unwrap());
        assert_eq!(client_store.identity(), source);
        let terrain = Runtime::with_source(spawn, 1., Source::Authored(client_store)).unwrap();
        let ticket = std::fs::read(cleanup.tickets.join(format!("client-{i}.token"))).unwrap();
        let (input, inputs) = watch::channel(Intent::default());
        let (states, mut state) = watch::channel(State::default());
        let (ready, readiness) = watch::channel(None);
        let (commands, queue) = mpsc::channel(8);
        let shutdown = shutdown.clone();
        let task = tokio::spawn(async move {
            net::run_client_gameplay(
                address,
                &ticket,
                inputs,
                states,
                shutdown,
                identity,
                mode(readiness, queue, source),
            )
            .await
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
        assert!((initial[0] - spawn[0]).abs() < 0.001);
        assert!((initial[2] - spawn[1]).abs() < 0.001);
        peers.push(Peer {
            state,
            ready,
            commands,
            _input: input,
            driver: Driver::with_classic(source, false),
            terrain,
            initial,
            task,
        });
    }
    drive(&mut peers, Duration::from_millis(900)).await;
    for peer in &peers {
        let state = peer.state.borrow();
        let correction = state.correction.unwrap().state;
        assert_eq!(correction.ready_sequence, 0);
        let p = correction.pose.feet.metres();
        assert!((p[0] - peer.initial[0]).abs() < 0.01 && (p[2] - peer.initial[2]).abs() < 0.01);
        assert_eq!(state.snapshot.as_ref().unwrap().players.len(), 2);
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
        assert_eq!(state.snapshot.as_ref().unwrap().players.len(), 2);
    }
    let ticket = std::fs::read(cleanup.tickets.join("client-3.token")).unwrap();
    let (_input, input) = watch::channel(Intent::default());
    let (states, _state) = watch::channel(State::default());
    let (_ready, ready) = watch::channel(None);
    let (_commands, queue) = mpsc::channel(8);
    let mut stale = source;
    stale[0] ^= 1;
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
    assert!(error.to_string().contains("world mismatch"), "{error}");
    cleanup.stop.send_replace(true);
    for peer in peers {
        peer.task.await.unwrap().unwrap();
    }
    let stats = server.await.unwrap().unwrap();
    println!(
        "{}",
        serde_json::json!({"passed":true,"clients":2,"world":identity,"server":stats,
        "scope":"Actual authored Synty collision, encrypted loopback admission and Classic driver; explicit headless readiness, no native-client/quest/persistence claim"})
    );
}
