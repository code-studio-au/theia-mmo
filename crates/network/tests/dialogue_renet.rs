//! Actual encrypted loopback runtime with trusted local tickets. This is not an
//! OIDC/account, database or native presentation qualification.
use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use theia_core::{content::ContentId, coordinates::ChunkId};
use theia_network::{
    client::State,
    dialogue::{Action, Profile, Rejection},
    renet_runtime::{
        self as net,
        dialogue::{Client, Intent as DialogueIntent},
    },
    simulation::World,
    wire::Intent,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-dialogue.json");
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Tickets(PathBuf);
impl Drop for Tickets {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
struct Peer {
    state: watch::Receiver<State>,
    actions: mpsc::Sender<DialogueIntent>,
    input: watch::Sender<Intent>,
    stop: watch::Sender<bool>,
    task: JoinHandle<theia_network::Result<()>>,
    _ready: watch::Sender<Option<Vec<ChunkId>>>,
}
impl Peer {
    fn start(
        address: SocketAddr,
        ticket: Vec<u8>,
        identity: [u8; 32],
        profile: Arc<Profile>,
        chunks: Vec<ChunkId>,
    ) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(Some(chunks));
        let (actions, requests) = mpsc::channel(4);
        let (states, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            let result = net::run_client_gameplay(
                address,
                &ticket,
                inputs,
                states.clone(),
                shutdown,
                identity,
                net::GameplayClient {
                    dialogue: Some(Client {
                        profile,
                        actions: requests,
                    }),
                    ready: readiness,
                    commands: None,
                    choices: None,
                    combat: None,
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
            actions,
            input,
            stop,
            task,
            _ready: ready,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> State {
        tokio::time::timeout(Duration::from_secs(6), async {
            loop {
                let state = self.state.borrow().clone();
                assert!(state.error.is_none(), "{:?}", state.error);
                if predicate(&state) {
                    return state;
                }
                self.state.changed().await.unwrap();
            }
        })
        .await
        .unwrap_or_else(|_| panic!("Dialogue observation timed out: {:?}", self.state.borrow()))
    }
    async fn action(&self, revision: u64, action: Action) {
        self.actions
            .send(DialogueIntent {
                expected_revision: revision,
                action,
            })
            .await
            .unwrap();
    }
    async fn reply(&mut self, sequence: u64, revision: u64) -> State {
        self.until(|s| {
            !s.dialogue_pending
                && s.dialogue_reply
                    .as_ref()
                    .is_some_and(|r| r.sequence == sequence)
                && s.dialogue.as_ref().is_some_and(|v| v.revision == revision)
        })
        .await
    }
    async fn finish(self) {
        self.stop.send_replace(true);
        tokio::time::timeout(Duration::from_secs(3), self.task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn encrypted_clients_keep_independent_dialogue_and_acknowledge_refusals_and_walkaway() {
    let profile = Arc::new(Profile::from_json(DATA).unwrap());
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    let guide = world
        .spawn_development_guide(profile.clone(), [8064., 8000.])
        .unwrap();
    let identity = world.identity();
    let chunks: Vec<_> = theia_network::procedural::neighbours(ChunkId { x: 62, z: 62 }, 1)
        .into_iter()
        .collect();
    let directory = Tickets(std::env::temp_dir().join(format!(
        "theia-dialogue-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &directory.0).unwrap();
    let token = |id| fs::read(directory.0.join(format!("client-{id}.token"))).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Peer::start(address, token(1), identity, profile.clone(), chunks.clone());
    let mut b = Peer::start(address, token(2), identity, profile, chunks);
    let ready = |s: &State| {
        s.dialogue.is_some()
            && s.guide.as_ref().is_some_and(|p| p.guide == Some(guide))
            && s.chunk_ack.is_some()
            && s.snapshot.as_ref().is_some_and(|v| v.players.len() == 2)
    };
    let initial_a = a.until(ready).await;
    let initial_b = b.until(ready).await;
    assert_ne!(
        initial_a.dialogue.as_ref().unwrap().actor,
        initial_b.dialogue.as_ref().unwrap().actor
    );
    assert_ne!(initial_a.dialogue.as_ref().unwrap().actor, guide.key);
    a.action(0, Action::Open(guide.key)).await;
    let state = a.reply(1, 1).await;
    assert!(state.dialogue_reply.unwrap().rejection.is_none());
    assert_eq!(
        state.dialogue.unwrap().open.unwrap().node.as_str(),
        "theia.dev.guide.welcome"
    );
    assert!(b.state.borrow().dialogue.as_ref().unwrap().open.is_none());
    b.action(0, Action::Open(guide.key)).await;
    b.reply(1, 1).await;
    b.action(1, Action::Choose(id("theia.dev.guide.ask-training")))
        .await;
    let state = b.reply(2, 2).await;
    assert_eq!(
        state.dialogue.unwrap().open.unwrap().node.as_str(),
        "theia.dev.guide.training"
    );
    a.action(1, Action::Choose(id("theia.dev.guide.finish-training")))
        .await;
    let state = a.reply(2, 1).await;
    assert_eq!(
        state.dialogue_reply.unwrap().rejection,
        Some(Rejection::InvalidAction)
    );
    assert_eq!(
        state.dialogue.unwrap().open.unwrap().node.as_str(),
        "theia.dev.guide.welcome"
    );
    a.action(1, Action::Choose(id("theia.dev.guide.ask-movement")))
        .await;
    let state = a.reply(3, 2).await;
    assert_eq!(
        state.dialogue.unwrap().open.unwrap().node.as_str(),
        "theia.dev.guide.movement"
    );
    a.action(2, Action::Close).await;
    assert!(a.reply(4, 3).await.dialogue.unwrap().open.is_none());
    a.action(2, Action::Open(guide.key)).await;
    let state = a.reply(5, 3).await;
    assert_eq!(
        state.dialogue_reply.unwrap().rejection,
        Some(Rejection::StaleRevision)
    );
    assert!(state.dialogue.unwrap().open.is_none());
    a.action(3, Action::Open(guide.key)).await;
    a.reply(6, 4).await;
    a.input.send_replace(Intent {
        axes: [-1000, 0],
        sprint: true,
        jump: false,
    });
    let left = a
        .until(|s| {
            s.dialogue
                .as_ref()
                .is_some_and(|v| v.revision == 5 && v.open.is_none())
        })
        .await;
    a.input.send_replace(Intent::default());
    assert!(
        left.guide.unwrap().guide.is_some(),
        "Drawing interest is wider than interaction range"
    );
    assert_eq!(b.state.borrow().dialogue.as_ref().unwrap().revision, 2);
    assert_eq!(
        b.state
            .borrow()
            .dialogue
            .as_ref()
            .unwrap()
            .open
            .as_ref()
            .unwrap()
            .node
            .as_str(),
        "theia.dev.guide.training"
    );
    assert!(left.inventory.is_none() && left.progression.is_none() && left.quest.is_none());
    assert!(left.transport.unwrap().dialogue_views_received >= 5);
    a.finish().await;
    b.finish().await;
    drop(guard);
    let stats = tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(stats.dialogue_requests_accepted, 6);
    assert_eq!(stats.dialogue_requests_rejected, 2);
    assert_eq!(stats.dialogue_replies_sent, 8);
    assert!(stats.guide_presences_sent >= 2 && stats.dialogue_views_sent >= 9);
    assert_eq!(stats.dialogue_backpressure_disconnects, 0);
    assert!(stats.max_event_queue_bytes <= 8192);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn malformed_and_oversized_interaction_frames_disconnect_actual_encrypted_peers() {
    use theia_network::{dialogue_wire, renet_transport};
    let profile = Arc::new(Profile::from_json(DATA).unwrap());
    let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
    world
        .spawn_development_guide(profile.clone(), [8064., 8000.])
        .unwrap();
    let identity = world.identity();
    let directory = Tickets(std::env::temp_dir().join(format!(
        "theia-dialogue-malformed-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let transport = net::issue_local(socket, &directory.0).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let guard = Stop(stop);
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut oversized = dialogue_wire::REQUEST_TAG.to_vec();
    oversized.resize(dialogue_wire::MAX_REQUEST_BYTES + 1, 0);
    for (index, payload) in [dialogue_wire::REQUEST_TAG.to_vec(), oversized]
        .into_iter()
        .enumerate()
    {
        let ticket = fs::read(directory.0.join(format!("client-{}.token", index + 1))).unwrap();
        let token = net::read_ticket(&ticket, address).unwrap();
        let mut transport = renet_transport::secure_client(
            UdpSocket::bind("127.0.0.1:0").unwrap(),
            net::now().unwrap(),
            token,
        )
        .unwrap();
        let mut client = renet::RenetClient::new(renet_transport::connection_config());
        let mut last = std::time::Instant::now();
        let mut admitted = false;
        let mut sent = false;
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let now = std::time::Instant::now();
                let delta = now.saturating_duration_since(last);
                last = now;
                client.update(delta);
                if transport.update(delta, &mut client).is_err() || client.is_disconnected() {
                    assert!(
                        admitted && sent,
                        "Disconnected before the malformed request"
                    );
                    break;
                }
                while let Some(bytes) = client.receive_message(renet_transport::EVENT_CHANNEL) {
                    if !admitted {
                        theia_network::wire::Welcome::decode(&bytes, identity).unwrap();
                        admitted = true;
                    } else if bytes.starts_with(dialogue_wire::VIEW_TAG) {
                        let view = dialogue_wire::decode_view(&bytes, &profile).unwrap();
                        assert_eq!(view.revision, 0);
                        assert!(view.open.is_none());
                    }
                }
                while client
                    .receive_message(renet_transport::MOVEMENT_CHANNEL)
                    .is_some()
                {}
                if admitted && !sent {
                    client.send_message(renet_transport::EVENT_CHANNEL, payload.clone());
                    sent = true;
                }
                if transport.send_packets(&mut client).is_err() {
                    assert!(
                        admitted && sent,
                        "Disconnected before the malformed request"
                    );
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("Malformed encrypted peer was not disconnected");
    }
    drop(guard);
    let stats = tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(stats.dialogue_requests_accepted, 0);
    assert_eq!(stats.dialogue_requests_rejected, 2);
    assert_eq!(stats.dialogue_replies_sent, 0);
}
