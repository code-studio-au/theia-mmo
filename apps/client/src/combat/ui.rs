//! Standard Bevy widgets and bounded, server-derived progress. No gameplay authority.
pub(super) mod tooltip;
pub(super) mod unit_frame;
use super::Config;
use bevy::{
    picking::hover::Hovered,
    prelude::*,
    text::FontSize,
    ui::Pressed,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
pub(super) use unit_frame::{owner_identity, owner_level};

#[derive(Component)]
pub(crate) struct ActionButton(pub(crate) usize);
#[derive(Component)]
pub(crate) struct ActionSlot(pub(crate) usize);
#[derive(Component)]
struct ActionBar(u64);
#[derive(Component)]
pub(crate) struct ActionLabel(pub(crate) usize);
#[derive(Component)]
pub(super) struct HealthFill;
#[derive(Component)]
pub(super) struct TargetHealthLabel;
#[derive(Component)]
struct CastFill;
#[derive(Component)]
struct CooldownFill(usize);
#[derive(Component)]
pub(crate) struct CastLabel;
#[derive(Component)]
struct CastSurface;
#[derive(Component)]
struct CastIcon;
#[derive(serde::Serialize)]
pub(crate) struct Cast {
    pub(crate) ability: String,
    pub(crate) name: String,
    pub(crate) elapsed: f64,
    pub(crate) total: f64,
}
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
    pub player_appearance: Option<theia_core::appearance::Appearance>,
    pub target_health_text: String,
    pub target_is_wolf: bool,
    pub player_name: Option<String>,
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
    pub cast_info: Option<Cast>,
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
        .add_systems(Startup, (super::action_bar::prepare, setup).chain())
        .add_systems(Update, refresh_bar.before(collect))
        .add_systems(Update, collect.before(super::present))
        .add_systems(
            Update,
            (paint, paint_portrait, paint_cast).after(super::present),
        )
        .add_systems(Last, observe);
}
/// Developer offscreen preview of the same widgets/painter, with explicit fixture
/// values. No Network resource, game command collection, server or account.
pub(crate) fn install_preview(app: &mut App) {
    app.add_systems(
        Startup,
        (super::action_bar::prepare, setup, super::setup).chain(),
    )
    .add_systems(
        Update,
        (preview_hover, paint, paint_portrait, paint_cast).chain(),
    )
    .add_systems(
        PostStartup,
        |mut labels: Query<(&mut Text, Has<super::Hud>, Has<crate::targeting::TargetHud>)>| {
            for (mut label, health, heading) in &mut labels {
                if health {
                    **label = "Health 57 / 100".into();
                }
                if heading {
                    **label = "Wolf".into();
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
    book: Option<Res<super::book::Ui>>,
) {
    let widget = pending.0.take();
    input.0 = None;
    if !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    input.0 = if let Some(slot) =
        widget.filter(|_| !book.as_ref().is_some_and(|b| b.open || b.closed_this_frame))
    {
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
    layout: Res<super::action_bar::Layout>,
    server: Option<Res<AssetServer>>,
    references: Option<Res<crate::reference_scene::Config>>,
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
                    Color::NONE
                } else {
                    Color::srgba(0.02, 0.03, 0.05, 0.9)
                }),
                bevy::picking::Pickable::IGNORE,
            ))
            .with_children(|panel| {
                if skin.is_some() {
                    unit_frame::portrait(
                        panel,
                        crate::frontend::thumbnail_atlas(server.as_deref(), references.as_deref()),
                        true,
                    );
                    panel.spawn((
                        PlayerHeading,
                        text("Player", 22.),
                        Node {
                            min_height: px(24),
                            ..default()
                        },
                        crate::fantasy_ui::hud_shadow(),
                        TextLayout {
                            linebreak: bevy::text::LineBreak::NoWrap,
                            ..default()
                        },
                        bevy::picking::Pickable::IGNORE,
                    ));
                    unit_frame::bar(
                        panel,
                        unit_frame::Pool::Health,
                        (),
                        PlayerHealthFill,
                        PlayerHealthLabel,
                    );
                    unit_frame::bar(panel, unit_frame::Pool::Mana, (), ManaFill, ManaLabel);
                    crate::status_effects::widgets(panel, true);
                } else {
                    panel.spawn((PlayerHealthLabel, text("Waiting for health", 16.)));
                    legacy_pool(
                        panel,
                        PlayerHealthFill,
                        "bar.fill-health",
                        Color::srgb(0.2, 0.68, 0.37),
                    );
                    panel.spawn((ManaLabel, text("Waiting for mana", 16.)));
                    legacy_pool(
                        panel,
                        ManaFill,
                        "bar.fill-mana",
                        Color::srgb(0.18, 0.42, 0.92),
                    );
                }
                if config.2 {
                    let mut label =
                        panel.spawn((LifeLabel, text("", if skin.is_some() { 14. } else { 16. })));
                    if skin.is_some() {
                        label.insert((
                            crate::fantasy_ui::hud_shadow(),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    }
                }
            });
    }

    if let Some(skin) = skin.as_deref() {
        compact_actions(&mut commands, &config, &layout, skin);
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
                    CastSurface,
                    Node {
                        width: px(340),
                        min_height: px(34),
                        padding: UiRect::axes(px(12), px(5)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.94)),
                    bevy::picking::Pickable::IGNORE,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        CastFill,
                        crate::ui_theme::Sprite("bar.fill-progress"),
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
                    if let Some(skin) = skin.as_deref() {
                        bar.spawn((
                            crate::ui_theme::Sprite("bar.frame"),
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                        ));
                        bar.spawn((
                            CastIcon,
                            ImageNode::new(skin.icon("")),
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(-28),
                                top: px(-6),
                                width: px(44),
                                height: px(44),
                                display: Display::None,
                                ..default()
                            },
                            bevy::picking::Pickable::IGNORE,
                        ));
                    }
                    bar.spawn((
                        CastLabel,
                        text("Waiting for combat", 16.),
                        crate::fantasy_ui::hud_shadow(),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
        });
}
fn legacy_pool(
    parent: &mut ChildSpawnerCommands,
    fill: impl Bundle,
    sprite: &'static str,
    colour: Color,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(10),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.12, 0.18)),
        ))
        .with_child((
            fill,
            crate::ui_theme::Sprite(sprite),
            Node {
                width: percent(0),
                height: percent(100),
                ..default()
            },
            BackgroundColor(colour),
        ));
}

/// Twelve Classic primary bindings; only configured actions issue commands.
/// Empty positions are decoration and never invent abilities.
fn compact_actions(
    commands: &mut Commands,
    config: &Config,
    layout: &super::action_bar::Layout,
    skin: &crate::fantasy_ui::Skin,
) {
    use crate::fantasy_ui::{ACTION_WIDTH, GOLD, PANEL, SLOT, SLOTS};
    use bevy::picking::Pickable;
    commands
        .spawn((
            ActionBar(layout.revision),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(20),
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
                    width: px(ACTION_WIDTH),
                    padding: UiRect::new(px(8), px(8), px(6), px(13)),
                    border: UiRect::all(px(1)),
                    column_gap: px(4),
                    row_gap: px(4),
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                crate::ui_theme::Panel,
                BackgroundColor(PANEL),
                BorderColor::all(GOLD),
            ))
            .with_children(|bar| {
                for (right, sprite) in [(false, "hotbar.end-left"), (true, "hotbar.end-right")] {
                    let mut node = Node {
                        position_type: PositionType::Absolute,
                        top: px(8),
                        width: px(45),
                        height: px(45),
                        ..default()
                    };
                    if right {
                        node.right = px(-34);
                    } else {
                        node.left = px(-34);
                    }
                    bar.spawn((crate::ui_theme::Sprite(sprite), node, Pickable::IGNORE));
                }
                for slot in 0..SLOTS {
                    let node = Node {
                        width: px(SLOT),
                        height: px(SLOT),
                        min_width: px(SLOT),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    };
                    let Some(ability) = layout.ability(&config.0, slot) else {
                        bar.spawn((
                            ActionSlot(slot),
                            crate::ui_theme::Sprite("slot.empty.default"),
                            node,
                            BackgroundColor(Color::srgba(0.025, 0.022, 0.02, 0.96)),
                            Outline::new(px(1), px(-1), GOLD),
                        ))
                        .with_child(slot_key(slot));
                        continue;
                    };
                    let name = ability_name(ability.id().as_str());
                    bar.spawn((
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        ActionButton(slot),
                        crate::ui_theme::Style::Slot,
                        ActionSlot(slot),
                        AccessibleLabel::new(format!("Cast {name} ({})", key_label(slot))),
                        node,
                        BackgroundColor(PANEL),
                        Outline::new(px(1), px(-1), GOLD),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Node {
                                width: px(40),
                                height: px(40.),
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
                        button.spawn(slot_key(slot));
                        button.spawn((
                            ActionLabel(slot),
                            text("Waiting for server", 16.),
                            Pickable::IGNORE,
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: px(54),
                                left: px(ACTION_WIDTH + 12. - slot as f32 * (SLOT + 4.)),
                                width: px(280),
                                padding: UiRect::all(px(10)),
                                border: UiRect::all(px(1)),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(PANEL),
                            BorderColor::all(GOLD),
                            GlobalZIndex(70),
                        ));
                    });
                }
            });
        });
}
fn slot_key(slot: usize) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            bottom: px(-13),
            justify_content: JustifyContent::Center,
            ..default()
        },
        bevy::picking::Pickable::IGNORE,
        children![(
            text(key_label(slot), 13.),
            crate::fantasy_ui::hud_shadow(),
            bevy::picking::Pickable::IGNORE
        )],
    )
}

type CastNodes = Or<(With<CastSurface>, With<CastIcon>)>;
type CastWidget = (
    &'static mut Node,
    Option<&'static mut ImageNode>,
    Has<CastSurface>,
);
fn paint_cast(
    model: Res<Model>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    mut widgets: Query<CastWidget, CastNodes>,
) {
    let Some(skin) = skin else {
        return;
    };
    let feedback = model.cast_info.is_some()
        || (model.connected
            && !matches!(
                model.status.as_str(),
                "" | "Select a living target" | "Ready" | "Waiting for server"
            ));
    for (mut node, image, surface) in &mut widgets {
        let display = if if surface {
            feedback
        } else {
            model.cast_info.is_some()
        } {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let (Some(mut image), Some(cast)) = (image, &model.cast_info) {
            let handle = skin.icon(&cast.ability);
            if image.image != handle {
                image.image = handle;
            }
        }
    }
}

fn refresh_bar(
    mut commands: Commands,
    layout: Res<super::action_bar::Layout>,
    config: Res<Config>,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    bars: Query<(Entity, &ActionBar)>,
) {
    let Some(skin) = skin else {
        return;
    };
    if bars.iter().all(|(_, bar)| bar.0 == layout.revision) {
        return;
    }
    for (entity, _) in &bars {
        commands.entity(entity).despawn();
    }
    compact_actions(&mut commands, &config, &layout, &skin);
}
fn pool_text(value: &str, prefix: &str, waiting: &str, skinned: bool) -> String {
    if value.is_empty() {
        if skinned {
            "—".into()
        } else {
            waiting.into()
        }
    } else if skinned {
        value.strip_prefix(prefix).unwrap_or(value).into()
    } else {
        value.into()
    }
}

type LevelBadge = (With<unit_frame::Level>, Without<unit_frame::PlayerPortrait>);
type WolfPhoto = (
    With<unit_frame::WolfPortrait>,
    Without<unit_frame::Level>,
    Without<unit_frame::PlayerPortrait>,
);
fn paint_portrait(
    model: Res<Model>,
    mut badges: Query<(&Children, &mut Node), LevelBadge>,
    mut labels: Query<&mut Text>,
    mut photos: Query<(&mut Node, &mut ImageNode, &unit_frame::PlayerPortrait)>,
    images: Option<Res<Assets<Image>>>,
    mut wolves: Query<&mut Node, WolfPhoto>,
) {
    for mut node in &mut wolves {
        let display = if model.target_is_wolf {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (children, mut badge) in &mut badges {
        let display = if model.player_level.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if badge.display != display {
            badge.display = display;
        }
        if let Some(level) = model.player_level {
            let value = level.to_string();
            for child in children.iter() {
                if let Ok(mut label) = labels.get_mut(child)
                    && label.0 != value
                {
                    **label = value.clone();
                }
            }
        }
    }
    for (mut node, mut image, atlas) in &mut photos {
        let rect = model
            .player_appearance
            .filter(|_| {
                images
                    .as_ref()
                    .is_some_and(|images| images.contains(atlas.0.id()))
            })
            .map(crate::frontend::face_thumbnail);
        let display = if rect.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if image.rect != rect {
            image.rect = rect;
        }
    }
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
    Option<&'a TargetHealthLabel>,
);
type ButtonComponents<'a> = (
    Has<Pressed>,
    Option<&'a Hovered>,
    &'a mut BackgroundColor,
    &'a ActionButton,
);
fn paint(
    (layout, model): (Res<super::action_bar::Layout>, Res<Model>),
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
    for (mut label, action, cast, player_health, mana, life, heading, target_health) in &mut labels
    {
        let value = if let Some(action) = action {
            let Some(ability) = layout.ability(&config.0, action.0) else {
                let value = format!("[{}] Empty slot", key_label(action.0));
                if label.0 != value {
                    **label = value;
                }
                continue;
            };
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
            model
                .cast_info
                .as_ref()
                .filter(|_| skin.is_some())
                .map_or_else(
                    || model.status.clone(),
                    |cast| {
                        format!(
                            "{}      {:.1} / {:.1}s",
                            cast.name, cast.elapsed, cast.total
                        )
                    },
                )
        } else if target_health.is_some() {
            model.target_health_text.clone()
        } else if player_health.is_some() {
            pool_text(
                &model.player_health_text,
                "Health ",
                "Waiting for health",
                skin.is_some(),
            )
        } else if mana.is_some() {
            pool_text(
                &model.mana_text,
                "Mana ",
                "Waiting for mana",
                skin.is_some(),
            )
        } else if life.is_some() {
            model.life_status.clone()
        } else if heading.is_some() {
            model.player_name.as_deref().unwrap_or("Player").into()
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
pub(super) mod tests {
    use super::*;
    #[test]
    fn player_heading_updates_with_level_and_drops_unavailable_progression() {
        let mut app = widget_app(true);
        app.update();
        for (level, expected) in [(Some(1), Some("1")), (Some(2), Some("2")), (None, None)] {
            app.world_mut().resource_mut::<Model>().player_level = level;
            app.update();
            let world = app.world_mut();
            let (node, children) = world
                .query_filtered::<(&Node, &Children), With<unit_frame::Level>>()
                .single(world)
                .unwrap();
            let value = (node.display == Display::Flex)
                .then(|| world.get::<Text>(children[0]).unwrap().0.as_str());
            assert_eq!(value, expected);
            assert_eq!(world.resource::<Input>().0, None);
        }
    }
    #[test]
    fn player_heading_renders_selected_name_and_preserves_generic_fallback() {
        let mut app = widget_app(true);
        app.update();
        for (name, level, expected) in [
            (Some("Arden"), Some(2), "Arden"),
            (Some("Abcdefghijklmnopqrst"), None, "Abcdefghijklmnopqrst"),
            (None, None, "Player"),
        ] {
            let mut model = app.world_mut().resource_mut::<Model>();
            model.player_name = name.map(str::to_owned);
            model.player_level = level;
            app.update();
            let world = app.world_mut();
            assert_eq!(
                world
                    .query_filtered::<&Text, With<PlayerHeading>>()
                    .single(world)
                    .unwrap()
                    .0,
                expected
            );
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
        let catalog = theia_core::abilities::Catalog::from_json(include_bytes!(
            "../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
        widget_app_catalog(skin, catalog)
    }
    pub(in crate::combat) fn widget_app_catalog(
        skin: bool,
        catalog: theia_core::abilities::Catalog,
    ) -> App {
        let mut app = App::new();
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
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
            .add_systems(Startup, (super::super::action_bar::prepare, setup).chain())
            .add_systems(
                Update,
                (refresh_bar, collect, paint, paint_portrait).chain(),
            );
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
            assert_eq!(
                health.single(world).unwrap().0.0,
                if skin { "45 / 100" } else { "Health 45 / 100" }
            );
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
    fn rebound_empty_slot_rebuilds_widgets_resolves_tooltip_and_suppresses_editing_click_casts() {
        let mut app = widget_app(true);
        app.init_resource::<super::super::book::Ui>();
        app.update();
        let catalog = app.world().resource::<Config>().0.clone();
        let sword = catalog.abilities()[1].id().clone();
        assert!(
            app.world_mut()
                .resource_mut::<super::super::action_bar::Layout>()
                .copy(&catalog, &sword, 11)
        );
        {
            let mut model = app.world_mut().resource_mut::<Model>();
            model.connected = true;
            model.has_target = true;
            model.actions = (0..12)
                .map(|slot| Action {
                    cooldown: if slot == 11 { 0.75 } else { 0. },
                    status: "Cooldown 1.1s".into(),
                })
                .collect();
        }
        app.update();
        let world = app.world_mut();
        let text = world
            .query::<(&ActionLabel, &Text)>()
            .iter(world)
            .find(|(slot, _)| slot.0 == 11)
            .unwrap()
            .1;
        assert!(text.0.starts_with("[=] Sword Slash") && text.0.contains("3 m range"));
        let fill = world
            .query::<(&CooldownFill, &Node)>()
            .iter(world)
            .find(|(slot, _)| slot.0 == 11)
            .unwrap()
            .1;
        assert_eq!(fill.width, percent(75));
        let button = world
            .query::<(Entity, &ActionButton)>()
            .iter(world)
            .find(|(_, slot)| slot.0 == 11)
            .unwrap()
            .0;
        world.trigger(Activate { entity: button });
        app.update();
        assert_eq!(
            app.world().resource::<Input>().0,
            Some((11, Source::Widget))
        );
        app.world_mut()
            .resource_mut::<super::super::book::Ui>()
            .open = true;
        app.world_mut().trigger(Activate { entity: button });
        app.update();
        assert_eq!(app.world().resource::<Input>().0, None);
        app.world_mut()
            .resource_mut::<super::super::book::Ui>()
            .open = false;
        assert!(
            app.world_mut()
                .resource_mut::<super::super::action_bar::Layout>()
                .swap(11, 10)
        );
        app.update();
        assert_eq!(
            app.world().resource::<Model>().actions[11].cooldown,
            0.75,
            "Rebinding cannot rewrite received authoritative action state"
        );
        let world = app.world_mut();
        assert!(
            !world
                .query::<&ActionButton>()
                .iter(world)
                .any(|slot| slot.0 == 11)
        );
        assert!(
            world
                .query::<&ActionButton>()
                .iter(world)
                .any(|slot| slot.0 == 10)
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
        let frames = world
            .query::<(&ImageNode, &ChildOf)>()
            .iter(world)
            .filter(|(_, parent)| world.get::<ActionButton>(parent.parent()).is_some())
            .count();
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
