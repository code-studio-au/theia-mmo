//! Inspector categories using the retained Sprinkles icons, theme and native focus.
use super::*;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy_sprinkles_editor::ui::icons::{ICON_CUBE, ICON_PIVOT_BOUNDBOX, ICON_SETTINGS};

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Tab {
    #[default]
    Attributes,
    Tools,
    View,
}
#[derive(Resource, Default)]
pub(super) struct State {
    pub active: Tab,
    context: Option<(EditorMode, bool, Option<outliner::Target>)>,
}
#[derive(Component)]
pub(super) struct Group(pub Tab);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<State>()
        .add_observer(activate)
        .add_zone_systems(sync.after(workspace::sync).after(properties::keyboard));
}

pub(super) fn layout(
    parent: &mut ChildSpawnerCommands,
    server: Option<&AssetServer>,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: percent(100),
            min_height: px(0),
            flex_grow: 1.,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    width: px(48),
                    flex_shrink: 0.,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(px(3), px(8)),
                    row_gap: px(8),
                    border: UiRect::right(px(1)),
                    ..default()
                },
                BorderColor::all(theme::BORDER),
            ))
            .with_children(|rail| {
                for (tab, label, icon) in [
                    (Tab::Attributes, "Attrs", ICON_CUBE),
                    (Tab::Tools, "Tools", ICON_SETTINGS),
                    (Tab::View, "View", ICON_PIVOT_BOUNDBOX),
                ] {
                    rail.spawn((
                        tab,
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        theme::Variant::Ghost,
                        bevy::input_focus::tab_navigation::TabIndex(0),
                        Node {
                            width: percent(100),
                            height: px(44),
                            flex_shrink: 0.,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(3),
                            border_radius: BorderRadius::all(px(4)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            ImageNode::new(server.map(|s| s.load(icon)).unwrap_or_default()),
                            Pickable::IGNORE,
                            Node {
                                width: px(18),
                                height: px(18),
                                ..default()
                            },
                        ));
                        button.spawn((theme::label(label, 10., theme::BODY), Pickable::IGNORE));
                    });
                }
            });
            row.spawn(Node {
                width: px(0),
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(body);
        });
}

pub(super) fn group(
    parent: &mut ChildSpawnerCommands,
    tab: Tab,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Group(tab),
            Visibility::Inherited,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.,
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(body);
}

fn activate(
    event: On<Activate>,
    buttons: Query<&Tab>,
    windows: Query<&Window>,
    session: Res<Session>,
    mut state: ResMut<State>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok(tab) = buttons.get(event.entity) else {
        return;
    };
    if workspace::busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    state.active = *tab;
    focus.set(event.entity, FocusCause::Navigated);
}

pub(super) fn sync(
    session: Res<Session>,
    mut state: ResMut<State>,
    mut groups: Query<(&Group, &mut Node, &mut Visibility)>,
    mut buttons: Query<(&Tab, &mut theme::Variant)>,
    mut panes: Query<(&scrolling::Pane, &mut ScrollPosition)>,
) {
    let context = (
        workspace::mode(&session),
        session.palette.is_some(),
        outliner::selection(&session),
    );
    if state.context != Some(context) {
        state.context = Some(context);
        state.active = if context.2.is_some() && !context.1 && !session.terrain_mode {
            Tab::Attributes
        } else {
            Tab::Tools
        };
    }
    let mut changed = false;
    for (group, mut node, mut visibility) in &mut groups {
        let display = if group.0 == state.active {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
            changed = true;
        }
        *visibility = if group.0 == state.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (tab, mut variant) in &mut buttons {
        *variant = if *tab == state.active {
            theme::Variant::Active
        } else {
            theme::Variant::Ghost
        };
    }
    if changed {
        for (scope, mut position) in &mut panes {
            if matches!(scope, scrolling::Pane::Workspace) {
                position.y = 0.;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn categories_preserve_source_redo_and_manual_choice_until_context_changes() {
        let zone = Zone::default();
        let mut editor = Editor::new(zone.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::SetSpawn {
                    position: [3., 2., 1.],
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let mut app = App::new();
        app.insert_resource(Session::new(
            editor,
            "unused.json".into(),
            Some(zone.clone()),
        ))
        .init_resource::<State>()
        .init_resource::<InputFocus>()
        .add_observer(activate);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let buttons: Vec<_> = [Tab::Attributes, Tab::Tools, Tab::View]
            .into_iter()
            .map(|tab| app.world_mut().spawn((tab, theme::Variant::Ghost)).id())
            .collect();
        for tab in [Tab::Attributes, Tab::Tools, Tab::View] {
            app.world_mut()
                .spawn((Group(tab), Node::default(), Visibility::Inherited));
        }
        let workspace = app
            .world_mut()
            .spawn((
                scrolling::Pane::Workspace,
                ScrollPosition(Vec2::new(0., 100.)),
            ))
            .id();
        let browser = app
            .world_mut()
            .spawn((scrolling::Pane::Browser, ScrollPosition(Vec2::new(0., 75.))))
            .id();
        app.world_mut().run_system_once(sync).unwrap();
        assert_eq!(app.world().resource::<State>().active, Tab::Tools);
        for entity in buttons.iter().copied() {
            app.world_mut().trigger(Activate { entity });
            app.world_mut().run_system_once(sync).unwrap();
            let active = *app.world().get::<Tab>(entity).unwrap();
            assert_eq!(app.world().resource::<State>().active, active);
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(entity));
            for (group, node, visibility) in app
                .world_mut()
                .query::<(&Group, &Node, &Visibility)>()
                .iter(app.world())
            {
                assert_eq!(
                    node.display,
                    if group.0 == active {
                        Display::Flex
                    } else {
                        Display::None
                    }
                );
                assert_eq!(
                    *visibility,
                    if group.0 == active {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    }
                );
            }
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &zone);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
        assert_eq!(app.world().get::<ScrollPosition>(workspace).unwrap().y, 0.);
        assert_eq!(app.world().get::<ScrollPosition>(browser).unwrap().y, 75.);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: buttons[0] });
        assert_eq!(app.world().resource::<State>().active, Tab::View);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.world_mut().resource_mut::<Session>().ui.input_owned = true;
        app.world_mut().trigger(Activate { entity: buttons[0] });
        assert_eq!(app.world().resource::<State>().active, Tab::View);
        for mode in [EditorMode::Sculpt, EditorMode::Paint] {
            app.world_mut().resource_mut::<Session>().set_mode(mode);
            app.world_mut().run_system_once(sync).unwrap();
            assert_eq!(app.world().resource::<State>().active, Tab::Tools);
            assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
        }
    }
}
