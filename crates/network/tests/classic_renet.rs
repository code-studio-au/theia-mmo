//! Actual encrypted Renet + driver through loss/jitter/duplication; headless only.
#[path = "support/relay.rs"]
#[allow(dead_code)]
mod relay;
use std::{net::UdpSocket, time::Duration};
use theia_core::{facing::Facing, time::TickRate};
use theia_network::{
    client::State,
    prediction_driver::Driver,
    procedural,
    renet_runtime::{self as net, GameplayClient},
    simulation::World,
    wire::Intent,
};
use tokio::{
    sync::{mpsc, watch},
    time::sleep,
};
fn mode(
    ready: net::ChunkReadiness,
    commands: mpsc::Receiver<theia_network::prediction_wire::Commands>,
    source: [u8; 32],
) -> GameplayClient {
    GameplayClient {
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
    .expect("Classic UDP acceptance timeout")
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn independent_facing_and_strafing_use_actual_driver_over_encrypted_impaired_udp_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        assert!(world.set_tick_rate(rate));
        world.enable_prediction().unwrap();
        let target = world.spawn_development_target([8004., 8004.]).unwrap();
        let source = world.identity();
        world.enable_classic().unwrap();
        let expected = world.identity();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let relay = relay::Relay::bind(relay::Profile {
            name: "classic-adverse",
            delay_ms: 12,
            jitter_ms: 8,
            loss_every: 9,
            duplicate_every: 7,
        });
        let address = relay.address;
        let control = relay.control.clone();
        let directory = std::env::temp_dir().join(format!(
            "theia-classic-udp-{}-{}",
            std::process::id(),
            rate.hz()
        ));
        let transport = net::issue_local_advertised(socket, &directory, vec![address]).unwrap();
        let ticket = std::fs::read(directory.join("client-1.token")).unwrap();
        let (stop, shutdown) = watch::channel(false);
        let relay_task = tokio::spawn(relay.run(backend, shutdown.clone()));
        let (owner, requests) = net::world_control();
        let server = tokio::spawn(net::run_server_with_world_control(
            transport,
            shutdown.clone(),
            world,
            requests,
        ));
        let (_input, input) = watch::channel(Intent::default());
        let (tx, mut state) = watch::channel(State::default());
        let (ready, readiness) = watch::channel(None);
        let (commands, queue) = mpsc::channel(8);
        let (choices, choice_queue) = watch::channel(None);
        let client = tokio::spawn(async move {
            let mut configuration = mode(readiness, queue, source);
            configuration.choices = Some(choice_queue);
            net::run_client_gameplay(
                address,
                &ticket,
                input,
                tx,
                shutdown,
                expected,
                configuration,
            )
            .await
        });
        let initial = until(&mut state, |s| s.correction.is_some())
            .await
            .correction
            .unwrap()
            .state;
        assert_eq!(initial.pose.facing, Some(Facing::default()));
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
        choices.send_replace(Some(theia_network::targeting::Choice {
            target: Some(target.entity),
        }));
        until(&mut state, |s| {
            s.targeting
                .as_ref()
                .is_some_and(|v| v.selected == Some(target.entity))
        })
        .await;
        let patch = procedural::Patch::build(procedural::owner(anchor.pose.feet).unwrap());
        let mut driver = Driver::with_classic(source, false);
        let began = std::time::Instant::now();
        while began.elapsed() < Duration::from_millis(2600) {
            let now = std::time::Instant::now();
            let seconds = now.duration_since(began).as_secs_f64();
            let facing = Facing::from_units(if seconds < 1.5 { 16384 } else { 49152 });
            let axes = facing
                .world_axes(if (0.5..1.1).contains(&seconds) {
                    [1000, 0]
                } else {
                    [0, 0]
                })
                .unwrap();
            let raw = state.borrow().clone();
            assert!(raw.error.is_none(), "{:?}", raw.error);
            if let Some(driven) = driver
                .tick(
                    &raw,
                    Intent {
                        facing: Some(facing),
                        sprint: seconds >= 0.8, // Switch gait during impaired movement.
                        axes,
                        ..Intent::default()
                    },
                    Some(&patch),
                    now,
                )
                .unwrap()
                && let Some(packet) = driven.packet
            {
                commands.send(packet).await.unwrap();
            }
            sleep(Duration::from_millis(8)).await;
        }
        let final_state = state.borrow().clone();
        let final_pose = final_state.correction.unwrap().state.pose;
        assert_eq!(final_pose.facing, Some(Facing::from_units(49152)));
        assert!((final_pose.feet.metres()[0] - anchor.pose.feet.metres()[0]).abs() < 0.01);
        assert!(final_pose.feet.metres()[2] < anchor.pose.feet.metres()[2] - 0.5);
        assert_eq!(
            final_state.snapshot.unwrap().players[0].facing,
            final_pose.facing
        );
        assert!(driver.stats.predicted_steps > 0 && driver.stats.corrections > 0);
        assert_eq!(
            final_state.targeting.as_ref().unwrap().selected,
            Some(target.entity)
        );
        choices.send_replace(Some(theia_network::targeting::Choice { target: None }));
        until(&mut state, |s| {
            s.targeting.as_ref().is_some_and(|v| v.selected.is_none())
        })
        .await;
        choices.send_replace(Some(theia_network::targeting::Choice {
            target: Some(target.entity),
        }));
        until(&mut state, |s| {
            s.targeting
                .as_ref()
                .is_some_and(|v| v.selected == Some(target.entity))
        })
        .await;
        assert!(owner.remove_target(target).await.unwrap());
        until(&mut state, |s| {
            s.targeting
                .as_ref()
                .is_some_and(|v| v.selected.is_none() && v.targets.is_empty())
        })
        .await;
        stop.send_replace(true);
        client.await.unwrap().unwrap();
        let stats = server.await.unwrap().unwrap();
        relay_task.await.unwrap();
        assert!(stats.queued_commands > 0);
        assert_eq!(stats.rejected_inputs, 0);
        assert!(stats.max_input_batch <= 32);
        let stats = control.lock().unwrap().stats.clone();
        assert!(stats.configured_drops > 0 && stats.duplicates > 0);
        assert_eq!(stats.overflow, 0);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
#[tokio::test]
async fn foreign_classic_binding_is_rejected_before_network_io() {
    let (_input, input) = watch::channel(Intent::default());
    let (state, _) = watch::channel(State::default());
    let (_stop, shutdown) = watch::channel(false);
    let (_ready, ready) = watch::channel(None);
    let (_commands, commands) = mpsc::channel(8);
    let error = net::run_client_gameplay(
        "127.0.0.1:1".parse().unwrap(),
        &[],
        input,
        state,
        shutdown,
        [0; 32],
        mode(ready, commands, procedural::IDENTITY),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("Unbound classic"));
}
