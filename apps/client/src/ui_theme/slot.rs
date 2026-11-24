//! Shared matte wells for ability and item tiles.
use bevy::prelude::*;

pub(super) const WELL: Color = Color::srgb(0.046, 0.052, 0.062);
const EDGE: Color = Color::srgb(0.48, 0.40, 0.29);

pub(crate) fn frame(mut node: Node) -> impl Bundle {
    node.border = UiRect::all(px(1));
    node.border_radius = BorderRadius::all(px(4));
    (node, BackgroundColor(WELL), BorderColor::all(EDGE))
}

/// Use the inner width of the frame and retain the image's cropped aspect ratio.
pub(crate) fn art() -> Node {
    Node {
        width: percent(100),
        max_height: percent(100),
        border_radius: BorderRadius::all(px(3)),
        flex_shrink: 0.,
        ..default()
    }
}

pub(super) fn colors(state: &str) -> (Color, BorderColor) {
    let (well, edge) = match state {
        "disabled" => (WELL, Color::srgb(0.25, 0.25, 0.25)),
        "pressed" => (
            Color::srgb(0.035, 0.038, 0.045),
            Color::srgb(0.66, 0.55, 0.38),
        ),
        "selected" | "focused" => (WELL, Color::srgb(0.81, 0.70, 0.47)),
        "hover" => (
            Color::srgb(0.080, 0.086, 0.098),
            Color::srgb(0.67, 0.57, 0.42),
        ),
        _ => (WELL, EDGE),
    };
    (well, BorderColor::all(edge))
}
