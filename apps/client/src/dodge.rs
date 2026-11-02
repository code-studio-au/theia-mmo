//! Normal keyboard input reuses the bounded shared edge and authority profile.
pub(crate) mod animation;
use bevy::prelude::*;
use std::collections::BTreeSet;
pub const KEY: KeyCode = KeyCode::KeyV;
fn axes(direction: Vec2, backward: Vec2) -> [i16; 2] {
    let direction = if direction.length_squared() > 0.0001 {
        direction
    } else {
        backward
    }
    .normalize_or_zero();
    [
        (direction.x.clamp(-1., 1.) * 1000.).round() as i16,
        (direction.y.clamp(-1., 1.) * 1000.).round() as i16,
    ]
}
#[allow(clippy::too_many_arguments)]
pub fn input(
    mut network: ResMut<crate::network::Network>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    movement: Res<crate::movement::MoveIntent>,
    orbit: Res<crate::camera::Orbit>,
    graybox: Option<Res<crate::graybox::Graybox>>,
    inventory: Option<Res<crate::inventory::Ui>>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
) {
    if network.dodge_input.is_none() {
        return;
    }
    let now = std::time::Instant::now();
    let state = network.state();
    let correction = state.correction.map(|c| c.state);
    let panel = inventory.is_some_and(|ui| ui.open)
        || keys.just_pressed(KeyCode::KeyI)
        || dialogue.is_some_and(|ui| ui.open || ui.pending || ui.close_when_open)
        || state.dialogue_pending
        || state.dialogue.as_ref().is_some_and(|ui| ui.open.is_some());
    let ready = correction.is_some_and(|c| {
        let ids: BTreeSet<_> = state
            .correction_ready
            .as_ref()
            .filter(|r| r.sequence == c.ready_sequence)
            .into_iter()
            .flat_map(|r| r.chunks.iter())
            .filter(|id| {
                state
                    .chunk_report
                    .as_ref()
                    .is_some_and(|r| r.chunks.contains(id))
            })
            .map(|&[x, z]| theia_core::coordinates::ChunkId { x, z })
            .collect();
        graybox
            .as_ref()
            .and_then(|g| g.patch.as_ref())
            .is_some_and(|p| p.covers(c.pose.feet, Some(&ids)))
    });
    let allowed = network.connected()
        && network.can_act()
        && !panel
        && ready
        && windows.single().is_ok_and(|w| w.focused)
        && state
            .correction_received
            .is_some_and(|at| now.saturating_duration_since(at).as_secs_f64() <= 0.5);
    let pressed = keys.just_pressed(KEY).then(|| {
        axes(
            movement.direction,
            (Quat::from_rotation_y(orbit.angles().x) * Vec3::Z).xz(),
        )
    });
    network.dodge_request = match network
        .dodge_input
        .as_mut()
        .unwrap()
        .update(correction, allowed, pressed, now)
    {
        Ok(value) => value,
        Err(error) => {
            warn!("Dodge input paused: {error}");
            None
        }
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn moving_dodge_uses_travel_and_stationary_dodge_uses_camera_relative_backwards() {
        assert_eq!(axes(Vec2::X, Vec2::NEG_Y), [1000, 0]);
        assert_eq!(axes(Vec2::ZERO, Vec2::NEG_Y), [0, -1000]);
        assert_eq!(axes(Vec2::splat(1.), Vec2::Y), [707, 707]);
        assert_eq!(axes(Vec2::ZERO, Vec2::X), [1000, 0]);
    }
}
