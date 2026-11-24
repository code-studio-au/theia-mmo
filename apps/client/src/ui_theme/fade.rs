//! Compose one opacity over the art, typography and controls in a HUD pane.
use bevy::{input_focus::InputFocus, picking::hover::Hovered, prelude::*};
use std::collections::HashMap;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pane {
    Chat,
    Journal,
}
const IDLE_ALPHA: f32 = 0.6;

#[derive(Clone, Copy, Default)]
struct Tint(Option<(Color, Color)>);
impl Tint {
    fn resolve(&mut self, current: Color, opacity: f32) -> Color {
        // Store both values so a fresh style write becomes the source and an
        // unchanged faded color never gets multiplied again on the next frame.
        let original = self
            .0
            .filter(|(_, previous)| *previous == current)
            .map_or(current, |(original, _)| original);
        let result = original.with_alpha(original.alpha() * opacity);
        self.0 = Some((original, result));
        result
    }
}
type Colors<'a> = (
    Option<&'a mut BackgroundColor>,
    Option<&'a mut BorderColor>,
    Option<&'a mut Outline>,
    Option<&'a mut ImageNode>,
    Option<&'a mut TextColor>,
    Option<&'a mut TextShadow>,
);
pub(super) fn install(app: &mut App) {
    app.add_systems(Last, present);
}
fn present(
    roots: Query<(Entity, &Pane, Option<&Hovered>)>,
    children: Query<&Children>,
    focus: Option<Res<InputFocus>>,
    chat: Option<Res<crate::chat::Ui>>,
    windows: Query<&Window>,
    mut colors: Query<Colors<'_>>,
    mut cache: Local<HashMap<Entity, [Tint; 9]>>,
) {
    let focused = focus.as_deref().and_then(InputFocus::get);
    let window_active = windows.single().is_ok_and(|window| window.focused);
    for (root, pane, hover) in &roots {
        let contains_focus = focused.is_some_and(|entity| {
            root == entity || children.iter_descendants(root).any(|child| child == entity)
        });
        let active = window_active
            && (contains_focus
                || match pane {
                    Pane::Chat => chat.as_ref().is_some_and(|state| state.active()),
                    Pane::Journal => hover.is_some_and(Hovered::get),
                });
        let opacity = if active { 1. } else { IDLE_ALPHA };
        for entity in std::iter::once(root).chain(children.iter_descendants(root)) {
            let Ok((background, border, outline, image, text, shadow)) = colors.get_mut(entity)
            else {
                continue;
            };
            let saved = cache.entry(entity).or_default();
            if let Some(mut value) = background {
                let color = saved[0].resolve(value.0, opacity);
                value.set_if_neq(BackgroundColor(color));
            }
            if let Some(mut value) = border {
                let next = BorderColor {
                    top: saved[1].resolve(value.top, opacity),
                    right: saved[2].resolve(value.right, opacity),
                    bottom: saved[3].resolve(value.bottom, opacity),
                    left: saved[4].resolve(value.left, opacity),
                };
                value.set_if_neq(next);
            }
            if let Some(mut value) = outline {
                let color = saved[5].resolve(value.color, opacity);
                if value.color != color {
                    value.color = color;
                }
            }
            if let Some(mut value) = image {
                let color = saved[6].resolve(value.color, opacity);
                if value.color != color {
                    value.color = color;
                }
            }
            if let Some(mut value) = text {
                let color = saved[7].resolve(value.0, opacity);
                value.set_if_neq(TextColor(color));
            }
            if let Some(mut value) = shadow {
                let color = saved[8].resolve(value.color, opacity);
                if value.color != color {
                    value.color = color;
                }
            }
        }
    }
    cache.retain(|entity, _| colors.contains(*entity));
}
pub(crate) fn background_opacity(world: &mut World, pane: Pane) -> Option<f32> {
    let root = world
        .query::<(Entity, &Pane)>()
        .iter(world)
        .find(|(_, mode)| **mode == pane)?
        .0;
    world
        .get::<Children>(root)?
        .iter()
        .find_map(|child| world.get::<ImageNode>(child).map(|art| art.color.alpha()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_restores_exact_source_color_and_accepts_style_changes() {
        let mut tint = Tint::default();
        let original = Color::srgb(0.2, 0.45, 0.6).with_alpha(0.8);
        let faded = tint.resolve(original, IDLE_ALPHA);
        assert_eq!(faded.alpha(), 0.8 * IDLE_ALPHA);
        assert_eq!(tint.resolve(faded, IDLE_ALPHA), faded);
        assert_eq!(tint.resolve(faded, 1.), original);
        let pressed = Color::srgb(0.45, 0.3, 0.1);
        assert_eq!(
            tint.resolve(pressed, IDLE_ALPHA),
            pressed.with_alpha(IDLE_ALPHA)
        );
    }
}
