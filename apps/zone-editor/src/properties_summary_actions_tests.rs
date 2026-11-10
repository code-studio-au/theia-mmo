//! Property-value entry points through the existing draft/focus/cancel systems.
use super::*;
fn button(app: &mut App, index: usize) -> Entity {
    app.world_mut()
        .query::<(Entity, &ValueButton)>()
        .iter(app.world())
        .find(|(_, value)| value.0 == index)
        .unwrap()
        .0
}
fn cancel(app: &mut App) {
    let entity = app
        .world_mut()
        .query::<(Entity, &super::super::Control)>()
        .iter(app.world())
        .find(|(_, control)| **control == super::super::Control::Cancel)
        .unwrap()
        .0;
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    app.world_mut().trigger(PointerPress {
        entity,
        button: PointerButton::Primary,
        count: 1,
        pointer: bevy::picking::events::Pointer::new(
            bevy::picking::pointer::PointerId::Mouse,
            bevy::picking::pointer::Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::new(500., 350.),
            },
        ),
        hit: bevy::picking::backend::HitData::new(window, 0., None, None),
    });
    for _ in 0..2 {
        app.update();
    }
}
#[test]
fn each_summary_value_focuses_its_exact_field_and_cancel_preserves_all_source_and_redo() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in [0, 1, 2, 3, 5] {
            let (mut app, fields) = super::tests::fixture(origin, target);
            let source = app.world().resource::<Session>().editor.zone().clone();
            for index in 0..fields.len() {
                let value = button(&mut app, index);
                app.world_mut().trigger(Activate { entity: value });
                for _ in 0..2 {
                    app.update();
                }
                let draft = app
                    .world()
                    .resource::<Session>()
                    .transform_edit
                    .as_ref()
                    .unwrap();
                assert_eq!(draft.fields, fields);
                assert_eq!(draft.active, index);
                let focus = app.world().resource::<InputFocus>().get().unwrap();
                assert_eq!(app.world().get::<Field>(focus).unwrap().0, index);
                assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
                assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
                cancel(&mut app);
                assert!(app.world().resource::<Session>().transform_edit.is_none());
            }
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
            assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
            assert!(!app.world().resource::<Session>().is_dirty());
        }
    }
}
#[test]
fn summary_entry_rejects_changed_selection_revision_point_hidden_fields_and_unfocused_input() {
    for cause in 0..5 {
        let target = if matches!(cause, 2 | 3) { 3 } else { 0 };
        let (mut app, _) = super::tests::fixture([0.; 3], target);
        let entity = button(&mut app, if cause == 3 { 6 } else { 0 });
        match cause {
            0 => app.world_mut().resource_mut::<Session>().selected = Some(2),
            1 => {
                app.world_mut()
                    .resource_mut::<Session>()
                    .editor
                    .execute(Command::Apply {
                        expected_revision: 2,
                        edits: vec![Edit::RenameZone {
                            name: "Changed externally".into(),
                        }],
                    })
                    .unwrap();
            }
            2 => app.world_mut().resource_mut::<Session>().path_point = Some(1),
            3 => {}
            _ => {
                let window = app
                    .world_mut()
                    .query_filtered::<Entity, With<Window>>()
                    .single(app.world())
                    .unwrap();
                app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
            }
        }
        let source = app.world().resource::<Session>().editor.zone().clone();
        let history = app.world().resource::<Session>().editor.history();
        let revision = app.world().resource::<Session>().editor.revision();
        app.world_mut().trigger(Activate { entity });
        app.update();
        assert!(app.world().resource::<Session>().transform_edit.is_none());
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), history);
        assert_eq!(
            app.world().resource::<Session>().editor.revision(),
            revision
        );
    }
}

#[test]
fn value_press_opens_before_release_and_reflow_cleanup_allows_reopening_without_source_edits() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    app.add_plugins((bevy::ui_widgets::ButtonPlugin, theme::plugin));
    let entity = button(&mut app, 2);
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    let pointer = bevy::picking::events::Pointer::new(
        bevy::picking::pointer::PointerId::Mouse,
        bevy::picking::pointer::Location {
            target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                .normalize(Some(window))
                .unwrap(),
            position: Vec2::new(500., 350.),
        },
    );
    let source = app.world().resource::<Session>().editor.zone().clone();
    for _ in 0..2 {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut().trigger(PointerPress {
            entity,
            button: PointerButton::Primary,
            count: 1,
            pointer: pointer.clone(),
            hit: bevy::picking::backend::HitData::new(window, 0., None, None),
        });
        app.update();
        assert_eq!(
            app.world()
                .resource::<Session>()
                .transform_edit
                .as_ref()
                .unwrap()
                .active,
            2
        );
        assert!(app.world().get::<bevy::ui::Pressed>(entity).is_some());
        let focused = app.world().resource::<InputFocus>().get().unwrap();
        app.world_mut()
            .trigger(bevy::picking::events::PointerRelease {
                entity: focused,
                button: PointerButton::Primary,
                pointer: pointer.clone(),
                hit: bevy::picking::backend::HitData::new(window, 0., None, None),
            });
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert!(app.world().get::<bevy::ui::Pressed>(entity).is_none());
        cancel(&mut app);
    }
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    assert_eq!(app.world().resource::<Session>().editor.revision(), 2);
}
