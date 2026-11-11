use crate::hosting::AuthoringAppExt;
use bevy::color::palettes::tailwind::ZINC_950;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_sprinkles::prelude::*;

use crate::io::{EditorData, examples_dir, project_path, save_editor_data, working_dir};
use crate::project::load_project_from_path;
use crate::state::{DirtyState, EditorState, Inspectable, Inspecting};
use crate::utils::simplify_path;
use crate::viewport::{
    AabbGeneration, CameraSettings, ViewportInputState, despawn_preview_on_project_change,
    handle_generate_aabb_request, handle_playback_play_event, handle_playback_reset_event,
    handle_playback_seek_event, handle_respawn_colliders, handle_respawn_emitters, orbit_camera,
    respawn_preview_on_emitter_change, setup_camera, setup_floor, spawn_preview_particle_system,
    sync_inspected_emitter_aabb, sync_playback_state, sync_viewport_settings, tick_aabb_generation,
    zoom_camera,
};
use std::path::{Path, PathBuf};

#[derive(Resource, Default)]
struct CliArgs {
    initial_file: Option<String>,
}

impl CliArgs {
    fn from_env() -> Self {
        Self {
            initial_file: std::env::args().nth(1),
        }
    }
}

pub struct SprinklesEditorPlugin;

impl Plugin for SprinklesEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::hosting::AuthoringHostPlugin);
        let embedded = app
            .world()
            .resource::<crate::hosting::AuthoringHost>()
            .embedded;
        if !embedded {
            app.insert_resource(ClearColor(ZINC_950.into()));
        }
        let args = if embedded {
            CliArgs { initial_file: None }
        } else {
            CliArgs::from_env()
        };
        app.insert_resource(args)
            .add_plugins(crate::assets::plugin)
            .add_plugins(SprinklesPlugin)
            .add_plugins(crate::viewport_bounds::plugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(crate::io::plugin)
            .add_plugins(crate::state::plugin)
            .add_plugins(crate::project::plugin)
            .init_resource::<CameraSettings>()
            .init_resource::<ViewportInputState>()
            .init_resource::<AabbGeneration>()
            .add_observer(respawn_preview_on_emitter_change)
            .add_observer(handle_respawn_emitters)
            .add_observer(handle_respawn_colliders)
            .add_observer(handle_playback_play_event)
            .add_observer(handle_playback_reset_event)
            .add_observer(handle_playback_seek_event)
            .add_observer(handle_generate_aabb_request)
            .add_systems(Startup, (setup_camera, setup_floor, load_initial_project))
            .add_authoring_systems((
                orbit_camera,
                zoom_camera,
                spawn_preview_particle_system,
                despawn_preview_on_project_change,
                sync_playback_state,
                sync_viewport_settings,
                sync_inspected_emitter_aabb,
                tick_aabb_generation,
            ));
    }
}

fn load_initial_project(
    cli_args: Res<CliArgs>,
    mut editor_state: ResMut<EditorState>,
    mut editor_data: ResMut<EditorData>,
    mut dirty_state: ResMut<DirtyState>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
) {
    if let Some(file) = &cli_args.initial_file {
        let cwd_path = working_dir().join(file);
        let path = if cwd_path.exists() {
            cwd_path
        } else {
            project_path(file)
        };
        if let Ok(result) = load_project_from_path(&path) {
            let has_emitters = !result.asset.emitters.is_empty();
            let handle = assets.add(result.asset);
            editor_state.open_project(handle, path, &mut dirty_state, result.was_migrated, &assets);
            if has_emitters {
                editor_state.inspecting = Some(Inspecting {
                    kind: Inspectable::Emitter,
                    index: 0,
                });
            }
            editor_data.cache.add_recent_project(file.clone());
            save_editor_data(&editor_data);
            return;
        }
    }

    if let Some(location) = &editor_data.cache.last_opened_project.clone() {
        let path = project_path(location);
        if path.exists()
            && let Ok(result) = load_project_from_path(&path)
        {
            let has_emitters = !result.asset.emitters.is_empty();
            let handle = assets.add(result.asset);
            editor_state.open_project(handle, path, &mut dirty_state, result.was_migrated, &assets);
            if has_emitters {
                editor_state.inspecting = Some(Inspecting {
                    kind: Inspectable::Emitter,
                    index: 0,
                });
            }
            return;
        }
    }

    let is_first_run = editor_data.cache.recent_projects.is_empty();

    if is_first_run && let Some((demo_path, result)) = load_bundled_demo(&examples_dir()) {
        let has_emitters = !result.asset.emitters.is_empty();
        let handle = assets.add(result.asset);
        let demo_file = simplify_path(&demo_path);
        editor_state.open_project(
            handle,
            demo_path,
            &mut dirty_state,
            result.was_migrated,
            &assets,
        );
        if has_emitters {
            editor_state.inspecting = Some(Inspecting {
                kind: Inspectable::Emitter,
                index: 0,
            });
        }

        editor_data.cache.add_recent_project(demo_file);
        save_editor_data(&editor_data);
        return;
    }

    let asset = ParticlesAsset::new(
        "New project".to_string(),
        ParticlesDimension::D3,
        Default::default(),
        vec![EmitterData {
            name: "Emitter 1".to_string(),
            ..Default::default()
        }],
        vec![],
        false,
        Default::default(),
    );
    let handle = assets.add(asset);
    editor_state.current_project = Some(handle);
    dirty_state.has_unsaved_changes = true;
    editor_state.inspecting = Some(Inspecting {
        kind: Inspectable::Emitter,
        index: 0,
    });
}

fn load_bundled_demo(
    root: &Path,
) -> Option<(PathBuf, bevy_sprinkles::asset::versions::MigrationResult)> {
    let path = root.join("3d-explosion.ron");
    if !path.is_file() {
        return None;
    }
    load_project_from_path(&path)
        .ok()
        .map(|result| (path, result))
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn first_run_demo_loads_from_configured_examples_directory() {
        let dir = std::env::temp_dir().join(format!(
            "theia-particle-demo-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        assert!(load_bundled_demo(&dir).is_none());
        let asset = ParticlesAsset::new(
            "Bundled test".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let path = dir.join("3d-explosion.ron");
        let source = ron::to_string(&asset).unwrap();
        std::fs::write(&path, &source).unwrap();
        let (loaded_path, result) = load_bundled_demo(&dir).unwrap();
        assert_eq!(loaded_path, path);
        assert_eq!(result.asset.name, "Bundled test");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cached_migrated_project_stays_dirty_until_saved_without_rewriting_source() {
        let dir = std::env::temp_dir().join(format!(
            "theia-particle-migration-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("old.ron");
        let asset = ParticlesAsset::new(
            "Migrated".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let current = ron::ser::to_string(&asset).unwrap();
        assert!(current.contains("sprinkles_version:\"0.3\""));
        let original = current.replace("sprinkles_version:\"0.3\"", "sprinkles_version:\"0.2\"");
        std::fs::write(&path, &original).unwrap();
        let mut data = EditorData::default();
        data.cache.last_opened_project = Some(path.to_string_lossy().into_owned());
        let mut app = App::new();
        app.init_resource::<CliArgs>()
            .init_resource::<EditorState>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .insert_resource(data);
        app.world_mut()
            .run_system_once(load_initial_project)
            .unwrap();
        let state = app.world().resource::<EditorState>();
        assert_eq!(state.current_project_path.as_deref(), Some(path.as_path()));
        assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
        assert!(
            ron::ser::to_string(
                app.world()
                    .resource::<Assets<ParticlesAsset>>()
                    .get(state.current_project.as_ref().unwrap())
                    .unwrap()
            )
            .unwrap()
            .contains("sprinkles_version:\"0.3\"")
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
