//! Real encrypted UDP regression for a deliberately lost first connection ACK.
#[path = "support/relay.rs"]
mod relay;
use std::{net::UdpSocket, time::Duration};
use theia_core::time::TickRate;
use theia_network::{client::State, renet_runtime as net, simulation::World, wire::Intent};
use tokio::sync::watch;

struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}

#[test]
fn seeded_loss_keeps_exact_block_budget_without_erasing_one_periodic_class() {
    for direction in [0, 1] {
        let drops: Vec<_> = (1..=1100)
            .map(|n| relay::loss_due(n, 11, Some(42), direction))
            .collect();
        assert!(
            drops
                .as_chunks::<11>()
                .0
                .iter()
                .all(|b| b.iter().filter(|&&v| v).count() == 1)
        );
        assert!(drops.windows(3).all(|w| w.iter().any(|&v| !v)));
        assert!((10..1100).step_by(11).any(|n| !drops[n]));
        assert_eq!(
            drops,
            (1..=1100)
                .map(|n| relay::loss_due(n, 11, Some(42), direction))
                .collect::<Vec<_>>()
        );
        assert_ne!(
            drops,
            (1..=1100)
                .map(|n| relay::loss_due(n, 11, Some(43), direction))
                .collect::<Vec<_>>()
        );
    }
    assert!((1..=1100).all(|n| relay::loss_due(n, 11, None, 0) == (n % 11 == 0)));
    assert!(!relay::loss_due(0, 11, Some(42), 0));
    assert!(!relay::loss_due(1, 0, Some(42), 0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn lost_first_keepalive_retries_before_world_admission_at_all_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let world = World::at_rate(rate);
        let identity = world.identity();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let backend = socket.local_addr().unwrap();
        let relay = relay::Relay::bind(relay::Profile::BASELINE);
        let address = relay.address;
        let control = relay.control.clone();
        control.lock().unwrap().drop_server_keepalives = 1;
        let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
        let ticket = issuer.issue(1, None, 60).unwrap().bytes().to_vec();
        let (stop, shutdown) = watch::channel(false);
        let guard = Stop(stop.clone());
        let relay = tokio::spawn(relay.run(backend, shutdown.clone()));
        let server = tokio::spawn(net::run_server_measured(transport, shutdown.clone(), world));
        let (_input, inputs) = watch::channel(Intent::default());
        let (publish, mut state) = watch::channel(State::default());
        let client = tokio::spawn(async move {
            net::run_client(address, &ticket, inputs, publish, shutdown, identity).await
        });
        // The normal client's internal five-second Welcome limit is unchanged.
        let admitted = tokio::time::timeout(Duration::from_secs(6), async {
            loop {
                let current = state.borrow().clone();
                if current.snapshot.is_some() {
                    break current;
                }
                state.changed().await.expect("Client closed before Welcome");
            }
        })
        .await
        .expect("Lost first KeepAlive prevented admission");
        assert!(admitted.error.is_none());
        assert!(admitted.welcome.is_some());
        assert_eq!(admitted.snapshot.as_ref().unwrap().players.len(), 1);
        assert_eq!(admitted.snapshot.as_ref().unwrap().tick_hz, rate.hz());
        stop.send_replace(true);
        client.await.unwrap().unwrap();
        let stats = server.await.unwrap().unwrap();
        relay.await.unwrap();
        assert_eq!(stats.connection_confirmation.confirmed, 1);
        assert_eq!(stats.connection_confirmation.rejected, 0);
        assert_eq!(stats.connection_confirmation.expired, 0);
        assert_eq!(stats.connection_confirmation.high_water, 1);
        let control = control.lock().unwrap();
        assert_eq!(control.stats.dropped_server_keepalives, 1);
        assert_eq!(control.stats.overflow, 0);
        drop(guard);
    }
}
