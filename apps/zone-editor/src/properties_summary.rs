//! Compact read-only attribute rows over the exact editor's validated snapshot.
use super::*;
#[derive(Component)]
struct Root(Option<Key>);
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    id: Option<u64>,
    kind: std::mem::Discriminant<Target>,
    point: Option<usize>,
    revision: u64,
}
impl Key {
    fn new(id: Option<u64>, target: &Target, session: &Session) -> Self {
        Self {
            id,
            kind: std::mem::discriminant(target),
            revision: session.editor.revision(),
            point: match target {
                Target::Path { point, .. } => *point,
                _ => None,
            },
        }
    }
}
#[derive(Component)]
struct ValueButton(usize);
#[derive(Component)]
struct Row(usize);
#[derive(Component)]
struct Label(usize);
#[derive(Component)]
pub(crate) struct Value(pub(crate) usize);

fn snapshot(session: &Session) -> Option<(Option<u64>, Target, String)> {
    if session.play.is_some() || session.palette.is_some() || session.path_draft.is_some() {
        return None;
    }
    let (id, mut target, subject) = capture(session, false)?;
    if let Target::Object(pose) = &mut target
        && let Some(drag) = session.drag.as_ref().filter(|drag| Some(drag.id) == id)
    {
        *pose = drag.preview.clone();
    }
    Some((id, target, subject))
}
pub(super) fn description(session: &Session) -> Option<String> {
    let (_, target, mut subject) = snapshot(session)?;
    match target {
        Target::Marker(marker) => subject.push_str(&format!(
            "\nDefinition: {}\nAuthoring annotation",
            marker.definition.as_deref().unwrap_or("unbound")
        )),
        Target::Path { path, .. } => subject.push_str(&format!("\n{} points", path.points.len())),
        Target::Object(_)
            if session
                .drag
                .as_ref()
                .is_some_and(|drag| session.selected == Some(drag.id)) =>
        {
            subject.push_str("\nTransform preview")
        }
        _ => {}
    }
    Some(subject)
}
pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Root(None),
            Pickable::IGNORE,
            Node {
                display: Display::None,
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|root| {
            for index in 0..7 {
                root.spawn((
                    Row(index),
                    Pickable::IGNORE,
                    Node {
                        display: Display::None,
                        width: percent(100),
                        min_height: px(20),
                        column_gap: px(8),
                        align_items: AlignItems::Center,
                        flex_shrink: 0.,
                        ..default()
                    },
                ))
                .with_children(|row| {
                    row.spawn((
                        Label(index),
                        Text::new(""),
                        TextFont {
                            weight: FontWeight::MEDIUM,
                            ..theme::font(10.)
                        },
                        TextColor(theme::MUTED),
                        TextLayout::no_wrap(),
                        Pickable::IGNORE,
                        Node {
                            width: px(88),
                            flex_shrink: 0.,
                            ..default()
                        },
                    ));
                    row.spawn((
                        ValueButton(index),
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        theme::Variant::Ghost,
                        Node {
                            min_width: px(0),
                            width: px(0),
                            flex_grow: 1.,
                            padding: UiRect::horizontal(px(6)),
                            height: px(28),
                            min_height: px(28),
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
                        BackgroundColor(theme::Variant::Ghost.background(false)),
                    ))
                    .with_child((
                        Value(index),
                        Hovered::default(),
                        Text::new(""),
                        theme::input_font(),
                        TextColor(theme::BODY),
                        TextLayout::no_wrap(),
                        Pickable::IGNORE,
                    ));
                });
            }
        })
        .with_child((
            theme::label("Select a value to edit attributes", 10., theme::MUTED),
            Pickable::IGNORE,
        ));
}
pub(super) fn plugin(app: &mut App) {
    app.add_observer(open_value)
        .add_systems(Update, sync.after(crate::present));
}
fn open_value(
    event: On<Activate>,
    buttons: Query<&ValueButton>,
    roots: Query<&Root>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut requests: ResMut<workspace::Requests>,
) {
    let Ok(button) = buttons.get(event.entity) else {
        return;
    };
    if workspace::busy(&session) || !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    let Some((id, target, _)) = snapshot(&session) else {
        return;
    };
    if button.0 >= target.labels().len() {
        return;
    }
    if !roots
        .single()
        .is_ok_and(|root| root.0 == Some(Key::new(id, &target, &session)))
    {
        session.status = "Selection changed; choose an updated attribute".into();
        return;
    }
    requests.transform = true;
    requests.transform_field = Some(button.0);
}
#[allow(clippy::type_complexity)]
fn sync(
    session: Res<Session>,
    mut roots: Query<(&mut Root, &mut Node)>,
    mut rows: Query<(&Row, &mut Node), Without<Root>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut values: Query<(&Value, &mut Text), Without<Label>>,
    buttons: Query<(Entity, Has<InteractionDisabled>), With<ValueButton>>,
    mut commands: Commands,
) {
    let snapshot = snapshot(&session);
    for (mut root, mut node) in &mut roots {
        root.0 = snapshot
            .as_ref()
            .map(|(id, target, _)| Key::new(*id, target, &session));
        let display = if snapshot.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let enabled = !workspace::busy(&session);
    for (entity, disabled) in &buttons {
        if disabled == enabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
    }
    let Some((_, target, _)) = snapshot else {
        return;
    };
    let fields = target.values();
    for (row, mut node) in &mut rows {
        let display = if row.0 < fields.len() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (label, mut text) in &mut labels {
        if let Some(next) = target.labels().get(label.0)
            && **text != *next
        {
            **text = (*next).into();
        }
    }
    for (value, mut text) in &mut values {
        if let Some(next) = fields.get(value.0)
            && **text != *next
        {
            **text = next.clone();
        }
    }
}

#[cfg(test)]
#[path = "properties_summary_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "properties_summary_actions_tests.rs"]
mod action_tests;
