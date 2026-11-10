//! Sprinkles scroll.rs presentation over Bevy 0.20's standard scroll widgets.
//! Upstream 0e978f5, MIT; see third_party/sprinkles/PROVENANCE.md.
use super::*;
#[path = "scrolling_focus.rs"]
mod focus;
use bevy::input::mouse::MouseScrollUnit;
use bevy::picking::events::PointerScroll;
use bevy::ui_widgets::{ControlOrientation, Scrollbar, ScrollbarThumb};

#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Pane {
    Workspace,
    Browser,
    Dialog,
    Attributes,
}
impl Pane {
    fn allowed(self, session: &Session) -> bool {
        !session.ui.input_owned
            && session.play.is_none()
            && session.drag.is_none()
            && session.stroke.is_none()
            && session.path_draft.is_none()
            && match self {
                Self::Workspace | Self::Browser => !session.has_modal(),
                Self::Dialog => session.name_edit.is_some() || session.file_dialog.is_some(),
                Self::Attributes => {
                    session.transform_edit.is_some()
                        && session.name_edit.is_none()
                        && session.file_dialog.is_none()
                }
            }
    }
}
#[derive(Component)]
struct Track(Entity);

pub(super) fn pane(
    parent: &mut ChildSpawnerCommands,
    padding: f32,
    scope: Pane,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    if matches!(scope, Pane::Dialog) {
        // Keep the scrollbar within the dialog body, clear of its fixed actions.
        parent
            .spawn(Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..default()
            })
            .with_children(|parent| spawn_pane(parent, padding, scope, body));
    } else {
        spawn_pane(parent, padding, scope, body);
    }
}
fn spawn_pane(
    parent: &mut ChildSpawnerCommands,
    padding: f32,
    scope: Pane,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let area = parent
        .spawn((
            scope,
            ScrollPosition::default(),
            Hovered::default(),
            Node {
                width: percent(100),
                flex_grow: 1.,
                min_height: px(0),
                padding: UiRect::all(px(padding)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .observe(wheel)
        .with_children(|area| {
            area.spawn(Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                row_gap: px(if matches!(scope, Pane::Dialog) {
                    12.
                } else {
                    8.
                }),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(body);
        })
        .id();
    parent
        .spawn((
            Track(area),
            GlobalZIndex(if matches!(scope, Pane::Dialog) {
                31
            } else if matches!(scope, Pane::Attributes) {
                21
            } else {
                17
            }),
            Hovered::default(),
            Scrollbar::new(area, ControlOrientation::Vertical, 24.),
            Node {
                position_type: PositionType::Absolute,
                width: px(3),
                right: px(3),
                top: px(3),
                bottom: px(3),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|track| {
            track.spawn((
                ScrollbarThumb {
                    border_radius: BorderRadius::all(px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::Srgba(bevy::color::palettes::tailwind::ZINC_600)),
            ));
        });
}

// Sprinkles' 24 logical pixels per wheel line; trackpads retain pixel deltas.
fn wheel(
    mut event: On<PointerScroll>,
    session: Res<Session>,
    windows: Query<&Window>,
    menus: Query<&bevy::ui_widgets::MenuFocusState, With<bevy::ui_widgets::MenuPopup>>,
    mut panes: Query<(&Pane, &ComputedNode, &mut ScrollPosition)>,
) {
    let Ok((scope, node, mut position)) = panes.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    if !scope.allowed(&session)
        || menus
            .iter()
            .any(|state| !matches!(state, bevy::ui_widgets::MenuFocusState::Closed))
        || !windows.single().is_ok_and(|w| w.focused)
        || !event.y.is_finite()
    {
        return;
    }
    let delta = -event.y
        * if event.unit == MouseScrollUnit::Line {
            24.
        } else {
            1.
        };
    let limit = ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
    position.y = (position.y + delta).clamp(0., limit);
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<focus::Reveal>()
        .add_observer(focus::request)
        .add_systems(
            PostUpdate,
            (focus::reveal, feedback)
                .chain()
                .after(bevy::ui::UiSystems::Layout)
                .after(bevy::ui::ui_layout_system)
                .after(properties::finish)
                .after(naming::finish)
                .after(file_picker::finish),
        )
        .add_systems(
            Update,
            ownership.after(panels::resize).before(panels::apply),
        );
}

// Preserve one UI gesture across the narrow scrollbar/panel boundary and release.
fn ownership(
    mut session: ResMut<Session>,
    windows: Query<&Window>,
    input: (Res<ButtonInput<MouseButton>>, Res<ButtonInput<KeyCode>>),
    tracks: Query<(&Hovered, &Scrollbar)>,
    panes: Query<&Pane>,
    mut thumbs: Query<&mut bevy::ui_widgets::ScrollbarDragState>,
    mut active: Local<bool>,
) {
    let (mouse, keys) = input;
    let focused = windows.single().is_ok_and(|window| window.focused);
    if !*active
        && focused
        && !session.ui.input_owned
        && mouse.just_pressed(MouseButton::Left)
        && tracks.iter().any(|(hover, bar)| {
            hover.0
                && panes
                    .get(bar.target)
                    .is_ok_and(|pane| pane.allowed(&session))
        })
    {
        *active = true;
    }
    if *active {
        session.ui.input_owned = true;
        if !focused || !mouse.pressed(MouseButton::Left) || keys.just_pressed(KeyCode::Escape) {
            *active = false;
            for mut drag in &mut thumbs {
                drag.dragging = false;
            }
        }
    }
}

fn feedback(
    mut panes: Query<(&Hovered, &ComputedNode, &mut ScrollPosition), With<Pane>>,
    mut tracks: Query<(&Track, &Hovered, &Children, &mut Visibility)>,
    thumbs: Query<&bevy::ui_widgets::ScrollbarDragState>,
) {
    for (_, node, mut position) in &mut panes {
        let limit = ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        let y = position.y.clamp(0., limit);
        if position.y != y {
            position.y = y;
        }
    }
    for (track, track_hover, children, mut visibility) in &mut tracks {
        let shown = panes.get(track.0).is_ok_and(|(hovered, node, _)| {
            (hovered.0
                || track_hover.0
                || children
                    .iter()
                    .any(|child| thumbs.get(child).is_ok_and(|drag| drag.dragging)))
                && node.content_size().y > node.size().y + 0.5
        });
        let desired = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
    }
}
