//! Bounded size controls use the retained exact form and shared command engine.
use super::yaw_tests::{change, fixture, key};
use super::*;

fn slider(app: &mut App) -> Entity {
    app.world_mut()
        .query::<(Entity, &Control)>()
        .iter(app.world())
        .find(|(_, control)| **control == Control::ValueSlider)
        .unwrap()
        .0
}
#[test]
fn scale_radius_and_both_guide_width_forms_preserve_other_fields_and_undo_once() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for (target, field, desired, expected_value, range) in [
            (1, 4, 1.37, 1.25, [0.25, 8.]),
            (2, 4, 3.37, 3.25, [0.25, 64.]),
            (3, 3, 6.12, 6., [0.25, 32.]),
            (5, 0, 6.12, 6., [0.25, 32.]),
        ] {
            let (mut app, _) = fixture(origin, target, false);
            let slider = slider(&mut app);
            assert_eq!(
                app.world().get::<SliderRange>(slider).unwrap().start(),
                range[0]
            );
            assert_eq!(
                app.world().get::<SliderRange>(slider).unwrap().end(),
                range[1]
            );
            assert_eq!(app.world().get::<SliderStep>(slider).unwrap().0, 0.25);
            let before = app.world().resource::<Session>().editor.zone().clone();
            let mut fields = app
                .world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields
                .clone();
            app.world_mut().trigger(bevy::ui_widgets::SetSliderValue {
                entity: slider,
                change: bevy::ui_widgets::SliderValueChange::Absolute(desired),
            });
            app.update();
            change(&mut app, slider, desired, false);
            change(&mut app, slider, desired, true);
            fields[field] = expected_value.to_string();
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .transform_edit
                    .as_ref()
                    .unwrap()
                    .fields,
                fields
            );
            assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
            assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(slider, FocusCause::Navigated);
            app.update();
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(slider));
            key(&mut app, KeyCode::Enter);
            let mut expected = before.clone();
            match target {
                1 => expected.plants[0].scale = expected_value,
                2 => expected.markers[0].radius_metres = expected_value,
                _ => expected.paths[0].width_metres = expected_value,
            }
            assert_eq!(app.world().resource::<Session>().editor.zone(), &expected);
            assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
            app.world_mut()
                .resource_mut::<Session>()
                .editor
                .execute(Command::Undo {
                    expected_revision: 3,
                })
                .unwrap();
            assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
            app.world_mut()
                .resource_mut::<Session>()
                .editor
                .execute(Command::Redo {
                    expected_revision: 4,
                })
                .unwrap();
            assert_eq!(app.world().resource::<Session>().editor.zone(), &expected);
        }
    }
}
#[test]
fn value_controls_retain_exact_noops_and_refuse_hidden_or_invalid_events() {
    for target in 0..6 {
        let (mut app, _) = fixture([0.; 3], target, true);
        let slider = slider(&mut app);
        let visible = matches!(target, 1 | 2 | 3 | 5);
        assert_eq!(
            app.world().get::<InteractionDisabled>(slider).is_none(),
            visible
        );
        let before = app.world().resource::<Session>().editor.zone().clone();
        let fields = app
            .world()
            .resource::<Session>()
            .transform_edit
            .as_ref()
            .unwrap()
            .fields
            .clone();
        let initial = app.world().get::<SliderValue>(slider).unwrap().0;
        change(&mut app, slider, initial, true);
        if visible {
            change(&mut app, slider, 2., false);
            change(&mut app, slider, initial, true);
        } else {
            change(&mut app, slider, 2., true);
        }
        for value in [f32::NAN, f32::INFINITY, -1., 65.] {
            change(&mut app, slider, value, true);
        }
        assert_eq!(
            app.world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields,
            fields
        );
        key(&mut app, KeyCode::Enter);
        assert!(app.world().resource::<Session>().transform_edit.is_none());
        assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
        assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    }
    let (mut app, _) = fixture([0.; 3], 1, true);
    let slider = slider(&mut app);
    let before = app.world().resource::<Session>().editor.zone().clone();
    let mut fields = app.world_mut().query::<(&Field, &mut EditableText)>();
    fields
        .iter_mut(app.world_mut())
        .find(|(field, _)| field.0 == 4)
        .unwrap()
        .1
        .editor_mut()
        .set_text("NaN");
    key(&mut app, KeyCode::Enter);
    assert!(app.world().resource::<Session>().transform_edit.is_some());
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    change(&mut app, slider, 1.25, true);
    key(&mut app, KeyCode::Enter);
    let mut expected = before;
    expected.plants[0].scale = 1.25;
    assert_eq!(app.world().resource::<Session>().editor.zone(), &expected);
    assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
}
