//! Theia-specific native slider styled with the adopted Sprinkles tokens.
use super::*;

#[derive(Component)]
pub(super) struct Track;

pub(crate) fn node() -> Node {
    Node {
        width: percent(100),
        height: px(24),
        ..default()
    }
}

pub(crate) fn track() -> impl Bundle {
    (
        Track,
        Pickable::IGNORE,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(10),
            height: px(4),
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        BackgroundColor(BORDER),
    )
}

pub(super) fn sync(
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    sliders: Query<(&Hovered, Has<bevy::ui::InteractionDisabled>), With<bevy::ui_widgets::Slider>>,
    mut tracks: Query<(&ChildOf, &mut BackgroundColor), With<Track>>,
) {
    for (parent, mut background) in &mut tracks {
        if let Ok((hovered, disabled)) = sliders.get(parent.parent()) {
            background.0 = if disabled {
                BORDER.with_alpha(0.4)
            } else if focus
                .as_ref()
                .is_some_and(|focus| focus.get() == Some(parent.parent()))
            {
                PRIMARY.with_alpha(0.6)
            } else if hovered.0 {
                BORDER.lighter(0.05)
            } else {
                BORDER
            };
        }
    }
}

pub(crate) fn thumb(left: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: px(14),
        height: px(14),
        top: px(5),
        left: percent(left),
        border_radius: BorderRadius::all(px(4)),
        ..default()
    }
}
