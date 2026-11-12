//! A single-frame Sprinkles-style action popover for the selected content.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::ui::OverrideClip;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};
use bevy_sprinkles_editor::ui::icons::ICON_MORE;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    target: outliner::Target,
    document_id: u64,
    point: Option<usize>,
}
#[derive(Component, Default)]
pub(super) struct State {
    snapshot: Option<(Identity, u64)>,
    pending: Option<(Identity, NavAction)>,
}
#[derive(Component)]
pub(crate) struct GuideControls;

fn identity(session: &Session, tabs: Option<&property_tabs::State>) -> Option<Identity> {
    if workspace::busy(session)
        || session.ui.properties_collapsed
        || session.palette.is_some()
        || session.plant_palette.is_some()
        || session.marker_palette.is_some()
        || tabs.is_some_and(|tabs| tabs.active != property_tabs::Tab::Attributes)
    {
        return None;
    }
    let target = outliner::selection(session)?;
    if !matches!(
        target,
        outliner::Target::Object(_)
            | outliner::Target::Guide(_)
            | outliner::Target::Marker(_)
            | outliner::Target::Plant(_)
            | outliner::Target::Particle(_)
            | outliner::Target::QuestArea(_)
            | outliner::Target::Scenery(_)
            | outliner::Target::TestPlayer(_)
    ) {
        return None;
    }
    Some(Identity {
        target,
        document_id: session.document_id,
        point: session.path_point,
    })
}
impl State {
    pub(super) fn valid(&self, session: &Session, tabs: Option<&property_tabs::State>) -> bool {
        self.snapshot.is_some_and(|(expected, revision)| {
            identity(session, tabs) == Some(expected) && session.editor.revision() == revision
        })
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_zone_systems(sync.after(workspace::sync).after(property_tabs::sync));
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, server: Option<&AssetServer>) {
    let mut root = parent.spawn((
        State::default(),
        Visibility::Hidden,
        Node {
            display: Display::None,
            width: px(28),
            height: px(28),
            flex_shrink: 0.,
            position_type: PositionType::Relative,
            ..default()
        },
    ));
    let owner = root.id();
    let mut button = Entity::PLACEHOLDER;
    let mut popup = Entity::PLACEHOLDER;
    root.with_children(|root| {
        button = root
            .spawn((
                Trigger("Property actions"),
                Button,
                MenuButton,
                Hovered::default(),
                theme::Variant::Ghost,
                TabIndex(0),
                Node {
                    width: px(28),
                    height: px(28),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_child((
                ImageNode::new(server.map(|s| s.load(ICON_MORE)).unwrap_or_default()),
                Pickable::IGNORE,
                Node {
                    width: px(18),
                    height: px(18),
                    ..default()
                },
            ))
            .id();
        popup = root
            .spawn((
                Popup("Property actions"),
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
                    padding: UiRect::all(px(2)),
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
                        (workspace::Control::Rename, "Rename", "F8"),
                        (workspace::Control::Transform, "Focus attributes", "F9"),
                        (
                            workspace::Control::ParticlePreview,
                            "Toggle open-project preview",
                            "",
                        ),
                        (
                            workspace::Control::BindParticle,
                            "Save project to local Effect ID",
                            "",
                        ),
                        (
                            workspace::Control::PreviewLibraryParticle,
                            "Toggle local Effect ID preview",
                            "",
                        ),
                        (
                            workspace::Control::Command(Action::Duplicate),
                            "Duplicate",
                            "Ctrl D",
                        ),
                        (workspace::Control::Command(Action::Ground), "Ground", ""),
                        (workspace::Control::Frame, "Frame", "F"),
                        (workspace::Control::Command(Action::Delete), "Delete", "Del"),
                    ],
                );
                test_players::menu(menu);
            })
            .id();
    });
    root.observe(
        move |event: On<MenuEvent>,
              session: Res<Session>,
              windows: Query<&Window>,
              tabs: Option<Res<property_tabs::State>>,
              mut states: Query<&mut State>,
              mut popups: Query<(&mut Node, &mut MenuFocusState), With<MenuPopup>>,
              mut focus: ResMut<InputFocus>| {
            let Ok(mut state) = states.get_mut(owner) else {
                return;
            };
            let Ok((mut node, mut menu_focus)) = popups.get_mut(popup) else {
                return;
            };
            let opening = match event.action {
                MenuAction::Open(nav) => Some(nav),
                MenuAction::Toggle if node.display == Display::None && state.pending.is_none() => {
                    Some(NavAction::First)
                }
                _ => None,
            };
            if let Some(nav) = opening {
                if !windows.single().is_ok_and(|window| window.focused) {
                    return;
                }
                let Some(current) = identity(&session, tabs.as_deref()) else {
                    return;
                };
                if session.inline_editing {
                    // Finish the current textbox on blur before snapshotting the menu.
                    state.pending = Some((current, nav));
                    focus.set(button, FocusCause::Navigated);
                } else {
                    state.snapshot = Some((current, session.editor.revision()));
                    node.display = Display::Flex;
                    *menu_focus = MenuFocusState::Opening(nav);
                }
            } else {
                match event.action {
                    MenuAction::CloseAll | MenuAction::Toggle => {
                        state.snapshot = None;
                        state.pending = None;
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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync(
    session: Res<Session>,
    tabs: Option<Res<property_tabs::State>>,
    windows: Query<&Window>,
    mut roots: Query<
        (Entity, &mut State, &mut Node, &mut Visibility, &Children),
        Without<GuideControls>,
    >,
    mut guides: Query<(&mut Node, &mut Visibility), (With<GuideControls>, Without<State>)>,
    mut popups: Query<
        (&mut Node, &mut MenuFocusState),
        (With<MenuPopup>, Without<State>, Without<GuideControls>),
    >,
    parents: Query<&ChildOf>,
    buttons: Query<(), With<MenuButton>>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let current = identity(&session, tabs.as_deref());
    let focused = windows.single().is_ok_and(|window| window.focused);
    for (mut node, mut visibility) in &mut guides {
        let visible = current.is_some_and(|i| matches!(i.target, outliner::Target::Guide(_)));
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
    for (owner, mut state, mut node, mut visibility, children) in &mut roots {
        node.display = if current.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *visibility = if current.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let popup = children.iter().find(|child| popups.contains(*child));
        let button = children.iter().find(|child| buttons.contains(*child));
        if let Some((expected, nav)) = state.pending {
            if !focused || current != Some(expected) || focus.get() != button {
                state.pending = None;
            } else if !session.inline_editing {
                state.pending = None;
                state.snapshot = Some((expected, session.editor.revision()));
                if let Some(popup) = popup
                    && let Ok((mut node, mut menu_focus)) = popups.get_mut(popup)
                {
                    node.display = Display::Flex;
                    *menu_focus = MenuFocusState::Opening(nav);
                }
            }
        }
        if state.snapshot.is_some() && (!state.valid(&session, tabs.as_deref()) || !focused) {
            state.snapshot = None;
            if let Some(popup) = popup {
                commands.trigger(MenuEvent {
                    source: popup,
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
#[path = "menus_property_actions_tests.rs"]
mod tests;
