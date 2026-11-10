use crate::hosting::AuthoringAppExt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_sprinkles::prelude::*;
use inflector::Inflector;

use crate::io::{EditorData, is_example_path, project_path, projects_dir, save_editor_data};
use crate::state::{DirtyState, EditorState, Inspectable, Inspecting};
use crate::ui::components::toasts::ToastEvent;
use crate::utils::{MAX_DISPLAY_PATH_LEN, simplify_path, truncate_path};

pub fn plugin(app: &mut App) {
    app.add_observer(on_open_project_event)
        .add_observer(on_browse_open_project_event)
        .add_observer(on_save_project_event)
        .add_observer(on_save_project_as_event)
        .add_authoring_systems((
            handle_save_keyboard_shortcut,
            poll_browse_open_result,
            poll_save_as_result,
            poll_save_result,
        ));
}

#[derive(Event)]
pub struct OpenProjectEvent(pub String);

#[derive(Event)]
pub struct BrowseOpenProjectEvent;

#[derive(Event)]
pub struct SaveProjectEvent;

#[derive(Event)]
pub struct SaveProjectAsEvent;

#[derive(Resource, Clone)]
pub struct BrowseOpenResult(pub Arc<Mutex<Option<PathBuf>>>);

#[derive(Resource, Clone)]
pub struct SaveAsResult(pub Arc<Mutex<Option<PathBuf>>>);

#[derive(Clone)]
pub enum SaveResultStatus {
    Success(String),
    SerializationError,
    WriteError(String),
    CreateError,
}

#[derive(Resource, Clone)]
pub struct SaveResult(pub Arc<Mutex<Option<SaveResultStatus>>>);

#[derive(Debug)]
pub(crate) enum LoadProjectError {
    /// The file could not be read from disk.
    Read(std::io::Error),
    /// The file contents could not be parsed as a valid project.
    Parse,
    /// The file has an unrecognized format version.
    UnknownVersion,
    /// This project requires a material the runtime cannot render.
    UnsupportedMaterial,
}

pub(crate) fn load_project_from_path(
    path: &std::path::Path,
) -> Result<bevy_sprinkles::asset::versions::MigrationResult, LoadProjectError> {
    let contents = std::fs::read_to_string(path).map_err(|err| {
        error!("Failed to read project file: {path:?}");
        error!("{err}");
        LoadProjectError::Read(err)
    })?;

    let result = bevy_sprinkles::asset::versions::migrate_str(&contents).map_err(|err| {
        error!("Failed to parse project file: {path:?}");
        error!("{err}");
        match err {
            bevy_sprinkles::asset::versions::MigrationError::UnknownVersion(_) => {
                LoadProjectError::UnknownVersion
            }
            _ => LoadProjectError::Parse,
        }
    })?;
    if result.asset.emitters.iter().any(|emitter| {
        matches!(
            emitter.draw_pass.material,
            DrawPassMaterial::CustomShader { .. }
        )
    }) {
        return Err(LoadProjectError::UnsupportedMaterial);
    }
    Ok(result)
}

fn on_open_project_event(
    event: On<OpenProjectEvent>,
    mut editor_state: ResMut<EditorState>,
    mut editor_data: ResMut<EditorData>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
    mut dirty_state: ResMut<DirtyState>,
    mut commands: Commands,
) {
    let location = &event.0;
    let path = project_path(location);
    let is_example = is_example_path(&path);

    let result = match load_project_from_path(&path) {
        Ok(result) => result,
        Err(err) => {
            let display = truncate_path(location, MAX_DISPLAY_PATH_LEN);
            let message = match &err {
                LoadProjectError::Read(io_err) => match io_err.kind() {
                    std::io::ErrorKind::NotFound => {
                        format!("Project file not found: \"{display}\"")
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        format!("Permission denied: \"{display}\"")
                    }
                    _ => format!("Could not read project file: \"{display}\""),
                },
                LoadProjectError::Parse => {
                    format!("Project \"{display}\" is corrupted or invalid")
                }
                LoadProjectError::UnsupportedMaterial => {
                    format!(
                        "Project \"{display}\" uses custom shader materials, which are not supported yet"
                    )
                }
                LoadProjectError::UnknownVersion => {
                    format!(
                        "Project \"{display}\" has an unknown version. You may need to update Sprinkles."
                    )
                }
            };
            commands.trigger(ToastEvent::error(message));
            return;
        }
    };

    let asset = result.asset;

    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| location.clone());

    if result.was_migrated {
        let current = bevy_sprinkles::asset::versions::current_format_version();
        dirty_state.has_unsaved_changes = true;
        commands.trigger(ToastEvent::success(format!(
            "Project \"{filename}\" will be updated to {current} on the next save"
        )));
    }

    let has_emitters = !asset.emitters.is_empty();
    let handle = assets.add(asset);
    editor_state.open_project(handle, path, &mut dirty_state);
    editor_state.inspecting = if has_emitters {
        Some(Inspecting {
            kind: Inspectable::Emitter,
            index: 0,
        })
    } else {
        None
    };

    if !is_example {
        editor_data.cache.add_recent_project(location.clone());
        save_editor_data(&editor_data);
    }
}

fn on_browse_open_project_event(_event: On<BrowseOpenProjectEvent>, mut commands: Commands) {
    let projects_dir = projects_dir();

    let path_result = Arc::new(Mutex::new(None));
    let path_result_clone = path_result.clone();

    let task = rfd::AsyncFileDialog::new()
        .set_title("Open Project")
        .set_directory(&projects_dir)
        .add_filter("RON files", &["ron"])
        .pick_file();

    IoTaskPool::get()
        .spawn(async move {
            if let Some(file_handle) = task.await {
                let path = file_handle.path().to_path_buf();
                if let Ok(mut guard) = path_result_clone.lock() {
                    *guard = Some(path);
                }
            }
        })
        .detach();

    commands.insert_resource(BrowseOpenResult(path_result));
}

fn poll_browse_open_result(result: Option<Res<BrowseOpenResult>>, mut commands: Commands) {
    let Some(result) = result else {
        return;
    };

    let path = {
        let Ok(mut guard) = result.0.lock() else {
            return;
        };
        guard.take()
    };

    if let Some(path) = path {
        commands.trigger(OpenProjectEvent(simplify_path(&path)));
        commands.remove_resource::<BrowseOpenResult>();
    }
}

pub fn save_project_to_path(
    path: PathBuf,
    asset: &bevy_sprinkles::asset::ParticlesAsset,
    result: Arc<Mutex<Option<SaveResultStatus>>>,
) {
    let Ok(contents) = ron::ser::to_string_pretty(asset, ron::ser::PrettyConfig::default()) else {
        if let Ok(mut guard) = result.lock() {
            *guard = Some(SaveResultStatus::SerializationError);
        }
        return;
    };

    IoTaskPool::get()
        .spawn(async move {
            let status = match File::create(&path) {
                Ok(mut file) => {
                    let filename = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if file.write_all(contents.as_bytes()).is_ok() {
                        SaveResultStatus::Success(filename)
                    } else {
                        SaveResultStatus::WriteError(filename)
                    }
                }
                Err(_) => SaveResultStatus::CreateError,
            };
            if let Ok(mut guard) = result.lock() {
                *guard = Some(status);
            }
        })
        .detach();
}

fn on_save_project_event(
    _event: On<SaveProjectEvent>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut dirty_state: ResMut<DirtyState>,
    mut commands: Commands,
) {
    let Some(handle) = &editor_state.current_project else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };

    if let Some(path) = &editor_state.current_project_path {
        let result = Arc::new(Mutex::new(None));
        save_project_to_path(path.clone(), asset, result.clone());
        commands.insert_resource(SaveResult(result));
        dirty_state.has_unsaved_changes = false;
    } else {
        commands.trigger(SaveProjectAsEvent);
    }
}

fn on_save_project_as_event(
    _event: On<SaveProjectAsEvent>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut commands: Commands,
) {
    let Some(handle) = &editor_state.current_project else {
        return;
    };
    let Some(asset) = assets.get(handle) else {
        return;
    };

    let projects_dir = projects_dir();
    let default_name = format!("{}.ron", asset.name.to_kebab_case());
    let asset_clone = asset.clone();

    let path_result = Arc::new(Mutex::new(None));
    let path_result_clone = path_result.clone();

    let save_result = Arc::new(Mutex::new(None));
    let save_result_clone = save_result.clone();

    let task = rfd::AsyncFileDialog::new()
        .set_title("Save Project As")
        .set_directory(&projects_dir)
        .set_file_name(&default_name)
        .add_filter("RON files", &["ron"])
        .save_file();

    IoTaskPool::get()
        .spawn(async move {
            if let Some(file_handle) = task.await {
                let path = file_handle.path().to_path_buf();
                save_project_to_path(path.clone(), &asset_clone, save_result_clone);
                if let Ok(mut guard) = path_result_clone.lock() {
                    *guard = Some(path);
                }
            }
        })
        .detach();

    commands.insert_resource(SaveAsResult(path_result));
    commands.insert_resource(SaveResult(save_result));
}

fn poll_save_as_result(
    result: Option<Res<SaveAsResult>>,
    mut editor_state: ResMut<EditorState>,
    mut editor_data: ResMut<EditorData>,
    mut dirty_state: ResMut<DirtyState>,
    mut commands: Commands,
) {
    let Some(result) = result else {
        return;
    };

    let path = {
        let Ok(mut guard) = result.0.lock() else {
            return;
        };
        guard.take()
    };

    if let Some(path) = path {
        editor_state.current_project_path = Some(path.clone());

        editor_data.cache.add_recent_project(simplify_path(&path));
        save_editor_data(&editor_data);
        dirty_state.has_unsaved_changes = false;
        commands.remove_resource::<SaveAsResult>();
    }
}

fn poll_save_result(result: Option<Res<SaveResult>>, mut commands: Commands) {
    let Some(result) = result else {
        return;
    };

    let status = {
        let Ok(mut guard) = result.0.lock() else {
            return;
        };
        guard.take()
    };

    if let Some(status) = status {
        match status {
            SaveResultStatus::Success(filename) => {
                commands.trigger(ToastEvent::success(format!("Saved \"{filename}\"")));
            }
            SaveResultStatus::SerializationError => {
                commands.trigger(ToastEvent::error("Cannot save project with invalid data"));
            }
            SaveResultStatus::WriteError(filename) => {
                commands.trigger(ToastEvent::error(format!(
                    "Failed to write to \"{filename}\""
                )));
            }
            SaveResultStatus::CreateError => {
                commands.trigger(ToastEvent::error("Failed to create project file"));
            }
        }
        commands.remove_resource::<SaveResult>();
    }
}

fn handle_save_keyboard_shortcut(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    let ctrl_or_cmd = keyboard.pressed(KeyCode::SuperLeft)
        || keyboard.pressed(KeyCode::SuperRight)
        || keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);

    if ctrl_or_cmd && keyboard.just_pressed(KeyCode::KeyS) {
        commands.trigger(SaveProjectEvent);
    }
}

#[cfg(test)]
mod hosting_tests {
    use super::*;
    use crate::hosting::{AuthoringHost, AuthoringHostPlugin};

    #[derive(Resource, Default)]
    struct Saves(usize);

    #[test]
    fn inactive_authoring_cannot_steal_the_hosts_save_shortcut() {
        let mut app = App::new();
        app.add_plugins(AuthoringHostPlugin)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Saves>()
            .add_authoring_systems(handle_save_keyboard_shortcut)
            .add_observer(|_: On<SaveProjectEvent>, mut saves: ResMut<Saves>| saves.0 += 1);
        for (active, expected) in [(true, 1), (false, 1), (false, 1), (true, 2)] {
            app.world_mut().resource_mut::<AuthoringHost>().active = active;
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(KeyCode::ControlLeft);
            keys.press(KeyCode::KeyS);
            app.update();
            assert_eq!(app.world().resource::<Saves>().0, expected);
        }
    }
}

#[cfg(test)]
mod material_load_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Resource, Default)]
    struct Errors(Vec<String>);

    #[test]
    fn unsupported_material_load_preserves_current_project_dirty_state_and_files() {
        let folder = std::env::temp_dir().join(format!(
            "theia-sprinkles-load-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("unsupported.ron");
        let mut asset = ParticlesAsset::new(
            "Unsupported".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        // Verify the regular startup/cache loader still accepts supported effects.
        std::fs::write(&path, ron::to_string(&asset).unwrap()).unwrap();
        assert!(load_project_from_path(&path).is_ok());
        asset.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
            vertex_shader: None,
            fragment_shader: None,
        };
        let bytes = ron::to_string(&asset).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            load_project_from_path(&path),
            Err(LoadProjectError::UnsupportedMaterial)
        ));

        let mut app = App::new();
        app.init_resource::<EditorState>()
            .init_resource::<EditorData>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<Errors>()
            .add_observer(on_open_project_event)
            .add_observer(|event: On<ToastEvent>, mut errors: ResMut<Errors>| {
                errors.0.push(event.content.clone())
            });
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(ParticlesAsset::new(
                "Unsaved work".into(),
                ParticlesDimension::D3,
                default(),
                vec![EmitterData::default()],
                vec![],
                false,
                default(),
            ));
        let original = ron::to_string(
            app.world()
                .resource::<Assets<ParticlesAsset>>()
                .get(&handle)
                .unwrap(),
        )
        .unwrap();
        {
            let mut state = app.world_mut().resource_mut::<EditorState>();
            state.current_project = Some(handle.clone());
            state.current_project_path = Some(folder.join("existing.ron"));
            state.inspecting = Some(Inspecting {
                kind: Inspectable::Emitter,
                index: 0,
            });
        }
        for dirty in [false, true] {
            app.world_mut()
                .resource_mut::<DirtyState>()
                .has_unsaved_changes = dirty;
            app.world_mut()
                .trigger(OpenProjectEvent(path.to_string_lossy().into_owned()));
            app.update();
            let state = app.world().resource::<EditorState>();
            assert_eq!(state.current_project.as_ref().unwrap().id(), handle.id());
            assert_eq!(
                state.current_project_path.as_ref().unwrap(),
                &folder.join("existing.ron")
            );
            assert_eq!(state.inspecting.as_ref().unwrap().index, 0);
            assert_eq!(
                app.world().resource::<DirtyState>().has_unsaved_changes,
                dirty
            );
            let assets = app.world().resource::<Assets<ParticlesAsset>>();
            assert_eq!(assets.len(), 1);
            assert_eq!(
                ron::to_string(assets.get(&handle).unwrap()).unwrap(),
                original
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
            assert!(!folder.join("existing.ron").exists());
            assert!(
                app.world()
                    .resource::<Errors>()
                    .0
                    .last()
                    .unwrap()
                    .contains("not supported")
            );
            assert!(
                app.world()
                    .resource::<EditorData>()
                    .cache
                    .recent_projects
                    .is_empty()
            );
        }
        std::fs::remove_dir_all(folder).unwrap();
    }
}
