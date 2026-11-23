//! Status observations for the admitted owner and selected target.
use crate::network::Network;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
};
use std::sync::Arc;
use theia_core::statuses::Catalog;
#[derive(Resource)]
struct Definitions(Arc<Catalog>);
#[derive(Component)]
struct Label;
#[derive(Clone, Copy)]
enum Subject {
    Owner,
    Target,
}
#[derive(Component)]
struct EffectRow(Subject);
#[derive(Component)]
struct EffectSlot;
#[derive(Component)]
struct DurationLabel;
#[derive(Component)]
struct Detail;

#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub active_frames: u64,
    pub empty_frames: u64,
    pub own_frames: u64,
    pub other_frames: u64,
    pub max_rows: usize,
}
pub(crate) fn install(app: &mut App, catalog: Arc<Catalog>) {
    app.insert_resource(Definitions(catalog))
        .init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (present, paint_icons).after(crate::network::present),
        );
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    if skin.is_some() {
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(270),
                width: px(348),
                padding: UiRect::all(px(10)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Label,
                Text::new("Statuses: waiting for server"),
                TextFont {
                    font_size: FontSize::Px(14.),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}
fn present(
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut metrics: ResMut<Metrics>,
    mut labels: Query<&mut Text, With<Label>>,
    selection: Option<Res<crate::world_input::Selection>>,
) {
    let state = network.state();
    let selected = selection
        .as_ref()
        .is_none_or(|s| s.friendly.is_none())
        .then(|| network.confirmed_hostile_target(&state))
        .flatten();
    let text = if state.error.is_some() {
        "Statuses: disconnected".to_owned()
    } else if let Some(view) = &state.statuses {
        let now = state
            .snapshot
            .as_ref()
            .map_or(view.tick, |s| s.tick.max(view.tick));
        let rows: Vec<_> = view
            .statuses
            .iter()
            .filter(|s| Some(s.target.entity) == selected)
            .collect();
        metrics.max_rows = metrics.max_rows.max(rows.len());
        if rows.is_empty() {
            metrics.empty_frames += 1;
            if selected.is_some() {
                "Target statuses: none"
            } else {
                "Select a target to inspect statuses"
            }
            .to_owned()
        } else {
            metrics.active_frames += 1;
            let mut text = String::from("Target statuses");
            for status in rows {
                let own = state.id == Some(status.source.entity.value());
                if own {
                    metrics.own_frames += 1;
                } else {
                    metrics.other_frames += 1;
                }
                let definition = definitions
                    .0
                    .definition(&status.status)
                    .expect("Validated status catalog");
                let seconds =
                    status.expires_at.saturating_sub(now) as f64 / f64::from(view.rate.hz());
                text.push_str(&format!(
                    "\n{}{} | {:.1}s",
                    definition.name(),
                    if own { " (you)" } else { " (other)" },
                    seconds
                ));
            }
            text
        }
    } else {
        "Statuses: waiting for server".to_owned()
    };
    for mut label in &mut labels {
        if label.0 != text {
            label.0.clone_from(&text);
        }
    }
}

fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        crate::fantasy_ui::hud_shadow(),
        Pickable::IGNORE,
    )
}

pub(crate) fn widgets(
    parent: &mut ChildSpawnerCommands,
    owner: bool,
    skin: &crate::fantasy_ui::Skin,
) {
    parent
        .spawn((
            EffectRow(if owner {
                Subject::Owner
            } else {
                Subject::Target
            }),
            Node {
                column_gap: px(4),
                width: percent(100),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            for _ in 0..theia_network::simulation::combat::MAX_TARGET_STATUSES {
                row.spawn((
                    EffectSlot,
                    crate::ui_theme::slot::frame(Node {
                        width: px(26),
                        height: px(26),
                        flex_shrink: 0.,
                        display: Display::None,
                        ..default()
                    }),
                    Hovered::default(),
                    AccessibleLabel::new("Status effect"),
                ))
                .with_children(|slot| {
                    slot.spawn((
                        skin.icon("theia.dev.scorch"),
                        crate::ui_theme::slot::art(),
                        bevy::picking::Pickable::IGNORE,
                    ));
                    slot.spawn((
                        DurationLabel,
                        text("", 11.),
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: px(0),
                            right: px(1),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        Detail,
                        text("", 14.),
                        crate::ui_theme::Panel,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(28),
                            min_width: px(148),
                            padding: UiRect::axes(px(8), px(5)),
                            display: Display::None,
                            ..default()
                        },
                        GlobalZIndex(70),
                    ));
                });
            }
        });
}

type IconText<'w, 's> = Query<
    'w,
    's,
    (&'static mut Text, &'static mut Node, Has<Detail>),
    (Or<(With<DurationLabel>, With<Detail>)>, Without<EffectSlot>),
>;
fn paint_icons(
    mut commands: Commands,
    network: Res<Network>,
    definitions: Res<Definitions>,
    selection: Option<Res<crate::world_input::Selection>>,
    rows: Query<(&EffectRow, &Children)>,
    mut slots: Query<(&Children, &Hovered, &mut Node, &AccessibleLabel), With<EffectSlot>>,
    mut texts: IconText,
) {
    if rows.is_empty() {
        return;
    }
    let state = network.state();
    let welcome = state
        .welcome
        .as_ref()
        .filter(|w| state.error.is_none() && state.id == Some(w.id));
    let view = state.statuses.as_ref().filter(|view| {
        welcome.is_some_and(|w| {
            view.epoch == w.epoch && state.snapshot.as_ref().is_some_and(|s| s.epoch == w.epoch)
        })
    });
    let selected = selection
        .as_ref()
        .is_none_or(|s| s.friendly.is_none())
        .then(|| network.confirmed_hostile_target(&state))
        .flatten()
        .map(|id| id.value());
    for (row, slots_in_row) in &rows {
        let id = match row.0 {
            Subject::Owner => welcome.map(|w| w.id),
            Subject::Target => selected,
        };
        let now = view.map_or(0, |view| {
            state
                .snapshot
                .as_ref()
                .map_or(view.tick, |s| s.tick.max(view.tick))
        });
        let mut active = view
            .into_iter()
            .flat_map(|v| &v.statuses)
            .filter(|s| Some(s.target.entity.value()) == id && s.expires_at > now);
        for child in slots_in_row.iter() {
            let Ok((content, hovered, mut node, access)) = slots.get_mut(child) else {
                continue;
            };
            let effect = active.next().and_then(|s| {
                let name = definitions.0.definition(&s.status)?.name();
                let hz = u64::from(view?.rate.hz());
                let seconds = s.expires_at.saturating_sub(now).div_ceil(hz);
                Some((name, seconds))
            });
            let display = if effect.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
            let Some((name, seconds)) = effect else {
                continue;
            };
            if access.0 != name {
                commands
                    .entity(child)
                    .insert(AccessibleLabel::new(name.to_owned()));
            }
            for child in content.iter() {
                if let Ok((mut text, mut detail_node, detail)) = texts.get_mut(child) {
                    let value = if detail {
                        let display = if hovered.0 {
                            Display::Flex
                        } else {
                            Display::None
                        };
                        if detail_node.display != display {
                            detail_node.display = display;
                        }
                        format!("{name} · {seconds}s")
                    } else {
                        format!("{seconds}")
                    };
                    if text.0 != value {
                        **text = value;
                    }
                }
            }
        }
    }
}
