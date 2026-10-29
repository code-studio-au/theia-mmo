//! Opt-in targeting profile 1. Bounded reliable messages; no client coordinates.
use crate::{
    Result,
    targeting::{MAX_TARGETS, Snapshot, Target},
};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
};
pub const SELECTION_TAG: &[u8; 4] = b"TS01";
pub const STATE_TAG: &[u8; 4] = b"TG01";
pub const SELECTION_BYTES: usize = 28;
const HEADER_BYTES: usize = 38;
const TARGET_BYTES: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub epoch: u64,
    pub sequence: u64,
    pub target: Option<EntityId>,
}
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
impl Selection {
    pub fn encode(self) -> Result<Vec<u8>> {
        if WorldEpoch::new(self.epoch).is_none() || self.sequence == 0 {
            return Err("Invalid target selection identity/sequence".into());
        }
        let mut bytes = SELECTION_TAG.to_vec();
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.target.map_or(0, EntityId::value).to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SELECTION_BYTES || !bytes.starts_with(SELECTION_TAG) {
            return Err("Invalid target selection frame".into());
        }
        let value = Self {
            epoch: word(bytes, 4),
            sequence: word(bytes, 12),
            target: EntityId::new(word(bytes, 20)),
        };
        if WorldEpoch::new(value.epoch).is_none() || value.sequence == 0 {
            return Err("Invalid target selection identity/sequence".into());
        }
        Ok(value)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub epoch: u64,
    pub tick: u64,
    /// Processed request, including a rejected request. Not a successful cast ack.
    pub request_sequence: u64,
    pub accepted: bool,
    pub selected: Option<EntityId>,
    pub targets: Vec<Target>,
}
impl State {
    pub fn new(
        epoch: u64,
        tick: u64,
        request_sequence: u64,
        accepted: bool,
        snapshot: Snapshot,
    ) -> Self {
        Self {
            epoch,
            tick,
            request_sequence,
            accepted,
            selected: snapshot.selected.map(|k| k.entity),
            targets: snapshot.targets,
        }
    }
    /// Ignore timestamp-only changes; static data need not repeat every tick.
    pub fn same_view(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.request_sequence == other.request_sequence
            && self.accepted == other.accepted
            && self.selected == other.selected
            && self.targets == other.targets
    }
    fn valid(&self) -> bool {
        WorldEpoch::new(self.epoch).is_some()
            && !(self.request_sequence == 0 && self.accepted)
            && self.targets.len() <= MAX_TARGETS
            && self
                .targets
                .iter()
                .all(|t| t.key.epoch.value() == self.epoch)
            && self
                .targets
                .windows(2)
                .all(|p| p[0].key.entity < p[1].key.entity)
            && self
                .selected
                .is_none_or(|id| self.targets.iter().any(|t| t.key.entity == id))
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        if !self.valid() {
            return Err("Invalid targeting state".into());
        }
        let mut bytes = Vec::with_capacity(HEADER_BYTES + self.targets.len() * TARGET_BYTES);
        bytes.extend(STATE_TAG);
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.request_sequence.to_le_bytes());
        bytes.push(u8::from(self.accepted));
        bytes.extend(self.selected.map_or(0, EntityId::value).to_le_bytes());
        bytes.push(self.targets.len() as u8);
        for target in &self.targets {
            bytes.extend(target.key.entity.value().to_le_bytes());
            for coordinate in target.position.metres() {
                bytes.extend(coordinate.to_le_bytes());
            }
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_BYTES || !bytes.starts_with(STATE_TAG) || bytes[28] > 1 {
            return Err("Invalid targeting state frame".into());
        }
        let count = usize::from(bytes[37]);
        if count > MAX_TARGETS || bytes.len() != HEADER_BYTES + count * TARGET_BYTES {
            return Err("Targeting state exceeds count/byte budget".into());
        }
        let epoch = word(bytes, 4);
        let incarnation = WorldEpoch::new(epoch).ok_or("Invalid target world incarnation")?;
        let mut targets = Vec::with_capacity(count);
        for item in bytes[HEADER_BYTES..].as_chunks::<TARGET_BYTES>().0 {
            let entity = EntityId::new(word(item, 0)).ok_or("Invalid target entity ID")?;
            let position = GlobalPosition::new([
                f64::from_bits(word(item, 8)),
                f64::from_bits(word(item, 16)),
                f64::from_bits(word(item, 24)),
            ])
            .ok_or("Invalid global target position")?;
            targets.push(Target {
                key: EntityKey {
                    epoch: incarnation,
                    entity,
                },
                position,
            });
        }
        let value = Self {
            epoch,
            tick: word(bytes, 12),
            request_sequence: word(bytes, 20),
            accepted: bytes[28] != 0,
            selected: EntityId::new(word(bytes, 29)),
            targets,
        };
        if !value.valid() {
            return Err("Invalid targeting state membership/order".into());
        }
        Ok(value)
    }
}
