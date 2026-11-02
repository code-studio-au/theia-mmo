//! Bounded cosmetic miss/deflect labels from authoritative events. Standard
//! Bevy UI and central origin conversion; never invents a combat result.
use bevy::{prelude::*, text::FontSize};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use theia_core::damage::{
    melee_special::Outcome as SpecialOutcome, spell::Outcome as MagicOutcome,
};
use theia_core::{coordinates::GlobalPosition, damage::white_melee::Outcome as WhiteOutcome};
use theia_network::simulation::combat::{
    Avoidance, Event, ability_damage::Outcome as AbilityOutcome,
};
const MAX_PENDING: usize = 32;
const MAX_ACTIVE: usize = 32;
const LIFETIME: Duration = Duration::from_secs(1);
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Observed;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Miss,
    Deflect,
    Melee(u32),
    Incoming(u32),
    Ability {
        outcome: AbilityOutcome,
        amount: u32,
    },
    Ordinary {
        outcome: WhiteOutcome,
        amount: u32,
        incoming: bool,
    },
}
impl Kind {
    fn melee(self) -> bool {
        matches!(
            self,
            Self::Melee(_)
                | Self::Ordinary {
                    incoming: false,
                    amount: 1..,
                    ..
                }
        )
    }
    fn incoming(self) -> bool {
        matches!(
            self,
            Self::Incoming(_)
                | Self::Ordinary {
                    incoming: true,
                    amount: 1..,
                    ..
                }
        )
    }
    fn critical(self) -> bool {
        matches!(
            self,
            Self::Ordinary {
                outcome: WhiteOutcome::Critical,
                ..
            } | Self::Ability {
                outcome: AbilityOutcome::MeleeSpecial(SpecialOutcome::Critical)
                    | AbilityOutcome::Arcane(MagicOutcome::Critical),
                ..
            }
        )
    }
    fn ability(self) -> Option<AbilityOutcome> {
        if let Self::Ability { outcome, .. } = self {
            Some(outcome)
        } else {
            None
        }
    }
    fn ordinary(self) -> Option<WhiteOutcome> {
        if let Self::Ordinary { outcome, .. } = self {
            Some(outcome)
        } else {
            None
        }
    }
    fn text(self) -> String {
        match self {
            Self::Miss => "Miss".into(),
            Self::Deflect => "Deflected".into(),
            Self::Melee(n) | Self::Incoming(n) => n.to_string(),
            Self::Ability { outcome, amount } => match outcome {
                AbilityOutcome::MeleeSpecial(SpecialOutcome::Miss) => "Miss".into(),
                AbilityOutcome::MeleeSpecial(SpecialOutcome::Dodge) => "Dodged".into(),
                AbilityOutcome::MeleeSpecial(SpecialOutcome::Parry) => "Parried".into(),
                AbilityOutcome::Arcane(MagicOutcome::Miss) => "Resisted".into(),
                _ => amount.to_string(),
            },
            Self::Ordinary {
                outcome, amount, ..
            } => match outcome {
                WhiteOutcome::Miss => "Miss".into(),
                WhiteOutcome::Dodge => "Dodged".into(),
                WhiteOutcome::Parry => "Parried".into(),
                WhiteOutcome::Block if amount == 0 => "Blocked".into(),
                WhiteOutcome::Block => format!("{amount} (Block)"),
                WhiteOutcome::Glancing => format!("{amount} (Glancing)"),
                WhiteOutcome::Crushing => format!("{amount} (Crushing)"),
                WhiteOutcome::Critical | WhiteOutcome::Hit => amount.to_string(),
            },
        }
    }
    fn color(self) -> Color {
        if self.incoming() {
            Color::srgb(1., 0.25, 0.2)
        } else if matches!(
            self,
            Self::Ability {
                outcome: AbilityOutcome::MeleeSpecial(
                    SpecialOutcome::Miss | SpecialOutcome::Dodge | SpecialOutcome::Parry
                ) | AbilityOutcome::Arcane(MagicOutcome::Miss),
                ..
            }
        ) || self == Self::Miss
            || matches!(
                self,
                Self::Ordinary {
                    outcome: WhiteOutcome::Miss | WhiteOutcome::Dodge | WhiteOutcome::Parry,
                    ..
                } | Self::Ordinary {
                    outcome: WhiteOutcome::Block,
                    amount: 0,
                    ..
                }
            )
        {
            Color::srgb(1., 0.88, 0.3)
        } else if matches!(
            self,
            Self::Ability {
                outcome: AbilityOutcome::Arcane(_),
                ..
            }
        ) {
            Color::srgb(0.75, 0.6, 1.)
        } else if self == Self::Deflect {
            Color::srgb(0.5, 0.85, 1.)
        } else {
            Color::WHITE
        }
    }
}
#[derive(Default, serde::Serialize)]
pub(crate) struct OutcomeCounts {
    pub misses: u64,
    pub dodges: u64,
    pub parries: u64,
    pub blocks: u64,
    pub glancing: u64,
    pub criticals: u64,
    pub crushing: u64,
    pub hits: u64,
}
impl OutcomeCounts {
    pub(crate) fn record(&mut self, outcome: WhiteOutcome) {
        let count = match outcome {
            WhiteOutcome::Miss => &mut self.misses,
            WhiteOutcome::Dodge => &mut self.dodges,
            WhiteOutcome::Parry => &mut self.parries,
            WhiteOutcome::Block => &mut self.blocks,
            WhiteOutcome::Glancing => &mut self.glancing,
            WhiteOutcome::Critical => &mut self.criticals,
            WhiteOutcome::Crushing => &mut self.crushing,
            WhiteOutcome::Hit => &mut self.hits,
        };
        *count = count.saturating_add(1);
    }
}
#[derive(Default, serde::Serialize)]
pub(crate) struct AbilityCounts {
    pub misses: u64,
    pub dodges: u64,
    pub parries: u64,
    pub hits: u64,
    pub criticals: u64,
}
impl AbilityCounts {
    fn record(&mut self, outcome: AbilityOutcome) {
        let count = match outcome {
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Miss)
            | AbilityOutcome::Arcane(MagicOutcome::Miss) => &mut self.misses,
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Dodge) => &mut self.dodges,
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Parry) => &mut self.parries,
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Hit)
            | AbilityOutcome::Arcane(MagicOutcome::Hit) => &mut self.hits,
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Critical)
            | AbilityOutcome::Arcane(MagicOutcome::Critical) => &mut self.criticals,
        };
        *count = count.saturating_add(1);
    }
}
#[derive(Default, serde::Serialize)]
pub(crate) struct AbilityOutcomes {
    pub special: AbilityCounts,
    pub arcane: AbilityCounts,
}
impl AbilityOutcomes {
    pub(crate) fn record(&mut self, outcome: AbilityOutcome) {
        match outcome {
            AbilityOutcome::MeleeSpecial(_) => self.special.record(outcome),
            AbilityOutcome::Arcane(_) => self.arcane.record(outcome),
        }
    }
}
struct Cue {
    position: GlobalPosition,
    kind: Kind,
}
#[derive(Resource, Default)]
pub(crate) struct Pending {
    queue: VecDeque<Cue>,
    pub metrics: Metrics,
}
#[derive(Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub ability_offered: AbilityOutcomes,
    pub ability_spawned: AbilityOutcomes,
    pub ability_visible: AbilityOutcomes,
    pub ordinary_offered: OutcomeCounts,
    pub ordinary_spawned: OutcomeCounts,
    pub ordinary_visible: OutcomeCounts,
    pub offered_incoming: u64,
    pub spawned_incoming: u64,
    pub visible_incoming_frames: u64,
    pub offered_melee: u64,
    pub spawned_melee: u64,
    pub visible_melee_frames: u64,
    pub offered_misses: u64,
    pub offered_deflects: u64,
    pub spawned_misses: u64,
    pub spawned_deflects: u64,
    pub visible_miss_frames: u64,
    pub visible_deflect_frames: u64,
    pub dropped: u64,
    pub max_active: usize,
    pub visible_now: usize,
    pub renderable_streak: u32,
    pub projected_frames: u64,
    pub layout_samples: u64,
    pub hidden_layout_frames: u64,
    pub empty_glyph_frames: u64,
    pub last_size: [f32; 2],
    pub last_center: [f32; 2],
}
impl Pending {
    pub(crate) fn offer(&mut self, event: &Event) {
        let (position, kind) = match event {
            Event::AbilityDamage(fact) => (
                &fact.position,
                Kind::Ability {
                    outcome: fact.result.outcome,
                    amount: fact.result.damage.resolved_damage,
                },
            ),
            Event::WhiteMelee(fact) => (
                &fact.position,
                Kind::Ordinary {
                    outcome: fact.result.outcome,
                    amount: fact.result.resolved_damage,
                    incoming: fact.target_generation.is_some(),
                },
            ),
            Event::PhysicalDamage(fact) => (
                &fact.position,
                if fact.target_generation.is_some() {
                    Kind::Incoming(fact.result.resolved_damage)
                } else {
                    Kind::Melee(fact.result.resolved_damage)
                },
            ),
            Event::Avoided { position, kind, .. } => (
                position,
                if *kind == Avoidance::Miss {
                    Kind::Miss
                } else {
                    Kind::Deflect
                },
            ),
            Event::MeleeDamage {
                position, amount, ..
            } => (position, Kind::Melee(*amount)),
            _ => return,
        };
        self.metrics.offered_misses += u64::from(kind == Kind::Miss);
        self.metrics.offered_deflects += u64::from(kind == Kind::Deflect);
        self.metrics.offered_incoming += u64::from(kind.incoming());
        self.metrics.offered_melee += u64::from(kind.melee());
        if let Some(outcome) = kind.ability() {
            self.metrics.ability_offered.record(outcome);
        }
        if let Some(outcome) = kind.ordinary() {
            self.metrics.ordinary_offered.record(outcome);
        }
        if self.queue.len() == MAX_PENDING {
            self.queue.pop_front();
            self.metrics.dropped += 1;
        }
        self.queue.push_back(Cue {
            position: *position,
            kind,
        });
    }
}
#[derive(Component)]
struct Label {
    position: GlobalPosition,
    kind: Kind,
    born: Instant,
}
pub(crate) fn install(app: &mut App) {
    app.init_resource::<Pending>()
        .add_systems(Update, spawn.after(crate::combat::Presentation))
        .add_systems(
            PostUpdate,
            paint
                .after(crate::camera::sync_camera_global)
                .before(bevy::ui::UiSystems::Layout),
        )
        .add_systems(Last, observe.in_set(Observed));
}
fn spawn(mut commands: Commands, mut pending: ResMut<Pending>, labels: Query<&Label>) {
    let mut active = labels.iter().count();
    while let Some(cue) = pending.queue.pop_front() {
        if active >= MAX_ACTIVE {
            pending.metrics.dropped += 1;
            continue;
        }
        let miss = cue.kind == Kind::Miss;
        commands.spawn((
            Label {
                position: cue.position,
                kind: cue.kind,
                born: Instant::now(),
            },
            Text::new(cue.kind.text()),
            TextFont {
                font_size: FontSize::Px(if cue.kind.critical() { 28. } else { 20. }),
                ..default()
            },
            TextColor(cue.kind.color()),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Visibility::Hidden,
            bevy::picking::Pickable::IGNORE,
        ));
        active += 1;
        pending.metrics.spawned_misses += u64::from(miss);
        pending.metrics.spawned_deflects += u64::from(cue.kind == Kind::Deflect);
        pending.metrics.spawned_incoming += u64::from(cue.kind.incoming());
        pending.metrics.spawned_melee += u64::from(cue.kind.melee());
        if let Some(outcome) = cue.kind.ability() {
            pending.metrics.ability_spawned.record(outcome);
        }
        if let Some(outcome) = cue.kind.ordinary() {
            pending.metrics.ordinary_spawned.record(outcome);
        }
    }
    pending.metrics.max_active = pending.metrics.max_active.max(active);
}
fn paint(
    mut commands: Commands,
    mut labels: Query<(Entity, &Label, &mut Node, &mut TextColor, &mut Visibility)>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    frame: Res<crate::floating_origin::Frame>,
    mut pending: ResMut<Pending>,
) {
    let camera = cameras.single().ok();
    for (entity, label, mut node, mut color, mut visibility) in &mut labels {
        let age = label.born.elapsed();
        if age >= LIFETIME {
            commands.entity(entity).despawn();
            continue;
        }
        let mut global = label.position.metres();
        global[1] += 1.5;
        let point = GlobalPosition::new(global).and_then(|p| frame.render.to_local(p));
        let projection = camera.and_then(|(camera, transform)| {
            let point = point?;
            let position = camera
                .world_to_viewport(transform, Vec3::from_array(point.metres()))
                .ok()?;
            let size = camera.logical_viewport_size()?;
            (position.x >= 0. && position.y >= 0. && position.x <= size.x && position.y <= size.y)
                .then_some(position)
        });
        if let Some(position) = projection {
            pending.metrics.projected_frames += 1;
            node.left = px(position.x - 30.);
            node.top = px(position.y - age.as_secs_f32() * 28.);
            *visibility = Visibility::Inherited;
            color.0 = color.0.with_alpha((1. - age.as_secs_f32()).clamp(0., 1.));
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
fn observe(
    labels: Query<(
        &Label,
        &ComputedNode,
        &InheritedVisibility,
        &bevy::ui::UiGlobalTransform,
        Option<&bevy::text::TextLayoutInfo>,
    )>,
    mut pending: ResMut<Pending>,
) {
    pending.metrics.visible_now = 0;
    for (label, node, visible, transform, glyphs) in &labels {
        pending.metrics.layout_samples += 1;
        pending.metrics.last_size = node.size().to_array();
        pending.metrics.last_center =
            (transform.affine().translation * node.inverse_scale_factor()).to_array();
        pending.metrics.hidden_layout_frames += u64::from(!visible.get());
        let glyphs = glyphs.is_some_and(|info| !info.glyphs.is_empty());
        pending.metrics.empty_glyph_frames += u64::from(!glyphs);
        if visible.get() && node.size().min_element() > 0. && glyphs {
            pending.metrics.visible_now += 1;
            pending.metrics.visible_incoming_frames += u64::from(label.kind.incoming());
            pending.metrics.visible_melee_frames += u64::from(label.kind.melee());
            if let Some(outcome) = label.kind.ability() {
                pending.metrics.ability_visible.record(outcome);
            }
            if let Some(outcome) = label.kind.ordinary() {
                pending.metrics.ordinary_visible.record(outcome);
            }
            pending.metrics.visible_miss_frames += u64::from(label.kind == Kind::Miss);
            pending.metrics.visible_deflect_frames += u64::from(label.kind == Kind::Deflect);
        }
    }
    pending.metrics.renderable_streak = if pending.metrics.visible_now > 0 {
        pending.metrics.renderable_streak.saturating_add(1)
    } else {
        0
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        content::ContentId,
        identity::{EntityId, EntityKey, WorldEpoch},
    };
    #[test]
    fn avoidance_labels_keep_the_visual_queue_and_entity_count_bounded() {
        let key = EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(2).unwrap(),
        };
        let mut pending = Pending::default();
        for i in 0..100 {
            pending.offer(&Event::Avoided {
                actor: key,
                sequence: i + 1,
                target: key,
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                position: GlobalPosition::new([15040., 25., 15040.]).unwrap(),
                kind: if i % 2 == 0 {
                    Avoidance::Miss
                } else {
                    Avoidance::Deflect
                },
            });
        }
        assert_eq!(pending.queue.len(), MAX_PENDING);
        assert_eq!(pending.metrics.dropped, 68);
        assert_eq!(pending.metrics.offered_misses, 50);
        assert_eq!(pending.metrics.offered_deflects, 50);
        let mut app = App::new();
        app.insert_resource(pending).add_systems(Update, spawn);
        app.update();
        assert_eq!(
            app.world_mut().query::<&Label>().iter(app.world()).count(),
            MAX_ACTIVE
        );
        assert_eq!(
            app.world().resource::<Pending>().metrics.max_active,
            MAX_ACTIVE
        );
        assert!(app.world().resource::<Pending>().queue.is_empty());
    }
    #[test]
    fn physical_labels_show_resolved_damage_and_separate_incoming_from_overkill() {
        let key = EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(2).unwrap(),
        };
        let result = theia_core::damage::physical::normal_reference_v1(
            theia_core::damage::Amount::fixed(15).unwrap(),
            1,
            100,
            0,
            2,
        )
        .unwrap();
        assert_eq!(
            (result.resolved_damage, result.health_lost, result.overkill),
            (12, 2, 10)
        );
        let mut fact = theia_network::simulation::combat::physical::Fact {
            attack: ContentId::new("theia.dev.training-sword-swing").unwrap(),
            actor: key,
            actor_generation: std::num::NonZeroU64::new(1),
            sequence: 1,
            target: key,
            target_generation: None,
            position: GlobalPosition::new([0., 0., 0.]).unwrap(),
            roll: 11,
            result,
            remaining: 0,
        };
        let mut pending = Pending::default();
        pending.offer(&Event::PhysicalDamage(fact.clone()));
        fact.actor_generation = None;
        fact.target_generation = std::num::NonZeroU64::new(1);
        pending.offer(&Event::PhysicalDamage(fact));
        assert!(matches!(pending.queue[0].kind, Kind::Melee(12)));
        assert!(matches!(pending.queue[1].kind, Kind::Incoming(12)));
        let mut app = App::new();
        app.insert_resource(pending).add_systems(Update, spawn);
        app.update();
        let metrics = &app.world().resource::<Pending>().metrics;
        assert_eq!((metrics.spawned_melee, metrics.spawned_incoming), (1, 1));
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .all(|text| text.0 == "12")
        );
    }
    #[test]
    fn ordinary_labels_distinguish_outcomes_and_show_full_critical_damage_with_incoming_color() {
        let mut pending = Pending::default();
        for incoming in [false, true] {
            for outcome in [
                WhiteOutcome::Miss,
                WhiteOutcome::Dodge,
                WhiteOutcome::Parry,
                WhiteOutcome::Block,
                WhiteOutcome::Glancing,
                WhiteOutcome::Critical,
                WhiteOutcome::Crushing,
                WhiteOutcome::Hit,
            ] {
                pending.offer(&Event::WhiteMelee(crate::combat::test_facts::fact(
                    outcome, incoming, 3,
                )));
            }
        }
        let mut blocked = crate::combat::test_facts::fact(WhiteOutcome::Block, false, 100);
        blocked.result = theia_core::damage::white_melee::impact::physical_reference_v1(
            blocked.result.starting_exact,
            1,
            100,
            0,
            100,
            theia_core::damage::white_melee::impact::Impact::Block { value: 1000 },
        )
        .unwrap();
        blocked.remaining = 100;
        pending.offer(&Event::WhiteMelee(blocked));
        let mut app = App::new();
        app.insert_resource(pending).add_systems(Update, spawn);
        app.update();
        let metrics = &app.world().resource::<Pending>().metrics;
        assert_eq!(metrics.ordinary_spawned.criticals, 2);
        assert_eq!(metrics.ordinary_spawned.blocks, 3);
        assert_eq!((metrics.spawned_melee, metrics.spawned_incoming), (5, 5));
        let mut labels = app
            .world_mut()
            .query::<(&Label, &Text, &TextFont, &TextColor)>();
        let mut texts = Vec::new();
        let mut crits = 0;
        for (label, text, font, color) in labels.iter(app.world()) {
            texts.push(text.0.clone());
            if matches!(
                label.kind,
                Kind::Ordinary {
                    outcome: WhiteOutcome::Critical,
                    ..
                }
            ) {
                crits += 1;
                assert_eq!(font.font_size, FontSize::Px(28.));
                if label.kind.incoming() {
                    assert_eq!(text.0, "8");
                    assert_eq!(color.0, Color::srgb(1., 0.25, 0.2));
                } else {
                    assert_eq!(text.0, "24");
                    assert_eq!(color.0, Color::WHITE);
                }
            }
        }
        assert_eq!(crits, 2);
        for expected in [
            "Miss",
            "Dodged",
            "Parried",
            "Blocked",
            "9 (Block)",
            "7 (Glancing)",
            "18 (Crushing)",
        ] {
            assert!(
                texts.iter().any(|text| text == expected),
                "Missing {expected}"
            );
        }
    }
    #[test]
    fn typed_ability_labels_separate_magic_special_avoidance_and_full_critical_overkill() {
        let mut pending = Pending::default();
        for outcome in [
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Hit),
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Critical),
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Miss),
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Dodge),
            AbilityOutcome::MeleeSpecial(SpecialOutcome::Parry),
            AbilityOutcome::Arcane(MagicOutcome::Hit),
            AbilityOutcome::Arcane(MagicOutcome::Critical),
            AbilityOutcome::Arcane(MagicOutcome::Miss),
        ] {
            pending.offer(&Event::AbilityDamage(
                crate::combat::test_facts::ability_fact(outcome, 3),
            ));
        }
        let mut app = App::new();
        app.insert_resource(pending).add_systems(Update, spawn);
        app.update();
        let mut texts = std::collections::BTreeSet::new();
        for (label, text, font, color) in app
            .world_mut()
            .query::<(&Label, &Text, &TextFont, &TextColor)>()
            .iter(app.world())
        {
            texts.insert(text.0.clone());
            assert_eq!(
                font.font_size,
                FontSize::Px(if label.kind.critical() { 28. } else { 20. })
            );
            if matches!(
                label.kind,
                Kind::Ability {
                    outcome: AbilityOutcome::Arcane(MagicOutcome::Hit | MagicOutcome::Critical),
                    ..
                }
            ) {
                assert_eq!(color.0, Color::srgb(0.75, 0.6, 1.));
            }
        }
        for text in [
            "16", "33", "Miss", "Dodged", "Parried", "20", "30", "Resisted",
        ] {
            assert!(texts.contains(text), "{text}");
        }
        let metrics = &app.world().resource::<Pending>().metrics;
        assert_eq!(metrics.ability_spawned.special.criticals, 1);
        assert_eq!(metrics.ability_spawned.arcane.criticals, 1);
        assert_eq!(metrics.ability_spawned.arcane.misses, 1);
        assert_eq!(metrics.spawned_melee, 0);
    }
}
