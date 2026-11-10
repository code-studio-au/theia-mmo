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
        sidebar::WIDTH + self.browser + self.inspector
    }
    pub fn window_to_viewport(&self, point: Vec2, scale: f32) -> Vec2 {
        (point - Vec2::new(self.canvas_left(), workspace::HEADER)) * scale
    }
    pub fn viewport_to_window(&self, point: Vec2, scale: f32) -> Vec2 {
        point / scale + Vec2::new(self.canvas_left(), workspace::HEADER)
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum Region {
    Browser,
    Inspector,
    Footer,
    Canvas,
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
    scrollbars: Query<&Hovered, With<bevy::ui_widgets::Scrollbar>>,
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
    } else if allowed
        && mouse.just_pressed(MouseButton::Left)
        && !scrollbars.iter().any(|hovered| hovered.0)
    {
        let Some(cursor) = cursor else { return };
        if cursor.y <= workspace::HEADER || cursor.y >= window.height() - 52. {
            return;
        }
        let edge = if (cursor.x - sidebar::WIDTH - layout.browser).abs() <= GRIP / 2. {
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
            Region::Browser => (sidebar::WIDTH, Some(session.ui.browser)),
            Region::Inspector => (
                sidebar::WIDTH + session.ui.browser,
                Some(session.ui.inspector),
            ),
            Region::Footer => (session.ui.canvas_left() + 12., None),
            Region::Canvas => (session.ui.canvas_left(), None),
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
            Edge::Browser => sidebar::WIDTH + session.ui.browser,
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
        press(&mut app, Vec2::new(sidebar::WIDTH + 300., 350.));
        assert!(app.world().resource::<Session>().ui.input_owned);
        cursor(&mut app, Vec2::new(sidebar::WIDTH + 450., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.browser, 450.);
        let mut windows = app.world_mut().query::<&Window>();
        let w = windows.single(app.world()).unwrap();
        let s = app.world().resource::<Session>();
        assert!(canvas_cursor(w, s).is_none());
        release(&mut app);
        press(&mut app, Vec2::new(sidebar::WIDTH + 810., 350.));
        cursor(&mut app, Vec2::new(2000., 350.));
        app.update();
        assert_eq!(app.world().resource::<Session>().ui.inspector, 600.);
        release(&mut app);
        press(&mut app, Vec2::new(sidebar::WIDTH + 450., 350.));
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
        press(&mut app, Vec2::new(sidebar::WIDTH + 1050., 350.));
        cursor(&mut app, Vec2::new(sidebar::WIDTH + 800., 350.));
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
    #[test]
    fn embedded_viewport_projection_and_picking_round_trip_after_resizing_and_dpi_changes() {
        use bevy::camera::{ComputedCameraValues, RenderTargetInfo};
        for scale in [1., 1.5, 2.] {
            for (browser, inspector) in [(300., 300.), (480., 600.), (420., 480.)] {
                let mut session = Session::new(
                    Editor::new(Zone::default()).unwrap(),
                    "unused.json".into(),
                    None,
                );
                session.ui.browser = browser;
                session.ui.inspector = inspector;
                let mut window = Window::default();
                window
                    .resolution
                    .set_physical_resolution((3440. * scale) as u32, (1440. * scale) as u32);
                window.resolution.set_scale_factor_override(Some(scale));
                let size = Vec2::new(
                    3440. - sidebar::WIDTH - browser - inspector,
                    1440. - workspace::HEADER - 52.,
                ) * scale;
                for preset in [
                    camera_views::Preset::Perspective,
                    camera_views::Preset::Top,
                    camera_views::Preset::Front,
                    camera_views::Preset::Right,
                ] {
                    let mut view = View {
                        target: Vec3::new(16., 3., -8.),
                        yaw: 0.6,
                        pitch: 0.8,
                        distance: 32.,
                        preset,
                        show_grid: true,
                    };
                    preset.select(&mut view);
                    let mut projection = Projection::default();
                    camera_views::projection(&view, false, &mut projection);
                    projection.update(size.x, size.y);
                    let camera = Camera {
                        computed: ComputedCameraValues {
                            clip_from_view: projection.get_clip_from_view(),
                            target_info: Some(RenderTargetInfo {
                                physical_size: size.as_uvec2(),
                                scale_factor: 1.,
                            }),
                            ..default()
                        },
                        ..default()
                    };
                    let transform = camera_views::transform(&view);
                    let global = GlobalTransform::from(transform);
                    for offset in [Vec3::ZERO, Vec3::new(3., 2., 0.), Vec3::new(-4., -1., 0.)] {
                        let point = view.target + transform.rotation * offset;
                        let image = camera.world_to_viewport(&global, point).unwrap();
                        let screen = session.ui.viewport_to_window(image, scale);
                        assert!(
                            screen.x > session.ui.canvas_left()
                                && screen.x < 3440.
                                && screen.y > workspace::HEADER
                                && screen.y < 1388.
                        );
                        window.set_cursor_position(Some(screen));
                        let local = canvas_cursor(&window, &session).unwrap();
                        assert!(local.abs_diff_eq(image, 0.001));
                        let ray = camera.viewport_to_world(&global, local).unwrap();
                        let normal = transform.rotation * Vec3::Z;
                        let distance = (point - ray.origin).dot(normal) / ray.direction.dot(normal);
                        assert!(
                            ray.get_point(distance).abs_diff_eq(point, 0.002),
                            "{preset:?} / scale {scale}"
                        );
                    }
                }
                window.set_cursor_position(Some(Vec2::new(browser + inspector / 2., 300.)));
                assert!(canvas_cursor(&window, &session).is_none());
            }
        }
    }
}
