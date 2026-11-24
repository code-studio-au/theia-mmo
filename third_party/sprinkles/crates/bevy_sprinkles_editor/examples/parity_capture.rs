//! Timed preview captures for comparison with a separately instrumented upstream host.
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use bevy_sprinkles::prelude::EmitterRuntime;
use bevy_sprinkles_editor::state::EditorState;

fn main() {
    bevy_sprinkles_editor::standalone_app()
        .add_systems(Update, capture)
        .run();
}

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<EditorState>,
    emitters: Query<&EmitterRuntime>,
    mut start: Local<Option<f64>>,
    mut stage: Local<usize>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.current_project.is_none() || emitters.is_empty() {
        return;
    }
    let started = *start.get_or_insert(time.elapsed_secs_f64());
    let elapsed = time.elapsed_secs_f64() - started;
    if *stage < 3 && elapsed >= [3., 6., 9.][*stage] {
        let path = std::path::PathBuf::from(
            std::env::var_os("SPRINKLES_PARITY_OUTPUT").expect("parity output"),
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.join(format!("capture-{}.png", *stage))));
        *stage += 1;
    }
    if *stage == 3 && elapsed > 12. {
        exit.write(AppExit::Success);
    }
}
