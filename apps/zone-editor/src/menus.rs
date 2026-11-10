//! Native Bevy application menus using the Sprinkles visual vocabulary.
use super::*;
#[path = "menus_context.rs"]
mod context;
#[path = "menus_property_actions.rs"]
mod property_actions;
#[path = "menus_select.rs"]
mod select;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{NavAction, TabIndex},
};
use bevy::ui_widgets::{MenuAction, MenuButton, MenuEvent, MenuFocusState, MenuItem, MenuPopup};
pub(super) use select::{Kind as SelectKind, SelectTrigger};

#[derive(Component)]
pub(super) struct Trigger(pub &'static str);
#[derive(Component)]
pub(super) struct Popup(pub &'static str);

pub(super) fn plugin(app: &mut App) {
    app.add_observer(activate).add_plugins((
        context::plugin,
        property_actions::plugin,
        select::plugin,
    ));
}

pub(super) fn spawn_property_actions(
    parent: &mut ChildSpawnerCommands,
    server: Option<&AssetServer>,
) {
    property_actions::spawn(parent, server);
}
pub(super) use property_actions::GuideControls;

pub(super) fn spawn_select(parent: &mut ChildSpawnerCommands, kind: SelectKind) {
    select::spawn(parent, kind);
}

pub(super) fn spawn_content(parent: &mut ChildSpawnerCommands, row: usize) {
    context::spawn(parent, row);
}

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    title: &'static str,
    items: &[(workspace::Control, &'static str, &'static str)],
) {
    let mut owner = parent.spawn(Node::default());
    let anchor = owner.id();
    let mut button = Entity::PLACEHOLDER;
    let mut popup = Entity::PLACEHOLDER;
    owner.with_children(|root| {
        button = root
            .spawn((
                Button,
                MenuButton,
                Trigger(title),
                Hovered::default(),
                theme::Variant::Ghost,
                Node {
                    min_height: px(28),
                    padding: UiRect::axes(px(12), px(4)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::Variant::Ghost.background(false)),
                children![text(title, 14.)],
            ))
            .id();
        popup = root
            .spawn((
                MenuPopup::default(),
                Popup(title),
                Hovered::default(),
                GlobalZIndex(15),
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    top: px(34),
                    left: px(0),
                    width: px(260),
                    padding: UiRect::all(px(4)),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BorderColor::all(theme::BORDER),
                BackgroundColor(theme::PANEL),
            ))
            .with_children(|menu| {
                spawn_items(menu, items);
            })
            .id();
    });
    parent.commands().entity(anchor).observe(
        move |event: On<MenuEvent>,
              session: Res<Session>,
              windows: Query<&Window>,
              mut popups: Query<(&mut Node, &mut MenuFocusState), With<MenuPopup>>,
              mut focus: ResMut<InputFocus>| {
            let Ok((mut node, mut state)) = popups.get_mut(popup) else {
                return;
            };
            let allowed =
                !workspace::busy(&session) && windows.single().is_ok_and(|window| window.focused);
            match event.action {
                MenuAction::Open(direction) if allowed => {
                    node.display = Display::Flex;
                    *state = MenuFocusState::Opening(direction);
                }
                MenuAction::Toggle if allowed => {
                    if node.display == Display::None {
                        node.display = Display::Flex;
                        *state = MenuFocusState::Opening(NavAction::First);
                    } else {
                        node.display = Display::None;
                        *state = MenuFocusState::Closed;
                    }
                }
                MenuAction::CloseAll => {
                    node.display = Display::None;
                    *state = MenuFocusState::Closed;
                }
                MenuAction::FocusRoot => focus.set(button, FocusCause::Navigated),
                _ => {}
            }
        },
    );
}

pub(super) fn spawn_items(
    menu: &mut ChildSpawnerCommands,
    items: &[(workspace::Control, &'static str, &'static str)],
) {
    for &(control, label, shortcut) in items {
        menu.spawn((
            control,
            MenuItem,
            Hovered::default(),
            TabIndex(0),
            Node {
                width: percent(100),
                min_height: px(28),
                padding: UiRect::axes(px(8), px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border_radius: BorderRadius::all(px(2)),
                ..default()
            },
            BackgroundColor(theme::Variant::Ghost.background(false)),
            children![
                text(label, 14.),
                (
                    Text::new(shortcut),
                    theme::Secondary,
                    theme::font(10.),
                    TextColor(theme::MUTED),
                    bevy::picking::Pickable::IGNORE
                )
            ],
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn activate(
    event: On<Activate>,
    items: Query<&workspace::Control, With<MenuItem>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut requests: ResMut<workspace::Requests>,
    mut content: ResMut<outliner::Content>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    contexts: Query<&context::State>,
    property_contexts: Query<&property_actions::State>,
    tabs: Option<Res<property_tabs::State>>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Ok(control) = items.get(event.entity) else {
        return;
    };
    if parents
        .iter_ancestors(event.entity)
        .find_map(|entity| contexts.get(entity).ok())
        .is_some_and(|context| !context.valid(&session, &content))
        || parents
            .iter_ancestors(event.entity)
            .find_map(|entity| property_contexts.get(entity).ok())
            .is_some_and(|context| !context.valid(&session, tabs.as_deref()))
    {
        session.status = "Content changed; reopen its actions menu".into();
        commands.trigger(MenuEvent {
            source: event.entity,
            action: MenuAction::CloseAll,
        });
        return;
    }
    workspace::execute(
        *control,
        &mut session,
        &mut view,
        &mut requests,
        &mut content,
    );
    commands.trigger(MenuEvent {
        source: event.entity,
        action: MenuAction::CloseAll,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use workspace::Control;
    fn setup(mut commands: Commands) {
        commands.spawn(Node::default()).with_children(|root| {
            spawn(
                root,
                "Edit",
                &[
                    (Control::Command(Action::Undo), "Undo", "Ctrl Z"),
                    (Control::Command(Action::Redo), "Redo", "Ctrl Y"),
                    (Control::FileOpen, "Open…", "Ctrl O"),
                ],
            )
        });
    }
    #[test]
    fn menu_events_open_close_focus_and_dispatch_the_shared_edit_exactly_once() {
        let zone = Zone {
            origin: [128_000_000_000.125, 20.25, -128_000_000_000.5],
            spawn: [128_000_000_000.125, 20.35, -128_000_000_000.5],
            objects: vec![Object {
                id: 1,
                name: "Exact menu selection".into(),
                primitive: Primitive::Box,
                pose: Pose {
                    position: [128_000_000_008.25, 21.5, -128_000_000_007.],
                    yaw_degrees: 17.25,
                    size: [2., 2.5, 3.],
                },
            }],
            ..default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Menu edit".into(),
                }],
            })
            .unwrap();
        let mut app = App::new();
        app.insert_resource(Session::new(
            editor,
            "unused.json".into(),
            Some(zone.clone()),
        ))
        .insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        })
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InputFocus>()
        .init_resource::<outliner::Content>()
        .init_resource::<workspace::Requests>()
        .add_plugins(plugin)
        .add_systems(Startup, setup)
        .add_zone_systems(crate::keyboard);
        app.world_mut().resource_mut::<Session>().selected = Some(1);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<MenuButton>>()
            .single(app.world())
            .unwrap();
        let popup = app
            .world_mut()
            .query_filtered::<Entity, With<MenuPopup>>()
            .single(app.world())
            .unwrap();
        let mut item = |action: Action| {
            app.world_mut().query::<(Entity,&Control)>().iter(app.world()).find(|(_,control)|matches!(control,Control::Command(a) if std::mem::discriminant(a)==std::mem::discriminant(&action))).unwrap().0
        };
        let undo = item(Action::Undo);
        let redo = item(Action::Redo);
        app.world_mut().trigger(MenuEvent {
            source: button,
            action: MenuAction::Toggle,
        });
        app.update();
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::Flex
        );
        assert_eq!(app.world().resource::<Session>().editor.revision(), 1);
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(undo, FocusCause::Navigated);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.revision(), 1);
        assert_eq!(
            app.world().resource::<Session>().editor.zone().objects,
            zone.objects
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().trigger(Activate { entity: undo });
        app.update();
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        app.world_mut().trigger(Activate { entity: undo });
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
        app.world_mut().trigger(MenuEvent {
            source: button,
            action: MenuAction::Open(NavAction::Last),
        });
        app.update();
        assert!(matches!(
            app.world().get::<MenuFocusState>(popup),
            Some(MenuFocusState::Opening(NavAction::Last))
        ));
        app.world_mut().trigger(Activate { entity: redo });
        app.update();
        assert_eq!(app.world().resource::<Session>().editor.revision(), 3);
        assert_eq!(
            app.world().resource::<Session>().editor.zone().name,
            "Menu edit"
        );
        app.world_mut().trigger(MenuEvent {
            source: button,
            action: MenuAction::FocusRoot,
        });
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(button));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ControlLeft);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert!(app.world().resource::<workspace::Requests>().find);
        assert_eq!(app.world().resource::<Session>().editor.revision(), 3);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(MenuEvent {
            source: button,
            action: MenuAction::Toggle,
        });
        app.update();
        app.world_mut().trigger(Activate { entity: undo });
        app.update();
        assert_eq!(
            app.world().get::<Node>(popup).unwrap().display,
            Display::None
        );
        assert_eq!(app.world().resource::<Session>().editor.revision(), 3);
        assert_eq!(
            app.world().resource::<Session>().editor.zone().origin,
            zone.origin
        );
    }
}
