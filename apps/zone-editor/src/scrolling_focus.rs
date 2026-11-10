//! Reveal newly focused inspector controls without fighting manual scrolling.
use super::*;
use bevy::input_focus::InputFocus;

#[derive(Default)]
pub(super) struct Reveal {
    last: Option<Entity>,
    pending: Option<Entity>,
}

pub(super) fn reveal(
    focus: Option<Res<InputFocus>>,
    session: Res<Session>,
    windows: Query<&Window>,
    parents: Query<&ChildOf>,
    nodes: Query<(&ComputedNode, &bevy::ui::UiGlobalTransform)>,
    mut panes: Query<(
        &Pane,
        &ComputedNode,
        &bevy::ui::UiGlobalTransform,
        &mut ScrollPosition,
    )>,
    mut state: Local<Reveal>,
) {
    let current = focus.as_ref().and_then(|focus| focus.get());
    if current != state.last {
        state.last = current;
        state.pending = current;
    }
    let Some(target) = state.pending else {
        return;
    };
    if !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    let Some(ancestor) = parents
        .iter_ancestors(target)
        .find(|ancestor| panes.contains(*ancestor))
    else {
        state.pending = None;
        return;
    };
    let Ok((scope, pane, pane_pose, mut position)) = panes.get_mut(ancestor) else {
        return;
    };
    if !scope.allowed(&session) {
        return;
    }
    let Ok((node, pose)) = nodes.get(target) else {
        state.pending = None;
        return;
    };
    // Opening a form may focus its field before its first visible layout.
    if node.size().min_element() <= 0. || pane.size().min_element() <= 0. {
        return;
    }
    let factor = pane.inverse_scale_factor();
    let top = (pane_pose.affine().translation.y - pane.size().y / 2.) * factor + 8.;
    let bottom = (pane_pose.affine().translation.y + pane.size().y / 2.) * factor - 8.;
    let first = (pose.affine().translation.y - node.size().y / 2.) * factor;
    let last = (pose.affine().translation.y + node.size().y / 2.) * factor;
    let delta = if first < top {
        first - top
    } else if last > bottom {
        last - bottom
    } else {
        0.
    };
    let limit = ((pane.content_size().y - pane.size().y) * factor).max(0.);
    let y = (position.y + delta).clamp(0., limit);
    if position.y != y {
        position.y = y;
    }
    state.pending = None;
}
