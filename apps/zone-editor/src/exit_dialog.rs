//! Explicit editor exit with a guard for unsaved zone and particle changes.
use super::*;
use bevy::app::AppExit;
use bevy::input_focus::{FocusCause, InputFocus, tab_navigation::TabGroup};
use bevy_sprinkles::asset::ParticlesAsset;
use bevy_sprinkles_editor::{
    hosting::AuthoringUiUpdate,
    project::ProjectOperation,
    state::{DirtyState, EditorState},
};

#[derive(Component)]
struct Root;
#[derive(Component)]
enum Choice {
    Exit,
    Cancel,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_observer(choose)
        .add_zone_systems(update.after(AuthoringUiUpdate));
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Root,
            TabGroup::modal(),
            GlobalZIndex(40),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::dialog::BACKDROP),
        ))
        .with_children(|root| {
            root.spawn(theme::dialog::panel(460.)).with_children(|panel| {
                panel.spawn(theme::dialog::header()).with_child(theme::label("Unsaved changes", 18., theme::DISPLAY));
                panel.spawn((
                    theme::label("Save your zone and particle effect before exiting, or discard the unsaved changes.", 14., theme::BODY),
                    Node { margin: UiRect::all(px(16)), ..default() },
                ));
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(8), margin: UiRect::all(px(16)), ..default() })
                    .with_children(|row| {
                        for (choice, label, variant) in [
                            (Choice::Cancel, "Cancel", theme::Variant::Default),
                            (Choice::Exit, "Exit without saving", theme::Variant::Destructive),
                        ] {
                            row.spawn((choice, Button, ActivateOnPress, Hovered::default(), variant,
                                bevy::input_focus::tab_navigation::TabIndex(0), theme::button_node(), BackgroundColor(variant.background(false))))
                                .with_child(theme::label(label, 13., theme::BODY));
                        }
                    });
            });
        });
}

fn choose(
    event: On<Activate>,
    choices: Query<&Choice>,
    mut session: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
) {
    if !session.exit_confirmation {
        return;
    }
    match choices.get(event.entity) {
        Ok(Choice::Exit) => {
            session.exit_confirmation = false;
            exit.write(AppExit::Success);
        }
        Ok(Choice::Cancel) => {
            session.exit_confirmation = false;
        }
        Err(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut requests: ResMut<workspace::Requests>,
    mut session: ResMut<Session>,
    dirty: Res<DirtyState>,
    editor: Res<EditorState>,
    assets: Res<Assets<ParticlesAsset>>,
    operation: ProjectOperation,
    keys: Res<ButtonInput<KeyCode>>,
    mut roots: Query<&mut Node, With<Root>>,
    mut exit: MessageWriter<AppExit>,
    mut defer: Local<bool>,
    mut was_open: Local<bool>,
    mut focus: ResMut<InputFocus>,
    choices: Query<(Entity, &Choice)>,
) {
    if std::mem::take(&mut requests.exit) {
        *defer = true;
    } else if std::mem::take(&mut *defer) {
        if operation.busy()
            || session.is_dirty()
            || session.inline_editing
            || (dirty.has_unsaved_changes && !editor.is_pristine_example(&assets))
        {
            session.exit_confirmation = true;
        } else {
            exit.write(AppExit::Success);
        }
    }
    if session.exit_confirmation && keys.just_pressed(KeyCode::Escape) {
        session.exit_confirmation = false;
    }
    if session.exit_confirmation && !*was_open {
        if let Some((entity, _)) = choices
            .iter()
            .find(|(_, choice)| matches!(choice, Choice::Cancel))
        {
            focus.set(entity, FocusCause::Navigated);
        }
    } else if !session.exit_confirmation
        && *was_open
        && focus.get().is_some_and(|entity| choices.contains(entity))
    {
        focus.clear();
    }
    *was_open = session.exit_confirmation;
    for mut node in &mut roots {
        node.display = if session.exit_confirmation {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let zone = Zone::default();
        let mut app = App::new();
        app.add_message::<AppExit>()
            .insert_resource(Session::new(
                Editor::new(zone.clone()).unwrap(),
                "unused.json".into(),
                Some(zone),
            ))
            .init_resource::<workspace::Requests>()
            .init_resource::<DirtyState>()
            .init_resource::<EditorState>()
            .init_resource::<Assets<ParticlesAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .add_plugins(plugin);
        app.update();
        app
    }

    fn request(app: &mut App) {
        app.world_mut().resource_mut::<workspace::Requests>().exit = true;
        app.update();
        app.update();
    }

    #[test]
    fn clean_exit_closes_and_dirty_zone_requires_explicit_discard() {
        let mut clean = app();
        request(&mut clean);
        assert!(clean.should_exit().is_some());
        let mut dirty = app();
        dirty
            .world_mut()
            .resource_mut::<Session>()
            .edit(vec![Edit::RenameZone {
                name: "Changed".into(),
            }]);
        request(&mut dirty);
        assert!(dirty.should_exit().is_none());
        assert!(dirty.world().resource::<Session>().exit_confirmation);
        let cancel = dirty
            .world_mut()
            .query::<(Entity, &Choice)>()
            .iter(dirty.world())
            .find_map(|(entity, choice)| matches!(choice, Choice::Cancel).then_some(entity))
            .unwrap();
        dirty.world_mut().trigger(Activate { entity: cancel });
        dirty.update();
        assert!(!dirty.world().resource::<Session>().exit_confirmation);
        assert!(dirty.should_exit().is_none());
        request(&mut dirty);
        let exit = dirty
            .world_mut()
            .query::<(Entity, &Choice)>()
            .iter(dirty.world())
            .find_map(|(entity, choice)| matches!(choice, Choice::Exit).then_some(entity))
            .unwrap();
        dirty.world_mut().trigger(Activate { entity: exit });
        dirty.update();
        assert!(dirty.should_exit().is_some());
    }

    #[test]
    fn dirty_particle_also_requires_confirmation() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<DirtyState>()
            .has_unsaved_changes = true;
        request(&mut app);
        assert!(app.should_exit().is_none());
        assert!(app.world().resource::<Session>().exit_confirmation);
    }
}
