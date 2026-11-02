//! Restorable policy fields, not an authority envelope. Callers must bind the
//! exact immutable profile, rate, world, owner/life and completed tick before
//! using this data for prediction. A snapshot never authorizes client movement.
use super::{Direction, Motion, Profile, State};
use crate::time::TickRate;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Active {
    pub started_at: u64,
    pub axes: [i16; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub tick: u64,
    pub ready_at: u64,
    pub active: Option<Active>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreError {
    InvalidCooldown,
    InvalidMotion,
    TimeExhausted,
}
impl State {
    /// None is an unobserved default state; enabled runtime owners must anchor
    /// their policy with advance(current_tick) before publishing a correction.
    pub fn snapshot(&self) -> Option<Snapshot> {
        let tick = self.last_tick?;
        let active = if let Some(motion) = self.motion {
            Some(Active {
                started_at: tick.checked_sub(u64::from(motion.steps))?,
                axes: motion.direction.axes(),
            })
        } else {
            None
        };
        Some(Snapshot {
            tick,
            ready_at: self.ready_at,
            active,
        })
    }
    pub fn restore(
        profile: &Profile,
        rate: TickRate,
        snapshot: Snapshot,
    ) -> Result<Self, RestoreError> {
        let cooldown = rate
            .ticks_for(profile.cooldown())
            .ok_or(RestoreError::TimeExhausted)?;
        if (snapshot.ready_at != 0 && snapshot.ready_at < cooldown)
            || (snapshot.ready_at > snapshot.tick && snapshot.ready_at - snapshot.tick > cooldown)
        {
            return Err(RestoreError::InvalidCooldown);
        }
        let motion = if let Some(active) = snapshot.active {
            let direction = Direction::from_axes(active.axes).ok_or(RestoreError::InvalidMotion)?;
            let elapsed = snapshot
                .tick
                .checked_sub(active.started_at)
                .ok_or(RestoreError::InvalidMotion)?;
            let steps = u32::try_from(elapsed).map_err(|_| RestoreError::InvalidMotion)?;
            let total_steps = u32::try_from(
                rate.ticks_for(profile.duration())
                    .ok_or(RestoreError::TimeExhausted)?,
            )
            .map_err(|_| RestoreError::TimeExhausted)?;
            if steps >= total_steps {
                return Err(RestoreError::InvalidMotion);
            }
            let ready_at = active
                .started_at
                .checked_add(cooldown)
                .ok_or(RestoreError::TimeExhausted)?;
            if ready_at != snapshot.ready_at {
                return Err(RestoreError::InvalidCooldown);
            }
            let mut motion = Motion {
                direction,
                distance: profile.distance,
                duration_ms: profile.duration_ms,
                rate,
                steps,
                total_steps,
                requested: 0.,
            };
            motion.requested = motion.requested_at(steps);
            Some(motion)
        } else {
            None
        };
        Ok(Self {
            motion,
            ready_at: snapshot.ready_at,
            last_tick: Some(snapshot.tick),
        })
    }
}
