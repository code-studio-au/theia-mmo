//! Real Bevy field/slider observer checks without a window or input backend.
use super::*;

pub(super) fn fixture(origin: [f64; 3], target: u8, snap: bool) -> (App, Entity) {
    let mut seed = Editor::new(Zone::default()).unwrap();
    seed.execute(
        serde_json::from_slice(include_bytes!(
            "../../../content/zones/selection.commands.jsonl"
        ))
        .unwrap(),
    )
    .unwrap();
    let mut zone = seed.zone().clone();
    let shift = |position: [f64; 3]| std::array::from_fn(|i| position[i] + origin[i]);
    zone.origin = shift(zone.origin);
    zone.spawn = shift(zone.spawn);
    for object in &mut zone.objects {
        object.pose.position = shift(object.pose.position);
        object.pose.yaw_degrees = 225.123456789;
    }
    for plant in &mut zone.plants {
        plant.position = shift(plant.position);
        plant.yaw_degrees = 225.123456789;
        plant.scale = 1.234567891;
    }
    for marker in &mut zone.markers {
        marker.position = shift(marker.position);
        marker.yaw_degrees = 225.123456789;
        marker.radius_metres = 12.345678901;
        marker.definition = Some("quest.precision-fixture".into());
    }
    for path in &mut zone.paths {
        path.width_metres = 4.123456789;
        for point in &mut path.points {
            *point = shift(*point);
        }
    }
    let mut editor = Editor::new(zone.clone()).unwrap();
    editor
        .execute(Command::Apply {
            expected_revision: 0,
            edits: vec![Edit::RenameZone {
                name: "Temporary name".into(),
            }],
        })
        .unwrap();
    editor
        .execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
    let mut session = Session::new(editor, PathBuf::new(), Some(zone));
    session.snap = snap;
    match target {
        0 => session.selected = Some(1),
        1 => session.plant_selected = Some(1001),
        2 => session.marker_selected = Some(1000),
        3 => {
            session.path_selected = Some(1002);
            session.path_point = Some(0);
        }
        5 => session.path_selected = Some(1002),
        _ => {}
    }
    let mut app = App::new();
    app.add_plugins(bevy::ui_widgets::SliderPlugin);
    app.insert_resource(session)
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InputFocus>()
        .insert_resource(workspace::Requests {
            transform: target != 4,
            spawn_transform: target == 4,
            ..default()
        })
        .add_observer(slider_change)
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard)
        .add_systems(PostUpdate, (finish, slider_sync).chain());
    app.world_mut().spawn(Window {
        resolution: (3440, 1440).into(),
        focused: true,
        ..default()
    });
    app.update();
    let slider = app
        .world_mut()
        .query_filtered::<Entity, With<YawSlider>>()
        .single(app.world())
        .unwrap();
    (app, slider)
}
pub(super) fn change(app: &mut App, slider: Entity, value: f32, is_final: bool) {
    app.world_mut().trigger(ValueChange {
        source: slider,
        value,
        is_final,
    });
    app.update();
}
pub(super) fn key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
}
#[test]
fn slider_edits_pending_fields_and_applies_one_shared_transaction_for_each_yaw_kind() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in 0..3 {
            let (mut app, slider) = fixture(origin, target, target == 0);
            let before = app.world().resource::<Session>().editor.zone().clone();
            let original_fields = app
                .world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .fields
                .clone();
            assert_eq!(original_fields[3], "225.123456789");
            let value = if target == 0 { 135.1 } else { 12.6 };
            // Exercise Bevy's own controlled-widget event route as well as
            // intermediate/final notifications from a held drag.
            app.world_mut().trigger(bevy::ui_widgets::SetSliderValue {
                entity: slider,
                change: bevy::ui_widgets::SliderValueChange::Absolute(value),
            });
            app.update();
            change(&mut app, slider, value, false);
            change(&mut app, slider, value, true);
            let expected_yaw = if target == 0 { 135. } else { 13. };
            let mut expected_fields = original_fields;
            expected_fields[3] = expected_yaw.to_string();
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .transform_edit
                    .as_ref()
                    .unwrap()
                    .fields,
                expected_fields
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
                0 => expected.objects[0].pose.yaw_degrees = expected_yaw,
                1 => expected.plants[0].yaw_degrees = expected_yaw,
                _ => expected.markers[0].yaw_degrees = expected_yaw,
            }
            assert!(app.world().resource::<Session>().transform_edit.is_none());
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
fn noops_cancel_hidden_targets_invalid_events_and_focus_loss_preserve_source_and_redo() {
    let (mut app, slider) = fixture([0.; 3], 0, true);
    let before = app.world().resource::<Session>().editor.zone().clone();
    let original = app
        .world()
        .resource::<Session>()
        .transform_edit
        .as_ref()
        .unwrap()
        .fields
        .clone();
    let initial = app.world().get::<SliderValue>(slider).unwrap().0;
    change(&mut app, slider, initial, true);
    change(&mut app, slider, 90., false);
    change(&mut app, slider, initial, true);
    for value in [f32::NAN, f32::INFINITY, 181., -181.] {
        change(&mut app, slider, value, true);
    }
    let foreign = app.world_mut().spawn_empty().id();
    change(&mut app, foreign, 90., true);
    assert_eq!(
        app.world()
            .resource::<Session>()
            .transform_edit
            .as_ref()
            .unwrap()
            .fields,
        original
    );
    key(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    for target in 0..6 {
        let (mut app, slider) = fixture([0.; 3], target, true);
        let before = app.world().resource::<Session>().editor.zone().clone();
        let original = app
            .world()
            .resource::<Session>()
            .transform_edit
            .as_ref()
            .unwrap()
            .fields
            .clone();
        change(&mut app, slider, 90., true);
        if target >= 3 {
            assert!(app.world().get::<InteractionDisabled>(slider).is_some());
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .transform_edit
                    .as_ref()
                    .unwrap()
                    .fields,
                original
            );
        }
        if target == 0 {
            let mut fields = app.world_mut().query::<(&Field, &mut EditableText)>();
            fields
                .iter_mut(app.world_mut())
                .find(|(field, _)| field.0 == 0)
                .unwrap()
                .1
                .editor_mut()
                .set_text("NaN");
            key(&mut app, KeyCode::Enter);
            assert!(app.world().resource::<Session>().transform_edit.is_some());
            assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        }
        key(&mut app, KeyCode::Escape);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        assert!(app.world().get::<InteractionDisabled>(slider).is_some());
    }
    let (mut app, slider) = fixture([0.; 3], 0, true);
    let before = app.world().resource::<Session>().editor.zone().clone();
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .entity_mut(window)
        .get_mut::<Window>()
        .unwrap()
        .focused = false;
    change(&mut app, slider, 90., true);
    assert!(app.world().resource::<Session>().transform_edit.is_none());
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
}
#[test]
fn cpu_layout_keeps_yaw_slider_exact_fields_and_apply_controls_inside_properties() {
    for target in 0..6 {
        let (mut app, slider) = fixture([0.; 3], target, true);
        crate::headless_ui::add_layout(&mut app);
        for _ in 0..3 {
            app.update();
        }
        let visible = target < 3;
        let row = app
            .world_mut()
            .query_filtered::<(&Node, &Visibility), With<YawRow>>()
            .single(app.world())
            .unwrap();
        assert_eq!(row.0.display == Display::Flex, visible);
        assert_eq!(*row.1 == Visibility::Inherited, visible);
        for (control, computed, transform) in app
            .world_mut()
            .query::<(&Control, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(app.world())
        {
            if (matches!(control, Control::YawSlider) && !visible)
                || (matches!(control, Control::ValueSlider) && !matches!(target, 1 | 2 | 3 | 5))
            {
                assert_eq!(computed.size(), Vec2::ZERO);
                continue;
            }
            let centre = transform.affine().translation;
            let half = computed.size() * 0.5;
            assert!(computed.size().min_element() > 0.);
            assert!(centre.x - half.x >= 3140. && centre.x + half.x <= 3440.);
            assert!(centre.y - half.y >= 840. && centre.y + half.y <= 1440.);
        }
        let fields = app
            .world()
            .resource::<Session>()
            .transform_edit
            .as_ref()
            .unwrap()
            .fields
            .len();
        for (field, computed, transform) in app
            .world_mut()
            .query::<(&Field, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(app.world())
        {
            if field.0 >= fields {
                continue;
            }
            let centre = transform.affine().translation;
            let half = computed.size() * 0.5;
            assert!(computed.size().y >= 32.);
            assert!(centre.x - half.x >= 3140. && centre.x + half.x <= 3440.);
            assert!(centre.y - half.y >= 840. && centre.y + half.y <= 1440.);
        }
        if visible {
            assert_eq!(
                app.world().get::<ComputedNode>(slider).unwrap().size().y,
                24.
            );
        }
    }
}
