//! Exact inline numeric edits over the shared validated authoring commands.
//! Sprinkles' numeric f32 formatting cannot represent global f64 positions.
use super::*;

#[derive(Component, Clone, Copy)]
pub(crate) enum Input {
    Number(usize),
    Name,
    Effect,
    MarkerDefinition,
    AreaDefinition,
    AreaObjectives,
}
impl Input {
    pub(super) fn value(
        self,
        session: &Session,
        id: Option<u64>,
        target: &Target,
    ) -> Option<String> {
        match self {
            Self::Number(index) => target.values().get(index).cloned(),
            Self::Name => session.editor.zone().authoring_name(id?).map(String::from),
            Self::MarkerDefinition => match target {
                Target::Marker(marker) => Some(marker.definition.clone().unwrap_or_default()),
                _ => None,
            },
            Self::AreaDefinition | Self::AreaObjectives => match target {
                Target::QuestArea(area) => Some(if matches!(self, Self::AreaDefinition) {
                    area.definition.clone()
                } else {
                    area.map_objectives.join(", ")
                }),
                _ => None,
            },
            Self::Effect => match target {
                Target::Particle(particle) => Some(particle.effect.clone()),
                _ => None,
            },
        }
    }
}
struct Pending {
    entity: Entity,
    id: Option<u64>,
    original: Target,
    revision: u64,
    document_id: u64,
    field: Input,
    original_text: String,
}
#[derive(Resource, Default)]
pub(crate) struct State {
    active: Option<Pending>,
}

impl State {
    pub(super) fn owns(&self, entity: Entity) -> bool {
        self.active
            .as_ref()
            .is_some_and(|pending| pending.entity == entity)
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<State>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<InputFocus>()
        .add_observer(begin)
        .add_observer(focus_frame)
        .add_systems(
            PostUpdate,
            finish
                .after(bevy::text::EditableTextSystems)
                .run_if(crate::particle_workspace::zone_active),
        );
}
fn commit(session: &mut Session, pending: &Pending, value: &str) -> Result<(), String> {
    if session.document_id != pending.document_id || session.editor.revision() != pending.revision {
        return Err("Attributes changed while editing; edit the current value again".into());
    }
    let edit = match pending.field {
        Input::Number(index) => {
            let mut fields = pending.original.values();
            fields[index] = value.into();
            pending.original.edit(pending.id, &fields)?
        }
        Input::Name => (value != pending.original_text).then(|| Edit::Rename {
            id: pending.id.expect("Name fields require authored content"),
            name: value.into(),
        }),
        Input::MarkerDefinition => match &pending.original {
            Target::Marker(original) => {
                let mut marker = original.clone();
                marker.definition = (!value.trim().is_empty()).then(|| value.trim().to_owned());
                (marker != *original).then_some(Edit::PutMarker { marker })
            }
            _ => return Err("Marker is no longer available".into()),
        },
        Input::AreaDefinition | Input::AreaObjectives => match &pending.original {
            Target::QuestArea(original) => {
                let mut area = original.clone();
                if matches!(pending.field, Input::AreaDefinition) {
                    area.definition = value.trim().into();
                } else {
                    area.map_objectives = if value.trim().is_empty() {
                        Vec::new()
                    } else {
                        value.split(',').map(|id| id.trim().to_string()).collect()
                    };
                }
                (area != *original).then_some(Edit::PutQuestArea { area })
            }
            _ => return Err("Quest area is no longer available".into()),
        },
        Input::Effect => match &pending.original {
            Target::Particle(original) if value != pending.original_text => {
                let mut particle = original.clone();
                particle.effect = value.into();
                Some(Edit::PutParticle { particle })
            }
            Target::Particle(_) => None,
            _ => return Err("Effect reference is no longer available".into()),
        },
    };
    if let Some(edit) = edit {
        let points = session.selected_path_len();
        session.editor.execute(Command::Apply {
            expected_revision: pending.revision,
            edits: vec![edit],
        })?;
        session.reconcile_after_edit(points);
        session.status = "Attribute updated · Save writes the file".into();
    }
    Ok(())
}
fn focus_frame(
    event: On<PointerPress>,
    frames: Query<&Children, With<summary::Frame>>,
    inputs: Query<(), With<Input>>,
    session: Res<Session>,
    windows: Query<&Window>,
    mut focus: ResMut<InputFocus>,
) {
    if event.button != PointerButton::Primary
        || workspace::busy(&session)
        || !windows.single().is_ok_and(|w| w.focused)
    {
        return;
    }
    if let Ok(children) = frames.get(event.entity)
        && let Some(entity) = children.iter().find(|entity| inputs.contains(*entity))
    {
        focus.set(entity, FocusCause::Pressed);
    }
}
fn refresh_key(session: &Session, roots: &mut Query<&mut summary::Root>) {
    let key =
        summary::snapshot(session).map(|(id, target, _)| summary::Key::new(id, &target, session));
    for mut root in roots.iter_mut() {
        root.0 = key;
    }
}
#[allow(clippy::too_many_arguments)]
fn begin(
    event: On<bevy::input_focus::FocusGained>,
    mut inputs: Query<(&Input, &mut EditableText)>,
    mut roots: Query<&mut summary::Root>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut state: ResMut<State>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok((input, _)) = inputs.get(event.entity) else {
        return;
    };
    let field = *input;
    if state
        .active
        .as_ref()
        .is_some_and(|pending| pending.entity == event.entity)
    {
        return;
    }
    if workspace::busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Some((id, target, _)) = summary::snapshot(&session) else {
        focus.clear();
        return;
    };
    if field.value(&session, id, &target).is_none()
        || !roots
            .single()
            .is_ok_and(|root| root.0 == Some(summary::Key::new(id, &target, &session)))
    {
        session.status = "Selection changed; choose an updated attribute".into();
        focus.clear();
        return;
    }
    if let Some(previous) = state.active.take()
        && let Ok((_, text)) = inputs.get(previous.entity)
    {
        match commit(&mut session, &previous, &text.value().to_string()) {
            Ok(()) => refresh_key(&session, &mut roots),
            Err(error) => session.status = error,
        }
    }
    let Some((id, target, _)) = summary::snapshot(&session) else {
        return;
    };
    // The permanent control now represents this exact snapshot, even when a
    // document/selection change coincides with keyboard navigation.
    let Some(original_text) = field.value(&session, id, &target) else {
        focus.clear();
        return;
    };
    if let Ok((_, mut text)) = inputs.get_mut(event.entity) {
        text.editor_mut().set_text(&original_text);
    }
    state.active = Some(Pending {
        entity: event.entity,
        id,
        original: target,
        field,
        original_text,
        revision: session.editor.revision(),
        document_id: session.document_id,
    });
    session.inline_editing = true;
    session.inline_dirty = false;
}
#[allow(clippy::too_many_arguments)]
fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut state: ResMut<State>,
    mut session: ResMut<Session>,
    mut focus: ResMut<InputFocus>,
    mut inputs: Query<&mut EditableText, With<Input>>,
    mut roots: Query<&mut summary::Root>,
    mut view: Option<ResMut<View>>,
) {
    let Some(pending) = state.active.take() else {
        return;
    };
    let Ok(text) = inputs.get(pending.entity) else {
        session.inline_editing = false;
        session.inline_dirty = false;
        session.inline_action = None;
        return;
    };
    session.inline_dirty = text.value() != pending.original_text.as_str();
    let cancel = keys.just_pressed(KeyCode::Escape) || !windows.single().is_ok_and(|w| w.focused);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let save = ctrl && !shift && keys.just_pressed(KeyCode::KeyS);
    let apply = !text.is_composing()
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || save
            || session.inline_action.is_some()
            || focus.get() != Some(pending.entity));
    if !cancel && !apply {
        state.active = Some(pending);
        return;
    }
    if !cancel {
        match commit(&mut session, &pending, &text.value().to_string()) {
            Ok(()) => refresh_key(&session, &mut roots),
            Err(error) => {
                session.status = error;
                session.inline_action = None;
                if focus.get() == Some(pending.entity)
                    && session.editor.revision() == pending.revision
                    && session.document_id == pending.document_id
                {
                    state.active = Some(pending);
                    return;
                }
            }
        }
    }
    if let Ok(mut text) = inputs.get_mut(pending.entity) {
        let canonical = summary::snapshot(&session)
            .and_then(|(id, target, _)| pending.field.value(&session, id, &target))
            .unwrap_or_default();
        text.editor_mut().set_text(&canonical);
    }
    if focus.get() == Some(pending.entity) {
        focus.clear();
    }
    session.inline_editing = false;
    session.inline_dirty = false;
    let action = session
        .inline_action
        .take()
        .or(save.then_some(Action::Save));
    if !cancel
        && let Some(action) = action
        && let Some(view) = view.as_deref_mut()
    {
        perform(action, &mut session, view);
    }
}
