//! Inline edits use exact source values, normal focus and shared command history.
use super::*;
use crate::properties::inline::Input;
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};

fn button(app: &mut App, index: usize) -> Entity {
    app.world_mut()
        .query::<(Entity, &Value)>()
        .iter(app.world())
        .find(|(_, value)| value.0 == index)
        .unwrap()
        .0
}
fn open(app: &mut App, index: usize) -> Entity {
    let entity = button(app, index);
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(entity, FocusCause::Navigated);
    app.update();
    let focus = app.world().resource::<InputFocus>().get().unwrap();
    assert!(app.world().get::<Input>(focus).is_some());
    assert!(app.world().resource::<Session>().transform_edit.is_none());
    focus
}
fn type_value(app: &mut App, input: Entity, value: &str) {
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .editor_mut()
        .set_text(value);
    app.update();
}
fn key(app: &mut App, code: KeyCode, logical: Key) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code: code,
            logical_key: logical.clone(),
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }
}
#[test]
fn each_inline_field_retains_exact_values_and_escape_preserves_source_redo_and_clean_state() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in [0, 1, 2, 3, 5] {
            let (mut app, fields) = super::tests::fixture(origin, target);
            let source = app.world().resource::<Session>().editor.zone().clone();
            for (index, value) in fields.iter().enumerate() {
                let permanent = button(&mut app, index);
                assert!(
                    app.world()
                        .get::<ComputedNode>(permanent)
                        .unwrap()
                        .size()
                        .min_element()
                        > 0.
                );
                let input = open(&mut app, index);
                assert_eq!(input, permanent);
                assert_eq!(
                    app.world()
                        .get::<EditableText>(input)
                        .unwrap()
                        .value()
                        .to_string(),
                    *value
                );
                assert!(!app.world().resource::<Session>().is_dirty());
                type_value(&mut app, input, "123.125");
                assert!(app.world().resource::<Session>().is_dirty());
                assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
                key(&mut app, KeyCode::Escape, Key::Escape);
                let session = app.world().resource::<Session>();
                assert!(!session.inline_editing);
                assert!(app.world().get::<EditableText>(permanent).is_some());
                assert!(
                    app.world()
                        .get::<ComputedNode>(permanent)
                        .unwrap()
                        .size()
                        .min_element()
                        > 0.
                );
                assert!(!session.is_dirty());
                assert_eq!(session.editor.zone(), &source);
                assert_eq!(session.editor.revision(), 2);
                assert_eq!(session.editor.history(), (0, 1));
            }
        }
    }
}
#[test]
fn enter_and_blur_apply_one_validated_exact_transaction_for_each_attribute_kind_and_undo_once() {
    for target in [0, 1, 2, 3, 5] {
        for blur in [false, true] {
            let (mut app, fields) =
                super::tests::fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], target);
            let source = app.world().resource::<Session>().editor.zone().clone();
            let index = if target == 5 { 0 } else { fields.len() - 1 };
            let value = (fields[index].parse::<f64>().unwrap() + 0.125).to_string();
            let (id, original, _) = snapshot(app.world().resource::<Session>()).unwrap();
            let mut next = fields.clone();
            next[index] = value.clone();
            let mut expected = Editor::new(source.clone()).unwrap();
            expected
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![original.edit(id, &next).unwrap().unwrap()],
                })
                .unwrap();
            let input = open(&mut app, index);
            type_value(&mut app, input, &value);
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            if blur {
                app.world_mut().resource_mut::<InputFocus>().clear();
                app.update();
            } else {
                key(&mut app, KeyCode::Enter, Key::Enter);
            }
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), expected.zone());
            assert_eq!(session.editor.history(), (1, 0));
            assert_eq!(session.editor.revision(), 3);
            assert!(!session.inline_editing);
            app.world_mut()
                .resource_mut::<Session>()
                .editor
                .execute(Command::Undo {
                    expected_revision: 3,
                })
                .unwrap();
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        }
    }
}
#[test]
fn invalid_enter_preserves_source_and_focus_and_leaving_restores_canonical_values() {
    for value in ["NaN", "inf", "-1", "broken"] {
        let (mut app, _) = super::tests::fixture([0.; 3], 0);
        let source = app.world().resource::<Session>().editor.zone().clone();
        let input = open(&mut app, 4);
        type_value(&mut app, input, value);
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
        app.world_mut().resource_mut::<InputFocus>().clear();
        app.update();
        assert!(!app.world().resource::<Session>().inline_editing);
        assert!(!app.world().resource::<Session>().is_dirty());
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    }
}
#[test]
fn stale_selection_and_revision_refuse_open_and_a_replaced_document_cannot_receive_old_edits() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    let entity = button(&mut app, 0);
    app.world_mut().resource_mut::<Session>().selected = None;
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(entity, FocusCause::Navigated);
    app.update();
    assert!(!app.world().resource::<Session>().inline_editing);
    app.world_mut().resource_mut::<Session>().selected = Some(1);
    app.update();
    let input = open(&mut app, 0);
    type_value(&mut app, input, "100.125");
    let source = app.world().resource::<Session>().editor.zone().clone();
    let mut editor = Editor::new(source.clone()).unwrap();
    editor
        .execute(Command::Apply {
            expected_revision: 0,
            edits: vec![Edit::RenameZone {
                name: "Temporary replacement".into(),
            }],
        })
        .unwrap();
    editor
        .execute(Command::Undo {
            expected_revision: 1,
        })
        .unwrap();
    let mut replacement = Session::new(editor, PathBuf::new(), Some(source.clone()));
    replacement.selected = Some(1);
    app.insert_resource(replacement);
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.update();
    let session = app.world().resource::<Session>();
    assert_eq!(session.editor.zone(), &source);
    assert_eq!(session.editor.history(), (0, 1));
    assert!(!session.inline_editing);
}
#[test]
fn inline_commits_leave_disk_unchanged_until_explicit_save_and_save_commits_the_focused_value_first()
 {
    let (mut app, _) = super::tests::fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
    app.insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.6,
        pitch: 0.8,
        distance: 32.,
        preset: camera_views::Preset::Perspective,
        show_grid: true,
    });
    let path = std::env::temp_dir().join(format!(
        "theia-inline-save-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.world()
        .resource::<Session>()
        .editor
        .zone()
        .save(&path)
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    app.world_mut().resource_mut::<Session>().path = Some(path.clone());
    let input = open(&mut app, 0);
    type_value(&mut app, input, "128000000009.375");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let input = open(&mut app, 0);
    type_value(&mut app, input, "128000000010.625");
    app.world_mut()
        .resource_scope(|world, mut session: Mut<Session>| {
            perform(
                Action::Save,
                &mut session,
                &mut world.resource_mut::<View>(),
            )
        });
    assert_eq!(std::fs::read(&path).unwrap(), before);
    app.update();
    let saved = Zone::load(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(saved.objects[0].pose.position[0], 128000000010.625);
    assert_eq!(&saved, app.world().resource::<Session>().editor.zone());
    assert!(!app.world().resource::<Session>().is_dirty());
    assert_eq!(app.world().resource::<Session>().editor.history(), (2, 0));
}

#[test]
fn tab_commits_the_previous_permanent_field_and_focuses_the_next_without_a_modal() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    let input = open(&mut app, 0);
    type_value(&mut app, input, "9.375");
    key(&mut app, KeyCode::Tab, Key::Tab);
    let next = app.world().resource::<InputFocus>().get().unwrap();
    assert_eq!(app.world().get::<Value>(next).unwrap().0, 1);
    assert!(app.world().resource::<Session>().inline_editing);
    assert!(app.world().resource::<Session>().transform_edit.is_none());
    assert_eq!(
        app.world().resource::<Session>().editor.zone().objects[0]
            .pose
            .position[0],
        9.375
    );
    assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
    key(&mut app, KeyCode::Escape, Key::Escape);
}

#[test]
fn permanent_names_apply_enter_and_blur_validate_cancel_and_share_exact_undo() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        for target in [0, 1, 2, 3, 5] {
            let (mut app, _) = super::tests::fixture(origin, target);
            app.insert_resource(View {
                target: Vec3::ZERO,
                yaw: 0.6,
                pitch: 0.8,
                distance: 32.,
                preset: camera_views::Preset::Perspective,
                show_grid: true,
            });
            let source = app.world().resource::<Session>().editor.zone().clone();
            let (id, _, _) = snapshot(app.world().resource::<Session>()).unwrap();
            let id = id.unwrap();
            let field = app
                .world_mut()
                .query_filtered::<Entity, With<Name>>()
                .single(app.world())
                .unwrap();
            let original = source.authoring_name(id).unwrap();
            assert_eq!(
                app.world()
                    .get::<EditableText>(field)
                    .unwrap()
                    .value()
                    .to_string(),
                original
            );
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(field, FocusCause::Navigated);
            app.update();
            type_value(&mut app, field, "Étoile 树 🌿");
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            key(&mut app, KeyCode::Enter, Key::Enter);
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .editor
                    .zone()
                    .authoring_name(id),
                Some("Étoile 树 🌿")
            );
            assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
            assert!(app.world().resource::<Session>().name_edit.is_none());
            app.world_mut()
                .resource_scope(|world, mut session: Mut<Session>| {
                    perform(
                        Action::Undo,
                        &mut session,
                        &mut world.resource_mut::<View>(),
                    );
                });
            app.update();
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(field, FocusCause::Navigated);
            app.update();
            type_value(&mut app, field, "");
            key(&mut app, KeyCode::Enter, Key::Enter);
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(field));
            assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
            assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
            key(&mut app, KeyCode::Escape, Key::Escape);
            assert_eq!(
                app.world()
                    .get::<EditableText>(field)
                    .unwrap()
                    .value()
                    .to_string(),
                original
            );
            assert!(!app.world().resource::<Session>().is_dirty());
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(field, FocusCause::Navigated);
            app.update();
            type_value(&mut app, field, "Blur committed");
            app.world_mut().resource_mut::<InputFocus>().clear();
            app.update();
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .editor
                    .zone()
                    .authoring_name(id),
                Some("Blur committed")
            );
            assert_eq!(app.world().resource::<Session>().editor.history(), (1, 0));
            app.world_mut()
                .resource_mut::<InputFocus>()
                .set(field, FocusCause::Navigated);
            app.update();
            type_value(&mut app, field, "Tab committed");
            key(&mut app, KeyCode::Tab, Key::Tab);
            assert_eq!(
                app.world().resource::<InputFocus>().get(),
                Some(button(&mut app, 0))
            );
            assert_eq!(
                app.world()
                    .resource::<Session>()
                    .editor
                    .zone()
                    .authoring_name(id),
                Some("Tab committed")
            );
            assert_eq!(app.world().resource::<Session>().editor.history(), (2, 0));
            key(&mut app, KeyCode::Escape, Key::Escape);
        }
    }
}

#[test]
fn spawn_request_focuses_permanent_exact_fields_and_saves_only_on_explicit_save() {
    for origin in [
        [0.125, 0.25, -0.5],
        [128_000_000_000.125, 20.25, -128_000_000_000.5],
    ] {
        let (mut app, _) = super::tests::fixture(origin, 0);
        app.insert_resource(View {
            target: Vec3::ZERO,
            yaw: 0.6,
            pitch: 0.8,
            distance: 32.,
            preset: camera_views::Preset::Perspective,
            show_grid: true,
        });
        let source = app.world().resource::<Session>().editor.zone().clone();
        let path = std::env::temp_dir().join(format!(
            "theia-inline-spawn-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        source.save(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        app.world_mut().resource_mut::<Session>().path = Some(path.clone());
        app.world_mut()
            .resource_mut::<Session>()
            .ui
            .properties_collapsed = true;
        app.world_mut()
            .resource_mut::<workspace::Requests>()
            .spawn_transform = true;
        for _ in 0..4 {
            app.update();
        }
        let session = app.world().resource::<Session>();
        assert_eq!(outliner::selection(session), Some(outliner::Target::Spawn));
        assert!(!session.ui.properties_collapsed);
        assert!(session.transform_edit.is_none());
        assert_eq!(session.editor.zone(), &source);
        assert_eq!(session.editor.history(), (0, 1));
        let input = button(&mut app, 0);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
        assert_eq!(
            app.world()
                .get::<EditableText>(input)
                .unwrap()
                .value()
                .to_string(),
            source.spawn[0].to_string()
        );
        let visible: Vec<_> = app
            .world_mut()
            .query::<(&Value, &ComputedNode)>()
            .iter(app.world())
            .filter(|(_, node)| node.size().min_element() > 0.)
            .map(|(v, _)| v.0)
            .collect();
        assert_eq!(visible, vec![0, 1, 2]);
        let names: Vec<_> = app
            .world_mut()
            .query_filtered::<&ComputedNode, With<Name>>()
            .iter(app.world())
            .filter(|node| node.size().min_element() > 0.)
            .collect();
        assert!(names.is_empty());
        let changed = origin[0] + 9.375;
        type_value(&mut app, input, &changed.to_string());
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(
            app.world().resource::<Session>().editor.zone().spawn[0],
            changed
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let input = open(&mut app, 1);
        type_value(&mut app, input, "NaN");
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
        assert_eq!(
            app.world().resource::<Session>().editor.zone().spawn[1],
            source.spawn[1]
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        key(&mut app, KeyCode::Escape, Key::Escape);
        let input = open(&mut app, 2);
        let next = origin[2] + 8.625;
        type_value(&mut app, input, &next.to_string());
        app.world_mut()
            .resource_scope(|world, mut session: Mut<Session>| {
                perform(
                    Action::Save,
                    &mut session,
                    &mut world.resource_mut::<View>(),
                );
            });
        assert_eq!(std::fs::read(&path).unwrap(), before);
        app.update();
        let saved = Zone::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(saved.spawn, [changed, source.spawn[1], next]);
        assert_eq!(saved.objects, source.objects);
        assert!(!app.world().resource::<Session>().is_dirty());
        assert_eq!(app.world().resource::<Session>().editor.history(), (2, 0));
        app.world_mut()
            .resource_scope(|world, mut session: Mut<Session>| {
                perform(
                    Action::Undo,
                    &mut session,
                    &mut world.resource_mut::<View>(),
                );
                perform(
                    Action::Undo,
                    &mut session,
                    &mut world.resource_mut::<View>(),
                );
            });
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(
            outliner::selection(app.world().resource::<Session>()),
            Some(outliner::Target::Spawn)
        );
    }
}

#[test]
fn particle_inline_position_edits_use_exact_world_coordinates_and_shared_history() {
    let origin = [128_000_000_000., 4., -128_000_000_000.];
    let (mut app, _) = super::tests::fixture(origin, 0);
    let particle = theia_zone::particles::ParticlePlacement {
        id: 17,
        name: "Campfire".into(),
        effect: "vfx.campfire".into(),
        position: [origin[0] + 0.125, origin[1], origin[2]],
        yaw_degrees: 0.,
        scale: 1.,
    };
    let source = Zone {
        schema: 8,
        origin,
        spawn: origin,
        particles: vec![particle],
        ..Zone::default()
    };
    let mut session = Session::new(
        Editor::new(source.clone()).unwrap(),
        PathBuf::new(),
        Some(source.clone()),
    );
    session.particle_selected = Some(17);
    app.insert_resource(session);
    for _ in 0..3 {
        app.update();
    }
    assert!(
        description(app.world().resource::<Session>())
            .unwrap()
            .contains("Unresolved asset reference")
    );
    let input = open(&mut app, 0);
    type_value(&mut app, input, "128000000000.375");
    key(&mut app, KeyCode::Enter, Key::Enter);
    let session = app.world().resource::<Session>();
    assert_eq!(
        session.editor.zone().particles[0].position[0],
        origin[0] + 0.375
    );
    assert_eq!(session.editor.zone().particles[0].effect, "vfx.campfire");
    assert!(session.is_dirty());
    let revision = session.editor.revision();
    app.world_mut()
        .resource_mut::<Session>()
        .execute(Command::Undo {
            expected_revision: revision,
        });
    assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    for _ in 0..3 {
        app.update();
    }
    let effect = app
        .world_mut()
        .query_filtered::<Entity, With<Effect>>()
        .single(app.world())
        .unwrap();
    assert!(
        app.world()
            .get::<ComputedNode>(effect)
            .unwrap()
            .size()
            .min_element()
            > 0.
    );
    assert_eq!(
        app.world().get::<EditableText>(effect).unwrap().value(),
        "vfx.campfire"
    );
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(effect, FocusCause::Navigated);
    app.update();
    type_value(&mut app, effect, "../invalid");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(
        app.world().resource::<Session>().editor.zone().particles[0].effect,
        "vfx.campfire"
    );
    key(&mut app, KeyCode::Escape, Key::Escape);
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(effect, FocusCause::Navigated);
    app.update();
    type_value(&mut app, effect, "vfx.torch");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(
        app.world().resource::<Session>().editor.zone().particles[0].effect,
        "vfx.torch"
    );
}
