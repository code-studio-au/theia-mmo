//! Checked status definitions and bounded simulation-tick schedules. No damage,
//! entity ownership, replication or persistence is performed by these contracts.
use crate::{content::ContentId, time::TickRate};
use serde::Deserialize;
use std::{collections::BTreeMap, time::Duration};
pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const MAX_DEFINITIONS: usize = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    PeriodicDamage { amount: u32 },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    statuses: Vec<RawDefinition>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefinition {
    id: String,
    name: String,
    duration_ms: u32,
    interval_ms: u32,
    effect: Effect,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    id: ContentId,
    name: String,
    duration_ms: u32,
    interval_ms: u32,
    effect: Effect,
}
impl Definition {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn effect(&self) -> Effect {
        self.effect
    }
    pub fn timing(&self, rate: TickRate) -> Timing {
        let ticks = |ms| {
            rate.ticks_for(Duration::from_millis(u64::from(ms)))
                .expect("Checked status timing fits qualified rates")
        };
        Timing {
            duration_ticks: ticks(self.duration_ms),
            interval_ticks: ticks(self.interval_ms),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    statuses: BTreeMap<ContentId, Definition>,
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Status catalog exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid status catalog")?;
        if raw.schema != 1 || !(1..=MAX_DEFINITIONS).contains(&raw.statuses.len()) {
            return Err("Unsupported status schema or count");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid status catalog ID")?;
        let mut statuses = BTreeMap::new();
        for raw in raw.statuses {
            let id = ContentId::new(&raw.id).ok_or("Invalid status ID")?;
            if !(50..=60_000).contains(&raw.interval_ms)
                || raw.duration_ms < raw.interval_ms
                || raw.duration_ms > 60_000
            {
                return Err("Status duration/interval exceeds development bounds");
            }
            if !(1..=48).contains(&raw.name.len())
                || raw.name.trim() != raw.name
                || !raw
                    .name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b" '-".contains(&c))
                || !raw.name.bytes().any(|c| c.is_ascii_alphanumeric())
            {
                return Err("Invalid development status name");
            }
            match raw.effect {
                Effect::PeriodicDamage { amount } if amount == 0 || amount > 1_000_000 => {
                    return Err("Status damage exceeds development bounds");
                }
                _ => {}
            }
            let definition = Definition {
                id: id.clone(),
                name: raw.name,
                duration_ms: raw.duration_ms,
                interval_ms: raw.interval_ms,
                effect: raw.effect,
            };
            if statuses.insert(id, definition).is_some() {
                return Err("Duplicate status ID");
            }
        }
        Ok(Self { id, statuses })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn definition(&self, id: &ContentId) -> Option<&Definition> {
        self.statuses.get(id)
    }
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &Definition> {
        self.statuses.values()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub duration_ticks: u64,
    pub interval_ticks: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidTiming,
    TimeExhausted,
    TimeRegressed,
    MissedTick,
    Finished,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Step {
    pub damage_due: bool,
    pub expired: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schedule {
    timing: Timing,
    next: Option<u64>,
    expires: u64,
    last: u64,
    refreshed: u64,
    ended: bool,
}
impl Schedule {
    pub fn start(now: u64, timing: Timing) -> Result<Self, Error> {
        if timing.interval_ticks == 0 || timing.duration_ticks < timing.interval_ticks {
            return Err(Error::InvalidTiming);
        }
        let expires = now
            .checked_add(timing.duration_ticks)
            .ok_or(Error::TimeExhausted)?;
        let next = now
            .checked_add(timing.interval_ticks)
            .ok_or(Error::TimeExhausted)?;
        Ok(Self {
            timing,
            next: Some(next),
            expires,
            last: now,
            refreshed: now,
            ended: false,
        })
    }
    pub fn expires_at(&self) -> u64 {
        self.expires
    }
    pub fn next_tick(&self) -> Option<u64> {
        self.next.filter(|n| !self.ended && *n <= self.expires)
    }
    pub fn finished(&self) -> bool {
        self.ended
    }
    /// Preserve cadence. Refresh cannot manufacture an immediate tick, shorten
    /// lifetime, change the definition's timing or conceal an unprocessed tick.
    pub fn refresh(&mut self, now: u64) -> Result<(), Error> {
        if self.ended || now > self.expires {
            return Err(Error::Finished);
        }
        if now < self.last || now < self.refreshed {
            return Err(Error::TimeRegressed);
        }
        if self.next.is_some_and(|n| n < now && n <= self.expires) {
            return Err(Error::MissedTick);
        }
        let expires = now
            .checked_add(self.timing.duration_ticks)
            .ok_or(Error::TimeExhausted)?;
        self.expires = self.expires.max(expires);
        self.refreshed = now;
        Ok(())
    }
    /// At most one due tick. The caller must visit every due simulation tick;
    /// skipped work fails instead of silently dropping damage or catching up in
    /// an unbounded loop. Repeated visits to one active tick are harmless.
    pub fn advance(&mut self, now: u64) -> Result<Step, Error> {
        if self.ended {
            return Err(Error::Finished);
        }
        if now < self.last || now < self.refreshed {
            return Err(Error::TimeRegressed);
        }
        if now == self.last {
            return Ok(Step::default());
        }
        if self.next.is_some_and(|n| n < now && n <= self.expires) {
            return Err(Error::MissedTick);
        }
        let damage_due = self.next.is_some_and(|n| n == now && n <= self.expires);
        if damage_due {
            self.next = now.checked_add(self.timing.interval_ticks);
        }
        let expired = now >= self.expires;
        self.last = now;
        self.ended = expired;
        Ok(Step {
            damage_due,
            expired,
        })
    }
}
