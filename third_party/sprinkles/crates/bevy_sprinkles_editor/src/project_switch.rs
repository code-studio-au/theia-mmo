//! Guard project replacement so unsaved particle work is not silently lost.
use std::path::PathBuf;

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy_sprinkles::prelude::*;

use crate::hosting::AuthoringAppExt;
use crate::io::{EditorData, is_example_path, save_editor_data};
use crate::project::{SaveCoordinator, TextFocusTracker, serialize_snapshot};
use crate::state::{DirtyState, EditorState, Inspectable, Inspecting};
use crate::ui::components::toasts::ToastEvent;
use crate::ui::widgets::dialog::{
    DialogActionEvent, DialogPurpose, DialogVariant, EditorDialog, OpenDialogEvent,
};
use crate::ui::widgets::text_edit::{SaveTextInputs, TextEditCommitEvent};

const PURPOSE: &str = "particle-project-switch";

pub(crate) struct ProjectSwitchPlugin;
impl Plugin for ProjectSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(request_switch)
            .add_observer(evaluate_switch)
            .add_observer(confirm_switch)
            .add_authoring_systems(show_confirmation);
    }
}

#[derive(Clone)]
enum Destination {
    Open {
        asset: ParticlesAsset,
        path: PathBuf,
        location: String,
        migrated: bool,
    },
    New {
        asset: ParticlesAsset,
        path: PathBuf,
    },
}

#[derive(Event, Clone)]
pub(crate) struct RequestProjectSwitch(Destination);
impl RequestProjectSwitch {
    pub(crate) fn open(
        asset: ParticlesAsset,
        path: PathBuf,
        location: String,
        migrated: bool,
    ) -> Self {
        Self(Destination::Open {
            asset,
            path,
            location,
            migrated,
        })
    }
    pub(crate) fn new_project(asset: ParticlesAsset, path: PathBuf) -> Self {
        Self(Destination::New { asset, path })
    }
}

#[derive(Event)]
struct EvaluateSwitch {
    destination: Destination,
    expected: Option<Handle<ParticlesAsset>>,
}

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Waiting,
    Opening,
    Showing,
}
#[derive(Resource)]
struct PendingSwitch {
    destination: Destination,
    expected: Handle<ParticlesAsset>,
    path: Option<PathBuf>,
    snapshot: String,
    phase: Phase,
}

#[allow(clippy::too_many_arguments)]
fn request_switch(
    event: On<RequestProjectSwitch>,
    state: Res<EditorState>,
    saves: Res<SaveCoordinator>,
    pending: Option<Res<PendingSwitch>>,
    mut focus: ResMut<InputFocus>,
    mut tracker: ResMut<TextFocusTracker>,
    inputs: SaveTextInputs,
    mut commands: Commands,
) {
    if saves.busy() || pending.is_some() {
        commands.trigger(ToastEvent::error(
            "Finish the current particle project operation first",
        ));
        return;
    }
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
        tracker.0 = None;
        if focus.get() == Some(entity) {
            focus.clear();
        }
        commands.trigger(TextEditCommitEvent { entity, text });
    }
    commands.trigger(EvaluateSwitch {
        destination: event.0.clone(),
        expected: state.current_project.clone(),
    });
}

#[allow(clippy::too_many_arguments)]
fn evaluate_switch(
    event: On<EvaluateSwitch>,
    mut state: ResMut<EditorState>,
    mut dirty: ResMut<DirtyState>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
    mut data: ResMut<EditorData>,
    mut saves: ResMut<SaveCoordinator>,
    pending: Option<Res<PendingSwitch>>,
    mut commands: Commands,
) {
    if state.current_project != event.expected || saves.busy() || pending.is_some() {
        commands.trigger(ToastEvent::error("The particle project changed; try again"));
        return;
    }
    if dirty.has_unsaved_changes
        && let Some(handle) = &state.current_project
    {
        let snapshot = match assets
            .get(handle)
            .and_then(|asset| serialize_snapshot(asset).ok())
        {
            Some(snapshot) => snapshot,
            None => {
                commands.trigger(ToastEvent::error(
                    "Could not protect the current particle project",
                ));
                return;
            }
        };
        commands.insert_resource(PendingSwitch {
            destination: event.destination.clone(),
            expected: handle.clone(),
            path: state.current_project_path.clone(),
            snapshot,
            phase: Phase::Waiting,
        });
        return;
    }
    apply_switch(
        event.destination.clone(),
        &mut state,
        &mut dirty,
        &mut assets,
        &mut data,
        &mut saves,
        &mut commands,
    );
}

fn show_confirmation(
    pending: Option<ResMut<PendingSwitch>>,
    dialogs: Query<Option<&DialogPurpose>, With<EditorDialog>>,
    mut commands: Commands,
) {
    let Some(mut pending) = pending else { return };
    let ours = dialogs
        .iter()
        .any(|purpose| purpose.is_some_and(|purpose| purpose.0 == PURPOSE));
    match pending.phase {
        Phase::Waiting if dialogs.is_empty() => {
            pending.phase = Phase::Opening;
            let mut event = OpenDialogEvent::new("Unsaved particle project", "Discard changes")
                .with_variant(DialogVariant::Destructive)
                .with_close_button(false)
                .with_close_on_click_outside(false)
                .with_purpose(PURPOSE);
            event.description = Some("Your current particle project has unsaved changes. Save it first or discard them to continue.".into());
            commands.trigger(event);
        }
        Phase::Opening if ours => pending.phase = Phase::Showing,
        Phase::Opening if !dialogs.is_empty() => pending.phase = Phase::Waiting,
        Phase::Showing if !ours => commands.remove_resource::<PendingSwitch>(),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn confirm_switch(
    event: On<DialogActionEvent>,
    purposes: Query<&DialogPurpose>,
    pending: Option<Res<PendingSwitch>>,
    mut state: ResMut<EditorState>,
    mut dirty: ResMut<DirtyState>,
    mut assets: ResMut<Assets<ParticlesAsset>>,
    mut data: ResMut<EditorData>,
    mut saves: ResMut<SaveCoordinator>,
    mut commands: Commands,
) {
    if !purposes
        .get(event.entity)
        .is_ok_and(|purpose| purpose.0 == PURPOSE)
    {
        return;
    }
    let Some(pending) = pending else { return };
    let unchanged = state.current_project.as_ref() == Some(&pending.expected)
        && state.current_project_path == pending.path
        && assets
            .get(&pending.expected)
            .and_then(|asset| serialize_snapshot(asset).ok())
            .as_ref()
            == Some(&pending.snapshot)
        && !saves.busy();
    let destination = pending.destination.clone();
    commands.remove_resource::<PendingSwitch>();
    if !unchanged {
        commands.trigger(ToastEvent::error("The particle project changed; try again"));
        return;
    }
    apply_switch(
        destination,
        &mut state,
        &mut dirty,
        &mut assets,
        &mut data,
        &mut saves,
        &mut commands,
    );
}

fn apply_switch(
    destination: Destination,
    state: &mut EditorState,
    dirty: &mut DirtyState,
    assets: &mut Assets<ParticlesAsset>,
    data: &mut EditorData,
    saves: &mut SaveCoordinator,
    commands: &mut Commands,
) {
    match destination {
        Destination::Open {
            asset,
            path,
            location,
            migrated,
        } => {
            let has_emitters = !asset.emitters.is_empty();
            if migrated {
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| location.clone());
                let current = bevy_sprinkles::asset::versions::current_format_version();
                commands.trigger(ToastEvent::success(format!(
                    "Project \"{filename}\" will be updated to {current} on the next save"
                )));
            }
            let is_example = is_example_path(&path);
            let handle = assets.add(asset);
            state.open_project(handle, path, dirty, migrated);
            state.inspecting = has_emitters.then_some(Inspecting {
                kind: Inspectable::Emitter,
                index: 0,
            });
            if !is_example {
                data.cache.add_recent_project(location);
                save_editor_data(data);
            }
        }
        Destination::New { asset, path } => {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let handle = assets.add(asset.clone());
            state.current_project = Some(handle.clone());
            state.current_project_path = None;
            state.inspecting = Some(Inspecting {
                kind: Inspectable::Emitter,
                index: 0,
            });
            dirty.has_unsaved_changes = true;
            if let Err(error) = saves.save_to_path(handle, &asset, path, true, true) {
                commands.trigger(ToastEvent::error(error));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosting::AuthoringHostPlugin;
    use crate::ui::widgets::text_edit::{EditorTextEdit, TextEditVariant};
    use bevy::text::EditableText;

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
    fn fixture() -> (App, Handle<ParticlesAsset>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AuthoringHostPlugin, ProjectSwitchPlugin))
            .init_resource::<InputFocus>()
            .init_resource::<TextFocusTracker>()
            .init_resource::<EditorState>()
            .init_resource::<DirtyState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<EditorData>()
            .init_resource::<SaveCoordinator>();
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .add(asset("Original"));
        app.world_mut()
            .resource_mut::<EditorState>()
            .current_project = Some(handle.clone());
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = true;
        (app, handle)
    }
    fn request(app: &mut App) {
        app.world_mut().trigger(RequestProjectSwitch::open(
            asset("Replacement"),
            crate::io::examples_dir().join("replacement.ron"),
            "replacement".into(),
            false,
        ));
        app.update();
    }
    fn dialog(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((EditorDialog, DialogPurpose(PURPOSE)))
            .id()
    }
    fn current(app: &App) -> Handle<ParticlesAsset> {
        app.world()
            .resource::<EditorState>()
            .current_project
            .clone()
            .unwrap()
    }

    #[test]
    fn dirty_switch_waits_for_its_own_confirmation_and_cancel_preserves_original() {
        let (mut app, original) = fixture();
        let old_dialog = app.world_mut().spawn(EditorDialog).id();
        request(&mut app);
        assert_eq!(current(&app), original);
        assert_eq!(
            app.world().resource::<PendingSwitch>().phase,
            Phase::Waiting
        );
        app.world_mut()
            .trigger(DialogActionEvent { entity: old_dialog });
        assert_eq!(current(&app), original);
        app.world_mut().despawn(old_dialog);
        app.update();
        let confirmation = dialog(&mut app);
        app.update();
        assert_eq!(
            app.world().resource::<PendingSwitch>().phase,
            Phase::Showing
        );
        app.world_mut().despawn(confirmation);
        app.update();
        assert!(app.world().get_resource::<PendingSwitch>().is_none());
        assert_eq!(current(&app), original);
        assert!(app.world().resource::<DirtyState>().has_unsaved_changes);
    }

    #[test]
    fn confirmation_only_switches_unchanged_project_and_stale_edits_are_preserved() {
        let (mut app, original) = fixture();
        request(&mut app);
        let confirmation = dialog(&mut app);
        app.world_mut()
            .resource_mut::<Assets<ParticlesAsset>>()
            .get_mut(&original)
            .unwrap()
            .name = "Later edit".into();
        app.world_mut().trigger(DialogActionEvent {
            entity: confirmation,
        });
        app.update();
        assert_eq!(current(&app), original);
        assert_eq!(
            app.world()
                .resource::<Assets<ParticlesAsset>>()
                .get(&original)
                .unwrap()
                .name,
            "Later edit"
        );
        app.world_mut().despawn(confirmation);
        request(&mut app);
        let confirmation = dialog(&mut app);
        app.world_mut().trigger(DialogActionEvent {
            entity: confirmation,
        });
        app.update();
        assert_ne!(current(&app), original);
        assert_eq!(
            app.world()
                .resource::<Assets<ParticlesAsset>>()
                .get(&current(&app))
                .unwrap()
                .name,
            "Replacement"
        );
    }

    #[test]
    fn focused_edit_is_committed_before_deciding_whether_to_warn() {
        let (mut app, original) = fixture();
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = false;
        app.add_observer(
            |event: On<TextEditCommitEvent>,
             state: Res<EditorState>,
             mut dirty: ResMut<DirtyState>,
             mut assets: ResMut<Assets<ParticlesAsset>>| {
                assets
                    .get_mut(state.current_project.as_ref().unwrap())
                    .unwrap()
                    .name = event.text.clone();
                dirty.has_unsaved_changes = true;
            },
        );
        let mut editable = EditableText::default();
        editable.editor_mut().set_text("Typed change");
        let field = app
            .world_mut()
            .spawn((EditorTextEdit, TextEditVariant::Default, editable))
            .id();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        request(&mut app);
        assert_eq!(current(&app), original);
        assert!(app.world().get_resource::<PendingSwitch>().is_some());
        assert_eq!(
            app.world()
                .resource::<Assets<ParticlesAsset>>()
                .get(&original)
                .unwrap()
                .name,
            "Typed change"
        );
    }

    #[test]
    fn new_project_request_does_not_create_a_file_before_confirmation() {
        let (mut app, original) = fixture();
        let path = std::env::temp_dir().join(format!(
            "theia-particle-pending-{}-{}.ron",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        app.world_mut().trigger(RequestProjectSwitch::new_project(
            asset("New"),
            path.clone(),
        ));
        app.update();
        assert_eq!(current(&app), original);
        assert!(app.world().get_resource::<PendingSwitch>().is_some());
        assert!(!path.exists());
    }
}
