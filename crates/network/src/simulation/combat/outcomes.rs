//! Standard seeded randomness. Entropy is acquired only when the authority is
//! configured before admission; fixed steps consume the owned ChaCha stream.
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use ring::rand::{SecureRandom, SystemRandom};
use theia_core::hit_outcomes::{BASIS_POINTS, Eligibility, Outcome, Roll, Table};
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub resolved: u64,
    pub hits: u64,
    pub misses: u64,
    pub deflects: u64,
    /// Logical unbiased rolls, not internal PRNG words/rejection samples.
    pub random_draws: u64,
}
#[derive(Clone)]
pub(super) struct Random {
    rng: ChaCha20Rng,
    draws: u64,
    base_draws: u64,
    glance_draws: u64,
}
impl Random {
    pub(super) fn new() -> crate::Result<Self> {
        let mut seed = [0; 32];
        SystemRandom::new()
            .fill(&mut seed)
            .map_err(|_| "Cannot seed combat outcome stream")?;
        Ok(Self::from_seed(seed))
    }
    fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            rng: ChaCha20Rng::from_seed(seed),
            draws: 0,
            base_draws: 0,
            glance_draws: 0,
        }
    }
    #[cfg(test)]
    pub(super) fn seed_for_test(seed: [u8; 32]) -> Self {
        Self::from_seed(seed)
    }
    pub(super) fn resolve(&mut self, table: Table, eligibility: Eligibility) -> Outcome {
        let roll = self.rng.random_range(0..BASIS_POINTS);
        self.draws = self.draws.saturating_add(1);
        table.resolve(Roll::new(roll).expect("Bounded uniform roll"), eligibility)
    }
    pub(super) fn base_roll(&mut self, range: theia_core::damage::Range) -> crate::Result<u32> {
        let next = self
            .base_draws
            .checked_add(1)
            .ok_or("Physical roll counter exhausted")?;
        let roll = self.rng.random_range(range.minimum()..=range.maximum());
        self.base_draws = next;
        Ok(roll)
    }
    pub(super) fn ordinary_roll(&mut self) -> crate::Result<Roll> {
        let next = self
            .draws
            .checked_add(1)
            .ok_or("Ordinary outcome counter exhausted")?;
        let roll = self.rng.random_range(0..BASIS_POINTS);
        self.draws = next;
        Ok(Roll::new(roll).expect("Checked uniform roll"))
    }
    pub(super) fn glance_roll(
        &mut self,
        range: theia_core::damage::white_melee::impact::GlanceRange,
    ) -> crate::Result<u8> {
        let next = self
            .glance_draws
            .checked_add(1)
            .ok_or("Ordinary glance counter exhausted")?;
        let roll = self.rng.random_range(range.minimum()..=range.maximum());
        self.glance_draws = next;
        Ok(roll)
    }
    pub(super) fn glance_draws(&self) -> u64 {
        self.glance_draws
    }
    pub(super) fn base_draws(&self) -> u64 {
        self.base_draws
    }
    pub(super) fn draws(&self) -> u64 {
        self.draws
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seeded_server_stream_replays_without_external_entropy_and_obeys_eligibility() {
        let table = Table::new(8500, 1000, 500).unwrap();
        let mut a = Random::from_seed([7; 32]);
        let mut b = Random::from_seed([7; 32]);
        let mut seen = [0usize; 3];
        for i in 0..10000 {
            let eligibility = Eligibility {
                deflect: i % 2 == 0,
            };
            let outcome = a.resolve(table, eligibility);
            assert_eq!(outcome, b.resolve(table, eligibility));
            assert!(eligibility.deflect || outcome != Outcome::Deflect);
            seen[match outcome {
                Outcome::Hit => 0,
                Outcome::Miss => 1,
                Outcome::Deflect => 2,
            }] += 1;
        }
        assert!(seen.into_iter().all(|count| count > 0));
        assert_eq!(a.draws(), 10000);
        assert_eq!(b.draws(), 10000);
    }
    #[test]
    fn exhausted_base_counter_fails_without_consuming_private_randomness() {
        let range = theia_core::damage::Range::new(0, 1_000_000).unwrap();
        let mut actual = Random::from_seed([13; 32]);
        actual.base_draws = u64::MAX;
        let mut expected = actual.clone();
        expected.base_draws = 0;
        assert!(actual.base_roll(range).is_err());
        assert_eq!(actual.base_draws, u64::MAX);
        actual.base_draws = 0;
        for _ in 0..100 {
            assert_eq!(
                actual.base_roll(range).unwrap(),
                expected.base_roll(range).unwrap()
            );
        }
        assert_eq!(actual.base_draws(), 100);
        assert_eq!(actual.draws(), 0);
    }
    #[test]
    fn exhausted_ordinary_counter_preserves_private_stream_before_any_draw() {
        let mut actual = Random::from_seed([29; 32]);
        actual.draws = u64::MAX;
        let mut expected = actual.clone();
        assert!(actual.ordinary_roll().is_err());
        assert_eq!(actual.draws, u64::MAX);
        actual.draws = 0;
        expected.draws = 0;
        let range = theia_core::damage::Range::new(0, 1_000_000).unwrap();
        for _ in 0..100 {
            assert_eq!(
                actual.base_roll(range).unwrap(),
                expected.base_roll(range).unwrap()
            );
        }
    }
    #[test]
    fn exhausted_glance_counter_preserves_private_stream_before_any_draw() {
        let range =
            theia_core::damage::white_melee::impact::GlanceRange::melee_reference_v1(300, 315)
                .unwrap();
        let mut actual = Random::from_seed([31; 32]);
        actual.glance_draws = u64::MAX;
        let mut expected = actual.clone();
        assert!(actual.glance_roll(range).is_err());
        assert_eq!(actual.glance_draws, u64::MAX);
        actual.glance_draws = 0;
        expected.glance_draws = 0;
        for _ in 0..100 {
            assert_eq!(
                actual.ordinary_roll().unwrap(),
                expected.ordinary_roll().unwrap()
            );
        }
    }
    #[test]
    fn glancing_draws_are_inclusive_and_counted_separately_from_base_and_outcome() {
        let range =
            theia_core::damage::white_melee::impact::GlanceRange::melee_reference_v1(300, 315)
                .unwrap();
        let mut actual = Random::from_seed([37; 32]);
        let mut expected = actual.clone();
        for _ in 0..1000 {
            let roll = actual.glance_roll(range).unwrap();
            assert_eq!(roll, expected.glance_roll(range).unwrap());
            assert!(range.pick(roll).is_some());
        }
        assert_eq!(
            (actual.glance_draws(), actual.base_draws(), actual.draws()),
            (1000, 0, 0)
        );
    }
}
