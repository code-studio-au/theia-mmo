//! Bounded fixed-tick movement proposals. These never move a body, spend a
//! resource, authorize a life or grant invulnerability. The authority/controller
//! must validate the current life and sweep every returned displacement.
use crate::{content::ContentId, time::TickRate};
use serde::Deserialize;
use std::time::Duration;

mod snapshot;
pub use snapshot::{Active, RestoreError, Snapshot};

pub const MAX_PROFILE_BYTES: usize = 4096;
#[derive(Clone, Debug)]
pub struct Profile {
    id: ContentId,
    distance: f64,
    duration_ms: u32,
    cooldown_ms: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    distance_metres: f64,
    duration_ms: u32,
    cooldown_ms: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Dodge profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid dodge profile")?;
        if raw.schema != 1 {
            return Err("Unsupported dodge schema");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid dodge profile ID")?;
        if !raw.distance_metres.is_finite()
            || !(0.5..=6.).contains(&raw.distance_metres)
            || !(100..=750).contains(&raw.duration_ms)
            || !(raw.duration_ms..=10_000).contains(&raw.cooldown_ms)
            || raw.distance_metres / (f64::from(raw.duration_ms) / 1000.) > 20.
        {
            return Err("Dodge profile exceeds checked motion budgets");
        }
        Ok(Self {
            id,
            distance: raw.distance_metres,
            duration_ms: raw.duration_ms,
            cooldown_ms: raw.cooldown_ms,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn distance_metres(&self) -> f64 {
        self.distance
    }
    pub fn duration(&self) -> Duration {
        Duration::from_millis(u64::from(self.duration_ms))
    }
    pub fn cooldown(&self) -> Duration {
        Duration::from_millis(u64::from(self.cooldown_ms))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction([i16; 2]);
impl Direction {
    pub fn from_axes(axes: [i16; 2]) -> Option<Self> {
        if axes == [0, 0] || !axes.iter().all(|n| (-1000..=1000).contains(n)) {
            return None;
        }
        Some(Self(axes))
    }
    pub fn axes(self) -> [i16; 2] {
        self.0
    }
    pub fn components(self) -> [f64; 2] {
        let [x, z] = self.0.map(f64::from);
        let length = x.hypot(z);
        [x / length, z / length]
    }
}
/// Facts evaluated by the authority, never trusted fields supplied by a client.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub alive: bool,
    pub grounded: bool,
    pub terrain_ready: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    StaleTick,
    Unavailable,
    Airborne,
    Unprepared,
    Busy,
    Cooldown,
    TimeExhausted,
    SkippedTick,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    /// Requested XZ displacement. The normal capsule controller must sweep it;
    /// collision may shorten it, and lost displacement is never carried over.
    pub horizontal_metres: [f64; 2],
    pub finished: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Motion {
    direction: Direction,
    distance: f64,
    duration_ms: u32,
    rate: TickRate,
    steps: u32,
    total_steps: u32,
    requested: f64,
}
impl Motion {
    fn requested_at(self, steps: u32) -> f64 {
        let fraction = if steps == self.total_steps {
            1.
        } else {
            (f64::from(steps) * 1000. / f64::from(self.rate.hz()) / f64::from(self.duration_ms))
                .min(1.)
        };
        self.distance * fraction
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    motion: Option<Motion>,
    ready_at: u64,
    last_tick: Option<u64>,
}
impl State {
    pub fn active(&self) -> bool {
        self.motion.is_some()
    }
    pub fn ready_at(&self) -> u64 {
        self.ready_at
    }
    /// Accept only a current fixed-tick request. No movement occurs on this tick.
    /// The caller owns command deduplication and current actor/life identity.
    pub fn start(
        &mut self,
        profile: &Profile,
        rate: TickRate,
        tick: u64,
        direction: Direction,
        gate: Gate,
    ) -> Result<(), Rejection> {
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(Rejection::StaleTick);
        }
        if !gate.alive {
            return Err(Rejection::Unavailable);
        }
        if !gate.grounded {
            return Err(Rejection::Airborne);
        }
        if !gate.terrain_ready {
            return Err(Rejection::Unprepared);
        }
        if self.active() {
            return Err(Rejection::Busy);
        }
        if tick < self.ready_at {
            return Err(Rejection::Cooldown);
        }
        let cooldown = rate
            .ticks_for(profile.cooldown())
            .ok_or(Rejection::TimeExhausted)?;
        let ready_at = tick.checked_add(cooldown).ok_or(Rejection::TimeExhausted)?;
        let total_steps = u32::try_from(
            rate.ticks_for(profile.duration())
                .ok_or(Rejection::TimeExhausted)?,
        )
        .map_err(|_| Rejection::TimeExhausted)?;
        self.motion = Some(Motion {
            direction,
            distance: profile.distance,
            duration_ms: profile.duration_ms,
            rate,
            steps: 0,
            total_steps,
            requested: 0.,
        });
        self.ready_at = ready_at;
        self.last_tick = Some(tick);
        Ok(())
    }
    /// Advance once per actual simulation tick. Duplicate/old calls cannot
    /// produce another proposal. Missing ticks retire motion rather than catch
    /// up with a large displacement; cooldown remains consumed.
    pub fn advance(&mut self, tick: u64) -> Result<Option<Step>, Rejection> {
        if self.last_tick.is_some_and(|last| tick <= last) {
            return Ok(None);
        }
        let previous = self.last_tick.replace(tick);
        let Some(motion) = &mut self.motion else {
            return Ok(None);
        };
        if previous.and_then(|last| last.checked_add(1)) != Some(tick) {
            self.motion = None;
            return Err(Rejection::SkippedTick);
        }
        // At most 45 steps: checked duration <=750ms and supported rate <=60Hz.
        motion.steps += 1;
        let finished = motion.steps == motion.total_steps;
        let requested = motion.requested_at(motion.steps);
        let distance = (requested - motion.requested).max(0.);
        motion.requested = requested;
        let step = Step {
            horizontal_metres: motion.direction.components().map(|n| n * distance),
            finished,
        };
        if finished {
            self.motion = None;
        }
        Ok(Some(step))
    }
    /// Death, interruption or blocked/unprepared movement may retire the motion.
    /// Cancellation never refunds or extends its accepted cooldown.
    pub fn cancel(&mut self) -> bool {
        self.motion.take().is_some()
    }
}
