//! Standalone real encrypted UDP endpoint-reuse/replay qualification. No renderer,
//! account provider, persistence or population-capacity claim. Never logs tickets.
#[path = "../tests/support/relay.rs"]
#[allow(dead_code)]
mod relay;
use serde_json::{Value, json};
use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};
use theia_core::time::TickRate;
use theia_network::{
    Result,
    client::State,
    renet_runtime as net,
    simulation::World,
    wire::{Intent, Player},
};
use tokio::{sync::watch, task::JoinHandle, time::Instant};
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
struct Peer {
    input: watch::Sender<Intent>,
    state: watch::Receiver<State>,
    stop: watch::Sender<bool>,
    task: JoinHandle<Result<()>>,
}
impl Peer {
    fn start(address: SocketAddr, ticket: Vec<u8>, world: [u8; 32]) -> Self {
        let (input, inputs) = watch::channel(Intent::default());
        let (publish, state) = watch::channel(State::default());
        let (stop, shutdown) = watch::channel(false);
        let task = tokio::spawn(async move {
            let result =
                net::run_client(address, &ticket, inputs, publish.clone(), shutdown, world).await;
            if let Err(error) = &result {
                publish.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        Self {
            input,
            state,
            stop,
            task,
        }
    }
    async fn until(&mut self, predicate: impl Fn(&State) -> bool) -> Result<State> {
        tokio::time::timeout(Duration::from_secs(6), async {
            loop {
                let state = self.state.borrow().clone();
                if let Some(error) = &state.error {
                    return Err(error.clone().into());
                }
                if predicate(&state) {
                    return Ok(state);
                }
                self.state
                    .changed()
                    .await
                    .map_err(|_| "Session state closed before acceptance")?;
            }
        })
        .await
        .map_err(|_| "Qualification state wait timed out")?
    }
    async fn finish(self) -> Result<()> {
        self.stop.send_replace(true);
        tokio::time::timeout(Duration::from_secs(3), self.task)
            .await
            .map_err(|_| "Client cleanup timed out")??
    }
}
fn actor(state: &State, id: u64) -> Option<&Player> {
    state.snapshot.as_ref()?.players.iter().find(|p| p.id == id)
}
fn two_players(state: &State) -> bool {
    state
        .snapshot
        .as_ref()
        .is_some_and(|s| s.players.len() == 2)
}
fn peer_summary(peer: &Option<Peer>) -> Option<Value> {
    peer.as_ref().map(|p| {
        let state = p.state.borrow();
        json!({
            "id":state.id,
            "welcome":state.welcome.is_some(),
            "snapshot_tick":state.snapshot.as_ref().map(|s|s.tick),
            "snapshot_players":state.snapshot.as_ref().map(|s|s.players.iter().map(|p|p.id).collect::<Vec<_>>()),
            "error":state.error
        })
    })
}
async fn case(
    rate: TickRate,
    profile: relay::Profile,
    reuse: bool,
    round: u32,
    drop_ack: bool,
    periodic_loss: bool,
) -> Result<Value> {
    let world = World::at_rate(rate);
    let identity = world.identity();
    let relays: Vec<_> = (0..3).map(|_| relay::Relay::bind(profile)).collect();
    let addresses: Vec<_> = relays.iter().map(|r| r.address).collect();
    let controls: Vec<_> = relays.iter().map(|r| r.control.clone()).collect();
    for control in &controls {
        let mut control = control.lock().unwrap();
        control.drop_server_keepalives = u32::from(drop_ack);
        control.loss_seed = (!periodic_loss).then_some(0x5448_4549_4100_0001);
    }
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let backend = socket.local_addr()?;
    let advertised = if reuse {
        addresses[..2].to_vec()
    } else {
        addresses.clone()
    };
    let (transport, issuer) = net::secure_issuer(socket, advertised)?;
    let (stop, shutdown) = watch::channel(false);
    let _guard = Stop(stop.clone());
    let workers: Vec<_> = relays
        .into_iter()
        .map(|r| tokio::spawn(r.run(backend, shutdown.clone())))
        .collect();
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let mut a = Some(Peer::start(
        addresses[0],
        issuer.issue(1, None, 120)?.bytes().to_vec(),
        identity,
    ));
    let mut b = Some(Peer::start(
        addresses[1],
        issuer.issue(2, None, 120)?.bytes().to_vec(),
        identity,
    ));
    let mut replacement = None;
    let mut phase = "initial_admission";
    let mut replacement_ms = None;
    let scenario: Result<Value> = async {
        let initial = a.as_mut().unwrap().until(two_players).await?;
        let old = initial.id.ok_or("Missing original actor")?;
        let start = actor(&initial, old).ok_or("Original actor missing")?.xz;
        b.as_mut().unwrap().until(two_players).await?;

        phase = "original_movement";
        a.as_ref().unwrap().input.send_replace(Intent {
            axes: [1000, 0],
            ..Default::default()
        });
        b.as_mut()
            .unwrap()
            .until(|s| actor(s, old).is_some_and(|p| p.xz[0] > start[0] + 1.))
            .await?;
        let captured: Vec<_> = controls[0]
            .lock()
            .unwrap()
            .captures
            .iter()
            .cloned()
            .collect();

        phase = "original_disconnect";
        a.take().unwrap().finish().await?;
        b.as_mut()
            .unwrap()
            .until(|s| s.snapshot.as_ref().is_some_and(|s| s.players.len() == 1))
            .await?;

        phase = "replacement_welcome";
        let ticket = issuer.issue(3, None, 120)?;
        let address = ticket.address();
        let index = if reuse { 0 } else { 2 };
        if address != addresses[index] {
            return Err("Replacement ticket relay does not match requested endpoint case".into());
        }
        let began = Instant::now();
        replacement = Some(Peer::start(address, ticket.bytes().to_vec(), identity));
        let fresh = replacement.as_mut().unwrap().until(two_players).await?;
        replacement_ms = Some(began.elapsed().as_secs_f64() * 1000.);
        let new = fresh.id.ok_or("Missing replacement actor")?;
        if new <= old {
            return Err("Replacement reused the original actor incarnation".into());
        }
        let pose = actor(&fresh, new).ok_or("Replacement body missing")?.xz;

        phase = "old_packet_injection";
        {
            let mut control = controls[index].lock().unwrap();
            control.stats.injected_old_packets += captured.len() as u64;
            control.inject.extend(captured.iter().cloned());
        }
        let tick = fresh.snapshot.as_ref().unwrap().tick + u64::from(rate.hz() / 2);
        let after = replacement
            .as_mut()
            .unwrap()
            .until(|s| s.snapshot.as_ref().is_some_and(|s| s.tick >= tick))
            .await?;
        let now = actor(&after, new)
            .ok_or("Injected traffic removed replacement")?
            .xz;
        if (now[0] - pose[0]).hypot(now[1] - pose[1]) > 0.001 {
            return Err("Old encrypted traffic moved the replacement".into());
        }
        b.as_mut()
            .unwrap()
            .until(|s| actor(s, old).is_none() && actor(s, new).is_some())
            .await?;
        Ok(json!({
            "old_actor": old,
            "replacement_actor": new,
            "old_packets_injected": captured.len(),
            "replacement_pose_unchanged": true
        }))
    }
    .await;
    let replacement_error = replacement
        .as_ref()
        .and_then(|p| p.state.borrow().error.clone());
    let client_states = json!({
        "original":peer_summary(&a),
        "observer":peer_summary(&b),
        "replacement":peer_summary(&replacement)
    });
    let mut cleanup_errors = Vec::new();
    for peer in [a.take(), b.take(), replacement.take()]
        .into_iter()
        .flatten()
    {
        if let Err(error) = peer.finish().await {
            cleanup_errors.push(error.to_string());
        }
    }
    stop.send_replace(true);
    let stats = tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .map_err(|_| "Server cleanup timed out")???;
    for worker in workers {
        worker.await?;
    }
    let relays: Vec<_> = controls
        .iter()
        .map(|c| c.lock().unwrap().stats.clone())
        .collect();
    let passed =
        scenario.is_ok() && cleanup_errors.is_empty() && relays.iter().all(|r| r.overflow == 0);
    Ok(
        json!({"passed":passed,"clients":client_states,"round":round,"drop_initial_connection_ack":drop_ack,"tick_hz":rate.hz(),"profile":profile.name,"reuse_backend_endpoint":reuse,"phase":phase,"replacement_ms":replacement_ms,"result":scenario.as_ref().ok(),"error":scenario.err().map(|e|e.to_string()),"replacement_error":replacement_error,"cleanup_errors":cleanup_errors,"confirmation":stats.connection_confirmation,"simulation":stats.simulation,"io":stats.io,"max_event_queue_bytes":stats.max_event_queue_bytes,"max_movement_queue_bytes":stats.max_movement_queue_bytes,"relays":relays}),
    )
}
async fn run() -> Result<bool> {
    let mut args = std::env::args().skip(1);
    let mut rounds = 1;
    let mut drop_ack = false;
    let mut periodic_loss = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--rounds" => rounds = args.next().ok_or("Missing rounds")?.parse::<u32>()?,
            "--drop-first-keepalive" if !drop_ack => drop_ack = true,
            "--periodic-loss" if !periodic_loss => periodic_loss = true,
            _ => {
                return Err(
                    "Usage: qualify_session_rejoin [--rounds 1..8] [--drop-first-keepalive] [--periodic-loss]".into(),
                );
            }
        }
    }
    if !(1..=8).contains(&rounds) {
        return Err("Expected 1..8 rounds".into());
    }
    let profiles = [
        relay::Profile::BASELINE,
        relay::Profile {
            name: if periodic_loss {
                "periodic-loss"
            } else {
                "seeded-block-loss"
            },
            delay_ms: 20,
            jitter_ms: 30,
            loss_every: 11,
            duplicate_every: 17,
        },
    ];
    let mut results = Vec::new();
    for round in 1..=rounds {
        for profile in profiles {
            for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
                for reuse in [false, true] {
                    let result = case(rate, profile, reuse, round, drop_ack, periodic_loss).await?;
                    println!("session_rejoin_case {}", serde_json::to_string(&result)?);
                    results.push(result);
                }
            }
        }
    }
    let passed = results.iter().all(|r| r["passed"] == true);
    let report = json!({"passed":passed,"scope":"Actual loopback encrypted UDP, bounded deterministic impairment and runtime deadlines; no native/OIDC/SQL/population claim","rounds":rounds,"cases":results});
    let bytes = serde_json::to_vec(&report)?;
    if bytes.len() > 262144 {
        return Err("Session rejoin report exceeds budget".into());
    }
    println!("session_rejoin_qualification {}", String::from_utf8(bytes)?);
    Ok(passed)
}
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    match run().await {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("Session rejoin qualification failed: {error}");
            std::process::exit(1);
        }
    }
}
