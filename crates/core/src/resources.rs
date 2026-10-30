//! Checked static development vitals and integer resource pools. No renderer,
//! transport, clock, persistence or gameplay authority is introduced here.
use crate::{content::ContentId, time::TickRate};
use serde::Deserialize;
pub const MAX_PROFILE_BYTES: usize = 4096;
pub const MAX_VALUE: u32 = 1_000_000;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    maximum_health: u32,
    maximum_mana: u32,
    mana_regeneration_per_second: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    health: u32,
    mana: u32,
    mana_regeneration: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Stats profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid stats profile")?;
        if raw.schema != 1 {
            return Err("Unsupported stats schema");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid stable stats profile ID")?;
        if !(1..=MAX_VALUE).contains(&raw.maximum_health)
            || !(1..=MAX_VALUE).contains(&raw.maximum_mana)
            || raw.mana_regeneration_per_second > MAX_VALUE
        {
            return Err("Stats profile values exceed development bounds");
        }
        Ok(Self {
            id,
            health: raw.maximum_health,
            mana: raw.maximum_mana,
            mana_regeneration: raw.mana_regeneration_per_second,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn maximum_health(&self) -> u32 {
        self.health
    }
    pub fn maximum_mana(&self) -> u32 {
        self.mana
    }
    pub fn mana_regeneration_per_second(&self) -> u32 {
        self.mana_regeneration
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pool {
    current: u32,
    maximum: u32,
    regeneration_remainder: u32,
}
impl Pool {
    pub fn new(current: u32, maximum: u32) -> Option<Self> {
        ((1..=MAX_VALUE).contains(&maximum) && current <= maximum).then_some(Self {
            current,
            maximum,
            regeneration_remainder: 0,
        })
    }
    pub fn current(self) -> u32 {
        self.current
    }
    pub fn maximum(self) -> u32 {
        self.maximum
    }
    /// Caller decides when an authoritative operation commits its cost. A failed
    /// spend changes nothing; duplicate/cancelled cast handling lives above this.
    pub fn spend(&mut self, amount: u32) -> bool {
        if amount > self.current {
            return false;
        }
        self.current -= amount;
        true
    }
    /// Exactly one simulation tick. Remainder is fractional resource, never wall
    /// time. A full pool discards surplus so later spending cannot unlock a bank.
    pub fn regenerate(&mut self, rate: TickRate, per_second: u32) {
        if self.current == self.maximum {
            self.regeneration_remainder = 0;
            return;
        }
        let numerator = u64::from(self.regeneration_remainder) + u64::from(per_second);
        let whole = numerator / u64::from(rate.hz());
        let next = u64::from(self.current) + whole;
        if next >= u64::from(self.maximum) {
            self.current = self.maximum;
            self.regeneration_remainder = 0;
        } else {
            self.current = next as u32;
            self.regeneration_remainder = (numerator % u64::from(rate.hz())) as u32;
        }
    }
}

/// Validated network/presentation observation; not a client gameplay command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vitals {
    health: Pool,
    mana: Pool,
}
impl Vitals {
    pub fn new(health: u32, maximum_health: u32, mana: u32, maximum_mana: u32) -> Option<Self> {
        Some(Self {
            health: Pool::new(health, maximum_health)?,
            mana: Pool::new(mana, maximum_mana)?,
        })
    }
    pub fn health(self) -> u32 {
        self.health.current()
    }
    pub fn maximum_health(self) -> u32 {
        self.health.maximum()
    }
    pub fn mana(self) -> u32 {
        self.mana.current()
    }
    pub fn maximum_mana(self) -> u32 {
        self.mana.maximum()
    }
}
