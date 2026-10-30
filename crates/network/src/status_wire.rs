//! Status observation profile 1, separate from qualified CS01/CS02 states.
//! No client status command, entity mutation or client-side ticking authority.
use crate::{
    Result,
    simulation::combat::{MAX_ACTIVE_STATUSES, MAX_TARGET_STATUSES, Status},
};
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    statuses::Catalog,
    time::TickRate,
};
pub const STATE_TAG: &[u8; 4] = b"SS01";
pub const HEADER_BYTES: usize = 22;
pub const STATUS_BYTES: usize = 33;
pub const MAX_BYTES: usize = HEADER_BYTES + MAX_ACTIVE_STATUSES * STATUS_BYTES;
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn ordinal(catalog: &Catalog, status: &theia_core::content::ContentId) -> Result<u8> {
    catalog
        .definitions()
        .position(|d| d.id() == status)
        .map(|i| i as u8)
        .ok_or_else(|| "Unknown status definition".into())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub epoch: u64,
    pub tick: u64,
    pub rate: TickRate,
    pub statuses: Vec<Status>,
}
impl State {
    pub fn same_view(&self, other: &Self) -> bool {
        self.epoch == other.epoch && self.rate == other.rate && self.statuses == other.statuses
    }
    fn valid(&self, catalog: &Catalog) -> bool {
        if WorldEpoch::new(self.epoch).is_none() || self.statuses.len() > MAX_ACTIVE_STATUSES {
            return false;
        }
        let mut previous = None;
        let mut count = 0;
        let mut target = None;
        for status in &self.statuses {
            let Some(definition) = catalog.definition(&status.status) else {
                return false;
            };
            let timing = definition.timing(self.rate);
            if status.source.epoch.value() != self.epoch
                || status.target.epoch.value() != self.epoch
                || status.source == status.target
                || status.expires_at <= self.tick
                || status.expires_at - self.tick > timing.duration_ticks
                || (status.next_tick.is_none()
                    && status.expires_at - self.tick >= timing.interval_ticks)
                || status.next_tick.is_some_and(|at| {
                    at <= self.tick
                        || at > status.expires_at
                        || at - self.tick > timing.interval_ticks
                })
            {
                return false;
            }
            let key = (
                status.target.entity.value(),
                status.status.clone(),
                status.source.entity.value(),
            );
            if previous.as_ref().is_some_and(|old| *old >= key) {
                return false;
            }
            previous = Some(key);
            if target == Some(status.target) {
                count += 1;
            } else {
                target = Some(status.target);
                count = 1;
            }
            if count > MAX_TARGET_STATUSES {
                return false;
            }
        }
        true
    }
    pub fn encode(&self, catalog: &Catalog) -> Result<Vec<u8>> {
        if !self.valid(catalog) {
            return Err("Invalid status observation membership/order/deadline".into());
        }
        let mut bytes = STATE_TAG.to_vec();
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.push(self.rate.hz() as u8);
        bytes.push(self.statuses.len() as u8);
        for status in &self.statuses {
            bytes.extend(status.source.entity.value().to_le_bytes());
            bytes.extend(status.target.entity.value().to_le_bytes());
            bytes.push(ordinal(catalog, &status.status)?);
            bytes.extend(status.expires_at.to_le_bytes());
            bytes.extend(status.next_tick.unwrap_or(0).to_le_bytes());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog) -> Result<Self> {
        if bytes.len() < HEADER_BYTES || bytes.len() > MAX_BYTES || !bytes.starts_with(STATE_TAG) {
            return Err("Invalid status observation frame".into());
        }
        let count = usize::from(bytes[21]);
        if count > MAX_ACTIVE_STATUSES || bytes.len() != HEADER_BYTES + count * STATUS_BYTES {
            return Err("Invalid status observation count/length".into());
        }
        let epoch = word(bytes, 4);
        let incarnation = WorldEpoch::new(epoch).ok_or("Invalid status epoch")?;
        let rate = TickRate::from_hz(u32::from(bytes[20])).ok_or("Invalid status tick rate")?;
        let definitions: Vec<_> = catalog.definitions().collect();
        let mut statuses = Vec::with_capacity(count);
        for row in bytes[HEADER_BYTES..].as_chunks::<STATUS_BYTES>().0 {
            let source = EntityKey {
                epoch: incarnation,
                entity: EntityId::new(word(row, 0)).ok_or("Invalid status source")?,
            };
            let target = EntityKey {
                epoch: incarnation,
                entity: EntityId::new(word(row, 8)).ok_or("Invalid status target")?,
            };
            let status = definitions
                .get(usize::from(row[16]))
                .ok_or("Unknown status ordinal")?
                .id()
                .clone();
            let expires_at = word(row, 17);
            let next_tick = match word(row, 25) {
                0 => None,
                at => Some(at),
            };
            statuses.push(Status {
                source,
                target,
                status,
                expires_at,
                next_tick,
            });
        }
        let state = Self {
            epoch,
            tick: word(bytes, 12),
            rate,
            statuses,
        };
        if !state.valid(catalog) {
            return Err("Invalid status observation membership/order/deadline".into());
        }
        Ok(state)
    }
}
