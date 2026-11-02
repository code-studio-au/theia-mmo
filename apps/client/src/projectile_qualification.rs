//! Explicit native-only tour using ordinary widget/key and equipment inputs.
use crate::network::Network;
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
#[derive(Resource)]
pub(crate) struct Run {
    variant: String,
    output: PathBuf,
    gate: PathBuf,
    outcome: String,
    phase: u8,
    expected: u64,
    item_sent: bool,
    intent: Intent,
    pointer: bool,
    release: bool,
    pointer_inputs: u64,
    key_inputs: u64,
    ready: bool,
    passed: bool,
    cancelled: bool,
    wrong_weapon: bool,
    range_refused: bool,
    shot: bool,
    shot_health: Option<u32>,
    shot_resolved: u64,
    pre_damage_frames: u64,
    last_report: Instant,
}
pub(crate) fn install(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(variant) = std::env::var("THEIA_PROJECTILE_QUALIFY") else {
        return Ok(());
    };
    if !matches!(variant.as_str(), "A" | "B") {
        return Err("Projectile variant must be A or B".into());
    }
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").ok_or("Projectile output is required")?,
    );
    let gate = PathBuf::from(
        std::env::var_os("THEIA_PROJECTILE_GATE").ok_or("Projectile gate is required")?,
    );
    let outcome = std::env::var("THEIA_PROJECTILE_OUTCOME").unwrap_or_else(|_| "hit".into());
    if !matches!(outcome.as_str(), "hit" | "miss" | "deflect") {
        return Err("Unknown projectile qualification outcome".into());
    }
    app.insert_resource(Run {
        variant,
        output,
        gate,
        outcome,
        phase: 0,
        expected: 0,
        item_sent: false,
        intent: Intent::default(),
        pointer: false,
        release: false,
        pointer_inputs: 0,
        key_inputs: 0,
        ready: false,
        passed: false,
        cancelled: false,
        wrong_weapon: false,
        range_refused: false,
        shot: false,
        shot_health: None,
        shot_resolved: 0,
        pre_damage_frames: 0,
        last_report: Instant::now(),
    })
    .add_systems(PreUpdate, drive.after(crate::graybox::autoplay))
    .add_systems(
        First,
        pointer
            .after(bevy::picking::PickingSystems::Input)
            .before(bevy::picking::PickingSystems::PostInput),
    )
    .add_systems(Last, report);
    Ok(())
}
impl Run {
    pub(crate) fn intent(&self) -> Intent {
        self.intent
    }
    fn count(&self, m: &crate::combat::Metrics) -> u64 {
        match self.outcome.as_str() {
            "miss" => m.misses,
            "deflect" => m.deflects,
            _ => m.damage,
        }
    }
    fn next(&mut self, phase: u8) {
        self.phase = phase;
        self.item_sent = false;
    }
    fn cast(&mut self, world: &mut World, state: &State, widget: bool) {
        self.expected = state
            .combat
            .as_ref()
            .and_then(|c| c.reply)
            .map_or(1, |r| r.sequence + 1);
        if widget {
            self.pointer = true;
        } else {
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Digit1);
            self.key_inputs += 1;
        }
    }
    fn equip(&mut self, world: &World, state: &State, item: &str) {
        if self.item_sent || state.inventory_pending {
            return;
        }
        if let Some(slot) = state.inventory.as_ref().and_then(|i| {
            i.bag
                .slots()
                .iter()
                .position(|s| s.as_ref().is_some_and(|s| s.item().as_str() == item))
        }) {
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
        .release(KeyCode::Digit1);
    let state = world.resource::<Network>().state();
    run.ready = state.error.is_none()
        && state.chunk_ack.is_some()
        && state.inventory.is_some()
        && state.combat.is_some()
        && world
            .get_resource::<crate::projectile_visual::Cues>()
            .is_some()
        && world
            .get_resource::<crate::reference_scene::attack::Metrics>()
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
    world
        .resource::<Network>()
        .request_target(Some(npc.key.entity));
    let resolved = run.count(world.resource::<crate::combat::Metrics>());
    let ready =
        combat.cast.is_none() && combat.gcd_until <= combat.tick && combat.cooldowns.is_empty();
    let rejected = |reason| {
        combat
            .reply
            .is_some_and(|r| r.sequence == run.expected && r.outcome == Outcome::Rejected(reason))
    };
    match run.phase {
        0 => {
            if hand(&state, "theia.dev.training-wand") {
                run.next(1)
            } else {
                run.equip(world, &state, "theia.dev.training-wand")
            }
        }
        1 => {
            if run.gate.is_file()
                && ready
                && state
                    .targeting
                    .as_ref()
                    .is_some_and(|t| t.selected == Some(npc.key.entity))
            {
                run.cast(world, &state, true);
                run.next(2);
            }
        }
        2 => {
            if resolved >= 1 && ready {
                run.cast(world, &state, false);
                run.next(3);
            }
        }
        3 => {
            if resolved >= 2 && ready {
                run.cast(world, &state, true);
                run.next(4);
            }
        }
        4 => {
            if resolved == 3 && ready {
                run.cast(world, &state, false);
                run.next(5);
            }
        }
        5 => {
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
            let metrics = world.resource::<crate::combat::Metrics>();
            if run.item_sent
                && state.inventory.as_ref().unwrap().bag.main_hand().is_none()
                && combat.cast.is_none()
                && metrics.cancelled == 1
                && metrics.last_cancelled_reason.as_deref() == Some("EquipmentRequired")
            {
                run.cancelled = true;
                run.next(6);
            }
        }
        6 => {
            if hand(&state, "theia.dev.training-sword") {
                run.cast(world, &state, true);
                run.next(7)
            } else {
                run.equip(world, &state, "theia.dev.training-sword")
            }
        }
        7 => {
            if rejected(Reason::EquipmentRequired) {
                run.wrong_weapon = true;
                run.next(8)
            }
        }
        8 => {
            if hand(&state, "theia.dev.training-wand") {
                run.next(9)
            } else {
                run.equip(world, &state, "theia.dev.training-wand")
            }
        }
        9 => {
            if let Some(position) = state.authority_focus() {
                let a = position.metres();
                let b = npc.position.metres();
                let dx = a[0] - b[0];
                let dz = a[2] - b[2];
                let distance = dx.hypot(dz);
                if distance < 34. {
                    run.intent.axes = [
                        (dx / distance.max(0.001) * 1000.) as i16,
                        (dz / distance.max(0.001) * 1000.) as i16,
                    ];
                    run.intent.sprint = true;
                } else if ready {
                    run.cast(world, &state, true);
                    run.next(10);
                }
            }
        }
        10 if rejected(Reason::OutOfRange) => {
            run.range_refused = true;
            run.next(11)
        }
        _ => {}
    }
    world.insert_resource(run);
}
fn pointer(world: &mut World) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    let mut run = world.remove_resource::<Run>().unwrap();
    if run.pointer || run.release {
        let button = world
            .query::<(
                &ComputedNode,
                &bevy::ui::UiGlobalTransform,
                &crate::combat::ui::ActionButton,
            )>()
            .iter(world)
            .find(|(_, _, b)| b.0 == 0)
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
            let action = if run.release {
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
            if run.release {
                run.release = false;
            } else {
                run.pointer = false;
                run.release = true;
                run.pointer_inputs += 1;
            }
        }
    }
    world.insert_resource(run);
}
fn report(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    let combat = world.resource::<crate::combat::Metrics>();
    let resolved = run.count(combat);
    let health = state
        .combat
        .as_ref()
        .and_then(|c| c.targets.first())
        .map(|t| t.current);
    let visuals = world.get_resource::<crate::projectile_visual::Cues>();
    let visible =
        visuals.is_some_and(|v| v.metrics.visible_now > 0 && v.metrics.visible_streak >= 3);
    let pipelines = world
        .get_resource::<crate::capture_ready::CaptureReady>()
        .is_some_and(|r| r.ready());
    let capture = visible && pipelines && resolved == 0 && health == Some(100) && !run.shot;
    if visible && pipelines && resolved == 0 && health == Some(100) {
        run.pre_damage_frames += 1;
        if !run.shot {
            run.shot = true;
            run.shot_health = health;
            run.shot_resolved = resolved;
        }
    }
    let visuals_ok = visuals.is_some_and(|v| {
        v.metrics.launches == 3
            && v.metrics.spawns == 3
            && v.metrics.terminals == 3
            && v.metrics.visible_frames > 0
            && v.metrics.dropped == 0
            && v.metrics.maximum_conversion_error_metres < 0.001
    });
    let ui = world.resource::<crate::combat::ui::Model>();
    let layout = world.resource::<crate::combat::ui::Metrics>();
    let expected_health = if run.outcome == "hit" { 40 } else { 100 };
    run.passed = run.ready
        && run.shot
        && run.pre_damage_frames > 0
        && visuals_ok
        && health == Some(expected_health)
        && resolved == 3
        && ui.actions.len() == 2
        && layout.invalid_layouts == 0
        && layout.button_layout_samples > 0
        && (run.variant == "B"
            || (run.phase == 11
                && run.cancelled
                && run.wrong_weapon
                && run.range_refused
                && run.pointer_inputs > 0
                && run.key_inputs > 0));
    if run.last_report.elapsed() > Duration::from_millis(200) {
        run.last_report = Instant::now();
        let data = serde_json::json!({"ready":run.ready,"passed":run.passed,"variant":run.variant,"phase":run.phase,"expected_outcome":run.outcome,
            "health":health,"cancelled_by_unequip":run.cancelled,"wrong_weapon_refused":run.wrong_weapon,"range_refused":run.range_refused,
            "pointer_inputs":run.pointer_inputs,"key_inputs":run.key_inputs,"shot_health":run.shot_health,"shot_resolved":run.shot_resolved,
            "pre_damage_frames":run.pre_damage_frames,"visuals":visuals.map(|v|&v.metrics),"combat":combat,"ui":ui,"layout":layout,
            "labels":&world.resource::<crate::combat_feedback::Pending>().metrics,
            "owner_position":state.authority_focus().map(|p|p.metres()),"creature_position":state.targeting.as_ref().and_then(|t|t.targets.first()).map(|t|t.position.metres())});
        std::fs::write(
            run.output.join("projectile.json"),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .expect("Write projectile native report");
    }
    if capture {
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(run.output.join("projectile.png")));
    }
    world.insert_resource(run);
}
