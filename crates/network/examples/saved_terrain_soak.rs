//! Real eight-peer UDP/physics soak. Headless collision readiness is NOT GPU readiness.
use std::{
    collections::BTreeSet,
    fs,
    net::UdpSocket,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::{
    Result,
    client::State,
    prediction_driver::Driver,
    prediction_wire::Commands,
    procedural::{Runtime, Source},
    renet_runtime as net,
    renet_transport::CHANNEL_MEMORY_BYTES,
    saved_terrain::Store,
    simulation::World,
    wire::Intent,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

struct StopOnDrop(watch::Sender<bool>);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
struct Tickets(PathBuf);
impl Drop for Tickets {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Peer {
    state: watch::Receiver<State>,
    ready: watch::Sender<Option<Vec<ChunkId>>>,
    commands: mpsc::Sender<Commands>,
    _input: watch::Sender<Intent>,
    task: JoinHandle<Result<()>>,
    terrain: Runtime,
    store: Arc<Store>,
    driver: Driver,
    visited: BTreeSet<ChunkId>,
    start: Option<GlobalPosition>,
    max_excursion: f64,
    max_patches: usize,
    max_visible: usize,
    min_visible: usize,
    saw_interest_pruning: bool,
    visibility_returned: bool,
}
fn route(index: usize, elapsed: f64) -> Intent {
    let elapsed = elapsed - 2.;
    if elapsed < 0. || elapsed % 20. >= 15. {
        return Intent::default();
    }
    let directions = [
        [1000, 0],
        [-1000, 0],
        [0, 1000],
        [0, -1000],
        [707, 707],
        [-707, 707],
        [707, -707],
        [-707, -707],
    ];
    let sign = if ((elapsed / 120.) as u64).is_multiple_of(2) {
        1
    } else {
        -1
    };
    Intent {
        axes: directions[index].map(|v| v * sign),
        sprint: true,
        jump: (elapsed + index as f64 * 0.25) % 7. < 0.25,
    }
}
#[cfg(target_os = "linux")]
fn rss_bytes() -> Option<u64> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()?
                .checked_mul(1024)
        })
}
#[cfg(not(target_os = "linux"))]
fn rss_bytes() -> Option<u64> {
    None
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let package = PathBuf::from(
        args.next()
            .ok_or("Expected PACKAGE OUTPUT [seconds:20..600]")?,
    );
    let output = PathBuf::from(args.next().ok_or("Missing output directory")?);
    let seconds: u64 = args.next().map(|v| v.parse()).transpose()?.unwrap_or(180);
    if args.next().is_some() || !(20..=600).contains(&seconds) {
        return Err("Duration must be 20..600 seconds".into());
    }
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&output)?;
    let store = Arc::new(Store::open(&package)?);
    let identity = store.identity();
    let mut world = World::with_terrain_source([8000., 8000.], 1., Source::Saved(store.clone()))?;
    world.enable_prediction()?;
    // Prepare independent client sources/query workers before starting wall-time
    // qualification. Each has its own counters and at most one pending job.
    let mut prepared = Vec::new();
    for _ in 0..8 {
        let client_store = Arc::new(Store::open(&package)?);
        let terrain =
            Runtime::with_source([8000., 8000.], 1., Source::Saved(client_store.clone()))?;
        prepared.push((client_store, terrain));
    }
    let tickets = Tickets(output.join("tickets"));
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let address = socket.local_addr()?;
    let transport = net::issue_local(socket, &tickets.0)?;
    let (stop, shutdown) = watch::channel(false);
    let guard = StopOnDrop(stop.clone());
    let server = tokio::spawn(net::run_server_measured(transport, shutdown.clone(), world));
    let mut peers = Vec::new();
    for (index, (client_store, terrain)) in prepared.into_iter().enumerate() {
        let ticket = fs::read(tickets.0.join(format!("client-{}.token", index + 1)))?;
        let (input, intent) = watch::channel(Intent::default());
        let (ready, readiness) = watch::channel(None);
        let (commands, command_queue) = mpsc::channel(8);
        let (states, state) = watch::channel(State::default());
        let stopped = shutdown.clone();
        let task = tokio::spawn(async move {
            let result = net::run_client_predicted(
                address,
                &ticket,
                intent,
                states.clone(),
                stopped,
                identity,
                readiness,
                command_queue,
            )
            .await;
            if let Err(error) = &result {
                states.send_modify(|s| s.error = Some(error.to_string()));
            }
            result
        });
        peers.push(Peer {
            state,
            ready,
            commands,
            _input: input,
            task,
            terrain,
            store: client_store,
            driver: Driver::default(),
            visited: BTreeSet::new(),
            start: None,
            max_excursion: 0.,
            max_patches: 0,
            max_visible: 0,
            min_visible: 8,
            saw_interest_pruning: false,
            visibility_returned: false,
        });
    }
    let admission = Instant::now();
    while !peers.iter().all(|p| {
        p.state
            .borrow()
            .snapshot
            .as_ref()
            .is_some_and(|s| s.players.len() == 8)
    }) {
        for peer in &peers {
            if let Some(error) = &peer.state.borrow().error {
                return Err(error.clone().into());
            }
        }
        if admission.elapsed() > Duration::from_secs(10) || server.is_finished() {
            return Err("Eight-peer admission failed".into());
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let ids: BTreeSet<_> = peers.iter().map(|p| p.state.borrow().id.unwrap()).collect();
    if ids.len() != 8 {
        return Err("Actor identities are not unique".into());
    }
    let start = Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(8));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    let mut interrupted = false;
    let mut samples = Vec::with_capacity(seconds as usize + 1);
    let mut next_sample = 0.;
    let mut peak_rss = 0;
    while start.elapsed() < Duration::from_secs(seconds) {
        tokio::select! {_=&mut interrupt=>{interrupted=true;break;}, _=interval.tick()=>{}}
        if server.is_finished() {
            return Err("Authority exited during soak".into());
        }
        let elapsed = start.elapsed().as_secs_f64();
        for (index, peer) in peers.iter_mut().enumerate() {
            let state = peer.state.borrow().clone();
            if let Some(error) = state.error.as_ref().or(peer.terrain.error.as_ref()) {
                return Err(error.clone().into());
            }
            let focus = state
                .authority_focus()
                .ok_or("Missing authoritative position")?;
            peer.terrain.poll(&[focus]);
            let patch = peer.terrain.patch(focus).cloned();
            if let Some(patch) = &patch {
                let loaded: Vec<_> = patch.ids.iter().copied().collect();
                peer.ready.send_if_modified(|old| {
                    if old.as_ref() == Some(&loaded) {
                        false
                    } else {
                        *old = Some(loaded);
                        true
                    }
                });
            }
            if let Some(driven) = peer.driver.tick(
                &state,
                route(index, elapsed),
                patch.as_deref(),
                Instant::now(),
            )? && let Some(packet) = driven.packet
            {
                peer.commands
                    .try_send(packet)
                    .map_err(|_| "Bounded client command queue overflow")?;
            }
            peer.visited
                .insert(focus.chunk(128).ok_or("Invalid chunk")?);
            let initial = peer.start.get_or_insert(focus).metres();
            let p = focus.metres();
            peer.max_excursion = peer
                .max_excursion
                .max((p[0] - initial[0]).hypot(p[2] - initial[2]));
            peer.max_patches = peer.max_patches.max(peer.terrain.patches.len());
            if let Some(snapshot) = &state.snapshot {
                peer.max_visible = peer.max_visible.max(snapshot.players.len());
                peer.min_visible = peer.min_visible.min(snapshot.players.len());
                if snapshot.players.len() < 8 {
                    peer.saw_interest_pruning = true;
                } else if peer.saw_interest_pruning {
                    let returned: BTreeSet<_> = snapshot.players.iter().map(|p| p.id).collect();
                    if returned != ids {
                        return Err("Actor identities changed during spatial re-entry".into());
                    }
                    peer.visibility_returned = true;
                }
            }
        }
        if elapsed >= next_sample {
            let rss = rss_bytes();
            peak_rss = peak_rss.max(rss.unwrap_or(0));
            let positions: Vec<_> = peers
                .iter()
                .map(|p| p.state.borrow().authority_focus().map(|p| p.metres()))
                .collect();
            samples
                .push(serde_json::json!({"seconds":elapsed,"rss_bytes":rss,"positions":positions}));
            println!("soak_progress seconds={elapsed:.1} peers=8 rss_bytes={peak_rss}");
            next_sample += 1.;
        }
    }
    let reports:Vec<_>=peers.iter().map(|p| {
        let state=p.state.borrow();
        serde_json::json!({"id":state.id,"visited_chunks":p.visited.len(),"max_excursion_metres":p.max_excursion,"max_collision_patches":p.max_patches,"collision_generated":p.terrain.generated,"collision_retired":p.terrain.retired,"max_visible":p.max_visible,"min_visible":p.min_visible,"saw_interest_pruning":p.saw_interest_pruning,"visibility_returned":p.visibility_returned,"reads":p.store.stats(),"driver":p.driver.stats,"transport":state.transport})
    }).collect();
    stop.send_replace(true);
    for peer in peers {
        peer.task.await??;
    }
    let stats = server.await??;
    drop(guard);
    drop(tickets);
    let mut failures = Vec::new();
    let tickets_removed = !output.join("tickets").exists();
    if !tickets_removed {
        failures.push("Temporary tickets survived shutdown".to_string());
    }
    if interrupted {
        failures.push("Interrupted: full duration was not qualified".to_string());
    }
    if stats.max_players != 8
        || stats.movement_recoveries != 0
        || stats.rejected_inputs != 0
        || stats.rejected_chunk_ready != 0
    {
        failures.push("Authority identity/movement/readiness failure".into());
    }
    if stats.terrain_peak_resident > 9
        || stats.max_movement_queue_bytes > CHANNEL_MEMORY_BYTES
        || stats.max_event_queue_bytes > CHANNEL_MEMORY_BYTES
    {
        failures.push("Authority residency/queue budget exceeded".into());
    }
    if stats.simulation.missed_deadlines != 0 || stats.skipped_intervals != 0 {
        failures.push("Authority missed a fixed-step deadline".into());
    }
    if peak_rss > 1024 * 1024 * 1024 {
        failures.push("Combined headless process exceeded the 1 GiB qualification ceiling".into());
    }
    for peer in &reports {
        if peer["visited_chunks"].as_u64().unwrap() < 2
            || peer["max_excursion_metres"].as_f64().unwrap() < 50.
            || peer["max_collision_patches"].as_u64().unwrap() > 2
            || peer["max_visible"] != 8
        {
            failures.push("A peer failed actual traversal/residency/replication checks".into());
        }
        if peer["driver"]["max_history"].as_u64().unwrap() > 64
            || peer["driver"]["clock_phase_backsteps"] != 0
            || peer["driver"]["predicted_steps"].as_u64().unwrap() < 100
            || peer["driver"]["clock_acquisitions"] != 1
        {
            failures.push("A peer failed bounded prediction/clock checks".into());
        }
        if seconds >= 60 && peer["collision_retired"] == 0 {
            failures.push("A peer did not retire actual collision".into());
        }
    }
    if seconds >= 150
        && !reports
            .iter()
            .any(|p| p["min_visible"].as_u64().unwrap() < 8)
    {
        failures.push("No measured spatial relevance pruning".into());
    }
    if seconds >= 180 && !reports.iter().all(|p| p["visibility_returned"] == true) {
        failures
            .push("Not every peer observed the same eight identities return after pruning".into());
    }
    let passed = failures.is_empty();
    let report = serde_json::json!({"passed":passed,"tickets_removed":tickets_removed,"failures":failures,"seconds":start.elapsed().as_secs_f64(),"requested_seconds":seconds,"interrupted":interrupted,"world":identity,"platform":std::env::consts::OS,"tick_hz":30,"peers":reports,"server":stats,"server_tile_reads":store.stats(),"peak_combined_rss_bytes":peak_rss,"samples":samples,"scope":"Eight real encrypted loopback peers with actual application prediction/controller and saved terrain; headless collision readiness only, no GPU/native-client/population-capacity claim"});
    fs::write(
        output.join("verification.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("soak_complete passed={passed} output={}", output.display());
    if !passed {
        return Err("Saved terrain soak failed; inspect verification.json".into());
    }
    Ok(())
}
