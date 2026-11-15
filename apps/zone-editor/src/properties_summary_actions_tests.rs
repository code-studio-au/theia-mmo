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
            let next = if target == 2 {
                app.world_mut()
                    .query::<(Entity, &Input)>()
                    .iter(app.world())
                    .find_map(|(entity, input)| {
                        matches!(input, Input::MarkerDefinition).then_some(entity)
                    })
                    .unwrap()
            } else {
                button(&mut app, 0)
            };
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(next));
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

#[test]
fn quest_area_bounds_edit_inline_keep_selection_reject_inversion_and_undo() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    let zone: Zone = serde_json::from_slice(include_bytes!(
        "../../../content/zones/quest-areas.zone.json"
    ))
    .unwrap();
    let id = zone.quest_areas[0].id;
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        PathBuf::new(),
        Some(zone.clone()),
    );
    session.quest_area_selected = Some(id);
    app.insert_resource(session);
    app.update();
    app.update();
    let visible: Vec<_> = app
        .world_mut()
        .query::<(&Value, &ComputedNode)>()
        .iter(app.world())
        .filter(|(_, n)| n.size().min_element() > 0.)
        .map(|(v, _)| v.0)
        .collect();
    assert_eq!(visible.len(), 6);
    let headings: Vec<_> = app
        .world_mut()
        .query::<(&Heading, &Text, &ComputedNode)>()
        .iter(app.world())
        .filter(|(_, _, n)| n.size().min_element() > 0.)
        .map(|(_, t, _)| t.0.clone())
        .collect();
    assert_eq!(headings, ["Minimum bounds", "Maximum bounds"]);
    let input = open(&mut app, 0);
    let changed = zone.quest_areas[0].min[0] + 0.125;
    type_value(&mut app, input, &changed.to_string());
    key(&mut app, KeyCode::Enter, Key::Enter);
    let session = app.world().resource::<Session>();
    assert_eq!(session.editor.zone().quest_areas[0].min[0], changed);
    assert_eq!(
        outliner::selection(session),
        Some(outliner::Target::QuestArea(id))
    );
    assert_eq!(session.saved_zone.as_ref(), Some(&zone));
    assert!(session.is_dirty());
    let accepted = session.editor.zone().clone();
    let input = open(&mut app, 3);
    type_value(&mut app, input, &(changed - 1.).to_string());
    key(&mut app, KeyCode::Enter, Key::Enter);
    let session = app.world().resource::<Session>();
    assert_eq!(session.editor.zone(), &accepted);
    assert_eq!(session.editor.history(), (1, 0));
    assert_eq!(session.quest_area_selected, Some(id));
    app.world_mut()
        .resource_mut::<Session>()
        .execute(Command::Undo {
            expected_revision: 1,
        });
    assert_eq!(app.world().resource::<Session>().editor.zone(), &zone);
}

#[test]
fn marker_catalog_id_is_inline_validated_clearable_and_undoable_without_saving() {
    let (mut app, _) = super::tests::fixture([0.; 3], 2);
    let original = app.world().resource::<Session>().editor.zone().clone();
    let id = app.world().resource::<Session>().marker_selected.unwrap();
    let field = app
        .world_mut()
        .query::<(Entity, &Input, &ComputedNode)>()
        .iter(app.world())
        .find_map(|(entity, input, node)| {
            if matches!(input, Input::MarkerDefinition) {
                assert!(node.size().min_element() > 0.);
                Some(entity)
            } else {
                None
            }
        })
        .unwrap();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    type_value(&mut app, field, " theia.dev.guide ");
    key(&mut app, KeyCode::Enter, Key::Enter);
    let accepted = app.world().resource::<Session>().editor.zone().clone();
    assert_eq!(
        accepted
            .markers
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .definition
            .as_deref(),
        Some("theia.dev.guide")
    );
    let revision = app.world().resource::<Session>().editor.revision();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    type_value(&mut app, field, "../invalid");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &accepted);
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
    key(&mut app, KeyCode::Escape, Key::Escape);
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    type_value(&mut app, field, "");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert!(
        app.world()
            .resource::<Session>()
            .editor
            .zone()
            .markers
            .iter()
            .find(|m| m.id == id)
            .unwrap()
            .definition
            .is_none()
    );
    let mut session = app.world_mut().resource_mut::<Session>();
    let revision = session.editor.revision();
    session.execute(Command::Undo {
        expected_revision: revision,
    });
    assert_eq!(session.editor.zone(), &accepted);
    assert_eq!(session.saved_zone.as_ref(), Some(&original));
}

#[test]
fn quest_area_binding_fields_commit_validate_and_preserve_source_file_snapshot() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    let zone: Zone = serde_json::from_slice(include_bytes!(
        "../../../content/zones/quest-areas.zone.json"
    ))
    .unwrap();
    let id = zone.quest_areas[0].id;
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        PathBuf::new(),
        Some(zone.clone()),
    );
    session.quest_area_selected = Some(id);
    app.insert_resource(session);
    app.update();
    app.update();
    let fields: Vec<_> = app
        .world_mut()
        .query::<(Entity, &Input, &ComputedNode)>()
        .iter(app.world())
        .filter_map(|(e, input, node)| match input {
            Input::AreaDefinition => {
                assert!(node.size().min_element() > 0.);
                Some((false, e))
            }
            Input::AreaObjectives => {
                assert!(node.size().min_element() > 0.);
                Some((true, e))
            }
            _ => None,
        })
        .collect();
    assert_eq!(fields.len(), 2);
    for (objectives, entity) in fields {
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(entity, FocusCause::Navigated);
        app.update();
        let valid = if objectives {
            " theia.quest.a.visit , theia.quest.b.visit "
        } else {
            "theia.area.custom"
        };
        type_value(&mut app, entity, valid);
        key(&mut app, KeyCode::Enter, Key::Enter);
        let session = app.world().resource::<Session>();
        let accepted = session.editor.zone().clone();
        if objectives {
            assert_eq!(
                accepted.quest_areas[0].map_objectives,
                ["theia.quest.a.visit", "theia.quest.b.visit"]
            );
        } else {
            assert_eq!(accepted.quest_areas[0].definition, valid);
        }
        assert_eq!(session.saved_zone.as_ref(), Some(&zone));
        assert_eq!(session.quest_area_selected, Some(id));
        let revision = session.editor.revision();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(entity, FocusCause::Navigated);
        app.update();
        let invalid = if objectives {
            "theia.quest.a.visit, theia.quest.a.visit"
        } else {
            "Invalid/ID"
        };
        type_value(&mut app, entity, invalid);
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &accepted);
        assert_eq!(
            app.world().resource::<Session>().editor.revision(),
            revision
        );
        key(&mut app, KeyCode::Escape, Key::Escape);
    }
}

#[test]
fn scenery_inline_transforms_keep_selection_and_save_explicitly() {
    let origin = [128_000_000_000.125, 0., -128_000_000_000.25];
    let (mut app, _) = super::tests::fixture(origin, 0);
    let zone = Zone {
        schema: 10,
        origin,
        spawn: origin,
        scenery: vec![theia_zone::scenery::SceneryPlacement {
            id: 1,
            name: "Archway".into(),
            asset: "synty.forest.archway".into(),
            position: origin,
            yaw_degrees: 0.,
            scale: 1.,
        }],
        ..default()
    };
    let mut session = Session::new(
        Editor::new(zone.clone()).unwrap(),
        PathBuf::new(),
        Some(zone.clone()),
    );
    session.scenery_selected = Some(1);
    app.insert_resource(session);
    app.update();
    app.update();
    let visible = app
        .world_mut()
        .query::<(&Value, &ComputedNode)>()
        .iter(app.world())
        .filter(|(_, node)| node.size().min_element() > 0.)
        .count();
    assert_eq!(visible, 5);
    for (index, value) in [(0, origin[0] + 0.125), (3, 45.), (4, 2.)] {
        let input = open(&mut app, index);
        type_value(&mut app, input, &value.to_string());
        // Enter accepts the value directly from the permanent field; Save is separate.
        key(&mut app, KeyCode::Enter, Key::Enter);
        let session = app.world().resource::<Session>();
        let p = &session.editor.zone().scenery[0];
        assert_eq!(
            match index {
                0 => p.position[0],
                3 => p.yaw_degrees,
                _ => p.scale,
            },
            value
        );
        assert_eq!(session.scenery_selected, Some(1));
        assert_eq!(session.saved_zone.as_ref(), Some(&zone));
    }
    let before = app.world().resource::<Session>().editor.zone().clone();
    let input = open(&mut app, 4);
    type_value(&mut app, input, "-1");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &before);
    key(&mut app, KeyCode::Escape, Key::Escape);
    let mut session = app.world_mut().resource_mut::<Session>();
    session.edit(vec![Edit::DeleteScenery { id: 1 }]);
    assert_eq!(session.scenery_selected, None);
    let revision = session.editor.revision();
    session.execute(Command::Undo {
        expected_revision: revision,
    });
    assert_eq!(session.editor.zone(), &before);
}

#[test]
fn spawn_population_attributes_commit_exact_values_and_reject_invalid_drafts() {
    let (mut app, _) = super::tests::fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
    let saved = app.world().resource::<Session>().saved_zone.clone();
    spawn_groups::add(&mut app.world_mut().resource_mut::<Session>());
    for _ in 0..3 {
        app.update();
    }
    let original = app.world().resource::<Session>().editor.zone().spawn_groups[0].clone();
    let input = open(&mut app, 4);
    type_value(&mut app, input, "7");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(
        app.world().resource::<Session>().editor.zone().spawn_groups[0].target_alive,
        7
    );
    let accepted = app.world().resource::<Session>().editor.zone().clone();
    for (index, value) in [(4, "65"), (4, "2.5"), (5, "0"), (5, "86401"), (2, "NaN")] {
        let input = open(&mut app, index);
        type_value(&mut app, input, value);
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &accepted);
        key(&mut app, KeyCode::Escape, Key::Escape);
    }
    let centre = spawn_groups::centre(&original);
    let input = open(&mut app, 0);
    type_value(&mut app, input, &(centre[0] + 0.125).to_string());
    key(&mut app, KeyCode::Enter, Key::Enter);
    let group = &app.world().resource::<Session>().editor.zone().spawn_groups[0];
    for (old, new) in original.volume.polygon().iter().zip(group.volume.polygon()) {
        assert_eq!(new[0] - old[0], 0.125);
        assert_eq!(new[1], old[1]);
    }
    for (kind, value, valid) in [
        (0, "theia.creature.wolf", true),
        (0, "../wolf.json", false),
        (1, "theia.model.wolf", true),
        (1, "", true),
        (2, "18446744073709551615", true),
        (2, "18446744073709551616", false),
    ] {
        let field = app
            .world_mut()
            .query::<(Entity, &Input)>()
            .iter(app.world())
            .find_map(|(e, i)| match (kind, i) {
                (0, Input::SpawnCreature) | (1, Input::SpawnModel) | (2, Input::SpawnSeed) => {
                    Some(e)
                }
                _ => None,
            })
            .unwrap();
        let before = app.world().resource::<Session>().editor.zone().clone();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        app.update();
        type_value(&mut app, field, value);
        key(&mut app, KeyCode::Enter, Key::Enter);
        let after = app.world().resource::<Session>().editor.zone();
        if valid {
            assert_ne!(&before, after);
        } else {
            assert_eq!(&before, after);
            key(&mut app, KeyCode::Escape, Key::Escape);
        }
    }
    assert_eq!(
        app.world().resource::<Session>().editor.zone().spawn_groups[0].seed,
        u64::MAX
    );
    assert_eq!(app.world().resource::<Session>().saved_zone, saved);
}

#[test]
fn spawn_patrol_field_assigns_atomically_and_shares_undo_without_saving() {
    let (mut app, _) = super::tests::fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
    let saved = app.world().resource::<Session>().saved_zone.clone();
    spawn_groups::add(&mut app.world_mut().resource_mut::<Session>());
    let path = {
        let mut session = app.world_mut().resource_mut::<Session>();
        let centre = spawn_groups::centre(&session.editor.zone().spawn_groups[0]);
        let id = session.editor.zone().next_id().unwrap();
        session.edit(vec![Edit::PutPath {
            path: theia_zone::paths::GuidePath {
                id,
                name: "Meadow patrol".into(),
                kind: theia_zone::paths::PathKind::Patrol,
                width_metres: 1.,
                points: vec![centre, [centre[0] + 4., centre[1], centre[2]]],
            },
        }]);
        id
    };
    app.update();
    app.update();
    let field = app
        .world_mut()
        .query::<(Entity, &Input)>()
        .iter(app.world())
        .find_map(|(e, i)| matches!(i, Input::SpawnPatrol).then_some(e))
        .unwrap();
    let before = app.world().resource::<Session>().editor.zone().clone();
    let history = app.world().resource::<Session>().editor.history();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    type_value(&mut app, field, &path.to_string());
    key(&mut app, KeyCode::Enter, Key::Enter);
    let assigned = app.world().resource::<Session>().editor.zone().clone();
    assert_eq!(assigned.schema, 15);
    assert_eq!(assigned.spawn_groups[0].patrol_path, Some(path));
    assert_eq!(assigned.spawn_groups[0].roam_radius_metres, 0.);
    assert_eq!(
        app.world().resource::<Session>().editor.history().0,
        history.0 + 1
    );
    for value in ["999999", "-1", "NaN", "0"] {
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(field, FocusCause::Navigated);
        app.update();
        type_value(&mut app, field, value);
        key(&mut app, KeyCode::Enter, Key::Enter);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &assigned);
        key(&mut app, KeyCode::Escape, Key::Escape);
    }
    {
        let mut session = app.world_mut().resource_mut::<Session>();
        let revision = session.editor.revision();
        session.execute(Command::Undo {
            expected_revision: revision,
        });
        assert_eq!(session.editor.zone(), &before);
        let revision = session.editor.revision();
        session.execute(Command::Redo {
            expected_revision: revision,
        });
        assert_eq!(session.editor.zone(), &assigned);
    }
    app.update();
    let radius = open(&mut app, 6);
    type_value(&mut app, radius, "4");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &assigned);
    key(&mut app, KeyCode::Escape, Key::Escape);
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(field, FocusCause::Navigated);
    app.update();
    type_value(&mut app, field, "");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(
        app.world().resource::<Session>().editor.zone().spawn_groups[0].patrol_path,
        None
    );
    assert_eq!(app.world().resource::<Session>().saved_zone, saved);
}

#[test]
fn spawn_boundary_inline_edit_and_topology_actions_share_exact_history() {
    use workspace::Control;
    let (mut app, _) = super::tests::fixture([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
    spawn_groups::add(&mut app.world_mut().resource_mut::<Session>());
    app.update();
    let original = app.world().resource::<Session>().editor.zone().clone();
    spawn_groups::boundary_action(
        &mut app.world_mut().resource_mut::<Session>(),
        Control::SpawnBoundaryPoint(1),
    );
    app.update();
    let p = original.spawn_groups[0].volume.polygon()[0];
    let input = open(&mut app, 0);
    type_value(&mut app, input, &(p[0] + 0.125).to_string());
    key(&mut app, KeyCode::Enter, Key::Enter);
    let changed = app.world().resource::<Session>().editor.zone().clone();
    assert_eq!(
        changed.spawn_groups[0].volume.polygon()[0],
        [p[0] + 0.125, p[1]]
    );
    assert_eq!(
        &changed.spawn_groups[0].volume.polygon()[1..],
        &original.spawn_groups[0].volume.polygon()[1..]
    );
    let input = open(&mut app, 1);
    type_value(&mut app, input, "NaN");
    key(&mut app, KeyCode::Enter, Key::Enter);
    assert_eq!(app.world().resource::<Session>().editor.zone(), &changed);
    key(&mut app, KeyCode::Escape, Key::Escape);
    {
        let mut session = app.world_mut().resource_mut::<Session>();
        spawn_groups::boundary_action(&mut session, Control::SpawnBoundaryInsert);
        assert_eq!(session.spawn_boundary_point, Some(1));
        assert_eq!(
            session.editor.zone().spawn_groups[0].volume.polygon().len(),
            5
        );
        spawn_groups::boundary_action(&mut session, Control::SpawnBoundaryRemove);
        assert_eq!(session.editor.zone(), &changed);
        for _ in 0..3 {
            let revision = session.editor.revision();
            session.execute(Command::Undo {
                expected_revision: revision,
            });
        }
        assert_eq!(session.editor.zone(), &original);
        session.set_mode(EditorMode::Objects);
        assert_eq!(session.spawn_boundary_point, None);
    }
    app.update();
}

#[test]
fn spawn_boundary_navigation_commits_pending_field_before_changing_vertex() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    app.insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.6,
        pitch: 0.8,
        distance: 32.,
        preset: camera_views::Preset::Perspective,
        show_grid: true,
    });
    spawn_groups::add(&mut app.world_mut().resource_mut::<Session>());
    spawn_groups::boundary_action(
        &mut app.world_mut().resource_mut::<Session>(),
        workspace::Control::SpawnBoundaryPoint(1),
    );
    app.update();
    let input = open(&mut app, 0);
    type_value(&mut app, input, "-7.5");
    {
        let world = app.world_mut();
        world.resource_scope(|world, mut session: Mut<Session>| {
            let mut view = world.resource_mut::<View>();
            perform(Action::SpawnBoundaryPoint(1), &mut session, &mut view);
            assert_eq!(session.spawn_boundary_point, Some(0));
        });
    }
    app.update();
    let session = app.world().resource::<Session>();
    assert_eq!(session.spawn_boundary_point, Some(1));
    assert_eq!(
        session.editor.zone().spawn_groups[0].volume.polygon()[0][0],
        -7.5
    );
    assert_eq!(
        session.editor.zone().spawn_groups[0].volume.polygon()[1][0],
        8.
    );
}

#[test]
fn spawn_creature_assignment_commits_pending_fields_and_refreshes_loaded_context_feedback() {
    let (mut app, _) = super::tests::fixture([0.; 3], 0);
    app.insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.6,
        pitch: 0.8,
        distance: 32.,
        preset: camera_views::Preset::Perspective,
        show_grid: true,
    });
    {
        let mut session = app.world_mut().resource_mut::<Session>();
        spawn_groups::add(&mut session);
        let mut group = session.editor.zone().spawn_groups[0].clone();
        group.creature = "theia.creature.unresolved".into();
        session.edit(vec![Edit::PutSpawnGroup { group }]);
        session.quest_context = Some(
            crate::quest_inspector::Context::load(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content/development-gameplay-classic-quest-areas.json"),
            )
            .unwrap(),
        );
    }
    app.update();
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("does not match"))
    );
    let input = open(&mut app, 0);
    type_value(&mut app, input, "-7.5");
    app.world_mut()
        .resource_scope(|world, mut session: Mut<Session>| {
            let mut view = world.resource_mut::<View>();
            perform(Action::BindSpawnCreature, &mut session, &mut view);
            assert_eq!(
                session.editor.zone().spawn_groups[0].creature,
                "theia.creature.unresolved"
            );
        });
    app.update();
    {
        let session = app.world().resource::<Session>();
        let group = &session.editor.zone().spawn_groups[0];
        assert_eq!(spawn_groups::centre(group)[0], -7.5);
        assert_eq!(
            group.creature,
            session
                .quest_context
                .as_ref()
                .unwrap()
                .creature()
                .id()
                .as_str()
        );
    }
    app.update();
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Pursuing definition resolved"))
    );
    // Context removal alone must refresh the cached inspection, without a zone edit.
    let revision = app.world().resource::<Session>().editor.revision();
    app.world_mut().resource_mut::<Session>().quest_context = None;
    app.update();
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Load gameplay to inspect"))
    );
    app.world_mut().resource_mut::<Session>().creature_context = Some(
        crate::spawn_groups::Context::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/gameplay/development-creature-passive.json"),
        )
        .unwrap(),
    );
    app.update();
    assert_eq!(
        app.world().resource::<Session>().editor.revision(),
        revision
    );
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text.0.contains("Passive definition resolved")
                && text.0.contains("Creature file:"))
    );
}
