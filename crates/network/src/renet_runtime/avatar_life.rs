//! Optional public visual lifecycle; no health or client-authority command.
use super::{Peer, RenetServer, ServerStats, retire_peers};
use crate::{
    Result,
    avatar_life_wire::{self as wire, Applied, ClientState, Observation, Profile},
    client::TransportStats,
    renet_transport::{CHANNEL_MEMORY_BYTES, CRITICAL_PRESENTATION_CHANNEL},
    simulation::World,
    wire::Welcome,
};
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
use tokio::sync::mpsc;

pub const PRESENTATION_CAPACITY: usize = 16;
pub struct Client {
    pub profile: Arc<Profile>,
    pub events: mpsc::Sender<Observation>,
}
impl Client {
    pub fn validate(&self) -> Result<()> {
        if self.events.max_capacity() != PRESENTATION_CAPACITY || self.events.is_closed() {
            return Err("Invalid public avatar life presentation queue".into());
        }
        Ok(())
    }
}
pub(super) fn publish(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &mut World,
    stats: &mut ServerStats,
) -> Result<()> {
    let Some(profile) = world.public_avatar_life_profile().cloned() else {
        return Ok(());
    };
    let view = world
        .public_avatar_life_snapshot()?
        .ok_or("Missing public avatar life observation")?;
    let bytes = wire::encode(&profile, world.tick_rate(), &view)?;
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        if !peer.public_presentation_ready
            || peer.avatar_life_sent.as_ref().is_some_and(|old| {
                let mut same = view.clone();
                same.tick = old.tick;
                same == *old
            })
        {
            continue;
        }
        if !server.can_send_message(id, CRITICAL_PRESENTATION_CHANNEL, bytes.len()) {
            slow.push(id);
            continue;
        }
        server.send_message(id, CRITICAL_PRESENTATION_CHANNEL, bytes.clone());
        stats.avatar_life_max_queue_bytes = stats.avatar_life_max_queue_bytes.max(
            CHANNEL_MEMORY_BYTES
                - server.channel_available_memory(id, CRITICAL_PRESENTATION_CHANNEL),
        );
        stats.avatar_life_max_frame_bytes = stats.avatar_life_max_frame_bytes.max(bytes.len());
        stats.avatar_life_states_sent = stats.avatar_life_states_sent.saturating_add(1);
        peer.avatar_life_sent = Some(view.clone());
    }
    stats.avatar_life_backpressure_disconnects = stats
        .avatar_life_backpressure_disconnects
        .saturating_add(slow.len() as u64);
    retire_peers(server, peers, world, &slow)
}
pub(super) fn receive(
    mode: &Client,
    welcome: &Welcome,
    rates: [Option<TickRate>; 2],
    bytes: &[u8],
    current: &mut Option<ClientState>,
    stats: &mut TransportStats,
) -> Result<()> {
    let rate = wire::frame_rate(bytes)?;
    if rates.into_iter().flatten().any(|expected| expected != rate) {
        return Err("Public avatar life rate disagrees with movement".into());
    }
    let mut next = match current {
        Some(value) => value.clone(),
        None => ClientState::new(
            mode.profile.clone(),
            EntityKey {
                epoch: WorldEpoch::new(welcome.epoch)
                    .ok_or("Missing public avatar life world incarnation")?,
                entity: EntityId::new(welcome.id).ok_or("Missing public avatar owner")?,
            },
            rate,
            welcome.world,
        )?,
    };
    match next.apply(bytes)? {
        Applied::Changed => {
            let event = next
                .current()
                .ok_or("Public avatar life missing after validation")?
                .clone();
            mode.events.try_send(event).map_err(|_| "Public avatar life presentation queue full or closed; disconnecting instead of dropping critical timing")?;
            *current = Some(next);
            stats.avatar_life_states_received = stats.avatar_life_states_received.saturating_add(1);
            stats.avatar_life_max_frame_bytes = stats.avatar_life_max_frame_bytes.max(bytes.len());
        }
        Applied::Duplicate => {
            stats.avatar_life_duplicate_states =
                stats.avatar_life_duplicate_states.saturating_add(1)
        }
        Applied::Stale => {
            stats.avatar_life_stale_states = stats.avatar_life_stale_states.saturating_add(1)
        }
    }
    Ok(())
}
