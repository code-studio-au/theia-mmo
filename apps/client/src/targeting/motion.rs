//! Reuses the existing remote-pawn interpolation math for moving target visuals.
//! Lifecycle/selection/hits stay authoritative; fresh keys start a fresh track.
use super::Assets;
use crate::{
    floating_origin::Frame,
    network::{Network, RemotePawn},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub enabled: bool,
    pub catch_up_speed_mps: f32,
    pub samples: u64,
    pub moving_frames: u64,
    pub authority_updates: u64,
    pub max_authority_lag_m: f64,
    pub max_frame_displacement_m: f64,
    pub max_presented_speed_mps: f64,
}
#[derive(SystemParam)]
pub(super) struct Presentation<'w> {
    pub network: Res<'w, Network>,
    pub frame: Res<'w, Frame>,
    pub assets: Res<'w, Assets>,
    pub time: Res<'w, Time>,
    pub metrics: ResMut<'w, Metrics>,
}
#[derive(Component)]
pub(crate) struct TargetMotion {
    track: RemotePawn,
    tick: u64,
    previous: GlobalPosition,
}
impl TargetMotion {
    pub(super) fn new(key: EntityKey, position: GlobalPosition, tick: u64) -> Self {
        Self {
            track: RemotePawn::stationary(key.entity.value(), position),
            tick,
            previous: position,
        }
    }
    pub(crate) fn global(&self) -> GlobalPosition {
        self.track.global_sample()
    }
    pub(super) fn observe(
        &mut self,
        position: GlobalPosition,
        tick: u64,
        hz: u32,
        seconds: f32,
        metrics: &mut Metrics,
    ) -> GlobalPosition {
        if tick > self.tick {
            let a = self.global().metres();
            let b = position.metres();
            let catch_up = ((a[0] - b[0]).hypot(a[2] - b[2])
                / f64::from(metrics.catch_up_speed_mps.max(0.1))) as f32;
            let interval = ((tick - self.tick) as f32 / hz.max(1) as f32).max(catch_up);
            self.track.retarget(position, interval);
            self.tick = tick;
            metrics.authority_updates += 1;
        }
        self.track.advance_presentation(seconds);
        let global = self.global();
        let a = global.metres();
        let b = position.metres();
        let p = self.previous.metres();
        let displacement = (a[0] - p[0]).hypot(a[2] - p[2]);
        metrics.samples += 1;
        if displacement > 0.00001 {
            metrics.moving_frames += 1;
        }
        metrics.max_authority_lag_m = metrics
            .max_authority_lag_m
            .max((a[0] - b[0]).hypot(a[2] - b[2]));
        metrics.max_frame_displacement_m = metrics.max_frame_displacement_m.max(displacement);
        if seconds > 0.0001 {
            metrics.max_presented_speed_mps = metrics
                .max_presented_speed_mps
                .max(displacement / f64::from(seconds));
        }
        self.previous = global;
        global
    }
}
pub(super) fn install(app: &mut App, speed: Option<f32>) {
    app.insert_resource(Metrics {
        enabled: speed.is_some(),
        catch_up_speed_mps: speed.map_or(0., |v| (v * 2.).max(1.)),
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::identity::{EntityId, WorldEpoch};
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn position(x: f64) -> GlobalPosition {
        GlobalPosition::new([15000. + x, 1., 15000.]).unwrap()
    }
    #[test]
    fn delayed_burst_catch_up_does_not_lurch_during_the_measured_sub_metre_window() {
        let mut motion = TargetMotion::new(key(1), position(0.), 0);
        let mut metrics = Metrics {
            enabled: true,
            catch_up_speed_mps: 4.,
            ..default()
        };
        for tick in 10..=16 {
            motion.observe(
                position(0.5 + (tick - 10) as f64 / 30. * 2.),
                tick,
                30,
                1. / 144.,
                &mut metrics,
            );
        }
        for _ in 0..60 {
            motion.observe(position(0.9), 16, 30, 1. / 144., &mut metrics);
        }
        assert!(
            metrics.max_presented_speed_mps <= 4.001,
            "{}",
            metrics.max_presented_speed_mps
        );
        assert!((motion.global().metres()[0] - 15000.9).abs() < 0.00001);
    }
    #[test]
    fn a_fresh_life_starts_at_home_without_interpolating_the_dead_life() {
        let mut metrics = Metrics {
            enabled: true,
            catch_up_speed_mps: 4.,
            ..default()
        };
        let mut old = TargetMotion::new(key(1), position(20.), 20);
        old.observe(position(20.1), 21, 30, 0.01, &mut metrics);
        let fresh = TargetMotion::new(key(2), position(0.), 22);
        assert_eq!(fresh.global(), position(0.));
        assert!(old.global().metres()[0] > 15020.);
    }
}
