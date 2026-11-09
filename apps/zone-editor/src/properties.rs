//! Numeric object, plant and marker transforms; standard text/focus widgets, existing commands.
use super::*;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{NavAction, TabGroup, TabIndex, TabNavigation},
};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};
use theia_zone::{markers::Marker, vegetation::Plant};

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
#[derive(Component)]
pub(super) struct FieldRow(usize);
#[derive(Component)]
pub(super) struct FieldLabel(usize);
#[derive(Clone, Debug, PartialEq)]
enum Target {
    Object(Pose),
    Plant(Plant),
    Marker(Marker),
}
impl Target {
    fn labels(&self) -> &'static [&'static str] {
        match self {
            Self::Object(_) => &LABELS,
            Self::Plant(_) => &[
                "World X (m)",
                "World Y (m)",
                "World Z (m)",
                "Yaw (degrees)",
                "Scale",
            ],
            Self::Marker(_) => &[
                "World X (m)",
                "World Y (m)",
                "World Z (m)",
                "Yaw (degrees)",
                "Radius (m)",
            ],
        }
    }
    fn values(&self) -> Vec<String> {
        match self {
            Self::Object(pose) => values(pose).to_vec(),
            Self::Plant(p) => [
                p.position[0],
                p.position[1],
                p.position[2],
                p.yaw_degrees,
                p.scale,
            ]
            .map(|v| v.to_string())
            .to_vec(),
            Self::Marker(m) => [
                m.position[0],
                m.position[1],
                m.position[2],
                m.yaw_degrees,
                m.radius_metres,
            ]
            .map(|v| v.to_string())
            .to_vec(),
        }
    }
    fn edit(&self, id: u64, fields: &[String]) -> Result<Option<Edit>, String> {
        match self {
            Self::Object(original) => {
                let pose = parse(fields)?;
                Ok((pose != *original).then_some(Edit::Transform { id, pose }))
            }
            Self::Plant(original) => {
                let v = numbers(fields, self.labels())?;
                let mut plant = original.clone();
                plant.position = [v[0], v[1], v[2]];
                plant.yaw_degrees = v[3];
                plant.scale = v[4];
                Ok((plant != *original).then_some(Edit::PutPlant { plant }))
            }
            Self::Marker(original) => {
                let v = numbers(fields, self.labels())?;
                let mut marker = original.clone();
                marker.position = [v[0], v[1], v[2]];
                marker.yaw_degrees = v[3];
                marker.radius_metres = v[4];
                Ok((marker != *original).then_some(Edit::PutMarker { marker }))
            }
        }
    }
}
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Apply,
    Cancel,
}
pub(super) struct Draft {
    pub id: u64,
    revision: u64,
    original: Target,
    subject: String,
    pub fields: Vec<String>,
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
fn numbers(fields: &[String], labels: &[&str]) -> Result<Vec<f64>, String> {
    if fields.len() != labels.len() {
        return Err("Incorrect transform field count".into());
    }
    fields
        .iter()
        .zip(labels)
        .map(|(value, label)| {
            value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("{label} must be a finite number"))
        })
        .collect()
}
fn parse(fields: &[String]) -> Result<Pose, String> {
    let values = numbers(fields, &LABELS)?;
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
                right: px(0),
                bottom: px(0),
                width: px(outliner::WIDTH),
                height: px(workspace::INSPECTOR),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::FlexStart,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.065, 0.1)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100),
                    padding: px(12).all(),
                    row_gap: px(6),
                    flex_direction: FlexDirection::Column,
                    border: px(2).all(),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.2, 0.65, 0.8)),
                BackgroundColor(Color::srgb(0.04, 0.065, 0.1)),
            ))
            .with_children(|panel| {
                panel.spawn((Caption, text("TRANSFORM", 16.), TextLayout::no_wrap()));
                panel.spawn(text("World metres | Y-up | yaw degrees", 13.));
                for (index, label) in LABELS.iter().enumerate() {
                    panel
                        .spawn((
                            FieldRow(index),
                            Visibility::Inherited,
                            Node {
                                column_gap: px(6),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|row| {
                            row.spawn((
                                FieldLabel(index),
                                text(label, 13.),
                                Node {
                                    width: px(88),
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
                                    min_height: px(32),
                                    border: px(2).all(),
                                    ..default()
                                },
                                BorderColor::all(Color::srgb(0.35, 0.85, 1.)),
                                BackgroundColor(Color::srgb(0.015, 0.025, 0.04)),
                                TextFont {
                                    font_size: FontSize::Px(16.),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TextLayout::no_wrap(),
                                TextCursorStyle::default(),
                            ));
                        });
                }
                panel.spawn(text(
                    "Enter applies all fields.\nEscape cancels. Tab changes field.",
                    13.,
                ));
                panel
                    .spawn(Node {
                        column_gap: px(6),
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
                                    padding: px(8).all(),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.1, 0.24, 0.3)),
                            ))
                            .with_child(text(label, 14.));
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
    mut requests: ResMut<workspace::Requests>,
) {
    let requested = std::mem::take(&mut requests.transform);
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
    if !(keys.just_pressed(KeyCode::F9) || requested) || session.has_modal() {
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
    let zone = session.editor.zone();
    let (id, original, subject) =
        if let Some(object) = zone.objects.iter().find(|o| Some(o.id) == session.selected) {
            (
                object.id,
                Target::Object(object.pose.clone()),
                format!("{:?} #{} | {}", object.primitive, object.id, object.name),
            )
        } else if let Some(plant) = zone
            .plants
            .iter()
            .find(|p| Some(p.id) == session.plant_selected)
        {
            (
                plant.id,
                Target::Plant(plant.clone()),
                format!("{:?} #{} | {}", plant.kind, plant.id, plant.name),
            )
        } else if let Some(marker) = zone
            .markers
            .iter()
            .find(|m| Some(m.id) == session.marker_selected)
        {
            (
                marker.id,
                Target::Marker(marker.clone()),
                format!("{:?} #{} | {}", marker.kind, marker.id, marker.name),
            )
        } else {
            session.status = "Select an object, plant or marker to edit its transform".into();
            return;
        };
    let draft = Draft {
        id,
        revision: session.editor.revision(),
        fields: original.values(),
        original,
        subject,
        active: 0,
        selection: [0; 2],
        action: None,
    };
    for (entity, field, mut text) in &mut fields {
        // Preserve the font/layout styles Bevy prepared for this reusable field.
        text.clear();
        text.editor_mut()
            .set_text(draft.fields.get(field.0).map_or("", String::as_str));
        if field.0 == 0 {
            focus.set(entity, FocusCause::Navigated);
        }
    }
    session.transform_edit = Some(draft);
    session.status = "Editing transform - Apply validates all fields together".into();
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    fields: Query<(Entity, &Field, &EditableText)>,
    mut roots: Query<&mut Node, With<Root>>,
    mut captions: Query<&mut Text, With<Caption>>,
    mut rows: Query<(&FieldRow, &mut Node, &mut Visibility), Without<Root>>,
    mut labels: Query<(&FieldLabel, &mut Text), Without<Caption>>,
) {
    let Some(mut draft) = session.transform_edit.take() else {
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
        return;
    };
    let mut composing = false;
    let mut edited = false;
    for (row, mut node, mut visibility) in &mut rows {
        let visible = row.0 < draft.fields.len();
        // TabNavigation follows inherited visibility, not layout Display::None.
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (label, mut text) in &mut labels {
        if let Some(value) = draft.original.labels().get(label.0) {
            **text = (*value).into();
        }
    }
    for (entity, field, text) in &fields {
        if field.0 >= draft.fields.len() {
            continue;
        }
        let value = text.value().to_string();
        edited |= value != draft.fields[field.0];
        draft.fields[field.0] = value;
        if focus.get() == Some(entity) {
            draft.active = field.0;
        }
        composing |= text.is_composing();
    }
    if edited {
        session.status = "Editing transform - Apply validates all fields together".into();
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
        match draft
            .original
            .edit(draft.id, &draft.fields)
            .and_then(|edit| {
                let Some(edit) = edit else {
                    return Ok(false);
                };
                let count = session.selected_path_len();
                session.editor.execute(Command::Apply {
                    expected_revision: draft.revision,
                    edits: vec![edit],
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
            caption.0 = format!("TRANSFORM\n{}", draft.subject);
        }
        draft.action = None;
        session.transform_edit = Some(draft);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plant_marker_fields_preserve_metadata_and_share_candidate_history() {
        use theia_zone::{markers::MarkerKind, vegetation::PlantKind};
        for origin in [[0.; 3], [128_000_000_000., 20., -128_000_000_000.]] {
            for target in [
                Target::Plant(Plant {
                    id: 7,
                    name: "Tree".into(),
                    kind: PlantKind::Tree,
                    position: origin,
                    yaw_degrees: 0.,
                    scale: 1.,
                }),
                Target::Plant(Plant {
                    id: 7,
                    name: "Shrub".into(),
                    kind: PlantKind::Shrub,
                    position: origin,
                    yaw_degrees: 0.,
                    scale: 1.,
                }),
                Target::Marker(Marker {
                    id: 7,
                    name: "NPC".into(),
                    kind: MarkerKind::Npc,
                    position: origin,
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("npc.guide".into()),
                }),
                Target::Marker(Marker {
                    id: 7,
                    name: "Quest".into(),
                    kind: MarkerKind::Quest,
                    position: origin,
                    yaw_degrees: 0.,
                    radius_metres: 1.,
                    definition: Some("quest.note".into()),
                }),
            ] {
                let mut zone = Zone {
                    schema: 7,
                    origin,
                    spawn: origin,
                    ..default()
                };
                match &target {
                    Target::Plant(p) => zone.plants.push(p.clone()),
                    Target::Marker(m) => zone.markers.push(m.clone()),
                    _ => unreachable!(),
                }
                let mut editor = Editor::new(zone.clone()).unwrap();
                let mut fields = target.values();
                assert!(target.edit(7, &fields).unwrap().is_none());
                assert!(target.edit(7, &fields[..4]).is_err());
                for index in 0..5 {
                    let mut invalid = fields.clone();
                    invalid[index] = "NaN".into();
                    assert!(
                        target
                            .edit(7, &invalid)
                            .unwrap_err()
                            .contains(target.labels()[index])
                    );
                }
                fields[0] = (origin[0] + 8.125).to_string();
                fields[1] = (origin[1] + 2.25).to_string();
                fields[2] = (origin[2] - 8.5).to_string();
                fields[3] = "37.5".into();
                fields[4] = "1.25".into();
                editor
                    .execute(Command::Apply {
                        expected_revision: 0,
                        edits: vec![target.edit(7, &fields).unwrap().unwrap()],
                    })
                    .unwrap();
                let changed = editor.zone().clone();
                assert_eq!(editor.history(), (1, 0));
                match &target {
                    Target::Plant(p) => {
                        let mut expected = p.clone();
                        expected.position = [origin[0] + 8.125, origin[1] + 2.25, origin[2] - 8.5];
                        expected.yaw_degrees = 37.5;
                        expected.scale = 1.25;
                        assert_eq!(changed.plants, [expected]);
                    }
                    Target::Marker(m) => {
                        let mut expected = m.clone();
                        expected.position = [origin[0] + 8.125, origin[1] + 2.25, origin[2] - 8.5];
                        expected.yaw_degrees = 37.5;
                        expected.radius_metres = 1.25;
                        assert_eq!(changed.markers, [expected]);
                    }
                    _ => unreachable!(),
                }
                editor
                    .execute(Command::Undo {
                        expected_revision: 1,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &zone);
                fields[4] = "65".into();
                assert!(
                    editor
                        .execute(Command::Apply {
                            expected_revision: 2,
                            edits: vec![target.edit(7, &fields).unwrap().unwrap()]
                        })
                        .is_err()
                );
                assert_eq!(editor.history(), (0, 1));
                assert_eq!(editor.revision(), 2);
                assert_eq!(editor.zone(), &zone);
                editor
                    .execute(Command::Redo {
                        expected_revision: 2,
                    })
                    .unwrap();
                assert_eq!(editor.zone(), &changed);
            }
        }
    }
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
