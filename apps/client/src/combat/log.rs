//! Small owner-bound history of received combat facts. No chat, damage prediction or commands.
use bevy::{picking::Pickable, prelude::*, text::FontSize};
use std::collections::VecDeque;
use theia_core::{
    damage::{melee_special, spell, white_melee},
    identity::EntityKey,
};
use theia_network::{
    client::State,
    simulation::combat::{Avoidance, Event, ability_damage::Outcome},
};
const CAPACITY: usize = 6;
#[derive(Resource, Default)]
pub(super) struct History {
    context: Option<([u8; 32], EntityKey)>,
    last_batch: u64,
    accepting: bool,
    lines: VecDeque<String>,
}
impl History {
    pub(super) fn begin(&mut self, state: &State) {
        let context = (|| {
            let welcome = state.welcome.as_ref()?;
            let snapshot = state.snapshot.as_ref()?;
            if state.error.is_some()
                || state.id != Some(welcome.id)
                || welcome.world == [0; 32]
                || snapshot.epoch != welcome.epoch
            {
                return None;
            }
            Some((
                welcome.world,
                EntityKey {
                    entity: theia_core::identity::EntityId::new(welcome.id)?,
                    epoch: theia_core::identity::WorldEpoch::new(welcome.epoch)?,
                },
            ))
        })();
        if self.context != context {
            *self = Self::default();
            self.context = context;
        }
        self.accepting = false;
    }
    pub(super) fn batch(&mut self, sequence: u64) {
        self.accepting = self.context.is_some() && sequence > self.last_batch;
        if self.accepting {
            self.last_batch = sequence;
        }
    }
    fn push(&mut self, text: String) {
        if self.lines.len() == CAPACITY {
            self.lines.pop_front();
        }
        self.lines.push_back(text);
    }
    pub(super) fn offer(&mut self, event: &Event) {
        if !self.accepting {
            return;
        }
        let Some((_, owner)) = self.context else {
            return;
        };
        let text = match event {
            Event::Damage {
                actor,
                target,
                amount,
                ..
            }
            | Event::MeleeDamage {
                actor,
                target,
                amount,
                ..
            } => damage(owner, *actor, *target, *amount, None),
            Event::PhysicalDamage(fact) => damage(
                owner,
                fact.actor,
                fact.target,
                fact.result.resolved_damage,
                None,
            )
            .map(|text| overkill(text, fact.result.overkill)),
            Event::WhiteMelee(fact) => ordinary(
                owner,
                fact.actor,
                fact.target,
                fact.result.resolved_damage,
                fact.result.outcome,
            )
            .map(|text| overkill(text, fact.result.overkill)),
            Event::AbilityDamage(fact) => {
                let outcome = match fact.result.outcome {
                    Outcome::MeleeSpecial(melee_special::Outcome::Miss) => Some("miss"),
                    Outcome::MeleeSpecial(melee_special::Outcome::Dodge) => Some("dodge"),
                    Outcome::MeleeSpecial(melee_special::Outcome::Parry) => Some("parry"),
                    Outcome::Arcane(spell::Outcome::Miss) => Some("resist"),
                    Outcome::MeleeSpecial(melee_special::Outcome::Critical)
                    | Outcome::Arcane(spell::Outcome::Critical) => Some("critical"),
                    _ => None,
                };
                result(
                    owner,
                    fact.actor,
                    fact.target,
                    fact.result.damage.resolved_damage,
                    outcome,
                )
                .map(|text| overkill(text, fact.result.damage.overkill))
            }
            Event::Avoided {
                actor,
                target,
                kind,
                ..
            } => result(
                owner,
                *actor,
                *target,
                0,
                Some(if *kind == Avoidance::Miss {
                    "miss"
                } else {
                    "deflect"
                }),
            ),
            Event::Cancelled { actor, reason, .. } if *actor == owner => {
                Some(format!("Action cancelled: {}", super::reason_text(*reason)))
            }
            _ => None,
        };
        if let Some(text) = text {
            self.push(text);
        }
    }
    fn text(&self) -> String {
        if self.lines.is_empty() {
            String::new()
        } else {
            format!(
                "Combat Log\n{}",
                self.lines.iter().cloned().collect::<Vec<_>>().join("\n")
            )
        }
    }
}
fn overkill(mut text: String, amount: u32) -> String {
    if amount > 0 {
        if text.ends_with('.') {
            text.pop();
        }
        text.push_str(&format!(" ({amount} overkill)."));
    }
    text
}
fn outgoing(owner: EntityKey, actor: EntityKey, target: EntityKey) -> Option<bool> {
    if actor.epoch != owner.epoch || target.epoch != owner.epoch || actor == target {
        return None;
    }
    if actor == owner {
        Some(true)
    } else if target == owner {
        Some(false)
    } else {
        None
    }
}
fn damage(
    owner: EntityKey,
    actor: EntityKey,
    target: EntityKey,
    amount: u32,
    kind: Option<&str>,
) -> Option<String> {
    let direction = if outgoing(owner, actor, target)? {
        "You deal"
    } else {
        "You take"
    };
    Some(format!(
        "{direction} {amount} damage{}.",
        kind.map_or(String::new(), |s| format!(" ({s})"))
    ))
}
fn result(
    owner: EntityKey,
    actor: EntityKey,
    target: EntityKey,
    amount: u32,
    kind: Option<&str>,
) -> Option<String> {
    let own = outgoing(owner, actor, target)?;
    let text = match (kind, own) {
        (Some("miss"), true) => "Your attack misses.",
        (Some("miss"), false) => "An attack misses you.",
        (Some("dodge"), true) => "Your attack is dodged.",
        (Some("dodge"), false) => "You dodge an attack.",
        (Some("parry"), true) => "Your attack is parried.",
        (Some("parry"), false) => "You parry an attack.",
        (Some("resist"), true) => "Your spell is resisted.",
        (Some("resist"), false) => "You resist a spell.",
        (Some("deflect"), true) => "Your attack is deflected.",
        (Some("deflect"), false) => "You deflect an attack.",
        _ => return damage(owner, actor, target, amount, kind),
    };
    Some(text.to_owned())
}
fn ordinary(
    owner: EntityKey,
    actor: EntityKey,
    target: EntityKey,
    amount: u32,
    outcome: white_melee::Outcome,
) -> Option<String> {
    use white_melee::Outcome::*;
    let kind = match outcome {
        Miss => Some("miss"),
        Dodge => Some("dodge"),
        Parry => Some("parry"),
        Block => Some("blocked"),
        Glancing => Some("glancing"),
        Critical => Some("critical"),
        Crushing => Some("crushing"),
        Hit => None,
    };
    result(owner, actor, target, amount, kind)
}
#[derive(Component)]
struct Panel;
#[derive(Component)]
struct Lines;
#[derive(Component)]
struct Scrollback {
    following: bool,
}
#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
enum Tab {
    Chat,
    #[default]
    Combat,
}
#[derive(Resource, Default)]
struct ActiveTab(Tab);
pub(crate) fn install(app: &mut App) {
    if app.world().contains_resource::<ActiveTab>() {
        return;
    }
    app.add_message::<bevy::input::mouse::MouseWheel>()
        .init_resource::<History>()
        .init_resource::<ActiveTab>()
        .add_observer(select_tab)
        .add_systems(Startup, setup)
        .add_systems(Update, (paint.after(super::Presentation), scroll))
        .add_systems(Last, follow_tail);
}
fn select_tab(
    event: On<bevy::ui_widgets::Activate>,
    tabs: Query<&Tab>,
    mut active: ResMut<ActiveTab>,
) {
    if let Ok(tab) = tabs.get(event.entity) {
        active.0 = *tab;
    }
}
fn setup(
    mut commands: Commands,
    skin: Option<Res<crate::fantasy_ui::Skin>>,
    chat: Option<Res<crate::chat::Ui>>,
) {
    if skin.is_none() {
        return;
    }
    commands
        .spawn((
            Panel,
            crate::ui_theme::Panel,
            crate::ui_theme::fade::Pane::Chat,
            Node {
                position_type: PositionType::Absolute,
                left: px(14),
                bottom: px(104),
                width: px(380),
                max_width: percent(32),
                height: px(270),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                display: Display::None,
                ..default()
            },
            BackgroundColor(crate::ui_theme::PANEL),
            GlobalZIndex(10),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        column_gap: px(3),
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: px(3),
                        border: UiRect::bottom(px(1)),
                        padding: UiRect::bottom(px(6)),
                        ..default()
                    },
                    BorderColor::all(crate::ui_theme::GOLD),
                ))
                .with_children(|row| {
                    for (tab, title) in [(Tab::Chat, "General"), (Tab::Combat, "Combat Log")] {
                        row.spawn((
                            tab,
                            bevy::ui_widgets::Button,
                            bevy::picking::hover::Hovered::default(),
                            crate::ui_theme::Style::Secondary,
                            AccessibleLabel::new(title),
                            Node {
                                padding: UiRect::axes(px(8), px(5)),
                                ..default()
                            },
                            BackgroundColor(crate::fantasy_ui::PANEL),
                        ))
                        .with_child((
                            Text::new(title),
                            TextFont {
                                font_size: FontSize::Px(16.),
                                ..default()
                            },
                            TextColor(crate::ui_theme::CREAM),
                            Pickable::IGNORE,
                        ));
                    }
                    for title in ["Trade", "Guild", "Party"] {
                        let id = crate::fantasy_ui::kit::disabled(row, title);
                        row.commands().entity(id).insert(Node {
                            padding: UiRect::axes(px(6), px(5)),
                            min_height: px(28),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        });
                    }
                });
            panel
                .spawn((
                    Scrollback { following: true },
                    ScrollPosition::default(),
                    bevy::picking::hover::Hovered::default(),
                    Node {
                        flex_grow: 1.,
                        min_height: px(0),
                        width: percent(100),
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            y: OverflowAxis::Scroll,
                        },
                        ..default()
                    },
                ))
                .with_child((
                    Lines,
                    Pickable::IGNORE,
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                    TextColor(crate::ui_theme::CREAM),
                    TextLayout::new(Justify::Left, bevy::text::LineBreak::WordOrCharacter),
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
            panel
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|r| {
                    if chat.is_some() {
                        crate::chat::row(r);
                        return;
                    }
                    crate::fantasy_ui::kit::icon(r, 23, true);
                    r.spawn((
                        bevy::ui_widgets::TextInput,
                        bevy::text::EditableText::default(),
                        bevy::ui::InteractionDisabled,
                        AccessibleLabel::new("Chat message — unavailable"),
                        Node {
                            width: Val::Auto,
                            flex_grow: 1.,
                            ..crate::ui_theme::input_box(false, false)
                        },
                        BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                        BackgroundColor(crate::ui_theme::PANEL),
                    ))
                    .with_child((
                        Text::new("Type a message…"),
                        TextFont::from_font_size(14.),
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                        Pickable::IGNORE,
                    ));
                });
        });
}

fn scroll(
    mut events: MessageReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
    mut panes: Query<(
        &bevy::picking::hover::Hovered,
        &ComputedNode,
        &mut ScrollPosition,
        &mut Scrollback,
    )>,
) {
    let delta: f32 = events
        .read()
        .map(|e| {
            -e.y * if e.unit == bevy::input::mouse::MouseScrollUnit::Line {
                24.
            } else {
                1.
            }
        })
        .sum();
    if delta == 0. || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    for (hover, node, mut position, mut state) in &mut panes {
        if !hover.0 {
            continue;
        }
        let maximum =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        position.y = (position.y + delta).clamp(0., maximum);
        state.following = position.y >= maximum - 1.;
    }
}
fn follow_tail(mut panes: Query<(&ComputedNode, &mut ScrollPosition, &Scrollback)>) {
    for (node, mut position, state) in &mut panes {
        if state.following {
            position.y =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn paint(
    mut commands: Commands,
    history: Res<History>,
    mut active: ResMut<ActiveTab>,
    chat: Option<Res<crate::chat::Ui>>,
    network: Option<Res<crate::network::Network>>,
    tabs: Query<(Entity, &Tab)>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut labels: Query<&mut Text, With<Lines>>,
) {
    if chat.as_ref().is_some_and(|c| c.active()) {
        active.0 = Tab::Chat;
    }
    let state = (active.0 == Tab::Chat)
        .then(|| network.as_ref().map(|n| n.state()))
        .flatten();
    let value = match active.0 {
        Tab::Combat => {
            let value = history.text();
            if value.is_empty() {
                "No combat events yet.".to_owned()
            } else {
                value
                    .strip_prefix("Combat Log\n")
                    .unwrap_or(&value)
                    .to_owned()
            }
        }
        Tab::Chat => state.as_ref().and_then(|n| n.chat.as_ref()).map_or_else(
            || "Chat is not enabled in this playtest.".to_owned(),
            |view| {
                let lines: Vec<_> = view
                    .inbox
                    .history()
                    .map(|d| format!("[{:?}] {}: {}", d.channel, d.name.as_str(), d.text.as_str()))
                    .collect();
                if lines.is_empty() {
                    "No messages yet.".to_owned()
                } else {
                    lines.join("\n")
                }
            },
        ),
    };
    for (entity, tab) in &tabs {
        if *tab == active.0 {
            commands.entity(entity).insert(crate::ui_theme::Selected);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui_theme::Selected>();
        }
    }
    for mut node in &mut panels {
        let display = if history.context.is_some()
            || network
                .as_ref()
                .is_some_and(|n| !n.loading_blocked && n.connected() && n.state().chat.is_some())
        {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for mut text in &mut labels {
        if text.0 != value {
            text.0 = value.clone();
        }
    }
}

/// Opt-in layout evidence only; no message bodies or additional history copy.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let viewport = world
        .query_filtered::<(&Node, &ComputedNode, &ScrollPosition, &Scrollback), With<Scrollback>>()
        .iter(world)
        .next()
        .map(|(node, computed, position, state)| {
            serde_json::json!({
                "size": computed.size().to_array(),
                "clip_x": node.overflow.x == OverflowAxis::Clip,
                "following": state.following,
                "scroll_y": position.y,
            })
        });
    let text = world
        .query_filtered::<(&ComputedNode, &bevy::text::TextLayoutInfo), With<Lines>>()
        .iter(world)
        .next()
        .map(|(node, layout)| {
            serde_json::json!({
                "node_size": node.size().to_array(),
                "layout_size": layout.size.to_array(),
                "glyph_count": layout.glyphs.len(),
            })
        });
    serde_json::json!({"viewport": viewport, "text": text})
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{content::ContentId, coordinates::GlobalPosition, identity::EntityId};
    fn fixture() -> (State, EntityKey, EntityKey) {
        let (state, _) = crate::quest_journal::tests::fixture();
        let owner = state.quest_journal.as_ref().unwrap().actor;
        (
            state,
            owner,
            EntityKey {
                entity: EntityId::new(19).unwrap(),
                ..owner
            },
        )
    }
    fn hit(actor: EntityKey, target: EntityKey, amount: u32) -> Event {
        Event::Damage {
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            actor,
            target,
            sequence: 1,
            position: GlobalPosition::new([0.; 3]).unwrap(),
            amount,
            remaining: 100,
        }
    }
    #[test]
    fn bounded_received_batches_do_not_replay_or_claim_other_actors_damage() {
        let (state, owner, target) = fixture();
        let mut history = History::default();
        history.begin(&state);
        for i in 1..=10 {
            history.batch(i);
            history.offer(&hit(owner, target, i as u32));
        }
        assert_eq!(history.lines.len(), CAPACITY);
        assert_eq!(history.lines.front().unwrap(), "You deal 5 damage.");
        let text = history.text();
        history.batch(10);
        history.offer(&hit(owner, target, 999));
        assert_eq!(history.text(), text);
        history.batch(9);
        history.offer(&hit(owner, target, 999));
        assert_eq!(history.text(), text);
        history.batch(11);
        let other = EntityKey {
            entity: EntityId::new(20).unwrap(),
            ..owner
        };
        history.offer(&hit(other, target, 999));
        assert_eq!(history.text(), text);
        history.offer(&hit(target, owner, 7));
        assert_eq!(history.lines.back().unwrap(), "You take 7 damage.");
    }
    #[test]
    fn disconnect_owner_epoch_and_world_changes_clear_history() {
        for case in 0..4 {
            let (mut state, owner, target) = fixture();
            let mut history = History::default();
            history.begin(&state);
            history.batch(1);
            history.offer(&hit(owner, target, 5));
            assert!(!history.lines.is_empty());
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.id = Some(99),
                2 => state.welcome.as_mut().unwrap().epoch += 1,
                _ => state.welcome.as_mut().unwrap().world = [2; 32],
            }
            history.begin(&state);
            assert!(history.lines.is_empty());
            history.offer(&hit(owner, target, 5));
            assert!(history.lines.is_empty(), "No open accepted batch");
        }
    }
    #[test]
    fn outcomes_are_descriptions_of_supplied_facts_not_new_damage_rolls() {
        let (_, owner, target) = fixture();
        assert_eq!(
            ordinary(owner, owner, target, 0, white_melee::Outcome::Parry).unwrap(),
            "Your attack is parried."
        );
        assert_eq!(
            ordinary(owner, target, owner, 0, white_melee::Outcome::Dodge).unwrap(),
            "You dodge an attack."
        );
        assert_eq!(
            ordinary(owner, owner, target, 17, white_melee::Outcome::Critical).unwrap(),
            "You deal 17 damage (critical)."
        );
        assert_eq!(
            ordinary(owner, target, owner, 3, white_melee::Outcome::Block).unwrap(),
            "You take 3 damage (blocked)."
        );
        assert!(damage(owner, target, target, 1, None).is_none());
    }
    #[test]
    fn resolved_physical_facts_and_owner_cancellations_use_received_values() {
        let (state, owner, target) = fixture();
        let mut history = History::default();
        history.begin(&state);
        history.batch(1);
        let resolved = theia_core::damage::physical::normal_reference_v1(
            theia_core::damage::Amount::fixed(100).unwrap(),
            1,
            485,
            0,
            17,
        )
        .unwrap();
        assert_eq!(
            (
                resolved.starting_damage,
                resolved.resolved_damage,
                resolved.health_lost
            ),
            (100, 50, 17)
        );
        let fact = theia_network::simulation::combat::physical::Fact {
            attack: ContentId::new("theia.dev.training-swing").unwrap(),
            actor: owner,
            actor_generation: std::num::NonZeroU64::new(1),
            sequence: 1,
            target,
            target_generation: None,
            position: GlobalPosition::new([0.; 3]).unwrap(),
            roll: 100,
            result: resolved,
            remaining: 0,
        };
        history.offer(&Event::PhysicalDamage(fact));
        assert_eq!(
            history.lines.back().unwrap(),
            "You deal 50 damage (33 overkill).",
            "Log reports resolved damage, not starting damage or health lost"
        );
        history.offer(&Event::Cancelled {
            actor: owner,
            sequence: 2,
            reason: theia_network::simulation::combat::Reason::NotFacing,
        });
        assert!(
            history
                .lines
                .back()
                .unwrap()
                .starts_with("Action cancelled: ")
        );
        let prior = history.text();
        history.offer(&Event::Cancelled {
            actor: target,
            sequence: 2,
            reason: theia_network::simulation::combat::Reason::NotFacing,
        });
        assert_eq!(history.text(), prior);
    }
    #[test]
    fn shared_widget_updates_and_clears_without_network_or_input() {
        let (state, owner, target) = fixture();
        let mut app = App::new();
        app.insert_resource(crate::fantasy_ui::Skin::fixture());
        install(&mut app);
        app.update();
        {
            let mut history = app.world_mut().resource_mut::<History>();
            history.begin(&state);
            history.batch(1);
            history.offer(&hit(owner, target, 12));
        }
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Text, With<Lines>>()
                .single(world)
                .unwrap()
                .0
                .contains("You deal 12 damage.")
        );
        let chat = world
            .query::<(Entity, &Tab)>()
            .iter(world)
            .find(|(_, tab)| **tab == Tab::Chat)
            .unwrap()
            .0;
        world.trigger(bevy::ui_widgets::Activate { entity: chat });
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Text, With<Lines>>()
                .single(world)
                .unwrap()
                .contains("Chat is not enabled")
        );
        assert_eq!(
            world.resource::<History>().lines.back().unwrap(),
            "You deal 12 damage."
        );
        let combat = world
            .query::<(Entity, &Tab)>()
            .iter(world)
            .find(|(_, tab)| **tab == Tab::Combat)
            .unwrap()
            .0;
        world.trigger(bevy::ui_widgets::Activate { entity: combat });
        app.update();
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Text, With<Lines>>()
                .single(world)
                .unwrap()
                .contains("You deal 12 damage.")
        );
        world.resource_mut::<History>().begin(&State::default());
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<&Node, With<Panel>>()
                .single(world)
                .unwrap()
                .display,
            Display::None
        );
    }
}
