//! Bounded inspector/browser resizing following Sprinkles' panel interaction.
//! UI preferences only; coordinates and validated authoring history are separate.
use super::*;

const GRIP: f32 = 12.;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Browser,
    Inspector,
}
#[derive(Clone, Copy)]
struct Resize {
    edge: Edge,
    start: Vec2,
    original: [f32; 2],
}
pub(super) struct Layout {
    pub browser: f32,
    pub inspector: f32,
    drag: Option<Resize>,
    pub input_owned: bool,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            browser: outliner::WIDTH,
            inspector: workspace::INSPECTOR_WIDTH,
            drag: None,
            input_owned: false,
        }
    }
}
impl Layout {
    pub fn canvas_left(&self) -> f32 {
        self.browser + self.inspector
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum Region {
    Browser,
    Inspector,
    Footer,
}
#[derive(Component)]
pub(super) struct Grip(Edge);

pub(super) fn setup(mut commands: Commands) {
    for edge in [Edge::Browser, Edge::Inspector] {
        commands.spawn((
            Grip(edge),
            GlobalZIndex(16),
            Hovered::default(),
            Node {
                position_type: PositionType::Absolute,
                top: px(workspace::HEADER),
                bottom: px(52),
                width: px(GRIP),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
    }
}

pub(super) fn resize(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
) {
    let Ok(window) = windows.single() else { return };
    let allowed = window.focused
        && !session.has_modal()
        && session.play.is_none()
        && session.drag.is_none()
        && session.stroke.is_none()
        && session.path_draft.is_none();
    let cursor = window.cursor_position();
    let layout = &mut session.ui;
    layout.input_owned = false;
    if let Some(drag) = layout.drag {
        layout.input_owned = true;
        if !allowed || cursor.is_none() || keys.just_pressed(KeyCode::Escape) {
            [layout.browser, layout.inspector] = drag.original;
            layout.drag = None;
            return;
        }
        let delta = cursor.unwrap().x - drag.start.x;
        match drag.edge {
            Edge::Browser => layout.browser = (drag.original[0] + delta).clamp(300., 480.),
            Edge::Inspector => layout.inspector = (drag.original[1] + delta).clamp(300., 600.),
        }
        if !mouse.pressed(MouseButton::Left) {
            layout.drag = None;
        }
    } else if allowed && mouse.just_pressed(MouseButton::Left) {
        let Some(cursor) = cursor else { return };
        if cursor.y <= workspace::HEADER || cursor.y >= window.height() - 52. {
            return;
        }
        let edge = if (cursor.x - layout.browser).abs() <= GRIP / 2. {
            Some(Edge::Browser)
        } else if (cursor.x - layout.canvas_left()).abs() <= GRIP / 2. {
            Some(Edge::Inspector)
        } else {
            None
        };
        if let Some(edge) = edge {
            layout.drag = Some(Resize {
                edge,
                start: cursor,
                original: [layout.browser, layout.inspector],
            });
            layout.input_owned = true;
        }
    }
}

pub(super) fn apply(
    mut commands: Commands,
    windows: Query<(Entity, &Window, Option<&bevy::window::CursorIcon>)>,
    session: Res<Session>,
    mut regions: Query<(&Region, &mut Node)>,
    mut grips: Query<(&Grip, &Hovered, &mut Node, &mut BackgroundColor), Without<Region>>,
) {
    for (region, mut node) in &mut regions {
        let (left, width) = match region {
            Region::Browser => (0., Some(session.ui.browser)),
            Region::Inspector => (session.ui.browser, Some(session.ui.inspector)),
            Region::Footer => (session.ui.canvas_left() + 12., None),
        };
        if node.left != px(left) {
            node.left = px(left);
        }
        if let Some(width) = width
            && node.width != px(width)
        {
            node.width = px(width);
        }
    }
    let mut over_grip = false;
    for (grip, hovered, mut node, mut background) in &mut grips {
        over_grip |= hovered.0;
        node.display = if session.play.is_some() {
            Display::None
        } else {
            Display::Flex
        };
        let edge = match grip.0 {
            Edge::Browser => session.ui.browser,
            Edge::Inspector => session.ui.canvas_left(),
        };
        if node.left != px(edge - GRIP / 2.) {
            node.left = px(edge - GRIP / 2.);
        }
        background.0 = if session.ui.drag.is_some_and(|drag| drag.edge == grip.0) {
            theme::PRIMARY.with_alpha(0.5)
        } else if hovered.0 {
            theme::BORDER
        } else {
            Color::NONE
        };
    }
    if let Ok((entity, window, cursor)) = windows.single() {
        let resize_cursor =
            bevy::window::CursorIcon::System(bevy::window::SystemCursorIcon::EwResize);
        let active = window.focused
            && session.play.is_none()
            && !session.has_modal()
            && (over_grip || session.ui.drag.is_some());
        if active && cursor != Some(&resize_cursor) {
            commands.entity(entity).insert(resize_cursor);
        } else if !active && cursor == Some(&resize_cursor) {
            commands
                .entity(entity)
                .insert(bevy::window::CursorIcon::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cursor(app: &mut App, position: Vec2) {
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .set_cursor_position(Some(position));
    }
    fn press(app: &mut App, position: Vec2) {
        cursor(app, position);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear_just_pressed(MouseButton::Left);
    }
    fn release(app: &mut App) {
        let active = app.world().resource::<Session>().ui.drag.is_some();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.input_owned, active);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear_just_released(MouseButton::Left);
        app.update();
        assert!(!app.world().resource::<Session>().ui.input_owned);
    }
    #[test]
    fn resize_owns_the_entire_gesture_preserves_exact_redo_and_cancels_preferences() {
        let zone = Zone {
            origin: [128_000_000_000.125, 20.25, -128_000_000_000.5],
            spawn: [128_000_000_000.125, 20.35, -128_000_000_000.5],
            ..default()
        };
        let mut editor = Editor::new(zone.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Temporary".into(),
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let mut session = Session::new(editor, "unused.json".into(), Some(zone.clone()));
        session.palette = Some(Primitive::Box);
        let mut app = App::new();
        app.insert_resource(session)
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, resize);
        app.world_mut().spawn(Window {
            focused: true,
            resolution: (3440, 1440).into(),
            ..default()
        });
        press(&mut app, Vec2::new(300., 350.));
        assert!(app.world().resource::<Session>().ui.input_owned);
        cursor(&mut app, Vec2::new(450., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.browser, 450.);
        let mut windows = app.world_mut().query::<&Window>();
        let w = windows.single(app.world()).unwrap();
        let s = app.world().resource::<Session>();
        assert!(canvas_cursor(w, s).is_none());
        release(&mut app);
        press(&mut app, Vec2::new(810., 350.));
        cursor(&mut app, Vec2::new(2000., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.inspector, 600.);
        release(&mut app);
        press(&mut app, Vec2::new(450., 350.));
        cursor(&mut app, Vec2::new(100., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.browser, 300.);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.browser, 450.);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        release(&mut app);
        press(&mut app, Vec2::new(1050., 350.));
        cursor(&mut app, Vec2::new(800., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.inspector, 350.);
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .focused = false;
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.inspector, 600.);
        release(&mut app);
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &zone);
        assert_eq!(session.editor.history(), (0, 1));
        assert_eq!(session.editor.revision(), 2);
        assert_eq!(session.palette, Some(Primitive::Box));
        assert!(!session.is_dirty());
    }
}
