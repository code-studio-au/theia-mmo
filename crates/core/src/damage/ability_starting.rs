//! Starting damage for explicitly configured main-hand specials. No outcome,
//! mitigation, rounding, equipment ownership, RNG or health mutation is implied.
use super::{Amount, MAX_COEFFICIENT_BP, MAX_VALUE, Weapon};

pub mod catalog;

/// Only the AP contribution uses this speed. The weapon's base range and swing
/// clock are unchanged. Normalization is selected by data, never inferred from
/// the animation, cast time, cooldown or current attack speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApSpeed(Option<u32>);
impl ApSpeed {
    pub const fn weapon_base() -> Self {
        Self(None)
    }
    /// Explicit caller-supplied scaling speed; this does not establish a weapon
    /// class or imply every special is normalized.
    pub const fn normalized(milliseconds: u32) -> Option<Self> {
        if milliseconds >= 100 && milliseconds <= 10_000 {
            Some(Self(Some(milliseconds)))
        } else {
            None
        }
    }
    pub fn milliseconds(self, weapon: Weapon) -> u32 {
        self.0.unwrap_or(weapon.base_speed_ms())
    }
}

/// Explicit fixture-v1 order: (weapon roll + AP/14 * selected seconds) * supplied
/// weapon percentage + flat bonus. Individual Classic abilities require their
/// own evidenced coefficients/order. All intermediate amounts are bounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponSpecial {
    weapon: Weapon,
    ap_speed: ApSpeed,
    weapon_percent_bp: u32,
    flat_bonus: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Breakdown {
    pub ap_speed_ms: u32,
    pub weapon_with_ap: Amount,
    pub scaled_weapon: Amount,
    pub flat_bonus: u32,
    pub starting: Amount,
}
impl WeaponSpecial {
    pub const fn new(
        weapon: Weapon,
        ap_speed: ApSpeed,
        weapon_percent_bp: u32,
        flat_bonus: u32,
    ) -> Option<Self> {
        if weapon_percent_bp <= MAX_COEFFICIENT_BP && flat_bonus <= MAX_VALUE {
            Some(Self {
                weapon,
                ap_speed,
                weapon_percent_bp,
                flat_bonus,
            })
        } else {
            None
        }
    }
    pub fn weapon(self) -> Weapon {
        self.weapon
    }
    pub fn starting_damage(self, roll: u32, attack_power: u32) -> Result<Breakdown, &'static str> {
        let ap_speed_ms = self.ap_speed.milliseconds(self.weapon);
        let weapon_with_ap = Weapon::new(self.weapon.range(), ap_speed_ms)
            .ok_or("Invalid special AP speed")?
            .starting_damage(roll, attack_power)?;
        let scaled_weapon = weapon_with_ap.scale_bp(self.weapon_percent_bp)?;
        let (n, d) = scaled_weapon.exact_ratio();
        let starting = Amount::ratio(
            u128::from(n) + u128::from(self.flat_bonus) * u128::from(d),
            u128::from(d),
        )?;
        Ok(Breakdown {
            ap_speed_ms,
            weapon_with_ap,
            scaled_weapon,
            flat_bonus: self.flat_bonus,
            starting,
        })
    }
}
