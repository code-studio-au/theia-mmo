//! Explicit native-only qualification: ordinary inputs, committed observations.
use crate::{network::Network, reference_scene::attack};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use theia_core::inventory::EquipmentOperation;
use theia_network::{
    client::State,
    simulation::combat::{Outcome, Reason},
    wire::Intent,
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Expected {
    Hit,
    Miss,
    Deflect,
}
impl Expected {
    fn count(self, metrics: &crate::combat::Metrics) -> u64 {
        match self {
            Self::Hit => metrics.damage,
            Self::Miss => metrics.misses,
            Self::Deflect => metrics.deflects,
        }
    }
    fn health(self) -> u32 {
        if self == Self::Hit { 40 } else { 100 }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Deflect => "deflect",
        }
    }
}
#[derive(Resource)]
pub(crate) struct Run {
    variant: String,
    outcome: Expected,
    output: PathBuf,
    gate: PathBuf,
    phase: u8,
    changed: Instant,
    item_sent: bool,
    expected: u64,
    intent: Intent,
    pending_pointer: bool,
    world_pointer: Option<(bevy::picking::pointer::PointerButton, bool, Option<Vec2>)>,
    release_pointer: bool,
    pointer_inputs: u64,
    key_inputs: u64,
    cancelled: bool,
    wand_refused: bool,
    range_refused: bool,
    shot: bool,
    ready: bool,
    passed: bool,
    last_report: Instant,
}
pub(crate) fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(variant) = std::env::var("THEIA_PLAYER_MELEE_QUALIFY") else {
        return Ok(());
    };
    if !matches!(variant.as_str(), "A" | "B") {
        return Err("Melee qualification variant must be A or B".into());
    }
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").ok_or("Melee qualification needs output")?,
    );
    let gate = PathBuf::from(
        std::env::var_os("THEIA_PLAYER_MELEE_GATE")
            .ok_or("Melee qualification needs startup gate")?,
    );
    let now = Instant::now();
    let outcome = match std::env::var("THEIA_PLAYER_MELEE_OUTCOME")
        .as_deref()
        .unwrap_or("hit")
    {
        "hit" => Expected::Hit,
        "miss" => Expected::Miss,
        "deflect" => Expected::Deflect,
        _ => return Err("Unknown melee qualification outcome".into()),
    };
    app.insert_resource(Run {
        variant,
        outcome,
        output,
        gate,
        phase: 0,
        changed: now,
        item_sent: false,
        expected: 0,
        intent: Intent::default(),
        pending_pointer: false,
        world_pointer: None,
        release_pointer: false,
        pointer_inputs: 0,
        key_inputs: 0,
        cancelled: false,
        wand_refused: false,
        range_refused: false,
        shot: false,
        ready: false,
        passed: false,
        last_report: now,
    })
    .add_systems(PreUpdate, drive.after(crate::graybox::autoplay))
    .add_systems(
        First,
        pointer_input
            .after(bevy::picking::PickingSystems::Input)
            .before(bevy::picking::PickingSystems::PostInput),
    )
    .add_systems(
        Last,
        report
            .after(attack::observe)
            .after(crate::combat_feedback::Observed),
    );
    Ok(())
}
impl Run {
    pub(crate) fn intent(&self) -> Intent {
        self.intent
    }
    fn next(&mut self, phase: u8) {
        self.phase = phase;
        self.changed = Instant::now();
        self.item_sent = false;
        self.intent = Intent::default();
    }
    fn cast(&mut self, world: &mut World, state: &State, widget: bool) {
        self.expected = state
            .combat
            .as_ref()
            .and_then(|c| c.reply)
            .map_or(1, |r| r.sequence + 1);
        if widget {
            self.pending_pointer = true;
        } else {
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Digit2);
            self.key_inputs += 1;
        }
    }
    fn equip(&mut self, world: &World, state: &State, item: &str) {
        if self.item_sent || state.inventory_pending {
            return;
        }
        let slot = state.inventory.as_ref().and_then(|i| {
            i.bag
                .slots()
                .iter()
                .position(|s| s.as_ref().is_some_and(|s| s.item().as_str() == item))
        });
        if let Some(slot) = slot {
            self.item_sent = world
                .resource::<Network>()
                .request_equipment(EquipmentOperation::Equip { from: slot as u8 });
        }
    }
}
fn hand(state: &State, item: &str) -> bool {
    state
        .inventory
        .as_ref()
        .and_then(|i| i.bag.main_hand())
        .is_some_and(|s| s.item().as_str() == item)
}
fn drive(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::Digit2);
    let state = world.resource::<Network>().state();
    run.ready = state.error.is_none()
        && state.chunk_ack.is_some()
        && state.inventory.is_some()
        && state.combat.is_some()
        && world
            .get_resource::<attack::Metrics>()
            .is_some_and(|m| m.available);
    run.intent = Intent::default();
    if !run.ready || run.variant == "B" {
        world.insert_resource(run);
        return;
    }
    let combat = state.combat.as_ref().unwrap();
    let Some(npc) = state.targeting.as_ref().and_then(|t| t.targets.first()) else {
        world.insert_resource(run);
        return;
    };
    let mouse_mode = std::env::var_os("THEIA_WORLD_INPUT_QUALIFY").is_some();
    if !mouse_mode {
        world
            .resource::<Network>()
            .request_target(Some(npc.key.entity));
    }
    let Some(position) = state.authority_focus() else {
        world.insert_resource(run);
        return;
    };
    let a = position.metres();
    let b = npc.position.metres();
    let dx = b[0] - a[0];
    let dz = b[2] - a[2];
    let distance = dx.hypot(dz);
    let heading = if distance > 0.001 {
        [
            (dx / distance * 1000.) as i16,
            (dz / distance * 1000.) as i16,
        ]
    } else {
        [1000, 0]
    };
    let ready =
        combat.cast.is_none() && combat.gcd_until <= combat.tick && combat.cooldowns.is_empty();
    let health = combat.targets.first().map_or(100, |t| t.current);
    let metrics = world.resource::<crate::combat::Metrics>();
    let resolved = run.outcome.count(metrics);
    let equipment_cancelled = metrics.last_cancelled_reason.as_deref() == Some("EquipmentRequired")
        && metrics.cancelled == 1;
    let rejected = |reason| {
        combat
            .reply
            .is_some_and(|r| r.sequence == run.expected && r.outcome == Outcome::Rejected(reason))
    };
    match run.phase {
        0 => {
            if hand(&state, "theia.dev.training-sword") {
                run.next(1)
            } else {
                run.equip(world, &state, "theia.dev.training-sword")
            }
        }
        1 => {
            if run.gate.is_file() {
                if distance > 1.4 {
                    run.intent.axes = heading
                } else if ready
                    && state
                        .targeting
                        .as_ref()
                        .is_some_and(|t| t.selected == Some(npc.key.entity))
                {
                    if mouse_mode {
                        run.expected = combat.reply.map_or(1, |r| r.sequence + 1);
                        run.world_pointer = Some((
                            bevy::picking::pointer::PointerButton::Secondary,
                            false,
                            None,
                        ));
                    } else {
                        run.cast(world, &state, true);
                    }
                    run.next(2);
                } else if mouse_mode
                    && ready
                    && run.world_pointer.is_none()
                    && run.changed.elapsed() > Duration::from_millis(500)
                {
                    run.world_pointer =
                        Some((bevy::picking::pointer::PointerButton::Primary, false, None));
                    run.changed = Instant::now();
                }
            }
        }
        2 => {
            if resolved >= 1 && combat.cast.is_none() {
                run.next(3)
            }
        }
        3 | 5 | 7 => {
            if ready {
                if distance > 1.4 {
                    run.intent.axes = heading
                } else {
                    let phase = run.phase;
                    run.cast(world, &state, phase != 3);
                    run.next(phase + 1);
                }
            }
        }
        4 => {
            if run.changed.elapsed() < Duration::from_millis(450) {
                run.intent.axes = [0, 200];
            }
            if resolved >= 2 && combat.cast.is_none() {
                run.next(5)
            }
        }
        6 => {
            if run.changed.elapsed() < Duration::from_millis(750) {
                // Sustain ordinary running around the target inside reach.
                let radial = ((distance - 1.4) * 4.).clamp(-1., 1.);
                let x = -dz / distance.max(0.001) + dx / distance.max(0.001) * radial;
                let z = dx / distance.max(0.001) + dz / distance.max(0.001) * radial;
                let length = x.hypot(z).max(0.001);
                run.intent.axes = [(x / length * 1000.) as i16, (z / length * 1000.) as i16];
                run.intent.sprint = true;
            }
            if resolved >= 3 && combat.cast.is_none() {
                run.next(7)
            }
        }
        8 => {
            if combat
                .cast
                .as_ref()
                .is_some_and(|c| c.sequence == run.expected)
                && !run.item_sent
            {
                let slot = state
                    .inventory
                    .as_ref()
                    .unwrap()
                    .bag
                    .slots()
                    .iter()
                    .position(Option::is_none)
                    .unwrap();
                run.item_sent = world
                    .resource::<Network>()
                    .request_equipment(EquipmentOperation::Unequip { to: slot as u8 });
            }
            if run.item_sent
                && state.inventory.as_ref().unwrap().bag.main_hand().is_none()
                && combat.cast.is_none()
                && health == run.outcome.health()
                && equipment_cancelled
                && resolved == 3
            {
                run.cancelled = true;
                run.next(10)
            }
        }
        10 => {
            if hand(&state, "theia.dev.training-wand") {
                run.cast(world, &state, false);
                run.next(11)
            } else {
                run.equip(world, &state, "theia.dev.training-wand")
            }
        }
        11 if rejected(Reason::EquipmentRequired) => {
            run.wand_refused = true;
            run.next(12)
        }
        12 => {
            if hand(&state, "theia.dev.training-sword") {
                run.next(13)
            } else {
                run.equip(world, &state, "theia.dev.training-sword")
            }
        }
        13 => {
            if distance < 7. {
                run.intent.axes = [-heading[0], -heading[1]];
                run.intent.sprint = true;
            } else if ready {
                run.cast(world, &state, true);
                run.next(14)
            }
        }
        14 if rejected(Reason::OutOfRange) => {
            run.range_refused = true;
            run.next(15)
        }
        _ => {}
    }
    world.insert_resource(run);
}
fn pointer_input(world: &mut World) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    let mut run = world.remove_resource::<Run>().unwrap();
    if let Some((button, release, position)) = run.world_pointer {
        let key = world
            .resource::<Network>()
            .state()
            .targeting
            .as_ref()
            .and_then(|t| t.targets.first())
            .map(|t| t.key);
        if let Some(position) = position.or_else(|| {
            key.and_then(|key| crate::world_input::qualification::enemy_screen(world, key))
        }) {
            crate::world_input::qualification::mouse(world, position, button, release);
            run.world_pointer = if release {
                None
            } else {
                run.pointer_inputs += 1;
                Some((button, true, Some(position)))
            };
        }
    }
    if run.pending_pointer || run.release_pointer {
        let button = world
            .query::<(
                &ComputedNode,
                &bevy::ui::UiGlobalTransform,
                &crate::combat::ui::ActionButton,
            )>()
            .iter(world)
            .find(|(_, _, b)| b.0 == 1)
            .map(|(n, t, _)| (n.size(), t.affine().translation * n.inverse_scale_factor()));
        let window = world
            .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
            .iter(world)
            .next();
        if let (Some((size, position)), Some(window)) = (button, window)
            && size.min_element() > 0.
        {
            let location = Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position,
            };
            let action = if run.release_pointer {
                PointerAction::Release(PointerButton::Primary)
            } else {
                PointerAction::Press(PointerButton::Primary)
            };
            world.write_message(PointerInput::new(
                PointerId::Mouse,
                location.clone(),
                PointerAction::Move { delta: Vec2::ZERO },
            ));
            world.write_message(PointerInput::new(PointerId::Mouse, location, action));
            if run.release_pointer {
                run.release_pointer = false;
            } else {
                run.pending_pointer = false;
                run.release_pointer = true;
                run.pointer_inputs += 1;
            }
        }
    }
    world.insert_resource(run);
}
fn report(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let metrics = world.get_resource::<attack::Metrics>();
    let samples = metrics.map(|m| &m.samples);
    let samples = samples
        .map(|s| serde_json::to_value(s).unwrap())
        .unwrap_or_default();
    let poses = samples
        .as_array()
        .map(|s| {
            s.iter()
                .filter(|s| s["local"].as_bool() == Some(run.variant == "A"))
                .filter_map(|s| s["clip"].as_str())
                .collect::<std::collections::BTreeSet<_>>()
        })
        .unwrap_or_default();
    let all_poses = ["Idle", "Walk", "Run"]
        .iter()
        .all(|name| poses.contains(name));
    let ui = world.get_resource::<crate::combat::ui::Model>();
    let layout = world.get_resource::<crate::combat::ui::Metrics>();
    let ui_ok = ui.is_some_and(|ui| ui.actions.len() == 2)
        && layout.is_some_and(|m| m.button_layout_samples > 0 && m.invalid_layouts == 0);
    let state = world.resource::<Network>().state();
    let health = state
        .combat
        .as_ref()
        .and_then(|c| c.targets.first())
        .map(|t| t.current);
    let resolved = run
        .outcome
        .count(world.resource::<crate::combat::Metrics>());
    let labels = world.resource::<crate::combat_feedback::Pending>();
    let labels_ready = match run.outcome {
        Expected::Hit => true,
        Expected::Miss => {
            labels.metrics.spawned_misses == 3
                && labels.metrics.visible_miss_frames > 0
                && labels.metrics.dropped == 0
        }
        Expected::Deflect => {
            labels.metrics.spawned_deflects == 3
                && labels.metrics.visible_deflect_frames > 0
                && labels.metrics.dropped == 0
        }
    };
    let shot_ready = run.outcome == Expected::Hit
        || (resolved == 3
            && labels.metrics.visible_now > 0
            && labels.metrics.renderable_streak >= 3);
    run.passed = run.ready
        && all_poses
        && ui_ok
        && health == Some(run.outcome.health())
        && resolved == 3
        && labels_ready
        && (run.variant == "B"
            || (run.phase == 15
                && run.cancelled
                && run.wand_refused
                && run.range_refused
                && run.key_inputs > 0
                && run.pointer_inputs > 0));
    if !run.shot && all_poses && shot_ready {
        run.shot = true;
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(run.output.join("melee.png")));
    }
    if run.last_report.elapsed() > Duration::from_millis(200) {
        run.last_report = Instant::now();
        let data = serde_json::json!({"world_input":world.get_resource::<crate::world_input::Selection>(),"expected_outcome":run.outcome.name(),"labels":&world.resource::<crate::combat_feedback::Pending>().metrics,"unrelated_pipeline_pending_frames":world.resource::<crate::graybox::Graybox>().unrelated_pipeline_pending_frames,"owner_position":state.authority_focus().map(|p|p.metres()),"creature_position":state.targeting.as_ref().and_then(|t|t.targets.first()).map(|t|t.position.metres()),"ready":run.ready,"passed":run.passed,"variant":run.variant,"phase":run.phase,"health":health,"cancelled_by_unequip":run.cancelled,"wand_refused":run.wand_refused,"range_refused":run.range_refused,"pointer_inputs":run.pointer_inputs,"key_inputs":run.key_inputs,"attacks":world.get_resource::<attack::Metrics>(),"combat":world.get_resource::<crate::combat::Metrics>(),"ui":world.get_resource::<crate::combat::ui::Model>(),"layout":world.get_resource::<crate::combat::ui::Metrics>()});
        std::fs::write(
            run.output.join("melee.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .expect("Write native melee report");
    }
    world.insert_resource(run);
}
