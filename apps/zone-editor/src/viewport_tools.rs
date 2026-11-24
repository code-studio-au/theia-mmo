//! Selected-content tools over the viewport, sharing the inspector's commands.
use super::*;
use bevy_sprinkles_editor::ui::icons::{
    ICON_ARROW_DOWN, ICON_EMPTY_AXIS, ICON_EXPAND_HORIZONTAL, ICON_PIVOT_BOUNDBOX, ICON_REPEAT,
};
use bevy_sprinkles_editor::ui::widgets::popover::EditorPopover;

const WIDTH: f32 = 56.;
const ITEM: f32 = 36.;
const GAP: f32 = 4.;
const PADDING: f32 = 6.;
const HEIGHT: f32 = ITEM * 5. + GAP * 4. + PADDING * 2. + 2.;
const OFFSET: f32 = 12.;
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct ToolButton;
#[derive(Resource, Default)]
struct PointerOwner(bool);
type PopupFilter = Or<(With<EditorPopover>, With<bevy::ui_widgets::MenuPopup>)>;

pub(super) fn bounds(window: &Window, session: &Session) -> Option<Rect> {
    if session.play.is_some()
        || session.has_modal()
        || session.palette.is_some()
        || session.plant_palette.is_some()
        || session.marker_palette.is_some()
        || session.path_draft.is_some()
        || session.terrain_mode
        || session.particle_selected.is_some()
        || session.quest_area_selected.is_some()
        || session.spawn_group_selected.is_some()
        || outliner::selection(session) == Some(outliner::Target::Spawn)
        || (outliner::selection(session).is_none() && session.group.is_empty())
    {
        return None;
    }
    let min = Vec2::new(
        session.ui.canvas_left() + OFFSET,
        workspace::HEADER + OFFSET,
    );
    let max = min + Vec2::new(WIDTH, HEIGHT);
    (max.x < window.width()
        && max.y < window.height() - session.ui.canvas_bottom(window.height(), true))
    .then_some(Rect::from_corners(min, max))
}

pub(super) fn blocks(window: &Window, session: &Session) -> bool {
    session.ui.overlay_owned
        || window
            .cursor_position()
            .is_some_and(|cursor| bounds(window, session).is_some_and(|rect| rect.contains(cursor)))
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PointerOwner>()
        .add_systems(Startup, setup)
        .add_zone_systems(
            guard
                .before(panels::resize)
                .before(viewport::pointer)
                .before(terrain::pointer)
                .before(paths::pointer)
                .before(markers::pointer)
                .before(vegetation::pointer)
                .before(camera_controls),
        )
        .add_zone_systems(sync.after(workspace::sync));
}

fn setup(
    mut commands: Commands,
    server: Option<Res<AssetServer>>,
    images: Option<Res<Assets<Image>>>,
) {
    let server = server.as_deref().filter(|_| images.is_some());
    commands
        .spawn((
            Root,
            GlobalZIndex(5),
            Visibility::Hidden,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: px(WIDTH),
                height: px(HEIGHT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(PADDING)),
                row_gap: px(GAP),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|strip| {
            for (control, caption, icon) in [
                (
                    workspace::Control::GizmoTool(gizmo::Tool::Move),
                    "Move",
                    ICON_EMPTY_AXIS,
                ),
                (
                    workspace::Control::GizmoTool(gizmo::Tool::Rotate),
                    "Rotate",
                    ICON_REPEAT,
                ),
                (
                    workspace::Control::GizmoTool(gizmo::Tool::Scale),
                    "Scale",
                    ICON_EXPAND_HORIZONTAL,
                ),
                (workspace::Control::Frame, "Frame", ICON_PIVOT_BOUNDBOX),
                (
                    workspace::Control::Command(Action::Ground),
                    "Ground",
                    ICON_ARROW_DOWN,
                ),
            ] {
                strip
                    .spawn((
                        ToolButton,
                        control,
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        theme::Variant::Ghost,
                        Node {
                            width: percent(100),
                            height: px(ITEM),
                            flex_shrink: 0.,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            row_gap: px(2),
                            border_radius: BorderRadius::all(px(3)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            ImageNode::new(server.map(|s| s.load(icon)).unwrap_or_default()),
                            Pickable::IGNORE,
                            Node {
                                width: px(16),
                                height: px(16),
                                ..default()
                            },
                        ));
                        button.spawn((theme::label(caption, 10., theme::BODY), Pickable::IGNORE));
                    });
            }
        });
}

fn guard(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    popovers: Query<&Hovered, PopupFilter>,
    mut owner: ResMut<PointerOwner>,
    mut session: ResMut<Session>,
) {
    let Ok(window) = windows.single() else { return };
    if !window.focused {
        owner.0 = false;
        session.ui.overlay_owned = false;
        return;
    }
    let buttons = [MouseButton::Left, MouseButton::Middle, MouseButton::Right];
    let popup_hovered = popovers.iter().any(Hovered::get);
    if mouse.any_just_pressed(buttons)
        && (popup_hovered
            || window.cursor_position().is_some_and(|cursor| {
                bounds(window, &session).is_some_and(|rect| rect.contains(cursor))
            }))
    {
        owner.0 = true;
    }
    // Include the release frame even when the pointer has left the toolbar.
    session.ui.overlay_owned = owner.0 || popup_hovered;
    if !mouse.any_pressed(buttons) {
        owner.0 = false;
    }
}

fn sync(
    windows: Query<&Window>,
    session: Res<Session>,
    mut roots: Query<(&mut Node, &mut Visibility), With<Root>>,
) {
    let Ok(window) = windows.single() else { return };
    let rect = bounds(window, &session);
    for (mut node, mut visibility) in &mut roots {
        if let Some(rect) = rect {
            node.left = px(rect.min.x);
            node.top = px(rect.min.y);
            node.display = Display::Flex;
            *visibility = Visibility::Inherited;
        } else {
            node.display = Display::None;
            *visibility = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn hovered_popup_owns_drag_over_the_actual_dock_edge() {
        for sprinkles in [false, true] {
            let zone = Zone::default();
            let mut session = Session::new(
                Editor::new(zone.clone()).unwrap(),
                "unused.json".into(),
                Some(zone),
            );
            session.particle_selected = Some(7);
            let edge = session.ui.canvas_left();
            let original = session.ui.width;
            let mut app = App::new();
            app.insert_resource(session)
                .init_resource::<PointerOwner>()
                .init_resource::<ButtonInput<MouseButton>>()
                .init_resource::<ButtonInput<KeyCode>>()
                .add_systems(Update, (guard, panels::resize).chain());
            let window = app
                .world_mut()
                .spawn(Window {
                    focused: true,
                    resolution: (1440, 900).into(),
                    ..default()
                })
                .id();
            if sprinkles {
                app.world_mut().spawn((EditorPopover, Hovered(true)));
            } else {
                app.world_mut()
                    .spawn((bevy::ui_widgets::MenuPopup::default(), Hovered(true)));
            }
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(Vec2::new(edge, 300.)));
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.update();
            assert!(app.world().resource::<Session>().ui.overlay_owned);
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(Vec2::new(edge + 80., 300.)));
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .clear();
            app.update();
            assert_eq!(app.world().resource::<Session>().ui.width, original);
            assert_eq!(app.world().resource::<Session>().particle_selected, Some(7));
        }
    }

    #[test]
    fn toolbar_owns_each_mouse_gesture_through_release_and_focus_loss_cancels() {
        for origin in [[0.; 3], [128_000_000_000.125, 20.25, -128_000_000_000.5]] {
            let mut zone = Zone {
                origin,
                spawn: origin,
                ..default()
            };
            zone.objects.push(Object {
                id: 1,
                name: "Box".into(),
                primitive: Primitive::Box,
                pose: Pose {
                    position: origin,
                    yaw_degrees: 0.,
                    size: [2.; 3],
                },
            });
            let mut editor = Editor::new(zone.clone()).unwrap();
            let mut spawn = origin;
            spawn[1] += 1.;
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::SetSpawn { position: spawn }],
                })
                .unwrap();
            editor
                .execute(Command::Undo {
                    expected_revision: 1,
                })
                .unwrap();
            let mut app = App::new();
            let mut session = Session::new(editor, "unused.json".into(), Some(zone.clone()));
            session.selected = Some(1);
            app.insert_resource(session)
                .init_resource::<PointerOwner>()
                .init_resource::<ButtonInput<MouseButton>>();
            let window = app
                .world_mut()
                .spawn(Window {
                    focused: true,
                    resolution: (3440, 1440).into(),
                    ..default()
                })
                .id();
            for button in [MouseButton::Left, MouseButton::Middle, MouseButton::Right] {
                let point = bounds(
                    app.world().get::<Window>(window).unwrap(),
                    app.world().resource::<Session>(),
                )
                .unwrap()
                .center();
                app.world_mut()
                    .get_mut::<Window>(window)
                    .unwrap()
                    .set_cursor_position(Some(point));
                assert!(
                    canvas_cursor(
                        app.world().get::<Window>(window).unwrap(),
                        app.world().resource::<Session>()
                    )
                    .is_none()
                );
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(button);
                app.world_mut().run_system_once(guard).unwrap();
                app.world_mut()
                    .get_mut::<Window>(window)
                    .unwrap()
                    .set_cursor_position(Some(Vec2::new(1200., 400.)));
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .clear();
                app.world_mut().run_system_once(guard).unwrap();
                assert!(
                    canvas_cursor(
                        app.world().get::<Window>(window).unwrap(),
                        app.world().resource::<Session>()
                    )
                    .is_none()
                );
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .release(button);
                app.world_mut().run_system_once(guard).unwrap();
                assert!(app.world().resource::<Session>().ui.overlay_owned);
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .clear();
                app.world_mut().run_system_once(guard).unwrap();
                assert!(
                    canvas_cursor(
                        app.world().get::<Window>(window).unwrap(),
                        app.world().resource::<Session>()
                    )
                    .is_some()
                );
            }
            let popup = app.world_mut().spawn((EditorPopover, Hovered(true))).id();
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(Vec2::new(1200., 400.)));
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.world_mut().run_system_once(guard).unwrap();
            assert!(
                canvas_cursor(
                    app.world().get::<Window>(window).unwrap(),
                    app.world().resource::<Session>()
                )
                .is_none()
            );
            app.world_mut().entity_mut(popup).insert(Hovered(false));
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .clear();
            app.world_mut().run_system_once(guard).unwrap();
            assert!(app.world().resource::<Session>().ui.overlay_owned);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            app.world_mut().run_system_once(guard).unwrap();
            assert!(app.world().resource::<Session>().ui.overlay_owned);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .clear();
            app.world_mut().run_system_once(guard).unwrap();
            assert!(!app.world().resource::<Session>().ui.overlay_owned);
            assert_eq!(app.world().resource::<Session>().selected, Some(1));
            app.world_mut().resource_mut::<PointerOwner>().0 = true;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
            app.world_mut().run_system_once(guard).unwrap();
            assert!(!app.world().resource::<PointerOwner>().0);
            assert!(!app.world().resource::<Session>().ui.overlay_owned);
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &zone);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
    }
}
