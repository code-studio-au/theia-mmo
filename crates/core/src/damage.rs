//! Checked starting-damage arithmetic. These helpers neither roll outcomes nor
//! authorize health changes. The AP divisor is an Era reference; speed basis,
//! spell coefficients and final rounding remain explicit caller policies.

pub mod ability_impact;
pub mod ability_starting;
pub mod catalog;
pub mod melee_special;
pub mod physical;
pub mod resolution;
pub mod spell;
pub mod stats;
pub mod white_melee;

/// Engineering limits for this numerical slice, not Classic balance limits.
pub const MAX_VALUE: u32 = 1_000_000;
pub const MAX_COEFFICIENT_BP: u32 = 100_000;
const AP_SPEED_DENOMINATOR: u32 = 14_000; // AP/14 * milliseconds/1000.
const BASIS_POINTS: u32 = 10_000;

/// Inclusive authoritative base range. The caller supplies a server-owned draw;
/// checking the range does not establish randomness or trust in that caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    minimum: u32,
    maximum: u32,
}
impl Range {
    pub const fn new(minimum: u32, maximum: u32) -> Option<Self> {
        if minimum <= maximum && maximum <= MAX_VALUE {
            Some(Self { minimum, maximum })
        } else {
            None
        }
    }
    pub fn minimum(self) -> u32 {
        self.minimum
    }
    pub fn maximum(self) -> u32 {
        self.maximum
    }
    /// Checked creature/base roll, without requiring player inventory equipment.
    pub fn starting_damage(self, roll: u32) -> Result<Amount, &'static str> {
        if !self.accepts(roll) {
            return Err("Base damage roll is outside its range");
        }
        Amount::fixed(roll)
    }
    fn accepts(self, roll: u32) -> bool {
        (self.minimum..=self.maximum).contains(&roll)
    }
}

/// Exact nonnegative rational damage, bounded before any integer conversion.
/// Fractions survive starting-damage calculation; no health rounding is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amount {
    numerator: u64,
    denominator: u64,
}
impl Amount {
    fn checked(numerator: u64, denominator: u32) -> Result<Self, &'static str> {
        Self::ratio(u128::from(numerator), u128::from(denominator))
    }
    fn ratio(numerator: u128, denominator: u128) -> Result<Self, &'static str> {
        if denominator == 0
            || denominator
                .checked_mul(u128::from(MAX_VALUE))
                .is_none_or(|limit| numerator > limit)
        {
            return Err("Damage exceeds numerical bounds");
        }
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        Ok(Self {
            numerator: u64::try_from(numerator / a)
                .map_err(|_| "Damage precision exceeds bound")?,
            denominator: u64::try_from(denominator / a)
                .map_err(|_| "Damage precision exceeds bound")?,
        })
    }
    pub const fn exact_ratio(self) -> (u64, u64) {
        (self.numerator, self.denominator)
    }
    /// Explicit numerical fixture policy; not a claim of Classic health rounding.
    pub fn floor(self) -> u32 {
        (self.numerator / self.denominator) as u32
    }
    /// Explicit numerical fixture policy; rounds an exact half upward.
    pub fn nearest_half_up(self) -> u32 {
        ((u128::from(self.numerator) + u128::from(self.denominator / 2))
            / u128::from(self.denominator)) as u32
    }
    pub fn fixed(value: u32) -> Result<Self, &'static str> {
        Self::checked(u64::from(value), 1)
    }
    fn scale_bp(self, basis_points: u32) -> Result<Self, &'static str> {
        Self::ratio(
            u128::from(self.numerator) * u128::from(basis_points),
            u128::from(self.denominator) * u128::from(BASIS_POINTS),
        )
    }
    fn difference(self, remaining: Self) -> Result<Self, &'static str> {
        let left = u128::from(self.numerator) * u128::from(remaining.denominator);
        let right = u128::from(remaining.numerator) * u128::from(self.denominator);
        Self::ratio(
            left.checked_sub(right)
                .ok_or("Prevention increased damage")?,
            u128::from(self.denominator) * u128::from(remaining.denominator),
        )
    }
    fn prevent_flat(self, value: u32) -> Result<(Self, Self), &'static str> {
        let flat = u128::from(value) * u128::from(self.denominator);
        let remaining = Self::ratio(
            u128::from(self.numerator).saturating_sub(flat),
            u128::from(self.denominator),
        )?;
        Ok((remaining, self.difference(remaining)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weapon {
    range: Range,
    base_speed_ms: u32,
}
impl Weapon {
    /// This is the declared damage-scaling speed, independent of the swing clock
    /// and haste. No normalized special-attack speed is inferred here.
    pub const fn new(range: Range, base_speed_ms: u32) -> Option<Self> {
        if base_speed_ms >= 100 && base_speed_ms <= 10_000 {
            Some(Self {
                range,
                base_speed_ms,
            })
        } else {
            None
        }
    }
    pub fn range(self) -> Range {
        self.range
    }
    pub fn base_speed_ms(self) -> u32 {
        self.base_speed_ms
    }
    pub fn starting_damage(self, roll: u32, attack_power: u32) -> Result<Amount, &'static str> {
        if !self.range.accepts(roll) || attack_power > MAX_VALUE {
            return Err("Invalid weapon roll or nonnegative attack power");
        }
        let base = u64::from(roll).checked_mul(u64::from(AP_SPEED_DENOMINATOR));
        let stats = u64::from(attack_power).checked_mul(u64::from(self.base_speed_ms));
        let numerator = base
            .and_then(|base| stats.and_then(|stats| base.checked_add(stats)))
            .ok_or("Weapon starting damage overflow")?;
        Amount::checked(numerator, AP_SPEED_DENOMINATOR)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spell {
    range: Range,
    coefficient_bp: u32,
}
impl Spell {
    /// The coefficient is supplied explicitly. Rank, direct/periodic effects and
    /// downranking rules must be established by a later versioned definition.
    pub const fn new(range: Range, coefficient_bp: u32) -> Option<Self> {
        if coefficient_bp <= MAX_COEFFICIENT_BP {
            Some(Self {
                range,
                coefficient_bp,
            })
        } else {
            None
        }
    }
    pub fn range(self) -> Range {
        self.range
    }
    pub fn starting_damage(self, roll: u32, bonus_damage: u32) -> Result<Amount, &'static str> {
        if !self.range.accepts(roll) || bonus_damage > MAX_VALUE {
            return Err("Invalid spell roll or nonnegative bonus damage");
        }
        let base = u64::from(roll).checked_mul(u64::from(BASIS_POINTS));
        let stats = u64::from(bonus_damage).checked_mul(u64::from(self.coefficient_bp));
        let numerator = base
            .and_then(|base| stats.and_then(|stats| base.checked_add(stats)))
            .ok_or("Spell starting damage overflow")?;
        Amount::checked(numerator, BASIS_POINTS)
    }
}
