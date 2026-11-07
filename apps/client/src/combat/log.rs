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
pub(super) fn install(app: &mut App) {
    app.init_resource::<History>()
        .add_systems(Startup, setup)
        .add_systems(Update, paint.after(super::Presentation));
}
fn setup(mut commands: Commands, skin: Option<Res<crate::fantasy_ui::Skin>>) {
    if skin.is_none() {
        return;
    }
    commands
        .spawn((
            Panel,
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(94),
                width: px(278),
                max_height: px(240),
                padding: UiRect::all(px(6)),
                overflow: Overflow::clip(),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.02, 0.35)),
            GlobalZIndex(10),
        ))
        .with_child((
            Lines,
            Pickable::IGNORE,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(14.),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.8, 0.65)),
        ));
}
fn paint(
    history: Res<History>,
    mut panels: Query<&mut Node, With<Panel>>,
    mut labels: Query<&mut Text, With<Lines>>,
) {
    let value = history.text();
    for mut node in &mut panels {
        let display = if value.is_empty() {
            Display::None
        } else {
            Display::Flex
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
