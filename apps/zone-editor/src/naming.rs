//! Standard Bevy TextInput for shared rename commands and editor-local search.
use super::*;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::text::{EditableText, TextEdit};
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};

#[derive(Clone, Copy, Debug)]
pub(super) enum Target {
    Zone,
    Entry(u64),
    Search,
}
pub(super) struct Draft {
    target: Target,
    revision: u64,
    original: String,
    pub text: String,
    pub selection: [usize; 2],
    subject: String,
    action: Option<Control>,
}
impl Draft {
    pub fn is_authoring(&self) -> bool {
        !matches!(self.target, Target::Search)
    }
    fn hint(&self) -> &'static str {
        if self.is_authoring() {
            "Editing name - Enter applies, Escape cancels"
        } else {
            "Find content - Enter applies, Escape cancels"
        }
    }
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Field;
#[derive(Component)]
pub(super) struct Caption;
#[derive(Component)]
pub(super) struct Description;
#[derive(Component)]
pub(super) struct ApplyLabel;
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Apply,
    Cancel,
}

pub(super) fn setup(mut commands: Commands) {
    commands
        .spawn((
            Root,
            GlobalZIndex(20),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(theme::dialog::BACKDROP),
        ))
        .with_children(|root| {
            root.spawn(theme::dialog::panel(448.))
                .with_children(|panel| {
                    panel
                        .spawn(theme::dialog::header())
                        .with_child((Caption, theme::label("Rename", 18., theme::DISPLAY)));
                    scrolling::pane(panel, 24., scrolling::Pane::Dialog, |body| {
                        body.spawn((Description, theme::label("", 14., theme::MUTED)));
                        body.spawn((
                            Field,
                            TextInput,
                            SelectAllOnFocus,
                            EditableText {
                                max_characters: Some(80),
                                allow_newlines: false,
                                ..default()
                            },
                            Node {
                                width: percent(100),
                                ..theme::input_node()
                            },
                            BorderColor::all(theme::BORDER),
                            BackgroundColor(Color::NONE),
                            theme::input_font(),
                            TextColor(theme::BODY),
                            TextLayout::no_wrap(),
                            theme::text_cursor(),
                        ));
                        body.spawn((theme::label(
                            "Enter applies | Escape cancels",
                            12.,
                            theme::MUTED,
                        ),));
                    });
                    panel.spawn(theme::dialog::footer()).with_children(|row| {
                        for (control, label) in
                            [(Control::Cancel, "Cancel"), (Control::Apply, "Apply name")]
                        {
                            row.spawn((
                                control,
                                Button,
                                Hovered::default(),
                                Node {
                                    min_height: px(28),
                                    padding: UiRect::axes(px(12), px(6)),
                                    ..default()
                                },
                                BackgroundColor(theme::Variant::Default.background(false)),
                            ))
                            .with_children(|button| {
                                let mut label = button.spawn(theme::label(label, 12., theme::BODY));
                                if matches!(control, Control::Apply) {
                                    label.insert(ApplyLabel);
                                }
                            });
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
        && let Some(draft) = &mut session.name_edit
    {
        draft.action = Some(*control);
    }
}

pub(super) fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    mut fields: Query<(Entity, &mut EditableText), With<Field>>,
    mut requests: ResMut<workspace::Requests>,
    content: Res<outliner::Content>,
) {
    let requested = std::mem::take(&mut requests.name);
    let find = std::mem::take(&mut requests.find);
    if session.has_modal()
        || !(keys.just_pressed(KeyCode::F8) || requested || find)
        || !windows.single().is_ok_and(|w| w.focused)
    {
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
    {
        session.status = "Finish or cancel the preview before editing text".into();
        return;
    }
    let zone =
        !requested && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));
    let id = session
        .selected
        .or(session.path_selected)
        .or(session.marker_selected)
        .or(session.plant_selected);
    let target = if find {
        Target::Search
    } else if zone {
        Target::Zone
    } else {
        id.map_or(Target::Zone, Target::Entry)
    };
    let original = match target {
        Target::Zone => session.editor.zone().name.clone(),
        Target::Search => content.query.clone(),
        Target::Entry(id) => session
            .editor
            .zone()
            .authoring_name(id)
            .expect("Reconciled selection")
            .into(),
    };
    let Ok((entity, mut field)) = fields.single_mut() else {
        return;
    };
    // Keep Bevy's prepared font/layout styles when reusing the field. Replacing
    // EditableText with a fresh PlainEditor discards them while TextFont stays unchanged.
    field.clear();
    field.editor_mut().set_text(&original);
    field.queue_edit(TextEdit::SelectAll);
    focus.set(entity, FocusCause::Navigated);
    let subject = match target {
        Target::Zone => "zone".into(),
        Target::Search => "content (name, kind or #ID)".into(),
        Target::Entry(id) => {
            let zone = session.editor.zone();
            let kind = if zone.objects.iter().any(|o| o.id == id) {
                "object"
            } else if zone.paths.iter().any(|p| p.id == id) {
                "guide"
            } else if zone.markers.iter().any(|m| m.id == id) {
                "marker"
            } else {
                "plant"
            };
            format!("{kind} #{id}")
        }
    };
    let draft = Draft {
        target,
        subject,
        revision: session.editor.revision(),
        text: original.clone(),
        selection: [0; 2],
        original,
        action: None,
    };
    session.status = draft.hint().into();
    session.name_edit = Some(draft);
}

// Read after Bevy has applied queued text edits, including input received with Enter.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    fields: Query<(Entity, &EditableText), With<Field>>,
    mut roots: Query<&mut Node, With<Root>>,
    mut captions: Query<&mut Text, With<Caption>>,
    mut descriptions: Query<&mut Text, (With<Description>, Without<Caption>)>,
    mut apply_labels: Query<&mut Text, (With<ApplyLabel>, Without<Caption>, Without<Description>)>,
    mut content: ResMut<outliner::Content>,
) {
    let Some(mut draft) = session.name_edit.take() else {
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::None;
        }
        return;
    };
    let Ok((entity, field)) = fields.single() else {
        session.name_edit = Some(draft);
        return;
    };
    let text = field.value().to_string();
    if text != draft.text {
        session.status = draft.hint().into();
    }
    draft.text = text;
    let range = field.editor().raw_selection().text_range();
    draft.selection = [range.start, range.end];
    let cancel = !windows.single().is_ok_and(|w| w.focused)
        || keys.just_pressed(KeyCode::Escape)
        || matches!(draft.action, Some(Control::Cancel));
    let apply = !field.is_composing()
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || matches!(draft.action, Some(Control::Apply)));
    let close = if cancel {
        session.status = if draft.is_authoring() {
            "Name edit cancelled"
        } else {
            "Search cancelled"
        }
        .into();
        true
    } else if apply && !draft.is_authoring() {
        match content.set_query(&draft.text) {
            Ok(()) => {
                session.status = "Content search updated".into();
                true
            }
            Err(error) => {
                session.status = error;
                false
            }
        }
    } else if apply {
        if draft.text == draft.original {
            session.status = "Name unchanged".into();
            true
        } else {
            let edit = match draft.target {
                Target::Zone => Edit::RenameZone {
                    name: draft.text.clone(),
                },
                Target::Entry(id) => Edit::Rename {
                    id,
                    name: draft.text.clone(),
                },
                Target::Search => unreachable!("Search commits only editor-local preferences"),
            };
            let points_before = session.selected_path_len();
            match session.editor.execute(Command::Apply {
                expected_revision: draft.revision,
                edits: vec![edit],
            }) {
                Ok(()) => {
                    session.reconcile_after_edit(points_before);
                    session.status = "Name updated".into();
                    true
                }
                Err(error) => {
                    session.status = error;
                    false
                }
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
        // Keep the single field focused even after a click on the modal backdrop/buttons.
        if focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
        if let Ok(mut root) = roots.single_mut() {
            root.display = Display::Flex;
        }
        if let Ok(mut caption) = captions.single_mut() {
            caption.0 = format!(
                "{} {}",
                if draft.is_authoring() {
                    "Rename"
                } else {
                    "Find"
                },
                draft.subject
            );
        }
        if let Ok(mut description) = descriptions.single_mut() {
            description.0 = format!(
                "{} / {} UTF-8 bytes\n{}",
                draft.text.len(),
                if draft.is_authoring() { 80 } else { 128 },
                session.status
            );
        }
        if let Ok(mut label) = apply_labels.single_mut() {
            label.0 = if draft.is_authoring() {
                "Apply name"
            } else {
                "Find"
            }
            .into();
        }
        draft.action = None;
        session.name_edit = Some(draft);
    }
}
