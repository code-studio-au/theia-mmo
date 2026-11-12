//! Dock gestures own input through release and never mutate authoring data.
use super::*;

fn fixture() -> (App, Entity) {
    let zone = Zone::default();
    let session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        PathBuf::new(),
        Some(zone),
    );
    let mut app = App::new();
    app.insert_resource(session)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, resize);
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            resolution: (1440, 900).into(),
            ..default()
        })
        .id();
    (app, window)
}
fn point(app: &mut App, window: Entity, x: f32, y: f32) {
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(x, y)));
}
fn press(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
}
#[test]
fn split_and_width_resize_cancel_and_release_preserve_source_and_own_world_input() {
    for edge in [Edge::Width, Edge::Split] {
        let (mut app, window) = fixture();
        let source = app.world().resource::<Session>().editor.zone().clone();
        let layout = &app.world().resource::<Session>().ui;
        let initial = [layout.width, layout.explorer_fraction];
        let start = match edge {
            Edge::Width => Vec2::new(layout.canvas_left(), 300.),
            Edge::Split => Vec2::new(sidebar::WIDTH + 100., layout.properties_top(900.)),
            Edge::Assets => unreachable!(),
        };
        point(&mut app, window, start.x, start.y);
        press(&mut app);
        assert!(app.world().resource::<Session>().ui.input_owned);
        point(&mut app, window, start.x + 100., start.y + 100.);
        app.update();
        let layout = &app.world().resource::<Session>().ui;
        assert_ne!([layout.width, layout.explorer_fraction], initial);
        assert!(
            canvas_cursor(
                app.world().get::<Window>(window).unwrap(),
                app.world().resource::<Session>()
            )
            .is_none()
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        let layout = &app.world().resource::<Session>().ui;
        assert_eq!([layout.width, layout.explorer_fraction], initial);
        assert!(layout.input_owned);
        assert!(layout.drag.is_none());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        point(&mut app, window, start.x, start.y);
        press(&mut app);
        point(&mut app, window, start.x + 80., start.y + 80.);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<Session>().ui.input_owned);
        assert!(app.world().resource::<Session>().ui.drag.is_none());
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.update();
        assert!(!app.world().resource::<Session>().ui.input_owned);
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), (0, 0));
        assert!(!session.is_dirty());
    }
}
#[test]
fn popup_gestures_cannot_start_either_panel_divider() {
    for edge in [Edge::Width, Edge::Split] {
        let (mut app, window) = fixture();
        let layout = &app.world().resource::<Session>().ui;
        let initial = [layout.width, layout.explorer_fraction];
        let start = match edge {
            Edge::Width => Vec2::new(layout.canvas_left(), 300.),
            Edge::Split => Vec2::new(sidebar::WIDTH + 100., layout.properties_top(900.)),
            Edge::Assets => unreachable!(),
        };
        point(&mut app, window, start.x, start.y);
        app.world_mut().resource_mut::<Session>().ui.overlay_owned = true;
        press(&mut app);
        point(&mut app, window, start.x + 90., start.y + 90.);
        app.update();
        let layout = &app.world().resource::<Session>().ui;
        assert!(layout.drag.is_none());
        assert_eq!([layout.width, layout.explorer_fraction], initial);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.world_mut().resource_mut::<Session>().ui.overlay_owned = false;
        point(&mut app, window, start.x, start.y);
        press(&mut app);
        assert!(app.world().resource::<Session>().ui.drag.is_some());
    }
}

#[test]
fn status_bar_cannot_start_dock_or_world_gestures() {
    let (mut app, window) = fixture();
    let left = app.world().resource::<Session>().ui.canvas_left();
    for x in [left, left + 100.] {
        point(&mut app, window, x, 900. - status_bar::HEIGHT / 2.);
        press(&mut app);
        let session = app.world().resource::<Session>();
        assert!(session.ui.drag.is_none());
        assert!(canvas_cursor(app.world().get::<Window>(window).unwrap(), session).is_none());
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
    }
}

#[test]
fn resize_feedback_respects_menu_scrollbar_and_window_ownership() {
    use bevy::ui_widgets::{MenuFocusState, MenuPopup, Scrollbar};
    use bevy::window::{CursorIcon, SystemCursorIcon};
    let zone = Zone::default();
    let mut app = App::new();
    app.insert_resource(Session::new(
        Editor::new(zone.clone()).unwrap(),
        "unused.json".into(),
        Some(zone.clone()),
    ))
    .init_resource::<ButtonInput<MouseButton>>()
    .init_resource::<ButtonInput<KeyCode>>()
    .add_systems(Startup, setup)
    .add_zone_systems((resize, apply).chain());
    let window = app
        .world_mut()
        .spawn(Window {
            focused: true,
            resolution: (3440, 1440).into(),
            ..default()
        })
        .id();
    app.update();
    let grip = app
        .world_mut()
        .query::<(Entity, &Grip)>()
        .iter(app.world())
        .find(|(_, grip)| grip.0 == Edge::Width)
        .unwrap()
        .0;
    app.world_mut().entity_mut(grip).insert(Hovered(true));
    point(&mut app, window, CANVAS_LEFT, 350.);
    app.update();
    let resize_cursor = CursorIcon::System(SystemCursorIcon::ColResize);
    assert_eq!(app.world().get::<CursorIcon>(window), Some(&resize_cursor));
    let popup = app
        .world_mut()
        .spawn((MenuPopup::default(), MenuFocusState::Open))
        .id();
    point(&mut app, window, CANVAS_LEFT, 350.);
    press(&mut app);
    assert!(!app.world().resource::<Session>().ui.input_owned);
    assert_eq!(
        app.world().get::<CursorIcon>(window),
        Some(&CursorIcon::default())
    );
    assert_eq!(
        app.world().get::<BackgroundColor>(grip).unwrap().0,
        Color::NONE
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .entity_mut(popup)
        .insert(MenuFocusState::Closed);
    let scrollbar = app
        .world_mut()
        .spawn((
            Scrollbar {
                target: grip,
                orientation: bevy::ui_widgets::ControlOrientation::Vertical,
                min_thumb_length: 24.,
            },
            Hovered(true),
        ))
        .id();
    point(&mut app, window, CANVAS_LEFT, 350.);
    press(&mut app);
    assert!(!app.world().resource::<Session>().ui.input_owned);
    assert_eq!(
        app.world().get::<CursorIcon>(window),
        Some(&CursorIcon::default())
    );
    assert_eq!(
        app.world().get::<BackgroundColor>(grip).unwrap().0,
        Color::NONE
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut().entity_mut(scrollbar).insert(Hovered(false));
    app.update();
    assert_eq!(app.world().get::<CursorIcon>(window), Some(&resize_cursor));
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.update();
    assert_eq!(
        app.world().get::<CursorIcon>(window),
        Some(&CursorIcon::default())
    );
    assert_eq!(
        app.world().get::<BackgroundColor>(grip).unwrap().0,
        Color::NONE
    );
    let session = app.world().resource::<Session>();
    assert_eq!(session.ui.width, workspace::INSPECTOR_WIDTH);
    assert_eq!(session.editor.zone(), &zone);
    assert_eq!(session.editor.revision(), 0);
    assert_eq!(session.editor.history(), (0, 0));
    assert!(!session.is_dirty());
}

#[test]
fn embedded_viewport_projection_and_picking_round_trip_after_resizing_and_dpi_changes() {
    use bevy::camera::{ComputedCameraValues, RenderTargetInfo};
    for scale in [1., 1.5, 2.] {
        for width in [300., 600., 420.] {
            let mut session = Session::new(
                Editor::new(Zone::default()).unwrap(),
                "unused.json".into(),
                None,
            );
            session.ui.width = width;
            let mut window = Window::default();
            window
                .resolution
                .set_physical_resolution((3440. * scale) as u32, (1440. * scale) as u32);
            window.resolution.set_scale_factor_override(Some(scale));
            let size = Vec2::new(
                3440. - sidebar::WIDTH - width,
                1440. - workspace::HEADER - status_bar::HEIGHT,
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
                            && screen.y < 1440. - status_bar::HEIGHT
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
            window.set_cursor_position(Some(Vec2::new(sidebar::WIDTH + width / 2., 300.)));
            assert!(canvas_cursor(&window, &session).is_none());
        }
    }
}

#[test]
fn asset_split_resizes_cancels_and_reflows_viewport_without_changing_left_stack() {
    let (mut app, window) = fixture();
    app.world_mut().resource_mut::<Session>().ui.assets_open = true;
    let before = app.world().resource::<Session>().editor.zone().clone();
    let ui = &app.world().resource::<Session>().ui;
    let height = ui.assets_height;
    let left = ui.width;
    let split = ui.explorer_fraction;
    let top = 900. - ui.canvas_bottom(900., true);
    point(&mut app, window, 1000., top);
    press(&mut app);
    point(&mut app, window, 1000., top - 80.);
    app.update();
    let ui = &app.world().resource::<Session>().ui;
    assert_eq!(ui.assets_height, height + 80.);
    assert_eq!((ui.width, ui.explorer_fraction), (left, split));
    assert_eq!(ui.canvas_bottom(900., false), status_bar::HEIGHT);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    app.update();
    assert_eq!(app.world().resource::<Session>().ui.assets_height, height);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
}
