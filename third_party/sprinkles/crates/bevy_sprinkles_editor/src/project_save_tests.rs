//! Real temporary-file writes and Bevy save events; no dialogs or operating-system input.
use super::*;
use crate::hosting::{AuthoringHost, AuthoringHostPlugin};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn folder() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "theia-particle-save-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    path
}
fn asset(name: &str) -> ParticlesAsset {
    ParticlesAsset::new(
        name.into(),
        ParticlesDimension::D3,
        default(),
        vec![EmitterData::default()],
        vec![],
        false,
        default(),
    )
}
#[derive(Resource, Default)]
struct Messages(Vec<String>);
fn fixture(path: PathBuf) -> (App, Handle<ParticlesAsset>) {
    let mut app = App::new();
    app.add_plugins((bevy::app::TaskPoolPlugin::default(), AuthoringHostPlugin))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InputFocus>()
        .init_resource::<EditorState>()
        .init_resource::<DirtyState>()
        .init_resource::<EditorData>()
        .init_resource::<Assets<ParticlesAsset>>()
        .init_resource::<Messages>()
        .add_plugins(plugin)
        .add_observer(|event: On<ToastEvent>, mut messages: ResMut<Messages>| {
            messages.0.push(event.content.clone())
        });
    let handle = app
        .world_mut()
        .resource_mut::<Assets<ParticlesAsset>>()
        .add(asset("Before save"));
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.current_project = Some(handle.clone());
        state.current_project_path = Some(path);
    }
    app.world_mut()
        .resource_mut::<DirtyState>()
        .has_unsaved_changes = true;
    (app, handle)
}
fn wait(app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.world().resource::<SaveCoordinator>().busy() {
        assert!(Instant::now() < deadline, "Save did not finish");
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn save_waits_for_success_preserves_intervening_edits_and_serializes_concurrent_requests() {
    let dir = folder();
    let path = dir.join("effect.ron");
    std::fs::write(&path, b"old project remains until replacement").unwrap();
    let (mut app, handle) = fixture(path.clone());
    app.world_mut().trigger(SaveProjectEvent);
    assert!(app.world().resource::<SaveCoordinator>().busy());
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    app.world_mut()
        .resource_mut::<Assets<ParticlesAsset>>()
        .get_mut(&handle)
        .unwrap()
        .name = "Edited during save".into();
    app.world_mut().trigger(SaveProjectEvent);

    // Completion must still be processed if the user returns to the Zone workspace.
    app.world_mut().resource_mut::<AuthoringHost>().active = false;
    wait(&mut app);
    assert!(
        app.world()
            .resource::<Messages>()
            .0
            .iter()
            .any(|message| message.contains("already in progress"))
    );
    let written = load_project_from_path(&path).unwrap().asset;
    assert_eq!(written.name, "Before save");
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    assert_eq!(
        app.world()
            .resource::<EditorState>()
            .current_project_path
            .as_deref(),
        Some(path.as_path())
    );
    app.world_mut().resource_mut::<AuthoringHost>().active = true;
    app.world_mut().trigger(SaveProjectEvent);
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    wait(&mut app);
    assert_eq!(
        load_project_from_path(&path).unwrap().asset.name,
        "Edited during save"
    );
    assert!(!app.world().resource::<DirtyState>().has_unsaved_changes);
    assert!(std::fs::read_dir(&dir).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_write_preserves_existing_target_and_dirty_state() {
    let dir = folder();
    let target = dir.join("occupied.ron");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("sentinel"), b"keep").unwrap();
    let (mut app, _) = fixture(target.clone());
    app.world_mut().trigger(SaveProjectEvent);
    wait(&mut app);
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    assert_eq!(std::fs::read(target.join("sentinel")).unwrap(), b"keep");
    assert!(
        app.world()
            .resource::<Messages>()
            .0
            .iter()
            .any(|message| message.contains("Could not save"))
    );
    assert!(std::fs::read_dir(&dir).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_completion_cannot_mark_a_replacement_project_clean() {
    let dir = folder();
    let path = dir.join("old.ron");
    let (mut app, old) = fixture(path.clone());
    app.world_mut().trigger(SaveProjectEvent);
    let new = app
        .world_mut()
        .resource_mut::<Assets<ParticlesAsset>>()
        .add(asset("Other project"));
    {
        let mut state = app.world_mut().resource_mut::<EditorState>();
        state.current_project = Some(new.clone());
        state.current_project_path = None;
    }
    app.world_mut()
        .resource_mut::<DirtyState>()
        .has_unsaved_changes = true;
    wait(&mut app);
    let state = app.world().resource::<EditorState>();
    assert_eq!(state.current_project.as_ref(), Some(&new));
    assert!(state.current_project_path.is_none());
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    assert_eq!(
        load_project_from_path(&path).unwrap().asset.name,
        "Before save"
    );
    assert_eq!(
        app.world()
            .resource::<Assets<ParticlesAsset>>()
            .get(&old)
            .unwrap()
            .name,
        "Before save"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_as_assigns_path_only_on_success_and_keeps_later_edits_dirty() {
    let mut assets = Assets::<ParticlesAsset>::default();
    let handle = assets.add(asset("Before save"));
    let snapshot = serialize_snapshot(assets.get(&handle).unwrap()).unwrap();
    let pending = PendingSave {
        handle: handle.clone(),
        snapshot,
        phase: SavePhase::Choosing(Arc::new(Mutex::new(None))),
    };
    let mut state = EditorState {
        current_project: Some(handle.clone()),
        ..default()
    };
    let mut dirty = DirtyState {
        has_unsaved_changes: true,
    };
    let path = Path::new("draft.ron");
    assert!(!finish_write(
        &pending, path, true, false, &mut state, &mut dirty, &assets
    ));
    assert!(state.current_project_path.is_none() && dirty.has_unsaved_changes);
    assets.get_mut(&handle).unwrap().name = "Edited after request".into();
    assert!(finish_write(
        &pending, path, true, true, &mut state, &mut dirty, &assets
    ));
    assert_eq!(state.current_project_path.as_deref(), Some(path));
    assert!(dirty.has_unsaved_changes);
    assets.get_mut(&handle).unwrap().name = "Before save".into();
    assert!(finish_write(
        &pending, path, true, true, &mut state, &mut dirty, &assets
    ));
    assert!(!dirty.has_unsaved_changes);
}

#[test]
fn cancelling_save_as_releases_the_save_slot_without_changing_the_project() {
    let dir = folder();
    let (mut app, handle) = fixture(dir.join("existing.ron"));
    let snapshot = serialize_snapshot(
        app.world()
            .resource::<Assets<ParticlesAsset>>()
            .get(&handle)
            .unwrap(),
    )
    .unwrap();
    app.world_mut().resource_mut::<SaveCoordinator>().pending = Some(PendingSave {
        handle: handle.clone(),
        snapshot,
        phase: SavePhase::Choosing(Arc::new(Mutex::new(Some(None)))),
    });
    app.update();
    assert!(!app.world().resource::<SaveCoordinator>().busy());
    assert_eq!(
        app.world()
            .resource::<EditorState>()
            .current_project
            .as_ref(),
        Some(&handle)
    );
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn atomic_save_follows_existing_symlinks_and_preserves_target_mode() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = folder();
    let target = dir.join("target.ron");
    let link = dir.join("link.ron");
    std::fs::write(&target, b"before").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
    symlink(&target, &link).unwrap();
    atomic_write(&link, b"after", false).unwrap();
    assert!(
        std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(std::fs::read(&target).unwrap(), b"after");
    assert_eq!(
        std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn new_project_never_overwrites_an_existing_file() {
    let dir = folder();
    let path = dir.join("existing.ron");
    std::fs::write(&path, b"original").unwrap();
    assert_eq!(
        atomic_write(&path, b"replacement", true)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    let new = dir.join("new.ron");
    atomic_write(&new, b"new project", true).unwrap();
    assert_eq!(std::fs::read(new).unwrap(), b"new project");
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn atomic_save_respects_read_only_files() {
    use std::os::unix::fs::PermissionsExt;
    let dir = folder();
    let path = dir.join("readonly.ron");
    std::fs::write(&path, b"original").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    assert_eq!(
        atomic_write(&path, b"replacement", false)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    std::fs::remove_dir_all(dir).unwrap();
}

fn editable(
    app: &mut App,
    text: &str,
    variant: crate::ui::widgets::text_edit::TextEditVariant,
) -> Entity {
    use crate::ui::widgets::text_edit::EditorTextEdit;
    use bevy::text::EditableText;
    let mut value = EditableText::default();
    value.editor_mut().set_text(text);
    let entity = app.world_mut().spawn((EditorTextEdit, variant, value)).id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(entity, FocusCause::Navigated);
    entity
}

fn bind_name(app: &mut App) {
    app.add_observer(
        |event: On<TextEditCommitEvent>,
         state: Res<EditorState>,
         mut assets: ResMut<Assets<ParticlesAsset>>,
         mut dirty: ResMut<DirtyState>| {
            if let Some(mut asset) = state
                .current_project
                .as_ref()
                .and_then(|handle| assets.get_mut(handle))
            {
                asset.name = event.text.clone();
                dirty.has_unsaved_changes = true;
            }
        },
    );
}

#[test]
fn save_commits_focused_text_before_snapshot() {
    let dir = folder();
    let path = dir.join("effect.ron");
    let (mut app, _) = fixture(path.clone());
    bind_name(&mut app);
    editable(
        &mut app,
        "Typed name",
        crate::ui::widgets::text_edit::TextEditVariant::Default,
    );
    app.world_mut().trigger(SaveProjectEvent);
    wait(&mut app);
    assert_eq!(
        load_project_from_path(&path).unwrap().asset.name,
        "Typed name"
    );
    assert!(!app.world().resource::<DirtyState>().has_unsaved_changes);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn save_commits_text_when_button_click_just_removed_focus() {
    let dir = folder();
    let path = dir.join("effect.ron");
    let (mut app, _) = fixture(path.clone());
    bind_name(&mut app);
    editable(
        &mut app,
        "Blurred name",
        crate::ui::widgets::text_edit::TextEditVariant::Default,
    );
    app.update();
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.world_mut().trigger(SaveProjectEvent);
    wait(&mut app);
    assert_eq!(
        load_project_from_path(&path).unwrap().asset.name,
        "Blurred name"
    );
    assert!(!app.world().resource::<DirtyState>().has_unsaved_changes);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn invalid_focused_number_blocks_save_and_restores_focus() {
    let dir = folder();
    let path = dir.join("effect.ron");
    std::fs::write(&path, b"original").unwrap();
    let (mut app, _) = fixture(path.clone());
    let entity = editable(
        &mut app,
        "NaN",
        crate::ui::widgets::text_edit::TextEditVariant::NumericF32,
    );
    app.update();
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.world_mut().trigger(SaveProjectEvent);
    app.update();
    assert!(!app.world().resource::<SaveCoordinator>().busy());
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(entity));
    assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    assert_eq!(std::fs::read(path).unwrap(), b"original");
    std::fs::remove_dir_all(dir).unwrap();
}
