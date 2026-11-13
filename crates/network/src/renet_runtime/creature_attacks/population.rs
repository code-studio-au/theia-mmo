//! Population timing on the reliable priority channel and bounded presentation handoff.
//! Readiness gates publication; queue pressure is an explicit failure.
use crate::{
    Result,
    client::TransportStats,
    creature_attack_wire::{
        Applied,
        population::{self as wire, Frame, PopulationProfile, PopulationState},
    },
    wire::Welcome,
};
use std::sync::Arc;
use theia_core::{identity::WorldEpoch, time::TickRate};
use tokio::sync::mpsc;
pub const PRESENTATION_CAPACITY: usize = super::PRESENTATION_CAPACITY;
pub struct Client {
    pub profile: Arc<PopulationProfile>,
    pub events: mpsc::Sender<Frame>,
}
impl Client {
    pub fn validate(&self) -> Result<()> {
        if self.events.max_capacity() != PRESENTATION_CAPACITY || self.events.is_closed() {
            return Err("Invalid population attack presentation queue".into());
        }
        Ok(())
    }
}
pub fn receive(
    mode: &Client,
    welcome: &Welcome,
    rates: [Option<TickRate>; 2],
    bytes: &[u8],
    current: &mut Option<PopulationState>,
    stats: &mut TransportStats,
) -> Result<()> {
    mode.validate()?;
    let rate = wire::frame_rate(bytes)?;
    let epoch =
        WorldEpoch::new(welcome.epoch).ok_or("Missing population attack world incarnation")?;
    if rates.into_iter().flatten().any(|r| r != rate) {
        return Err("Population attack rate disagrees with movement".into());
    }
    let mut next = match current {
        Some(value) => {
            if value.world() != welcome.world
                || value.epoch() != epoch
                || value.rate() != rate
                || value.profile().hash() != mode.profile.hash()
            {
                return Err("Population attack admission changed".into());
            }
            value.clone()
        }
        None => PopulationState::new(mode.profile.clone(), epoch, rate, welcome.world)?,
    };
    match next.apply(bytes)? {
        Applied::Changed => {
            let frame = next
                .current()
                .ok_or("Population attack frame unavailable")?
                .clone();
            mode.events.try_send(frame).map_err(|_| "Population attack presentation queue full or closed; disconnecting instead of dropping critical timing")?;
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
pub(super) fn publish(
    server: &mut super::RenetServer,
    peers: &mut std::collections::BTreeMap<u64, super::Peer>,
    world: &mut crate::simulation::World,
    stats: &mut super::ServerStats,
) -> Result<()> {
    let Some(profile) = world.public_population_attack_profile().cloned() else {
        return Ok(());
    };
    let view = world
        .public_population_attack_snapshot()?
        .ok_or("Missing population attack observation")?;
    let bytes = wire::encode(&profile, world.tick_rate(), &view)?;
    let mut slow = Vec::new();
    for (&id, peer) in peers.iter_mut() {
        if !peer.public_presentation_ready {
            continue;
        }
        if let Some(old) = &peer.public_population_attack_sent {
            // Leave/retirement can occur between fixed ticks. Defer that full
            // update to the next authority tick instead of publishing conflicting
            // contents for one tick on the reliable ordered channel.
            if old.tick == view.tick
                || (old.epoch == view.epoch
                    && old.actors.len() == view.actors.len()
                    && old.actors.iter().zip(&view.actors).all(|(a, b)| {
                        a.profile == b.profile && a.creature == b.creature && a.windup == b.windup
                    }))
            {
                continue;
            }
        }
        if !server.can_send_message(
            id,
            crate::renet_transport::CREATURE_ATTACK_CHANNEL,
            bytes.len(),
        ) {
            slow.push(id);
            continue;
        }
        server.send_message(
            id,
            crate::renet_transport::CREATURE_ATTACK_CHANNEL,
            bytes.clone(),
        );
        stats.public_attack_max_queue_bytes = stats.public_attack_max_queue_bytes.max(
            crate::renet_transport::CHANNEL_MEMORY_BYTES
                - server
                    .channel_available_memory(id, crate::renet_transport::CREATURE_ATTACK_CHANNEL),
        );
        stats.public_attack_max_frame_bytes = stats.public_attack_max_frame_bytes.max(bytes.len());
        stats.public_attack_states_sent = stats.public_attack_states_sent.saturating_add(1);
        peer.public_population_attack_sent = Some(view.clone());
    }
    stats.public_attack_backpressure_disconnects = stats
        .public_attack_backpressure_disconnects
        .saturating_add(slow.len() as u64);
    super::retire_peers(server, peers, world, &slow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature_attack_wire::{Observation, Profile, Windup};
    use theia_core::identity::{EntityId, EntityKey};
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn fixture() -> (Arc<PopulationProfile>, Welcome) {
        let life = crate::player_life_wire::Profile::from_json(
            include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
            include_bytes!("../../../../../content/gameplay/development-creature-pursuit.json"),
            include_bytes!("../../../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        let profile = Arc::new(PopulationProfile::new(Arc::new(Profile::new(Arc::new(
            life,
        )))));
        let welcome = Welcome {
            id: 100,
            epoch: 7,
            world: profile.identity([1; 32]),
            terrain_stream: true,
            origin: [0.; 3],
        };
        (profile, welcome)
    }
    fn bytes(profile: &PopulationProfile, rate: TickRate, tick: u64) -> Vec<u8> {
        let actors = [1, 2]
            .map(|id| Observation {
                profile: profile.source().hash(),
                creature: key(id),
                tick,
                windup: Some(Windup {
                    target: key(100),
                    starts: tick,
                    completes: tick + profile.source().windup_ticks(rate).unwrap(),
                }),
            })
            .to_vec();
        wire::encode(
            profile,
            rate,
            &Frame {
                epoch: key(1).epoch,
                tick,
                actors,
            },
        )
        .unwrap()
    }
    #[test]
    fn valid_changes_reach_queue_once_and_movement_admission_binds_state() {
        let (profile, welcome) = fixture();
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (tx, mut rx) = mpsc::channel(PRESENTATION_CAPACITY);
            let mode = Client {
                profile: profile.clone(),
                events: tx,
            };
            let mut current = None;
            let mut stats = TransportStats::default();
            let first = bytes(&profile, rate, 10);
            receive(
                &mode,
                &welcome,
                [Some(rate), None],
                &first,
                &mut current,
                &mut stats,
            )
            .unwrap();
            assert_eq!(rx.try_recv().unwrap().actors.len(), 2);
            receive(
                &mode,
                &welcome,
                [None, Some(rate)],
                &first,
                &mut current,
                &mut stats,
            )
            .unwrap();
            assert!(rx.try_recv().is_err());
            assert_eq!(stats.public_attack_duplicate_states, 1);
            let next = bytes(&profile, rate, 11);
            receive(
                &mode,
                &welcome,
                [Some(rate); 2],
                &next,
                &mut current,
                &mut stats,
            )
            .unwrap();
            rx.try_recv().unwrap();
            receive(&mode, &welcome, [None; 2], &first, &mut current, &mut stats).unwrap();
            assert_eq!(stats.public_attack_stale_states, 1);
            for bad in [
                Welcome {
                    epoch: 8,
                    ..welcome.clone()
                },
                Welcome {
                    world: [2; 32],
                    ..welcome.clone()
                },
            ] {
                assert!(
                    receive(
                        &mode,
                        &bad,
                        [Some(rate); 2],
                        &next,
                        &mut current,
                        &mut stats
                    )
                    .is_err()
                );
            }
            let other = if rate == TickRate::Hz20 {
                TickRate::Hz30
            } else {
                TickRate::Hz20
            };
            assert!(
                receive(
                    &mode,
                    &welcome,
                    [Some(other), None],
                    &next,
                    &mut current,
                    &mut stats
                )
                .is_err()
            );
            assert_eq!(current.as_ref().unwrap().current().unwrap().tick, 11);
            assert_eq!(stats.public_attack_states_received, 2);
            assert!(rx.try_recv().is_err());
        }
    }
    #[test]
    fn full_or_closed_presentation_queue_refuses_commit_without_losing_current_state() {
        let (profile, welcome) = fixture();
        let rate = TickRate::Hz30;
        let (tx, mut rx) = mpsc::channel(PRESENTATION_CAPACITY);
        let mode = Client {
            profile: profile.clone(),
            events: tx,
        };
        let mut current = None;
        let mut stats = TransportStats::default();
        for tick in 10..10 + PRESENTATION_CAPACITY as u64 {
            receive(
                &mode,
                &welcome,
                [Some(rate); 2],
                &bytes(&profile, rate, tick),
                &mut current,
                &mut stats,
            )
            .unwrap();
        }
        let accepted = current.as_ref().unwrap().current().unwrap().tick;
        let pending = bytes(&profile, rate, accepted + 1);
        assert!(
            receive(
                &mode,
                &welcome,
                [Some(rate); 2],
                &pending,
                &mut current,
                &mut stats
            )
            .is_err()
        );
        assert_eq!(current.as_ref().unwrap().current().unwrap().tick, accepted);
        assert_eq!(
            stats.public_attack_states_received,
            PRESENTATION_CAPACITY as u64
        );
        rx.try_recv().unwrap();
        receive(
            &mode,
            &welcome,
            [Some(rate); 2],
            &pending,
            &mut current,
            &mut stats,
        )
        .unwrap();
        assert_eq!(
            current.as_ref().unwrap().current().unwrap().tick,
            accepted + 1
        );
        drop(rx);
        assert!(
            receive(
                &mode,
                &welcome,
                [Some(rate); 2],
                &bytes(&profile, rate, accepted + 2),
                &mut current,
                &mut stats
            )
            .is_err()
        );
        assert_eq!(
            current.as_ref().unwrap().current().unwrap().tick,
            accepted + 1
        );
        let (tx, _rx) = mpsc::channel(1);
        assert!(
            Client {
                profile,
                events: tx
            }
            .validate()
            .is_err()
        );
    }
    #[test]
    #[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_AUTHORED_NAV and THEIA_AUTHORED_SPAWNS"]
    fn authored_population_publication_waits_for_readiness_and_retires_slow_peer() {
        use crate::{
            authored_world, navigation, procedural,
            simulation::{World, creatures, population::Binding},
        };
        let store = Arc::new(
            authored_world::Store::open(std::path::Path::new(
                &std::env::var_os("THEIA_AUTHORED_WORLD").unwrap(),
            ))
            .unwrap(),
        );
        let nav = navigation::Store::open(
            std::path::Path::new(&std::env::var_os("THEIA_AUTHORED_NAV").unwrap()),
            store.identity(),
        )
        .unwrap();
        let request = serde_json::from_slice(
            &std::fs::read(std::env::var_os("THEIA_AUTHORED_SPAWNS").unwrap()).unwrap(),
        )
        .unwrap();
        let prepared = navigation::spawn_preparation::check_authored(&store, &nav, &request)
            .unwrap()
            .into_prepared()
            .unwrap();
        let saved = store.spawn(0).unwrap().metres();
        let mut creature: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-creature-passive.json"
        ))
        .unwrap();
        creature["aggro"] = serde_json::json!({"perception_metres":1.,"leash_metres":35.,"chase_speed":5.,"return_speed":5.,"los_grace_ms":2000,"engagement_ms":30000,"reacquire_ms":2000});
        let creature = serde_json::to_vec(&creature).unwrap();
        let definition = creatures::Definition::from_json(&creature).unwrap();
        let mut world = World::with_terrain_source(
            [saved[0], saved[2]],
            1.,
            procedural::Source::Authored(store),
        )
        .unwrap();
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        world
            .enable_navigating_population(
                prepared,
                Binding::new(definition.id(), definition.clone(), 30).unwrap(),
                navigation::worker::Worker::load(&nav).unwrap(),
            )
            .unwrap();
        let stats =
            include_bytes!("../../../../../content/gameplay/development-character-stats.json");
        world
            .enable_resource_combat(
                include_bytes!("../../../../../content/gameplay/development-abilities.json"),
                100,
                30000,
                stats,
            )
            .unwrap();
        let life = crate::player_life_wire::Profile::from_json(
            stats,
            &creature,
            include_bytes!("../../../../../content/gameplay/development-creature-retaliation.json"),
            include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
        )
        .unwrap();
        world.enable_population_player_life(Arc::new(life)).unwrap();
        world.enable_public_population_attacks().unwrap();
        world.require_client_chunks().unwrap();
        let mut server = super::super::RenetServer::new(
            crate::renet_transport::connection_config_with_public_attacks(),
        );
        server.add_connection(1);
        let peer = crate::renet_runtime::admit_peer(
            &mut world,
            &mut server,
            1,
            None,
            None,
            tokio::time::Instant::now(),
        )
        .unwrap()
        .unwrap();
        let actor = peer.actor;
        let mut peers = std::collections::BTreeMap::from([(1, peer)]);
        let mut stats = super::super::ServerStats::default();
        publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
        assert_eq!(stats.public_attack_states_sent, 0);
        peers.get_mut(&1).unwrap().public_presentation_ready = true;
        publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
        assert_eq!(stats.public_attack_states_sent, 1);
        world.step();
        publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
        assert_eq!(stats.public_attack_states_sent, 1);
        peers.get_mut(&1).unwrap().public_population_attack_sent = None;
        let available =
            server.channel_available_memory(1, crate::renet_transport::CREATURE_ATTACK_CHANNEL);
        server.send_message(
            1,
            crate::renet_transport::CREATURE_ATTACK_CHANNEL,
            vec![0; available],
        );
        publish(&mut server, &mut peers, &mut world, &mut stats).unwrap();
        assert!(peers.is_empty());
        assert!(world.player_life_snapshot(actor).is_none());
        assert_eq!(
            (
                stats.public_attack_states_sent,
                stats.public_attack_backpressure_disconnects
            ),
            (1, 1)
        );
    }
}
