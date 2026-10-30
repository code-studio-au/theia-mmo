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
pub(crate) struct ActionButton;
#[derive(Component)]
struct ActionLabel;
#[derive(Component)]
pub(super) struct HealthFill;
#[derive(Component)]
struct CastFill;
#[derive(Component)]
struct CooldownFill;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Keyboard,
    Widget,
}
#[derive(Resource, Default)]
pub(super) struct Input(pub Option<Source>);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Model {
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
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub button_layout_samples: u64,
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
struct WidgetPending(bool);
fn activate(
    event: On<Activate>,
    buttons: Query<(), With<ActionButton>>,
    mut pending: ResMut<WidgetPending>,
) {
    if buttons.contains(event.entity) {
        pending.0 = true;
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
    input.0 = if std::mem::take(&mut pending.0) {
        Some(Source::Widget)
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(Source::Keyboard)
    } else {
        None
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
fn setup(mut commands: Commands, config: Res<Config>) {
    if config.1.is_some() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(12),
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
            });
    }

    let name = config.0.abilities()[0]
        .id()
        .as_str()
        .rsplit('.')
        .next()
        .unwrap()
        .replace('-', " ");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(0),
                bottom: px(108),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    ActivateOnPress,
                    Hovered::default(),
                    ActionButton,
                    AccessibleLabel::new(format!("Cast {name} (1)")),
                    Node {
                        width: px(272),
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
                        ActionLabel,
                        text(&format!("[1] {name}\nWaiting for server"), 18.),
                    ));
                    button.spawn((
                        CooldownFill,
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
        } else if cooldown.is_some() {
            model.cooldown
        } else if player_health.is_some() {
            model.player_health
        } else if mana.is_some() {
            model.mana
        } else {
            continue;
        };
        node.width = percent(ratio * 100.);
    }
    let name = config.0.abilities()[0]
        .id()
        .as_str()
        .rsplit('.')
        .next()
        .unwrap()
        .replace('-', " ");
    for (mut label, action, cast, player_health, mana) in &mut labels {
        if action.is_some() {
            let state = if !model.connected {
                "Disconnected / loading"
            } else if !model.has_target {
                "Select a target with Tab"
            } else {
                &model.action
            };
            let cost = config.0.abilities()[0].mana_cost();
            **label = if cost > 0 {
                format!("[1] {name}\n{cost} mana | {state}")
            } else {
                format!("[1] {name}\n{state}")
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
        }
    }
    for (pressed, hovered, mut colour) in &mut buttons {
        colour.0 = if !model.connected || !model.has_target {
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
fn observe(
    model: Res<Model>,
    fills: Query<FillLayout<'_>>,
    parents: Query<&ComputedNode>,
    buttons: Query<(&ComputedNode, &InheritedVisibility), With<ActionButton>>,
    mut metrics: ResMut<Metrics>,
) {
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
        } else if cooldown.is_some() {
            metrics.cooldown_samples += u64::from(model.cooldown > 0. && model.cooldown < 1.);
            model.cooldown
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
        let button = app.world_mut().spawn(ActionButton).id();
        app.update();
        assert_eq!(app.world().resource::<Input>().0, None);
        for _ in 0..100 {
            app.world_mut().trigger(Activate { entity: button });
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit1);
        app.update();
        assert_eq!(app.world().resource::<Input>().0, Some(Source::Widget));
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
        assert_eq!(app.world().resource::<Input>().0, Some(Source::Keyboard));
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
