//! Quest identity and map links share the permanent inline text/history route.
use super::*;
use crate::particle_workspace::ZoneAppExt;

#[derive(Component)]
struct Group;
#[derive(Component)]
pub(super) struct TextField;

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Group,
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
                theme::label("Quest binding", 12., theme::DISPLAY),
                Pickable::IGNORE,
            ));
            for (input, title, order) in [
                (inline::Input::AreaDefinition, "Area ID", 2),
                (inline::Input::AreaObjectives, "Map objectives", 3),
            ] {
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
                theme::label("Objective IDs, separated by commas", 10., theme::MUTED),
                Pickable::IGNORE,
            ));
        });
}

pub(super) fn plugin(app: &mut App) {
    app.add_zone_systems(sync.after(crate::present));
}

fn sync(
    session: Res<Session>,
    mut groups: Query<(&mut Node, &mut Visibility), With<Group>>,
    mut fields: Query<(Entity, &inline::Input, &mut EditableText), With<TextField>>,
    focus: Res<InputFocus>,
    editing: Res<inline::State>,
) {
    let snapshot = summary::snapshot(&session);
    let area = snapshot
        .as_ref()
        .filter(|(_, target, _)| matches!(target, Target::QuestArea(_)));
    for (mut node, mut visibility) in &mut groups {
        node.display = if area.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if area.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some((id, target, _)) = area {
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
