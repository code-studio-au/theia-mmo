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
pub(crate) struct Config(pub Arc<Catalog>);
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub event_batches: u64,
    pub event_gaps: u64,
    pub last_batch: u64,
    pub started: u64,
    pub queued: u64,
    pub cancelled: u64,
    pub damage: u64,
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
pub fn install(app: &mut App, catalog: Arc<Catalog>) {
    ui::install(app);
    app.insert_resource(Config(catalog))
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
        Reason::Moving => "Cannot cast while moving",
        Reason::Interrupted => "Cast interrupted",
        Reason::Unavailable => "Ability not ready",
        Reason::Superseded => "Queued spell replaced",
        Reason::UnknownAbility | Reason::TimeExhausted => "Ability unavailable",
    }
}
fn note(ui: &mut Ui, text: &str) {
    ui.notice = text.into();
    ui.until = Some(Instant::now() + Duration::from_secs(2));
}
fn present(
    mut network: ResMut<Network>,
    config: Res<Config>,
    input: Res<ui::Input>,
    mut metrics: ResMut<Metrics>,
    mut ui: ResMut<Ui>,
    (mut hud, mut panel): (Query<&mut Text, With<Hud>>, ResMut<ui::Model>),
    (mut feedback, mut feedback_metrics): (
        Option<ResMut<crate::spell_feedback::Pending>>,
        Option<ResMut<crate::spell_feedback::Metrics>>,
    ),
) {
    let state = network.state();
    *panel = ui::Model::default();
    panel.connected = network.connected();
    panel.status = "Waiting for server".into();
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
            if let (Some(pending), Some(metrics)) = (&mut feedback, &mut feedback_metrics) {
                pending.offer(&event, batch.tick, metrics);
            }
            match event {
                Event::Started { .. } => metrics.started += 1,
                Event::Queued { .. } => metrics.queued += 1,
                Event::Cancelled { actor, reason, .. } => {
                    metrics.cancelled += 1;
                    if Some(actor.entity.value()) == state.id {
                        note(&mut ui, reason_text(reason));
                    }
                }
                Event::Damage { .. } => metrics.damage += 1,
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
        let ability = &config.0.abilities()[0];
        let selected = state.targeting.as_ref().and_then(|v| v.selected);
        if let Some(source) = input.0
            && network.connected()
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
        let rate =
            theia_core::time::TickRate::from_hz(hz as u32).expect("Validated server tick rate");
        let timing = config.0.timing(ability.id().as_str(), rate).unwrap();
        panel.cooldown = ui::remaining(
            now,
            ready_at,
            timing.cooldown_ticks.max(timing.global_cooldown_ticks),
        );
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
        let action = if let Some(cast) = &combat.cast
            && cast.completes > now
        {
            format!(
                "Casting {:.1}s{}",
                cast.completes.saturating_sub(now) as f64 / hz,
                if combat.queued.is_some() {
                    " | next spell queued"
                } else {
                    ""
                }
            )
        } else if ready_at > now {
            format!("Cooldown {:.1}s", ready_at.saturating_sub(now) as f64 / hz)
        } else {
            "Ready".into()
        };
        let name = ability
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
    if state.error.is_some() {
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
