//! Sprinkles data-panel row actions using native Bevy menus and shared commands.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::ui::OverrideClip;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};

#[derive(Component)]
pub(super) struct State {
    row: usize,
    snapshot: Option<(outliner::Target, u64)>,
}
impl State {
    pub(super) fn valid(&self, session: &Session, content: &outliner::Content) -> bool {
        self.snapshot.is_some_and(|(target, revision)| {
            session.editor.revision() == revision
                && outliner::selection(session) == Some(target)
                && content
                    .row(self.row)
                    .is_some_and(|entry| entry.target == target)
        })
    }
}
#[derive(Component)]
struct More(usize);
fn editable(target: outliner::Target) -> bool {
    matches!(
        target,
        outliner::Target::Object(_)
            | outliner::Target::Guide(_)
            | outliner::Target::Marker(_)
            | outliner::Target::Plant(_)
    )
}
pub(super) fn plugin(app: &mut App) {
    app.add_observer(right_click)
        .add_zone_systems(sync.after(outliner::sync));
}
pub(super) fn spawn(parent: &mut ChildSpawnerCommands, row: usize) {
    let mut root = parent.spawn((
        State {
            row,
            snapshot: None,
        },
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            right: px(0),
            top: px(0),
            width: px(24),
            height: px(24),
            ..default()
        },
    ));
    let owner = root.id();
    let mut button = Entity::PLACEHOLDER;
    let mut popup = Entity::PLACEHOLDER;
    root.with_children(|root| {
        button = root
            .spawn((
                More(row),
                Trigger("Content actions"),
                Button,
                MenuButton,
                theme::Variant::Ghost,
                Hovered::default(),
                Node {
                    width: px(24),
                    height: px(24),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|dots| {
                for _ in 0..3 {
                    dots.spawn((
                        Pickable::IGNORE,
                        Node {
                            width: px(2),
                            height: px(2),
                            border_radius: BorderRadius::all(px(1)),
                            ..default()
                        },
                        BackgroundColor(theme::BODY),
                    ));
                }
            })
            .id();
        popup = root
            .spawn((
                Popup("Content actions"),
                MenuPopup::default(),
                OverrideClip,
                Hovered::default(),
                GlobalZIndex(18),
                Popover {
                    window_margin: 8.,
                    positions: [
                        (PopoverSide::Right, PopoverAlign::Start),
                        (PopoverSide::Right, PopoverAlign::End),
                        (PopoverSide::Left, PopoverAlign::Start),
                        (PopoverSide::Left, PopoverAlign::End),
                    ]
                    .map(|(side, align)| PopoverPlacement {
                        side,
                        align,
                        gap: 4.,
                    })
                    .into(),
                },
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    width: px(240),
                    padding: UiRect::all(px(4)),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER),
            ))
            .with_children(|menu| {
                spawn_items(
                    menu,
                    &[
                        (workspace::Control::Rename, "Rename…", "F8"),
                        (workspace::Control::Transform, "Focus attributes", "F9"),
                        (
                            workspace::Control::Command(Action::Duplicate),
                            "Duplicate",
                            "Ctrl D",
                        ),
                        (workspace::Control::Command(Action::Ground), "Ground", ""),
                        (workspace::Control::Frame, "Frame", "F"),
                        (workspace::Control::Command(Action::Delete), "Delete", "Del"),
                    ],
                )
            })
            .id();
    });
    root.observe(
        move |event: On<MenuEvent>,
              windows: Query<&Window>,
              content: Res<outliner::Content>,
              mut session: ResMut<Session>,
              mut view: ResMut<View>,
              mut states: Query<&mut State>,
              mut popups: Query<(&mut Node, &mut MenuFocusState), With<MenuPopup>>,
              mut focus: ResMut<InputFocus>| {
            let Ok(mut context) = states.get_mut(owner) else {
                return;
            };
            let Ok((mut node, mut menu_focus)) = popups.get_mut(popup) else {
                return;
            };
            let opening = match event.action {
                MenuAction::Open(nav) => Some(nav),
                MenuAction::Toggle if node.display == Display::None => Some(NavAction::First),
                _ => None,
            };
            if let Some(nav) = opening {
                if workspace::busy(&session)
                    || !windows.single().is_ok_and(|window| window.focused)
                    || !content.row(row).is_some_and(|entry| editable(entry.target))
                {
                    return;
                }
                if let Some(target) =
                    outliner::select_row(row, 1, &content, &mut session, &mut view)
                {
                    context.snapshot = Some((target, session.editor.revision()));
                    node.display = Display::Flex;
                    *menu_focus = MenuFocusState::Opening(nav);
                }
            } else {
                match event.action {
                    MenuAction::CloseAll | MenuAction::Toggle => {
                        node.display = Display::None;
                        *menu_focus = MenuFocusState::Closed;
                    }
                    MenuAction::FocusRoot => focus.set(button, FocusCause::Navigated),
                    _ => {}
                }
            }
        },
    );
}
fn right_click(
    event: On<PointerPress>,
    rows: Query<&outliner::Control>,
    buttons: Query<(Entity, &More)>,
    mut commands: Commands,
) {
    if event.button != PointerButton::Secondary {
        return;
    }
    let Ok(outliner::Control::Row(index)) = rows.get(event.entity) else {
        return;
    };
    if let Some((button, _)) = buttons.iter().find(|(_, more)| more.0 == *index) {
        commands.trigger(MenuEvent {
            source: button,
            action: MenuAction::Open(NavAction::First),
        });
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sync(
    session: Res<Session>,
    content: Res<outliner::Content>,
    mut commands: Commands,
    mut roots: Query<(Entity, &mut State, &mut Node, &Children)>,
    popups: Query<(), With<MenuPopup>>,
    parents: Query<&ChildOf>,
    mut focus: ResMut<InputFocus>,
    windows: Query<&Window>,
) {
    for (owner, mut state, mut node, children) in &mut roots {
        let visible = content
            .row(state.row)
            .is_some_and(|entry| editable(entry.target));
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if state.snapshot.is_some()
            && (!state.valid(&session, &content)
                || workspace::busy(&session)
                || !windows.single().is_ok_and(|window| window.focused))
        {
            state.snapshot = None;
            for child in children.iter().filter(|child| popups.contains(*child)) {
                commands.trigger(MenuEvent {
                    source: child,
                    action: MenuAction::CloseAll,
                });
            }
            if focus.get().is_some_and(|entity| {
                parents
                    .iter_ancestors(entity)
                    .any(|ancestor| ancestor == owner)
            }) {
                focus.clear();
            }
        }
    }
}

#[cfg(test)]
#[path = "menus_context_tests.rs"]
pub(in crate::menus) mod tests;
