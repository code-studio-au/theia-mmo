//! Small presentation-only cache; amortize admitted-world ray queries.
use super::projection::{RADIUS, View};
use std::collections::{BTreeMap, VecDeque};
use theia_core::coordinates::GlobalPosition;
const STEP: f64 = 4.;
pub(super) const QUERY_BUDGET: usize = 128;
pub(super) const MAX_SAMPLES: usize = 65 * 65;
#[derive(Default)]
pub(super) struct Cache {
    bounds: Option<[i64; 4]>,
    samples: BTreeMap<[i64; 2], Option<f32>>,
    pending: VecDeque<[i64; 2]>,
}
fn key(point: GlobalPosition) -> [i64; 2] {
    let p = point.metres();
    [(p[0] / STEP).floor() as i64, (p[2] / STEP).floor() as i64]
}
impl Cache {
    pub fn height(&self, point: GlobalPosition) -> Option<f32> {
        self.samples.get(&key(point)).copied().flatten()
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    pub fn warm(
        &mut self,
        view: View,
        mut height: impl FnMut(GlobalPosition) -> Option<f32>,
    ) -> usize {
        let [x, _, z] = view.centre.metres();
        let bounds = [
            (x - RADIUS) / STEP,
            (x + RADIUS) / STEP,
            (z - RADIUS) / STEP,
            (z + RADIUS) / STEP,
        ]
        .map(|n| n.floor() as i64);
        if self.bounds != Some(bounds) {
            self.bounds = Some(bounds);
            self.samples.retain(|[x, z], _| {
                (bounds[0]..=bounds[1]).contains(x) && (bounds[2]..=bounds[3]).contains(z)
            });
            self.pending.clear();
            for z in bounds[2]..=bounds[3] {
                for x in bounds[0]..=bounds[1] {
                    if !self.samples.contains_key(&[x, z]) {
                        self.pending.push_back([x, z]);
                    }
                }
            }
            let centre = key(view.centre);
            self.pending.make_contiguous().sort_unstable_by_key(|cell| {
                cell[0].abs_diff(centre[0]) + cell[1].abs_diff(centre[1])
            });
            debug_assert!(self.samples.len() + self.pending.len() <= MAX_SAMPLES);
        }
        let mut queries = 0;
        for _ in 0..QUERY_BUDGET {
            let Some(cell) = self.pending.pop_front() else {
                break;
            };
            let point = GlobalPosition::new([
                cell[0] as f64 * STEP + STEP / 2.,
                0.,
                cell[1] as f64 * STEP + STEP / 2.,
            ]);
            self.samples.insert(cell, point.and_then(&mut height));
            queries += 1;
        }
        queries
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_samples_prioritize_the_player_and_preserve_far_signed_cells() {
        for x in [128_000_000_000.125, -128_000_000_000.125] {
            let mut cache = Cache::default();
            let view = View {
                centre: GlobalPosition::new([x, 0., x]).unwrap(),
            };
            assert_eq!(
                cache.warm(view, |p| {
                    let p = p.metres();
                    assert!((p[0] - x).abs() <= RADIUS + STEP);
                    assert!((p[2] - x).abs() <= RADIUS + STEP);
                    Some(7.)
                }),
                QUERY_BUDGET
            );
            assert_eq!(cache.height(view.centre), Some(7.));
            assert_eq!(
                cache.height(GlobalPosition::new([x + 120., 0., x + 120.]).unwrap()),
                None
            );
        }
    }
    #[test]
    fn cache_is_bounded_reuses_pan_overlap_and_caches_unknown_cells() {
        let mut cache = Cache::default();
        let mut view = View {
            centre: GlobalPosition::new([8192., 0., 8192.]).unwrap(),
        };
        let mut total = 0;
        loop {
            let queries = cache.warm(view, |_| None);
            assert!(queries <= QUERY_BUDGET);
            total += queries;
            assert!(cache.len() <= MAX_SAMPLES);
            if queries == 0 {
                break;
            }
        }
        assert_eq!(total, MAX_SAMPLES);
        assert_eq!(
            cache.warm(view, |_| panic!("known unknowns must not be requeried")),
            0
        );
        view.centre = GlobalPosition::new([8196., 0., 8192.]).unwrap();
        assert_eq!(cache.warm(view, |_| Some(3.)), 65);
        assert_eq!(cache.len(), MAX_SAMPLES);
        assert_eq!(
            cache.height(GlobalPosition::new([8325., 999., 8192.]).unwrap()),
            Some(3.)
        );
        view.centre = GlobalPosition::new([10000., 0., 10000.]).unwrap();
        assert_eq!(cache.warm(view, |_| Some(9.)), QUERY_BUDGET);
        assert_eq!(cache.len(), QUERY_BUDGET);
        assert_eq!(
            cache.height(GlobalPosition::new([8325., 0., 8192.]).unwrap()),
            None
        );
    }
}
