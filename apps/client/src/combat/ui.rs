//! Standard Bevy widgets and bounded, server-derived progress. No gameplay authority.
mod tooltip;
mod unit_frame;
use super::Config;
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
pub(super) use unit_frame::owner_level;

#[derive(Component)]
pub(crate) struct ActionButton(pub(crate) usize);
#[derive(Component)]
pub(crate) struct ActionLabel(pub(crate) usize);
#[derive(Component)]
pub(super) struct HealthFill;
#[derive(Component)]
struct CastFill;
#[derive(Component)]
struct CooldownFill(usize);
#[derive(Component)]
pub(crate) struct CastLabel;
#[derive(Component)]
struct PlayerHealthFill;
#[derive(Component)]
struct ManaFill;
#[derive(Component)]
struct PlayerHealthLabel;
#[derive(Component)]
pub(crate) struct PlayerHeading;
#[derive(Component)]
struct ManaLabel;
#[derive(Component)]
struct LifeLabel;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Keyboard,
    Widget,
}
#[derive(Resource, Default)]
pub(super) struct Input(pub Option<(usize, Source)>);
pub(super) const VISIBLE_ACTIONS: usize = 5;
const PRIMARY_KEYS: [KeyCode; 12] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
    KeyCode::Minus,
    KeyCode::Equal,
];
const KEY_LABELS: [&str; 12] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="];
pub(super) fn action_limit(skin: bool) -> usize {
    if skin {
        PRIMARY_KEYS.len()
    } else {
        VISIBLE_ACTIONS
    }
}
fn key_label(slot: usize) -> &'static str {
    KEY_LABELS[slot]
}
pub(super) fn ability_name(id: &str) -> String {
    id.rsplit('.')
        .next()
        .unwrap_or(id)
        .split('-')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().unwrap().to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Default, serde::Serialize)]
pub(crate) struct Action {
    pub cooldown: f32,
    pub status: String,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Model {
    pub player_level: Option<u8>,
    pub life_blocked: bool,
    pub life_status: String,
    pub membership: Option<bool>,
    pub vitals_ready: bool,
    pub player_health: f32,
    pub mana: f32,
    pub player_health_text: String,
    pub mana_text: String,
    pub health: f32,
    pub cast: f32,
    pub cooldown: f32,
    pub action: String,
    pub status: String,
    pub has_target: bool,
    pub connected: bool,
    pub actions: Vec<Action>,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub button_layout_samples: u64,
    pub combat_label_frames: u64,
    pub peace_label_frames: u64,
    pub health_samples: u64,
    pub player_health_samples: u64,
    pub mana_samples: u64,
    pub cast_samples: u64,
    pub cooldown_samples: u64,
    pub max_fill_error_px: f32,
    pub invalid_layouts: u64,
}
/// Ratios are cosmetic views of authoritative tick deadlines. Saturating math
/// handles late/stale observations; it never acknowledges readiness or damage.
pub(super) fn remaining(now: u64, until: u64, duration: u64) -> f32 {
    if duration == 0 {
        return 0.;
    }
    (until.saturating_sub(now) as f64 / duration as f64).clamp(0., 1.) as f32
}
#[derive(Resource, Default)]
struct WidgetPending(Option<usize>);
fn activate(
    event: On<Activate>,
    buttons: Query<&ActionButton>,
    mut pending: ResMut<WidgetPending>,
) {
    if let Ok(button) = buttons.get(event.entity) {
        pending.0.get_or_insert(button.0);
    }
}
pub(super) fn install(app: &mut App) {
    assert!(
        app.is_plugin_added::<bevy::ui::picking_backend::UiPickingPlugin>(),
        "Combat UI requires Bevy UI picking"
    );
    app.init_resource::<Input>()
        .init_resource::<WidgetPending>()
        .add_observer(activate)
        .init_resource::<Model>()
        .init_resource::<Metrics>()
        .add_systems(Startup, setup)
        .add_systems(Update, collect.before(super::present))
        .add_systems(Update, paint.after(super::present))
        .add_systems(Last, observe);
}
/// Developer offscreen preview of the same widgets/painter, with explicit fixture
/// values. No Network resource, game command collection, server or account.
pub(crate) fn install_preview(app: &mut App) {
    app.add_systems(Startup, (setup, super::setup))
        .add_systems(Update, (preview_hover, paint).chain())
        .add_systems(
            PostStartup,
            |mut labels: Query<(&mut Text, Has<super::Hud>, Has<crate::targeting::TargetHud>)>| {
                for (mut label, health, heading) in &mut labels {
                    if health {
                        **label = "Health 57 / 100".into();
                    }
                    if heading {
                        **label = "Target: Wolf".into();
                    }
                }
            },
        );
}
fn preview_hover(mut commands: Commands, actions: Query<(Entity, &ActionButton)>) {
    // A visual fixture, not OS input or a gameplay command.
    for (entity, action) in &actions {
        if action.0 == 0 {
            commands.entity(entity).insert(Hovered(true));
        }
    }
}
fn collect(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<WidgetPending>,
    mut input: ResMut<Input>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    windows: Query<&Window>,
) {
    let widget = pending.0.take();
    input.0 = None;
    if !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    input.0 = if let Some(slot) = widget {
        Some((slot, Source::Widget))
    } else {
        PRIMARY_KEYS
            .iter()
            .take(action_limit(skin.is_some()))
            .position(|key| keys.just_pressed(*key))
            .map(|slot| (slot, Source::Keyboard))
    };
}
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
    )
}
fn setup(
    mut commands: Commands,
    config: Res<Config>,
    graybox: Option<Res<crate::graybox::Graybox>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
) {
    if config.1.is_some() {
        commands
            .spawn((
                if skin.is_some() {
                    crate::fantasy_ui::unit_frame(true)
                } else {
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(if graybox.is_some() {
                            crate::minimap::VITALS_RIGHT
                        } else {
                            12.
                        }),
                        top: px(12),
                        width: px(300),
                        padding: UiRect::all(px(10)),
                        row_gap: px(6),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    }
                },
                crate::fantasy_ui::UnitFrame(true),
                BackgroundColor(if skin.is_some() {
                    crate::fantasy_ui::PANEL
                } else {
                    Color::srgba(0.02, 0.03, 0.05, 0.9)
                }),
                BorderColor::all(crate::fantasy_ui::GOLD),
            ))
            .with_children(|panel| {
                if skin.is_some() {
                    panel.spawn((PlayerHeading, text("Player", 16.)));
                }
                panel.spawn((
                    PlayerHealthLabel,
                    text("Waiting for health", if skin.is_some() { 14. } else { 16. }),
                ));
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            height: px(if skin.is_some() { 12. } else { 10. }),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.16, 0.12)),
                    ))
                    .with_child((
                        PlayerHealthFill,
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.68, 0.37)),
                    ));
                panel.spawn((
                    ManaLabel,
                    text("Waiting for mana", if skin.is_some() { 14. } else { 16. }),
                ));
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            height: px(if skin.is_some() { 8. } else { 10. }),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.12, 0.18)),
                    ))
                    .with_child((
                        ManaFill,
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.42, 0.92)),
                    ));
                if config.2 {
                    panel.spawn((
                        LifeLabel,
                        text(
                            "Waiting for player life",
                            if skin.is_some() { 14. } else { 16. },
                        ),
                    ));
                }
            });
    }

    if let Some(skin) = skin.as_deref() {
        compact_actions(&mut commands, &config, skin);
    } else {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(0),
                    bottom: px(108),
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    column_gap: px(8),
                    ..default()
                },
                bevy::picking::Pickable::IGNORE,
            ))
            .with_children(|parent| {
                for (slot, ability) in config
                    .0
                    .abilities()
                    .iter()
                    .take(VISIBLE_ACTIONS)
                    .enumerate()
                {
                    let name = ability_name(ability.id().as_str());
                    let key = key_label(slot);
                    parent
                        .spawn((
                            Button,
                            ActivateOnPress,
                            Hovered::default(),
                            ActionButton(slot),
                            AccessibleLabel::new(format!("Cast {name} ({key})")),
                            Node {
                                width: px(if config.0.abilities().len() == 1 {
                                    272.
                                } else {
                                    224.
                                }),
                                min_height: px(88),
                                padding: UiRect::all(px(12)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.07, 0.12, 0.19)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                ActionLabel(slot),
                                text(&format!("[{key}] {name}\nWaiting for server"), 18.),
                            ));
                            button.spawn((
                                CooldownFill(slot),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    bottom: px(0),
                                    width: percent(0),
                                    height: px(6),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.92, 0.68, 0.2)),
                            ));
                        });
                }
            });
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(0),
                bottom: px(if skin.is_some() { 112. } else { 208. }),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: px(320),
                        min_height: px(30),
                        padding: UiRect::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        CastFill,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            bottom: px(0),
                            width: percent(0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.36, 0.57)),
                    ));
                    bar.spawn((CastLabel, text("Waiting for combat", 16.)));
                });
        });
}
/// Twelve Classic primary bindings; only configured actions issue commands.
/// Empty positions are decoration and never invent abilities.
fn compact_actions(commands: &mut Commands, config: &Config, skin: &crate::fantasy_ui::Skin) {
    use crate::fantasy_ui::{ACTION_WIDTH, GOLD, PANEL, SLOT, SLOTS};
    use bevy::picking::Pickable;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(12),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    max_width: percent(96),
                    padding: UiRect::all(px(8)),
                    border: UiRect::all(px(1)),
                    column_gap: px(4),
                    row_gap: px(4),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(GOLD),
            ))
            .with_children(|bar| {
                for slot in 0..SLOTS {
                    let node = Node {
                        width: px(SLOT),
                        height: px(SLOT),
                        min_width: px(SLOT),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    };
                    let Some(ability) = config.0.abilities().get(slot) else {
                        bar.spawn((
                            node,
                            BackgroundColor(Color::srgba(0.025, 0.022, 0.02, 0.96)),
                            Outline::new(px(1), px(-1), GOLD),
                        ))
                        .with_children(|empty| {
                            empty
                                .spawn((
                                    text(key_label(slot), 12.),
                                    Pickable::IGNORE,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(5),
                                        top: px(2),
                                        ..default()
                                    },
                                ))
                                .insert(TextColor(GOLD));
                        });
                        continue;
                    };
                    let name = ability_name(ability.id().as_str());
                    bar.spawn((
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        ActionButton(slot),
                        AccessibleLabel::new(format!("Cast {name} ({})", key_label(slot))),
                        node,
                        BackgroundColor(PANEL),
                        Outline::new(px(1), px(-1), GOLD),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Node {
                                width: px(36),
                                height: px(if ability.id().as_str() == "theia.dev.sword-slash" {
                                    18.
                                } else {
                                    36.
                                }),
                                ..default()
                            },
                            ImageNode::new(skin.icon(ability.id().as_str())),
                            Pickable::IGNORE,
                        ));
                        button.spawn((
                            CooldownFill(slot),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0),
                                bottom: px(4),
                                width: percent(0),
                                height: px(5),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.92, 0.68, 0.2)),
                            Pickable::IGNORE,
                        ));
                        button.spawn((
                            text(key_label(slot), 14.),
                            Pickable::IGNORE,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(5),
                                top: px(2),
                                ..default()
                            },
                        ));
                        button.spawn((
                            ActionLabel(slot),
                            text("Waiting for server", 16.),
                            Pickable::IGNORE,
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: px(44),
                                left: px(ACTION_WIDTH + 12. - slot as f32 * (SLOT + 4.)),
                                width: px(280),
                                padding: UiRect::all(px(10)),
                                border: UiRect::all(px(1)),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(PANEL),
                            BorderColor::all(GOLD),
                            GlobalZIndex(15),
                        ));
                    });
                }
            });
        });
}
type FillComponents<'a> = (
    &'a mut Node,
    Option<&'a HealthFill>,
    Option<&'a CastFill>,
    Option<&'a CooldownFill>,
    Option<&'a PlayerHealthFill>,
    Option<&'a ManaFill>,
);
type LabelComponents<'a> = (
    &'a mut Text,
    Option<&'a ActionLabel>,
    Option<&'a CastLabel>,
    Option<&'a PlayerHealthLabel>,
    Option<&'a ManaLabel>,
    Option<&'a LifeLabel>,
    Option<&'a PlayerHeading>,
);
type ButtonComponents<'a> = (
    Has<Pressed>,
    Option<&'a Hovered>,
    &'a mut BackgroundColor,
    &'a ActionButton,
);
fn paint(
    model: Res<Model>,
    config: Res<Config>,
    mut fills: Query<FillComponents<'_>, Without<ActionLabel>>,
    mut labels: Query<LabelComponents<'_>>,
    mut buttons: Query<ButtonComponents<'_>, With<ActionButton>>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut tooltips: Query<(&ActionLabel, &mut Node), With<ActionLabel>>,
) {
    for (mut node, health, cast, cooldown, player_health, mana) in &mut fills {
        let ratio = if health.is_some() {
            model.health
        } else if cast.is_some() {
            model.cast
        } else if let Some(cooldown) = cooldown {
            model
                .actions
                .get(cooldown.0)
                .map_or(model.cooldown, |a| a.cooldown)
        } else if player_health.is_some() {
            model.player_health
        } else if mana.is_some() {
            model.mana
        } else {
            continue;
        };
        let width = percent(ratio * 100.);
        if node.width != width {
            node.width = width;
        }
    }
    for (mut label, action, cast, player_health, mana, life, heading) in &mut labels {
        let value = if let Some(action) = action {
            let ability = &config.0.abilities()[action.0];
            let state = if !model.connected {
                "Disconnected / loading"
            } else if model.life_blocked {
                &model.life_status
            } else if !model.has_target {
                "Select a target with Tab"
            } else {
                model
                    .actions
                    .get(action.0)
                    .map_or(model.action.as_str(), |a| a.status.as_str())
            };
            if skin.is_some() {
                tooltip::describe(ability, key_label(action.0), state)
            } else {
                let name = ability_name(ability.id().as_str());
                let key = key_label(action.0);
                let cost = ability.mana_cost();
                if cost > 0 {
                    format!("[{key}] {name}\n{cost} mana | {state}")
                } else {
                    format!("[{key}] {name}\n{state}")
                }
            }
        } else if cast.is_some() {
            model.status.clone()
        } else if player_health.is_some() {
            if model.vitals_ready {
                model.player_health_text.clone()
            } else {
                "Waiting for health".into()
            }
        } else if mana.is_some() {
            if model.vitals_ready {
                model.mana_text.clone()
            } else {
                "Waiting for mana".into()
            }
        } else if life.is_some() {
            model.life_status.clone()
        } else if heading.is_some() {
            model.player_level.map_or_else(
                || "Player".into(),
                |level| format!("Player · Level {level}"),
            )
        } else {
            continue;
        };
        if label.0 != value {
            **label = value;
        }
    }
    for (pressed, hovered, mut colour, action) in &mut buttons {
        if skin.is_some() {
            for (label, mut node) in &mut tooltips {
                if label.0 == action.0 {
                    let display = if hovered.is_some_and(|h| h.0) {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    if node.display != display {
                        node.display = display;
                    }
                }
            }
        }
        let value = if !model.connected || !model.has_target || model.life_blocked {
            Color::srgb(0.13, 0.14, 0.16)
        } else {
            if pressed {
                Color::srgb(0.14, 0.36, 0.5)
            } else if hovered.is_some_and(|h| h.0) {
                Color::srgb(0.1, 0.24, 0.34)
            } else {
                if skin.is_some() {
                    crate::fantasy_ui::PANEL
                } else {
                    Color::srgb(0.07, 0.12, 0.19)
                }
            }
        };
        if colour.0 != value {
            colour.0 = value;
        }
    }
}
type FillLayout<'a> = (
    &'a ComputedNode,
    &'a ChildOf,
    Option<&'a HealthFill>,
    Option<&'a CastFill>,
    Option<&'a CooldownFill>,
    Option<&'a PlayerHealthFill>,
    Option<&'a ManaFill>,
);
type LifeLayout<'a> = (
    &'a Text,
    &'a ComputedNode,
    &'a InheritedVisibility,
    Option<&'a bevy::text::TextLayoutInfo>,
);
fn observe(
    model: Res<Model>,
    fills: Query<FillLayout<'_>>,
    parents: Query<&ComputedNode>,
    buttons: Query<(&ComputedNode, &InheritedVisibility), With<ActionButton>>,
    mut metrics: ResMut<Metrics>,
    life_labels: Query<LifeLayout<'_>, With<LifeLabel>>,
) {
    for (label, node, visible, glyphs) in &life_labels {
        if model.connected
            && visible.get()
            && node.size().min_element() > 0.
            && glyphs.is_some_and(|g| !g.glyphs.is_empty())
            && label.0 == model.life_status
        {
            metrics.combat_label_frames +=
                u64::from(model.membership == Some(true) && label.0.starts_with("In combat"));
            metrics.peace_label_frames +=
                u64::from(model.membership == Some(false) && label.0.starts_with("Out of combat"));
        }
    }
    for (node, visible) in &buttons {
        if visible.get() && node.size().min_element() > 0. {
            metrics.button_layout_samples += 1;
        }
    }
    for (node, parent, health, cast, cooldown, player_health, mana) in &fills {
        let ratio = if health.is_some() {
            metrics.health_samples += u64::from(model.health > 0. && model.health < 1.);
            model.health
        } else if cast.is_some() {
            metrics.cast_samples += u64::from(model.cast > 0. && model.cast < 1.);
            model.cast
        } else if let Some(cooldown) = cooldown {
            let ratio = model
                .actions
                .get(cooldown.0)
                .map_or(model.cooldown, |a| a.cooldown);
            metrics.cooldown_samples += u64::from(ratio > 0. && ratio < 1.);
            ratio
        } else if player_health.is_some() {
            metrics.player_health_samples += u64::from(model.vitals_ready);
            model.player_health
        } else if mana.is_some() {
            metrics.mana_samples += u64::from(model.vitals_ready && model.mana < 1.);
            model.mana
        } else {
            continue;
        };
        let Ok(parent) = parents.get(parent.parent()) else {
            continue;
        };
        let width = node.size().x;
        let expected = parent.size().x * ratio;
        if !width.is_finite() || !expected.is_finite() || width < 0. || width > parent.size().x + 1.
        {
            metrics.invalid_layouts += 1;
        }
        metrics.max_fill_error_px = metrics.max_fill_error_px.max((width - expected).abs());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_heading_updates_with_level_and_drops_unavailable_progression() {
        let mut app = widget_app(true);
        app.update();
        for (level, expected) in [
            (Some(1), "Player · Level 1"),
            (Some(2), "Player · Level 2"),
            (None, "Player"),
        ] {
            app.world_mut().resource_mut::<Model>().player_level = level;
            app.update();
            let world = app.world_mut();
            let heading = world
                .query_filtered::<&Text, With<PlayerHeading>>()
                .single(world)
                .unwrap();
            assert_eq!(heading.0, expected);
            assert_eq!(world.resource::<Input>().0, None);
        }
    }
    #[test]
    fn compact_bar_supports_all_twelve_default_keys_without_repeating_held_input() {
        let mut app = widget_app(true);
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
        let template = raw["abilities"][0].clone();
        raw["abilities"] = serde_json::Value::Array(
            (0..12)
                .map(|slot| {
                    let mut ability = template.clone();
                    ability["id"] = format!("theia.fixture.action-{slot}").into();
                    ability
                })
                .collect(),
        );
        app.world_mut().resource_mut::<Config>().0 = std::sync::Arc::new(
            theia_core::abilities::Catalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ActionButton>()
                .iter(app.world())
                .count(),
            12
        );
        for (key, slot) in [
            (KeyCode::Digit6, 5),
            (KeyCode::Digit0, 9),
            (KeyCode::Minus, 10),
            (KeyCode::Equal, 11),
        ] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            assert_eq!(
                app.world().resource::<Input>().0,
                Some((slot, Source::Keyboard))
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
            app.update();
            assert_eq!(
                app.world().resource::<Input>().0,
                None,
                "Held keys do not repeat"
            );
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(key);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }
        let world = app.world_mut();
        let mut labels = world.query::<(&ActionLabel, &Text)>();
        for (slot, key) in [(9, "0"), (10, "-"), (11, "=")] {
            let (_, label) = labels
                .iter(world)
                .find(|(action, _)| action.0 == slot)
                .unwrap();
            assert!(label.0.starts_with(&format!("[{key}] Action {slot}")));
        }
        let mut legacy = widget_app(false);
        legacy.update();
        legacy
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Equal);
        legacy.update();
        assert_eq!(
            legacy.world().resource::<Input>().0,
            None,
            "Retained legacy presentation keeps five keys"
        );
    }
    #[test]
    fn actual_hovered_widget_updates_tooltip_status_without_submitting_an_ability() {
        let mut app = widget_app(true);
        app.update();
        let button = app
            .world_mut()
            .query::<(Entity, &ActionButton)>()
            .iter(app.world())
            .find(|(_, action)| action.0 == 0)
            .unwrap()
            .0;
        app.world_mut().entity_mut(button).insert(Hovered(true));
        app.update();
        {
            let world = app.world_mut();
            let (_, text, node) = world
                .query::<(&ActionLabel, &Text, &Node)>()
                .iter(world)
                .find(|(label, _, _)| label.0 == 0)
                .unwrap();
            assert!(text.0.contains("30 m range") && text.0.ends_with("Disconnected / loading"));
            assert_eq!(node.display, Display::Flex);
        }
        {
            let mut model = app.world_mut().resource_mut::<Model>();
            model.connected = true;
            model.has_target = true;
            model.actions = vec![Action {
                cooldown: 0.5,
                status: "Cooldown 1.0s".into(),
            }];
        }
        app.update();
        let world = app.world_mut();
        let (_, text) = world
            .query::<(&ActionLabel, &Text)>()
            .iter(world)
            .find(|(label, _)| label.0 == 0)
            .unwrap();
        assert!(text.0.ends_with("Cooldown 1.0s"));
        assert!(world.resource::<Input>().0.is_none());
        assert!(world.resource::<WidgetPending>().0.is_none());
        let mut legacy = widget_app(false);
        legacy.update();
        let world = legacy.world_mut();
        let (_, text) = world
            .query::<(&ActionLabel, &Text)>()
            .iter(world)
            .find(|(label, _)| label.0 == 0)
            .unwrap();
        assert_eq!(text.0, "[1] Arcane Bolt\n25 mana | Disconnected / loading");
    }
    fn widget_app(skin: bool) -> App {
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        let catalog = theia_core::abilities::Catalog::from_json(include_bytes!(
            "../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
        let stats = theia_core::resources::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-character-stats.json"
        ))
        .unwrap();
        app.insert_resource(Config(std::sync::Arc::new(catalog), Some(stats), true))
            .init_resource::<Model>()
            .init_resource::<Input>()
            .init_resource::<WidgetPending>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_observer(activate)
            .add_systems(Startup, setup)
            .add_systems(Update, (collect, paint).chain());
        if skin {
            app.insert_resource(crate::fantasy_ui::Skin::fixture());
        }
        app
    }
    #[derive(Resource, Default, Clone, Copy, Debug)]
    struct DirtyFrame(usize, usize, usize);
    fn dirty_frame(
        text: Query<(), Changed<Text>>,
        nodes: Query<(), Changed<Node>>,
        colours: Query<(), Changed<BackgroundColor>>,
        mut dirty: ResMut<DirtyFrame>,
    ) {
        *dirty = DirtyFrame(
            text.iter().count(),
            nodes.iter().count(),
            colours.iter().count(),
        );
    }
    #[test]
    fn steady_hud_avoids_dirty_widgets_but_real_state_changes_still_paint() {
        let mut samples = Vec::new();
        for skin in [false, true] {
            let mut app = widget_app(skin);
            app.init_resource::<DirtyFrame>()
                .add_systems(Last, dirty_frame);
            app.update();
            app.update();
            let idle = *app.world().resource::<DirtyFrame>();
            samples.push((skin, idle.0, idle.1, idle.2));
            {
                let mut model = app.world_mut().resource_mut::<Model>();
                model.connected = true;
                model.has_target = true;
                model.vitals_ready = true;
                model.player_health = 0.45;
                model.player_health_text = "Health 45 / 100".into();
            }
            app.update();
            let changed = *app.world().resource::<DirtyFrame>();
            assert!(changed.0 > 0 && changed.1 > 0 && changed.2 > 0);
            let world = app.world_mut();
            let mut health = world.query_filtered::<(&Text,), With<PlayerHealthLabel>>();
            assert_eq!(health.single(world).unwrap().0.0, "Health 45 / 100");
            app.update();
            let settled = *app.world().resource::<DirtyFrame>();
            samples.push((skin, settled.0, settled.1, settled.2));
        }
        assert_eq!(
            samples,
            [
                (false, 0, 0, 0),
                (false, 0, 0, 0),
                (true, 0, 0, 0),
                (true, 0, 0, 0)
            ]
        );
    }
    #[test]
    fn compact_skin_keeps_authoritative_feedback_and_only_configured_actions_activate() {
        let mut app = widget_app(true);
        app.update(); // Also detects invalid bundles and conflicting query access.
        let world = app.world_mut();
        let buttons: Vec<_> = world
            .query::<(Entity, &ActionButton)>()
            .iter(world)
            .map(|(e, a)| (e, a.0))
            .collect();
        assert_eq!(buttons.len(), 2);
        let frames = world.query::<&ImageNode>().iter(world).count();
        assert_eq!(
            frames, 2,
            "Only configured ability icons require image nodes"
        );
        let first = buttons.iter().find(|(_, slot)| *slot == 0).unwrap().0;
        world.entity_mut(first).insert(Hovered(true));
        *world.resource_mut::<Model>() = Model {
            connected: true,
            has_target: true,
            vitals_ready: true,
            player_health: 0.37,
            mana: 0.4,
            player_health_text: "Health 37 / 100".into(),
            mana_text: "Mana 40 / 100".into(),
            actions: vec![Action {
                cooldown: 0.75,
                status: "Face the target".into(),
            }],
            ..default()
        };
        world.trigger(Activate { entity: first });
        app.update();
        assert_eq!(app.world().resource::<Input>().0, Some((0, Source::Widget)));
        let world = app.world_mut();
        let mut labels = world.query::<(&ActionLabel, &Text, &Node)>();
        for (action, text, node) in labels.iter(world) {
            if action.0 == 0 {
                assert!(text.0.contains("Face the target"));
                assert_eq!(node.display, Display::Flex);
            } else {
                assert_eq!(node.display, Display::None);
            }
        }
        let mut health = world.query_filtered::<&Node, With<PlayerHealthFill>>();
        assert_eq!(health.single(world).unwrap().width, percent(37));
        let mut cooldown = world.query::<(&CooldownFill, &Node)>();
        assert_eq!(
            cooldown
                .iter(world)
                .find(|(a, _)| a.0 == 0)
                .unwrap()
                .1
                .width,
            percent(75)
        );
        world.entity_mut(first).insert(Hovered(false));
        world.resource_mut::<Model>().connected = false;
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            None,
            "No repeated action from painting or hover"
        );
        let world = app.world_mut();
        let mut labels = world.query::<(&Text, &Node, &ActionLabel)>();
        assert!(
            labels
                .iter(world)
                .all(|(text, node, _)| text.0.contains("Disconnected / loading")
                    && node.display == Display::None)
        );
    }
    #[test]
    fn widget_activations_and_keyboard_coalesce_without_repeating_or_buffering_clicks() {
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.init_resource::<Input>()
            .init_resource::<WidgetPending>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_observer(activate)
            .add_systems(Update, collect);
        let button = app.world_mut().spawn(ActionButton(0)).id();
        app.update();
        assert_eq!(app.world().resource::<Input>().0, None);
        for _ in 0..100 {
            app.world_mut().trigger(Activate { entity: button });
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit1);
        app.update();
        assert_eq!(app.world().resource::<Input>().0, Some((0, Source::Widget)));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            None,
            "Activations must not create a backlog"
        );
        let unrelated = app.world_mut().spawn_empty().id();
        app.world_mut().trigger(Activate { entity: unrelated });
        app.update();
        assert_eq!(app.world().resource::<Input>().0, None);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Digit1);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit1);
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            Some((0, Source::Keyboard))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        let second = app.world_mut().spawn(ActionButton(1)).id();
        app.world_mut().trigger(Activate { entity: second });
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        assert_eq!(app.world().resource::<Input>().0, Some((1, Source::Widget)));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit2);
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            Some((1, Source::Keyboard))
        );
    }
    #[test]
    fn unfocused_ability_input_is_discarded_including_pending_widgets_and_held_keys() {
        let mut app = widget_app(true);
        app.update();
        let world = app.world_mut();
        let window = world
            .query_filtered::<Entity, With<Window>>()
            .single(world)
            .unwrap();
        let action = world
            .query::<(Entity, &ActionButton)>()
            .iter(world)
            .find(|(_, a)| a.0 == 0)
            .unwrap()
            .0;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit1);
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            Some((0, Source::Keyboard))
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut().trigger(Activate { entity: action });
        app.update();
        assert!(app.world().resource::<Input>().0.is_none());
        assert!(app.world().resource::<WidgetPending>().0.is_none());
        // Simulate the input plugin's next-frame edge clearing without touching OS input.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        app.update();
        assert!(
            app.world().resource::<Input>().0.is_none(),
            "No deferred click or held-key cast after focus returns"
        );
        app.world_mut().trigger(Activate { entity: action });
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            Some((0, Source::Widget)),
            "A fresh focused action still works"
        );
    }
    #[test]
    fn absent_or_ambiguous_windows_do_not_collect_ability_commands() {
        for count in [0, 2] {
            let mut app = App::new();
            app.init_resource::<Input>()
                .init_resource::<WidgetPending>()
                .init_resource::<ButtonInput<KeyCode>>()
                .add_systems(Update, collect);
            for _ in 0..count {
                app.world_mut().spawn(Window {
                    focused: true,
                    ..default()
                });
            }
            app.world_mut().resource_mut::<WidgetPending>().0 = Some(1);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Digit1);
            app.update();
            assert!(app.world().resource::<Input>().0.is_none());
            assert!(app.world().resource::<WidgetPending>().0.is_none());
        }
    }
    #[test]
    fn server_tick_progress_is_bounded_for_expired_zero_length_and_far_future_deadlines() {
        assert_eq!(remaining(10, 40, 30), 1.);
        assert_eq!(remaining(25, 40, 30), 0.5);
        assert_eq!(remaining(40, 40, 30), 0.);
        assert_eq!(remaining(41, 40, 30), 0.);
        assert_eq!(remaining(0, u64::MAX, 1), 1.);
        assert_eq!(remaining(0, 10, 0), 0.);
    }
}
