//! Explicit synthetic native input through Bevy keyboard/picking and production
//! widgets. Never enabled during normal play; never uploads a node or counter.
use super::{Config, DialogueText, GuideVisual, Pane, Ui, Widget};
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
#[derive(Resource)]
struct Run {
    started: Instant,
    output: PathBuf,
    variant_b: bool,
    phase: u8,
    stable: u64,
    initial_empty: Option<bool>,
    key_pending: Option<KeyCode>,
    key_release: Option<KeyCode>,
    pointer_pending: Option<usize>,
    pointer_release: Option<usize>,
    key_inputs: u64,
    pointer_inputs: u64,
    layout_samples: u64,
    invalid_layouts: u64,
    text_samples: u64,
    invalid_text: u64,
    max_guide_bodies: usize,
    walk_direction: Vec2,
}
pub(super) fn install(app: &mut App) {
    if std::env::var("THEIA_DIALOGUE_QUALIFY").as_deref() != Ok("1") {
        return;
    }
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").expect("Native dialogue requires output"),
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
        phase: 0,
        stable: 0,
        initial_empty: None,
        key_pending: None,
        key_release: None,
        pointer_pending: None,
        pointer_release: None,
        key_inputs: 0,
        pointer_inputs: 0,
        layout_samples: 0,
        invalid_layouts: 0,
        text_samples: 0,
        invalid_text: 0,
        max_guide_bodies: 0,
        walk_direction: Vec2::NEG_X,
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
        if let Some(key) = run.key_release.or(run.key_pending) {
            let release = run.key_release.is_some();
            let logical_key = match key {
                KeyCode::KeyE => Key::Character("e".into()),
                KeyCode::F1 => Key::F1,
                KeyCode::Escape => Key::Escape,
                _ => unreachable!(),
            };
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
        if let Some(index) = run.pointer_release.or(run.pointer_pending) {
            let location = world
                .query::<(&Widget, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                .iter(world)
                .find(|(widget, node, _)| {
                    matches!(widget,Widget::Choice(i) if *i==index)
                        && node.size().min_element() > 1.
                })
                .map(|(_, node, transform)| {
                    transform.affine().translation * node.inverse_scale_factor()
                });
            if let Some(position) = location {
                let target =
                    bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
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
        }
    }
    world.insert_resource(run);
}
fn advance(run: &mut Run, phase: u8) {
    run.phase = phase;
    run.stable = 0;
}
fn capture(world: &mut World, run: &mut Run, name: &str) -> bool {
    run.stable += 1;
    let path = run.output.join(name);
    if run.stable >= 12 && run.stable % 30 == 12 && !path.is_file() {
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));
    }
    path.is_file()
}
fn drive(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    let mut direction = Vec2::ZERO;
    if state.error.is_none()
        && let (Some(view), Some(guide), Some(actor)) = (
            state.dialogue.as_ref(),
            state.guide.as_ref().and_then(|p| p.guide),
            state.authority_focus(),
        )
    {
        run.initial_empty
            .get_or_insert(view.revision == 0 && view.open.is_none());
        let a = actor.metres();
        let b = guide.position.metres();
        let toward = Vec2::new((b[0] - a[0]) as f32, (b[2] - a[2]) as f32);
        let node = view.open.as_ref().map(|open| open.node.as_str());
        match run.phase {
            0 if run.started.elapsed() >= Duration::from_millis(3200) => {
                if toward.length() > 1.8 {
                    direction = (toward * 0.5).clamp_length_max(1.);
                } else if !state.dialogue_pending
                    && world.resource::<Ui>().visible_guide_frames > 30
                {
                    run.key_pending = Some(KeyCode::KeyE);
                    advance(&mut run, 1);
                }
            }
            1 if node == Some("theia.dev.guide.welcome") && !state.dialogue_pending => {
                if capture(world, &mut run, "dialogue-welcome.png") {
                    run.pointer_pending = Some(usize::from(run.variant_b));
                    advance(&mut run, 2);
                }
            }
            2 if node
                == Some(if run.variant_b {
                    "theia.dev.guide.training"
                } else {
                    "theia.dev.guide.movement"
                })
                && !state.dialogue_pending =>
            {
                if capture(world, &mut run, "dialogue-branch.png") {
                    run.key_pending = Some(KeyCode::F1);
                    advance(&mut run, 3);
                }
            }
            3 if node == Some("theia.dev.guide.goodbye") && !state.dialogue_pending => {
                if capture(world, &mut run, "dialogue-terminal.png") {
                    run.key_pending = Some(KeyCode::Escape);
                    advance(&mut run, 4);
                }
            }
            4 if node.is_none() && !state.dialogue_pending => {
                if capture(world, &mut run, "dialogue-closed.png") {
                    run.key_pending = Some(KeyCode::KeyE);
                    advance(&mut run, 5);
                }
            }
            5 if node == Some("theia.dev.guide.welcome") && !state.dialogue_pending => {
                run.walk_direction = if toward.length() > 0.1 {
                    -toward.normalize()
                } else {
                    Vec2::NEG_X
                };
                advance(&mut run, 6);
            }
            6 => {
                direction = run.walk_direction;
                if toward.length() > 6. && node.is_none() && !state.dialogue_pending {
                    direction = Vec2::ZERO;
                    if capture(world, &mut run, "dialogue-walkaway.png") {
                        advance(&mut run, 7);
                    }
                }
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
    if state.error.is_none()
        && let Some(view) = &state.dialogue
    {
        let ui = world.resource::<Ui>();
        let ui_json = serde_json::to_value(ui).unwrap();
        let open = ui.open;
        let visible = ui.visible_guide_frames;
        let open_frames = ui.open_frames;
        let window = world
            .query_filtered::<&Window, With<bevy::window::PrimaryWindow>>()
            .iter(world)
            .next()
            .map(|w| [w.physical_width(), w.physical_height()])
            .unwrap_or([0, 0]);
        let pane = world
            .query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<Pane>>()
            .iter(world)
            .next()
            .map(|(node, t)| {
                (
                    node.size() * node.inverse_scale_factor(),
                    t.affine().translation * node.inverse_scale_factor(),
                )
            });
        if open && open_frames > 12 {
            run.layout_samples += 1;
            let valid = pane.is_some_and(|(size, center)| {
                size.min_element() > 0.
                    && (center - size * 0.5).min_element() >= -0.5
                    && center.x + size.x * 0.5 <= window[0] as f32 + 0.5
                    && center.y + size.y * 0.5 <= window[1] as f32 + 0.5
            });
            if !valid {
                run.invalid_layouts += 1;
            }
        }
        let bodies = world
            .query_filtered::<Has<avian3d::prelude::RigidBody>, With<GuideVisual>>()
            .iter(world)
            .filter(|body| *body)
            .count();
        run.max_guide_bodies = run.max_guide_bodies.max(bodies);
        let expected_text = view
            .open
            .as_ref()
            .and_then(|open| world.resource::<Config>().0.profile().node(&open.node))
            .map(|node| node.text().to_owned());
        let actual_text = world
            .query_filtered::<&Text, With<DialogueText>>()
            .iter(world)
            .next()
            .map(|text| text.0.clone());
        let text_matches = expected_text.is_none_or(|text| actual_text.as_deref() == Some(&text));
        if open && run.stable >= 8 {
            run.text_samples += 1;
            if !text_matches {
                run.invalid_text += 1;
            }
        }
        let passed = run.phase == 7
            && run.initial_empty == Some(true)
            && run.key_inputs >= 4
            && run.pointer_inputs >= 1
            && run.layout_samples > 10
            && run.invalid_layouts == 0
            && run.text_samples > 5
            && run.invalid_text == 0
            && run.max_guide_bodies == 0
            && visible > 50;
        let report = serde_json::json!({"passed":passed,"phase":run.phase,"initial_closed_revision_zero":run.initial_empty,"variant":if run.variant_b {"B"}else{"A"},"actor":view.actor.entity.value(),"epoch":view.actor.epoch.value(),"profile_hash":view.profile_hash,"revision":view.revision,"open":view.open.is_some(),"window_size":window,"keyboard_events":run.key_inputs,"pointer_inputs":run.pointer_inputs,"layout_samples":run.layout_samples,"invalid_layouts":run.invalid_layouts,"text_samples":run.text_samples,"invalid_text":run.invalid_text,"max_guide_physics_bodies":run.max_guide_bodies,"current_text_matches_observed_node":text_matches,"ui":ui_json,"scope":"Actual observed own-character informational dialogue; synthetic native Bevy keyboard/picking and ordinary movement, no physical OS input or client node upload"});
        let path = run.output.join("dialogue.json");
        let temporary = run.output.join("dialogue.json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        std::fs::rename(temporary, path).unwrap();
    }
    world.insert_resource(run);
}
