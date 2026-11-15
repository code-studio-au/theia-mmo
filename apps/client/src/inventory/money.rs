//! Shared gold/silver/copper rows. The bag's confirmed copper ledger is the source.
use super::*;
#[derive(Component)]
pub(super) struct Value {
    denomination: usize,
    fixed: Option<u64>,
}
const COLORS: [Color; 3] = [
    Color::srgb(0.95, 0.73, 0.18),
    Color::srgb(0.78, 0.83, 0.88),
    Color::srgb(0.78, 0.42, 0.21),
];
pub(super) fn row(parent: &mut ChildSpawnerCommands, amount: Option<u64>, footer: bool) {
    parent
        .spawn((
            Node {
                width: if footer { Val::Auto } else { percent(100) },
                height: px(28),
                flex_shrink: 0.,
                column_gap: px(5),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                position_type: if footer {
                    PositionType::Absolute
                } else {
                    PositionType::Relative
                },
                bottom: if footer { px(12) } else { Val::Auto },
                left: if footer { px(8) } else { Val::Auto },
                right: if footer { px(8) } else { Val::Auto },
                padding: UiRect::horizontal(px(12)),
                ..default()
            },
            bevy::ui::IgnoreScroll(BVec2::splat(footer)),
            Pickable::IGNORE,
            BackgroundColor(if footer {
                crate::fantasy_ui::PANEL
            } else {
                Color::NONE
            }),
        ))
        .with_children(|row| {
            for (denomination, color) in COLORS.into_iter().enumerate() {
                row.spawn((
                    Value {
                        denomination,
                        fixed: amount,
                    },
                    Text::new("0"),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
                row.spawn((
                    Node {
                        width: px(11),
                        height: px(11),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(percent(50)),
                        margin: UiRect::right(px(4)),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor::all(Color::srgb(0.25, 0.18, 0.08)),
                    Pickable::IGNORE,
                ));
            }
        });
}
pub(super) fn paint(ui: Res<Ui>, mut values: Query<(&Value, &mut Text)>) {
    for (value, mut text) in &mut values {
        let amount = value
            .fixed
            .or_else(|| ui.bag.as_ref().and_then(Bag::copper));
        let next = amount.map_or_else(
            || "—".into(),
            |c| [c / 10000, (c / 100) % 100, c % 100][value.denomination].to_string(),
        );
        if text.0 != next {
            text.0 = next;
        }
    }
}
