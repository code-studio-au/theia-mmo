//! InputPlugin messages stay in the test ECS; no OS keyboard/display connection.
use super::yaw_tests::fixture_with_input;
use super::*;
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};

fn send(app: &mut App, code: KeyCode, logical: Key, state: ButtonState) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    app.world_mut().write_message(KeyboardInput {
        key_code: code,
        logical_key: logical,
        state,
        text: None,
        repeat: false,
        window,
    });
    app.update();
}
fn press(app: &mut App, code: KeyCode, logical: Key) {
    send(app, code, logical.clone(), ButtonState::Pressed);
    send(app, code, logical, ButtonState::Released);
}
#[test]
fn tab_wraps_only_visible_fields_and_sliders_through_actual_input_dispatch() {
    for target in 0..6 {
        let (mut app, yaw) = fixture_with_input([0.; 3], target, true, true);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let fields = app
            .world()
            .resource::<Session>()
            .transform_edit
            .as_ref()
            .unwrap()
            .fields
            .len();
        let mut sequence: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Field)>()
            .iter(app.world())
            .filter(|(_, field)| field.0 < fields)
            .map(|(entity, field)| (field.0, entity))
            .collect();
        sequence.sort_by_key(|(index, _)| *index);
        let mut sequence: Vec<_> = sequence.into_iter().map(|(_, entity)| entity).collect();
        if target < 3 {
            sequence.push(yaw);
        }
        if matches!(target, 1 | 2 | 3 | 5) {
            sequence.push(
                app.world_mut()
                    .query::<(Entity, &Control)>()
                    .iter(app.world())
                    .find(|(_, control)| **control == Control::ValueSlider)
                    .unwrap()
                    .0,
            );
        }
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            Some(sequence[0])
        );
        for entity in sequence.iter().skip(1).chain(std::iter::once(&sequence[0])) {
            press(&mut app, KeyCode::Tab, Key::Tab);
            assert_eq!(
                app.world().resource::<InputFocus>().get(),
                Some(*entity),
                "target {target}"
            );
        }
        send(
            &mut app,
            KeyCode::ShiftLeft,
            Key::Shift,
            ButtonState::Pressed,
        );
        press(&mut app, KeyCode::Tab, Key::Tab);
        send(
            &mut app,
            KeyCode::ShiftLeft,
            Key::Shift,
            ButtonState::Released,
        );
        assert_eq!(
            app.world().resource::<InputFocus>().get(),
            sequence.last().copied()
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    }
}
#[test]
fn arrow_home_end_keep_slider_focus_and_endpoints_without_committing_source() {
    for (target, after_arrow) in [(0, -120.), (1, 1.5), (2, 12.5), (3, 4.25), (5, 4.25)] {
        let (mut app, yaw) = fixture_with_input([0.; 3], target, true, true);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let slider = if target == 0 {
            yaw
        } else {
            app.world_mut()
                .query::<(Entity, &Control)>()
                .iter(app.world())
                .find(|(_, control)| **control == Control::ValueSlider)
                .unwrap()
                .0
        };
        let field = match target {
            0 | 3 => 3,
            5 => 0,
            _ => 4,
        };
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(slider, FocusCause::Navigated);
        press(&mut app, KeyCode::ArrowRight, Key::ArrowRight);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields[field]
                .parse::<f32>()
                .unwrap(),
            after_arrow
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(slider));
        let low = app.world().get::<SliderRange>(slider).unwrap().start();
        let high = app.world().get::<SliderRange>(slider).unwrap().end();
        press(&mut app, KeyCode::Home, Key::Home);
        assert_eq!(app.world().get::<SliderValue>(slider).unwrap().0, low);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields[field]
                .parse::<f32>()
                .unwrap(),
            low
        );
        press(&mut app, KeyCode::End, Key::End);
        assert_eq!(app.world().get::<SliderValue>(slider).unwrap().0, high);
        assert_eq!(
            app.world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields[field]
                .parse::<f32>()
                .unwrap(),
            high
        );
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(slider));
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        press(&mut app, KeyCode::Escape, Key::Escape);
        assert!(app.world().resource::<Session>().transform_edit.is_none());
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    }
}

#[test]
fn tab_reveals_exact_attributes_in_a_short_inspector_without_editing_source_or_redo() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in 0..6 {
            let (mut app, _) = fixture_with_input(origin, target, true, true);
            app.add_plugins(scrolling::plugin);
            crate::headless_ui::add_layout(&mut app);
            let window = app
                .world_mut()
                .query_filtered::<Entity, With<Window>>()
                .single(app.world())
                .unwrap();
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .resolution
                .set_physical_resolution(3440, 420);
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
                    .y = 420;
                camera.viewport.as_mut().unwrap().physical_size.y = 420;
            }
            for _ in 0..3 {
                app.update();
            }
            let source = app.world().resource::<Session>().editor.zone().clone();
            let dirty = app.world().resource::<Session>().is_dirty();
            let original = app
                .world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields
                .clone();
            let pane = app
                .world_mut()
                .query_filtered::<Entity, With<scrolling::Pane>>()
                .single(app.world())
                .unwrap();
            let mut focused = Vec::new();
            loop {
                let entity = app.world().resource::<InputFocus>().get().unwrap();
                if focused.contains(&entity) {
                    break;
                }
                focused.push(entity);
                let node = app.world().get::<ComputedNode>(entity).unwrap();
                let pose = app
                    .world()
                    .get::<bevy::ui::UiGlobalTransform>(entity)
                    .unwrap();
                let first = pose.affine().translation.y - node.size().y / 2.;
                let last = pose.affine().translation.y + node.size().y / 2.;
                assert!(
                    first >= workspace::HEADER && last <= 420.,
                    "target {target}, focus {entity:?}: {first}..{last}"
                );
                press(&mut app, KeyCode::Tab, Key::Tab);
                for _ in 0..2 {
                    app.update();
                }
            }
            assert!(focused.len() >= original.len());
            let entity = app.world().resource::<InputFocus>().get().unwrap();
            let node = app.world().get::<ComputedNode>(entity).unwrap();
            let pose = app
                .world()
                .get::<bevy::ui::UiGlobalTransform>(entity)
                .unwrap();
            assert!(pose.affine().translation.y - node.size().y / 2. >= workspace::HEADER);
            assert!(pose.affine().translation.y + node.size().y / 2. <= 420.);
            let computed = app.world().get::<ComputedNode>(pane).unwrap();
            let limit = ((computed.content_size().y - computed.size().y)
                * computed.inverse_scale_factor())
            .max(0.);
            let expected = (app.world().get::<ScrollPosition>(pane).unwrap().y + 24.).min(limit);
            app.world_mut()
                .trigger(bevy::picking::events::PointerScroll {
                    entity: pane,
                    pointer: bevy::picking::events::Pointer::new(
                        bevy::picking::pointer::PointerId::Mouse,
                        bevy::picking::pointer::Location {
                            target: bevy::camera::RenderTarget::Window(
                                bevy::window::WindowRef::Entity(window),
                            )
                            .normalize(Some(window))
                            .unwrap(),
                            position: Vec2::new(500., 300.),
                        },
                    ),
                    hit: bevy::picking::backend::HitData::new(window, 0., None, None),
                    unit: bevy::input::mouse::MouseScrollUnit::Line,
                    x: 0.,
                    y: -1.,
                    phase: bevy::input::touch::TouchPhase::Moved,
                });
            for _ in 0..3 {
                app.update();
            }
            assert_eq!(app.world().get::<ScrollPosition>(pane).unwrap().y, expected);
            let session = app.world().resource::<Session>();
            assert_eq!(session.transform_edit.as_ref().unwrap().fields, original);
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.history(), (0, 1));
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.is_dirty(), dirty);
            press(&mut app, KeyCode::Escape, Key::Escape);
            let session = app.world().resource::<Session>();
            assert!(session.transform_edit.is_none());
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
    }
}
