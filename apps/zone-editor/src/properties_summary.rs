//! Compact read-only attribute rows over the exact editor's validated snapshot.
use super::*;
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Row(usize);
#[derive(Component)]
struct Label(usize);
#[derive(Component)]
pub(crate) struct Value(pub(crate) usize);

fn snapshot(session: &Session) -> Option<(Target, String)> {
    if session.play.is_some() || session.palette.is_some() || session.path_draft.is_some() {
        return None;
    }
    let (id, mut target, subject) = capture(session, false)?;
    if let Target::Object(pose) = &mut target
        && let Some(drag) = session.drag.as_ref().filter(|drag| Some(drag.id) == id)
    {
        *pose = drag.preview.clone();
    }
    Some((target, subject))
}
pub(super) fn description(session: &Session) -> Option<String> {
    let (target, mut subject) = snapshot(session)?;
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
            Root,
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
                        Pickable::IGNORE,
                        Node {
                            min_width: px(0),
                            flex_grow: 1.,
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
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
        });
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, sync.after(crate::present));
}
#[allow(clippy::type_complexity)]
fn sync(
    session: Res<Session>,
    mut roots: Query<&mut Node, With<Root>>,
    mut rows: Query<(&Row, &mut Node), Without<Root>>,
    mut labels: Query<(&Label, &mut Text)>,
    mut values: Query<(&Value, &mut Text), Without<Label>>,
) {
    let snapshot = snapshot(&session);
    for mut node in &mut roots {
        let display = if snapshot.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let Some((target, _)) = snapshot else { return };
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
