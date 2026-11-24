//! Presentation only: contact authority and interaction ranges remain untouched.
use crate::network::RemotePawn;
use bevy::prelude::Component;
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
use theia_network::dialogue::Guide;

#[derive(Component, Default)]
pub(super) struct Motion {
    track: Option<(EntityKey, RemotePawn)>,
    since_change: f32,
}
impl Motion {
    pub(super) fn sample(
        &mut self,
        guide: Option<Guide>,
        seconds: f32,
        tick_seconds: f32,
    ) -> Option<GlobalPosition> {
        let Some(guide) = guide else {
            *self = Self::default();
            return None;
        };
        if self.track.as_ref().is_none_or(|(key, _)| *key != guide.key) {
            self.track = Some((
                guide.key,
                RemotePawn::stationary(guide.key.entity.value(), guide.position),
            ));
            self.since_change = 0.;
            return Some(guide.position);
        }
        let (_, track) = self.track.as_mut().unwrap();
        let seconds = seconds.clamp(0., 0.25);
        track.advance_presentation(seconds);
        self.since_change = (self.since_change + seconds).min(0.25);
        if track.global_endpoints()[1] != guide.position {
            // A stationary contact sends no changed presence. Do not interpret a
            // long pause as a slow movement interval when it starts again.
            let interval = if self.since_change > 0.2 {
                tick_seconds
            } else {
                self.since_change.max(tick_seconds)
            };
            track.retarget(guide.position, interval);
            self.since_change = 0.;
        }
        Some(track.global_sample())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::identity::{EntityId, WorldEpoch};
    fn guide(x: f64, id: u64) -> Guide {
        Guide {
            key: EntityKey {
                epoch: WorldEpoch::new(1).unwrap(),
                entity: EntityId::new(id).unwrap(),
            },
            position: GlobalPosition::new([8_000_000. + x, 4., 8_000_000.]).unwrap(),
        }
    }
    #[test]
    fn moving_contacts_blend_hold_and_restart_without_extrapolation() {
        for fps in [20, 30, 60, 120, 144] {
            let mut motion = Motion::default();
            let mut previous = motion.sample(Some(guide(0., 1)), 0., 1. / 30.).unwrap();
            let mut blended = false;
            for frame in 1..=fps * 3 {
                let seconds = f64::from(frame) / f64::from(fps);
                let x = (seconds * 30.).floor() / 30. * 7.;
                let sample = motion
                    .sample(Some(guide(x, 1)), 1. / fps as f32, 1. / 30.)
                    .unwrap();
                let px = sample.metres()[0] - 8_000_000.;
                assert!(px <= x + 1e-8 && px >= previous.metres()[0] - 8_000_000. - 1e-8);
                assert!(x - px < 0.8, "fps={fps} lag={}", x - px);
                if frame > fps && px < x && (px * 30. / 7. - (px * 30. / 7.).round()).abs() > 0.01 {
                    blended = true;
                }
                previous = sample;
            }
            if fps > 30 {
                assert!(blended, "fps={fps}");
            }
            for _ in 0..fps {
                previous = motion
                    .sample(Some(guide(21., 1)), 1. / fps as f32, 1. / 30.)
                    .unwrap();
            }
            assert_eq!(previous, guide(21., 1).position);
            let resumed = motion
                .sample(Some(guide(21. + 7. / 30., 1)), 1. / fps as f32, 1. / 30.)
                .unwrap();
            assert_eq!(resumed, previous);
            assert_eq!(
                motion.sample(Some(guide(40., 2)), 0., 1. / 30.),
                Some(guide(40., 2).position)
            );
            assert_eq!(motion.sample(None, 0., 1. / 30.), None);
            assert_eq!(
                motion.sample(Some(guide(50., 2)), 0., 1. / 30.),
                Some(guide(50., 2).position)
            );
        }
    }

    #[test]
    fn uneven_frames_rebase_and_teleport_preserve_checked_global_positions() {
        use theia_core::coordinates::LocalFrame;
        let mut motion = Motion::default();
        motion.sample(Some(guide(0., 1)), 0., 1. / 30.);
        let mut now: f64 = 0.;
        for dt in [0.007, 0.016, 0.05, 0.009, 0.08]
            .into_iter()
            .cycle()
            .take(100)
        {
            now += f64::from(dt);
            let target = guide((now * 30.).floor() / 30. * 7., 1);
            let sample = motion.sample(Some(target), dt, 1. / 30.).unwrap();
            for origin in [guide(0., 1).position, guide(128., 1).position] {
                let frame = LocalFrame::new(origin);
                let restored = frame.to_global(frame.to_local(sample).unwrap()).unwrap();
                assert!((restored.metres()[0] - sample.metres()[0]).abs() < 0.00002);
            }
            assert!(target.position.metres()[0] - sample.metres()[0] < 2.);
        }
        // Existing remote interpolation snaps a discontinuity instead of
        // sweeping a contact through unrelated world geometry.
        assert_eq!(
            motion.sample(Some(guide(500., 1)), 0.01, 1. / 30.),
            Some(guide(500., 1).position)
        );
    }
}
