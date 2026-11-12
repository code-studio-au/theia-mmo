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
            group.spawn((
                theme::label(
                    if spawn {
                        "Draft only · live population is not connected"
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
) {
    let snapshot = summary::snapshot(&session);
    for (group, mut node, mut visibility) in &mut groups {
        let visible = snapshot.as_ref().is_some_and(|(_, target, _)| {
            matches!(
                (group, target),
                (Group::Area, Target::QuestArea(_))
                    | (Group::Marker, Target::Marker(_))
                    | (Group::Population, Target::SpawnGroup(_))
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
