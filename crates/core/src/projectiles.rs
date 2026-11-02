//! Checked, finite homing travel only. Authority must validate identity and the
//! returned swept segment before applying any arrival effect.
use crate::{coordinates::GlobalPosition, time::TickRate};
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(try_from = "RawProfile")]
pub struct Profile {
    speed_metres_per_second: f64,
    maximum_distance_metres: f64,
    lifetime_ms: u32,
    target_radius_metres: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    speed_metres_per_second: f64,
    maximum_distance_metres: f64,
    lifetime_ms: u32,
    target_radius_metres: f64,
}
impl TryFrom<RawProfile> for Profile {
    type Error = String;
    fn try_from(raw: RawProfile) -> Result<Self, Self::Error> {
        if !raw.speed_metres_per_second.is_finite()
            || !(0.1..=100.).contains(&raw.speed_metres_per_second)
            || !raw.maximum_distance_metres.is_finite()
            || !(0.1..=256.).contains(&raw.maximum_distance_metres)
            || !(100..=10_000).contains(&raw.lifetime_ms)
            || !raw.target_radius_metres.is_finite()
            || !(0.01..=1.).contains(&raw.target_radius_metres)
            || raw.target_radius_metres > raw.maximum_distance_metres
        {
            return Err("Projectile travel exceeds checked motion budgets".into());
        }
        Ok(Self {
            speed_metres_per_second: raw.speed_metres_per_second,
            maximum_distance_metres: raw.maximum_distance_metres,
            lifetime_ms: raw.lifetime_ms,
            target_radius_metres: raw.target_radius_metres,
        })
    }
}
impl Profile {
    pub fn speed_metres_per_second(self) -> f64 {
        self.speed_metres_per_second
    }
    pub fn maximum_distance_metres(self) -> f64 {
        self.maximum_distance_metres
    }
    pub fn lifetime(self) -> Duration {
        Duration::from_millis(u64::from(self.lifetime_ms))
    }
    pub fn target_radius_metres(self) -> f64 {
        self.target_radius_metres
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Travelling,
    Arrived,
    Expired,
}
/// An Arrived result is a candidate impact, never an authorization to damage.
/// Collision along the entire segment still takes precedence over arrival.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub from: GlobalPosition,
    pub to: GlobalPosition,
    pub state: State,
}
#[derive(Clone, Debug)]
pub struct Flight {
    profile: Profile,
    rate: TickRate,
    steps: u32,
    position: GlobalPosition,
    elapsed: Duration,
    travelled: f64,
    state: State,
}
impl Flight {
    pub fn launch(profile: Profile, position: GlobalPosition, rate: TickRate) -> Self {
        Self {
            profile,
            rate,
            steps: 0,
            position,
            elapsed: Duration::ZERO,
            travelled: 0.,
            state: State::Travelling,
        }
    }
    pub fn position(&self) -> GlobalPosition {
        self.position
    }
    pub fn travelled_metres(&self) -> f64 {
        self.travelled
    }
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    pub fn state(&self) -> State {
        self.state
    }
    /// Advance one supported fixed step, clipping travel to both remaining
    /// budgets. Repeated calls after termination cannot create another segment.
    /// The caller must retire a flight when its source/target life is invalid.
    pub fn advance(&mut self, target: GlobalPosition) -> Option<Segment> {
        let from = self.position;
        if self.state != State::Travelling {
            return None;
        }
        self.steps += 1;
        // Compute from the tick count rather than accumulating rounded periods:
        // thirty 30 Hz steps must equal one second, not 999,999,990 ns.
        let elapsed = Duration::from_secs_f64(
            (f64::from(self.steps) / f64::from(self.rate.hz()))
                .min(self.profile.lifetime().as_secs_f64()),
        );
        let step = elapsed.saturating_sub(self.elapsed);
        let remaining_distance = (self.profile.maximum_distance_metres - self.travelled).max(0.);
        let budget =
            (self.profile.speed_metres_per_second * step.as_secs_f64()).min(remaining_distance);
        let start = from.metres();
        let goal = target.metres();
        let delta: [f64; 3] = std::array::from_fn(|i| goal[i] - start[i]);
        let distance = delta.iter().map(|d| d * d).sum::<f64>().sqrt();
        let required = (distance - self.profile.target_radius_metres).max(0.);
        // Contact comparisons tolerate only floating-point coordinate precision;
        // the travelled segment itself never exceeds either motion budget.
        let contact_slop = 16.
            * f64::EPSILON
            * start
                .iter()
                .chain(goal.iter())
                .map(|v| v.abs())
                .fold(1., f64::max);
        let travel = required.min(budget);
        if distance > 0. && travel > 0. {
            // This convex interpolation stays inside checked global bounds.
            let fraction = (travel / distance).clamp(0., 1.);
            self.position = GlobalPosition::new(std::array::from_fn(|i| {
                (start[i] + delta[i] * fraction).clamp(start[i].min(goal[i]), start[i].max(goal[i]))
            }))
            .expect("Convex interpolation of checked globals");
        }
        self.travelled = (self.travelled + travel).min(self.profile.maximum_distance_metres);
        self.elapsed = elapsed;
        self.state = if required <= budget + contact_slop {
            State::Arrived
        } else if self.elapsed >= self.profile.lifetime()
            || self.travelled >= self.profile.maximum_distance_metres
        {
            State::Expired
        } else {
            State::Travelling
        };
        Some(Segment {
            from,
            to: self.position,
            state: self.state,
        })
    }
}
