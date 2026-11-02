//! Pure probability contract. The authority supplies a private uniform roll and
//! current defender eligibility after validating the attack. No RNG, health,
//! resource, transport, clock or gameplay authority is introduced here.
use serde::Deserialize;

pub const BASIS_POINTS: u16 = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Hit,
    Miss,
    Deflect,
}

/// Trusted current defender facts, evaluated by the authority, not a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eligibility {
    pub deflect: bool,
}

/// An unbiased draw in [0, 10_000). Construction checks the domain; it does not
/// establish who generated the draw. Live randomness belongs to the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roll(u16);
impl Roll {
    pub const fn new(value: u16) -> Option<Self> {
        if value < BASIS_POINTS {
            Some(Self(value))
        } else {
            None
        }
    }
    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    hit_bp: u16,
    miss_bp: u16,
    deflect_bp: u16,
}

/// Explicit nominal weights, summing to exactly 100%. If deflection is not
/// eligible, that bucket becomes hit; miss probability never changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Raw")]
pub struct Table {
    hit_bp: u16,
    miss_bp: u16,
    deflect_bp: u16,
}
impl TryFrom<Raw> for Table {
    type Error = &'static str;
    fn try_from(raw: Raw) -> Result<Self, Self::Error> {
        Self::new(raw.hit_bp, raw.miss_bp, raw.deflect_bp)
            .ok_or("Hit/miss/deflect probabilities must total 10,000 basis points")
    }
}
impl Table {
    pub const fn new(hit_bp: u16, miss_bp: u16, deflect_bp: u16) -> Option<Self> {
        // Widen before addition so even malformed u16::MAX weights cannot wrap.
        if hit_bp as u32 + miss_bp as u32 + deflect_bp as u32 == BASIS_POINTS as u32 {
            Some(Self {
                hit_bp,
                miss_bp,
                deflect_bp,
            })
        } else {
            None
        }
    }
    pub const fn certain_hit() -> Self {
        Self {
            hit_bp: BASIS_POINTS,
            miss_bp: 0,
            deflect_bp: 0,
        }
    }
    pub const fn hit_bp(self) -> u16 {
        self.hit_bp
    }
    pub const fn miss_bp(self) -> u16 {
        self.miss_bp
    }
    pub const fn deflect_bp(self) -> u16 {
        self.deflect_bp
    }
    pub const fn resolve(self, roll: Roll, eligibility: Eligibility) -> Outcome {
        if roll.0 < self.miss_bp {
            Outcome::Miss
        } else if (roll.0 as u32) < self.miss_bp as u32 + self.deflect_bp as u32
            && eligibility.deflect
        {
            Outcome::Deflect
        } else {
            Outcome::Hit
        }
    }
}
