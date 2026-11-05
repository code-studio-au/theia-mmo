//! Public NPC windup timing only. No health, damage, result or player-life key.
use crate::{player_life_wire, terrain::sha256};
use std::{sync::Arc, time::Duration};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};
const PROFILE_DOMAIN: &[u8] = b"theia-public-creature-attack-profile-v1\0";
const WORLD_DOMAIN: &[u8] = b"theia-public-creature-attack-world-v1\0";
pub const TAG: &[u8; 4] = b"NA01";
pub const MIN_FRAME_BYTES: usize = 62;
pub const MAX_FRAME_BYTES: usize = 86;
#[derive(Clone, Debug)]
pub struct Profile {
    life: Arc<player_life_wire::Profile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn new(life: Arc<player_life_wire::Profile>) -> Self {
        let mut bytes = PROFILE_DOMAIN.to_vec();
        bytes.extend(life.hash());
        Self {
            life,
            hash: sha256(&bytes),
        }
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn life(&self) -> &player_life_wire::Profile {
        &self.life
    }
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = WORLD_DOMAIN.to_vec();
        bytes.extend(base);
        bytes.extend(self.hash);
        sha256(&bytes)
    }
    pub fn windup_ticks(&self, rate: TickRate) -> Option<u64> {
        rate.ticks_for(Duration::from_millis(u64::from(
            self.life.retaliation().windup_ms(),
        )))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windup {
    pub target: EntityKey,
    pub starts: u64,
    pub completes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observation {
    pub profile: [u8; 32],
    pub creature: EntityKey,
    pub tick: u64,
    pub windup: Option<Windup>,
}
pub fn is_state(bytes: &[u8]) -> bool {
    bytes.starts_with(TAG)
}
pub(crate) fn valid(profile: &Profile, rate: TickRate, value: &Observation) -> bool {
    if value.profile != profile.hash() {
        return false;
    }
    value.windup.is_none_or(|windup| {
        windup.target.epoch == value.creature.epoch
            && windup.target != value.creature
            && windup.starts <= value.tick
            && value.tick < windup.completes
            && windup.completes.checked_sub(windup.starts) == profile.windup_ticks(rate)
    })
}
pub fn frame_rate(bytes: &[u8]) -> Result<TickRate, &'static str> {
    if !is_state(bytes) || !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len()) {
        return Err("Invalid public creature attack header");
    }
    TickRate::from_hz(u32::from(bytes[4])).ok_or("Invalid public creature attack rate")
}
pub fn encode(
    profile: &Profile,
    rate: TickRate,
    value: &Observation,
) -> Result<Vec<u8>, &'static str> {
    if !valid(profile, rate, value) {
        return Err("Invalid public creature attack observation");
    }
    let mut bytes = Vec::with_capacity(MAX_FRAME_BYTES);
    bytes.extend(TAG);
    bytes.push(rate.hz() as u8);
    bytes.extend(value.profile);
    bytes.extend(value.creature.epoch.value().to_le_bytes());
    bytes.extend(value.creature.entity.value().to_le_bytes());
    bytes.extend(value.tick.to_le_bytes());
    bytes.push(u8::from(value.windup.is_some()));
    if let Some(windup) = value.windup {
        bytes.extend(windup.target.entity.value().to_le_bytes());
        bytes.extend(windup.starts.to_le_bytes());
        bytes.extend(windup.completes.to_le_bytes());
    }
    Ok(bytes)
}
pub fn decode(
    profile: &Profile,
    epoch: WorldEpoch,
    rate: TickRate,
    bytes: &[u8],
) -> Result<Observation, &'static str> {
    if frame_rate(bytes)? != rate {
        return Err("Public creature attack rate disagrees with movement");
    }
    let pending = match bytes[61] {
        0 => false,
        1 => true,
        _ => return Err("Invalid public creature attack flag"),
    };
    if bytes.len()
        != if pending {
            MAX_FRAME_BYTES
        } else {
            MIN_FRAME_BYTES
        }
    {
        return Err("Invalid public creature attack size");
    }
    let word = |offset| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
    let source_epoch = WorldEpoch::new(word(37))
        .filter(|e| *e == epoch)
        .ok_or("Foreign public creature attack epoch")?;
    let creature = EntityKey {
        epoch: source_epoch,
        entity: EntityId::new(word(45)).ok_or("Invalid public creature source")?,
    };
    let value = Observation {
        profile: bytes[5..37].try_into().unwrap(),
        creature,
        tick: word(53),
        windup: if pending {
            Some(Windup {
                target: EntityKey {
                    epoch: source_epoch,
                    entity: EntityId::new(word(62)).ok_or("Invalid public creature target")?,
                },
                starts: word(70),
                completes: word(78),
            })
        } else {
            None
        },
    };
    if !valid(profile, rate, &value) {
        return Err("Invalid public creature attack observation");
    }
    Ok(value)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    Changed,
    Duplicate,
    Stale,
}
#[derive(Clone, Debug)]
pub struct ClientState {
    profile: Arc<Profile>,
    epoch: WorldEpoch,
    rate: TickRate,
    current: Option<Observation>,
    world: [u8; 32],
}
impl ClientState {
    pub fn new(
        profile: Arc<Profile>,
        epoch: WorldEpoch,
        rate: TickRate,
        world: [u8; 32],
    ) -> Result<Self, &'static str> {
        if world == [0; 32] {
            return Err("Invalid public creature attack world identity");
        }
        Ok(Self {
            profile,
            epoch,
            rate,
            current: None,
            world,
        })
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn rate(&self) -> TickRate {
        self.rate
    }
    pub fn current(&self) -> Option<&Observation> {
        self.current.as_ref()
    }
    pub fn world(&self) -> [u8; 32] {
        self.world
    }

    /// Presentation may suppress expired late-arriving windups using its current
    /// authority tick. It never predicts a hit or alters a player's target/facing.
    pub fn active_at(&self, tick: u64) -> Option<&Windup> {
        self.current
            .as_ref()?
            .windup
            .as_ref()
            .filter(|w| w.starts <= tick && tick < w.completes)
    }
    pub fn apply(&mut self, bytes: &[u8]) -> Result<Applied, &'static str> {
        let value = decode(&self.profile, self.epoch, self.rate, bytes)?;
        if let Some(old) = self.current {
            if value.tick < old.tick {
                return Ok(Applied::Stale);
            }
            if value.tick == old.tick {
                return if value == old {
                    Ok(Applied::Duplicate)
                } else {
                    Err("Conflicting public creature observations at one tick")
                };
            }
        }
        self.current = Some(value);
        Ok(Applied::Changed)
    }
}
