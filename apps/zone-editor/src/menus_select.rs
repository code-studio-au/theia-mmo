//! Sprinkles combobox presentation over native Bevy menu selection.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::ui::{InteractionDisabled, OverrideClip};
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Document,
    Camera,
    Gizmo,
}
impl Kind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Camera => "Camera",
            Self::Gizmo => "Gizmo mode",
        }
    }
    pub(crate) fn value<'a>(self, session: &'a Session, view: &View) -> Cow<'a, str> {
        let value = match self {
            Self::Document => {
                return if session.is_dirty() {
                    Cow::Owned(format!("* {}", session.editor.zone().name))
                } else {
                    Cow::Borrowed(session.editor.zone().name.as_str())
                };
            }
            Self::Camera => match view.preset {
                camera_views::Preset::Perspective => "Perspective",
                camera_views::Preset::Top => "Top",
                camera_views::Preset::Front => "Front",
                camera_views::Preset::Right => "Right",
            },
            Self::Gizmo => match session.gizmo_tool {
                gizmo::Tool::Move => "Move",
                gizmo::Tool::Rotate => "Rotate Y",
                gizmo::Tool::Scale => "Scale uniformly",
            },
        };
        Cow::Borrowed(value)
    }
    pub(crate) fn enabled(self, session: &Session, view: &View) -> bool {
        match self {
            Self::Document => workspace::Control::New,
            Self::Camera => workspace::Control::View(view.preset),
            Self::Gizmo => workspace::Control::GizmoTool(session.gizmo_tool),
        }
        .enabled(session)
    }
}
#[derive(Component)]
pub(crate) struct SelectTrigger {
    pub kind: Kind,
    popup: Entity,
    owner: Entity,
}
#[derive(Component)]
struct Label(Kind);
#[derive(Component)]
struct Arrow(Kind);

pub(super) fn plugin(app: &mut App) {
    app.add_zone_systems(sync.after(workspace::sync));
}
pub(super) fn spawn(parent: &mut ChildSpawnerCommands, kind: Kind) {
    let mut root = parent.spawn(Node {
        width: if kind == Kind::Document {
            Val::Auto
        } else {
            percent(100)
        },
        max_width: if kind == Kind::Document {
            px(320)
        } else {
            Val::Auto
        },
        min_width: px(0),
        flex_shrink: 0.,
        flex_direction: FlexDirection::Column,
        row_gap: px(3),
        ..default()
    });
    let owner = root.id();
    let mut button = Entity::PLACEHOLDER;
    let mut popup = Entity::PLACEHOLDER;
    root.with_children(|root| {
        if kind != Kind::Document {
            let mut label_font = theme::font(10.);
            label_font.weight = FontWeight::MEDIUM;
            root.spawn((
                Text::new(kind.title()),
                label_font,
                TextColor(theme::MUTED),
                Pickable::IGNORE,
            ));
        }
        button = root
            .spawn((
                Button,
                MenuButton,
                Trigger(kind.title()),
                if kind == Kind::Document {
                    theme::Variant::Ghost
                } else {
                    theme::Variant::Default
                },
                Hovered::default(),
                Node {
                    width: if kind == Kind::Document {
                        Val::Auto
                    } else {
                        percent(100)
                    },
                    max_width: if kind == Kind::Document {
                        px(320)
                    } else {
                        Val::Auto
                    },
                    column_gap: px(6),
                    justify_content: JustifyContent::SpaceBetween,
                    ..theme::button_node()
                },
                BackgroundColor(theme::Variant::Default.background(false)),
                BorderColor::all(theme::BORDER),
            ))
            .with_children(|button| {
                button
                    .spawn((
                        Pickable::IGNORE,
                        Node {
                            min_width: px(0),
                            flex_grow: 1.,
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
                    ))
                    .with_child((
                        Label(kind),
                        theme::label("", 12., theme::BODY),
                        TextLayout::no_wrap(),
                        Pickable::IGNORE,
                    ));
                button
                    .spawn((
                        Pickable::IGNORE,
                        Node {
                            width: px(16),
                            height: px(16),
                            flex_shrink: 0.,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ))
                    .with_child((
                        Arrow(kind),
                        Pickable::IGNORE,
                        Node {
                            width: px(6),
                            height: px(6),
                            margin: UiRect::bottom(px(3)),
                            border: UiRect {
                                right: px(1),
                                bottom: px(1),
                                ..default()
                            },
                            ..default()
                        },
                        BorderColor::all(theme::BODY),
                        UiTransform::from_rotation(Rot2::degrees(45.)),
                    ));
            })
            .id();
        popup = root
            .spawn((
                Popup(kind.title()),
                MenuPopup::default(),
                OverrideClip,
                Hovered::default(),
                GlobalZIndex(18),
                Popover {
                    window_margin: 8.,
                    positions: [PopoverSide::Bottom, PopoverSide::Top]
                        .map(|side| PopoverPlacement {
                            side,
                            align: PopoverAlign::Start,
                            gap: 4.,
                        })
                        .into(),
                },
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    width: if kind == Kind::Document {
                        px(260)
                    } else {
                        percent(100)
                    },
                    padding: UiRect::all(px(4)),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER),
            ))
            .with_children(|menu| match kind {
                Kind::Document => spawn_items(
                    menu,
                    &[
                        (workspace::Control::New, "New zone", "Ctrl N"),
                        (workspace::Control::FileOpen, "Open…", "Ctrl O"),
                        (workspace::Control::RenameZone, "Rename zone…", "Shift F8"),
                        (workspace::Control::FileSaveAs, "Save as…", "Ctrl Shift S"),
                        (workspace::Control::ExportWorld, "Export world…", ""),
                        (
                            workspace::Control::ExportParticleChunks,
                            "Export particles to chunks…",
                            "",
                        ),
                        (
                            workspace::Control::Command(Action::Load),
                            "Reload",
                            "Ctrl Shift O",
                        ),
                    ],
                ),
                Kind::Camera => spawn_items(
                    menu,
                    &[
                        (
                            workspace::Control::View(camera_views::Preset::Perspective),
                            "Perspective",
                            "",
                        ),
                        (
                            workspace::Control::View(camera_views::Preset::Top),
                            "Top",
                            "",
                        ),
                        (
                            workspace::Control::View(camera_views::Preset::Front),
                            "Front",
                            "",
                        ),
                        (
                            workspace::Control::View(camera_views::Preset::Right),
                            "Right",
                            "",
                        ),
                    ],
                ),
                Kind::Gizmo => spawn_items(
                    menu,
                    &[
                        (workspace::Control::GizmoTool(gizmo::Tool::Move), "Move", ""),
                        (
                            workspace::Control::GizmoTool(gizmo::Tool::Rotate),
                            "Rotate Y",
                            "",
                        ),
                        (
                            workspace::Control::GizmoTool(gizmo::Tool::Scale),
                            "Scale uniformly",
                            "",
                        ),
                    ],
                ),
            })
            .id();
    });
    root.commands()
        .entity(button)
        .insert(SelectTrigger { kind, popup, owner });
    root.observe(
        move |event: On<MenuEvent>,
              session: Res<Session>,
              view: Res<View>,
              windows: Query<&Window>,
              mut popups: Query<(&mut Node, &mut MenuFocusState), With<MenuPopup>>,
              mut focus: ResMut<InputFocus>| {
            let Ok((mut node, mut state)) = popups.get_mut(popup) else {
                return;
            };
            let allowed = kind.enabled(&session, &view)
                && windows.single().is_ok_and(|window| window.focused);
            match event.action {
                MenuAction::Open(nav) if allowed => {
                    node.display = Display::Flex;
                    *state = MenuFocusState::Opening(nav);
                }
                MenuAction::Toggle if allowed => {
                    if node.display == Display::None {
                        node.display = Display::Flex;
                        *state = MenuFocusState::Opening(NavAction::First);
                    } else {
                        node.display = Display::None;
                        *state = MenuFocusState::Closed;
                    }
                }
                MenuAction::CloseAll => {
                    node.display = Display::None;
                    *state = MenuFocusState::Closed;
                }
                MenuAction::FocusRoot => focus.set(button, FocusCause::Navigated),
                _ => {}
            }
        },
    );
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sync(
    session: Res<Session>,
    view: Res<View>,
    windows: Query<&Window>,
    mut commands: Commands,
    mut buttons: Query<(
        Entity,
        &SelectTrigger,
        &mut theme::Variant,
        Has<InteractionDisabled>,
    )>,
    popups: Query<&MenuFocusState>,
    mut labels: Query<(&Label, &mut Text)>,
    mut arrows: Query<(&Arrow, &mut UiTransform, &mut BorderColor)>,
    parents: Query<&ChildOf>,
    mut focus: ResMut<InputFocus>,
) {
    for (entity, selector, mut variant, disabled) in &mut buttons {
        let enabled = selector.kind.enabled(&session, &view);
        if disabled == enabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
        let open = popups
            .get(selector.popup)
            .is_ok_and(|state| !matches!(state, MenuFocusState::Closed));
        *variant = if !enabled {
            theme::Variant::Disabled
        } else if open {
            theme::Variant::Active
        } else if selector.kind == Kind::Document {
            theme::Variant::Ghost
        } else {
            theme::Variant::Default
        };
        for (label, mut text) in &mut labels {
            if label.0 == selector.kind {
                let value = selector.kind.value(&session, &view);
                if **text != value.as_ref() {
                    **text = value.into_owned();
                }
            }
        }
        for (arrow, mut pose, mut border) in &mut arrows {
            if arrow.0 == selector.kind {
                pose.rotation = Rot2::degrees(if open { 225. } else { 45. });
                *border = BorderColor::all(if !enabled {
                    theme::MUTED
                } else if open {
                    theme::PRIMARY.lighter(0.05)
                } else {
                    theme::BODY
                });
            }
        }
        if open && (!enabled || !windows.single().is_ok_and(|window| window.focused)) {
            commands.trigger(MenuEvent {
                source: selector.popup,
                action: MenuAction::CloseAll,
            });
            if focus.get().is_some_and(|entity| {
                parents
                    .iter_ancestors(entity)
                    .any(|ancestor| ancestor == selector.owner)
            }) {
                focus.clear();
            }
        }
    }
}

#[cfg(test)]
#[path = "menus_select_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "menus_document_tests.rs"]
mod document_tests;
