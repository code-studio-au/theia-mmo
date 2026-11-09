//! Exact binary metre presets; visual and keyboard placement share the zone grid.
pub(super) const STEPS: [f64; 7] = [0.125, 0.25, 0.5, 1., 2., 4., 8.];
#[derive(Clone, Copy)]
pub(super) struct Grid {
    pub origin: [f64; 3],
    pub step: f64,
}
impl Default for Grid {
    fn default() -> Self {
        Self {
            origin: [0.; 3],
            step: 1.,
        }
    }
}
impl Grid {
    pub fn coordinate(self, value: f64, axis: usize) -> Option<f64> {
        theia_zone::snap(value, *self.origin.get(axis)?, self.step).ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_share_checked_origin_relative_quantization_in_far_frames() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            for step in STEPS {
                let grid = Grid { origin, step };
                for (axis, anchor) in origin.into_iter().enumerate() {
                    for direction in [-1., 1.] {
                        let point = anchor + step * 2.3 * direction;
                        assert_eq!(
                            grid.coordinate(point, axis),
                            Some(anchor + step * 2. * direction)
                        );
                        assert_eq!(
                            grid.coordinate(anchor + step * 0.5 * direction, axis),
                            Some(anchor + step * direction)
                        );
                    }
                }
            }
        }
        assert!(
            Grid {
                step: 0.,
                ..Grid::default()
            }
            .coordinate(1., 0)
            .is_none()
        );
        assert!(Grid::default().coordinate(f64::NAN, 0).is_none());
        assert!(Grid::default().coordinate(1., 3).is_none());
    }
}
