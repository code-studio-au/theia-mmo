//! Explicit ability eligibility and swing-clock policy; no authority or I/O.
use crate::{facing::Arc, time::TickRate};
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swing {
    Continue,
    Suspend,
    Delay { delay_ms: u32 },
    Reset,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawSwing {
    Continue {},
    Suspend {},
    Delay { delay_ms: u32 },
    Reset {},
}
impl<'de> Deserialize<'de> for Swing {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match RawSwing::deserialize(d)? {
            RawSwing::Continue {} => Self::Continue,
            RawSwing::Suspend {} => Self::Suspend,
            RawSwing::Delay { delay_ms } => Self::Delay { delay_ms },
            RawSwing::Reset {} => Self::Reset,
        })
    }
}
impl Swing {
    pub fn delay_ticks(self, rate: TickRate) -> Option<u64> {
        match self {
            Self::Delay { delay_ms } => rate.ticks_for(Duration::from_millis(u64::from(delay_ms))),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Facing {
    Any {},
    Frontal { arc_degrees: f64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    minimum_range_metres: f64,
    facing: Facing,
    autoattack: Swing,
}
impl Rules {
    pub(super) fn validate(self, maximum: f64) -> Result<(), &'static str> {
        if !self.minimum_range_metres.is_finite()
            || self.minimum_range_metres < 0.
            || self.minimum_range_metres >= maximum
        {
            return Err("Invalid ability minimum range");
        }
        if let Facing::Frontal { arc_degrees } = self.facing
            && Arc::from_degrees(arc_degrees).is_none()
        {
            return Err("Invalid ability frontal arc");
        }
        if let Swing::Delay { delay_ms } = self.autoattack
            && !(1..=10_000).contains(&delay_ms)
        {
            return Err("Ability swing delay outside bounds");
        }
        Ok(())
    }
    pub fn minimum_range_metres(self) -> f64 {
        self.minimum_range_metres
    }
    pub fn arc(self) -> Option<Arc> {
        match self.facing {
            Facing::Any {} => None,
            Facing::Frontal { arc_degrees } => Arc::from_degrees(arc_degrees),
        }
    }
    pub fn autoattack(self) -> Swing {
        self.autoattack
    }
}
