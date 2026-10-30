//! Executable Renet bridge. Transport polling and fixed simulation are separate;
//! only superseding inputs/snapshots cross the presentation boundary.
mod admissions;
mod checkpoints;
pub mod dialogue;
pub mod inventory;
pub mod progression;
mod tickets;
use crate::{
    Result,
    client::{State, TransportStats},
    renet_transport::*,
    simulation::World,
    wire::*,
};
pub use admissions::{AccountAdmissions, AdmissionLease, PlayerAdmission};
pub use checkpoints::{ActorCheckpoint, AuthorityCheckpoint, CheckpointBridge};
use renet::{RenetClient, RenetServer, ServerEvent};
use renet_netcode::{ConnectToken, NetcodeServerTransport};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    net::{SocketAddr, UdpSocket},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use theia_core::time::TickMetrics;
pub use tickets::{IssuedTicket, TicketIssuer, secure_issuer};
use tokio::{
    sync::{Semaphore, mpsc, oneshot, watch},
    time::{Instant, MissedTickBehavior},
};

pub fn now() -> Result<Duration> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?)
}

/// Local development issuer. Private signing key stays in memory. Files contain
/// bearer tickets, never the signing key, and must stay outside version control.
pub fn issue_local(socket: UdpSocket, directory: &Path) -> Result<NetcodeServerTransport> {
    let address = socket.local_addr()?;
    issue_local_advertised(socket, directory, vec![address])
}

/// Trusted loopback qualification only: issue per-client tickets for bounded
/// relay endpoints while keeping the private server key in this process.
pub fn issue_local_advertised(
    socket: UdpSocket,
    directory: &Path,
    advertised: Vec<SocketAddr>,
) -> Result<NetcodeServerTransport> {
    let (transport, issuer) = secure_issuer(socket, advertised)?;
    std::fs::create_dir_all(directory)?;
    for id in 1..=MAX_PLAYERS as u64 {
        let ticket = issuer.issue(id, None, 120)?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(directory.join(format!("client-{id}.token")))?;
        file.write_all(ticket.bytes())?;
        file.flush()?;
    }
    Ok(transport)
}

pub fn read_ticket(bytes: &[u8], address: SocketAddr) -> Result<ConnectToken> {
    if !address.ip().is_loopback() || bytes.len() > 2048 {
        return Err("Invalid local ticket/address".into());
    }
    let mut cursor = std::io::Cursor::new(bytes);
    let token = ConnectToken::read(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err("Trailing ticket bytes".into());
    }
    if token.protocol_id != PROTOCOL_ID
        || token
            .server_addresses
            .iter()
            .flatten()
            .any(|a| *a != address)
        || !token.server_addresses.contains(&Some(address))
        || token.expire_timestamp <= now()?.as_secs()
    {
        return Err("Expired or mismatched Renet ticket".into());
    }
    Ok(token)
}

struct Peer {
    dialogue: Option<dialogue::Peer>,
    admission: Option<Box<dyn AdmissionLease>>,
    progression: Option<progression::Peer>,
    inventory: Option<inventory::Peer>,
    inventory_requests: u32,
    actor: u64,
    latest: Option<Input>,
    received: u32,
    period: Instant,
    pending_chunk_ack: Option<ChunkReady>,
    offered_generation: u64,
    target_sequence: u64,
    target_accepted: bool,
    target_sent: Option<crate::targeting_wire::State>,
    combat_sent: Option<crate::combat_wire::State>,
    status_sent: Option<crate::status_wire::State>,
    appearance_sent: Option<crate::appearance_wire::State>,
    combat_event_sequence: u64,
    cast_requests: u32,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ServerStats {
    pub dialogue_requests_accepted: u64,
    pub dialogue_requests_rejected: u64,
    pub dialogue_views_sent: u64,
    pub guide_presences_sent: u64,
    pub dialogue_replies_sent: u64,
    pub dialogue_backpressure_disconnects: u64,
    pub experience_opportunities_forwarded: u64,
    pub progression_states_sent: u64,
    pub quest_states_sent: u64,
    pub reward_opportunities_forwarded: u64,
    pub reward_states_sent: u64,
    pub appearance_states_sent: u64,
    pub patrol: Option<crate::simulation::creatures::Snapshot>,
    pub status_states_sent: u64,
    pub status_backpressure_disconnects: u64,
    pub inventory_states_sent: u64,
    pub inventory_requests_forwarded: u64,
    pub inventory_requests_ignored: u64,
    pub inventory_backpressure_disconnects: u64,
    pub checkpoint_publications: u64,
    pub persistent_spawn_admissions: u64,
    pub combat_states_sent: u64,
    pub combat_events_sent: u64,
    pub combat_backpressure_disconnects: u64,
    pub cast_requests_accepted: u64,
    pub cast_requests_rejected: u64,
    pub cast_requests_ignored: u64,
    pub target_states_sent: u64,
    pub target_selections_accepted: u64,
    pub target_selections_rejected: u64,
    pub terrain_peak_resident: usize,
    pub terrain_generated: u64,
    pub terrain_retired: u64,
    pub terrain_resident: usize,
    pub queued_commands: u64,
    pub max_movement_queue_bytes: usize,
    pub max_event_queue_bytes: usize,
    pub max_pending_inputs: usize,
    pub max_input_batch: usize,
    pub received_inputs: u64,
    pub rejected_inputs: u64,
    pub applied_inputs: u64,
    pub max_players: usize,
    pub skipped_intervals: u64,
    pub wall_seconds: f64,
    pub tick_hz: u32,
    pub chunk_generation: u64,
    pub chunk_rejections: u64,
    pub chunk_blocked_steps: u64,
    pub movement_recoveries: u64,
    pub chunk_unready_steps: u64,
    pub accepted_chunk_ready: u64,
    pub chunk_ready_acks_sent: u64,
    pub chunk_offers_sent: u64,
    pub rejected_chunk_ready: u64,
    pub max_control_batch: usize,
    pub simulation: TimingStats,
    pub io: TimingStats,
}

/// Quantile values are logarithmic histogram upper bounds; maxima are exact.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct TimingStats {
    pub samples: u64,
    pub missed_deadlines: u64,
    pub work_p50_upper_us: f64,
    pub work_p95_upper_us: f64,
    pub work_max_us: f64,
    pub lateness_p95_upper_us: f64,
    pub lateness_max_us: f64,
}
impl From<&TickMetrics> for TimingStats {
    fn from(metrics: &TickMetrics) -> Self {
        let micros = |d: Duration| d.as_secs_f64() * 1_000_000.;
        Self {
            samples: metrics.steps,
            missed_deadlines: metrics.missed_deadlines,
            work_p50_upper_us: micros(metrics.work.percentile_upper(50)),
            work_p95_upper_us: micros(metrics.work.percentile_upper(95)),
            work_max_us: micros(metrics.max_work),
            lateness_p95_upper_us: micros(metrics.lateness.percentile_upper(95)),
            lateness_max_us: micros(metrics.max_lateness),
        }
    }
}
impl ServerStats {
    fn observe(&mut self, server: &RenetServer, peers: &BTreeMap<u64, Peer>) {
        self.max_pending_inputs = self
            .max_pending_inputs
            .max(peers.values().filter(|p| p.latest.is_some()).count());
        for id in server.clients_id() {
            self.max_movement_queue_bytes = self
                .max_movement_queue_bytes
                .max(CHANNEL_MEMORY_BYTES - server.channel_available_memory(id, MOVEMENT_CHANNEL));
            self.max_event_queue_bytes = self
                .max_event_queue_bytes
                .max(CHANNEL_MEMORY_BYTES - server.channel_available_memory(id, EVENT_CHANNEL));
        }
    }
}

/// Local owner-side world management only; these requests are never client messages.
/// One transaction and one queued command per controller. Loading happens on a
/// blocking worker; publication is serialized with authoritative actor updates.
#[derive(Clone)]
pub struct WorldControl {
    requests: mpsc::Sender<WorldRequest>,
    gate: std::sync::Arc<Semaphore>,
}
pub struct WorldRequests(mpsc::Receiver<WorldRequest>);
pub type ChunkControl = WorldControl;
pub type ChunkRequests = WorldRequests;
enum WorldRequest {
    Plan(
        Vec<theia_core::coordinates::ChunkId>,
        oneshot::Sender<Result<crate::chunk_world::LoadPlan>>,
    ),
    Publish(crate::chunk_world::Prepared, oneshot::Sender<Result<u64>>),
    RemoveTarget(theia_core::identity::EntityKey, oneshot::Sender<bool>),
}
pub fn chunk_control() -> (ChunkControl, ChunkRequests) {
    world_control()
}
pub fn world_control() -> (WorldControl, WorldRequests) {
    let (requests, receiver) = mpsc::channel(1);
    (
        WorldControl {
            requests,
            gate: std::sync::Arc::new(Semaphore::new(1)),
        },
        WorldRequests(receiver),
    )
}
impl WorldControl {
    /// Trusted local management only; not a client message or combat hot path.
    pub async fn remove_target(&self, key: theia_core::identity::EntityKey) -> Result<bool> {
        let _permit = self
            .gate
            .clone()
            .try_acquire_owned()
            .map_err(|_| "World transaction already in flight")?;
        let (reply, receive) = oneshot::channel();
        self.requests
            .send(WorldRequest::RemoveTarget(key, reply))
            .await?;
        Ok(receive.await?)
    }
    pub async fn replace(&self, required: &[theia_core::coordinates::ChunkId]) -> Result<u64> {
        if required.len() > crate::chunks::MAX_RESIDENT {
            return Err("Chunk request exceeds bounded budget".into());
        }
        let _permit = self
            .gate
            .clone()
            .try_acquire_owned()
            .map_err(|_| "Chunk transaction already in flight")?;
        let (reply, receive) = oneshot::channel();
        self.requests
            .send(WorldRequest::Plan(required.to_vec(), reply))
            .await?;
        let plan = receive.await??;
        // Retain the permit inside the blocking task even if the caller cancels.
        let (candidate, _permit) =
            tokio::task::spawn_blocking(move || (plan.load(), _permit)).await?;
        let (reply, receive) = oneshot::channel();
        self.requests
            .send(WorldRequest::Publish(candidate?, reply))
            .await?;
        receive.await?
    }
}

/// Actual Renet loop with bounded local chunk management. Normal executable map
/// selection is unchanged until the paired client residency gate is qualified.
pub async fn run_server_with_chunks(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
    requests: ChunkRequests,
) -> Result<ServerStats> {
    run_server_inner(transport, shutdown, world, requests, None, None, None).await
}

/// Production owner loop with bounded trusted management; requests are not wire data.
pub async fn run_server_with_world_control(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
    requests: WorldRequests,
) -> Result<ServerStats> {
    run_server_inner(transport, shutdown, world, requests, None, None, None).await
}

type Demand = Option<Vec<theia_core::coordinates::ChunkId>>;
/// Opt-in bounded lookahead policy. One owner task loads off-thread; actor and
/// recovery coverage still revalidate at publication in the authoritative loop.
pub async fn run_server_streaming(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
) -> Result<ServerStats> {
    if world.chunks().is_none() {
        return Err("Chunk streaming requires a chunk world".into());
    }
    let (control, requests) = chunk_control();
    let (updates, mut demand) = watch::channel::<Demand>(None);
    let mut stop = shutdown.clone();
    let owner = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _=stop.changed()=>break,
                result=demand.changed()=>{
                    if result.is_err() {break;}
                    let next=demand.borrow_and_update().clone();
                    if let Some(ids)=next {
                        match control.replace(&ids).await {
                            Ok(generation)=>println!("chunk_policy_published generation={generation}"),
                            Err(error)=>eprintln!("chunk_policy_rejected {error}"),
                        }
                    }
                }
            }
        }
    });
    let result = run_server_inner(
        transport,
        shutdown,
        world,
        requests,
        Some(updates),
        None,
        None,
    )
    .await;
    owner.abort();
    let _ = owner.await;
    let stats = result?;
    println!("transport_summary {}", serde_json::to_string(&stats)?);
    Ok(stats)
}

pub async fn run_server(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
) -> Result<()> {
    let stats = run_server_measured(transport, shutdown, world).await?;
    println!("transport_summary {}", serde_json::to_string(&stats)?);
    Ok(())
}

/// Same executable loop with returned diagnostics for controlled-network qualification.
pub async fn run_server_measured(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
) -> Result<ServerStats> {
    let (_, requests) = chunk_control();
    run_server_inner(transport, shutdown, world, requests, None, None, None).await
}
/// Opt-in local-issuer persistence bridge. The application owns DB work and
/// stable identities; this authority only restores validated poses and emits
/// checked f64 snapshots through one bounded latest-value channel.
pub async fn run_server_with_checkpoints(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
    bridge: CheckpointBridge,
) -> Result<ServerStats> {
    if world.procedural().is_none() {
        return Err("Checkpoint bridge requires streamed terrain".into());
    }
    let (_, requests) = world_control();
    let stats = run_server_inner(
        transport,
        shutdown,
        world,
        requests,
        None,
        Some(bridge),
        None,
    )
    .await?;
    println!("transport_summary {}", serde_json::to_string(&stats)?);
    Ok(stats)
}

/// Account-authorized entry is opt-in; legacy tickets keep their lifetime replay
/// guard. The application must monitor database health and retire leases durably.
pub async fn run_server_with_admissions(
    transport: NetcodeServerTransport,
    shutdown: watch::Receiver<bool>,
    world: World,
    authority: std::sync::Arc<dyn AccountAdmissions>,
) -> Result<ServerStats> {
    if world.procedural().is_none() || !world.prediction_enabled() || world.dev_tour {
        return Err("Account admission requires normal predicted streamed terrain".into());
    }
    let (_, requests) = world_control();
    run_server_inner(
        transport,
        shutdown,
        world,
        requests,
        None,
        None,
        Some(authority),
    )
    .await
}
fn admit_peer(
    world: &mut World,
    server: &mut RenetServer,
    client: u64,
    position: Option<theia_core::coordinates::GlobalPosition>,
    admission: Option<Box<dyn AdmissionLease>>,
    current: Instant,
) -> Result<Option<Peer>> {
    let actor = if let Some(position) = position {
        world.join_restored(position)
    } else {
        world.join()
    };
    let Some(actor) = actor else {
        server.disconnect(client);
        return Ok(None);
    };
    let inventory = admission
        .as_ref()
        .map(|a| a.inventory())
        .transpose()?
        .flatten()
        .map(inventory::Peer::new)
        .transpose()?;
    match (
        world.reward_profile(),
        inventory.as_ref().and_then(|p| p.port.rewards.as_ref()),
    ) {
        (None, None) => {}
        (Some(a), Some(b)) if a.hash() == b.hash() => {}
        _ => return Err("Admitted reward profile differs from authoritative world".into()),
    }
    match (
        world.vendor_profile(),
        inventory.as_ref().and_then(|p| p.port.vendor.as_ref()),
    ) {
        (None, None) => {}
        (Some(a), Some(b)) if a.hash() == b.hash() => {}
        _ => return Err("Admitted vendor profile differs from authoritative world".into()),
    }
    let progression = admission
        .as_ref()
        .map(|a| a.progression())
        .transpose()?
        .flatten()
        .map(progression::Peer::new)
        .transpose()?;
    match (world.progression_profile(), progression.as_ref()) {
        (None, None) => {}
        (Some(a), Some(b)) if a.hash() == b.port.profile.hash() => {}
        _ => return Err("Admitted XP profile differs from authoritative world".into()),
    }
    match (
        world.quest_profile(),
        progression.as_ref().and_then(|p| p.port.quest.as_ref()),
    ) {
        (None, None) => {}
        (Some(a), Some(b)) if a.hash() == b.hash() => {}
        _ => return Err("Admitted objective profile differs from authority".into()),
    }
    let peer = Peer {
        dialogue: world.dialogue_profile().map(|_| dialogue::Peer::default()),
        progression,
        inventory,
        inventory_requests: 0,
        admission,
        actor,
        latest: None,
        received: 0,
        period: current,
        pending_chunk_ack: None,
        offered_generation: 0,
        target_sequence: 0,
        target_accepted: false,
        target_sent: None,
        combat_sent: None,
        status_sent: None,
        appearance_sent: None,
        combat_event_sequence: 0,
        cast_requests: 0,
    };
    admissions::capture(world, &peer)?;
    if peer.admission.is_some() {
        println!(
            "account_admit {}",
            serde_json::json!({"actor":actor,"epoch":world.epoch(),"position":world.global_position(actor).unwrap().metres(),"input_stream":world.correction(actor).map(|c|c.state.input.stream_epoch)})
        );
    }
    server.send_message(
        client,
        EVENT_CHANNEL,
        Welcome {
            terrain_stream: world.procedural().is_some(),
            id: actor,
            epoch: world.epoch(),
            world: world.identity(),
            origin: world.origin(),
        }
        .encode(),
    );
    println!("player_join id={actor} transport=renet");
    Ok(Some(peer))
}

// Combat events are ordered gameplay data. A full reliable queue closes that
// peer instead of silently dropping an event or coalescing through a death.
fn retire_peers(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    ids: &[u64],
) -> Result<()> {
    for &id in ids {
        server.disconnect(id);
        if let Some(peer) = peers.remove(&id) {
            admissions::capture(world, &peer)?;
            world.leave(peer.actor);
            println!("player_leave id={} transport=renet", peer.actor);
        }
    }
    Ok(())
}
fn publish_progression(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let mut retired = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let Some(owner) = &mut peer.progression else {
            continue;
        };
        let view = owner.port.state.borrow().clone();
        if view.failed {
            return Err("Earned progression owner failed".into());
        }
        if view.closed {
            retired.push(id);
            continue;
        }
        let Some(progress) = view.progress else {
            continue;
        };
        match (&owner.port.quest, &view.quest) {
            (None, None) | (Some(_), Some(_)) => {}
            (Some(_), None) => continue,
            (None, Some(_)) => return Err("Unexpected committed objective state".into()),
        }
        let actor = world
            .entity_key(peer.actor)
            .ok_or("Progression actor missing")?;
        if owner.sent.as_ref() != Some(&progress) {
            let state = crate::progression_wire::State {
                actor,
                profile_hash: owner.port.profile.hash(),
                progress: progress.clone(),
            };
            let bytes = state.encode(&owner.port.profile)?;
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                retired.push(id);
                continue;
            }
            server.send_message(id, EVENT_CHANNEL, bytes);
            owner.sent = Some(progress);
            stats.progression_states_sent += 1;
        }
        // XP can stay capped while an objective still advances. Its observation
        // must not be hidden behind an unchanged XP revision/coalescing guard.
        if let (Some(profile), Some(progress)) = (&owner.port.quest, view.quest)
            && owner.quest_sent.as_ref() != Some(&progress)
        {
            let state = crate::quest_wire::State {
                actor,
                profile_hash: profile.hash(),
                progress: progress.clone(),
            };
            let bytes = state.encode(profile)?;
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                retired.push(id);
                continue;
            }
            server.send_message(id, EVENT_CHANNEL, bytes);
            owner.quest_sent = Some(progress);
            stats.quest_states_sent += 1;
        }
    }
    retire_peers(server, peers, world, &retired)
}
fn forward_experience_opportunity(
    world: &World,
    peer: &Peer,
    stats: &mut ServerStats,
    fact: crate::simulation::progression::Opportunity,
) -> Result<()> {
    if world.entity_key(peer.actor) != Some(fact.actor) {
        return Err("XP source binding mismatch".into());
    }
    let port = &peer
        .progression
        .as_ref()
        .ok_or("XP fact requires character owner")?
        .port;
    if world
        .progression_profile()
        .is_none_or(|p| p.hash() != port.profile.hash())
    {
        return Err("XP fact profile mismatch".into());
    }
    port.earned
        .try_send(fact.event)
        .map_err(|_| "XP fact queue unavailable")?;
    stats.experience_opportunities_forwarded += 1;
    Ok(())
}
fn forward_experience_opportunities(
    world: &World,
    peers: &BTreeMap<u64, Peer>,
    stats: &mut ServerStats,
    events: &[crate::simulation::combat::Event],
) -> Result<()> {
    for fact in world.experience_opportunities(events)? {
        let peer = peers
            .values()
            .find(|p| world.entity_key(p.actor) == Some(fact.actor))
            .ok_or("XP source has no admitted peer")?;
        forward_experience_opportunity(world, peer, stats, fact)?;
    }
    Ok(())
}
fn publish_inventory(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let mut retired = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let Some(inventory) = &mut peer.inventory else {
            continue;
        };
        match inventory.view(
            world
                .entity_key(peer.actor)
                .ok_or("Inventory actor missing")?,
        )? {
            inventory::View::Loading => {}
            inventory::View::Closed => retired.push(id),
            inventory::View::State(state) => {
                let bytes = state.encode(&inventory.port.catalog, inventory.port.hash)?;
                if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                    stats.inventory_backpressure_disconnects += 1;
                    retired.push(id);
                    continue;
                }
                server.send_message(id, EVENT_CHANNEL, bytes);
                inventory.sent(*state);
                stats.inventory_states_sent += 1;
            }
        }
        let actor = world.entity_key(peer.actor).ok_or("Reward actor missing")?;
        if let Some(view) = inventory.rewards_view(actor)? {
            let profile = inventory
                .port
                .rewards
                .as_ref()
                .ok_or("Reward profile missing")?;
            let bytes = view.encode(profile.catalog(), profile.hash())?;
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                stats.inventory_backpressure_disconnects += 1;
                retired.push(id);
                continue;
            }
            server.send_message(id, EVENT_CHANNEL, bytes);
            inventory.rewards_sent(view);
            stats.reward_states_sent += 1;
        }
    }
    retire_peers(server, peers, world, &retired)
}
fn forward_reward_opportunity(
    world: &World,
    peer: &Peer,
    stats: &mut ServerStats,
    fact: crate::simulation::rewards::Opportunity,
) -> Result<()> {
    if world.entity_key(peer.actor) != Some(fact.actor) {
        return Err("Earned reward source mismatch".into());
    }
    let port = &peer
        .inventory
        .as_ref()
        .ok_or("Earned reward requires item owner")?
        .port;
    if port
        .rewards
        .as_ref()
        .zip(world.reward_profile())
        .is_none_or(|(a, b)| a.hash() != b.hash())
    {
        return Err("Earned reward profile mismatch".into());
    }
    port.opportunities
        .as_ref()
        .ok_or("Reward opportunity port missing")?
        .try_send(fact.reward)
        .map_err(|_| "Earned reward queue unavailable")?;
    stats.reward_opportunities_forwarded += 1;
    Ok(())
}
fn forward_reward_opportunities(
    world: &World,
    peers: &BTreeMap<u64, Peer>,
    stats: &mut ServerStats,
    events: &[crate::simulation::combat::Event],
) -> Result<()> {
    for fact in world.reward_opportunities(events)? {
        let peer = peers
            .values()
            .find(|p| world.entity_key(p.actor) == Some(fact.actor))
            .ok_or("Earned reward has no admitted peer")?;
        forward_reward_opportunity(world, peer, stats, fact)?;
    }
    Ok(())
}

fn publish_appearance(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    if world.completed_tick() == 0 {
        return Ok(());
    }
    let mut committed = BTreeMap::new();
    for peer in peers.values() {
        let Some(inventory) = &peer.inventory else {
            continue;
        };
        if !inventory.port.catalog.equipment_enabled() {
            continue;
        }
        let state = inventory.port.state.borrow();
        if state.failed || state.closed {
            continue;
        }
        if let Some(bag) = &state.bag {
            let hand = bag
                .main_hand()
                .map(|s| {
                    inventory
                        .port
                        .catalog
                        .definition(s.item())
                        .and_then(|d| d.main_hand_appearance())
                        .cloned()
                        .ok_or("Committed item has no appearance")
                })
                .transpose()?;
            committed.insert(peer.actor, (inventory.port.hash, hand));
        }
    }
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let Some(inventory) = &peer.inventory else {
            continue;
        };
        if !inventory.port.catalog.equipment_enabled() {
            continue;
        }
        let mut actors = Vec::new();
        for player in world.snapshot_for(peer.actor).players {
            if let Some((hash, hand)) = committed.get(&player.id) {
                if hash != &inventory.port.hash {
                    return Err("Visible equipment catalogs differ".into());
                }
                actors.push(crate::appearance_wire::Actor {
                    key: world
                        .entity_key(player.id)
                        .ok_or("Appearance actor missing")?,
                    main_hand: hand.clone(),
                });
            }
        }
        let view = crate::appearance_wire::State {
            epoch: world.epoch(),
            tick: world.completed_tick(),
            catalog_hash: inventory.port.hash,
            actors,
        };
        if peer
            .appearance_sent
            .as_ref()
            .is_some_and(|old| old.same_view(&view))
        {
            continue;
        }
        let bytes = view.encode(&inventory.port.catalog, inventory.port.hash)?;
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
            stats.inventory_backpressure_disconnects += 1;
            slow.push(id);
            continue;
        }
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.appearance_sent = Some(view);
        stats.appearance_states_sent += 1;
    }
    retire_peers(server, peers, world, &slow)
}
fn publish_combat_events(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
    events: &[crate::simulation::combat::Event],
) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let catalog = world
        .combat_catalog()
        .ok_or("Combat event without catalog")?;
    let tick = world.completed_tick();
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        for chunk in events.chunks(crate::combat_wire::MAX_BATCH_EVENTS) {
            let sequence = peer
                .combat_event_sequence
                .checked_add(1)
                .ok_or("Combat event sequence exhausted")?;
            let batch = crate::combat_wire::Batch {
                epoch: world.epoch(),
                sequence,
                tick,
                events: chunk.to_vec(),
            };
            let bytes = batch.encode(catalog)?;
            if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                slow.push(id);
                break;
            }
            server.send_message(id, EVENT_CHANNEL, bytes);
            peer.combat_event_sequence = sequence;
            stats.combat_events_sent += chunk.len() as u64;
        }
    }
    stats.combat_backpressure_disconnects += slow.len() as u64;
    retire_peers(server, peers, world, &slow)?;
    Ok(())
}
fn publish_status_state(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let Some(catalog) = world.combat_status_catalog() else {
        return Ok(());
    };
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let view = crate::status_wire::State {
            epoch: world.epoch(),
            tick: world.completed_tick(),
            rate: world.tick_rate(),
            statuses: world
                .combat_status_snapshot(peer.actor)
                .ok_or("Status peer without actor")?,
        };
        if peer
            .status_sent
            .as_ref()
            .is_some_and(|old| old.same_view(&view))
        {
            continue;
        }
        let bytes = view.encode(catalog)?;
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
            slow.push(id);
            continue;
        }
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.status_sent = Some(view);
        stats.status_states_sent += 1;
    }
    stats.status_backpressure_disconnects += slow.len() as u64;
    retire_peers(server, peers, world, &slow)
}
fn publish_combat_state(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let Some(catalog) = world.combat_catalog() else {
        return Ok(());
    };
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let view = crate::combat_wire::State::new(
            world.epoch(),
            world
                .combat_snapshot(peer.actor)
                .ok_or("Combat peer without actor")?,
            catalog,
        )?;
        if peer
            .combat_sent
            .as_ref()
            .is_some_and(|old| old.same_view(&view))
        {
            continue;
        }
        let bytes = view.encode(catalog)?;
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
            slow.push(id);
            continue;
        }
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.combat_sent = Some(view);
        stats.combat_states_sent += 1;
    }
    stats.combat_backpressure_disconnects += slow.len() as u64;
    retire_peers(server, peers, world, &slow)?;
    Ok(())
}
async fn run_server_inner(
    mut transport: NetcodeServerTransport,
    mut shutdown: watch::Receiver<bool>,
    mut world: World,
    mut requests: ChunkRequests,
    demand: Option<watch::Sender<Demand>>,
    mut checkpoints: Option<CheckpointBridge>,
    authority: Option<std::sync::Arc<dyn AccountAdmissions>>,
) -> Result<ServerStats> {
    world.require_client_chunks()?;
    let mut server = RenetServer::new(connection_config());
    let mut stats = ServerStats::default();
    let mut peers = BTreeMap::<u64, Peer>::new();
    let mut pending = BTreeMap::<u64, admissions::Pending>::new();
    // A local ticket may create only one session in this server lifetime.
    let mut spent = BTreeSet::new();
    let mut io = tokio::time::interval(Duration::from_millis(5));
    io.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let rate = world.tick_rate();
    let mut tick = tokio::time::interval(rate.period());
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last = Instant::now();
    let mut metrics = TickMetrics::default();
    let mut io_metrics = TickMetrics::default();
    let mut combat_events = Vec::with_capacity(crate::simulation::combat::MAX_STEP_EVENTS);
    let wall_start = Instant::now();
    let mut previous_scheduled: Option<Instant> = None;
    let result: Result<()> = async {
        loop { tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            scheduled = tick.tick() => {
                let started = Instant::now();
                if let Some(previous) = previous_scheduled {
                    let intervals = scheduled.duration_since(previous).as_nanos() / rate.period().as_nanos();
                    stats.skipped_intervals = stats.skipped_intervals.saturating_add(intervals.saturating_sub(1) as u64);
                }
                previous_scheduled = Some(scheduled);
                let mut revoked = Vec::new();
                for (&id, peer) in &peers {
                    if let Some(admission) = &peer.admission && !admission.live()? { revoked.push(id); }
                }
                retire_peers(&mut server, &mut peers, &mut world, &revoked)?;
                for peer in peers.values_mut() {
                    if let Some(input) = peer.latest.take() && world.input(peer.actor, input) { stats.applied_inputs += 1; }
                }
                let snapshot = world.step();
                stats.patrol = world.patrol_snapshot();
                for peer in peers.values() { admissions::capture(&world, peer)?; }
                if let Some(bridge)=&mut checkpoints {
                    stats.checkpoint_publications+=u64::from(bridge.capture(&world,&peers));
                }
                if world.combat_enabled() {
                    let events = world.combat_events().to_vec();
                    forward_reward_opportunities(&world, &peers, &mut stats, &events)?;
                    forward_experience_opportunities(&world, &peers, &mut stats, &events)?;
                    publish_combat_events(&mut server, &mut peers, &mut world, &mut stats, &events)?;
                }
                if world.procedural().is_some() {
                    for (&id,peer) in &peers {server.send_message(id,MOVEMENT_CHANNEL,encode(&world.snapshot_for(peer.actor))?); if let Some(correction)=world.correction(peer.actor) {server.send_message(id,MOVEMENT_CHANNEL,correction.encode()?);}}
                } else {
                    let bytes = encode(&snapshot)?;
                    server.broadcast_message(MOVEMENT_CHANNEL, bytes);
                }
                for (&id, peer) in &mut peers {
                    if world.targeting_enabled() {
                        let view = crate::targeting_wire::State::new(world.epoch(), snapshot.tick,
                            peer.target_sequence, peer.target_accepted, world.targeting_snapshot(peer.actor).unwrap());
                        if peer.target_sent.as_ref().is_none_or(|old| !old.same_view(&view)) {
                            let bytes = view.encode()?;
                            if server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                                server.send_message(id, EVENT_CHANNEL, bytes);
                                peer.target_sent = Some(view);
                                stats.target_states_sent += 1;
                            }
                        }
                    }
                    if let Some(report) = &peer.pending_chunk_ack {
                        let bytes = encode(&ChunkEvent::Ack(ChunkAck::new(report, snapshot.tick)))?;
                        if server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
                            server.send_message(id, EVENT_CHANNEL, bytes);
                            peer.pending_chunk_ack = None;
                            stats.chunk_ready_acks_sent += 1;
                        }
                    }
                }
                publish_combat_state(&mut server, &mut peers, &mut world, &mut stats)?;
                publish_status_state(&mut server,&mut peers,&mut world,&mut stats)?;
                dialogue::publish(&mut server,&mut peers,&mut world,&mut stats)?;
                stats.observe(&server, &peers);
                metrics.record(rate, started.elapsed(), started.saturating_duration_since(scheduled));
            }
            Some(request) = requests.0.recv() => {
                match request {
                    WorldRequest::RemoveTarget(key,reply)=>{let _=reply.send(world.remove_development_target(key));}
                    WorldRequest::Plan(ids, reply) => { let _ = reply.send(world.plan_chunks(&ids)); }
                    WorldRequest::Publish(candidate, reply) => {
                        let result = world.activate_chunks(candidate).map(|()| world.chunks().unwrap().residency().generation());
                        let _ = reply.send(result);
                    }
                }
            }
            scheduled = io.tick() => {
                if let Some(authority) = &authority {
                    let retained = authority.retained_positions()?;
                    world.poll_terrain_keeping(&retained.into_iter().flatten().collect::<Vec<_>>())?;
                } else if let Some(bridge)=&checkpoints {world.poll_terrain_keeping(&bridge.pending())?;}
                else {world.poll_terrain();}
                if let Some(error) = world.procedural().and_then(|t| t.error.as_ref()) { return Err(error.clone().into()); }
                if let Some(terrain) = world.procedural() {
                    stats.terrain_peak_resident=stats.terrain_peak_resident.max(terrain.patches.len());
                    stats.terrain_generated=terrain.generated;
                    stats.terrain_retired=terrain.retired;
                    stats.terrain_resident=terrain.patches.len();
                }
                let current = Instant::now();
                let delta = current.saturating_duration_since(last); last = current;
                server.update(delta);
                transport.update(delta, &mut server)?;
                while let Some(event) = server.get_event() {
                    match event {
                        ServerEvent::ClientConnected { client_id } => {
                            if !(1..=MAX_PLAYERS as u64).contains(&client_id) {server.disconnect(client_id);continue;}
                            if let Some(authority) = &authority {
                                if peers.contains_key(&client_id) || pending.contains_key(&client_id) {server.disconnect(client_id);continue;}
                                let Some(data) = transport.user_data(client_id) else {server.disconnect(client_id);continue;};
                                let Some(admission) = authority.clone().consume(client_id, &data)? else {server.disconnect(client_id);continue;};
                                pending.insert(client_id, admissions::Pending {admission, deadline:current+Duration::from_secs(10)});
                                continue;
                            }
                            if !spent.insert(client_id) {server.disconnect(client_id);continue;}
                            let restored=checkpoints.as_ref().and_then(|b|b.position(client_id));
                            if let Some(peer) = admit_peer(&mut world, &mut server, client_id, restored, None, current)? {
                                if let Some(bridge)=&mut checkpoints {
                                    bridge.admitted(client_id);
                                    println!("persistent_admit {}",serde_json::json!({"client_id":client_id,"actor":peer.actor,"epoch":world.epoch(),"position":world.global_position(peer.actor).unwrap().metres()}));
                                }
                                stats.persistent_spawn_admissions+=u64::from(restored.is_some());
                                peers.insert(client_id, peer);
                            }
                        }
                        ServerEvent::ClientDisconnected { client_id, .. } => {
                            pending.remove(&client_id);
                            if let Some(bridge)=&mut checkpoints {stats.checkpoint_publications+=u64::from(bridge.capture(&world,&peers));}
                            if let Some(peer) = peers.remove(&client_id) {
                                admissions::capture(&world,&peer)?;
                                world.leave(peer.actor); println!("player_leave id={} transport=renet", peer.actor);
                            }
                        }
                    }
                }
                // Connected peers wait for the saved pose's collision publication;
                // no Welcome or actor is created against unprepared geometry.
                let mut completed = Vec::new();
                for (&id, reservation) in &pending {
                    if !server.is_connected(id) || current >= reservation.deadline || !reservation.admission.lease.live()? {
                        server.disconnect(id); completed.push((id, false));
                    } else if world.terrain_prepared_for(reservation.admission.position) {
                        completed.push((id, true));
                    }
                }
                for (id, ready) in completed {
                    let reservation = pending.remove(&id).unwrap();
                    if ready {
                        let PlayerAdmission {position, lease} = reservation.admission;
                        if let Some(peer) = admit_peer(&mut world, &mut server, id, Some(position), Some(lease), current)? {
                            stats.persistent_spawn_admissions += 1;
                            peers.insert(id, peer);
                        }
                    }
                }
                let mut retired_combat_peers = Vec::new();
                for (&id, peer) in &mut peers {
                    if let Some(admission) = &peer.admission && !admission.live()? {
                        server.disconnect(id); retired_combat_peers.push(id); continue;
                    }
                    if current.duration_since(peer.period) >= Duration::from_secs(1) { peer.period = current; peer.received = 0; peer.cast_requests = 0; peer.inventory_requests = 0; }
                    let mut controls = 0;
                    let mut invalid_control = false;
                    for _ in 0..8 {
                        let Some(bytes) = server.receive_message(id, EVENT_CHANNEL) else { break; };
                        controls += 1; peer.received += 1;
                        if peer.received > 180 || (world.chunks().is_none() && world.procedural().is_none()) {
                            invalid_control = true; stats.rejected_chunk_ready += 1; break;
                        }
                        if bytes.starts_with(crate::dialogue_wire::REQUEST_TAG) {
                            let Some(dialogue)=&mut peer.dialogue else {stats.dialogue_requests_rejected+=1;invalid_control=true;break;};
                            if dialogue.request(&mut world,peer.actor,&bytes,&mut stats).is_err() {invalid_control=true;break;}
                            continue;
                        }
                        if bytes.starts_with(crate::inventory_wire::REQUEST_TAG) || bytes.starts_with(crate::inventory_wire::ITEM_REQUEST_TAG) || bytes.starts_with(crate::reward_wire::REQUEST_TAG) || bytes.starts_with(crate::vendor_wire::REQUEST_TAG) {
                            peer.inventory_requests+=1;
                            if peer.inventory_requests>8 {invalid_control=true;break;}
                            let Some(inventory)=&mut peer.inventory else {invalid_control=true;break;};
                            match inventory.request_in_world(&world,peer.actor,&bytes) {
                                Ok(true)=>stats.inventory_requests_forwarded+=1,
                                Ok(false)=>stats.inventory_requests_ignored+=1,
                                Err(_)=>{invalid_control=true;break;},
                            }
                            continue;
                        }
                        if bytes.starts_with(crate::combat_wire::REQUEST_TAG) {
                            peer.cast_requests += 1;
                            if !world.combat_enabled() || peer.cast_requests > 10 || combat_events.len()+2 > crate::simulation::combat::MAX_STEP_EVENTS {
                                stats.cast_requests_rejected += 1; invalid_control=true; break;
                            }
                            let command = match crate::combat_wire::Request::decode(&bytes, world.combat_catalog().unwrap())
                                .and_then(|message| message.command(world.combat_catalog().unwrap())) {
                                Ok(command)=>command,
                                Err(_)=>{stats.cast_requests_rejected+=1;invalid_control=true;break;}
                            };
                            let result=world.cast(peer.actor,command);
                            use crate::simulation::combat::Outcome;
                            match result.reply.outcome {
                                Outcome::Ignored=>stats.cast_requests_ignored+=1,
                                Outcome::Rejected(_)=>stats.cast_requests_rejected+=1,
                                Outcome::Started {..} | Outcome::Queued {..}=>stats.cast_requests_accepted+=1,
                            }
                            for fact in world.experience_opportunities(&result.events)? {
                                forward_experience_opportunity(&world, peer, &mut stats, fact)?;
                            }
                            for fact in world.reward_opportunities(&result.events)? {
                                forward_reward_opportunity(&world, peer, &mut stats, fact)?;
                            }
                            combat_events.extend(result.events);
                            continue;
                        }
                        if bytes.starts_with(crate::targeting_wire::SELECTION_TAG) {
                            if !world.targeting_enabled() { invalid_control=true; stats.target_selections_rejected+=1; break; }
                            let message = match crate::targeting_wire::Selection::decode(&bytes) {
                                Ok(message)=>message,
                                Err(_)=>{invalid_control=true;stats.target_selections_rejected+=1;break;}
                            };
                            if message.epoch!=world.epoch() || message.sequence<=peer.target_sequence
                                || message.sequence.saturating_sub(peer.target_sequence)>128 {
                                stats.target_selections_rejected+=1;continue;
                            }
                            peer.target_sequence=message.sequence;
                            peer.target_accepted=world.select_target(peer.actor,message.target.map(|entity|
                                theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(message.epoch).unwrap(),entity}));
                            if peer.target_accepted {stats.target_selections_accepted+=1;} else {stats.target_selections_rejected+=1;}
                            continue;
                        }
                        match ChunkReady::decode(&bytes) {
                            Ok(message) => {
                                if world.client_ready(peer.actor, &message) {
                                    stats.accepted_chunk_ready += 1;
                                    println!("chunk_ready actor={} sequence={} chunks={}", peer.actor, message.sequence, message.chunks.len());
                                    peer.pending_chunk_ack = Some(message);
                                } else { stats.rejected_chunk_ready += 1; }
                            }
                            Err(_) => { invalid_control = true; stats.rejected_chunk_ready += 1; break; }
                        }
                    }
                    stats.max_control_batch = stats.max_control_batch.max(controls);
                    if invalid_control { peer.latest = None; server.disconnect(id); if world.combat_enabled() { retired_combat_peers.push(id); } continue; }
                    let mut batch = 0;
                    for _ in 0..32 {
                        let Some(bytes) = server.receive_message(id, MOVEMENT_CHANNEL) else { break; };
                        peer.received += 1;
                        batch += 1; stats.received_inputs += 1;
                        if peer.received > 180 { stats.rejected_inputs += 1; peer.latest = None; server.disconnect(id); break; }
                        if world.prediction_enabled() {
                            match crate::prediction_wire::Commands::decode(&bytes) {
                                Ok(packet) => {stats.queued_commands += world.command(peer.actor,&packet) as u64;}
                                Err(_) => {stats.rejected_inputs += 1; server.disconnect(id); break;}
                            }
                            continue;
                        }
                        match Input::decode(&bytes) {
                            Ok(input) => {
                                if input.epoch != world.epoch() { stats.rejected_inputs += 1; continue; }
                                if peer.latest.is_none_or(|old| input.sequence > old.sequence) { peer.latest = Some(input); }
                            }
                            Err(_) => { stats.rejected_inputs += 1; peer.latest = None; server.disconnect(id); break; }
                        }
                    }
                    stats.max_input_batch = stats.max_input_batch.max(batch);
                    if let Some(chunks)=world.chunks() {
                        let generation=chunks.residency().generation();
                        if peer.offered_generation!=generation {
                            let offer=ChunkOffer {version:VERSION,epoch:world.epoch(),generation,chunks:chunks.residency().loaded().iter().map(|id|[id.x,id.z]).collect()};
                            let bytes=encode(&ChunkEvent::Offer(offer))?;
                            if server.can_send_message(id,EVENT_CHANNEL,bytes.len()) {
                                server.send_message(id,EVENT_CHANNEL,bytes);peer.offered_generation=generation;stats.chunk_offers_sent+=1;
                            }
                        }
                    }
                }
                {
                    retired_combat_peers.extend(peers.keys().copied().filter(|id| !server.is_connected(*id)));
                    retired_combat_peers.sort_unstable(); retired_combat_peers.dedup();
                    retire_peers(&mut server,&mut peers,&mut world,&retired_combat_peers)?;
                    if world.combat_enabled() {publish_combat_events(&mut server,&mut peers,&mut world,&mut stats,&combat_events)?;}
                    publish_inventory(&mut server,&mut peers,&mut world,&mut stats)?;
                    publish_progression(&mut server,&mut peers,&mut world,&mut stats)?;
                    publish_appearance(&mut server,&mut peers,&mut world,&mut stats)?;
                    dialogue::publish(&mut server,&mut peers,&mut world,&mut stats)?;
                    combat_events.clear();
                }
                stats.observe(&server, &peers);
                stats.max_players = stats.max_players.max(peers.len());
                if let Some(updates)=&demand
                    && let Some(next)=world.chunk_demand()
                    && world.chunks().is_some_and(|c|c.residency().loaded()!=next)
                    && updates.borrow().as_ref()!=Some(&next)
                {updates.send_replace(Some(next));}
                transport.send_packets(&mut server);
                // The I/O pump has a 5ms budget, independent of simulation rate.
                io_metrics.record_budget(Duration::from_millis(5), current.elapsed(), current.saturating_duration_since(scheduled));
            }
        }}
        Ok(())
    }.await;
    if let Some(bridge) = &mut checkpoints {
        stats.checkpoint_publications += u64::from(bridge.capture(&world, &peers));
    }
    let final_capture = peers
        .values()
        .try_for_each(|peer| admissions::capture(&world, peer));
    transport.disconnect_all(&mut server);
    pending.clear();
    peers.clear();
    println!(
        "simulation_summary transport=renet tick_hz={} steps={} missed_deadlines={} max_work_us={} max_lateness_us={}",
        rate.hz(),
        metrics.steps,
        metrics.missed_deadlines,
        metrics.max_work.as_micros(),
        metrics.max_lateness.as_micros()
    );
    result?;
    final_capture?;
    if let Some(chunks) = world.chunks() {
        stats.chunk_generation = chunks.residency().generation();
        stats.chunk_rejections = chunks.rejected();
        stats.chunk_blocked_steps = chunks.blocked();
        stats.chunk_unready_steps = chunks.unready_steps();
    }
    if let Some(terrain) = world.procedural() {
        eprintln!(
            "{}",
            serde_json::json!({"terrain_summary":{"saved_terrain":match &terrain.source {crate::procedural::Source::Saved(s)=>Some(s.stats()), _=>None},"generated":terrain.generated,"retired":terrain.retired,"resident":terrain.patches.len(),"movement_recoveries":world.movement_recoveries()}})
        );
    }
    stats.movement_recoveries = world.movement_recoveries();
    stats.wall_seconds = wall_start.elapsed().as_secs_f64();
    stats.tick_hz = rate.hz();
    stats.simulation = (&metrics).into();
    stats.io = (&io_metrics).into();
    Ok(stats)
}

pub type ChunkReadiness = watch::Receiver<Option<Vec<theia_core::coordinates::ChunkId>>>;

pub async fn run_client(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
) -> Result<()> {
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode::default(),
    )
    .await
}

/// Readiness is superseding local state, not a queue of asset completion events.
/// Only publish a set after actual visual/collision publication on the client.
pub async fn run_client_with_chunks(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    ready: ChunkReadiness,
) -> Result<()> {
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode {
            ready: Some(ready),
            ..ClientMode::default()
        },
    )
    .await
}

/// Opt-in profile 1. Commands originate from the fixed-step prediction owner,
/// with a bounded queue; the I/O worker never invents gameplay steps.
#[allow(clippy::too_many_arguments)]
pub async fn run_client_predicted(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    ready: ChunkReadiness,
    commands: mpsc::Receiver<crate::prediction_wire::Commands>,
) -> Result<()> {
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode {
            ready: Some(ready),
            commands: Some(commands),
            choices: None,
            combat: None,
            dialogue: None,
            quest: None,
            progression: None,
            inventory: None,
            statuses: None,
        },
    )
    .await
}

pub type TargetChoices = watch::Receiver<Option<crate::targeting::Choice>>;
/// The opt-in profile owns its readiness, optional prediction and target mailbox.
pub struct TargetingClient {
    pub ready: ChunkReadiness,
    pub commands: Option<mpsc::Receiver<crate::prediction_wire::Commands>>,
    pub choices: TargetChoices,
}
#[derive(Default)]
struct ClientMode {
    dialogue: Option<dialogue::Client>,
    ready: Option<ChunkReadiness>,
    commands: Option<mpsc::Receiver<crate::prediction_wire::Commands>>,
    choices: Option<TargetChoices>,
    combat: Option<CombatActions>,
    quest: Option<std::sync::Arc<crate::quest_wire::Profile>>,
    progression: Option<std::sync::Arc<crate::progression_wire::Profile>>,
    inventory: Option<inventory::Client>,
    statuses: Option<std::sync::Arc<theia_core::statuses::Catalog>>,
}
/// expected_world includes targeting::profile_identity; baseline wrappers remain unchanged.
pub async fn run_client_targeted(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    mode: TargetingClient,
) -> Result<()> {
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode {
            ready: Some(mode.ready),
            commands: mode.commands,
            choices: Some(mode.choices),
            combat: None,
            dialogue: None,
            quest: None,
            progression: None,
            inventory: None,
            statuses: None,
        },
    )
    .await
}

/// A bounded action queue, unlike superseding movement/selection mailboxes.
#[derive(Clone, Debug)]
pub struct CastChoice {
    pub ability: theia_core::content::ContentId,
    pub target: theia_core::identity::EntityId,
}
pub struct CombatClient {
    pub targeting: TargetingClient,
    pub catalog: std::sync::Arc<theia_core::abilities::Catalog>,
    pub casts: mpsc::Receiver<CastChoice>,
    pub events: mpsc::Sender<crate::combat_wire::Batch>,
}
pub struct CombatActions {
    pub catalog: std::sync::Arc<theia_core::abilities::Catalog>,
    pub casts: mpsc::Receiver<CastChoice>,
    pub events: mpsc::Sender<crate::combat_wire::Batch>,
}
pub async fn run_client_combat(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    mode: CombatClient,
) -> Result<()> {
    if mode.casts.max_capacity() > 8 || mode.events.max_capacity() > 16 {
        return Err("Combat action/event queue exceeds prototype budget".into());
    }
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode {
            ready: Some(mode.targeting.ready),
            commands: mode.targeting.commands,
            choices: Some(mode.targeting.choices),
            combat: Some(CombatActions {
                catalog: mode.catalog,
                casts: mode.casts,
                events: mode.events,
            }),
            dialogue: None,
            quest: None,
            progression: None,
            inventory: None,
            statuses: None,
        },
    )
    .await
}
pub struct GameplayClient {
    pub dialogue: Option<dialogue::Client>,
    pub ready: ChunkReadiness,
    pub commands: Option<mpsc::Receiver<crate::prediction_wire::Commands>>,
    pub choices: Option<TargetChoices>,
    pub combat: Option<CombatActions>,
    pub quest: Option<std::sync::Arc<crate::quest_wire::Profile>>,
    pub progression: Option<std::sync::Arc<crate::progression_wire::Profile>>,
    pub inventory: Option<inventory::Client>,
    pub statuses: Option<std::sync::Arc<theia_core::statuses::Catalog>>,
}
/// Combined opt-in gameplay profiles. Existing baseline wrappers remain valid.
pub async fn run_client_gameplay(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    mode: GameplayClient,
) -> Result<()> {
    run_client_inner(
        address,
        ticket,
        input,
        state,
        shutdown,
        expected_world,
        ClientMode {
            ready: Some(mode.ready),
            commands: mode.commands,
            choices: mode.choices,
            combat: mode.combat,
            dialogue: mode.dialogue,
            quest: mode.quest,
            progression: mode.progression,
            inventory: mode.inventory,
            statuses: mode.statuses,
        },
    )
    .await
}
async fn run_client_inner(
    address: SocketAddr,
    ticket: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    mut shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
    mode: ClientMode,
) -> Result<()> {
    let ClientMode {
        ready,
        mut commands,
        mut choices,
        mut combat,
        inventory,
        dialogue,
        progression,
        quest,
        statuses,
    } = mode;
    if quest.is_some() && progression.is_none() {
        return Err("Objectives require progression profile".into());
    }
    if statuses.is_some() && combat.is_none() {
        return Err("Statuses require the combat profile".into());
    }
    let mut dialogue = dialogue.map(dialogue::ClientState::new).transpose()?;
    let mut inventory = inventory.map(inventory::ClientState::new).transpose()?;
    if combat
        .as_ref()
        .is_some_and(|m| m.casts.max_capacity() > 8 || m.events.max_capacity() > 16)
    {
        return Err("Combat action/event queue exceeds prototype budget".into());
    }
    let token = read_ticket(ticket, address)?;
    let bind = if address.is_ipv4() {
        "127.0.0.1:0"
    } else {
        "[::1]:0"
    };
    let mut transport = secure_client(UdpSocket::bind(bind)?, now()?, token)?;
    let mut client = RenetClient::new(connection_config());
    let mut io = tokio::time::interval(Duration::from_millis(5));
    io.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last = Instant::now();
    let started = last;
    let mut next_input = last;
    let mut sequence = 0u64;
    let mut target_sequence = 0u64;
    let mut cast_sequence = 0u64;
    let mut combat_event_sequence = 0u64;
    let mut combat_state: Option<crate::combat_wire::State> = None;
    let mut quest_state: Option<crate::quest_wire::State> = None;
    let mut progression_state: Option<crate::progression_wire::State> = None;
    let mut status_state: Option<crate::status_wire::State> = None;
    let mut targeting: Option<crate::targeting_wire::State> = None;
    let mut ready_sequence = 0u64;
    let mut last_ready = None;
    let mut last_report: Option<ChunkReady> = None;
    let mut last_ack: Option<ChunkAck> = None;
    let mut last_offer: Option<ChunkOffer> = None;
    let mut id = None;
    let mut admission: Option<Welcome> = None;
    let mut last_origin = None;
    let mut rebases = 0;
    let mut latest_tick = 0;
    let mut correction: Option<crate::prediction_wire::Corrected> = None;
    let mut correction_received = None;
    let mut correction_ready = None;
    let mut ready_history = BTreeMap::<u64, ChunkReady>::new();
    let mut stats = TransportStats::default();
    let result: Result<()> = async {
        loop { tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            _ = io.tick() => {
                let current = Instant::now();
                let delta = current.saturating_duration_since(last); last = current;
                client.update(delta);
                transport.update(delta, &mut client)?;
                if id.is_none() && started.elapsed() > Duration::from_secs(5) { return Err("Renet welcome timed out".into()); }
                if commands.is_some() && id.is_some() && correction.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not enable prediction profile 1".into());}
                if combat.is_some() && id.is_some() && combat_state.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not enable combat profile 2".into());}
                if choices.is_some() && id.is_some() && targeting.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not enable targeting profile 1".into());}
                if id.is_some() && let Some(mode)=&inventory {mode.check_deadline(current,started)?;}
                if let Some(mode)=&dialogue {mode.check(started.elapsed(),id.is_some(),current)?;}
                if quest.is_some() && id.is_some() && quest_state.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not publish committed objective".into());}
                if progression.is_some() && id.is_some() && progression_state.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not publish committed XP".into());}
                if statuses.is_some() && id.is_some() && status_state.is_none() && started.elapsed()>Duration::from_secs(8) {return Err("Server did not enable status profile 1".into());}
                for _ in 0..8 {
                    let Some(bytes) = client.receive_message(EVENT_CHANNEL) else { break; };
                    if id.is_none() {
                        let welcome=Welcome::decode(&bytes, expected_world)?;
                        if commands.is_some() && !welcome.terrain_stream {return Err("Prediction requires streamed terrain".into());}
                        id=Some(welcome.id); last_origin=Some(welcome.origin);
                        admission=Some(welcome.clone());
                        state.send_replace(State {id,welcome:Some(welcome),..State::default()});
                    } else {
                        if dialogue::is_frame(&bytes) {
                            let mode=dialogue.as_mut().ok_or("Unexpected dialogue observation")?;
                            mode.receive(&bytes,admission.as_ref().ok_or("Dialogue before admission")?,&mut stats)?;
                            state.send_modify(|s|{mode.copy_to(s);s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::quest_wire::STATE_TAG) {
                            let profile=quest.as_ref().ok_or("Unexpected objective observation")?;
                            let view=crate::quest_wire::State::decode(&bytes,profile)?;
                            let welcome=admission.as_ref().unwrap();
                            let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                            view.validate_successor(actor,quest_state.as_ref())?;
                            quest_state=Some(view);stats.quest_states_received+=1;
                            state.send_modify(|s|{s.quest=quest_state.clone();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::progression_wire::STATE_TAG) {
                            let profile=progression.as_ref().ok_or("Unexpected XP observation")?;
                            let view=crate::progression_wire::State::decode(&bytes,profile)?;
                            let welcome=admission.as_ref().unwrap();
                            let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                            view.validate_successor(actor,progression_state.as_ref())?;
                            progression_state=Some(view);stats.progression_states_received+=1;
                            state.send_modify(|s|{s.progression=progression_state.clone();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::status_wire::STATE_TAG) {
                            let catalog=statuses.as_ref().ok_or("Unexpected opt-in status state")?;
                            let view=crate::status_wire::State::decode(&bytes,catalog)?;
                            if view.epoch!=admission.as_ref().unwrap().epoch {return Err("Foreign status epoch".into());}
                            if state.borrow().snapshot.as_ref().is_some_and(|s|s.tick_hz!=view.rate.hz()) {return Err("Status tick rate disagrees with authority snapshot".into());}
                            if status_state.as_ref().is_some_and(|old|view.tick<old.tick) {continue;}
                            status_state=Some(view);stats.status_states_received+=1;
                            state.send_modify(|s|{s.statuses=status_state.clone();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::reward_wire::STATE_TAG) {
                            let mode=inventory.as_mut().ok_or("Unexpected opt-in reward state")?;
                            let welcome=admission.as_ref().unwrap();
                            let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                            mode.receive_rewards(&bytes,actor)?;
                            state.send_modify(|s|{s.rewards=mode.rewards.clone();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::appearance_wire::STATE_TAG) {
                            let mode=inventory.as_mut().ok_or("Unexpected opt-in appearance state")?;
                            let welcome=admission.as_ref().unwrap();
                            let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                            mode.receive_appearance(&bytes,actor)?;stats.appearance_states_received+=1;
                            state.send_modify(|s|{s.appearance=mode.appearance.clone();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::inventory_wire::STATE_TAG) || bytes.starts_with(crate::inventory_wire::ITEM_STATE_TAG) {
                            let mode=inventory.as_mut().ok_or("Unexpected opt-in inventory state")?;
                            let welcome=admission.as_ref().unwrap();
                            let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                            mode.receive(&bytes,actor)?;stats.inventory_states_received+=1;
                            state.send_modify(|s|{s.inventory=mode.view.clone();s.inventory_pending=mode.pending();s.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::combat_wire::STATE_TAG) || bytes.starts_with(crate::combat_wire::RESOURCE_STATE_TAG) {
                            let mode=combat.as_ref().ok_or("Unexpected opt-in combat state")?;
                            let view=crate::combat_wire::State::decode(&bytes,&mode.catalog)?;
                            if view.epoch!=admission.as_ref().unwrap().epoch || view.reply.is_some_and(|r| r.sequence>cast_sequence) {
                                return Err("Foreign or unsent combat acknowledgement".into());
                            }
                            if combat_state.as_ref().is_some_and(|old| view.tick<old.tick || view.reply.map_or(0,|r| r.sequence)<old.reply.map_or(0,|r| r.sequence)) {continue;}
                            combat_state=Some(view);stats.combat_states_received+=1;
                            state.send_modify(|value| {value.combat=combat_state.clone();value.transport=Some(stats.clone());});
                            continue;
                        }
                        if bytes.starts_with(crate::combat_wire::EVENTS_TAG) {
                            let mode=combat.as_ref().ok_or("Unexpected opt-in combat events")?;
                            let batch=crate::combat_wire::Batch::decode(&bytes,&mode.catalog)?;
                            if batch.epoch!=admission.as_ref().unwrap().epoch || batch.sequence!=combat_event_sequence.checked_add(1).ok_or("Combat event sequence exhausted")? {
                                return Err("Foreign or discontinuous combat event stream".into());
                            }
                            let next=batch.sequence;let count=batch.events.len();
                            mode.events.try_send(batch).map_err(|_| "Combat presentation queue full or closed; disconnecting instead of dropping gameplay events")?;
                            combat_event_sequence=next;stats.combat_events_received+=count as u64;
                            state.send_modify(|value| value.transport=Some(stats.clone()));
                            continue;
                        }
                        if bytes.starts_with(crate::targeting_wire::STATE_TAG) {
                            if choices.is_none() {return Err("Unexpected opt-in targeting state".into());}
                            let view=crate::targeting_wire::State::decode(&bytes)?;
                            if view.epoch!=admission.as_ref().unwrap().epoch || view.request_sequence>target_sequence {
                                return Err("Foreign or unsent target acknowledgement".into());
                            }
                            if targeting.as_ref().is_some_and(|old| view.tick<old.tick || view.request_sequence<old.request_sequence) {continue;}
                            targeting=Some(view);stats.target_states_received+=1;
                            state.send_modify(|value| {value.targeting=targeting.clone();value.transport=Some(stats.clone());});
                            continue;
                        }
                        let ack=match ChunkEvent::decode(&bytes)? {
                            ChunkEvent::Ack(ack)=>ack,
                            ChunkEvent::Offer(offer)=>{
                                if offer.epoch!=admission.as_ref().unwrap().epoch {return Err("Foreign chunk offer".into());}
                                if last_offer.as_ref().is_some_and(|old| offer.generation<=old.generation) {continue;}
                                last_offer=Some(offer);stats.chunk_offers_received+=1;
                                state.send_modify(|value| {value.chunk_offer=last_offer.clone(); value.transport=Some(stats.clone());});
                                continue;
                            }
                        };
                        if ack.epoch != admission.as_ref().unwrap().epoch || ack.sequence > ready_sequence {
                            return Err("Unsent or foreign chunk acknowledgement".into());
                        }
                        // A superseded report cannot release geometry for the
                        // current request. Ordered delivery can still lag it.
                        if ack.sequence < ready_sequence { continue; }
                        if !last_report.as_ref().is_some_and(|report| ack.matches(report))
                            || last_ack.as_ref().is_some_and(|old| ack.tick < old.tick)
                        {
                            return Err("Mismatched chunk acknowledgement".into());
                        }
                        if last_ack.as_ref() != Some(&ack) {
                            last_ack=Some(ack);
                            stats.chunk_ready_acks_received += 1;
                            state.send_modify(|value| {
                                value.chunk_ack=last_ack.clone();
                                value.transport=Some(stats.clone());
                            });
                        }
                    }
                }
                // Snapshots may arrive before reliable welcome; discard those
                // rather than publishing an identity-less presentation state.
                while let Some(bytes) = client.receive_message(MOVEMENT_CHANNEL) {
                    if bytes.starts_with(crate::prediction_wire::CORRECTION_TAG) {
                        if commands.is_none() {return Err("Server requires the opt-in prediction profile".into());}
                        let packet=crate::prediction_wire::Corrected::decode(&bytes)?;
                        let Some(welcome)=&admission else {continue;};
                        if packet.state.world!=welcome.world || packet.state.actor.epoch.value()!=welcome.epoch || packet.state.actor.entity.value()!=welcome.id || packet.state.ready_sequence>ready_sequence || crate::procedural::owner(packet.state.pose.feet).is_none() || packet.state.pose.feet.metres()[1].abs()>1023. {return Err("Foreign prediction correction".into());}
                        if correction.is_some_and(|old|packet.state.input.stream_epoch<old.state.input.stream_epoch || packet.state.input.tick<old.state.input.tick || (packet.state.input.stream_epoch==old.state.input.stream_epoch && packet.state.input.tick==old.state.input.tick)) {continue;}
                        correction_ready=ready_history.get(&packet.state.ready_sequence).cloned();
                        correction=Some(packet);correction_received=Some(current.into_std());
                        state.send_modify(|value| {value.correction=correction;value.correction_received=correction_received;value.correction_ready=correction_ready.clone();});
                        continue;
                    }
                    let snapshot = Snapshot::decode(&bytes)?;
                    if status_state.as_ref().is_some_and(|s|s.rate.hz()!=snapshot.tick_hz) {return Err("Authority snapshot disagrees with status tick rate".into());}
                    stats.snapshots_received += 1;
                    if snapshot.tick <= latest_tick { stats.stale_snapshots_discarded += 1; }
                    if let Some(actor) = id && snapshot.tick > latest_tick {
                        let welcome=admission.as_ref().expect("Identity requires welcome");
                        if !welcome.accepts(&snapshot) {return Err("Snapshot epoch, actor or frame mismatch".into());}
                        if Some(snapshot.origin)!=last_origin { rebases+=1; last_origin=Some(snapshot.origin); }
                        debug_assert_eq!(welcome.id,actor);
                        latest_tick = snapshot.tick;
                        let info = client.network_info();
                        stats.rtt_ms = info.rtt * 1000.; stats.packet_loss = info.packet_loss;
                        stats.bytes_sent_per_second = info.bytes_sent_per_second;
                        stats.bytes_received_per_second = info.bytes_received_per_second;
                        let mut update=State { dialogue:None, guide:None, dialogue_reply:None, dialogue_pending:false, quest:quest_state.clone(), progression:progression_state.clone(), rewards:inventory.as_ref().and_then(|m|m.rewards.clone()), appearance:inventory.as_ref().and_then(|m|m.appearance.clone()), statuses:status_state.clone(), inventory:inventory.as_ref().and_then(|m|m.view.clone()), inventory_pending:inventory.as_ref().is_some_and(|m|m.pending()), combat:combat_state.clone(), targeting:targeting.clone(), id, snapshot: Some(snapshot), error: None, transport: Some(stats.clone()), welcome:admission.clone(), wire_rebases:rebases, chunk_report:last_report.clone(), chunk_ack:last_ack.clone(),chunk_offer:last_offer.clone(),correction,correction_received,correction_ready:correction_ready.clone() };
                        if let Some(mode)=&dialogue {mode.copy_to(&mut update);}
                        state.send_replace(update);
                    }
                }
                if let (Some(mode),Some(welcome))=(&mut dialogue,&admission)
                    && mode.send(&mut client,welcome,current,&mut stats)? {
                    state.send_modify(|s|{mode.copy_to(s);s.transport=Some(stats.clone());});
                }
                if commands.is_none() && id.is_some() && current >= next_input {
                    next_input = current + Duration::from_secs_f64(1. / f64::from(TICK_HZ));
                    sequence = sequence.checked_add(1).ok_or("Input sequence exhausted")?;
                    let intent = *input.borrow();
                    if !intent.valid() { return Err("Invalid local intent".into()); }
                    client.send_message(MOVEMENT_CHANNEL, encode(&Input { version: VERSION, epoch:admission.as_ref().unwrap().epoch, sequence, intent })?);
                }
                if let Some(queue)=&mut commands {
                    let mut latest=None;
                    for _ in 0..8 {match queue.try_recv() {Ok(packet)=>latest=Some(packet),Err(_)=>break,}}
                    if let (Some(welcome),Some(packet))=(&admission,latest) {
                        if packet.epoch!=welcome.epoch {return Err("Foreign local prediction commands".into());}
                        let bytes=packet.encode()?;
                        if client.can_send_message(MOVEMENT_CHANNEL,bytes.len()) {client.send_message(MOVEMENT_CHANNEL,bytes);}
                    }
                }
                if let (Some(welcome),Some(mode))=(&admission,&mut inventory)
                    && client.can_send_message(EVENT_CHANNEL,crate::reward_wire::REQUEST_BYTES) {
                    let actor=theia_core::identity::EntityKey {epoch:theia_core::identity::WorldEpoch::new(welcome.epoch).unwrap(),entity:theia_core::identity::EntityId::new(welcome.id).unwrap()};
                    if let Some(bytes)=mode.next(actor,current)? {
                        client.send_message(EVENT_CHANNEL,bytes);stats.inventory_requests_sent+=1;
                        state.send_modify(|s|{s.inventory_pending=true;s.transport=Some(stats.clone());});
                    }
                }
                if let (Some(welcome),Some(mode))=(&admission,&mut combat) {
                    for _ in 0..4 {
                        if !client.can_send_message(EVENT_CHANNEL,crate::combat_wire::REQUEST_BYTES) {break;}
                        let Ok(choice)=mode.casts.try_recv() else {break;};
                        let next=cast_sequence.checked_add(1).ok_or("Cast request sequence exhausted")?;
                        let request=crate::combat_wire::Request::new(welcome.epoch,next,&choice.ability,choice.target,&mode.catalog)?;
                        client.send_message(EVENT_CHANNEL,request.encode()?);cast_sequence=next;stats.cast_requests_sent+=1;
                    }
                }
                if let Some(welcome)=&admission
                    && let Some(choices)=&mut choices
                    && (target_sequence==0 || choices.has_changed().unwrap_or(false))
                    && client.can_send_message(EVENT_CHANNEL,crate::targeting_wire::SELECTION_BYTES)
                    && let Some(choice)=*choices.borrow_and_update() {
                    // Consume the mailbox atomically only after capacity exists.
                    // Repeating an explicit choice is a fresh user request even
                    // if relevance previously cleared that same selected ID.
                    let next=target_sequence.checked_add(1).ok_or("Target sequence exhausted")?;
                    let message=crate::targeting_wire::Selection {epoch:welcome.epoch,sequence:next,target:choice.target};
                    client.send_message(EVENT_CHANNEL,message.encode()?);target_sequence=next;
                    stats.target_selections_sent+=1;
                }
                if let Some(welcome) = &admission
                    && let Some(chunks) = ready.as_ref().and_then(|r|r.borrow().clone())
                    && last_ready.as_ref() != Some(&chunks)
                {
                    let next = ready_sequence.checked_add(1).ok_or("Readiness sequence exhausted")?;
                    let message = ChunkReady {version:VERSION, epoch:welcome.epoch, sequence:next, chunks:chunks.iter().map(|id|[id.x,id.z]).collect()};
                    if !message.valid() { return Err("Invalid local chunk readiness".into()); }
                    let bytes = encode(&message)?;
                    if client.can_send_message(EVENT_CHANNEL, bytes.len()) {
                        client.send_message(EVENT_CHANNEL, bytes);
                        ready_sequence = next;
                        last_ready = Some(chunks);
                        stats.chunk_ready_reports_sent += 1;
                        ready_history.insert(next,message.clone());
                        while ready_history.len()>8 {ready_history.pop_first();}
                        last_report = Some(message);
                        state.send_modify(|value| {
                            value.chunk_report = last_report.clone();
                            value.transport = Some(stats.clone());
                        });
                    }
                }
                stats.max_movement_queue_bytes = stats.max_movement_queue_bytes.max(CHANNEL_MEMORY_BYTES - client.channel_available_memory(MOVEMENT_CHANNEL));
                stats.max_event_queue_bytes = stats.max_event_queue_bytes.max(CHANNEL_MEMORY_BYTES - client.channel_available_memory(EVENT_CHANNEL));
                transport.send_packets(&mut client)?;
            }
        }}
        Ok(())
    }.await;
    transport.disconnect();
    result
}
