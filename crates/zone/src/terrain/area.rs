//! Deterministic centre-level plateau with a smooth outer transition.
use super::*;

pub const LARGE_RADIUS: f64 = 160.;
pub const MAX_EDGE: f64 = 80.;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreaNoise {
    /// Maximum deviation from the sampled level, in metres.
    pub height_metres: f32,
    /// Spatial wavelength, in metres. World anchored, not rerolled per stamp.
    pub width_metres: f64,
    pub anchor: [f64; 2],
}

pub(crate) fn validate(height: f32, edge: f64, noise: Option<AreaNoise>, limit: f32) -> Result<()> {
    require(
        height.is_finite() && height.abs() <= limit,
        "Invalid area flatten height",
    )?;
    require(
        edge.is_finite() && (1. ..=MAX_EDGE).contains(&edge),
        "Edge blend must be 1..80 metres",
    )?;
    if let Some(n) = noise {
        require(
            n.height_metres.is_finite()
                && (0. ..=8.).contains(&n.height_metres)
                && n.width_metres.is_finite()
                && (4. ..=128.).contains(&n.width_metres)
                && n.anchor.iter().all(|v| v.is_finite() && v.abs() <= 1e15),
            "Invalid area noise height, width or anchor",
        )?;
    }
    Ok(())
}

pub fn influence(distance: f64, radius: f64, edge: f64) -> f32 {
    let t = ((distance - radius) / edge).clamp(0., 1.);
    // Quintic falloff: zero first/second derivatives at both ends of the band.
    (1. - t * t * t * (t * (t * 6. - 15.) + 10.)) as f32
}

impl Brush {
    pub fn maximum_radius(&self) -> f64 {
        match self.operation {
            BrushOperation::Smooth { .. } | BrushOperation::FlattenArea { .. } => LARGE_RADIUS,
            _ => 32.,
        }
    }
    pub fn support_radius(&self) -> f64 {
        self.radius_metres
            + match self.operation {
                BrushOperation::FlattenArea { edge_metres, .. } => edge_metres,
                _ => 0.,
            }
    }
    pub fn extended(&self) -> bool {
        self.radius_metres > 32. || matches!(self.operation, BrushOperation::FlattenArea { .. })
    }
    pub(crate) fn area_value(&self, position: [f64; 2], old: f32) -> f32 {
        let BrushOperation::FlattenArea {
            height,
            edge_metres,
            noise,
        } = self.operation
        else {
            unreachable!()
        };
        let distance = (position[0] - self.centre[0]).hypot(position[1] - self.centre[1]);
        let weight = influence(distance, self.radius_metres, edge_metres);
        let offset = noise.map_or(0., |n| {
            let at = |p: [f64; 2]| perlin(p[0] / n.width_metres, p[1] / n.width_metres);
            ((at(position) - at(n.anchor)) * 0.5) as f32 * n.height_metres
        });
        let target = height + offset;
        if weight == 1. {
            target
        } else {
            old + (target - old) * weight
        }
    }
}

// Fixed integer hash and gradient lattice: reproducible after save/replay and
// independent of tile ownership, stamp count, frame rate and traversal order.
fn perlin(x: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iz = z.floor() as i64;
    let u = x - x.floor();
    let v = z - z.floor();
    let fade = |t: f64| t * t * t * (t * (t * 6. - 15.) + 10.);
    let grad = |a: i64, b: i64, dx: f64, dz: f64| {
        let mut h = (a as u64).wrapping_mul(0x9e3779b97f4a7c15)
            ^ (b as u64).wrapping_mul(0xbf58476d1ce4e5b9);
        h = (h ^ (h >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        h ^= h >> 27;
        match h & 3 {
            0 => dx + dz,
            1 => dx - dz,
            2 => -dx + dz,
            _ => -dx - dz,
        }
    };
    let mix = |a: f64, b: f64, t: f64| a + (b - a) * t;
    // Each corner dot is bounded by 2; convex interpolation divided by 2
    // conservatively bounds noise in [-1, 1].
    mix(
        mix(grad(ix, iz, u, v), grad(ix + 1, iz, u - 1., v), fade(u)),
        mix(
            grad(ix, iz + 1, u, v - 1.),
            grad(ix + 1, iz + 1, u - 1., v - 1.),
            fade(u),
        ),
        fade(v),
    ) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_is_anchored_bounded_repeatable_and_plateau_is_exact() {
        let centre = [64.25, 63.75];
        let plain = Brush {
            centre,
            radius_metres: 16.,
            operation: BrushOperation::FlattenArea {
                height: 10.,
                edge_metres: 8.,
                noise: None,
            },
        };
        assert_eq!(plain.area_value([80.25, 63.75], -20.), 10.);
        assert_eq!(plain.area_value([84.25, 63.75], -20.), -5.);
        assert_eq!(plain.area_value([88.25, 63.75], -20.), -20.);
        let mut noisy = plain.clone();
        noisy.operation = BrushOperation::FlattenArea {
            height: 10.,
            edge_metres: 8.,
            noise: Some(AreaNoise {
                height_metres: 2.,
                width_metres: 9.,
                anchor: centre,
            }),
        };
        assert_eq!(noisy.area_value(centre, 99.), 10.);
        for x in 48..80 {
            for z in 48..80 {
                let p = [x as f64, z as f64];
                let h = noisy.area_value(p, 10.);
                assert!((8. ..=12.).contains(&h));
                assert_eq!(h, noisy.area_value(p, 10.));
            }
        }
        let mut patch = HeightPatch::flat([0, 0]);
        patch.apply(&plain).unwrap();
        assert_eq!(patch.heights[16 * 33 + 16], 10.);
        assert!(patch.heights[..33].iter().all(|h| *h == 0.));
    }
}
