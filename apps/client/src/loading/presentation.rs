//! Keep preparing the world, but never publish its unfinished frames.
use super::{Phase, State};
use bevy::{camera::CameraOutputMode, prelude::*};

#[derive(Component)]
pub(super) struct LoadingCamera;
#[derive(Component)]
pub(super) struct WorldOutput(CameraOutputMode);
type PresentationCameras = Or<(With<Camera3d>, With<LoadingCamera>)>;
type CameraState<'a> = (
    Entity,
    &'a mut Camera,
    Has<LoadingCamera>,
    Option<&'a WorldOutput>,
);

/// Native windows otherwise become visible before their first swapchain image.
/// Keep entry unmapped until the real loading UI has had stable rendered frames.
/// This is one-time ownership: later user minimization/visibility is preserved.
pub(super) fn reveal(
    state: Res<State>,
    ready: Res<crate::capture_ready::CaptureReady>,
    mut windows: Query<&mut Window>,
    mut shown: Local<bool>,
) {
    if !*shown
        && (ready.ready() || ready.failed > 0 || state.phase == Phase::Failed)
        && let Ok(mut window) = windows.single_mut()
    {
        window.visible = true;
        *shown = true;
    }
}

pub(super) fn sync(
    mut commands: Commands,
    state: Res<State>,
    frontend: Option<Res<crate::frontend::Menu>>,
    mut cameras: Query<
        CameraState<'_>,
        (PresentationCameras, Without<crate::frontend::PreviewCamera>),
    >,
) {
    let choosing = frontend.is_some_and(|f| f.active());
    for (entity, mut camera, loading, saved) in &mut cameras {
        camera.is_active = !choosing;
        if loading {
            camera.is_active = !choosing && state.phase != Phase::Playing;
        } else if state.phase != Phase::Playing {
            if saved.is_none() {
                commands
                    .entity(entity)
                    .insert((WorldOutput(camera.output_mode), IsDefaultUiCamera));
            }
            // During world loading, warm the real view without publishing it.
            // The title/roster phase keeps it inactive: only the front-end camera
            // owns that surface, including initial monitor-tile resizing.
            camera.output_mode = CameraOutputMode::Skip;
        } else if let Some(saved) = saved {
            camera.output_mode = saved.0;
            commands.entity(entity).remove::<WorldOutput>();
        }
    }
}
