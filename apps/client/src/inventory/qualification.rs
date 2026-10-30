//! Explicit native UI qualification. Ordinary play installs no automation.
use super::{ClaimButton, EquipButton, SlotButton, SplitButton, Ui, UnequipButton};
use crate::network::Network;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Copy)]
enum Target {
    Slot(u8),
    Split,
    Equip,
    Unequip,
    Claim,
}
enum Action {
    WaitOffer,
    Click(Target),
    Wait {
        revision: u64,
        sequence: u64,
        rejected: bool,
    },
    Observe {
        hand: Option<&'static str>,
        name: &'static str,
    },
}
#[derive(Resource)]
struct Run {
    rewards: bool,
    earned: Option<serde_json::Value>,
    casts: u64,
    next_cast: Instant,
    next_select: Instant,
    equipment: bool,
    phase_ready: Option<Instant>,
    observations: Vec<serde_json::Value>,
    output: PathBuf,
    variant: String,
    actions: Vec<Action>,
    step: usize,
    pointer_pending: Option<Target>,
    pointer_release: Option<Target>,
    awaiting_click: bool,
    pointer_inputs: u64,
    opened: bool,
    checked: bool,
    resumed: bool,
    started: Option<Instant>,
    next: Instant,
    ready: Option<Instant>,
    shot: bool,
    written: Instant,
    layout_samples: u64,
    invalid_layouts: u64,
}
pub(super) fn install(app: &mut App) {
    let equipment = std::env::var("THEIA_EQUIPMENT_QUALIFY").ok();
    let Some(variant) = equipment
        .clone()
        .or_else(|| std::env::var("THEIA_INVENTORY_QUALIFY").ok())
    else {
        return;
    };
    assert!(
        matches!(variant.as_str(), "A" | "B"),
        "Inventory qualification variant must be A or B"
    );
    let (temporary, partial, final_slot) = if variant == "A" { (3, 4, 5) } else { (4, 3, 6) };
    let click = |slot| Action::Click(Target::Slot(slot));
    let wait = |revision, sequence, rejected| Action::Wait {
        revision,
        sequence,
        rejected,
    };
    let rewards = std::env::var("THEIA_REWARDS_QUALIFY").as_deref() == Ok("1");
    assert!(
        !rewards || equipment.is_some(),
        "Reward qualification requires equipment fixture"
    );
    let mut actions = if equipment.is_some() {
        let (first, second, invalid, empty, last, first_name, second_name) = if variant == "A" {
            (
                3,
                4,
                0,
                5,
                4,
                "theia.dev.simple-sword",
                "theia.dev.simple-wand",
            )
        } else {
            (
                4,
                3,
                2,
                6,
                3,
                "theia.dev.simple-wand",
                "theia.dev.simple-sword",
            )
        };
        vec![
            click(first),
            Action::Click(Target::Equip),
            wait(1, 1, false),
            Action::Observe {
                hand: Some(first_name),
                name: "equipment-first",
            },
            click(second),
            Action::Click(Target::Equip),
            wait(2, 2, false),
            Action::Observe {
                hand: Some(second_name),
                name: "equipment-replaced",
            },
            click(invalid),
            Action::Click(Target::Equip),
            wait(2, 3, true),
            Action::Click(Target::Unequip),
            click(empty),
            wait(3, 4, false),
            Action::Observe {
                hand: None,
                name: "equipment-empty",
            },
            click(last),
            Action::Click(Target::Equip),
            wait(4, 5, false),
            Action::Observe {
                hand: Some(first_name),
                name: "equipment-final",
            },
        ]
    } else {
        vec![
            click(0),
            click(temporary),
            wait(1, 1, false),
            click(1),
            Action::Click(Target::Split),
            click(partial),
            wait(2, 2, false),
            click(2),
            click(temporary),
            wait(3, 3, false),
            click(partial),
            click(1),
            wait(4, 4, false),
            click(1),
            Action::Click(Target::Split),
            click(2),
            wait(4, 5, true),
            click(1),
            click(2),
            wait(5, 6, false),
            click(temporary),
            click(final_slot),
            wait(6, 7, false),
        ]
    };
    if rewards {
        actions.extend([
            Action::WaitOffer,
            Action::Click(Target::Claim),
            wait(5, 6, false),
        ]);
    }
    app.insert_resource(Run {
        rewards,
        earned: None,
        casts: 0,
        next_cast: Instant::now(),
        next_select: Instant::now(),
        equipment: equipment.is_some(),
        phase_ready: None,
        observations: Vec::new(),
        output: std::env::var_os("THEIA_GRAYBOX_QUALIFY")
            .map(PathBuf::from)
            .expect("Inventory qualification requires graybox output"),
        variant,
        actions,
        step: 0,
        pointer_pending: None,
        pointer_release: None,
        awaiting_click: false,
        pointer_inputs: 0,
        opened: false,
        checked: false,
        resumed: false,
        started: None,
        next: Instant::now(),
        ready: None,
        shot: false,
        written: Instant::now(),
        layout_samples: 0,
        invalid_layouts: 0,
    })
    .add_systems(
        First,
        pointer
            .after(bevy::picking::PickingSystems::Input)
            .before(bevy::picking::PickingSystems::PostInput),
    )
    .add_systems(PreUpdate, drive.after(crate::graybox::autoplay))
    .add_systems(Last, observe);
}
fn location(world: &mut World, target: Target) -> Option<(Vec2, Vec2)> {
    match target {
        Target::Slot(index) => world
            .query::<(&SlotButton, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
            .iter(world)
            .find(|(button, _, _)| button.0 == index)
            .map(|(_, node, transform)| {
                (
                    node.size(),
                    transform.affine().translation * node.inverse_scale_factor(),
                )
            }),
        Target::Split => world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<SplitButton>>()
            .iter(world)
            .next()
            .map(|(node, transform)| {
                (
                    node.size(),
                    transform.affine().translation * node.inverse_scale_factor(),
                )
            }),
        Target::Claim => world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<ClaimButton>>()
            .iter(world)
            .next()
            .map(|(node, transform)| {
                (
                    node.size(),
                    transform.affine().translation * node.inverse_scale_factor(),
                )
            }),
        Target::Equip | Target::Unequip => world
            .query::<(
                &ComputedNode,
                &bevy::ui::UiGlobalTransform,
                Has<EquipButton>,
                Has<UnequipButton>,
            )>()
            .iter(world)
            .find(|(_, _, equip, unequip)| {
                matches!(target, Target::Equip) && *equip
                    || matches!(target, Target::Unequip) && *unequip
            })
            .map(|(node, transform, _, _)| {
                (
                    node.size(),
                    transform.affine().translation * node.inverse_scale_factor(),
                )
            }),
    }
}
// Goes through Bevy picking, hit testing and the production widget observers.
// This is synthetic native pointer delivery, not an operating-system mouse test.
fn pointer(world: &mut World) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    let mut run = world.remove_resource::<Run>().unwrap();
    if let Some(target) = run.pointer_release.or(run.pointer_pending)
        && let Some((size, position)) = location(world, target)
        && size.min_element() > 0.
        && let Some(window) = world
            .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
            .iter(world)
            .next()
    {
        let target = bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
            .normalize(Some(window))
            .unwrap();
        let location = Location { target, position };
        let action = if run.pointer_release.is_some() {
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
        if run.pointer_release.take().is_none() {
            run.pointer_release = run.pointer_pending.take();
            run.pointer_inputs += 1;
        }
    }
    world.insert_resource(run);
}
fn drive(
    network: Res<Network>,
    combat: Option<Res<crate::combat::Config>>,
    ui: Res<Ui>,
    mut run: ResMut<Run>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut movement: ResMut<crate::movement::MoveIntent>,
) {
    keys.release(KeyCode::KeyI);
    if !network.connected() {
        return;
    }
    let state = network.state();
    let Some(view) = &state.inventory else { return };
    if !run.checked {
        assert!(
            view.bag.revision() == 0
                || view.bag.revision()
                    == if run.rewards {
                        5
                    } else if run.equipment {
                        4
                    } else {
                        6
                    },
            "Unexpected inventory qualification starting revision"
        );
        run.resumed = view.bag.revision()
            == if run.rewards {
                5
            } else if run.equipment {
                4
            } else {
                6
            };
        run.checked = true;
        run.started = Some(Instant::now());
        if run.resumed {
            run.step = run.actions.len();
        }
    }
    assert!(
        run.started.unwrap().elapsed() < Duration::from_secs(if run.rewards { 40 } else { 20 })
            || run.step == run.actions.len(),
        "Native inventory widget script timed out"
    );
    if run.rewards {
        // Keep the early restored-pose evidence, then follow within spell range
        // using ordinary movement intents. Stop casting after own availability.
        let elapsed = run.started.unwrap().elapsed().as_secs_f64();
        movement.direction = Vec2::ZERO;
        movement.sprint = false;
        movement.jump = false;
        let available = state
            .rewards
            .as_ref()
            .is_some_and(|s| !s.available.is_empty());
        if !run.resumed
            && elapsed > 4.1
            && let Some(target) = state.targeting.as_ref().and_then(|v| v.targets.first())
        {
            if let Some(focus) = state.authority_focus() {
                let a = focus.metres();
                let b = target.position.metres();
                movement.direction = (Vec2::new((b[0] - 6. - a[0]) as f32, (b[2] - a[2]) as f32)
                    * 0.4)
                    .clamp_length_max(1.);
            }
            if state
                .targeting
                .as_ref()
                .is_some_and(|v| v.selected != Some(target.key.entity))
                && Instant::now() >= run.next_select
            {
                network.request_target(Some(target.key.entity));
                run.next_select = Instant::now() + Duration::from_millis(500);
            }
            if !available
                && run.earned.is_none()
                && Instant::now() >= run.next_cast
                && state.combat.as_ref().is_some_and(|c| {
                    c.targets
                        .iter()
                        .any(|h| h.entity == target.key.entity && h.current > 0)
                })
            {
                let ability = combat
                    .as_ref()
                    .expect("Reward qualification requires combat")
                    .0
                    .abilities()[0]
                    .id()
                    .clone();
                if network.request_cast(ability, target.key.entity) {
                    run.casts += 1;
                }
                run.next_cast = Instant::now() + Duration::from_millis(1700);
            }
        }
    }
    if run.equipment && !run.resumed {
        let seconds = run.started.unwrap().elapsed().as_secs_f64();
        if seconds < 8. {
            movement.direction = if run.variant == "B" && (3.0..3.65).contains(&seconds) {
                Vec2::X
            } else {
                Vec2::ZERO
            };
            movement.sprint = false;
            movement.jump = false;
        }
        // Preserve the existing first-pose observation before 3s, then separate
        // the nonblocking players through actual intents for readable captures.
        if seconds < 4.1 {
            return;
        }
    }
    if !ui.open && !run.opened {
        keys.press(KeyCode::KeyI);
        run.opened = true;
        return;
    }
    if !ui.open || Instant::now() < run.next {
        return;
    }
    if run.awaiting_click {
        if run.pointer_pending.is_none() && run.pointer_release.is_none() {
            run.awaiting_click = false;
            run.step += 1;
            run.next = Instant::now() + Duration::from_millis(60);
        }
        return;
    }
    match run.actions.get(run.step) {
        Some(Action::WaitOffer) => {
            if let Some(offered) = state.rewards.as_ref().filter(|s| !s.available.is_empty()) {
                run.earned = Some(
                    serde_json::json!({"generation":offered.generation,"count":offered.available.len(),"bag_revision":view.bag.revision(),"reward":offered.available[0].reward.as_str()}),
                );
                run.step += 1;
            }
        }
        Some(Action::Click(target)) => {
            run.pointer_pending = Some(*target);
            run.awaiting_click = true;
        }
        Some(Action::Wait {
            revision,
            sequence,
            rejected,
        }) => {
            if let Some(reply) = view.reply {
                assert!(
                    reply.sequence <= *sequence,
                    "Native inventory script skipped an acknowledgement"
                );
                if reply.sequence == *sequence && ui.last_reply == *sequence && !ui.pending {
                    assert_eq!(view.bag.revision(), *revision);
                    assert_eq!(
                        matches!(
                            reply.outcome,
                            theia_network::inventory_wire::Outcome::Rejected(_)
                        ),
                        *rejected
                    );
                    run.step += 1;
                    run.next = Instant::now() + Duration::from_millis(60);
                }
            }
        }
        None => {}
        Some(Action::Observe { .. }) => {}
    }
}
fn equipment_observation(world: &mut World, hand: Option<&str>) -> Option<serde_json::Value> {
    let state = world.resource::<Network>().state();
    let actor = state.id?;
    let bag = &state.inventory.as_ref()?.bag;
    let private = bag
        .main_hand()
        .and_then(|s| {
            world
                .resource::<super::Config>()
                .0
                .catalog()
                .definition(s.item())
        })
        .and_then(|d| d.main_hand_appearance())
        .map(|id| id.as_str());
    if private != hand {
        return None;
    }
    if !state.appearance.as_ref()?.actors.iter().any(|a| {
        a.key.entity.value() == actor && a.main_hand.as_ref().map(|id| id.as_str()) == hand
    }) {
        return None;
    }
    let attachments: Vec<_> = world
        .query::<(Entity, &crate::equipment::Attachment)>()
        .iter(world)
        .filter(|(_, a)| a.actor.entity.value() == actor)
        .map(|(e, a)| (e, a.appearance.clone()))
        .collect();
    let mut visible = 0;
    if let Some(hand) = hand {
        if attachments.len() != 1 || attachments[0].1.as_str() != hand {
            return None;
        }
        for entity in crate::reference_scene::descendants(world, attachments[0].0) {
            visible += usize::from(
                world.get::<Mesh3d>(entity).is_some()
                    && world.get::<ViewVisibility>(entity).is_some_and(|v| v.get()),
            );
        }
        if visible == 0 {
            return None;
        }
    } else if !attachments.is_empty() {
        return None;
    }
    let metrics = world.get_resource::<crate::equipment::Metrics>()?;
    if metrics.local_samples < 10 || metrics.last_error.is_some() {
        return None;
    }
    assert!(
        metrics.max_socket_error < 0.002
            && metrics.max_scale_error < 0.002
            && metrics.max_grip_offset < 0.04,
        "Equipment socket/scale did not follow the actual animated hand: {}",
        serde_json::to_string(metrics).unwrap()
    );
    Some(
        serde_json::json!({"actor":actor,"hand":hand,"visible_meshes":visible,"attachments":attachments.len(),"metrics":metrics}),
    )
}
fn observe(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let (open, pending, bag) = {
        let ui = world.resource::<Ui>();
        (ui.open, ui.pending, ui.bag.clone())
    };
    let viewport = world
        .query_filtered::<&Window, With<bevy::window::PrimaryWindow>>()
        .iter(world)
        .next()
        .map(|window| Vec2::new(window.width(), window.height()));
    if open
        && bag.is_some()
        && let Some(viewport) = viewport
    {
        for (node, transform) in world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<super::Pane>>()
            .iter(world)
        {
            let size = node.size() * node.inverse_scale_factor();
            let centre = transform.affine().translation * node.inverse_scale_factor();
            if size.min_element() > 0. {
                let low = centre - size * 0.5;
                let high = centre + size * 0.5;
                run.invalid_layouts += u64::from(
                    low.x < 0. || low.y < 0. || high.x > viewport.x || high.y > viewport.y,
                );
            }
        }
        let mut rows = std::collections::BTreeSet::new();
        let mut columns = std::collections::BTreeSet::new();
        let mut count = 0;
        for (node, transform) in world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<SlotButton>>()
            .iter(world)
        {
            if node.size().min_element() <= 0. {
                continue;
            }
            let centre = transform.affine().translation * node.inverse_scale_factor();
            count += 1;
            rows.insert(centre.y.round() as i32);
            columns.insert(centre.x.round() as i32);
            run.invalid_layouts += u64::from(
                !centre.is_finite()
                    || centre.x < 0.
                    || centre.x > viewport.x
                    || centre.y < 0.
                    || centre.y > viewport.y,
            );
        }
        if count == 16 {
            run.layout_samples += 1;
            run.invalid_layouts += u64::from(rows.len() != 4 || columns.len() != 4);
        }
    }
    let mut phase_captured = false;
    if !pending && let Some(Action::Observe { hand, name }) = run.actions.get(run.step) {
        let (hand, name) = (*hand, *name);
        if let Some(observed) = equipment_observation(world, hand) {
            let ready = run.phase_ready.get_or_insert_with(Instant::now);
            if ready.elapsed() > Duration::from_millis(350) {
                std::fs::create_dir_all(&run.output).unwrap();
                world
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(run.output.join(format!("{name}.png"))));
                phase_captured = true;
                run.observations.push(observed);
                run.step += 1;
                run.phase_ready = None;
                run.next = Instant::now() + Duration::from_millis(60);
            }
        } else {
            run.phase_ready = None;
        }
    }
    let complete = run.checked && run.step == run.actions.len() && !pending;
    if complete && let Some(bag) = &bag {
        assert_eq!(
            bag.revision(),
            if run.rewards {
                5
            } else if run.equipment {
                4
            } else {
                6
            }
        );
        let mut totals = std::collections::BTreeMap::new();
        for stack in bag.slots().iter().flatten().chain(bag.main_hand()) {
            *totals.entry(stack.item().as_str()).or_insert(0_u32) += stack.count();
        }
        assert_eq!(totals.get("theia.dev.training-stone"), Some(&28));
        assert_eq!(
            totals.get("theia.dev.training-token"),
            Some(&if run.rewards { 100 } else { 99 })
        );
        if run.rewards {
            assert!(
                world
                    .resource::<Network>()
                    .state()
                    .rewards
                    .as_ref()
                    .is_some_and(|r| r.available.is_empty())
            );
        }
        if run.equipment {
            assert_eq!(totals.get("theia.dev.training-sword"), Some(&1));
            assert_eq!(totals.get("theia.dev.training-wand"), Some(&1));
            assert_eq!(
                bag.main_hand().unwrap().item().as_str(),
                if run.variant == "A" {
                    "theia.dev.training-sword"
                } else {
                    "theia.dev.training-wand"
                }
            );
        } else {
            assert_eq!(bag.slots()[1].as_ref().unwrap().count(), 8);
            assert_eq!(bag.slots()[2].as_ref().unwrap().count(), 20);
            let final_slot = if run.variant == "A" { 5 } else { 6 };
            assert_eq!(bag.slots()[final_slot].as_ref().unwrap().count(), 99);
        }
    }
    if world
        .resource::<crate::capture_ready::CaptureReady>()
        .ready()
    {
        run.ready.get_or_insert_with(Instant::now);
    }
    let mut captured = false;
    let equipment = if run.equipment {
        equipment_observation(
            world,
            Some(if run.variant == "A" {
                "theia.dev.simple-sword"
            } else {
                "theia.dev.simple-wand"
            }),
        )
    } else {
        None
    };
    if complete
        && open
        && run.layout_samples > 10
        && !run.shot
        && !phase_captured
        && (!run.equipment || equipment.is_some())
        && run
            .ready
            .is_some_and(|t| t.elapsed() > Duration::from_millis(500))
    {
        std::fs::create_dir_all(&run.output).unwrap();
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(run.output.join("inventory.png")));
        run.shot = true;
        captured = true;
    }
    if run.written.elapsed() > Duration::from_millis(250) || captured {
        std::fs::create_dir_all(&run.output).unwrap();
        let snapshot = bag
            .as_ref()
            .map(|b| serde_json::from_slice::<serde_json::Value>(&b.to_json()).unwrap());
        let report = serde_json::json!({"rewards":run.rewards,"earned":run.earned,"reward_casts":run.casts,"passed":complete&&run.shot&&run.invalid_layouts==0&&(!run.equipment||equipment.is_some()),"equipment":equipment,"equipment_phases":run.observations,"equipment_metrics":world.get_resource::<crate::equipment::Metrics>(),"variant":run.variant,"resumed":run.resumed,"step":run.step,"steps":run.actions.len(),"pointer_inputs":run.pointer_inputs,"layout_samples":run.layout_samples,"invalid_layouts":run.invalid_layouts,"snapshot":snapshot,"ui":world.resource::<Ui>(),"scope":"Native Bevy picking/widgets and authoritative observations; not operating-system mouse delivery"});
        std::fs::write(
            run.output.join("inventory.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        run.written = Instant::now();
    }
    world.insert_resource(run);
}
