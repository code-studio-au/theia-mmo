//! Opt-in critical public NPC timing. Never carries private vitals or accepts
//! client attack facts. Publication starts only after validated chunk readiness.
use super::{Peer, RenetServer, ServerStats, retire_peers};
use crate::{
    Result,
    client::TransportStats,
    creature_attack_wire::{self as wire, Applied, ClientState, Observation, Profile},
    renet_transport::{CHANNEL_MEMORY_BYTES, CREATURE_ATTACK_CHANNEL},
    simulation::World,
    wire::Welcome,
};
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{identity::WorldEpoch, time::TickRate};
use tokio::sync::mpsc;

pub const PRESENTATION_CAPACITY: usize = 16;
pub struct Client {
    pub profile: Arc<Profile>,
    pub events: mpsc::Sender<Observation>,
}
impl Client {
    pub fn validate(&self) -> Result<()> {
        if self.events.max_capacity() != PRESENTATION_CAPACITY || self.events.is_closed() {
            return Err("Invalid public attack presentation queue".into());
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
    let Some(profile) = world.public_creature_attack_profile().cloned() else {
        return Ok(());
    };
    let view = world
        .public_creature_attack_snapshot()?
        .ok_or("Missing public attack observation")?;
    let bytes = wire::encode(&profile, world.tick_rate(), &view)?;
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        if !peer.public_presentation_ready
            || peer.public_attack_sent.is_some_and(|old| {
                let mut same = view;
                same.tick = old.tick;
                same == old
            })
        {
            continue;
        }
        if !server.can_send_message(id, CREATURE_ATTACK_CHANNEL, bytes.len()) {
            slow.push(id);
            continue;
        }
        server.send_message(id, CREATURE_ATTACK_CHANNEL, bytes.clone());
        stats.public_attack_max_queue_bytes = stats.public_attack_max_queue_bytes.max(
            CHANNEL_MEMORY_BYTES - server.channel_available_memory(id, CREATURE_ATTACK_CHANNEL),
        );
        stats.public_attack_max_frame_bytes = stats.public_attack_max_frame_bytes.max(bytes.len());
        stats.public_attack_states_sent = stats.public_attack_states_sent.saturating_add(1);
        peer.public_attack_sent = Some(view);
    }
    stats.public_attack_backpressure_disconnects = stats
        .public_attack_backpressure_disconnects
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
        return Err("Public attack rate disagrees with movement".into());
    }
    let mut next = match current {
        Some(value) => value.clone(),
        None => ClientState::new(
            mode.profile.clone(),
            WorldEpoch::new(welcome.epoch).ok_or("Missing public attack world incarnation")?,
            rate,
            welcome.world,
        )?,
    };
    match next.apply(bytes)? {
        Applied::Changed => {
            let event = *next
                .current()
                .ok_or("Public attack missing after validation")?;
            mode.events.try_send(event).map_err(|_| "Public attack presentation queue full or closed; disconnecting instead of dropping critical timing")?;
            *current = Some(next);
            stats.public_attack_states_received =
                stats.public_attack_states_received.saturating_add(1);
            stats.public_attack_max_frame_bytes =
                stats.public_attack_max_frame_bytes.max(bytes.len());
        }
        Applied::Duplicate => {
            stats.public_attack_duplicate_states =
                stats.public_attack_duplicate_states.saturating_add(1)
        }
        Applied::Stale => {
            stats.public_attack_stale_states = stats.public_attack_stale_states.saturating_add(1)
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
