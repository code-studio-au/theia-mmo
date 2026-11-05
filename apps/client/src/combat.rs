//! Small standard Bevy presentation of server combat; never computes damage.
pub(crate) mod ui;
use crate::network::Network;
use bevy::{prelude::*, text::FontSize};
use std::{
    io::Read,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::{abilities::Catalog, coordinates::GlobalPosition};
use theia_network::simulation::combat::{Event, Outcome, Reason};
#[derive(Resource)]
pub(crate) struct Config(
    pub Arc<Catalog>,
    pub Option<theia_core::resources::Profile>,
    pub bool,
);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub public_life_rosters: u64,
    pub public_attack_events: u64,
    pub public_attack_windups: u64,
    pub public_attack_idle: u64,
    pub life_events: u64,
    pub membership_events: u64,
    pub in_combat: Option<bool>,
    pub combat_entries: u64,
    pub combat_exits: u64,
    pub membership_opponents: usize,
    pub player_deaths: u64,
    pub player_respawns: u64,
    pub player_generation: u64,
    pub player_health: Option<u32>,
    pub incoming_windups: u64,
    pub blocked_life_casts: u64,
    pub resource_frames: u64,
    pub mana_rejections: u64,
    pub mana_decreases: u64,
    pub mana_regenerations: u64,
    pub last_mana: Option<u32>,
    pub minimum_mana: Option<u32>,
    pub event_batches: u64,
    pub event_gaps: u64,
    pub last_batch: u64,
    pub started: u64,
    pub last_cast_ticks: u64,
    pub last_cast_ability: Option<String>,
    pub queued: u64,
    pub cancelled: u64,
    pub last_cancelled_reason: Option<String>,
    pub damage: u64,
    pub ability_events: u64,
    pub ability_outcomes: crate::combat_feedback::AbilityOutcomes,
    pub ability_fatal_hits: u64,
    pub stale_ability_cosmetics: u64,
    pub last_ability: Option<String>,
    pub last_ability_starting: u32,
    pub last_ability_prevented: u32,
    pub last_ability_resolved: u32,
    pub last_ability_health_lost: u32,
    pub last_ability_overkill: u32,
    pub ordinary_events: u64,
    pub ordinary_outgoing: crate::combat_feedback::OutcomeCounts,
    pub ordinary_incoming: crate::combat_feedback::OutcomeCounts,
    pub physical_damage: u64,
    pub incoming_physical_damage: u64,
    pub stale_physical_cosmetics: u64,
    pub last_physical_starting: u32,
    pub last_physical_prevented: u32,
    pub last_physical_resolved: u32,
    pub last_physical_health_lost: u32,
    pub last_physical_overkill: u32,
    pub melee_damage: u64,
    pub melee_paused_frames: u64,
    pub projectile_launches: u64,
    pub projectile_ends: u64,
    pub misses: u64,
    pub deflects: u64,
    pub deaths: u64,
    pub respawns: u64,
    pub last_respawn_previous: u64,
    pub last_respawn_entity: u64,
    pub moving_cast_authority_samples: u64,
    pub selected_health_frames: u64,
    pub defeated_frames: u64,
    pub last_health: Option<u32>,
    pub max_health: u32,
    pub rejected_requests: u64,
    pub input_backpressure: u64,
    pub keyboard_requests: u64,
    pub widget_requests: u64,
}
#[derive(Resource, Default)]
struct Ui {
    notice: String,
    until: Option<Instant>,
    last_reply: u64,
    last_melee_reply: u64,
    previous: Option<(u64, GlobalPosition)>,
}
#[derive(Component)]
struct Hud;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Presentation;
pub fn load_catalog(path: &Path) -> Result<(Vec<u8>, Arc<Catalog>), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(theia_core::abilities::MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let catalog = Catalog::from_json(&bytes)?;
    Ok((bytes, Arc::new(catalog)))
}
pub fn load_resources(
    path: &Path,
) -> Result<(Vec<u8>, theia_core::resources::Profile), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(theia_core::resources::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let profile = theia_core::resources::Profile::from_json(&bytes)?;
    Ok((bytes, profile))
}
pub fn install(
    app: &mut App,
    catalog: Arc<Catalog>,
    profile: Option<theia_core::resources::Profile>,
    player_life: bool,
) {
    ui::install(app);
    crate::combat_feedback::install(app);
    if catalog.abilities().iter().any(|a| a.projectile().is_some()) {
        crate::projectile_visual::install(app);
    }
    app.insert_resource(Config(catalog, profile, player_life))
        .init_resource::<crate::reference_scene::attack::Cues>()
        .insert_resource(crate::reference_scene::attack::Metrics {
            capture: std::env::var_os("THEIA_PLAYER_MELEE_QUALIFY").is_some()
                || std::env::var_os("THEIA_DIRECTIONAL_LOCOMOTION_QUALIFY").is_some(),
            ..default()
        })
        .init_resource::<Metrics>()
        .init_resource::<Ui>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            present.after(crate::network::present).in_set(Presentation),
        );
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(186),
                width: px(348),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                padding: UiRect::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
        ))
        .with_child((
            Hud,
            Text::new("Preparing abilities..."),
            TextFont {
                font_size: FontSize::Px(18.),
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(10),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.17, 0.08, 0.1)),
                ))
                .with_child((
                    ui::HealthFill,
                    Node {
                        width: percent(0),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.68, 0.37)),
                ));
        });
}
fn reason_text(reason: Reason) -> &'static str {
    match reason {
        Reason::UnknownTarget | Reason::Dead => "Target unavailable",
        Reason::OutOfRange => "Out of range",
        Reason::NotReady => "Waiting for terrain",
        Reason::Obstructed => "Line of sight blocked",
        Reason::InsufficientMana => "Not enough mana",
        Reason::StatusCapacity => "Too many active statuses",
        Reason::EquipmentRequired => "Equip the required weapon",
        Reason::Moving => "Cannot cast while moving",
        Reason::Interrupted => "Cast interrupted",
        Reason::Unavailable => "Ability not ready",
        Reason::NotFacing => "Face the target",
        Reason::TooClose => "Too close",
        Reason::Superseded => "Queued spell replaced",
        Reason::UnknownAbility | Reason::TimeExhausted => "Ability unavailable",
    }
}
fn melee_reason(reason: theia_network::simulation::combat::autoattack::Pause) -> &'static str {
    use theia_network::simulation::combat::autoattack::Pause;
    match reason {
        Pause::NoWeapon => "Equip a melee weapon",
        Pause::InvalidTarget => "Target unavailable",
        Pause::DeadPlayer => "You are dead",
        Pause::NotReady => "Waiting for world",
        Pause::OutOfRange => "Out of range",
        Pause::NotFacing => "Face the target",
        Pause::Obstructed => "Line of sight blocked",
        Pause::Casting => "Casting",
        Pause::EventBudget => "Waiting for combat",
        Pause::TimeExhausted => "Attack unavailable",
    }
}
fn note(ui: &mut Ui, text: &str) {
    ui.notice = text.into();
    ui.until = Some(Instant::now() + Duration::from_secs(2));
}
type PresentationFeedback<'w> = (
    Option<ResMut<'w, crate::spell_feedback::Pending>>,
    Option<ResMut<'w, crate::spell_feedback::Metrics>>,
    Option<ResMut<'w, crate::creature_visual::Pending>>,
    Option<ResMut<'w, crate::reference_scene::attack::Cues>>,
    Option<ResMut<'w, crate::combat_feedback::Pending>>,
    Option<ResMut<'w, crate::projectile_visual::Cues>>,
);
fn stale_own_life(event: &Event, owner: Option<u64>, generation: u64) -> bool {
    let (actor, actor_generation, target, target_generation) = match event {
        Event::PhysicalDamage(f) => (f.actor, f.actor_generation, f.target, f.target_generation),
        Event::WhiteMelee(f) => (f.actor, f.actor_generation, f.target, f.target_generation),
        Event::AbilityDamage(f) => (f.actor, Some(f.actor_generation), f.target, None),
        _ => return false,
    };
    let life = if Some(actor.entity.value()) == owner {
        actor_generation
    } else if Some(target.entity.value()) == owner {
        target_generation
    } else {
        None
    };
    life.is_some_and(|value| value.get() != generation)
}
fn present(
    mut network: ResMut<Network>,
    config: Res<Config>,
    (input, selection): (Res<ui::Input>, Option<Res<crate::world_input::Selection>>),
    mut metrics: ResMut<Metrics>,
    mut ui: ResMut<Ui>,
    (mut hud, mut panel): (Query<&mut Text, With<Hud>>, ResMut<ui::Model>),
    (
        mut feedback,
        mut feedback_metrics,
        mut creature_visuals,
        mut attacks,
        mut results,
        mut projectiles,
    ): PresentationFeedback<'_>,
) {
    let state = network.state();
    *panel = ui::Model::default();
    panel.connected = network.connected();
    panel.status = "Waiting for server".into();
    for _ in 0..theia_network::renet_runtime::avatar_life::PRESENTATION_CAPACITY {
        if network.next_avatar_life_event().is_none() {
            break;
        }
        metrics.public_life_rosters += 1;
    }
    for _ in 0..theia_network::renet_runtime::creature_attacks::PRESENTATION_CAPACITY {
        let Some(event) = network.next_public_attack_event() else {
            break;
        };
        metrics.public_attack_events += 1;
        metrics.public_attack_windups += u64::from(event.windup.is_some());
        metrics.public_attack_idle += u64::from(event.windup.is_none());
    }
    for _ in 0..theia_network::renet_runtime::player_life::PRESENTATION_CAPACITY {
        let Some(event) = network.next_life_event() else {
            break;
        };
        metrics.life_events += 1;
        if let Some(summary) = event.combat {
            let active = summary.in_combat();
            metrics.membership_events += 1;
            metrics.combat_entries += u64::from(active && metrics.in_combat != Some(true));
            metrics.combat_exits += u64::from(!active && metrics.in_combat == Some(true));
            metrics.in_combat = Some(active);
            metrics.membership_opponents = summary.opponents;
        } else {
            metrics.in_combat = None;
            metrics.membership_opponents = 0;
        }

        metrics.player_deaths += u64::from(event.health == 0 && metrics.player_health != Some(0));
        metrics.player_respawns += u64::from(
            metrics.player_generation > 0 && event.key.generation.get() > metrics.player_generation,
        );
        metrics.player_generation = event.key.generation.get();
        metrics.player_health = Some(event.health);
        metrics.incoming_windups += u64::from(event.windup.is_some());
    }
    if input.0.is_some() && !network.can_act() {
        metrics.blocked_life_casts += 1;
    }
    for _ in 0..16 {
        let Some(batch) = network.next_combat_events() else {
            break;
        };
        if batch.sequence != metrics.last_batch + 1 {
            metrics.event_gaps += 1;
        }
        metrics.last_batch = batch.sequence;
        metrics.event_batches += 1;
        for event in batch.events {
            if stale_own_life(&event, state.id, metrics.player_generation) {
                if matches!(&event, Event::AbilityDamage(_)) {
                    metrics.stale_ability_cosmetics += 1;
                } else {
                    metrics.stale_physical_cosmetics += 1;
                }
                continue;
            }
            if let Some(attacks) = &mut attacks {
                match &event {
                    Event::PhysicalDamage(fact) => {
                        if let Some(profile) = network.physical_profile() {
                            attacks.offer_physical(fact, profile, Instant::now());
                        }
                    }
                    Event::WhiteMelee(fact) => {
                        if let Some(profile) = network.white_profile() {
                            attacks.offer_white(fact, profile.physical().catalog(), Instant::now());
                        }
                    }
                    _ => {}
                }
            }
            if let Some(projectiles) = &mut projectiles {
                projectiles.offer(
                    &event,
                    &config.0,
                    batch.tick,
                    state
                        .snapshot
                        .as_ref()
                        .map_or(batch.tick, |s| s.tick)
                        .max(state.combat.as_ref().map_or(0, |s| s.tick)),
                    state.snapshot.as_ref().map_or(30, |s| s.tick_hz),
                );
            }
            if let Some(results) = &mut results {
                results.offer(&event);
            }
            if let Some(attacks) = &mut attacks {
                attacks.offer(
                    &event,
                    &config.0,
                    batch.tick,
                    state.snapshot.as_ref().map_or(30, |s| s.tick_hz),
                    Instant::now(),
                );
            }
            if let Some(pending) = &mut creature_visuals {
                pending.offer(&event);
            }
            let physical = matches!(
                &event,
                Event::MeleeDamage { .. } | Event::PhysicalDamage(_) | Event::WhiteMelee(_)
            ) || matches!(&event,Event::AbilityDamage(f) if matches!(f.result.outcome,theia_network::simulation::combat::ability_damage::Outcome::MeleeSpecial(_)))
                || match &event {
                    Event::Started { cast, .. } => config.0.ability(cast.ability.as_str()),
                    Event::Damage { ability, .. } => config.0.ability(ability.as_str()),
                    Event::Avoided { ability, .. } => config.0.ability(ability.as_str()),
                    _ => None,
                }
                .is_some_and(|ability| {
                    ability.animation() == Some(theia_core::abilities::AnimationCue::SwordAttack)
                });
            if !physical
                && let (Some(pending), Some(metrics)) = (&mut feedback, &mut feedback_metrics)
            {
                pending.offer(&event, batch.tick, metrics);
            }
            match event {
                Event::ProjectileLaunched { .. } => metrics.projectile_launches += 1,
                Event::ProjectileEnded { .. } => metrics.projectile_ends += 1,
                Event::Avoided { actor, kind, .. } => {
                    use theia_network::simulation::combat::Avoidance;
                    metrics.misses += u64::from(kind == Avoidance::Miss);
                    metrics.deflects += u64::from(kind == Avoidance::Deflect);
                    if Some(actor.entity.value()) == state.id {
                        note(
                            &mut ui,
                            if kind == Avoidance::Miss {
                                "Miss"
                            } else {
                                "Deflected"
                            },
                        );
                    }
                }
                Event::Started { cast, .. } => {
                    metrics.started += 1;
                    metrics.last_cast_ticks = cast.completes.saturating_sub(cast.started);
                    metrics.last_cast_ability = Some(cast.ability.as_str().to_owned());
                }
                Event::Queued { .. } => metrics.queued += 1,
                Event::Cancelled { actor, reason, .. } => {
                    metrics.cancelled += 1;
                    metrics.last_cancelled_reason = Some(format!("{reason:?}"));
                    if Some(actor.entity.value()) == state.id {
                        metrics.mana_rejections += u64::from(reason == Reason::InsufficientMana);
                        note(&mut ui, reason_text(reason));
                    }
                }
                Event::Damage { .. } => metrics.damage += 1,
                Event::AbilityDamage(fact) => {
                    use theia_network::simulation::combat::ability_damage::Outcome;
                    metrics.ability_events += 1;
                    metrics.ability_outcomes.record(fact.result.outcome);
                    let positive = u64::from(fact.result.damage.resolved_damage > 0);
                    metrics.damage += positive;
                    metrics.physical_damage += positive
                        * u64::from(matches!(fact.result.outcome, Outcome::MeleeSpecial(_)));
                    metrics.misses += u64::from(matches!(
                        fact.result.outcome,
                        Outcome::MeleeSpecial(theia_core::damage::melee_special::Outcome::Miss)
                            | Outcome::Arcane(theia_core::damage::spell::Outcome::Miss)
                    ));
                    metrics.ability_fatal_hits +=
                        u64::from(fact.remaining == 0 && fact.result.damage.health_lost > 0);
                    metrics.last_ability = Some(fact.ability.as_str().into());
                    metrics.last_ability_starting = fact.result.damage.starting_damage;
                    metrics.last_ability_prevented = fact.result.damage.armour_prevented;
                    metrics.last_ability_resolved = fact.result.damage.resolved_damage;
                    metrics.last_ability_health_lost = fact.result.damage.health_lost;
                    metrics.last_ability_overkill = fact.result.damage.overkill;
                }
                Event::WhiteMelee(fact) => {
                    let positive = u64::from(fact.result.resolved_damage > 0);
                    metrics.ordinary_events += 1;
                    metrics.damage += positive;
                    metrics.physical_damage += positive;
                    metrics.incoming_physical_damage +=
                        positive * u64::from(fact.target_generation.is_some());
                    metrics.melee_damage += positive * u64::from(fact.actor_generation.is_some());
                    metrics.misses += u64::from(
                        fact.result.outcome == theia_core::damage::white_melee::Outcome::Miss,
                    );
                    if fact.target_generation.is_some() {
                        metrics.ordinary_incoming.record(fact.result.outcome);
                    } else {
                        metrics.ordinary_outgoing.record(fact.result.outcome);
                    }
                    metrics.last_physical_starting = fact.result.starting_damage;
                    metrics.last_physical_prevented = fact.result.armour.map_or(0, |a| a.prevented);
                    metrics.last_physical_resolved = fact.result.resolved_damage;
                    metrics.last_physical_health_lost = fact.result.health_lost;
                    metrics.last_physical_overkill = fact.result.overkill;
                }
                Event::PhysicalDamage(fact) => {
                    metrics.damage += 1;
                    metrics.physical_damage += 1;
                    metrics.incoming_physical_damage += u64::from(fact.target_generation.is_some());
                    metrics.melee_damage += u64::from(fact.actor_generation.is_some());
                    metrics.last_physical_starting = fact.result.starting_damage;
                    metrics.last_physical_prevented = fact.result.armour_prevented;
                    metrics.last_physical_resolved = fact.result.resolved_damage;
                    metrics.last_physical_health_lost = fact.result.health_lost;
                    metrics.last_physical_overkill = fact.result.overkill;
                }
                Event::MeleeDamage { .. } => {
                    metrics.damage += 1;
                    metrics.melee_damage += 1;
                }
                Event::Died { .. } => metrics.deaths += 1,
                Event::Respawned {
                    previous, target, ..
                } => {
                    metrics.respawns += 1;
                    metrics.last_respawn_previous = previous.entity.value();
                    metrics.last_respawn_entity = target.entity.value();
                }
            }
        }
    }
    let mut label = "Waiting for combat state...".to_owned();
    if let Some(combat) = &state.combat {
        if let Some(vitals) = combat.vitals {
            metrics.resource_frames += 1;
            if let Some(old) = metrics.last_mana {
                metrics.mana_decreases += u64::from(vitals.mana() < old);
                metrics.mana_regenerations += u64::from(vitals.mana() > old);
            }
            metrics.last_mana = Some(vitals.mana());
            metrics.minimum_mana = Some(
                metrics
                    .minimum_mana
                    .map_or(vitals.mana(), |old| old.min(vitals.mana())),
            );
            panel.vitals_ready = true;
            panel.player_health = vitals.health() as f32 / vitals.maximum_health() as f32;
            panel.mana = vitals.mana() as f32 / vitals.maximum_mana() as f32;
            panel.player_health_text =
                format!("Health {} / {}", vitals.health(), vitals.maximum_health());
            panel.mana_text = format!("Mana {} / {}", vitals.mana(), vitals.maximum_mana());
        }
        let ability = &config.0.abilities()[0];
        let selected = if selection.as_ref().is_some_and(|s| s.friendly.is_some()) {
            None
        } else {
            state.targeting.as_ref().and_then(|v| v.selected)
        };
        if let Some((slot, source)) = input.0
            && network.connected()
            && network.can_act()
            && let Some(ability) = config
                .0
                .abilities()
                .get(slot)
                .filter(|_| slot < ui::VISIBLE_ACTIONS)
        {
            match source {
                ui::Source::Keyboard => metrics.keyboard_requests += 1,
                ui::Source::Widget => metrics.widget_requests += 1,
            }

            if let Some(target) = selected {
                if !network.request_cast(ability.id().clone(), target) {
                    metrics.input_backpressure += 1;
                    note(&mut ui, "Ability input busy; try again");
                }
            } else {
                note(&mut ui, "Select a target with Tab");
            }
        }
        if let Some(reply) = combat.reply
            && reply.sequence > ui.last_reply
        {
            ui.last_reply = reply.sequence;
            if let Outcome::Rejected(reason) = reply.outcome {
                metrics.rejected_requests += 1;
                metrics.mana_rejections += u64::from(reason == Reason::InsufficientMana);
                note(&mut ui, reason_text(reason));
            } else {
                ui.until = None;
            }
        }
        let now = state
            .snapshot
            .as_ref()
            .map_or(combat.tick, |s| s.tick.max(combat.tick));
        let hz = state
            .snapshot
            .as_ref()
            .map_or(30., |s| f64::from(s.tick_hz));
        let health = combat.targets.iter().find(|h| Some(h.entity) == selected);
        panel.has_target = health.is_some_and(|h| h.current > 0);
        panel.health = health.map_or(0., |h| h.current as f32 / h.maximum as f32);
        let health_line = if let Some(health) = health {
            metrics.selected_health_frames += 1;
            metrics.last_health = Some(health.current);
            metrics.max_health = metrics.max_health.max(health.maximum);
            format!("Training target: {} / {}", health.current, health.maximum)
        } else if let Some(dead) = combat.targets.iter().find(|h| h.current == 0) {
            metrics.defeated_frames += 1;
            metrics.last_health = Some(0);
            format!(
                "Target defeated | respawn {:.1}s",
                dead.respawn_at.unwrap_or(now).saturating_sub(now) as f64 / hz
            )
        } else {
            "Select a target with Tab".into()
        };
        let rate =
            theia_core::time::TickRate::from_hz(hz as u32).expect("Validated server tick rate");
        for ability in config.0.abilities().iter().take(ui::VISIBLE_ACTIONS) {
            let ready_at = combat
                .cooldowns
                .iter()
                .find(|(id, _)| id == ability.id())
                .map_or(0, |(_, at)| *at)
                .max(if ability.uses_global_cooldown() {
                    combat.gcd_until
                } else {
                    0
                });
            let timing = config.0.timing(ability.id().as_str(), rate).unwrap();
            let cooldown = ui::remaining(
                now,
                ready_at,
                timing.cooldown_ticks.max(timing.global_cooldown_ticks),
            );
            let status = if let Some(cast) = &combat.cast
                && cast.completes > now
            {
                format!(
                    "Casting {:.1}s{}",
                    cast.completes.saturating_sub(now) as f64 / hz,
                    if combat.queued.is_some() {
                        " | next ability queued"
                    } else {
                        ""
                    }
                )
            } else if ready_at > now {
                format!("Cooldown {:.1}s", ready_at.saturating_sub(now) as f64 / hz)
            } else if combat
                .vitals
                .is_some_and(|v| v.mana() < ability.mana_cost())
            {
                "Needs mana".into()
            } else {
                "Ready".into()
            };
            panel.actions.push(ui::Action { cooldown, status });
        }
        if let Some(cast) = &combat.cast
            && cast.completes > now
        {
            panel.cast = 1.
                - ui::remaining(
                    now,
                    cast.completes,
                    cast.completes.saturating_sub(cast.started),
                );
        }
        panel.cooldown = panel.actions[0].cooldown;
        let action = panel.actions[0].status.clone();
        let active_ability = combat
            .cast
            .as_ref()
            .and_then(|cast| config.0.ability(cast.ability.as_str()))
            .unwrap_or(ability);
        let name = active_ability
            .id()
            .as_str()
            .rsplit('.')
            .next()
            .unwrap()
            .replace('-', " ");
        panel.action = action.clone();
        panel.status = if combat.cast.as_ref().is_some_and(|c| c.completes > now) {
            format!("{name} | {action}")
        } else if panel.has_target {
            action.clone()
        } else {
            "Select a living target".into()
        };
        if let Some(melee) = &state.autoattack {
            use theia_network::simulation::combat::autoattack::{Outcome, Phase};
            if let Some(reply) = melee.reply
                && reply.sequence > ui.last_melee_reply
            {
                ui.last_melee_reply = reply.sequence;
                if let Outcome::Rejected(reason) = reply.outcome {
                    note(&mut ui, melee_reason(reason));
                }
            }
            if combat.cast.is_none() {
                panel.status = match melee.phase {
                    Phase::Idle => panel.status.clone(),
                    Phase::Engaged | Phase::Attacking => "Autoattack engaged".into(),
                    Phase::Paused(reason) => {
                        metrics.melee_paused_frames += 1;
                        format!("Autoattack: {}", melee_reason(reason))
                    }
                };
            }
        }
        label = health_line;
        if ui.until.is_some_and(|until| until > Instant::now()) {
            label.push('\n');
            label.push_str(&ui.notice);
            panel.status.clone_from(&ui.notice);
        }
        if let Some(snapshot) = &state.snapshot
            && let Some(actor) = snapshot.players.iter().find(|p| Some(p.id) == state.id)
            && let Some(position) = snapshot.global_position(actor)
        {
            if let Some((tick, previous)) = ui.previous
                && snapshot.tick > tick
                && combat
                    .cast
                    .as_ref()
                    .is_some_and(|c| c.started <= snapshot.tick && snapshot.tick < c.completes)
            {
                let a = previous.metres();
                let b = position.metres();
                if (a[0] - b[0]).hypot(a[2] - b[2]) > 0.0001 {
                    metrics.moving_cast_authority_samples += 1;
                }
            }
            ui.previous = Some((snapshot.tick, position));
        }
    }
    if config.2 {
        panel.life_blocked = !network.can_act();
        panel.life_status = "Waiting for player life".into();
        if let Some(life) = &state.player_life
            && let Some(value) = life.current()
        {
            panel.vitals_ready = true;
            panel.player_health = value.health as f32 / value.maximum_health as f32;
            panel.player_health_text =
                format!("Health {} / {}", value.health, value.maximum_health);
            let now = state
                .snapshot
                .as_ref()
                .map_or(value.tick, |s| s.tick)
                .max(state.correction.map_or(value.tick, |c| c.state.input.tick));
            let seconds =
                |until: u64| until.saturating_sub(now) as f64 / f64::from(life.rate().hz());
            panel.life_status = if let Some(at) = value.respawn_at {
                if at > now {
                    format!("Dead | respawn {:.1}s", seconds(at))
                } else {
                    "Dead | preparing respawn".into()
                }
            } else if panel.life_blocked {
                "Respawning | waiting for movement".into()
            } else if let Some(windup) = value.windup {
                format!("Incoming attack {:.1}s", seconds(windup.completes))
            } else {
                "Alive".into()
            };
            if value.health > 0
                && !panel.life_blocked
                && let Some(summary) = value.combat
            {
                let active = summary.in_combat();
                panel.membership = Some(active);
                let status = if active { "In combat" } else { "Out of combat" };
                panel.life_status = if let Some(windup) = value.windup {
                    format!("{status} | attack {:.1}s", seconds(windup.completes))
                } else {
                    status.into()
                };
            }
        } else {
            panel.vitals_ready = false;
        }
        if panel.life_blocked {
            panel.status = panel.life_status.clone();
            panel.cast = 0.;
        }
    }
    if state.error.is_some() {
        panel.membership = None;
        panel.life_status = "Disconnected".into();
        label = "Disconnected".into();
        *panel = ui::Model {
            status: "Disconnected".into(),
            ..default()
        };
    }
    if let Ok(mut text) = hud.single_mut() {
        **text = label;
    }
}

#[cfg(test)]
pub(crate) mod test_facts;
#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::damage::white_melee::Outcome;
    #[test]
    fn life_generation_filter_keeps_current_lethal_facts_and_suppresses_obsolete_own_cosmetics() {
        for incoming in [false, true] {
            let fact = test_facts::fact(Outcome::Critical, incoming, 1);
            assert_eq!(fact.remaining, 0);
            let event = Event::WhiteMelee(fact);
            assert!(!stale_own_life(&event, Some(13), 1));
            assert!(stale_own_life(&event, Some(13), 2));
            assert!(stale_own_life(&event, Some(13), 0));
            assert!(!stale_own_life(&event, Some(99), 2));
        }
    }
    #[test]
    fn typed_ability_cosmetics_keep_current_lethal_results_and_reject_stale_own_life() {
        use theia_network::simulation::combat::ability_damage::Outcome;
        let fact = test_facts::ability_fact(
            Outcome::MeleeSpecial(theia_core::damage::melee_special::Outcome::Critical),
            1,
        );
        let event = Event::AbilityDamage(fact.clone());
        assert!(!stale_own_life(&event, Some(13), 1));
        assert!(stale_own_life(&event, Some(13), 2));
        assert!(stale_own_life(&event, Some(13), 0));
        assert!(!stale_own_life(&event, Some(99), 2));
        assert_eq!(fact.remaining, 0);
    }
}
