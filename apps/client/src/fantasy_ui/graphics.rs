//! Local presentation limits using Bevy's ordinary render schedule.
use bevy::{
    prelude::*,
    render::{ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems},
    window::PresentMode,
};
use std::time::{Duration, Instant};

/// Local terrain presentation only; never changes server interest or collision.
#[derive(
    Resource, Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ViewDistance {
    #[default]
    Nearby,
    Far,
    Maximum,
}
impl ViewDistance {
    pub(crate) const ALL: [Self; 3] = [Self::Nearby, Self::Far, Self::Maximum];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Nearby => "Nearby",
            Self::Far => "Far",
            Self::Maximum => "Maximum",
        }
    }
    pub(crate) fn radius(self) -> i64 {
        match self {
            Self::Nearby => 4,
            Self::Far => 8,
            Self::Maximum => 15,
        }
    }
}
pub(crate) const CAPS: [u16; 8] = [30, 60, 90, 120, 144, 165, 240, 0];
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameCap(pub u16);
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Vsync(pub bool);
impl Default for Vsync {
    fn default() -> Self {
        Self(true)
    }
}
#[derive(Resource)]
struct UncappedBenchmark;
impl Default for FrameCap {
    fn default() -> Self {
        Self(60)
    }
}
impl FrameCap {
    pub(super) fn valid(self) -> bool {
        CAPS.contains(&self.0)
    }
    pub(crate) fn label(self) -> String {
        if self.0 == 0 {
            "Unlimited".into()
        } else {
            format!("{} FPS", self.0)
        }
    }
    fn remaining(self, elapsed: Duration) -> Duration {
        if self.0 == 0 || !self.valid() {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1. / f64::from(self.0)).saturating_sub(elapsed)
        }
    }
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<FrameCap>()
        .init_resource::<Vsync>()
        .init_resource::<ViewDistance>();
    // Preserve the existing explicit benchmark override without rewriting the
    // player's saved choice. Ordinary editor playtests do not set this flag.
    if std::env::var("THEIA_PERFORMANCE_UNCAPPED").as_deref() == Ok("1") {
        app.insert_resource(UncappedBenchmark);
    }
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .init_resource::<FrameCap>()
            .add_systems(ExtractSchedule, extract)
            .add_systems(Render, limit.after(RenderSystems::PostCleanup));
        app.add_systems(Update, presentation);
    }
}
fn extract(main: Res<MainWorld>, mut cap: ResMut<FrameCap>) {
    if main.contains_resource::<UncappedBenchmark>() {
        *cap = FrameCap(0);
    } else if let Some(value) = main.get_resource::<FrameCap>() {
        *cap = *value;
    }
}
fn presentation(
    vsync: Res<Vsync>,
    benchmark: Option<Res<UncappedBenchmark>>,
    mut windows: Query<&mut Window>,
) {
    let mode = if vsync.0 && benchmark.is_none() {
        // FIFO is the universally supported, strictly tear-free mode.
        // AutoVsync may select relaxed FIFO, which can tear on late frames.
        PresentMode::Fifo
    } else {
        PresentMode::AutoNoVsync
    };
    for mut window in &mut windows {
        if window.present_mode != mode {
            window.present_mode = mode;
        }
    }
}

fn limit(cap: Res<FrameCap>, mut previous: Local<Option<Instant>>) {
    if let Some(last) = *previous {
        let remaining = cap.remaining(last.elapsed());
        if !remaining.is_zero() {
            // Sleep the render thread after presentation; no busy-waiting,
            // changed simulation time, or input-triggered bypass of the cap.
            std::thread::sleep(remaining);
        }
    }
    *previous = Some(Instant::now());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vsync_updates_native_window_choice_without_changing_cap_or_saved_preference() {
        let mut app = App::new();
        app.init_resource::<Vsync>()
            .insert_resource(FrameCap(120))
            .add_systems(Update, presentation);
        let window = app.world_mut().spawn(Window::default()).id();
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().present_mode,
            PresentMode::Fifo
        );
        app.world_mut().resource_mut::<Vsync>().0 = false;
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().present_mode,
            PresentMode::AutoNoVsync
        );
        app.world_mut().resource_mut::<Vsync>().0 = true;
        app.world_mut().insert_resource(UncappedBenchmark);
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().present_mode,
            PresentMode::AutoNoVsync
        );
        assert!(app.world().resource::<Vsync>().0);
        app.world_mut().remove_resource::<UncappedBenchmark>();
        app.update();
        assert_eq!(
            app.world().get::<Window>(window).unwrap().present_mode,
            PresentMode::Fifo
        );
        assert_eq!(app.world().resource::<FrameCap>().0, 120);
    }
    #[test]
    fn limiter_subtracts_work_and_does_not_catch_up_after_slow_frames() {
        let cap = FrameCap(60);
        let work = Duration::from_millis(5);
        assert_eq!(
            cap.remaining(work) + work,
            Duration::from_secs_f64(1. / 60.)
        );
        assert_eq!(cap.remaining(Duration::from_millis(80)), Duration::ZERO);
        assert_eq!(FrameCap(0).remaining(Duration::ZERO), Duration::ZERO);
        for value in CAPS {
            assert!(FrameCap(value).valid());
        }
        for value in [1, 29, 59, 1000, u16::MAX] {
            assert!(!FrameCap(value).valid());
        }
    }
}
