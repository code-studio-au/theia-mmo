//! Population timing validation and bounded presentation handoff.
//! Session/channel adoption is separate; queue pressure is an explicit failure.
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
}
