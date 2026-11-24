//! Public visual lifecycle only: actor/life identity and alive/dead, no vitals,
//! outcomes, movement commands or respawn entitlement. Transport adoption is opt-in.
use crate::{player_life_wire, terrain::sha256};
use std::{num::NonZeroU64, sync::Arc};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
pub const TAG: &[u8; 4] = b"AV01";
pub const HEADER_BYTES: usize = 54;
pub const ACTOR_BYTES: usize = 17;
pub const MAX_ACTORS: usize = crate::wire::MAX_PLAYERS;
pub const MAX_FRAME_BYTES: usize = HEADER_BYTES + ACTOR_BYTES * MAX_ACTORS;
#[derive(Clone, Debug)]
pub struct Profile {
    life: Arc<player_life_wire::Profile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn new(life: Arc<player_life_wire::Profile>) -> Self {
        let mut bytes = b"theia-public-avatar-life-profile-v1\0".to_vec();
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
        let mut bytes = b"theia-public-avatar-life-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.hash);
        sha256(&bytes)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Actor {
    pub key: LifeKey,
    pub dead: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub profile: [u8; 32],
    pub epoch: WorldEpoch,
    pub tick: u64,
    pub actors: Vec<Actor>,
}
pub fn is_state(bytes: &[u8]) -> bool {
    bytes.starts_with(TAG)
}
pub(crate) fn valid(profile: &Profile, value: &Observation) -> bool {
    value.profile == profile.hash()
        && value.actors.len() <= MAX_ACTORS
        && value
            .actors
            .iter()
            .all(|a| a.key.actor.epoch == value.epoch)
        && value
            .actors
            .windows(2)
            .all(|p| p[0].key.actor.entity < p[1].key.actor.entity)
}
pub fn frame_rate(bytes: &[u8]) -> Result<TickRate, &'static str> {
    if !is_state(bytes) || !(HEADER_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len()) {
        return Err("Invalid public avatar life header");
    }
    TickRate::from_hz(u32::from(bytes[4])).ok_or("Invalid public avatar life rate")
}
pub fn encode(
    profile: &Profile,
    rate: TickRate,
    value: &Observation,
) -> Result<Vec<u8>, &'static str> {
    if !valid(profile, value) {
        return Err("Invalid public avatar life observation");
    }
    let mut bytes = Vec::with_capacity(HEADER_BYTES + ACTOR_BYTES * value.actors.len());
    bytes.extend(TAG);
    bytes.push(rate.hz() as u8);
    bytes.extend(value.profile);
    bytes.extend(value.epoch.value().to_le_bytes());
    bytes.extend(value.tick.to_le_bytes());
    bytes.push(value.actors.len() as u8);
    for actor in &value.actors {
        bytes.extend(actor.key.actor.entity.value().to_le_bytes());
        bytes.extend(actor.key.generation.get().to_le_bytes());
        bytes.push(u8::from(actor.dead));
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
        return Err("Public avatar life rate disagrees with movement");
    }
    let count = usize::from(bytes[53]);
    if count > MAX_ACTORS || bytes.len() != HEADER_BYTES + ACTOR_BYTES * count {
        return Err("Invalid public avatar life size");
    }
    let word = |offset| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
    if word(37) != epoch.value() {
        return Err("Foreign public avatar life epoch");
    }
    let mut actors = Vec::with_capacity(count);
    for index in 0..count {
        let offset = HEADER_BYTES + ACTOR_BYTES * index;
        actors.push(Actor {
            key: LifeKey {
                actor: EntityKey {
                    epoch,
                    entity: EntityId::new(word(offset)).ok_or("Invalid public avatar actor")?,
                },
                generation: NonZeroU64::new(word(offset + 8))
                    .ok_or("Invalid public avatar life generation")?,
            },
            dead: match bytes[offset + 16] {
                0 => false,
                1 => true,
                _ => return Err("Invalid public avatar death flag"),
            },
        });
    }
    let observation = Observation {
        profile: bytes[5..37].try_into().unwrap(),
        epoch,
        tick: word(45),
        actors,
    };
    if !valid(profile, &observation) {
        return Err("Invalid public avatar life observation");
    }
    Ok(observation)
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
    owner: EntityKey,
    rate: TickRate,
    world: [u8; 32],
    current: Option<Observation>,
}
impl ClientState {
    pub fn new(
        profile: Arc<Profile>,
        owner: EntityKey,
        rate: TickRate,
        world: [u8; 32],
    ) -> Result<Self, &'static str> {
        if world == [0; 32] {
            return Err("Invalid public avatar world identity");
        }
        Ok(Self {
            profile,
            owner,
            rate,
            world,
            current: None,
        })
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn rate(&self) -> TickRate {
        self.rate
    }
    pub fn world(&self) -> [u8; 32] {
        self.world
    }
    pub fn current(&self) -> Option<&Observation> {
        self.current.as_ref()
    }
    pub fn actor(&self, actor: EntityKey) -> Option<&Actor> {
        if actor.epoch != self.owner.epoch {
            return None;
        }
        let actors = &self.current.as_ref()?.actors;
        actors
            .binary_search_by_key(&actor.entity, |a| a.key.actor.entity)
            .ok()
            .map(|index| &actors[index])
    }
    pub fn apply(&mut self, bytes: &[u8]) -> Result<Applied, &'static str> {
        let value = decode(&self.profile, self.owner.epoch, self.rate, bytes)?;
        if !value.actors.iter().any(|a| a.key.actor == self.owner) {
            return Err("Public avatar roster omits owner");
        }
        if let Some(old) = &self.current {
            if value.tick < old.tick {
                return Ok(Applied::Stale);
            }
            if value.tick == old.tick {
                return if value == *old {
                    Ok(Applied::Duplicate)
                } else {
                    Err("Conflicting public avatar observations at one tick")
                };
            }
            for next in &value.actors {
                if let Some(previous) = self.actor(next.key.actor)
                    && (next.key.generation < previous.key.generation
                        || (next.key.generation == previous.key.generation
                            && previous.dead
                            && !next.dead))
                {
                    return Err(
                        "Public avatar life regressed or revived without a fresh generation",
                    );
                }
            }
        }
        self.current = Some(value);
        Ok(Applied::Changed)
    }
}
