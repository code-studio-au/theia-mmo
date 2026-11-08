//! Numeric primitive transforms; standard text/focus widgets, existing commands.
use super::*;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};

const LABELS: [&str; 7] = [
    "World X (m)",
    "World Y (m)",
    "World Z (m)",
    "Yaw (degrees)",
    "Size X (m)",
    "Size Y (m)",
    "Size Z (m)",
];
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct Field(pub usize);
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Apply,
    Cancel,
}
pub(super) struct Draft {
    pub id: u64,
    revision: u64,
    original: Pose,
    subject: String,
    pub fields: [String; 7],
    pub active: usize,
    pub selection: [usize; 2],
    action: Option<Control>,
}
fn values(pose: &Pose) -> [String; 7] {
    [
        pose.position[0],
        pose.position[1],
        pose.position[2],
        pose.yaw_degrees,
        pose.size[0],
        pose.size[1],
        pose.size[2],
    ]
    .map(|v| v.to_string())
}
fn parse(fields: &[String; 7]) -> Result<Pose, String> {
    let mut values = [0.; 7];
    for (i, value) in fields.iter().enumerate() {
        values[i] = value
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("{} must be a finite number", LABELS[i]))?;
    }
    Ok(Pose {
        position: [values[0], values[1], values[2]],
        yaw_degrees: values[3],
        size: [values[4], values[5], values[6]],
    })
}

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Root,
            TabGroup::modal(),
            GlobalZIndex(20),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.035, 0.82)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: px(960),
                    padding: px(28).all(),
                    row_gap: px(14),
                    flex_direction: FlexDirection::Column,
                    border: px(2).all(),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.2, 0.65, 0.8)),
                BackgroundColor(Color::srgb(0.04, 0.065, 0.1)),
            ))
            .with_children(|panel| {
                panel.spawn((Caption, text("Object transform", 22.)));
                panel.spawn(text(
                    "Global metres | Y-up | sphere: equal XYZ | cylinder: equal XZ",
                    16.,
                ));
                for (index, label) in LABELS.iter().enumerate() {
                    panel
                        .spawn(Node {
                            column_gap: px(18),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                text(label, 19.),
                                Node {
                                    width: px(190),
                                    ..default()
                                },
                            ));
                            row.spawn((
                                Field(index),
                                TextInput,
                                SelectAllOnFocus,
                                TabIndex(index as i32),
                                Hovered::default(),
                                EditableText {
                                    max_characters: Some(64),
                                    allow_newlines: false,
                                    ..default()
                                },
                                Node {
                                    flex_grow: 1.,
                                    min_width: px(0),
                                    min_height: px(52),
                                    border: px(2).all(),
                                    ..default()
                                },
                                BorderColor::all(Color::srgb(0.35, 0.85, 1.)),
                                BackgroundColor(Color::srgb(0.015, 0.025, 0.04)),
                                TextFont {
                                    font_size: FontSize::Px(23.),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TextLayout::no_wrap(),
                                TextCursorStyle::default(),
                            ));
                        });
                }
                panel.spawn(text(
                    "Tab / Shift-Tab: next / previous field | Enter applies all | Escape cancels",
                    16.,
                ));
                panel
                    .spawn(Node {
                        column_gap: px(14),
                        ..default()
                    })
                    .with_children(|row| {
                        for (control, label) in [
                            (Control::Apply, "Apply transform"),
                            (Control::Cancel, "Cancel"),
                        ] {
                            row.spawn((
                                control,
                                Button,
                                Hovered::default(),
                                Node {
                                    padding: px(14).all(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.1, 0.24, 0.3)),
                            ))
                            .with_child(text(label, 19.));
                        }
                    });
            });
        });
}

pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    mut session: ResMut<Session>,
) {
    if event.button == PointerButton::Primary
        && let Ok(control) = controls.get(event.entity)
        && let Some(draft) = &mut session.transform_edit
    {
        draft.action = Some(*control);
    }
}

pub(super) fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    nav: TabNavigation,
    mut fields: Query<(Entity, &Field, &mut EditableText)>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    if session.transform_edit.is_some() {
        if keys.just_pressed(KeyCode::Tab)
            && !focus
                .get()
                .and_then(|e| fields.get(e).ok())
                .is_some_and(|(_, _, f)| f.is_composing())
        {
            let previous = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            match nav.navigate(
                &focus,
                if previous {
                    NavAction::Previous
                } else {
                    NavAction::Next
                },
            ) {
                Ok(next) => focus.set(next, FocusCause::Navigated),
                Err(error) => session.status = format!("Field navigation: {error}"),
            }
        }
        return;
    }
    if !keys.just_pressed(KeyCode::F9) || session.has_modal() {
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
    {
        session.status = "Finish or cancel the preview before editing properties".into();
        return;
    }
    let Some(object) = session
        .editor
        .zone()
        .objects
        .iter()
        .find(|o| Some(o.id) == session.selected)
    else {
        session.status = "Select an object to edit its transform".into();
        return;
    };
    let draft = Draft {
        id: object.id,
        revision: session.editor.revision(),
        original: object.pose.clone(),
        subject: format!("{:?} #{} | {}", object.primitive, object.id, object.name),
        fields: values(&object.pose),
        active: 0,
        selection: [0; 2],
        action: None,
    };
    for (entity, field, mut text) in &mut fields {
        // Preserve the font/layout styles Bevy prepared for this reusable field.
        text.clear();
        text.editor_mut().set_text(&draft.fields[field.0]);
        if field.0 == 0 {
            focus.set(entity, FocusCause::Navigated);
        }
    }
    session.transform_edit = Some(draft);
    session.status = "Editing transform - Apply validates all seven fields together".into();
}

pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    fields: Query<(Entity, &Field, &EditableText)>,
    mut roots: Query<&mut Node, With<Root>>,
    mut captions: Query<&mut Text, With<Caption>>,
) {
    let Some(mut draft) = session.transform_edit.take() else {
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
        return;
    };
    let mut composing = false;
    for (entity, field, text) in &fields {
        draft.fields[field.0] = text.value().to_string();
        if focus.get() == Some(entity) {
            draft.active = field.0;
        }
        composing |= text.is_composing();
    }
    // Resolve after updating active, since query iteration order is unspecified.
    let active_entity =
        fields
            .iter()
            .find(|(_, f, _)| f.0 == draft.active)
            .map(|(entity, _, text)| {
                let range = text.editor().raw_selection().text_range();
                draft.selection = [range.start, range.end];
                entity
            });
    let cancel = !windows.single().is_ok_and(|w| w.focused)
        || keys.just_pressed(KeyCode::Escape)
        || matches!(draft.action, Some(Control::Cancel));
    let apply = !composing
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || matches!(draft.action, Some(Control::Apply)));
    let close = if cancel {
        session.status = "Transform edit cancelled".into();
        true
    } else if apply {
        match parse(&draft.fields).and_then(|pose| {
            if pose == draft.original {
                return Ok(false);
            }
            let count = session.selected_path_len();
            session.editor.execute(Command::Apply {
                expected_revision: draft.revision,
                edits: vec![Edit::Transform { id: draft.id, pose }],
            })?;
            session.reconcile_after_edit(count);
            Ok(true)
        }) {
            Ok(changed) => {
                session.status = if changed {
                    "Transform updated"
                } else {
                    "Transform unchanged"
                }
                .into();
                true
            }
            Err(error) => {
                session.status = error;
                false
            }
        }
    } else {
        false
    };
    if close {
        focus.clear();
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
    } else {
        if let Some(entity) = active_entity
            && focus.get() != Some(entity)
        {
            focus.set(entity, FocusCause::Navigated);
        }
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::Flex;
        }
        if let Ok(mut caption) = captions.single_mut() {
            caption.0 = format!("Object transform | {}\n{}", draft.subject, session.status);
        }
        draft.action = None;
        session.transform_edit = Some(draft);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_fields_round_trip_far_f64_positions_without_display_rounding() {
        for sign in [-1., 1.] {
            let pose = Pose {
                position: [
                    sign * 128_000_000_000. + 0.012345,
                    20.123456789,
                    -sign * 128_000_000_000. - 0.987654,
                ],
                yaw_degrees: -37.123456789,
                size: [1.123456789, 2.123456789, 3.123456789],
            };
            assert_eq!(parse(&values(&pose)).unwrap(), pose);
            let mut fields = values(&pose);
            fields[4] = " 1.25e1 ".into();
            let changed = parse(&fields).unwrap();
            assert_eq!(changed.position, pose.position);
            assert_eq!(changed.yaw_degrees, pose.yaw_degrees);
            assert_eq!(changed.size, [12.5, pose.size[1], pose.size[2]]);
        }
    }
    #[test]
    fn malformed_and_nonfinite_fields_are_refused_before_commands() {
        let pose = Pose {
            position: [1., 2., 3.],
            yaw_degrees: 0.,
            size: [1.; 3],
        };
        for index in 0..7 {
            for invalid in ["", " ", "1,5", "2m", "NaN", "inf", "-inf", "1e999"] {
                let mut fields = values(&pose);
                fields[index] = invalid.into();
                assert!(parse(&fields).unwrap_err().contains(LABELS[index]));
            }
        }
    }
    #[test]
    fn parsed_fields_share_full_candidate_validation_and_one_entry_history_for_all_shapes() {
        for primitive in [Primitive::Box, Primitive::Sphere, Primitive::Cylinder] {
            for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
                let mut zone = Zone {
                    origin,
                    spawn: [origin[0], origin[1] + 0.1, origin[2] + 6.],
                    ..default()
                };
                let pose = Pose {
                    position: [origin[0] + 8., origin[1] + 1., origin[2] - 8.],
                    yaw_degrees: 0.,
                    size: [2.; 3],
                };
                zone.objects.push(Object {
                    id: 1,
                    name: "Numeric fixture".into(),
                    primitive,
                    pose: pose.clone(),
                });
                let mut editor = Editor::new(zone.clone()).unwrap();
                let mut fields = values(&pose);
                fields[0] = (origin[0] + 8.125).to_string();
                fields[1] = (origin[1] + 1.5).to_string();
                fields[2] = (origin[2] - 8.5).to_string();
                fields[3] = "37.5".into();
                for value in &mut fields[4..] {
                    *value = "3".into();
                }
                let changed = parse(&fields).unwrap();
                editor
                    .execute(Command::Apply {
                        expected_revision: 0,
                        edits: vec![Edit::Transform {
                            id: 1,
                            pose: changed.clone(),
                        }],
                    })
                    .unwrap();
                assert_eq!(editor.history(), (1, 0));
                assert_eq!(editor.zone().objects[0].pose, changed);
                editor
                    .execute(Command::Undo {
                        expected_revision: 1,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &zone);
                for (index, invalid) in [
                    (0, (origin[0] + 513.).to_string()),
                    (4, "-1".into()),
                    (3, "360001".into()),
                ] {
                    let mut invalid_fields = fields.clone();
                    invalid_fields[index] = invalid;
                    assert!(
                        editor
                            .execute(Command::Apply {
                                expected_revision: 2,
                                edits: vec![Edit::Transform {
                                    id: 1,
                                    pose: parse(&invalid_fields).unwrap()
                                }]
                            })
                            .is_err()
                    );
                    assert_eq!(editor.revision(), 2);
                    assert_eq!(editor.history(), (0, 1));
                    assert_eq!(editor.zone(), &zone);
                }
                if primitive != Primitive::Box {
                    let mut invalid_fields = fields.clone();
                    invalid_fields[4] = "4".into();
                    assert!(
                        editor
                            .execute(Command::Apply {
                                expected_revision: 2,
                                edits: vec![Edit::Transform {
                                    id: 1,
                                    pose: parse(&invalid_fields).unwrap()
                                }]
                            })
                            .is_err()
                    );
                    assert_eq!(editor.history(), (0, 1));
                }
                editor
                    .execute(Command::Redo {
                        expected_revision: 2,
                    })
                    .unwrap();
                assert_eq!(editor.zone().objects[0].pose, changed);
                assert_eq!(editor.zone().objects[0].name, "Numeric fixture");
            }
        }
    }
}
