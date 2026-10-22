use std::time::Duration;

/// Qualified simulation rates. Input sampling and rendering have independent clocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TickRate {
    Hz20,
    #[default]
    Hz30,
    Hz60,
}

impl TickRate {
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            20 => Some(Self::Hz20),
            30 => Some(Self::Hz30),
            60 => Some(Self::Hz60),
            _ => None,
        }
    }
    pub const fn hz(self) -> u32 {
        match self {
            Self::Hz20 => 20,
            Self::Hz30 => 30,
            Self::Hz60 => 60,
        }
    }
    pub fn period(self) -> Duration {
        Duration::from_secs_f64(1. / f64::from(self.hz()))
    }
    pub fn seconds(self) -> f32 {
        1. / self.hz() as f32
    }
    /// Ceiling conversion keeps deadlines from expiring before the requested time.
    pub fn ticks_for(self, duration: Duration) -> Option<u64> {
        let scaled = duration.as_nanos().checked_mul(u128::from(self.hz()))?;
        u64::try_from(scaled.div_ceil(1_000_000_000)).ok()
    }
}

/// Fixed-memory logarithmic histogram. Quantiles are inclusive upper bounds in
/// nanoseconds (powers of two minus one), not exact sampled percentiles.
#[derive(Clone, Debug)]
pub struct DurationHistogram {
    buckets: [u64; 64],
    samples: u64,
}
impl Default for DurationHistogram {
    fn default() -> Self {
        Self {
            buckets: [0; 64],
            samples: 0,
        }
    }
}
impl DurationHistogram {
    pub fn record(&mut self, duration: Duration) {
        let nanos = duration.as_nanos().min(u64::MAX as u128) as u64;
        let bucket = (64 - nanos.leading_zeros() as usize).min(63);
        self.buckets[bucket] = self.buckets[bucket].saturating_add(1);
        self.samples = self.samples.saturating_add(1);
    }
    pub fn percentile_upper(&self, percentile: u8) -> Duration {
        if self.samples == 0 {
            return Duration::ZERO;
        }
        let target =
            (u128::from(self.samples) * u128::from(percentile.clamp(1, 100))).div_ceil(100);
        let mut count = 0u128;
        for (index, &samples) in self.buckets.iter().enumerate() {
            count += u128::from(samples);
            if count >= target {
                return if index == 63 {
                    Duration::MAX
                } else {
                    Duration::from_nanos((1u64 << index) - 1)
                };
            }
        }
        Duration::MAX
    }
}

#[derive(Clone, Debug, Default)]
pub struct TickMetrics {
    pub steps: u64,
    pub missed_deadlines: u64,
    pub total_work: Duration,
    pub max_work: Duration,
    pub max_lateness: Duration,
    pub work: DurationHistogram,
    pub lateness: DurationHistogram,
}

impl TickMetrics {
    /// Record simulation/replication work without sleeping or initiating I/O.
    pub fn record(&mut self, rate: TickRate, work: Duration, lateness: Duration) {
        self.record_budget(rate.period(), work, lateness);
    }
    pub fn record_budget(&mut self, budget: Duration, work: Duration, lateness: Duration) {
        self.steps = self.steps.saturating_add(1);
        self.total_work = self.total_work.saturating_add(work);
        self.max_work = self.max_work.max(work);
        self.max_lateness = self.max_lateness.max(lateness);
        self.work.record(work);
        self.lateness.record(lateness);
        if work.saturating_add(lateness) >= budget {
            self.missed_deadlines = self.missed_deadlines.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_histogram_reports_inclusive_quantile_bounds() {
        let mut histogram = DurationHistogram::default();
        assert_eq!(histogram.percentile_upper(95), Duration::ZERO);
        for nanos in 0..100 {
            histogram.record(Duration::from_nanos(nanos));
        }
        assert_eq!(histogram.percentile_upper(50), Duration::from_nanos(63));
        assert_eq!(histogram.percentile_upper(95), Duration::from_nanos(127));
        histogram.record(Duration::MAX);
        assert_eq!(histogram.percentile_upper(100), Duration::MAX);
    }

    #[test]
    fn rates_and_simulation_deadlines_are_checked() {
        for hz in [20, 30, 60] {
            let rate = TickRate::from_hz(hz).unwrap();
            assert_eq!(rate.hz(), hz);
            assert_eq!(
                rate.ticks_for(Duration::from_millis(1500)),
                Some(hz as u64 * 3 / 2)
            );
            assert_eq!(rate.ticks_for(Duration::ZERO), Some(0));
            assert_eq!(rate.ticks_for(Duration::from_nanos(1)), Some(1));
        }
        assert!(TickRate::from_hz(0).is_none());
        assert!(TickRate::from_hz(1000).is_none());
        assert_eq!(TickRate::Hz60.ticks_for(Duration::MAX), None);
    }

    #[test]
    fn metrics_distinguish_slow_work_and_late_wakeups() {
        let mut metrics = TickMetrics::default();
        metrics.record(TickRate::Hz30, Duration::from_millis(2), Duration::ZERO);
        metrics.record(TickRate::Hz30, Duration::from_millis(40), Duration::ZERO);
        metrics.record(
            TickRate::Hz30,
            Duration::from_millis(1),
            Duration::from_millis(50),
        );
        assert_eq!(metrics.steps, 3);
        assert_eq!(metrics.missed_deadlines, 2);
        assert_eq!(metrics.total_work, Duration::from_millis(43));
        assert_eq!(metrics.max_work, Duration::from_millis(40));
        metrics.record(
            TickRate::Hz30,
            Duration::from_millis(4),
            Duration::from_millis(30),
        );
        assert_eq!(metrics.missed_deadlines, 3);
    }
}
