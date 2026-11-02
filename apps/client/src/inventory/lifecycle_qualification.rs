//! Explicit native accepted-quest integration. Real Bevy keyboard/picking only;
//! ordinary play installs no driver or direct grant/count mutation path.
use super::{Config, EquipmentLabel, Pane, Ui, Widget};
use crate::{movement::MoveIntent, network::Network};
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Copy)]
enum Target {
    Quest,
    Guide(bool),
    Claim,
    Buy,
    Slot(u8),
    Equip,
}
impl Target {
    fn matches(self, widget: &Widget) -> bool {
        matches!(
            (self, widget),
            (Self::Claim, Widget::Claim)
                | (Self::Buy, Widget::Buy(0))
                | (Self::Equip, Widget::Equip)
        ) || matches!((self,widget),(Self::Slot(a),Widget::Slot(b)) if a==*b)
    }
}
#[derive(Resource)]
struct Run {
    started: Instant,
    output: PathBuf,
    variant_b: bool,
    gate: Option<PathBuf>,
    phase: u8,
    stable: u64,
    bag_target: u64,
    pointer_location: Option<Vec2>,
    key_pending: Option<KeyCode>,
    key_release: Option<KeyCode>,
    pointer_pending: Option<Target>,
    pointer_release: Option<Target>,
    key_inputs: u64,
    pointer_inputs: u64,
    casts: u64,
    dead_input_attempted: bool,
    next_input: Instant,
    initial_revision: Option<u64>,
    initial_closed: Option<bool>,
    seen: Vec<serde_json::Value>,
    lifecycle_seen: Vec<serde_json::Value>,
    dialogue_reply_seen: u64,
    dialogue_rejections: Vec<serde_json::Value>,
    last_dialogue_input: Option<Instant>,
    last_guide_click: Option<Instant>,
    layout_samples: u64,
    invalid_layouts: u64,
    text_samples: u64,
    invalid_text: u64,
    last_write: Instant,
}
pub(super) fn install(app: &mut App) {
    if std::env::var("THEIA_LIFECYCLE_QUALIFY").as_deref() != Ok("1") {
        return;
    }
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").expect("Zone qualification output"),
    );
    let variant_b = output
        .file_name()
        .unwrap()
        .to_string_lossy()
        .ends_with("-b");
    app.insert_resource(Run {
        started: Instant::now(),
        output,
        variant_b,
        gate: std::env::var_os("THEIA_ZONE_COMBAT_GATE").map(PathBuf::from),
        phase: 0,
        stable: 0,
        bag_target: 0,
        pointer_location: None,
        key_pending: None,
        key_release: None,
        pointer_pending: None,
        pointer_release: None,
        key_inputs: 0,
        pointer_inputs: 0,
        casts: 0,
        dead_input_attempted: false,
        next_input: Instant::now(),
        initial_revision: None,
        initial_closed: None,
        seen: Vec::new(),
        lifecycle_seen: Vec::new(),
        dialogue_reply_seen: 0,
        dialogue_rejections: Vec::new(),
        last_dialogue_input: None,
        last_guide_click: None,
        layout_samples: 0,
        invalid_layouts: 0,
        text_samples: 0,
        invalid_text: 0,
        last_write: Instant::now(),
    })
    .add_systems(
        First,
        inputs
            .after(bevy::picking::PickingSystems::Input)
            .before(bevy::picking::PickingSystems::PostInput),
    )
    .add_systems(PreUpdate, drive.after(crate::graybox::autoplay))
    .add_systems(Last, observe);
}
fn inputs(world: &mut World) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    let mut run = world.remove_resource::<Run>().unwrap();
    let window = world
        .query_filtered::<Entity, With<bevy::window::PrimaryWindow>>()
        .iter(world)
        .next();
    if let Some(window) = window {
        if let Some(key) = run.key_release.or(run.key_pending).filter(|key| {
            run.key_release.is_some()
                || !matches!(key, KeyCode::KeyE | KeyCode::KeyF | KeyCode::Escape)
                || run
                    .last_dialogue_input
                    .is_none_or(|t| t.elapsed() >= Duration::from_millis(350))
        }) {
            let release = run.key_release.is_some();
            let logical_key = match key {
                KeyCode::KeyI => Key::Character("i".into()),
                KeyCode::KeyE => Key::Character("e".into()),
                KeyCode::KeyF => Key::Character("f".into()),
                KeyCode::Tab => Key::Tab,
                KeyCode::Digit1 => Key::Character("1".into()),

                KeyCode::Escape => Key::Escape,
                _ => unreachable!(),
            };
            if !release && matches!(key, KeyCode::KeyE | KeyCode::KeyF | KeyCode::Escape) {
                run.last_dialogue_input = Some(Instant::now());
            }
            world.write_message(KeyboardInput {
                key_code: key,
                logical_key,
                state: if release {
                    ButtonState::Released
                } else {
                    ButtonState::Pressed
                },
                text: None,
                repeat: false,
                window,
            });
            if run.key_release.take().is_none() {
                run.key_release = run.key_pending.take();
                run.key_inputs += 1;
            }
        }
        if let Some(target_widget) = run.pointer_release.or(run.pointer_pending) {
            let location = if run.pointer_release.is_some() {
                run.pointer_location
            } else if matches!(target_widget, Target::Guide(_)) {
                crate::world_input::qualification::guide_screen(world)
            } else if matches!(target_widget, Target::Quest) {
                world.query_filtered::<(&ComputedNode,&bevy::ui::UiGlobalTransform),With<crate::quest_lifecycle::ActionButton>>().iter(world)
                    .find(|(node,_)|node.size().min_element()>1.)
                    .map(|(node,transform)|transform.affine().translation*node.inverse_scale_factor())
            } else {
                world
                    .query::<(&Widget, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                    .iter(world)
                    .find(|(widget, node, _)| {
                        target_widget.matches(widget) && node.size().min_element() > 1.
                    })
                    .map(|(_, node, transform)| {
                        transform.affine().translation * node.inverse_scale_factor()
                    })
            };
            if let Some(position) = location {
                let target =
                    bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                        .normalize(Some(window))
                        .unwrap();
                let location = Location { target, position };
                let button = if matches!(target_widget, Target::Guide(true)) {
                    PointerButton::Secondary
                } else {
                    PointerButton::Primary
                };
                let action = if run.pointer_release.is_some() {
                    PointerAction::Release(button)
                } else {
                    PointerAction::Press(button)
                };
                world.write_message(PointerInput::new(
                    PointerId::Mouse,
                    location.clone(),
                    PointerAction::Move { delta: Vec2::ZERO },
                ));
                world.write_message(PointerInput::new(PointerId::Mouse, location, action));
                if run.pointer_release.take().is_none() {
                    run.pointer_location = Some(position);
                    run.pointer_release = run.pointer_pending.take();
                    run.pointer_inputs += 1;
                } else {
                    run.pointer_location = None;
                }
            }
        }
    }
    world.insert_resource(run);
}

fn advance(run: &mut Run, phase: u8) {
    run.phase = phase;
    run.stable = 0;
}
fn capture(world: &mut World, run: &mut Run, name: &str) -> bool {
    let path = run.output.join(name);
    if path.is_file() {
        return true;
    }
    run.stable += 1;
    if run.stable >= 30 && run.stable.is_multiple_of(30) && !path.is_file() {
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
    }
    path.is_file()
}
fn drive(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    if let Some(reply) = state
        .dialogue_reply
        .as_ref()
        .filter(|r| r.sequence > run.dialogue_reply_seen)
    {
        run.dialogue_reply_seen = reply.sequence;
        if let Some(reason) = reply.rejection
            && run.dialogue_rejections.len() < 16
        {
            run.dialogue_rejections.push(serde_json::json!({"phase":run.phase,"sequence":reply.sequence,"reason":format!("{reason:?}")}));
        }
    }
    let mut direction = Vec2::ZERO;
    if state.error.is_none()
        && let (Some(dialogue), Some(guide), Some(actor), Some(items), Some(quest)) = (
            state.dialogue.as_ref(),
            state.guide.as_ref().and_then(|g| g.guide),
            state.authority_focus(),
            state.inventory.as_ref(),
            state.quest_lifecycle.as_ref(),
        )
    {
        run.initial_revision.get_or_insert(items.bag.revision());
        run.initial_closed
            .get_or_insert(dialogue.revision == 0 && dialogue.open.is_none());
        let fresh = run.initial_revision == Some(0);
        let desired = if run.variant_b { 4 } else { 3 };
        let pointer_ready = run.pointer_pending.is_none() && run.pointer_release.is_none();
        let ui_ready = !world.resource::<Ui>().pending && !state.inventory_pending;
        let a = actor.metres();
        let g = guide.position.metres();
        let toward = Vec2::new((g[0] - a[0]) as f32, (g[2] - a[2]) as f32);
        let phase = run.phase;
        let player_life = state.player_life.is_some();
        let waiting_for_death = player_life
            && run.casts > 0
            && world.resource::<crate::combat::Metrics>().player_respawns == 0;
        if phase == 1
            && quest.progress.count() < 3
            && run.gate.as_ref().is_none_or(|p| p.is_file())
            && let Some(target) = state.targeting.as_ref().and_then(|v| v.targets.first())
        {
            let a = actor.metres();
            let b = target.position.metres();
            direction = (Vec2::new(
                (b[0] - if player_life { 2.5 } else { 6. } - a[0]) as f32,
                (b[2] - a[2]) as f32,
            ) * 0.4)
                .clamp_length_max(1.);
        }
        match phase {
            0 if run.started.elapsed() >= Duration::from_millis(3200) => {
                if (!fresh || run.gate.as_ref().is_none_or(|p| p.is_file()))
                    && (!fresh
                        || std::env::var_os("THEIA_QUEST_MARKERS_QUALIFY").is_none()
                        || capture(world, &mut run, "marker-available.png"))
                {
                    advance(&mut run, if fresh { 9 } else { 2 })
                }
            }
            9 => {
                if toward.length() > 1.8 {
                    direction = (toward * 0.5).clamp_length_max(1.);
                } else if dialogue.open.is_none()
                    && !state.dialogue_pending
                    && run.key_pending.is_none()
                    && run.key_release.is_none()
                {
                    if std::env::var_os("THEIA_WORLD_INPUT_QUALIFY").is_some() {
                        if run.pointer_pending.is_none()
                            && run.pointer_release.is_none()
                            && run
                                .last_guide_click
                                .is_none_or(|t| t.elapsed() > Duration::from_millis(500))
                        {
                            let selected = world
                                .resource::<crate::world_input::Selection>()
                                .friendly
                                .is_some();
                            run.pointer_pending = Some(Target::Guide(selected));
                            run.last_guide_click = Some(Instant::now());
                        }
                    } else {
                        run.key_pending =
                            Some(if std::env::var_os("THEIA_WORLD_INPUT_QUALIFY").is_some() {
                                KeyCode::KeyF
                            } else {
                                KeyCode::KeyE
                            });
                    }
                } else if dialogue.open.is_some()
                    && !state.dialogue_pending
                    && !world.resource::<crate::quest_lifecycle::Ui>().pending
                    && capture(world, &mut run, "life-offer.png")
                {
                    run.pointer_pending = Some(Target::Quest);
                    advance(&mut run, 10);
                }
            }
            10 if quest.progress.status() == theia_core::quests::lifecycle::Status::Accepted
                && quest.progress.count() == 0
                && !state.quest_lifecycle_pending
                && !world.resource::<crate::quest_lifecycle::Ui>().pending
                && pointer_ready
                && capture(world, &mut run, "life-accepted.png") =>
            {
                run.key_pending = Some(KeyCode::Escape);
                advance(&mut run, 11);
            }
            11 if dialogue.open.is_none()
                && !state.dialogue_pending
                && run.key_pending.is_none()
                && run.key_release.is_none() =>
            {
                if world.resource::<Ui>().open {
                    run.key_pending = Some(KeyCode::KeyI);
                } else if std::env::var_os("THEIA_QUEST_MARKERS_QUALIFY").is_none()
                    || capture(world, &mut run, "marker-active.png")
                {
                    advance(&mut run, 1);
                }
            }
            1 => {
                if waiting_for_death {
                    direction = Vec2::ZERO;
                    if state
                        .player_life
                        .as_ref()
                        .and_then(|life| life.current())
                        .is_some_and(|v| v.health == 0)
                    {
                        // Attempt real input while dead; the normal prediction
                        // and action paths must suppress it.
                        direction = Vec2::X;
                        if !run.dead_input_attempted
                            && run.key_release.is_none()
                            && run.key_pending.is_none()
                        {
                            run.key_pending = Some(KeyCode::Digit1);
                            run.dead_input_attempted = true;
                        }
                        let pose_ready = world
                            .get_resource::<crate::reference_scene::LifeAnimationMetrics>()
                            .is_none_or(|m| !m.available || m.death_finished_frames > 0);
                        if pose_ready {
                            capture(world, &mut run, "player-dead.png");
                        }
                    }
                } else if quest.progress.count() == 3 {
                    if capture(world, &mut run, "zone-objective.png")
                        && (std::env::var_os("THEIA_QUEST_MARKERS_QUALIFY").is_none()
                            || capture(world, &mut run, "marker-ready.png"))
                    {
                        advance(&mut run, 2);
                    }
                } else if run.gate.as_ref().is_none_or(|p| p.is_file())
                    && Instant::now() >= run.next_input
                    && run.key_release.is_none()
                    && run.key_pending.is_none()
                    && world.resource::<Network>().can_act()
                    && let Some(target) = state.targeting.as_ref().and_then(|v| v.targets.first())
                {
                    if state
                        .targeting
                        .as_ref()
                        .is_some_and(|v| v.selected != Some(target.key.entity))
                    {
                        run.key_pending = Some(KeyCode::Tab);
                        run.next_input = Instant::now() + Duration::from_millis(500);
                    } else if (!player_life
                        || (target.position.metres()[0] - a[0])
                            .hypot(target.position.metres()[2] - a[2])
                            < 2.9)
                        && state.combat.as_ref().is_some_and(|c| {
                            c.targets
                                .iter()
                                .any(|h| h.entity == target.key.entity && h.current > 0)
                        })
                    {
                        run.key_pending = Some(KeyCode::Digit1);
                        run.casts += 1;
                        run.next_input = Instant::now()
                            + Duration::from_millis(if player_life { 250 } else { 1700 });
                    }
                }
            }
            2 => {
                if toward.length() > 1.8 {
                    direction = (toward * 0.5).clamp_length_max(1.);
                } else if dialogue.open.is_none()
                    && !world.resource::<crate::dialogue::Ui>().pending
                    && !state.dialogue_pending
                    && run.key_pending.is_none()
                    && run.key_release.is_none()
                {
                    run.key_pending =
                        Some(if std::env::var_os("THEIA_WORLD_INPUT_QUALIFY").is_some() {
                            KeyCode::KeyF
                        } else {
                            KeyCode::KeyE
                        });
                } else if dialogue.open.is_some()
                    && !state.dialogue_pending
                    && world.resource::<Ui>().open
                    && capture(world, &mut run, "zone-dialogue.png")
                {
                    if fresh {
                        run.bag_target = 1;
                        run.pointer_pending = Some(Target::Claim);
                        advance(&mut run, 3);
                    } else {
                        advance(&mut run, 6);
                    }
                }
            }
            3 if items.bag.revision() == 1
                && ui_ready
                && pointer_ready
                && state
                    .rewards
                    .as_ref()
                    .is_some_and(|r| r.available.is_empty()) =>
            {
                if capture(world, &mut run, "zone-reward.png") {
                    run.bag_target = 2;
                    run.pointer_pending = Some(Target::Buy);
                    advance(&mut run, 4);
                }
            }
            4 if ui_ready && pointer_ready && items.bag.revision() == run.bag_target => {
                if items.bag.revision() < desired - 1 {
                    run.bag_target += 1;
                    run.pointer_pending = Some(Target::Buy);
                } else if capture(world, &mut run, "zone-trade.png") {
                    run.pointer_pending = Some(Target::Slot(if run.variant_b { 4 } else { 3 }));
                    advance(&mut run, 5);
                }
            }
            5 if pointer_ready
                && ui_ready
                && world.resource::<Ui>().selected == Some(if run.variant_b { 4 } else { 3 }) =>
            {
                run.pointer_pending = Some(Target::Equip);
                advance(&mut run, 6);
            }
            6 if items.bag.revision() == desired
                && items.bag.main_hand().is_some()
                && ui_ready
                && pointer_ready =>
            {
                if fresh
                    && quest.progress.status() != theia_core::quests::lifecycle::Status::TurnedIn
                {
                    if capture(world, &mut run, "life-ready.png") {
                        run.pointer_pending = Some(Target::Quest);
                        advance(&mut run, 12);
                    }
                } else if capture(world, &mut run, "zone-complete.png") {
                    run.key_pending = Some(KeyCode::Escape);
                    advance(&mut run, 7);
                }
            }
            12 if quest.progress.status() == theia_core::quests::lifecycle::Status::TurnedIn
                && !state.quest_lifecycle_pending
                && !world.resource::<crate::quest_lifecycle::Ui>().pending
                && pointer_ready
                && capture(world, &mut run, "life-turned-in.png") =>
            {
                advance(&mut run, 6);
            }
            7 if dialogue.open.is_none()
                && !state.dialogue_pending
                && capture(world, &mut run, "zone-closed.png") =>
            {
                advance(&mut run, 8)
            }
            _ => {}
        }
    }
    let mut movement = world.resource_mut::<MoveIntent>();
    movement.direction = direction;
    movement.sprint = false;
    movement.jump = false;
    world.insert_resource(run);
}
fn observe(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    let window = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(|w| Vec2::new(w.width(), w.height()));
    if state.error.is_none()
        && let (Some(quest), Some(xp), Some(items)) = (
            state.quest_lifecycle.as_ref(),
            state.progression.as_ref(),
            state.inventory.as_ref(),
        )
    {
        let quest_def = world
            .resource::<crate::quest_lifecycle::Definitions>()
            .0
            .clone();
        let progress = quest.progress.progress(quest_def.profile()).unwrap();
        if run
            .lifecycle_seen
            .last()
            .and_then(|v| v["revision"].as_u64())
            != Some(quest.progress.revision())
        {
            run.lifecycle_seen.push(serde_json::json!({"revision":quest.progress.revision(),"count":progress.count,"phase":progress.phase,"xp_total":xp.progress.total(),"xp_revision":xp.progress.revision()}));
        }
        let xp_profile = &world.resource::<crate::progression::Profile>().0;
        let xp_progress = xp_profile.profile().progress(&xp.progress).unwrap();
        let expected_xp = (progress.count * 25).min(xp_profile.profile().maximum_experience());
        if run.seen.last().and_then(|v| v["count"].as_u64()) != Some(u64::from(progress.count))
            && xp.progress.total() == expected_xp
        {
            run.seen
                .push(serde_json::json!({"count":progress.count,"xp_total":xp.progress.total()}));
        }
        let expected_quest = format!("{}\nTraining complete", quest_def.profile().name());
        let expected_xp = match xp_progress.required_in_level {
            Some(n) => format!(
                "Level {} | XP {} / {}",
                xp_progress.level, xp_progress.earned_in_level, n
            ),
            None => format!("Level {} | maximum level", xp_progress.level),
        };
        let expected_hand = items
            .bag
            .main_hand()
            .and_then(|s| world.resource::<Config>().0.catalog().definition(s.item()))
            .map(|d| format!("Main hand: {}", d.name()));
        if quest.progress.status() == theia_core::quests::lifecycle::Status::TurnedIn
            && items.bag.main_hand().is_some()
            && (run.stable >= 12 || run.phase == 8)
            && !world.resource::<Ui>().pending
        {
            let quest_text = world
                .query_filtered::<&Text, With<crate::quest_lifecycle::Label>>()
                .iter(world)
                .any(|t| t.0 == expected_quest);
            let xp_text = world
                .query_filtered::<&Text, With<crate::progression::Label>>()
                .iter(world)
                .any(|t| t.0 == expected_xp);
            let hand_text = world
                .query_filtered::<&Text, With<EquipmentLabel>>()
                .iter(world)
                .any(|t| expected_hand.as_ref() == Some(&t.0));
            let reward_text = if let Some(recipe) = quest_def.turn_in_xp() {
                let expected = format!(
                    "Completion reward: {} XP (level cap applies)",
                    recipe.experience()
                );
                world
                    .query_filtered::<&Text, With<crate::quest_lifecycle::RewardLabel>>()
                    .iter(world)
                    .any(|t| t.0 == expected)
            } else {
                true
            };
            run.text_samples += 1;
            if !quest_text || !xp_text || !hand_text || !reward_text {
                run.invalid_text += 1;
            }
        }
        if let Some(size) = window
            && world.resource::<Ui>().open
        {
            for (node, transform) in world
                .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Pane>>()
                .iter(world)
            {
                if node.size().min_element() > 1. {
                    run.layout_samples += 1;
                    let half = node.size() * node.inverse_scale_factor() * 0.5;
                    let center = transform.affine().translation * node.inverse_scale_factor();
                    if (center - half).min_element() < -1.
                        || (center + half - size).max_element() > 1.
                    {
                        run.invalid_layouts += 1;
                    }
                }
            }
        }
        if run.last_write.elapsed() >= Duration::from_millis(250) {
            let ui = world.resource::<Ui>();
            let desired = if run.variant_b { 4 } else { 3 };
            let hand = items.bag.main_hand().map(|s| s.item().as_str());
            let passed = run.phase == 8
                && run.initial_closed == Some(true)
                && progress.phase == theia_core::quests::lifecycle::Phase::TurnedIn
                && !world.resource::<crate::quest_lifecycle::Ui>().pending
                && xp.progress.total()
                    == if quest_def.turn_in_xp().is_some() {
                        100
                    } else {
                        50
                    }
                && items.bag.revision() == desired
                && hand
                    == Some(if run.variant_b {
                        "theia.dev.training-wand"
                    } else {
                        "theia.dev.training-sword"
                    })
                && !ui.pending
                && run.invalid_layouts == 0
                && run.layout_samples > 10
                && run.text_samples > 10
                && run.invalid_text == 0;
            let report = serde_json::json!({"dialogue_rejections":run.dialogue_rejections,"world_input":world.get_resource::<crate::world_input::Selection>(),"quest_marker":world.get_resource::<crate::quest_marker::Metrics>(),"player_animation":world.get_resource::<crate::reference_scene::LifeAnimationMetrics>(),"equipment_metrics":world.get_resource::<crate::equipment::Metrics>(),"creature_visuals":world.get_resource::<crate::creature_visual::Pending>().map(|p| &p.metrics),"combat_metrics":world.resource::<crate::combat::Metrics>(),"combat_ui":world.resource::<crate::combat::ui::Model>(),"dead_input_attempted":run.dead_input_attempted,"passed":passed,"phase":run.phase,"initial_closed":run.initial_closed,"resumed":run.initial_revision.is_some_and(|r|r>0),"initial_revision":run.initial_revision,"revision":items.bag.revision(),"snapshot":serde_json::from_slice::<serde_json::Value>(&items.bag.to_json()).unwrap(),"ui":ui,"lifecycle_ui":world.resource::<crate::quest_lifecycle::Ui>(),"lifecycle_seen":run.lifecycle_seen,"pointer_inputs":run.pointer_inputs,"key_inputs":run.key_inputs,"casts":run.casts,"layout_samples":run.layout_samples,"invalid_layouts":run.invalid_layouts,"text_samples":run.text_samples,"invalid_text":run.invalid_text,"window_size":window.map(|s|[s.x as u32,s.y as u32]),"quest":{"passed":passed,"valid":true,"count":progress.count,"revision":quest.progress.revision(),"complete":progress.phase==theia_core::quests::lifecycle::Phase::TurnedIn,"phase":progress.phase,"profile_hash":quest.profile_hash,"seen":run.seen,"casts":run.casts,"window_size":window.map(|s|[s.x as u32,s.y as u32])},"xp":{"total":xp.progress.total(),"revision":xp.progress.revision(),"level":xp_progress.level}});
            std::fs::write(
                run.output.join("zone.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            run.last_write = Instant::now();
        }
    }
    world.insert_resource(run);
}
