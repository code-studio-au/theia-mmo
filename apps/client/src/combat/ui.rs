//! Standard Bevy widgets and bounded, server-derived progress. No gameplay authority.
use super::Config;
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};

#[derive(Component)]
pub(crate) struct ActionButton(pub(crate) usize);
#[derive(Component)]
struct ActionLabel(usize);
#[derive(Component)]
pub(super) struct HealthFill;
#[derive(Component)]
struct CastFill;
#[derive(Component)]
struct CooldownFill(usize);
#[derive(Component)]
struct CastLabel;
#[derive(Component)]
struct PlayerHealthFill;
#[derive(Component)]
struct ManaFill;
#[derive(Component)]
struct PlayerHealthLabel;
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
#[derive(Default, serde::Serialize)]
pub(crate) struct Action {
    pub cooldown: f32,
    pub status: String,
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Model {
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
fn collect(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<WidgetPending>,
    mut input: ResMut<Input>,
) {
    input.0 = if let Some(slot) = pending.0.take() {
        Some((slot, Source::Widget))
    } else {
        [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
        ]
        .iter()
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
) {
    if config.1.is_some() {
        commands
            .spawn((
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
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
            ))
            .with_children(|panel| {
                panel.spawn((PlayerHealthLabel, text("Waiting for health", 16.)));
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            height: px(10),
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
                panel.spawn((ManaLabel, text("Waiting for mana", 16.)));
                panel
                    .spawn((
                        Node {
                            width: percent(100),
                            height: px(10),
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
                    panel.spawn((LifeLabel, text("Waiting for player life", 16.)));
                }
            });
    }

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
                let name = ability
                    .id()
                    .as_str()
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .replace('-', " ");
                let key = slot + 1;
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
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(0),
                bottom: px(208),
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
);
type ButtonComponents<'a> = (Has<Pressed>, Option<&'a Hovered>, &'a mut BackgroundColor);
fn paint(
    model: Res<Model>,
    config: Res<Config>,
    mut fills: Query<FillComponents<'_>>,
    mut labels: Query<LabelComponents<'_>>,
    mut buttons: Query<ButtonComponents<'_>, With<ActionButton>>,
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
        node.width = percent(ratio * 100.);
    }
    for (mut label, action, cast, player_health, mana, life) in &mut labels {
        if let Some(action) = action {
            let ability = &config.0.abilities()[action.0];
            let name = ability
                .id()
                .as_str()
                .rsplit('.')
                .next()
                .unwrap()
                .replace('-', " ");
            let key = action.0 + 1;
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
            let cost = ability.mana_cost();
            **label = if cost > 0 {
                format!("[{key}] {name}\n{cost} mana | {state}")
            } else {
                format!("[{key}] {name}\n{state}")
            };
        } else if cast.is_some() {
            **label = model.status.clone();
        } else if player_health.is_some() {
            **label = if model.vitals_ready {
                model.player_health_text.clone()
            } else {
                "Waiting for health".into()
            };
        } else if mana.is_some() {
            **label = if model.vitals_ready {
                model.mana_text.clone()
            } else {
                "Waiting for mana".into()
            };
        } else if life.is_some() {
            **label = model.life_status.clone();
        }
    }
    for (pressed, hovered, mut colour) in &mut buttons {
        colour.0 = if !model.connected || !model.has_target || model.life_blocked {
            Color::srgb(0.13, 0.14, 0.16)
        } else {
            if pressed {
                Color::srgb(0.14, 0.36, 0.5)
            } else if hovered.is_some_and(|h| h.0) {
                Color::srgb(0.1, 0.24, 0.34)
            } else {
                Color::srgb(0.07, 0.12, 0.19)
            }
        };
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
    fn widget_activations_and_keyboard_coalesce_without_repeating_or_buffering_clicks() {
        let mut app = App::new();
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
    fn server_tick_progress_is_bounded_for_expired_zero_length_and_far_future_deadlines() {
        assert_eq!(remaining(10, 40, 30), 1.);
        assert_eq!(remaining(25, 40, 30), 0.5);
        assert_eq!(remaining(40, 40, 30), 0.);
        assert_eq!(remaining(41, 40, 30), 0.);
        assert_eq!(remaining(0, u64::MAX, 1), 1.);
        assert_eq!(remaining(0, 10, 0), 0.);
    }
}
