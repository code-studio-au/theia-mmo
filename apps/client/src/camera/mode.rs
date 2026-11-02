//! Client presentation policy; mode changes never submit gameplay commands.
use bevy::picking::{
    PickingSystems,
    pointer::{PointerAction, PointerId, PointerInput, PointerLocation},
};
use bevy::prelude::*;

pub const TOGGLE: KeyCode = KeyCode::F8;
pub const CURSOR: KeyCode = KeyCode::ControlLeft;

#[derive(Resource, Default)]
pub(crate) struct Mode {
    pub requested: bool,
    pub active: bool,
    pub consumed_escape: bool,
    pub transitioned: bool,
    pub pivot_offset: f32,
    pub focus: Option<Vec3>,
}
impl Mode {
    pub fn hint(&self, panel: bool) -> &'static str {
        match (self.requested, self.active, panel) {
            (true, _, true) => "Action camera paused | Esc: close panel | I: backpack",
            (false, _, true) => "Cursor mode | Esc: close panel | I: backpack",
            (_, true, false) => "F8: classic | L Ctrl: cursor | Esc: cursor",
            (true, false, false) => "F8: classic | Release L Ctrl: mouse-look | Esc: cursor",
            _ => "F8: action camera | F: interact | Esc: quit",
        }
    }
    fn advance_pivot(&mut self, seconds: f32) {
        let goal = if self.requested { 1.2 } else { 0. };
        let seconds = if seconds.is_finite() {
            seconds.clamp(0., 0.05)
        } else {
            0.
        };
        self.pivot_offset += (goal - self.pivot_offset) * (1. - (-seconds / 0.08).exp());
        if (goal - self.pivot_offset).abs() < 0.0001 {
            self.pivot_offset = goal;
        }
    }
    pub fn update(&mut self, keys: &ButtonInput<KeyCode>, focused: bool, modal: bool) {
        let previous = self.active;
        self.consumed_escape = false;
        if focused && !modal && keys.just_pressed(TOGGLE) {
            self.requested = !self.requested;
        }
        if focused && !modal && self.requested && keys.just_pressed(KeyCode::Escape) {
            self.requested = false;
            self.consumed_escape = true;
        }
        self.active = self.requested && focused && !modal && !keys.pressed(CURSOR);
        self.transitioned = previous != self.active;
    }
}

#[allow(clippy::too_many_arguments)]
fn input_policy(
    mut mode: ResMut<Mode>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    dialogue: Option<Res<crate::dialogue::Ui>>,
    inventory: Option<Res<crate::inventory::Ui>>,
    mut inputs: ResMut<Messages<PointerInput>>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    time: Option<Res<Time>>,
    classic: Option<Res<crate::classic_controls::Controller>>,
) {
    let focused = windows.single().is_ok_and(|window| window.focused);
    let modal = dialogue
        .as_ref()
        .is_some_and(|ui| ui.open || ui.pending || ui.close_when_open)
        || inventory.as_ref().is_some_and(|ui| ui.open)
        || keys.just_pressed(KeyCode::KeyI);
    if classic.is_some() {
        mode.requested = false;
        mode.active = false;
        mode.consumed_escape = false;
        mode.transitioned = false;
    } else {
        mode.update(&keys, focused, modal);
    }
    mode.advance_pivot(time.map_or(1. / 60., |time| time.delta_secs()));
    if mode.active || mode.transitioned {
        // Locked mouse input belongs to reticle controls. Preserve other
        // pointers, discard stale UI/world clicks and cancel classic drags.
        let other: Vec<_> = inputs
            .drain()
            .filter(|event| event.pointer_id != PointerId::Mouse)
            .collect();
        inputs.write_batch(other);
        if mode.transitioned
            && let Some(location) = pointers.iter().find_map(|(id, p)| {
                (*id == PointerId::Mouse)
                    .then(|| p.location.clone())
                    .flatten()
            })
        {
            inputs.write(PointerInput::new(
                PointerId::Mouse,
                location,
                PointerAction::Cancel,
            ));
        }
    }
}

#[derive(Component)]
struct Reticle;
fn setup(mut commands: Commands) {
    commands.spawn((
        Reticle,
        Text::new("+"),
        TextFont {
            font_size: bevy::text::FontSize::Px(22.),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50.),
            top: percent(50.),
            // Centre the glyph, not its top-left corner.
            margin: UiRect {
                left: px(-7.),
                top: px(-14.),
                ..default()
            },
            ..default()
        },
        bevy::picking::Pickable::IGNORE,
        Visibility::Hidden,
        ZIndex(10),
    ));
}
fn present(mode: Res<Mode>, mut reticle: Query<&mut Visibility, With<Reticle>>) {
    for mut visible in &mut reticle {
        *visible = if mode.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
pub(crate) fn install(app: &mut App) {
    app.init_resource::<Mode>()
        .add_systems(
            PreUpdate,
            input_policy
                .after(bevy::input::InputSystems)
                .before(PickingSystems::ProcessInput),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, present);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pivot_transition_is_frame_rate_independent_and_panels_do_not_reset_it() {
        let mut offsets = Vec::new();
        for hz in [60, 120, 240] {
            let mut mode = Mode {
                requested: true,
                ..default()
            };
            for _ in 0..hz / 4 {
                mode.advance_pivot(1. / hz as f32);
            }
            offsets.push(mode.pivot_offset);
            let before = mode.pivot_offset;
            mode.update(&ButtonInput::default(), true, true);
            mode.advance_pivot(1. / hz as f32);
            assert!(mode.pivot_offset > before && mode.pivot_offset < 1.2);
            mode.requested = false;
            for _ in 0..hz {
                mode.advance_pivot(1. / hz as f32);
            }
            assert_eq!(mode.pivot_offset, 0.);
        }
        assert!(
            offsets
                .iter()
                .all(|offset| (*offset - offsets[0]).abs() < 1e-5)
        );
    }
    #[test]
    fn locked_mouse_cannot_click_ui_and_transition_cancels_held_pointer() {
        use bevy::picking::pointer::Location;
        let mut app = App::new();
        app.init_resource::<Mode>()
            .init_resource::<bevy::picking::pointer::PointerMap>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Messages<PointerInput>>()
            .add_systems(Update, input_policy);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let location = Location {
            target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                .normalize(Some(window))
                .unwrap(),
            position: Vec2::new(10., 20.),
        };
        app.world_mut()
            .spawn((PointerId::Mouse, PointerLocation::new(location.clone())));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(TOGGLE);
        app.world_mut().write_message(PointerInput::new(
            PointerId::Mouse,
            location.clone(),
            PointerAction::Press(bevy::picking::pointer::PointerButton::Primary),
        ));
        app.update();
        let messages: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<PointerInput>>()
            .drain()
            .collect();
        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0].action, PointerAction::Cancel));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().write_message(PointerInput::new(
            PointerId::Mouse,
            location,
            PointerAction::Release(bevy::picking::pointer::PointerButton::Primary),
        ));
        app.update();
        assert!(app.world().resource::<Messages<PointerInput>>().is_empty());
        assert!(app.world().resource::<Mode>().active);
    }
    #[test]
    fn toggle_modal_focus_and_temporary_cursor_preserve_requested_mode() {
        let mut mode = Mode::default();
        let mut keys = ButtonInput::default();
        keys.press(TOGGLE);
        mode.update(&keys, true, false);
        assert!(mode.requested && mode.active);
        keys.clear();
        mode.update(&keys, true, true);
        assert!(mode.requested && !mode.active);
        mode.update(&keys, false, false);
        assert!(mode.requested && !mode.active);
        keys.press(CURSOR);
        mode.update(&keys, true, false);
        assert!(!mode.active);
        keys.release(CURSOR);
        mode.update(&keys, true, false);
        assert!(mode.active);
        keys.press(KeyCode::Escape);
        mode.update(&keys, true, false);
        assert!(!mode.requested && !mode.active && mode.consumed_escape);
        keys.clear();
        mode.update(&keys, true, false);
        assert!(!mode.consumed_escape);
    }
    #[test]
    fn panels_and_unfocused_windows_cannot_toggle_or_consume_escape() {
        let mut mode = Mode {
            requested: true,
            ..default()
        };
        let mut keys = ButtonInput::default();
        keys.press(TOGGLE);
        keys.press(KeyCode::Escape);
        for (focused, modal) in [(false, false), (true, true)] {
            mode.update(&keys, focused, modal);
            assert!(mode.requested && !mode.active && !mode.consumed_escape);
        }
    }
}
