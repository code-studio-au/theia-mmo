//! Grouped attributes with inline editors over the validated exact snapshot.
use super::*;
use crate::particle_workspace::ZoneAppExt;
#[derive(Component)]
pub(super) struct Root(pub(super) Option<Key>);
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Key {
    id: Option<u64>,
    kind: std::mem::Discriminant<Target>,
    point: Option<usize>,
    revision: u64,
    document_id: u64,
}
impl Key {
    pub(super) fn new(id: Option<u64>, target: &Target, session: &Session) -> Self {
        Self {
            id,
            kind: std::mem::discriminant(target),
            revision: session.editor.revision(),
            document_id: session.document_id,
            point: match target {
                Target::Path { point, .. } => *point,
                _ => None,
            },
        }
    }
}
#[derive(Component)]
pub(crate) struct Frame;
#[derive(Component)]
pub(crate) struct Name;
#[derive(Component)]
struct NameRow;
#[derive(Component)]
struct EffectRow;
#[derive(Component)]
struct Effect;
#[derive(Component)]
struct Row(usize);
#[derive(Component)]
struct Heading(usize);
#[derive(Component)]
struct Label(usize);
#[derive(Component)]
pub(crate) struct Value(pub(crate) usize);

pub(super) fn snapshot(session: &Session) -> Option<(Option<u64>, Target, String)> {
    if session.play.is_some() || session.palette.is_some() || session.path_draft.is_some() {
        return None;
    }
    let (id, mut target, subject) = capture(
        session,
        outliner::selection(session) == Some(outliner::Target::Spawn),
    )?;
    if let Target::Object(pose) = &mut target
        && let Some(drag) = session.drag.as_ref().filter(|drag| Some(drag.id) == id)
    {
        *pose = drag.preview.clone();
    }
    if let Target::TestPlayer(player) = &mut target
        && let Some(drag) = session.drag.as_ref().filter(|drag| Some(drag.id) == id)
    {
        player.position = drag.preview.position;
    }
    Some((id, target, subject))
}
pub(super) fn description(session: &Session) -> Option<String> {
    let (id, target, mut subject) = snapshot(session)?;
    if id.is_some() {
        // The editable name has its own permanent field below this metadata.
        subject = subject
            .split_once(" | ")
            .map_or(subject.clone(), |(kind, _)| kind.into());
        if let Target::Path {
            point: Some(index), ..
        } = &target
        {
            subject.push_str(&format!("\nPoint {}", index + 1));
        }
    }
    match target {
        Target::Particle(_) => subject.push_str("\nUnresolved asset reference"),
        Target::TestPlayer(player) => subject.push_str(&format!(
            "\n{} | {}",
            if player.enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            match player.controller.as_deref() {
                None => "Manual controller",
                Some("test.walk-turn") => "Walk and turn",
                Some(id) => id,
            }
        )),
        Target::Scenery(model) => subject.push_str(&format!("\n{}", model.asset)),
        Target::QuestArea(_) => subject.push_str("\nQuest references checked on catalog export"),
        Target::Marker(_) => subject.push_str("\nAuthoring annotation"),
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
            // Keep Tab within the visible numeric form; mouse focus can leave it.
            TabGroup::modal(),
            Visibility::Hidden,
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
            root.spawn((
                NameRow,
                Visibility::Inherited,
                Node {
                    width: percent(100),
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    flex_shrink: 0.,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    theme::field_label("Name"),
                    Node {
                        width: px(88),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                field(row, super::inline::Input::Name, 0);
            });
            root.spawn((
                EffectRow,
                Visibility::Hidden,
                Node {
                    display: Display::None,
                    width: percent(100),
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    flex_shrink: 0.,
                    ..default()
                },
            ))
            .with_children(|row| {
                row.spawn((
                    theme::field_label("Effect ID"),
                    Node {
                        width: px(88),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                field(row, super::inline::Input::Effect, 1);
            });
            super::area::spawn(root);
            for index in 0..7 {
                if matches!(index, 0 | 3 | 4) {
                    root.spawn((
                        Heading(index),
                        theme::label("", 12., theme::DISPLAY),
                        Pickable::IGNORE,
                        Node {
                            display: Display::None,
                            margin: UiRect::top(px(8)),
                            ..default()
                        },
                    ));
                }
                root.spawn((
                    Row(index),
                    Visibility::Hidden,
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
                    field(row, super::inline::Input::Number(index), index as i32 + 4);
                });
            }
        });
}
pub(super) fn field(parent: &mut ChildSpawnerCommands, input: super::inline::Input, order: i32) {
    parent
        .spawn((
            Frame,
            Node {
                min_width: px(0),
                width: px(0),
                flex_grow: 1.,
                padding: UiRect::horizontal(px(6)),
                height: px(28),
                min_height: px(28),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(2)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(theme::BORDER),
            BackgroundColor(theme::Variant::Default.background(false)),
        ))
        .with_children(|frame| {
            let mut entity = frame.spawn((
                input,
                FramedInput,
                TextInput,
                SelectAllOnFocus,
                TabIndex(order),
                Hovered::default(),
                EditableText {
                    max_characters: Some(match input {
                        super::inline::Input::Name => 80,
                        super::inline::Input::Effect
                        | super::inline::Input::AreaDefinition
                        | super::inline::Input::MarkerDefinition => 96,
                        super::inline::Input::AreaObjectives => {
                            theia_core::quests::areas::MAX_MAP_OBJECTIVES_PER_AREA * 98
                        }
                        _ => 64,
                    }),
                    allow_newlines: false,
                    ..default()
                },
                Node {
                    min_width: px(0),
                    flex_grow: 1.,
                    height: Val::Auto,
                    min_height: px(16),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::NONE),
                theme::input_font(),
                TextColor(theme::BODY),
                TextLayout::no_wrap(),
                theme::text_cursor(),
            ));
            match input {
                super::inline::Input::Number(index) => {
                    entity.insert(Value(index));
                }
                super::inline::Input::Name => {
                    entity.insert(Name);
                }
                super::inline::Input::Effect => {
                    entity.insert(Effect);
                }
                super::inline::Input::AreaDefinition
                | super::inline::Input::AreaObjectives
                | super::inline::Input::MarkerDefinition => {
                    entity.insert(super::area::TextField);
                }
            }
        });
}
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((super::inline::plugin, super::area::plugin))
        .add_zone_systems(sync.after(crate::present));
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sync(
    session: Res<Session>,
    mut roots: Query<(&mut Root, &mut Node, &mut Visibility)>,
    mut rows: Query<
        (&Row, &mut Node, &mut Visibility),
        (Without<Root>, Without<NameRow>, Without<EffectRow>),
    >,
    mut labels: Query<(&Label, &mut Text)>,
    mut values: Query<
        (Entity, &Value, &mut EditableText),
        (Without<Label>, Without<Name>, Without<Effect>),
    >,
    mut names: Query<(Entity, &mut EditableText), (With<Name>, Without<Value>, Without<Effect>)>,
    mut effects: Query<(Entity, &mut EditableText), (With<Effect>, Without<Value>, Without<Name>)>,
    mut name_rows: Query<
        (&mut Node, &mut Visibility),
        (
            With<NameRow>,
            Without<Root>,
            Without<Row>,
            Without<Heading>,
            Without<EffectRow>,
        ),
    >,
    mut effect_rows: Query<
        (&mut Node, &mut Visibility),
        (
            With<EffectRow>,
            Without<Root>,
            Without<Row>,
            Without<Heading>,
            Without<NameRow>,
        ),
    >,
    inputs: Query<(Entity, Has<InteractionDisabled>), With<super::inline::Input>>,
    mut commands: Commands,
    focus: Res<InputFocus>,
    editing: Res<super::inline::State>,
    mut headings: Query<
        (&Heading, &mut Node, &mut Text),
        (
            Without<Root>,
            Without<Row>,
            Without<Label>,
            Without<Value>,
            Without<NameRow>,
            Without<EffectRow>,
        ),
    >,
) {
    let snapshot = snapshot(&session);
    for (mut root, mut node, mut visibility) in &mut roots {
        *visibility = if snapshot.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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
    for (entity, disabled) in &inputs {
        if disabled == enabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
    }
    let name = snapshot
        .as_ref()
        .and_then(|(id, _, _)| session.editor.zone().authoring_name((*id)?));
    for (mut node, mut visibility) in &mut name_rows {
        node.display = if name.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if name.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(name) = name {
        for (entity, mut text) in &mut names {
            if focus.get() != Some(entity) && !editing.owns(entity) && text.value() != name {
                text.editor_mut().set_text(name);
            }
        }
    }
    let effect = snapshot.as_ref().and_then(|(_, target, _)| match target {
        Target::Particle(particle) => Some(particle.effect.as_str()),
        _ => None,
    });
    for (mut node, mut visibility) in &mut effect_rows {
        node.display = if effect.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if effect.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(effect) = effect {
        for (entity, mut text) in &mut effects {
            if focus.get() != Some(entity) && !editing.owns(entity) && text.value() != effect {
                text.editor_mut().set_text(effect);
            }
        }
    }
    let Some((_, target, _)) = snapshot else {
        return;
    };
    let fields = target.values();
    for (heading, mut node, mut text) in &mut headings {
        let title = match (&target, heading.0) {
            (Target::TestPlayer(_), 3) => "Network conditions",
            (Target::QuestArea(_), 0) => "Minimum bounds",
            (Target::QuestArea(_), 3) => "Maximum bounds",
            (Target::QuestArea(_), _) => "",
            (Target::Path { point: None, .. }, 0) => "Path",
            (_, 0) => "Position",
            (Target::Path { point: Some(_), .. }, 3) => "Path",
            (_, 3) => "Rotation",
            (Target::Object(_), 4) => "Dimensions",
            (Target::Plant(_), 4) => "Scale",
            (Target::Particle(_) | Target::Scenery(_), 4) => "Scale",
            (Target::Marker(_), 4) => "Area",
            _ => "",
        };
        node.display = if heading.0 < fields.len() && !title.is_empty() {
            Display::Flex
        } else {
            Display::None
        };
        **text = title.into();
    }
    for (row, mut node, mut visibility) in &mut rows {
        *visibility = if row.0 < fields.len() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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
    for (entity, value, mut text) in &mut values {
        if focus.get() != Some(entity)
            && !editing.owns(entity)
            && let Some(next) = fields.get(value.0)
            && text.value() != next.as_str()
        {
            text.editor_mut().set_text(next);
        }
    }
}

#[cfg(test)]
#[path = "properties_summary_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "properties_summary_actions_tests.rs"]
mod action_tests;

#[cfg(test)]
#[path = "properties_summary_workspace_tests.rs"]
mod workspace_tests;

#[cfg(test)]
#[path = "properties_binding_gpu_test.rs"]
mod binding_gpu_test;
