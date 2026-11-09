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
