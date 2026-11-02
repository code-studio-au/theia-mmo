//! Owner-only critical life state. Full queues retire/disconnect instead of
//! silently losing windup/death/respawn transitions. No client damage command.
use super::{EVENT_CHANNEL, Peer, RenetServer, ServerStats, retire_peers};
use crate::{
    Result,
    client::TransportStats,
    player_life_wire::{self as wire, Applied, ClientState, Profile},
    simulation::{World, combat::player_life::Observation},
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
    pub membership: Option<Arc<crate::combat_membership_wire::Profile>>,
    pub events: mpsc::Sender<Observation>,
}
impl Client {
    pub fn validate(&self) -> Result<()> {
        if self.events.max_capacity() != PRESENTATION_CAPACITY || self.events.is_closed() {
            return Err("Invalid life presentation queue".into());
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
    let Some(profile) = world.player_life_profile().cloned() else {
        return Ok(());
    };
    if let Some(error) = world.player_life_error() {
        return Err(error.to_owned().into());
    }
    if let Some(error) = world.creature_reset_error() {
        return Err(error.to_owned().into());
    }
    let membership = world.combat_membership_profile().cloned();
    if let Some(error) = world.combat_membership_error() {
        return Err(error.to_owned().into());
    }
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        let view = world
            .player_life_snapshot(peer.actor)
            .ok_or("Player life peer has no own actor")?;
        if peer.life_sent.is_some_and(|old| {
            let mut same = view;
            same.tick = old.tick;
            same == old
        }) {
            continue;
        }
        let bytes = if let Some(membership) = &membership {
            crate::combat_membership_wire::encode(&profile, membership, world.tick_rate(), &view)?
        } else {
            wire::encode(&profile, world.tick_rate(), &view)?
        };
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
            slow.push(id);
            continue;
        }
        stats.life_max_frame_bytes = stats.life_max_frame_bytes.max(bytes.len());
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.life_sent = Some(view);
        stats.life_states_sent = stats.life_states_sent.saturating_add(1);
    }
    stats.life_backpressure_disconnects = stats
        .life_backpressure_disconnects
        .saturating_add(slow.len() as u64);
    retire_peers(server, peers, world, &slow)?;
    stats.player_life = world.player_life_metrics();
    Ok(())
}
pub(super) fn receive(
    mode: &Client,
    welcome: &Welcome,
    rates: [Option<TickRate>; 2],
    bytes: &[u8],
    current: &mut Option<ClientState>,
    stats: &mut TransportStats,
) -> Result<()> {
    let rate = if mode.membership.is_some() {
        crate::combat_membership_wire::frame_rate(bytes)?
    } else {
        wire::frame_rate(bytes)?
    };
    if rates.into_iter().flatten().any(|expected| expected != rate) {
        return Err("Player-life rate disagrees with movement".into());
    }
    let actor = EntityKey {
        epoch: WorldEpoch::new(welcome.epoch).ok_or("Missing life world incarnation")?,
        entity: EntityId::new(welcome.id).ok_or("Missing life actor")?,
    };
    let mut next = match current {
        Some(value) => value.clone(),
        None => {
            if let Some(membership) = &mode.membership {
                ClientState::with_membership(
                    mode.profile.clone(),
                    membership.clone(),
                    actor,
                    rate,
                    welcome.world,
                )?
            } else {
                ClientState::new(mode.profile.clone(), actor, rate, welcome.world)?
            }
        }
    };
    match next.apply(bytes)? {
        Applied::Changed => {
            let event = *next
                .current()
                .ok_or("Life state missing after validation")?;
            mode.events.try_send(event).map_err(|_|"Life presentation queue full or closed; disconnecting instead of dropping gameplay transitions")?;
            *current = Some(next);
            stats.life_states_received = stats.life_states_received.saturating_add(1);
        }
        Applied::Stale => stats.life_stale_states = stats.life_stale_states.saturating_add(1),
        Applied::Duplicate => {
            stats.life_duplicate_states = stats.life_duplicate_states.saturating_add(1)
        }
    }
    Ok(())
}

#[cfg(test)]
mod renet_tests;

#[cfg(test)]
mod membership_tests;
