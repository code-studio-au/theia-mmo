//! Observe renderer pipeline readiness through Bevy's RenderApp/MainWorld pattern.
use bevy::{
    prelude::*,
    render::{
        ExtractSchedule, MainWorld, RenderApp,
        render_resource::{CachedPipelineState, PipelineCache},
    },
    shader::ShaderCacheError,
};

pub struct CaptureReadyPlugin;
impl Plugin for CaptureReadyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaptureReady>();
        app.sub_app_mut(RenderApp)
            .add_systems(ExtractSchedule, observe);
    }
}

#[derive(Resource, Default)]
pub struct CaptureReady {
    pub pending: usize,
    pub failed: usize,
    pub compiled: usize,
    pub clear_frames: u32,
}
impl CaptureReady {
    fn record(&mut self, pending: usize, failed: usize, compiled: usize) {
        self.pending = pending;
        self.failed = failed;
        self.compiled = compiled;
        self.clear_frames = if pending == 0 && failed == 0 && compiled > 0 {
            self.clear_frames.saturating_add(1)
        } else {
            0
        };
    }
    pub fn ready(&self) -> bool {
        self.clear_frames >= 3
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
fn observe(mut main: ResMut<MainWorld>, pipelines: Res<PipelineCache>) {
    let retrying = pipelines
        .pipelines()
        .filter(|p| matches!(&p.state, CachedPipelineState::Err(error) if !terminal_error(error)))
        .count();
    let pending = pipelines.waiting_pipelines().count().max(retrying);
    let failed = pipelines
        .pipelines()
        .filter(|p| matches!(&p.state, CachedPipelineState::Err(error) if terminal_error(error)))
        .count();
    let compiled = pipelines
        .pipelines()
        .filter(|p| matches!(p.state, CachedPipelineState::Ok(_)))
        .count();
    if let Some(mut ready) = main.get_resource_mut::<CaptureReady>() {
        if ready.failed == 0 && failed > 0 {
            for pipeline in pipelines.pipelines() {
                if let CachedPipelineState::Err(error) = &pipeline.state
                    && terminal_error(error)
                {
                    error!("Render pipeline compilation failed: {error}");
                }
            }
        }
        ready.record(pending, failed, compiled);
    }
}
fn terminal_error(error: &ShaderCacheError) -> bool {
    matches!(
        error,
        ShaderCacheError::ProcessShaderError(_) | ShaderCacheError::CreateShaderModule(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shader_import_retries_are_distinct_from_permanent_compilation_failures() {
        assert!(!terminal_error(&ShaderCacheError::ShaderNotLoaded(
            Handle::<bevy::shader::Shader>::default().id()
        )));
        assert!(!terminal_error(
            &ShaderCacheError::ShaderImportNotYetAvailable
        ));
        assert!(terminal_error(&ShaderCacheError::ProcessShaderError(
            "synthetic test error".into()
        )));
        assert!(terminal_error(&ShaderCacheError::CreateShaderModule(
            "synthetic test error".into()
        )));
    }
    #[test]
    fn capture_needs_observed_compilation_and_stable_ready_frames() {
        let mut ready = CaptureReady::default();
        assert!(!ready.ready());
        ready.record(0, 0, 0);
        assert!(!ready.ready());
        ready.record(1, 0, 2);
        ready.record(0, 0, 3);
        ready.record(0, 0, 3);
        assert!(!ready.ready());
        ready.record(0, 0, 3);
        assert!(ready.ready());
        ready.record(1, 0, 3);
        assert!(!ready.ready());
        ready.record(0, 1, 3);
        assert!(!ready.ready());
        ready.reset();
        assert_eq!(ready.compiled, 0);
        assert!(!ready.ready());
    }
}
