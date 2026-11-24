//! Sprinkles dialog.rs static frame and sections; retained native modal controllers.
use super::*;
pub(crate) const BACKDROP: Color = Color::srgba(0., 0., 0., 0.8);
pub(crate) fn panel(max_width: f32) -> impl Bundle {
    (
        Node {
            width: percent(100),
            max_width: px(max_width),
            max_height: percent(90),
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(BORDER),
    )
}
pub(crate) fn header() -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(24)),
            border: UiRect::bottom(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            flex_shrink: 0.,
            ..default()
        },
        BorderColor::all(BORDER),
    )
}
pub(crate) fn footer() -> impl Bundle {
    (
        Node {
            padding: UiRect::all(px(24)),
            border: UiRect::top(px(1)),
            column_gap: px(6),
            justify_content: JustifyContent::End,
            flex_shrink: 0.,
            ..default()
        },
        BorderColor::all(BORDER),
    )
}
