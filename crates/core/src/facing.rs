//! Character orientation, independent of movement, camera and selected target.
//! Y-up metres; yaw zero faces -Z, positive yaw turns left toward -X.
use crate::coordinates::GlobalPosition;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

/// One unsigned revolution. Every bit pattern is canonical; never serialize a
/// floating-point yaw supplied by an untrusted client. Resolution is ~0.0055°.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Facing(u16);
impl Facing {
    pub const fn from_units(units: u16) -> Self {
        Self(units)
    }
    pub const fn units(self) -> u16 {
        self.0
    }
    pub fn from_radians(radians: f64) -> Option<Self> {
        radians.is_finite().then(|| {
            let units = (radians.rem_euclid(TAU) / TAU * 65536.).round() as u32;
            Self((units % 65536) as u16)
        })
    }
    /// Horizontal global bearing. Coincident XZ has no direction; callers keep
    /// their previous facing. Vertical separation does not invent a yaw.
    pub fn towards(from: GlobalPosition, to: GlobalPosition) -> Option<Self> {
        let a = from.metres();
        let b = to.metres();
        let (x, z) = (b[0] - a[0], b[2] - a[2]);
        if x == 0. && z == 0. {
            return None;
        }
        Self::from_radians((-x).atan2(-z))
    }
    pub fn radians(self) -> f64 {
        f64::from(self.0) * (TAU / 65536.)
    }
    pub fn forward(self) -> [f64; 2] {
        let (sin, cos) = self.radians().sin_cos();
        [-sin, -cos]
    }
    /// Local [right, backward] axes to world XZ, normalized before quantization.
    /// Strafing/backpedalling never changes this character's facing.
    pub fn world_axes(self, local: [i16; 2]) -> Option<[i16; 2]> {
        if local.iter().any(|v| !(-1000..=1000).contains(v)) {
            return None;
        }
        let [x, z] = local.map(f64::from);
        let divisor = x.hypot(z).max(1000.);
        let (sin, cos) = self.radians().sin_cos();
        Some([
            ((cos * x + sin * z) / divisor * 1000.).round() as i16,
            ((-sin * x + cos * z) / divisor * 1000.).round() as i16,
        ])
    }
    /// Smallest signed turn from self to other, in [-pi, pi).
    pub fn difference(self, other: Self) -> f64 {
        let units = i32::from(other.0) - i32::from(self.0);
        f64::from((units + 32768).rem_euclid(65536) - 32768) * (TAU / 65536.)
    }
}

/// Full horizontal attack angle, supplied by combat data rather than targeting.
/// Range, height, line of sight and target eligibility are independent checks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc(f64);
impl Arc {
    pub fn from_degrees(degrees: f64) -> Option<Self> {
        (degrees.is_finite() && degrees > 0. && degrees <= 360.).then_some(Self(degrees))
    }
    pub fn degrees(self) -> f64 {
        self.0
    }
    pub fn contains(self, facing: Facing, from: GlobalPosition, to: GlobalPosition) -> bool {
        if self.0 == 360. {
            return true;
        }
        let a = from.metres();
        let b = to.metres();
        let (x, z) = (b[0] - a[0], b[2] - a[2]);
        // Coincident XZ has no horizontal bearing. Range/height checks still
        // apply; nonblocking characters must not require an invented direction.
        if x == 0. && z == 0. {
            return true;
        }
        let [fx, fz] = facing.forward();
        let angle = (fx * z - fz * x).atan2(fx * x + fz * z).abs();
        angle <= self.0.to_radians() * 0.5 + 1e-12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};
    fn p(x: f64, y: f64, z: f64) -> GlobalPosition {
        GlobalPosition::new([x, y, z]).unwrap()
    }
    #[test]
    fn canonical_units_reject_nonfinite_and_wrap_at_the_same_heading() {
        for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Facing::from_radians(v).is_none());
        }
        for (r, units) in [(0., 0), (TAU, 0), (-FRAC_PI_2, 49152), (PI, 32768)] {
            assert_eq!(Facing::from_radians(r).unwrap().units(), units);
        }
        for units in 0..=u16::MAX {
            let f = Facing::from_units(units);
            assert_eq!(Facing::from_radians(f.radians()), Some(f));
        }
        assert!(
            Facing::from_units(65535)
                .difference(Facing::default())
                .abs()
                < 0.0001
        );
    }
    #[test]
    fn forward_back_and_strafe_use_character_basis_at_all_cardinal_headings() {
        for (units, forward, right) in [
            (0, [0, -1000], [1000, 0]),
            (16384, [-1000, 0], [0, -1000]),
            (32768, [0, 1000], [-1000, 0]),
            (49152, [1000, 0], [0, 1000]),
        ] {
            let f = Facing::from_units(units);
            assert_eq!(f.world_axes([0, -1000]), Some(forward));
            assert_eq!(f.world_axes([1000, 0]), Some(right));
            assert_eq!(f.world_axes([0, 1000]), Some(forward.map(|v| -v)));
            assert_eq!(f.world_axes([-1000, 0]), Some(right.map(|v| -v)));
            assert_eq!(f.units(), units);
        }
        assert!(Facing::default().world_axes([1001, 0]).is_none());
    }
    #[test]
    fn rotated_diagonals_remain_bounded_without_a_diagonal_speed_bonus() {
        for units in (0..=u16::MAX).step_by(97) {
            for axes in [[1000, 1000], [-1000, 1000], [0, 0], [333, -250]] {
                let world = Facing::from_units(units).world_axes(axes).unwrap();
                assert!(world.iter().all(|v| (-1000..=1000).contains(v)));
                let [x, z] = world.map(f64::from);
                assert!(x.hypot(z) <= 1000.71);
            }
        }
    }
    #[test]
    fn frontal_arc_uses_character_facing_and_f64_global_differences() {
        let arc = Arc::from_degrees(180.).unwrap();
        let from = p(900_000_000_000., 20., -900_000_000_000.);
        let front = p(900_000_000_000., -400., -900_000_000_001.);
        let behind = p(900_000_000_000., 20., -899_999_999_999.);
        assert!(arc.contains(Facing::default(), from, front));
        assert!(!arc.contains(Facing::default(), from, behind));
        assert!(arc.contains(Facing::from_units(32768), from, behind));
        assert!(arc.contains(Facing::default(), from, from));
        let origin = p(0., 0., 0.);
        assert!(arc.contains(Facing::default(), origin, p(1., 0., 0.)));
        assert!(!arc.contains(Facing::default(), origin, p(1., 0., 0.001)));
        assert!(Arc::from_degrees(90.).unwrap().contains(
            Facing::default(),
            origin,
            p(1., 0., -1.)
        ));
        assert!(!Arc::from_degrees(90.).unwrap().contains(
            Facing::default(),
            origin,
            p(1., 0., -0.99)
        ));
        assert!(Arc::from_degrees(360.).unwrap().contains(
            Facing::default(),
            origin,
            p(0., 0., 1.)
        ));
        for v in [0., -1., 360.01, f64::NAN, f64::INFINITY] {
            assert!(Arc::from_degrees(v).is_none());
        }
    }
    #[test]
    fn global_bearing_ignores_height_and_preserves_absence_for_coincident_xz() {
        let from = p(900_000_000_000., 50., -900_000_000_000.);
        for (x, z, units) in [
            (0., -1., 0),
            (-1., 0., 16384),
            (0., 1., 32768),
            (1., 0., 49152),
        ] {
            let to = p(from.metres()[0] + x, -500., from.metres()[2] + z);
            assert_eq!(Facing::towards(from, to), Some(Facing::from_units(units)));
        }
        assert_eq!(
            Facing::towards(from, p(from.metres()[0], -500., from.metres()[2])),
            None
        );
    }
}
