//! Bounded whole-population public windups. No health, results or life generations.
//! Full frames retire omitted actors; this is not interest-area re-entry support.
use super::{Applied, Observation, Profile, Windup};
use std::sync::Arc;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    time::TickRate,
};

pub const MAX_ACTORS: usize = 16;
pub const TAG: &[u8; 4] = b"NP01";
pub const HEADER_BYTES: usize = 54;
pub const MAX_FRAME_BYTES: usize = HEADER_BYTES + MAX_ACTORS * 33;
const _: () = assert!(MAX_FRAME_BYTES <= crate::wire::MAX_BYTES);
#[derive(Clone, Debug)]
pub struct PopulationProfile {
    source: Arc<Profile>,
    hash: [u8; 32],
}
impl PopulationProfile {
    pub fn new(source: Arc<Profile>) -> Self {
        let mut bytes = b"theia-public-population-attack-profile-v1\0".to_vec();
        bytes.extend(source.hash());
        bytes.push(MAX_ACTORS as u8);
        Self {
            source,
            hash: crate::terrain::sha256(&bytes),
        }
    }
    pub fn source(&self) -> &Profile {
        &self.source
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-public-population-attack-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.hash);
        crate::terrain::sha256(&bytes)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub epoch: WorldEpoch,
    pub tick: u64,
    pub actors: Vec<Observation>,
}
fn valid(profile: &PopulationProfile, rate: TickRate, frame: &Frame) -> bool {
    frame.actors.len() <= MAX_ACTORS
        && frame
            .actors
            .windows(2)
            .all(|p| p[0].creature.entity < p[1].creature.entity)
        && frame.actors.iter().all(|v| {
            v.creature.epoch == frame.epoch
                && v.tick == frame.tick
                && super::valid(profile.source(), rate, v)
        })
}
pub fn is_state(bytes: &[u8]) -> bool {
    bytes.starts_with(TAG)
}
pub fn encode(
    profile: &PopulationProfile,
    rate: TickRate,
    frame: &Frame,
) -> Result<Vec<u8>, &'static str> {
    if !valid(profile, rate, frame) {
        return Err("Invalid population attack frame");
    }
    let mut bytes = Vec::with_capacity(MAX_FRAME_BYTES);
    bytes.extend(TAG);
    bytes.push(rate.hz() as u8);
    bytes.extend(profile.hash());
    bytes.extend(frame.epoch.value().to_le_bytes());
    bytes.extend(frame.tick.to_le_bytes());
    bytes.push(frame.actors.len() as u8);
    for actor in &frame.actors {
        bytes.extend(actor.creature.entity.value().to_le_bytes());
        bytes.push(u8::from(actor.windup.is_some()));
        if let Some(w) = actor.windup {
            bytes.extend(w.target.entity.value().to_le_bytes());
            bytes.extend(w.starts.to_le_bytes());
            bytes.extend(w.completes.to_le_bytes());
        }
    }
    Ok(bytes)
}
pub fn decode(
    profile: &PopulationProfile,
    epoch: WorldEpoch,
    rate: TickRate,
    bytes: &[u8],
) -> Result<Frame, &'static str> {
    if !is_state(bytes)
        || !(HEADER_BYTES..=MAX_FRAME_BYTES).contains(&bytes.len())
        || bytes[4] != rate.hz() as u8
        || bytes[5..37] != profile.hash()
    {
        return Err("Invalid population attack header");
    }
    let word = |offset: usize| -> Result<u64, &'static str> {
        let data = bytes
            .get(offset..offset + 8)
            .ok_or("Truncated population attack word")?;
        Ok(u64::from_le_bytes(data.try_into().unwrap()))
    };
    if word(37)? != epoch.value() {
        return Err("Foreign population attack epoch");
    }
    let count = usize::from(bytes[53]);
    if count > MAX_ACTORS {
        return Err("Population attack actor limit exceeded");
    }
    let tick = word(45)?;
    let mut actors = Vec::with_capacity(count);
    let mut offset = HEADER_BYTES;
    for _ in 0..count {
        let creature = EntityKey {
            epoch,
            entity: EntityId::new(word(offset)?).ok_or("Invalid population creature ID")?,
        };
        let pending = *bytes
            .get(offset + 8)
            .ok_or("Truncated population attack flag")?;
        offset += 9;
        let windup = match pending {
            0 => None,
            1 => {
                let target = EntityKey {
                    epoch,
                    entity: EntityId::new(word(offset)?).ok_or("Invalid population target ID")?,
                };
                let starts = word(offset + 8)?;
                let completes = word(offset + 16)?;
                offset += 24;
                Some(Windup {
                    target,
                    starts,
                    completes,
                })
            }
            _ => return Err("Invalid population attack flag"),
        };
        actors.push(Observation {
            profile: profile.source().hash(),
            creature,
            tick,
            windup,
        });
    }
    let frame = Frame {
        epoch,
        tick,
        actors,
    };
    if offset != bytes.len() || !valid(profile, rate, &frame) {
        return Err("Invalid population attack body");
    }
    Ok(frame)
}
#[derive(Clone, Debug)]
pub struct PopulationState {
    profile: Arc<PopulationProfile>,
    epoch: WorldEpoch,
    rate: TickRate,
    world: [u8; 32],
    current: Option<Frame>,
    high_water_entity: u64,
}
impl PopulationState {
    pub fn new(
        profile: Arc<PopulationProfile>,
        epoch: WorldEpoch,
        rate: TickRate,
        world: [u8; 32],
    ) -> Result<Self, &'static str> {
        if world == [0; 32] {
            return Err("Invalid population attack world identity");
        }
        Ok(Self {
            profile,
            epoch,
            rate,
            world,
            current: None,
            high_water_entity: 0,
        })
    }
    pub fn current(&self) -> Option<&Frame> {
        self.current.as_ref()
    }
    pub fn world(&self) -> [u8; 32] {
        self.world
    }
    pub fn active_at(&self, creature: EntityKey, tick: u64) -> Option<&Windup> {
        self.current
            .as_ref()?
            .actors
            .iter()
            .find(|v| v.creature == creature)?
            .windup
            .as_ref()
            .filter(|w| w.starts <= tick && tick < w.completes)
    }
    pub fn apply(&mut self, bytes: &[u8]) -> Result<Applied, &'static str> {
        let frame = decode(&self.profile, self.epoch, self.rate, bytes)?;
        if let Some(old) = &self.current {
            if frame.tick < old.tick {
                return Ok(Applied::Stale);
            }
            if frame.tick == old.tick {
                return if frame == *old {
                    Ok(Applied::Duplicate)
                } else {
                    Err("Conflicting population attacks at one tick")
                };
            }
            if frame.actors.iter().any(|v| {
                v.creature.entity.value() <= self.high_water_entity
                    && !old.actors.iter().any(|o| o.creature == v.creature)
            }) {
                return Err("Retired population creature ID reused");
            }
        }
        self.high_water_entity = frame
            .actors
            .iter()
            .map(|v| v.creature.entity.value())
            .max()
            .unwrap_or(0)
            .max(self.high_water_entity);
        self.current = Some(frame);
        Ok(Applied::Changed)
    }
}

#[cfg(test)]
mod tests;
