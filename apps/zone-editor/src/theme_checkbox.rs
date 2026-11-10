//! Sprinkles checkbox.rs presentation using Bevy's native Checkbox behavior.
//! Pinned upstream MIT provenance: third_party/sprinkles/PROVENANCE.md.
use super::*;
use bevy::ui::{Checked, InteractionDisabled};
use bevy::ui_widgets::Checkbox;

#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Box;
#[derive(Component)]
pub(super) struct Mark;

pub(crate) fn spawn(parent: &mut ChildSpawnerCommands, control: workspace::Control, label: &str) {
    parent
        .spawn((
            Root,
            control,
            Checkbox,
            Hovered::default(),
            Node {
                min_height: px(28),
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|row| {
            row.spawn((
                Box,
                Pickable::IGNORE,
                Node {
                    width: px(16),
                    height: px(16),
                    flex_shrink: 0.,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(2)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderColor::all(BORDER),
            ))
            .with_children(|square| {
                // Draw the check with standard UI borders; no font glyph dependency.
                square.spawn((
                    Mark,
                    Pickable::IGNORE,
                    Node {
                        width: px(5),
                        height: px(8),
                        border: UiRect {
                            right: px(1.5),
                            bottom: px(1.5),
                            ..default()
                        },
                        margin: UiRect::bottom(px(2)),
                        display: Display::None,
                        ..default()
                    },
                    BorderColor::all(BODY),
                    UiTransform::from_rotation(Rot2::degrees(45.)),
                ));
            });
            row.spawn((
                workspace::ButtonLabel,
                text(label, 12.),
                TextLayout::no_wrap(),
            ));
        });
}

#[allow(clippy::type_complexity)]
pub(super) fn sync(
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    roots: Query<
        (
            Entity,
            &Hovered,
            Has<Checked>,
            Has<InteractionDisabled>,
            &Children,
        ),
        With<Root>,
    >,
    mut boxes: Query<(&Children, &mut BorderColor), With<Box>>,
    mut marks: Query<(&mut Node, &mut BorderColor), (With<Mark>, Without<Box>)>,
) {
    for (entity, hovered, checked, disabled, children) in &roots {
        for child in children.iter() {
            let Ok((children, mut border)) = boxes.get_mut(child) else {
                continue;
            };
            *border = BorderColor::all(if disabled {
                BORDER.with_alpha(0.5)
            } else if focus.as_ref().and_then(|focus| focus.get()) == Some(entity) {
                PRIMARY
            } else if hovered.0 {
                BORDER.lighter(0.05)
            } else {
                BORDER
            });
            for child in children.iter() {
                if let Ok((mut node, mut color)) = marks.get_mut(child) {
                    node.display = if checked {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    *color = BorderColor::all(if disabled { MUTED } else { BODY });
                }
            }
        }
    }
}
