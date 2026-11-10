//! Real CPU layout and ECS pointer dispatch; no renderer, window backend or OS input.
use super::*;
use bevy::input_focus::InputFocus;

fn resize(app: &mut App, window: Entity, height: u32) {
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set_physical_resolution(3440, height);
    for mut camera in app
        .world_mut()
        .query::<&mut Camera>()
        .iter_mut(app.world_mut())
    {
        camera
            .computed
            .target_info
            .as_mut()
            .unwrap()
            .physical_size
            .y = height;
        camera.viewport.as_mut().unwrap().physical_size.y = height;
    }
    settle(app);
}
fn settle(app: &mut App) {
    for _ in 0..3 {
        app.update();
    }
}
fn pointer_wheel(app: &mut App, pane: Entity, window: Entity, y: f32) {
    app.world_mut()
        .trigger(bevy::picking::events::PointerScroll {
            entity: pane,
            pointer: bevy::picking::events::Pointer::new(
                bevy::picking::pointer::PointerId::Mouse,
                bevy::picking::pointer::Location {
                    target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(
                        window,
                    ))
                    .normalize(Some(window))
                    .unwrap(),
                    position: Vec2::new(150., 300.),
                },
            ),
            hit: bevy::picking::backend::HitData::new(window, 0., None, None),
            unit: bevy::input::mouse::MouseScrollUnit::Line,
            x: 0.,
            y,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta
        .y = y;
    app.update();
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta
        .y = 0.;
    settle(app);
}
fn offset(app: &App, pane: Entity) -> f32 {
    app.world().get::<ScrollPosition>(pane).unwrap().y
}
fn fixture(origin: [f64; 3]) -> (App, Entity, Entity) {
    let (mut app, window) = tests::app(origin);
    app.init_resource::<ButtonInput<MouseButton>>()
        .add_plugins(scrolling::plugin);
    crate::headless_ui::add_layout(&mut app);
    resize(&mut app, window, 420);
    let pane = app
        .world_mut()
        .query::<(Entity, &scrolling::Pane)>()
        .iter(app.world())
        .find(|(_, scope)| matches!(scope, scrolling::Pane::Browser))
        .unwrap()
        .0;
    (app, window, pane)
}
#[test]
fn short_browser_scrolls_and_reveals_selection_without_paging_twice_or_stealing_focus() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let (mut app, window, pane) = fixture(origin);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let focus = app.world().resource::<InputFocus>().get();
        let node = app.world().get::<ComputedNode>(pane).unwrap();
        assert!(node.content_size().y - node.size().y > 24.);
        pointer_wheel(&mut app, pane, window, -1.);
        assert_eq!(offset(&app, pane), 24.);
        assert_eq!(app.world().resource::<Content>().page, 0);
        assert_eq!(
            app.world()
                .resource::<Content>()
                .last_scroll
                .as_ref()
                .unwrap()["routed_to_pane"],
            true
        );
        for selected in [17, 34, 43] {
            app.world_mut().resource_mut::<Session>().selected = Some(selected);
            settle(&mut app);
            let content = app.world().resource::<Content>();
            let row = content
                .entries
                .iter()
                .position(|entry| entry.target == Target::Object(selected))
                .unwrap()
                % ROWS;
            let entity = app
                .world_mut()
                .query::<(Entity, &Control)>()
                .iter(app.world())
                .find(|(_, control)| **control == Control::Row(row))
                .unwrap()
                .0;
            let node = app.world().get::<ComputedNode>(entity).unwrap();
            let pose = app
                .world()
                .get::<bevy::ui::UiGlobalTransform>(entity)
                .unwrap();
            assert!(node.size().y >= 24.);
            assert!(pose.affine().translation.y - node.size().y / 2. >= workspace::HEADER);
            assert!(pose.affine().translation.y + node.size().y / 2. <= 420. - 52.);
            assert_eq!(app.world().resource::<InputFocus>().get(), focus);
        }
        let before = offset(&app, pane);
        let page = app.world().resource::<Content>().page;
        pointer_wheel(&mut app, pane, window, 1.);
        assert_eq!(offset(&app, pane), (before - 24.).max(0.));
        assert_eq!(app.world().resource::<Content>().page, page);
        settle(&mut app);
        assert_eq!(offset(&app, pane), (before - 24.).max(0.));
        // Full-height browsing keeps the established bounded wheel paging.
        resize(&mut app, window, 1440);
        assert_eq!(offset(&app, pane), 0.);
        pointer_wheel(&mut app, pane, window, 1.);
        assert_eq!(app.world().resource::<Content>().page, page - 1);
        assert_eq!(offset(&app, pane), 0.);
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), (0, 1));
        assert_eq!(session.editor.revision(), 2);
        assert!(!session.is_dirty());
    }
}
#[test]
fn browser_scroll_guards_gestures_focus_and_modal_edits_and_resets_filtered_pages() {
    let (mut app, window, pane) = fixture([0.; 3]);
    pointer_wheel(&mut app, pane, window, -1.);
    assert_eq!(offset(&app, pane), 24.);
    app.world_mut().resource_mut::<Session>().ui.input_owned = true;
    pointer_wheel(&mut app, pane, window, -1.);
    assert_eq!(offset(&app, pane), 24.);
    app.world_mut().resource_mut::<Session>().ui.input_owned = false;
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    pointer_wheel(&mut app, pane, window, -1.);
    assert_eq!(offset(&app, pane), 24.);
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.world_mut().resource_mut::<workspace::Requests>().find = true;
    settle(&mut app);
    assert!(app.world().resource::<Session>().name_edit.is_some());
    pointer_wheel(&mut app, pane, window, -1.);
    assert_eq!(offset(&app, pane), 24.);
    app.world_mut().resource_mut::<Session>().name_edit = None;
    app.world_mut()
        .resource_mut::<Content>()
        .set_query("Élan")
        .unwrap();
    settle(&mut app);
    assert_eq!(offset(&app, pane), 0.);
    assert_eq!(app.world().resource::<Content>().entries.len(), 1);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    assert!(!app.world().resource::<Session>().is_dirty());
}
