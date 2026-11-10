//! Actual f64 attribute rows and CPU layout; no renderer, window backend or OS input.
use super::*;
use bevy::ecs::system::RunSystemOnce;
fn setup(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: px(sidebar::WIDTH + outliner::WIDTH + 12.),
            top: px(workspace::HEADER + 12.),
            width: px(276),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(spawn);
}
fn fixture(origin: [f64; 3], target: u8) -> (App, Vec<String>) {
    let (mut app, _) = super::super::yaw_tests::fixture_with_input(origin, target, true, true);
    let fields = app
        .world()
        .resource::<Session>()
        .transform_edit
        .as_ref()
        .unwrap()
        .fields
        .clone();
    app.world_mut().resource_mut::<Session>().transform_edit = None;
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.world_mut().run_system_once(setup).unwrap();
    app.add_plugins(plugin);
    crate::headless_ui::add_layout(&mut app);
    for _ in 0..3 {
        app.update();
    }
    (app, fields)
}
#[test]
fn selected_attribute_rows_keep_exact_editor_values_labels_geometry_source_and_pending_redo() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in 0..6 {
            let (mut app, expected) = fixture(origin, target);
            let source = app.world().resource::<Session>().editor.zone().clone();
            let values: Vec<_> = app
                .world_mut()
                .query::<(&Value, &Text, &ComputedNode, &ChildOf, &Hovered)>()
                .iter(app.world())
                .filter(|(_, _, node, _, _)| node.size().min_element() > 0.)
                .map(|(value, text, _, parent, _)| {
                    let wrapper = app.world().get::<ComputedNode>(parent.parent()).unwrap();
                    assert!(wrapper.size().x >= 170.);
                    (value.0, text.0.clone())
                })
                .collect();
            if target == 4 {
                assert!(values.is_empty());
            } else {
                assert_eq!(values.len(), expected.len());
                for (index, value) in values {
                    assert_eq!(value, expected[index]);
                }
                let capture = snapshot(app.world().resource::<Session>()).unwrap().0;
                for (label, text, node) in app
                    .world_mut()
                    .query::<(&Label, &Text, &ComputedNode)>()
                    .iter(app.world())
                {
                    if node.size().min_element() > 0. {
                        assert_eq!(text.0, capture.labels()[label.0]);
                    }
                }
                if target == 2 {
                    assert!(
                        description(app.world().resource::<Session>())
                            .unwrap()
                            .contains("Authoring annotation")
                    );
                }
            }
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.history(), (0, 1));
            assert_eq!(session.editor.revision(), 2);
            assert!(!session.is_dirty());
        }
    }
}
#[test]
fn summary_shows_exact_drag_preview_then_source_again_and_hides_for_offline_playtest() {
    let (mut app, _) = fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
    let source = app.world().resource::<Session>().editor.zone().clone();
    let before = source.objects[0].pose.clone();
    let mut preview = before.clone();
    preview.position[0] += 0.125;
    preview.yaw_degrees += 0.5;
    let expected = Target::Object(preview.clone()).values();
    app.world_mut().resource_mut::<Session>().drag = Some(Drag {
        id: 1,
        before,
        preview,
        revision: 2,
        gesture: gizmo::Gesture::plane(Ray3d::new(Vec3::Y * 10., Dir3::NEG_Y), Vec3::ZERO).unwrap(),
    });
    app.update();
    for (field, text) in app.world_mut().query::<(&Value, &Text)>().iter(app.world()) {
        assert_eq!(text.0, expected[field.0]);
    }
    assert!(
        description(app.world().resource::<Session>())
            .unwrap()
            .contains("Transform preview")
    );
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    app.world_mut().resource_mut::<Session>().drag = None;
    app.update();
    let expected = Target::Object(source.objects[0].pose.clone()).values();
    for (field, text) in app.world_mut().query::<(&Value, &Text)>().iter(app.world()) {
        assert_eq!(text.0, expected[field.0]);
    }
    app.world_mut().resource_mut::<Session>().play =
        Some(playtest::Playtest::new(&source).unwrap());
    for _ in 0..3 {
        app.update();
    }
    let root = app
        .world_mut()
        .query_filtered::<&Node, With<Root>>()
        .single(app.world())
        .unwrap();
    assert_eq!(root.display, Display::None);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
}
