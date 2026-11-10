//! Sprinkles scroll.rs presentation over Bevy 0.20's standard scroll widgets.
//! Upstream 0e978f5, MIT; see third_party/sprinkles/PROVENANCE.md.
use super::*;
use bevy::input::mouse::MouseScrollUnit;
use bevy::picking::events::PointerScroll;
use bevy::ui_widgets::{ControlOrientation, Scrollbar, ScrollbarThumb};

#[derive(Component)]
pub(super) struct Pane;
#[derive(Component)]
struct Track(Entity);

pub(super) fn pane(
    parent: &mut ChildSpawnerCommands,
    padding: f32,
    body: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let area = parent
        .spawn((
            Pane,
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
                row_gap: px(8),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(body);
        })
        .id();
    parent
        .spawn((
            Track(area),
            GlobalZIndex(17),
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
    mut panes: Query<(&ComputedNode, &mut ScrollPosition), With<Pane>>,
) {
    let Ok((node, mut position)) = panes.get_mut(event.entity) else {
        return;
    };
    event.propagate(false);
    if workspace::busy(&session)
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
    app.add_systems(PostUpdate, feedback.after(bevy::ui::UiSystems::Layout))
        .add_systems(
            Update,
            ownership.after(panels::resize).before(panels::apply),
        );
}

// Preserve one UI gesture across the narrow scrollbar/panel boundary and release.
fn ownership(
    mut session: ResMut<Session>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    tracks: Query<&Hovered, With<Scrollbar>>,
    mut thumbs: Query<&mut bevy::ui_widgets::ScrollbarDragState>,
    mut active: Local<bool>,
) {
    let focused = windows.single().is_ok_and(|window| window.focused);
    if !*active
        && focused
        && !workspace::busy(&session)
        && mouse.just_pressed(MouseButton::Left)
        && tracks.iter().any(|hover| hover.0)
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
