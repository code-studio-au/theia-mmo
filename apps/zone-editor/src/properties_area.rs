//! Quest identity and map links share the permanent inline text/history route.
use super::*;
use crate::particle_workspace::ZoneAppExt;

#[derive(Component)]
enum Group {
    Area,
    Marker,
    Population,
}
#[derive(Component)]
pub(super) struct TextField;
#[derive(Component)]
struct PopulationStatus;
#[derive(PartialEq)]
struct PopulationKey {
    document: u64,
    revision: u64,
    group: Option<u64>,
    context: Option<u64>,
    creature: Option<u64>,
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    spawn_group(parent, Group::Area);
    spawn_group(parent, Group::Marker);
    spawn_group(parent, Group::Population);
}
fn spawn_group(parent: &mut ChildSpawnerCommands, group: Group) {
    let marker = matches!(group, Group::Marker);
    let spawn = matches!(group, Group::Population);
    let fields = if spawn {
        vec![
            (inline::Input::SpawnCreature, "Creature ID", 2),
            (inline::Input::SpawnModel, "Model ID", 3),
            (inline::Input::SpawnSeed, "Seed", 4),
            (inline::Input::SpawnPatrol, "Patrol path ID", 5),
        ]
    } else if marker {
        vec![(inline::Input::MarkerDefinition, "Catalog ID", 2)]
    } else {
        vec![
            (inline::Input::AreaDefinition, "Area ID", 2),
            (inline::Input::AreaObjectives, "Map objectives", 3),
        ]
    };
    parent
        .spawn((
            group,
            Visibility::Hidden,
            Node {
                display: Display::None,
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                flex_shrink: 0.,
                ..default()
            },
        ))
        .with_children(|group| {
            group.spawn((
                theme::label(
                    if spawn {
                        "Population reference"
                    } else if marker {
                        "Gameplay reference"
                    } else {
                        "Quest binding"
                    },
                    12.,
                    theme::DISPLAY,
                ),
                Pickable::IGNORE,
            ));
            if spawn {
                theme::checkbox::spawn(group, workspace::Control::SpawnEnabled, "Enabled");
                group.spawn(workspace::button(
                    workspace::Control::Command(Action::BindSpawnCreature),
                    "Use loaded creature",
                ));
                group.spawn((
                    PopulationStatus,
                    theme::label("", 11., theme::MUTED),
                    Pickable::IGNORE,
                ));
                group
                    .spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(4),
                        row_gap: px(4),
                        ..default()
                    })
                    .with_children(|row| {
                        for (control, label) in [
                            (
                                workspace::Control::SpawnBoundaryPoint(-1),
                                "Previous vertex",
                            ),
                            (workspace::Control::SpawnBoundaryPoint(1), "Next vertex"),
                            (workspace::Control::SpawnBoundaryWhole, "Whole boundary"),
                            (workspace::Control::SpawnBoundaryInsert, "Split edge"),
                            (workspace::Control::SpawnBoundaryRemove, "Remove vertex"),
                        ] {
                            row.spawn(workspace::button(control, label));
                        }
                    });
            }
            for (input, title, order) in fields {
                group
                    .spawn(Node {
                        width: percent(100),
                        column_gap: px(8),
                        align_items: AlignItems::Center,
                        flex_shrink: 0.,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            theme::field_label(title),
                            Node {
                                width: px(88),
                                flex_shrink: 0.,
                                ..default()
                            },
                        ));
                        summary::field(row, input, order);
                    });
            }
            if spawn {
                group.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(4),
                    row_gap: px(4),
                    ..default()
                }).with_children(|row| {
                    for (action, label) in [
                        (Action::SpawnPatrolChoice(-1), "Previous patrol"),
                        (Action::SpawnPatrolChoice(1), "Next patrol"),
                        (Action::SpawnPatrolChoice(0), "Clear patrol"),
                        (Action::ViewSpawnPatrol, "View patrol"),
                    ] {
                        row.spawn(workspace::button(workspace::Control::Command(action), label));
                    }
                });
            }
            group.spawn((
                theme::label(
                    if spawn {
                        "Patrol: saved Patrol guide ID; assignment sets roaming to zero. Blank clears it. File: export population. Zone: start playtest."
                    } else if marker {
                        "Blank leaves this marker unbound"
                    } else {
                        "Objective IDs, separated by commas"
                    },
                    10.,
                    theme::MUTED,
                ),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn plugin(app: &mut App) {
    app.add_zone_systems(sync.after(crate::present));
}

fn sync(
    session: Res<Session>,
    mut groups: Query<(&Group, &mut Node, &mut Visibility)>,
    mut fields: Query<(Entity, &inline::Input, &mut EditableText), With<TextField>>,
    focus: Res<InputFocus>,
    editing: Res<inline::State>,
    mut population_status: Query<&mut Text, With<PopulationStatus>>,
    mut population_key: Local<Option<PopulationKey>>,
) {
    let snapshot = summary::snapshot(&session);
    let key = PopulationKey {
        document: session.document_id,
        revision: session.editor.revision(),
        group: session.spawn_group_selected,
        context: session.quest_context.as_ref().map(|c| c.generation()),
        creature: session.creature_context.as_ref().map(|c| c.generation),
    };
    if population_key.as_ref() != Some(&key) && !population_status.is_empty() {
        let status = crate::spawn_groups::inspection(&session);
        for mut text in &mut population_status {
            if text.0 != status {
                text.0.clone_from(&status);
            }
        }
        *population_key = Some(key);
    }
    for (group, mut node, mut visibility) in &mut groups {
        let visible = snapshot.as_ref().is_some_and(|(_, target, _)| {
            matches!(
                (group, target),
                (Group::Area, Target::QuestArea(_))
                    | (Group::Marker, Target::Marker(_))
                    | (Group::Population, Target::SpawnGroup(_, _))
            )
        });
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some((id, target, _)) = &snapshot {
        for (entity, input, mut text) in &mut fields {
            if focus.get() != Some(entity)
                && !editing.owns(entity)
                && let Some(value) = input.value(&session, *id, target)
                && text.value() != value.as_str()
            {
                text.editor_mut().set_text(&value);
            }
        }
    }
}
