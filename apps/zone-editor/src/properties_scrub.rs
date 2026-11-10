//! Numeric dragging adapted from Sprinkles text_edit.rs (0e978f5, MIT).
//! Keep the starting f64 and its exact string; never round a world coordinate.
use super::*;

#[derive(Component)]
pub(crate) struct Handle(pub usize);
pub(crate) struct Drag {
    field: Entity,
    index: usize,
    revision: u64,
    target: Option<u64>,
    x: f32,
    value: f64,
    text: String,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn scrub(
    mut commands: Commands,
    windows: Query<(Entity, &Window, Option<&bevy::window::CursorIcon>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<Session>,
    mut focus: ResMut<InputFocus>,
    mut handles: Query<(&Handle, &Hovered, &mut TextColor)>,
    mut fields: Query<(Entity, &Field, &mut EditableText)>,
    mut drag: Local<Option<Drag>>,
) {
    let Ok((window_entity, window, cursor)) = windows.single() else {
        return;
    };
    let draft = session.transform_edit.as_ref();
    let allowed = window.focused
        && draft.is_some()
        && !session.ui.input_owned
        && session.play.is_none()
        && session.name_edit.is_none()
        && session.file_dialog.is_none();
    let mut over_handle = false;
    for (handle, hovered, mut color) in &mut handles {
        let visible = draft.is_some_and(|draft| handle.0 < draft.fields.len());
        let active = drag.as_ref().is_some_and(|drag| drag.index == handle.0);
        color.0 = if allowed && visible && (hovered.0 || active) {
            theme::PRIMARY
        } else {
            theme::MUTED
        };
        over_handle |= allowed && visible && hovered.0;
        if allowed && visible && hovered.0 && mouse.just_pressed(MouseButton::Left) {
            let Some(position) = window.cursor_position() else {
                continue;
            };
            let Some((entity, _, field)) =
                fields.iter_mut().find(|(_, field, _)| field.0 == handle.0)
            else {
                continue;
            };
            let text = field.value().to_string();
            let Some(value) = text.trim().parse::<f64>().ok().filter(|v| v.is_finite()) else {
                continue;
            };
            if field.is_composing() {
                continue;
            }
            let draft = draft.unwrap();
            *drag = Some(Drag {
                field: entity,
                index: handle.0,
                revision: draft.revision,
                target: draft.id,
                x: position.x,
                value,
                text,
            });
            focus.set(entity, FocusCause::Pressed);
        }
    }
    if let Some(active) = drag.as_ref() {
        let matching = draft
            .is_some_and(|draft| draft.revision == active.revision && draft.id == active.target);
        if let Ok((_, _, mut field)) = fields.get_mut(active.field) {
            if !allowed
                || !matching
                || window.cursor_position().is_none()
                || keys.just_pressed(KeyCode::Escape)
            {
                if matching {
                    field.clear();
                    field.editor_mut().set_text(&active.text);
                }
                *drag = None;
            } else {
                let alt = keys.pressed(KeyCode::AltLeft)
                    || keys.pressed(KeyCode::AltRight)
                    || keys.pressed(KeyCode::SuperLeft)
                    || keys.pressed(KeyCode::SuperRight);
                let (amount, sensitivity) = if alt { (1., 10.) } else { (0.1, 5.) };
                let delta = window.cursor_position().unwrap().x - active.x;
                // Truncation gives a symmetric stationary dead zone. The exact
                // original text is restored at zero steps, including exponents.
                let steps = f64::from((delta / sensitivity).trunc());
                let value = active.value + steps * amount;
                if value.is_finite() && !field.is_composing() {
                    let text = if steps == 0. {
                        active.text.clone()
                    } else {
                        value.to_string()
                    };
                    if field.value().to_string() != text {
                        field.clear();
                        field.editor_mut().set_text(&text);
                    }
                }
                if !mouse.pressed(MouseButton::Left) {
                    *drag = None;
                }
            }
        } else {
            *drag = None;
        }
    }
    let resize_cursor = bevy::window::CursorIcon::System(bevy::window::SystemCursorIcon::ColResize);
    if allowed && (over_handle || drag.is_some()) && cursor != Some(&resize_cursor) {
        commands.entity(window_entity).insert(resize_cursor);
    } else if (!allowed || (!over_handle && drag.is_none())) && cursor == Some(&resize_cursor) {
        commands
            .entity(window_entity)
            .insert(bevy::window::CursorIcon::default());
    }
}

#[cfg(test)]
mod tests {
    use super::super::yaw_tests::fixture_with_input;
    use super::*;
    fn setup(origin: [f64; 3], target: u8) -> App {
        let (mut app, _) = fixture_with_input(origin, target, false, false);
        app.init_resource::<ButtonInput<MouseButton>>()
            .add_systems(Update, scrub.before(keyboard));
        let handles: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Handle)>()
            .iter(app.world())
            .map(|(entity, handle)| (entity, handle.0 == 0))
            .collect();
        for (entity, hovered) in handles {
            app.world_mut().entity_mut(entity).insert(Hovered(hovered));
        }
        app
    }
    fn move_to(app: &mut App, x: f32) {
        app.world_mut()
            .query::<&mut Window>()
            .single_mut(app.world_mut())
            .unwrap()
            .set_cursor_position(Some(Vec2::new(x, 400.)));
        app.update();
    }
    fn text(app: &mut App) -> String {
        app.world_mut()
            .query::<(&Field, &EditableText)>()
            .iter(app.world())
            .find(|(field, _)| field.0 == 0)
            .unwrap()
            .1
            .value()
            .to_string()
    }
    fn press(app: &mut App) {
        move_to(app, 1000.);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear_just_pressed(MouseButton::Left);
    }
    fn release(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear_just_released(MouseButton::Left);
    }
    #[test]
    fn numeric_drag_preserves_stationary_precision_and_applies_one_shared_edit_for_every_attribute_kind()
     {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            for target in 0..6 {
                let mut app = setup(origin, target);
                let source = app.world().resource::<Session>().editor.zone().clone();
                let original = text(&mut app);
                let value = original.parse::<f64>().unwrap();
                press(&mut app);
                move_to(&mut app, 1004.);
                assert_eq!(text(&mut app), original);
                move_to(&mut app, 1015.);
                assert!((text(&mut app).parse::<f64>().unwrap() - value - 0.3).abs() < 0.0001);
                move_to(&mut app, 1000.);
                assert_eq!(text(&mut app), original);
                move_to(&mut app, 1015.);
                release(&mut app);
                let session = app.world().resource::<Session>();
                assert_eq!(session.editor.zone(), &source);
                assert_eq!(session.editor.history(), (0, 1));
                assert_eq!(session.editor.revision(), 2);
                let draft = session.transform_edit.as_ref().unwrap();
                let edit = draft
                    .original
                    .edit(draft.id, &draft.fields)
                    .unwrap()
                    .unwrap();
                let mut oracle = Editor::new(source).unwrap();
                oracle
                    .execute(Command::Apply {
                        expected_revision: 0,
                        edits: vec![edit],
                    })
                    .unwrap();
                super::super::yaw_tests::key(&mut app, KeyCode::Enter);
                let session = app.world().resource::<Session>();
                assert!(session.transform_edit.is_none());
                assert_eq!(session.editor.zone(), oracle.zone());
                assert_eq!(session.editor.history(), (1, 0));
                assert_eq!(session.editor.revision(), 3);
            }
        }
    }
    #[test]
    fn numeric_drag_refuses_malformed_text_and_keeps_source_and_redo_on_cancel_or_focus_loss() {
        for focus_loss in [false, true] {
            let mut app = setup([128_000_000_000.125, 20.25, -128_000_000_000.5], 0);
            let source = app.world().resource::<Session>().editor.zone().clone();
            press(&mut app);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::AltLeft);
            move_to(&mut app, 1020.);
            if focus_loss {
                app.world_mut()
                    .query::<&mut Window>()
                    .single_mut(app.world_mut())
                    .unwrap()
                    .focused = false;
            } else {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Escape);
            }
            app.update();
            let session = app.world().resource::<Session>();
            assert!(session.transform_edit.is_none());
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.history(), (0, 1));
            assert_eq!(session.editor.revision(), 2);
        }
        let mut app = setup([0.; 3], 0);
        for (field, mut text) in app
            .world_mut()
            .query::<(&Field, &mut EditableText)>()
            .iter_mut(app.world_mut())
        {
            if field.0 == 0 {
                text.clear();
                text.editor_mut().set_text("bad");
            }
        }
        app.update();
        let source = app.world().resource::<Session>().editor.zone().clone();
        press(&mut app);
        move_to(&mut app, 1100.);
        release(&mut app);
        assert_eq!(text(&mut app), "bad");
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    }
}
