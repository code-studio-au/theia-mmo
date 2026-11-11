use crate::hosting::AuthoringAppExt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_sprinkles::prelude::*;
use inflector::Inflector;

use crate::io::{EditorData, project_path, projects_dir, save_editor_data};
use crate::state::{DirtyState, EditorState};
#[cfg(test)]
use crate::state::{Inspectable, Inspecting};
use crate::ui::components::toasts::ToastEvent;
use crate::ui::widgets::text_edit::{self, SaveTextInputs, TextEditCommitEvent};
use crate::utils::{MAX_DISPLAY_PATH_LEN, simplify_path, truncate_path};

pub fn plugin(app: &mut App) {
    app.init_resource::<crate::drafts::RecoveryStore>();
    let draft_store = app
        .world()
        .resource::<crate::drafts::RecoveryStore>()
        .0
        .clone();
    app.insert_non_send(crate::draft_capture::DraftCapture::new(draft_store));
    app.add_observer(crate::drafts::restore_draft)
        .init_resource::<SaveCoordinator>()
        .init_resource::<TextFocusTracker>()
        .add_observer(on_open_project_event)
        .add_observer(on_browse_open_project_event)
        .add_observer(on_save_project_event)
        .add_observer(on_save_project_as_event)
        .add_observer(on_save_project_to_library_event)
        .add_observer(continue_save_after_text_commit)
        .add_plugins(crate::project_switch::ProjectSwitchPlugin)
        .add_authoring_systems((handle_save_keyboard_shortcut, poll_browse_open_result))
        .add_inline_ui_systems(track_text_focus.before(text_edit::handle_unfocus))
        // A write may complete while the integrated host is on the Zone workspace.
        .add_systems(Update, (poll_save, crate::draft_capture::capture_drafts))
        .add_systems(Last, crate::draft_capture::capture_on_exit);
}

#[derive(Event)]
pub struct OpenProjectEvent(pub String);

#[derive(Event)]
pub struct BrowseOpenProjectEvent;

#[derive(Event)]
pub struct SaveProjectEvent;

#[derive(Event)]
pub struct SaveProjectAsEvent;

/// Explicitly save the open project under a stable local effect ID.
#[derive(Event)]
pub struct SaveProjectToLibraryEvent(pub String);

#[derive(Resource, Clone)]
pub struct BrowseOpenResult(pub Arc<Mutex<Option<PathBuf>>>);

#[derive(Resource, Default)]
pub(crate) struct SaveCoordinator {
    pending: Option<PendingSave>,
    committing: bool,
}

struct PendingSave {
    handle: Handle<ParticlesAsset>,
    snapshot: String,
    phase: SavePhase,
}

type DialogResult = Arc<Mutex<Option<Option<PathBuf>>>>;
type WriteResult = Arc<Mutex<Option<Result<(), String>>>>;

enum SavePhase {
    Choosing(DialogResult),
    Writing {
        path: PathBuf,
        assign_path: bool,
        result: WriteResult,
    },
}

impl SaveCoordinator {
    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some() || self.committing
    }

    pub(crate) fn save_to_path(
        &mut self,
        handle: Handle<ParticlesAsset>,
        asset: &ParticlesAsset,
        path: PathBuf,
        assign_path: bool,
        create_only: bool,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("A particle project save is already in progress".into());
        }
        let snapshot = serialize_snapshot(asset)?;
        let result = start_write(path.clone(), snapshot.clone(), create_only);
        self.pending = Some(PendingSave {
            handle,
            snapshot,
            phase: SavePhase::Writing {
                path,
                assign_path,
                result,
            },
        });
        Ok(())
    }

    fn choose_path(
        &mut self,
        handle: Handle<ParticlesAsset>,
        asset: &ParticlesAsset,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("A particle project save is already in progress".into());
        }
        let snapshot = serialize_snapshot(asset)?;
        let result: DialogResult = Arc::new(Mutex::new(None));
        let completed = result.clone();
        let task = rfd::AsyncFileDialog::new()
            .set_title("Save Project As")
            .set_directory(projects_dir())
            .set_file_name(format!("{}.ron", asset.name.to_kebab_case()))
            .add_filter("RON files", &["ron"])
            .save_file();
        IoTaskPool::get()
            .spawn(async move {
                let selected = task.await.map(|file| file.path().to_path_buf());
                if let Ok(mut guard) = completed.lock() {
                    *guard = Some(selected);
                }
            })
            .detach();
        self.pending = Some(PendingSave {
            handle,
            snapshot,
            phase: SavePhase::Choosing(result),
        });
        Ok(())
    }
}

pub(crate) fn serialize_snapshot(asset: &ParticlesAsset) -> Result<String, String> {
    ron::ser::to_string_pretty(asset, ron::ser::PrettyConfig::default())
        .map_err(|_| "Cannot save project with invalid data".into())
}

fn start_write(path: PathBuf, snapshot: String, create_only: bool) -> WriteResult {
    let result: WriteResult = Arc::new(Mutex::new(None));
    let completed = result.clone();
    IoTaskPool::get()
        .spawn(async move {
            let status = atomic_write(&path, snapshot.as_bytes(), create_only)
                .map_err(|error| error.to_string());
            if let Ok(mut guard) = completed.lock() {
                *guard = Some(status);
            }
        })
        .detach();
    result
}

// Write a new sibling, flush it, and replace only after the complete write.
// Resolve existing symlinks so saving through a link does not replace the link.
pub(crate) fn atomic_write(path: &Path, contents: &[u8], create_only: bool) -> std::io::Result<()> {
    let target = match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => path.canonicalize()?,
        Ok(_) => path.to_path_buf(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => path.to_path_buf(),
        Err(error) => return Err(error),
    };
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let permissions = match std::fs::metadata(&target) {
        Ok(_) if create_only => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Project file already exists",
            ));
        }
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Save target is not a regular file",
                ));
            }
            if metadata.permissions().readonly() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Project file is read-only",
                ));
            }
            // Check access without truncating the existing project.
            drop(OpenOptions::new().write(true).open(&target)?);
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);
    for _ in 0..32 {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".theia-particle-save-{}-{nonce}.tmp",
            std::process::id()
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let result = (|| {
            if let Some(permissions) = permissions.clone() {
                file.set_permissions(permissions)?;
            }
            file.write_all(contents)?;
            file.sync_all()?;
            drop(file);
            if create_only {
                // Linking is atomic and fails if another process created the target.
                std::fs::hard_link(&temporary, &target)?;
                std::fs::remove_file(&temporary)
            } else {
                std::fs::rename(&temporary, &target)
            }
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        return result;
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "Could not allocate a temporary save file",
    ))
}

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

    parse_project(&contents)
}

pub(crate) fn parse_project(
    contents: &str,
) -> Result<bevy_sprinkles::asset::versions::MigrationResult, LoadProjectError> {
    let result = bevy_sprinkles::asset::versions::migrate_str(contents).map_err(|err| {
        error!("Failed to parse particle project data");
        error!("{err}");
        match err {
            bevy_sprinkles::asset::versions::MigrationError::UnknownVersion(_) => {
                LoadProjectError::UnknownVersion
            }
            _ => LoadProjectError::Parse,
        }
    })?;
    if result.asset.unsupported_material_index().is_some() {
        return Err(LoadProjectError::UnsupportedMaterial);
    }
    Ok(result)
}

fn on_open_project_event(event: On<OpenProjectEvent>, mut commands: Commands) {
    let location = &event.0;
    let path = project_path(location);

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

    commands.trigger(crate::project_switch::RequestProjectSwitch::open(
        result.asset,
        path,
        location.clone(),
        result.was_migrated,
    ));
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

#[derive(Resource, Default)]
pub(crate) struct TextFocusTracker(pub(crate) Option<Entity>);

fn track_text_focus(
    focus: Res<InputFocus>,
    inputs: SaveTextInputs,
    mut tracker: ResMut<TextFocusTracker>,
) {
    tracker.0 = focus.get().filter(|entity| inputs.contains(*entity));
}

#[derive(Clone)]
enum SaveKind {
    Current,
    As,
    Library(PathBuf),
}
#[derive(Event)]
struct ContinueSave {
    kind: SaveKind,
    handle: Handle<ParticlesAsset>,
    path: Option<PathBuf>,
}

fn begin_save(
    kind: SaveKind,
    state: &EditorState,
    assets: &Assets<ParticlesAsset>,
    saves: &mut SaveCoordinator,
) -> Result<(), String> {
    let Some(handle) = &state.current_project else {
        return Ok(());
    };
    let Some(asset) = assets.get(handle) else {
        return Ok(());
    };
    match kind {
        SaveKind::Current => match &state.current_project_path {
            Some(path) => saves.save_to_path(handle.clone(), asset, path.clone(), false, false),
            None => saves.choose_path(handle.clone(), asset),
        },
        SaveKind::As => saves.choose_path(handle.clone(), asset),
        SaveKind::Library(path) => {
            if std::fs::symlink_metadata(&path)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err("Effect library target cannot be a symbolic link".into());
            }
            let parent = path.parent().ok_or("Invalid effect library path")?;
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Cannot create effect library: {error}"))?;
            let create_only = state.current_project_path.as_ref() != Some(&path);
            saves.save_to_path(handle.clone(), asset, path, true, create_only)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn request_save(
    kind: SaveKind,
    state: &EditorState,
    assets: &Assets<ParticlesAsset>,
    saves: &mut SaveCoordinator,
    focus: &mut InputFocus,
    tracker: &mut TextFocusTracker,
    inputs: &SaveTextInputs,
    commands: &mut Commands,
) {
    if saves.busy() {
        commands.trigger(ToastEvent::error(
            "A particle project save is already in progress",
        ));
        return;
    }
    let Some(handle) = state.current_project.as_ref() else {
        return;
    };
    let focused = focus
        .get()
        .filter(|entity| inputs.contains(*entity))
        .or(tracker.0);
    if let Some(entity) = focused {
        let text = match inputs.text_for_save(entity) {
            Ok(text) => text,
            Err(error) => {
                if inputs.contains(entity) {
                    focus.set(entity, FocusCause::Navigated);
                }
                commands.trigger(ToastEvent::error(error));
                return;
            }
        };
        saves.committing = true;
        tracker.0 = None;
        if focus.get() == Some(entity) {
            focus.clear();
        }
        // Bevy dispatches the field binding before continuing the save. Reserve
        // the coordinator meanwhile so a second request cannot save old data.
        commands.trigger(TextEditCommitEvent { entity, text });
        commands.trigger(ContinueSave {
            kind,
            handle: handle.clone(),
            path: state.current_project_path.clone(),
        });
    } else if let Err(error) = begin_save(kind, state, assets, saves) {
        commands.trigger(ToastEvent::error(error));
    }
}

fn on_save_project_event(
    _event: On<SaveProjectEvent>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut saves: ResMut<SaveCoordinator>,
    mut focus: ResMut<InputFocus>,
    mut tracker: ResMut<TextFocusTracker>,
    inputs: SaveTextInputs,
    mut commands: Commands,
) {
    request_save(
        SaveKind::Current,
        &editor_state,
        &assets,
        &mut saves,
        &mut focus,
        &mut tracker,
        &inputs,
        &mut commands,
    );
}
fn on_save_project_as_event(
    _event: On<SaveProjectAsEvent>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut saves: ResMut<SaveCoordinator>,
    mut focus: ResMut<InputFocus>,
    mut tracker: ResMut<TextFocusTracker>,
    inputs: SaveTextInputs,
    mut commands: Commands,
) {
    request_save(
        SaveKind::As,
        &editor_state,
        &assets,
        &mut saves,
        &mut focus,
        &mut tracker,
        &inputs,
        &mut commands,
    );
}

fn on_save_project_to_library_event(
    event: On<SaveProjectToLibraryEvent>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut saves: ResMut<SaveCoordinator>,
    mut focus: ResMut<InputFocus>,
    mut tracker: ResMut<TextFocusTracker>,
    inputs: SaveTextInputs,
    mut commands: Commands,
) {
    let Some(path) = crate::io::effect_library_path(&event.0) else {
        commands.trigger(ToastEvent::error(
            "Set a valid effect ID before saving to the library",
        ));
        return;
    };
    request_save(
        SaveKind::Library(path),
        &editor_state,
        &assets,
        &mut saves,
        &mut focus,
        &mut tracker,
        &inputs,
        &mut commands,
    );
}
fn continue_save_after_text_commit(
    event: On<ContinueSave>,
    editor_state: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    mut saves: ResMut<SaveCoordinator>,
    mut commands: Commands,
) {
    if !saves.committing {
        return;
    }
    saves.committing = false;
    if editor_state.current_project.as_ref() != Some(&event.handle)
        || (matches!(event.kind, SaveKind::Current)
            && editor_state.current_project_path != event.path)
    {
        commands.trigger(ToastEvent::error("The project changed; save again"));
        return;
    }
    if let Err(error) = begin_save(event.kind.clone(), &editor_state, &assets, &mut saves) {
        commands.trigger(ToastEvent::error(error));
    }
}

// Only the exact in-memory asset that completed a successful write can become clean.
// A different active document, a changed path, or intervening edits stay untouched.
fn finish_write(
    pending: &PendingSave,
    path: &Path,
    assign_path: bool,
    success: bool,
    editor_state: &mut EditorState,
    dirty_state: &mut DirtyState,
    assets: &Assets<ParticlesAsset>,
) -> bool {
    if editor_state.current_project.as_ref() != Some(&pending.handle)
        || (!assign_path && editor_state.current_project_path.as_deref() != Some(path))
    {
        return false;
    }
    if !success {
        dirty_state.has_unsaved_changes = true;
        return false;
    }
    if assign_path {
        editor_state.current_project_path = Some(path.to_path_buf());
    }
    dirty_state.has_unsaved_changes = assets
        .get(&pending.handle)
        .and_then(|asset| serialize_snapshot(asset).ok())
        .is_none_or(|current| current != pending.snapshot);
    assign_path
}

fn poll_save(
    mut saves: ResMut<SaveCoordinator>,
    mut editor_state: ResMut<EditorState>,
    mut dirty_state: ResMut<DirtyState>,
    mut editor_data: ResMut<EditorData>,
    assets: Res<Assets<ParticlesAsset>>,
    mut commands: Commands,
) {
    let Some(mut pending) = saves.pending.take() else {
        return;
    };
    match &pending.phase {
        SavePhase::Choosing(result) => {
            let selected = result.lock().map(|mut guard| guard.take()).map_err(|_| ());
            match selected {
                Ok(None) => saves.pending = Some(pending),
                Ok(Some(None)) => {}
                Ok(Some(Some(path))) => {
                    let result = start_write(path.clone(), pending.snapshot.clone(), false);
                    pending.phase = SavePhase::Writing {
                        path,
                        assign_path: true,
                        result,
                    };
                    saves.pending = Some(pending);
                }
                Err(_) => {
                    commands.trigger(ToastEvent::error("Save dialog failed"));
                }
            }
        }
        SavePhase::Writing {
            path,
            assign_path,
            result,
        } => {
            let outcome = result.lock().map(|mut guard| guard.take()).map_err(|_| ());
            match outcome {
                Ok(None) => saves.pending = Some(pending),
                completed => {
                    let status = completed
                        .unwrap_or_else(|_| Some(Err("Save worker failed".into())))
                        .expect("A completed save has a result");
                    let cache_path = finish_write(
                        &pending,
                        path,
                        *assign_path,
                        status.is_ok(),
                        &mut editor_state,
                        &mut dirty_state,
                        &assets,
                    );
                    if cache_path {
                        editor_data.cache.add_recent_project(simplify_path(path));
                        save_editor_data(&editor_data);
                    }
                    match status {
                        Ok(()) => {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy())
                                .unwrap_or_default();
                            commands.trigger(ToastEvent::success(format!("Saved \"{name}\"")));
                        }
                        Err(error) => {
                            commands.trigger(ToastEvent::error(format!(
                                "Could not save project: {error}"
                            )));
                        }
                    }
                }
            }
        }
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

#[cfg(test)]
#[path = "project_save_tests.rs"]
mod save_tests;
