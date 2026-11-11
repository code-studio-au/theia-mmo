//! Controlled preview captures for comparison with the pinned upstream host.
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    time::TimeUpdateStrategy,
};
use bevy_sprinkles::{
    asset::ParticlesAsset,
    prelude::{EmitterRuntime, ParticleSystemRuntime},
};
use bevy_sprinkles_editor::state::EditorState;
use std::time::{Duration, Instant};

fn main() {
    bevy_sprinkles_editor::standalone_app()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .add_systems(PostUpdate, capture)
        .run();
}

fn capture(
    mut commands: Commands,
    state: Res<EditorState>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
    mut emitters: Query<&mut EmitterRuntime>,
    mut systems: Query<&mut ParticleSystemRuntime>,
    mut seeded: Local<bool>,
    mut warm: Local<Option<Instant>>,
    mut frame: Local<Option<usize>>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.current_project.is_none() || emitters.is_empty() {
        return;
    }
    if !*seeded {
        let Some(mut asset) = state
            .current_project
            .as_ref()
            .and_then(|handle| assets.get_mut(handle))
        else {
            return;
        };
        for (index, emitter) in asset.emitters.iter_mut().enumerate() {
            emitter.time.fixed_seed = Some(1000 + index as u32);
        }
        for mut system in &mut systems {
            system.global_seed = 1000;
        }
        *seeded = true;
        return;
    }
    let start = *warm.get_or_insert_with(Instant::now);
    if start.elapsed() < Duration::from_secs(4) {
        return;
    }
    let Some(current) = *frame else {
        for mut emitter in &mut emitters {
            let seed = 1000 + emitter.emitter_index as u32;
            emitter.restart(Some(seed));
        }
        *frame = Some(0);
        return;
    };
    let next = current + 1;
    if [30, 60, 120, 180, 300].contains(&next) {
        let path = std::path::PathBuf::from(
            std::env::var_os("SPRINKLES_PARITY_OUTPUT").expect("parity output"),
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.join(format!("frame-{next}.png"))));
    }
    *frame = Some(next);
    if next >= 420 {
        exit.write(AppExit::Success);
    }
}
