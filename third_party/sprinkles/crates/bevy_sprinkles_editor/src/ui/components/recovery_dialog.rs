//! Recovery UI for separately stored particle drafts.
use std::sync::{Arc, Mutex};

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;

use crate::drafts::{DraftId, DraftStore, RecoveryStore, RestoreDraftEvent};
use crate::hosting::AuthoringAppExt;
use crate::ui::widgets::button::{ButtonClickEvent, ButtonProps, ButtonVariant, button};
use crate::ui::widgets::dialog::{
    CloseDialogEvent, DialogActionEvent, DialogChildrenSlot, EditorDialog, OpenDialogEvent,
};
use crate::ui::widgets::scroll::scrollbar;

pub fn plugin(app: &mut App) {
    app.add_observer(open_recovery)
        .add_observer(restore_selected)
        .add_observer(discard_clicked)
        .add_observer(close_recovery)
        .add_authoring_systems((populate_recovery, cleanup_recovery));
}

#[derive(Component)]
pub struct RecoveryButton;
#[derive(Component)]
struct RestoreButton(DraftId);
#[derive(Clone, Copy)]
enum DiscardAction {
    Ask,
    Confirm,
    Cancel,
}
#[derive(Component)]
struct DiscardChoice {
    id: DraftId,
    row: Entity,
    ask: Entity,
    confirm: Entity,
    cancel: Entity,
    action: DiscardAction,
}
struct Entry {
    id: DraftId,
    title: String,
    subtitle: String,
}
type LoadResult = Arc<Mutex<Option<Result<Vec<Entry>, String>>>>;
#[derive(Resource)]
struct RecoveryDialogState {
    result: LoadResult,
    populated: bool,
    dialog: Option<Entity>,
}

fn collect_entries(store: &DraftStore) -> Result<Vec<Entry>, String> {
    store
        .list()
        .map(|ids| {
            ids.into_iter()
                .rev()
                .map(|id| {
                    let (title, subtitle) = match store.recover(&id) {
                        Ok(draft) => (
                            draft.asset.name,
                            draft
                                .original_path
                                .map(|path| path.display().to_string())
                                .unwrap_or_else(|| "Unsaved project".into()),
                        ),
                        Err(_) => (
                            "Unreadable draft".into(),
                            format!("Draft {} — preserved", id.as_str()),
                        ),
                    };
                    Entry {
                        id,
                        title,
                        subtitle,
                    }
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

fn open_recovery(
    event: On<ButtonClickEvent>,
    buttons: Query<(), With<RecoveryButton>>,
    store: Res<RecoveryStore>,
    mut commands: Commands,
) {
    if !buttons.contains(event.entity) {
        return;
    }
    let store = store.0.clone();
    let result: LoadResult = Arc::new(Mutex::new(None));
    let completed = result.clone();
    IoTaskPool::get()
        .spawn(async move {
            let loaded = collect_entries(&store);
            if let Ok(mut result) = completed.lock() {
                *result = Some(loaded);
            }
        })
        .detach();
    commands.insert_resource(RecoveryDialogState {
        result,
        populated: false,
        dialog: None,
    });
    commands.trigger(
        OpenDialogEvent::new("Recover drafts", "Close")
            .without_cancel()
            .with_max_width(px(600)),
    );
}

fn populate_recovery(
    state: Option<ResMut<RecoveryDialogState>>,
    slots: Query<Entity, With<DialogChildrenSlot>>,
    dialogs: Query<Entity, With<EditorDialog>>,
    mut commands: Commands,
) {
    let Some(mut state) = state else { return };
    if state.populated {
        return;
    }
    let Ok(slot) = slots.single() else { return };
    state.dialog = dialogs.single().ok();
    let loaded = match state.result.lock() {
        Ok(mut result) => result.take(),
        Err(_) => Some(Err("Could not read drafts".into())),
    };
    let Some(loaded) = loaded else { return };
    state.populated = true;
    let scroll = commands
        .spawn((
            Hovered::default(),
            Node {
                width: percent(100),
                max_height: px(384),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();
    commands.entity(scroll).with_child(scrollbar(scroll));
    match loaded {
        Ok(entries) if !entries.is_empty() => {
            for entry in entries {
                let row = commands
                    .spawn((
                        Node {
                            width: percent(100),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ChildOf(scroll),
                    ))
                    .id();
                let restore = commands
                    .spawn_scene(button(
                        ButtonProps::new(entry.title)
                            .with_variant(ButtonVariant::Ghost)
                            .align_left()
                            .with_direction(FlexDirection::Column)
                            .with_subtitle(&entry.subtitle),
                    ))
                    .insert(RestoreButton(entry.id.clone()))
                    .insert(ChildOf(row))
                    .id();
                commands
                    .entity(restore)
                    .entry::<Node>()
                    .and_modify(|mut node| {
                        node.flex_grow = 1.0;
                        node.min_width = px(0);
                    });
                let ask = commands
                    .spawn_scene(button(
                        ButtonProps::new("Discard...").with_variant(ButtonVariant::Ghost),
                    ))
                    .insert(ChildOf(row))
                    .id();
                let confirm = commands
                    .spawn_scene(button(
                        ButtonProps::new("Confirm discard").with_variant(ButtonVariant::Ghost),
                    ))
                    .insert(ChildOf(row))
                    .id();
                let cancel = commands
                    .spawn_scene(button(
                        ButtonProps::new("Cancel").with_variant(ButtonVariant::Ghost),
                    ))
                    .insert(ChildOf(row))
                    .id();
                for (entity, action) in [
                    (ask, DiscardAction::Ask),
                    (confirm, DiscardAction::Confirm),
                    (cancel, DiscardAction::Cancel),
                ] {
                    commands.entity(entity).insert(DiscardChoice {
                        id: entry.id.clone(),
                        row,
                        ask,
                        confirm,
                        cancel,
                        action,
                    });
                }
                for entity in [confirm, cancel] {
                    commands
                        .entity(entity)
                        .entry::<Node>()
                        .and_modify(|mut node| node.display = Display::None);
                }
            }
        }
        Ok(_) => {
            commands
                .entity(scroll)
                .with_child(Text::new("No saved drafts yet."));
        }
        Err(_) => {
            commands
                .entity(scroll)
                .with_child(Text::new("Could not read the drafts folder."));
        }
    }
    commands.entity(slot).add_child(scroll);
}

fn restore_selected(
    event: On<ButtonClickEvent>,
    buttons: Query<&RestoreButton>,
    mut commands: Commands,
) {
    if let Ok(button) = buttons.get(event.entity) {
        commands.trigger(RestoreDraftEvent(button.0.clone()));
    }
}
fn discard_clicked(
    event: On<ButtonClickEvent>,
    choices: Query<&DiscardChoice>,
    mut nodes: Query<&mut Node>,
    store: Res<RecoveryStore>,
    mut commands: Commands,
) {
    let Ok(choice) = choices.get(event.entity) else {
        return;
    };
    if !nodes
        .get(event.entity)
        .is_ok_and(|node| node.display != Display::None)
    {
        return;
    }
    match choice.action {
        DiscardAction::Ask | DiscardAction::Cancel => {
            let confirming = matches!(choice.action, DiscardAction::Ask);
            for (entity, visible) in [
                (choice.ask, !confirming),
                (choice.confirm, confirming),
                (choice.cancel, confirming),
            ] {
                if let Ok(mut node) = nodes.get_mut(entity) {
                    node.display = if visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
            }
        }
        DiscardAction::Confirm => match store.0.discard(&choice.id) {
            Ok(()) => {
                commands.entity(choice.row).try_despawn();
                commands.trigger(crate::ui::components::toasts::ToastEvent::success(
                    "Draft discarded",
                ));
            }
            Err(_) => commands.trigger(crate::ui::components::toasts::ToastEvent::error(
                "Could not discard draft",
            )),
        },
    }
}
fn close_recovery(
    event: On<DialogActionEvent>,
    state: Option<Res<RecoveryDialogState>>,
    mut commands: Commands,
) {
    if state.is_some_and(|state| state.dialog == Some(event.entity)) {
        commands.remove_resource::<RecoveryDialogState>();
        commands.trigger(CloseDialogEvent);
    }
}
fn cleanup_recovery(
    state: Option<Res<RecoveryDialogState>>,
    dialogs: Query<(), With<EditorDialog>>,
    mut commands: Commands,
) {
    if state.is_some() && dialogs.is_empty() {
        commands.remove_resource::<RecoveryDialogState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drafts::Draft;
    use bevy_sprinkles::prelude::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn entries_identify_valid_and_unreadable_drafts_without_removing_them() {
        let dir = std::env::temp_dir().join(format!(
            "theia-draft-ui-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = DraftStore::new(dir.clone());
        let asset = ParticlesAsset::new(
            "Spark".into(),
            ParticlesDimension::D3,
            default(),
            vec![],
            vec![],
            false,
            default(),
        );
        let good = store
            .create(&Draft::new(None, ron::to_string(&asset).unwrap()))
            .unwrap();
        let bad = store.create(&Draft::new(None, "invalid".into())).unwrap();
        let entries = collect_entries(&store).unwrap();
        assert!(entries.iter().any(|entry| entry.id == good
            && entry.title == "Spark"
            && entry.subtitle == "Unsaved project"));
        assert!(
            entries
                .iter()
                .any(|entry| entry.id == bad && entry.title == "Unreadable draft")
        );
        assert_eq!(store.list().unwrap().len(), 2);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn discard_requires_confirmation_and_preserves_other_drafts() {
        let dir = std::env::temp_dir().join(format!(
            "theia-draft-discard-ui-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = DraftStore::new(dir.clone());
        let first = store.create(&Draft::new(None, "first".into())).unwrap();
        let second = store.create(&Draft::new(None, "second".into())).unwrap();
        let mut app = App::new();
        app.insert_resource(RecoveryStore(store.clone()))
            .add_observer(discard_clicked);
        let row = app.world_mut().spawn(Node::default()).id();
        let ask = app.world_mut().spawn(Node::default()).id();
        let confirm = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        let cancel = app
            .world_mut()
            .spawn(Node {
                display: Display::None,
                ..default()
            })
            .id();
        for (entity, action) in [
            (ask, DiscardAction::Ask),
            (confirm, DiscardAction::Confirm),
            (cancel, DiscardAction::Cancel),
        ] {
            app.world_mut().entity_mut(entity).insert(DiscardChoice {
                id: first.clone(),
                row,
                ask,
                confirm,
                cancel,
                action,
            });
        }
        app.update();
        app.world_mut()
            .trigger(ButtonClickEvent { entity: confirm });
        app.update();
        assert_eq!(store.list().unwrap().len(), 2);
        app.world_mut().trigger(ButtonClickEvent { entity: ask });
        app.update();
        assert_eq!(store.list().unwrap().len(), 2);
        assert_eq!(
            app.world().get::<Node>(confirm).unwrap().display,
            Display::Flex
        );
        app.world_mut().trigger(ButtonClickEvent { entity: cancel });
        app.update();
        assert_eq!(store.list().unwrap().len(), 2);
        assert_eq!(
            app.world().get::<Node>(confirm).unwrap().display,
            Display::None
        );
        app.world_mut().trigger(ButtonClickEvent { entity: ask });
        app.world_mut()
            .trigger(ButtonClickEvent { entity: confirm });
        app.update();
        assert_eq!(store.list().unwrap(), vec![second]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn clicking_an_entry_requests_only_its_draft() {
        #[derive(Resource, Default)]
        struct Requests(Vec<DraftId>);
        let dir = std::env::temp_dir().join(format!(
            "theia-draft-click-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = DraftStore::new(dir.clone());
        let id = store.create(&Draft::new(None, "snapshot".into())).unwrap();
        let mut app = App::new();
        app.init_resource::<Requests>()
            .add_observer(restore_selected)
            .add_observer(
                |event: On<RestoreDraftEvent>, mut requests: ResMut<Requests>| {
                    requests.0.push(event.0.clone())
                },
            );
        let entity = app.world_mut().spawn(RestoreButton(id.clone())).id();
        app.update();
        app.world_mut().trigger(ButtonClickEvent { entity });
        app.update();
        assert_eq!(app.world().resource::<Requests>().0, vec![id]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
