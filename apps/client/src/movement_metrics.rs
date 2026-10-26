//! Bounded render-motion observations. Never feeds movement or presentation.
use theia_core::coordinates::{ChunkId, GlobalPosition};
use theia_network::wire::Intent;

#[derive(Clone, Copy, PartialEq)]
pub struct Context {
    pub intent: Intent,
    pub speed: u8,
    pub stream: u64,
    pub accepted_ready: u64,
    pub reported_ready: u64,
    pub patch: Option<ChunkId>,
    pub clock_resets: u64,
}
#[derive(Clone, Copy)]
struct Sample {
    at: f64,
    position: GlobalPosition,
    context: Context,
}

#[derive(Default, serde::Serialize)]
pub struct Metrics {
    pub samples: u64,
    pub input_or_world_transitions: u64,
    pub pending_readiness: u64,
    pub idle: u64,
    pub settling: u64,
    pub frame_gaps: u64,
    pub large_displacements: u64,
    pub steady_samples: u64,
    pub steady_held_frames: u64,
    pub max_frame_ms: f64,
    pub max_steady_speed_ratio: f64,
    /// Speed/expected speed: <=0.1, 0.5, 0.9, 1.1, 1.5, 2, 4, then >4.
    pub steady_speed_ratio_buckets: [u64; 8],
    #[serde(skip)]
    previous: Option<Sample>,
    #[serde(skip)]
    quiet_since: f64,
}
impl Metrics {
    pub fn observe(&mut self, at: f64, position: GlobalPosition, context: Context) {
        let current = Sample {
            at,
            position,
            context,
        };
        let Some(previous) = self.previous.replace(current) else {
            self.quiet_since = at;
            return;
        };
        let dt = at - previous.at;
        if !dt.is_finite() || dt <= 0. {
            self.quiet_since = at;
            return;
        }
        self.samples += 1;
        self.max_frame_ms = self.max_frame_ms.max(dt * 1000.);
        let a = position.metres();
        let b = previous.position.metres();
        let distance = (a[0] - b[0]).hypot(a[2] - b[2]);
        if distance >= 64. || (a[1] - b[1]).abs() > 32. {
            self.large_displacements += 1;
            self.quiet_since = at;
            return;
        }
        if dt > 0.1 {
            self.frame_gaps += 1;
            self.quiet_since = at;
            return;
        }
        if context != previous.context {
            self.input_or_world_transitions += 1;
            self.quiet_since = at;
            return;
        }
        if context.accepted_ready != context.reported_ready || context.patch.is_none() {
            self.pending_readiness += 1;
            self.quiet_since = at;
            return;
        }
        let length = (f64::from(context.intent.axes[0]).hypot(f64::from(context.intent.axes[1]))
            / 1000.)
            .min(1.);
        if length == 0. {
            self.idle += 1;
            return;
        }
        if at - self.quiet_since < 0.5 {
            self.settling += 1;
            return;
        }
        let expected =
            length * if context.intent.sprint { 8. } else { 5. } * f64::from(context.speed);
        if expected <= 0. {
            self.settling += 1;
            return;
        }
        self.steady_samples += 1;
        if distance < 0.00001 {
            self.steady_held_frames += 1;
        }
        let ratio = distance / dt / expected;
        self.max_steady_speed_ratio = self.max_steady_speed_ratio.max(ratio);
        self.steady_speed_ratio_buckets
            [[0.1, 0.5, 0.9, 1.1, 1.5, 2., 4.].partition_point(|&bound| ratio > bound)] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            intent: Intent {
                axes: [1000, 0],
                ..Intent::default()
            },
            speed: 1,
            stream: 1,
            accepted_ready: 1,
            reported_ready: 1,
            patch: Some(ChunkId { x: 62, z: 62 }),
            clock_resets: 0,
        }
    }
    fn position(x: f64) -> GlobalPosition {
        GlobalPosition::new([8000. + x, 0., 8000.]).unwrap()
    }
    #[test]
    fn constant_global_speed_is_steady_and_input_transitions_do_not_create_false_holds() {
        let mut metrics = Metrics::default();
        let mut c = context();
        for i in 0..=600 {
            let t = f64::from(i) / 60.;
            metrics.observe(t, position(t * 5.), c);
        }
        assert!(metrics.steady_samples > 500);
        assert_eq!(metrics.steady_held_frames, 0);
        assert_eq!(
            metrics.steady_speed_ratio_buckets[3],
            metrics.steady_samples
        );
        let before = metrics.steady_samples;
        c.intent.axes = [0, 0];
        for i in 1..=60 {
            metrics.observe(10. + f64::from(i) / 60., position(50.), c);
        }
        assert!(metrics.idle > 0);
        assert_eq!(metrics.steady_samples, before);
        c.intent.axes = [0, 1000];
        for i in 1..=20 {
            metrics.observe(11. + f64::from(i) / 60., position(50.), c);
        }
        assert_eq!(
            metrics.steady_held_frames, 0,
            "A start/turn's settling window is separate"
        );
        for i in 21..=60 {
            metrics.observe(11. + f64::from(i) / 60., position(50.), c);
        }
        assert!(
            metrics.steady_held_frames > 0,
            "Sustained blocked/held input must remain observable"
        );
        assert_eq!(
            metrics.steady_speed_ratio_buckets.iter().sum::<u64>(),
            metrics.steady_samples
        );
    }
    #[test]
    fn loading_teleports_and_long_frames_are_reported_separately_from_steady_motion() {
        let mut metrics = Metrics::default();
        let mut c = context();
        metrics.observe(0., position(0.), c);
        c.reported_ready = 2;
        metrics.observe(0.01, position(0.), c);
        metrics.observe(0.02, position(0.), c);
        assert_eq!(metrics.pending_readiness, 1);
        c.accepted_ready = 2;
        metrics.observe(0.03, position(0.), c);
        metrics.observe(0.04, position(1000.), c);
        metrics.observe(0.3, position(1000.), c);
        assert_eq!(metrics.large_displacements, 1);
        assert_eq!(metrics.frame_gaps, 1);
        assert_eq!(metrics.steady_samples, 0);
        assert_eq!(
            metrics.samples,
            metrics.input_or_world_transitions
                + metrics.pending_readiness
                + metrics.idle
                + metrics.settling
                + metrics.frame_gaps
                + metrics.large_displacements
                + metrics.steady_samples
        );
    }
}
