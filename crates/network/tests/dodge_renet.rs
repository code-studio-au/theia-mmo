//! Real secure Renet UDP carrying movement profile 2. No native client claim.
#[path = "support/relay.rs"]
#[allow(dead_code)]
mod relay;
use std::{net::UdpSocket, sync::Arc, time::Duration};
use theia_core::time::TickRate;
use theia_network::{
    client::State,
    commands::Command,
    dodging::{self, Profile, Request},
    prediction_wire::Commands,
    procedural,
    renet_runtime::{self as net, DodgeClient, GameplayClient},
    simulation::World,
    wire::Intent,
};
use tokio::{
    sync::{mpsc, watch},
    time::{Instant, sleep},
};
fn policy() -> Arc<Profile> {
    Arc::new(
        Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-dodge.json"
        ))
        .unwrap(),
    )
}
fn mode(
    ready: net::ChunkReadiness,
    commands: mpsc::Receiver<Commands>,
    dodge: Option<DodgeClient>,
) -> GameplayClient {
    GameplayClient {
        classic_source: None,
        dodge,
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
async fn until(state: &mut watch::Receiver<State>, predicate: impl Fn(&State) -> bool) -> State {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let s = state.borrow().clone();
            assert!(s.error.is_none(), "{:?}", s.error);
            if predicate(&s) {
                return s;
            }
            state
                .changed()
                .await
                .expect("Client ended before acceptance");
        }
    })
    .await
    .expect("Dodge UDP acceptance timeout")
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn encrypted_impaired_dodge_commands_and_corrections_preserve_distance_nonce_and_cooldown_at_all_rates()
 {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        run_dodge(rate, false).await;
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_driver_and_expiring_single_tap_dodge_work_over_encrypted_impaired_udp_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        run_dodge(rate, true).await;
    }
}
async fn run_dodge(rate: TickRate, drive: bool) {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    assert!(world.set_tick_rate(rate));
    world.enable_prediction().unwrap();
    let source = world.identity();
    let p = policy();
    world.enable_dodge(p.clone()).unwrap();
    let expected = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let backend = socket.local_addr().unwrap();
    let relay = relay::Relay::bind(relay::Profile {
        name: "dodge-adverse",
        delay_ms: 12,
        jitter_ms: 8,
        loss_every: 9,
        duplicate_every: 7,
    });
    let address = relay.address;
    let control = relay.control.clone();
    let directory = std::env::temp_dir().join(format!(
        "theia-dodge-udp-{}-{}-{drive}",
        std::process::id(),
        rate.hz()
    ));
    let transport = net::issue_local_advertised(socket, &directory, vec![address]).unwrap();
    let ticket = std::fs::read(directory.join("client-1.token")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let relay_task = tokio::spawn(relay.run(backend, shutdown.clone()));
    let server = tokio::spawn(net::run_server_measured(transport, shutdown.clone(), world));
    let (_input, input) = watch::channel(Intent::default());
    let (state_tx, mut state) = watch::channel(State::default());
    let (ready, readiness) = watch::channel(None);
    let (commands, queue) = mpsc::channel(8);
    let client = tokio::spawn(async move {
        net::run_client_gameplay(
            address,
            &ticket,
            input,
            state_tx,
            shutdown,
            expected,
            mode(
                readiness,
                queue,
                Some(DodgeClient {
                    source_world: source,
                    profile: p,
                }),
            ),
        )
        .await
    });
    let first = until(&mut state, |s| s.correction.is_some()).await;
    let initial = first.correction.unwrap().state;
    assert_eq!(initial.world, dodging::profile_identity(source, &policy()));
    assert_eq!(initial.rate, rate);
    ready.send_replace(Some(
        procedural::neighbours(procedural::owner(initial.pose.feet).unwrap(), 1)
            .into_iter()
            .collect(),
    ));
    let anchor = until(&mut state, |s| {
        s.correction
            .is_some_and(|c| c.state.ready_sequence > 0 && c.state.pose.motion.grounded)
    })
    .await
    .correction
    .unwrap()
    .state;
    let deadline = Instant::now() + Duration::from_secs(6);
    let patch = procedural::Patch::build(procedural::owner(anchor.pose.feet).unwrap());
    let mut driver = theia_network::prediction_driver::Driver::with_dodge(source, policy(), false);
    let mut edge = theia_network::dodge_input::Input::new(source, policy());
    let began = std::time::Instant::now();
    let mut pressed = false;
    let mut sequence = 0;
    let mut sent_tick = 0;
    let mut ready_at = None;
    loop {
        assert!(Instant::now() < deadline, "Dodge did not complete");
        let c = state.borrow().correction.unwrap().state;
        assert!(
            c.pose
                .dodge
                .unwrap()
                .valid_at(&policy(), rate, c.input.tick)
        );
        if let Some(d) = c.pose.dodge.filter(|d| d.last_nonce == 1) {
            ready_at.get_or_insert(d.policy.ready_at);
            assert_eq!(Some(d.policy.ready_at), ready_at);
            if c.input.tick > d.policy.ready_at + u64::from(rate.hz() / 3) {
                break;
            }
        }
        if drive {
            let now = std::time::Instant::now();
            let tap = (!pressed && now.duration_since(began) > Duration::from_millis(300))
                .then_some([1000, 0]);
            pressed |= tap.is_some();
            let request = edge.update(Some(c), true, tap, now).unwrap();
            let raw = state.borrow().clone();
            if let Some(driven) = driver
                .tick(
                    &raw,
                    Intent {
                        facing: None,
                        dodge: request,
                        ..Default::default()
                    },
                    Some(&patch),
                    now,
                )
                .unwrap()
                && let Some(packet) = driven.packet
            {
                commands.send(packet).await.unwrap();
            }
        }
        let tick = c.input.tick + 4;
        if !drive && tick > sent_tick {
            sequence += 1;
            sent_tick = tick;
            commands
                .send(Commands {
                    epoch: c.actor.epoch.value(),
                    stream_epoch: c.input.stream_epoch,
                    ready_sequence: c.ready_sequence,
                    commands: vec![Command {
                        sequence,
                        tick,
                        intent: Intent {
                            facing: None,
                            dodge: Some(Request {
                                nonce: 1,
                                axes: [1000, 0],
                            }),
                            ..Default::default()
                        },
                    }],
                })
                .await
                .unwrap();
        }
        sleep(Duration::from_millis(if drive { 8 } else { 15 })).await;
    }
    let final_pose = state.borrow().correction.unwrap().state.pose;
    assert!(final_pose.dodge.unwrap().policy.active.is_none());
    assert_eq!(final_pose.dodge.unwrap().last_nonce, 1);
    assert!((final_pose.feet.metres()[0] - anchor.pose.feet.metres()[0] - 3.).abs() < 0.002);
    stop.send_replace(true);
    client.await.unwrap().unwrap();
    let stats = server.await.unwrap().unwrap();
    relay_task.await.unwrap();
    assert!(stats.queued_commands > 0);
    assert_eq!(stats.rejected_inputs, 0);
    assert!(stats.max_input_batch <= 32);
    let stats = control.lock().unwrap().stats.clone();
    assert!(stats.configured_drops > 0);
    assert!(stats.duplicates > 0);
    assert_eq!(stats.overflow, 0);
    std::fs::remove_dir_all(directory).unwrap();
}
#[tokio::test]
async fn unbound_client_policy_is_refused_before_connecting() {
    let (_input, input) = watch::channel(Intent::default());
    let (state_tx, _state) = watch::channel(State::default());
    let (_stop, shutdown) = watch::channel(false);
    let (_ready, readiness) = watch::channel(None);
    let (_commands, queue) = mpsc::channel(8);
    let error = net::run_client_gameplay(
        "127.0.0.1:1".parse().unwrap(),
        &[],
        input,
        state_tx,
        shutdown,
        [0; 32],
        mode(
            readiness,
            queue,
            Some(DodgeClient {
                source_world: procedural::IDENTITY,
                profile: policy(),
            }),
        ),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("Unbound dodge transport"));
}

#[tokio::test]
async fn legacy_prediction_client_cannot_silently_accept_profile_two_corrections() {
    let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
    world.enable_prediction().unwrap();
    world.enable_dodge(policy()).unwrap();
    let expected = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let directory = std::env::temp_dir().join(format!("theia-dodge-legacy-{}", std::process::id()));
    let transport = net::issue_local(socket, &directory).unwrap();
    let ticket = std::fs::read(directory.join("client-1.token")).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let server = tokio::spawn(net::run_server_measured(transport, shutdown.clone(), world));
    let (_input, input) = watch::channel(Intent::default());
    let (state_tx, _state) = watch::channel(State::default());
    let (_ready, readiness) = watch::channel(None);
    let (_commands, queue) = mpsc::channel(8);
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        net::run_client_gameplay(
            address,
            &ticket,
            input,
            state_tx,
            shutdown,
            expected,
            mode(readiness, queue, None),
        ),
    )
    .await
    .unwrap();
    assert!(result.is_err());
    stop.send_replace(true);
    server.await.unwrap().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}
