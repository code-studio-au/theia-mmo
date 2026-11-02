//! Pure bounded melee intents. The server supplies life/reach/LOS observations;
//! this module grants no client authority and applies no health changes.
use crate::{content::ContentId, player_life::LifeKey, time::TickRate};
use serde::Deserialize;
use std::time::Duration;
pub const MAX_PROFILE_BYTES: usize = 4096;
pub const PROFILE_DOMAIN: &[u8] = b"theia-creature-retaliation-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    creature: String,
    damage: u32,
    reach_metres: f64,
    windup_ms: u32,
    recovery_ms: u32,
    memory_ms: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    id: ContentId,
    creature: ContentId,
    damage: u32,
    reach: f64,
    windup_ms: u32,
    recovery_ms: u32,
    memory_ms: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Retaliation profile exceeds byte budget");
        }
        let r: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid retaliation profile")?;
        if r.schema != 1 {
            return Err("Unsupported retaliation schema");
        }
        let id = ContentId::new(&r.id).ok_or("Invalid retaliation profile ID")?;
        let creature = ContentId::new(&r.creature).ok_or("Invalid retaliation creature ID")?;
        if !(1..=1000).contains(&r.damage)
            || !r.reach_metres.is_finite()
            || !(0.25..=5.).contains(&r.reach_metres)
            || !(100..=5000).contains(&r.windup_ms)
            || !(100..=10000).contains(&r.recovery_ms)
            || !(1000..=60000).contains(&r.memory_ms)
            || r.memory_ms < r.windup_ms + r.recovery_ms
        {
            return Err("Retaliation profile values exceed bounds");
        }
        Ok(Self {
            id,
            creature,
            damage: r.damage,
            reach: r.reach_metres,
            windup_ms: r.windup_ms,
            recovery_ms: r.recovery_ms,
            memory_ms: r.memory_ms,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn damage(&self) -> u32 {
        self.damage
    }
    pub fn windup_ms(&self) -> u32 {
        self.windup_ms
    }
    pub fn reach_metres(&self) -> f64 {
        self.reach
    }
    fn ticks(rate: TickRate, ms: u32) -> u64 {
        rate.ticks_for(Duration::from_millis(u64::from(ms)))
            .unwrap()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    StaleTick,
    TimeExhausted,
    InvalidTransfer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub key: LifeKey,
    pub alive: bool,
    /// Authority evaluated distance and static LOS; not a client assertion.
    pub reachable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    Windup { target: LifeKey, completes: u64 },
    Cancelled { target: LifeKey },
    Strike { target: LifeKey, amount: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Engagement {
    target: LifeKey,
    expires: u64,
    ready: u64,
    windup: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    profile: Profile,
    rate: TickRate,
    observed: Option<u64>,
    advanced: Option<u64>,
    engaged: Option<Engagement>,
}
impl State {
    pub fn new(profile: Profile, rate: TickRate) -> Self {
        Self {
            profile,
            rate,
            observed: None,
            advanced: None,
            engaged: None,
        }
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn target(&self) -> Option<LifeKey> {
        self.engaged.map(|e| e.target)
    }
    pub fn windup_until(&self) -> Option<u64> {
        self.engaged.and_then(|e| e.windup)
    }
    fn check_tick(&self, now: u64) -> Result<(), Error> {
        if self.observed.is_some_and(|last| now < last) {
            Err(Error::StaleTick)
        } else {
            Ok(())
        }
    }
    /// Authority-selected victim transfer before this tick's strike evaluation.
    /// A previous windup is cancelled and pays a fresh recovery; an existing
    /// recovery is never shortened. The new victim receives a fresh windup.
    /// This telegraph/cancellation policy is an explicit Theia adaptation.
    pub fn transfer(&mut self, now: u64, target: LifeKey) -> Result<Event, Error> {
        self.check_tick(now)?;
        if self.advanced == Some(now) {
            return Err(Error::StaleTick);
        }
        let mut next = self
            .engaged
            .filter(|e| now < e.expires)
            .ok_or(Error::InvalidTransfer)?;
        if next.target.actor.epoch != target.actor.epoch {
            return Err(Error::InvalidTransfer);
        }
        if next.target == target {
            return Ok(Event::None);
        }
        let expires = now
            .checked_add(Profile::ticks(self.rate, self.profile.memory_ms))
            .ok_or(Error::TimeExhausted)?;
        let event = if next.windup.is_some() {
            next.ready = next.ready.max(
                now.checked_add(Profile::ticks(self.rate, self.profile.recovery_ms))
                    .ok_or(Error::TimeExhausted)?,
            );
            Event::Cancelled {
                target: next.target,
            }
        } else {
            Event::None
        };
        next.target = target;
        next.windup = None;
        next.expires = expires;
        self.engaged = Some(next);
        self.observed = Some(now);
        Ok(event)
    }
    /// Record a server-authorized engagement or refresh. Callers establish the
    /// permitted trigger (verified damage, hostile attempt or policy acquisition).
    /// Different attackers cannot steal/extend an unexpired engagement; refreshing
    /// the same target preserves its existing windup.
    pub fn record_hit(&mut self, now: u64, attacker: LifeKey) -> Result<bool, Error> {
        self.check_tick(now)?;
        if self
            .engaged
            .is_some_and(|e| now < e.expires && e.target != attacker)
        {
            self.observed = Some(now);
            return Ok(false);
        }
        let expires = now
            .checked_add(Profile::ticks(self.rate, self.profile.memory_ms))
            .ok_or(Error::TimeExhausted)?;
        let mut e = self
            .engaged
            .filter(|e| now < e.expires)
            .unwrap_or(Engagement {
                target: attacker,
                expires,
                ready: now,
                windup: None,
            });
        e.expires = expires;
        self.engaged = Some(e);
        self.observed = Some(now);
        Ok(true)
    }
    /// One fixed-step evaluation. Failure leaves the complete state unchanged.
    /// Profile and simulation rate are bound immutably at construction.
    pub fn advance(
        &mut self,
        now: u64,
        creature_alive: bool,
        target: Option<Target>,
    ) -> Result<Event, Error> {
        self.check_tick(now)?;
        if self.advanced == Some(now) {
            return Ok(Event::None);
        }
        let mut next = self.engaged;
        let event = if let Some(mut e) = next {
            let valid = target.is_some_and(|t| t.key == e.target && t.alive);
            if !creature_alive || !valid || now >= e.expires {
                next = None;
                if e.windup.is_some() {
                    Event::Cancelled { target: e.target }
                } else {
                    Event::None
                }
            } else if !target.is_some_and(|t| t.reachable) {
                if e.windup.take().is_some() {
                    e.ready = now
                        .checked_add(Profile::ticks(self.rate, self.profile.recovery_ms))
                        .ok_or(Error::TimeExhausted)?;
                    next = Some(e);
                    Event::Cancelled { target: e.target }
                } else {
                    Event::None
                }
            } else if let Some(completes) = e.windup {
                if now >= completes {
                    e.ready = now
                        .checked_add(Profile::ticks(self.rate, self.profile.recovery_ms))
                        .ok_or(Error::TimeExhausted)?;
                    e.windup = None;
                    next = Some(e);
                    Event::Strike {
                        target: e.target,
                        amount: self.profile.damage,
                    }
                } else {
                    Event::None
                }
            } else if now >= e.ready {
                let completes = now
                    .checked_add(Profile::ticks(self.rate, self.profile.windup_ms))
                    .ok_or(Error::TimeExhausted)?;
                e.windup = Some(completes);
                next = Some(e);
                Event::Windup {
                    target: e.target,
                    completes,
                }
            } else {
                Event::None
            }
        } else {
            Event::None
        };
        self.engaged = next;
        self.observed = Some(now);
        self.advanced = Some(now);
        Ok(event)
    }
}
