//! Explicit synthetic native shop input; normal play installs no automation.
use super::{BuyButton, Pane, Ui};
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
    key_pending: Option<KeyCode>,
    key_release: Option<KeyCode>,
    pointer_pending: Option<usize>,
    pointer_release: Option<usize>,
    key_inputs: u64,
    pointer_inputs: u64,
    initial_revision: Option<u64>,
    initial_closed: Option<bool>,
    layout_samples: u64,
    invalid_layouts: u64,
}
pub(super) fn install(app: &mut App) {
    if std::env::var("THEIA_VENDOR_QUALIFY").as_deref() != Ok("1") {
        return;
    }
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").expect("Vendor qualification output"),
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
        key_pending: None,
        key_release: None,
        pointer_pending: None,
        pointer_release: None,
        key_inputs: 0,
        pointer_inputs: 0,
        initial_revision: None,
        initial_closed: None,
        layout_samples: 0,
        invalid_layouts: 0,
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
                .query::<(&BuyButton, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                .iter(world)
                .find(|(widget, node, _)| widget.0 == index as u8 && node.size().min_element() > 1.)
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
fn advance(run: &mut Run, phase: u8) {
    run.phase = phase;
    run.stable = 0;
}
fn drive(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    let mut direction = Vec2::ZERO;
    if state.error.is_none()
        && let (Some(view), Some(guide), Some(actor), Some(items)) = (
            state.dialogue.as_ref(),
            state.guide.as_ref().and_then(|g| g.guide),
            state.authority_focus(),
            state.inventory.as_ref(),
        )
    {
        run.initial_revision.get_or_insert(items.bag.revision());
        run.initial_closed
            .get_or_insert(view.revision == 0 && view.open.is_none());
        let a = actor.metres();
        let g = guide.position.metres();
        let toward = Vec2::new((g[0] - a[0]) as f32, (g[2] - a[2]) as f32);
        let desired = if run.variant_b { 2 } else { 1 };
        let phase = run.phase;
        match phase {
            0 if run.started.elapsed() >= Duration::from_millis(3200) => {
                if toward.length() > 1.8 {
                    direction = (toward * 0.5).clamp_length_max(1.);
                } else if !state.dialogue_pending {
                    run.key_pending = Some(KeyCode::KeyE);
                    advance(&mut run, 1);
                }
            }
            1 if view.open.is_some() && !state.dialogue_pending && world.resource::<Ui>().open => {
                if capture(world, &mut run, "vendor-offer.png") {
                    if items.bag.revision() < desired {
                        run.pointer_pending = Some(0);
                        advance(&mut run, 2);
                    } else {
                        advance(&mut run, 3);
                    }
                }
            }
            2 if !world.resource::<Ui>().pending
                && !state.inventory_pending
                && items.bag.revision() >= run.pointer_inputs =>
            {
                if run.pointer_release.is_none()
                    && run.pointer_pending.is_none()
                    && run.pointer_inputs > 0
                {
                    if items.bag.revision() < desired {
                        run.pointer_pending = Some(0);
                    } else {
                        advance(&mut run, 3);
                    }
                }
            }
            3 if items.bag.revision() == desired && !state.inventory_pending => {
                if capture(world, &mut run, "vendor-committed.png") {
                    run.key_pending = Some(KeyCode::Escape);
                    advance(&mut run, 4);
                }
            }
            4 if view.open.is_none()
                && !state.dialogue_pending
                && capture(world, &mut run, "vendor-closed.png") =>
            {
                advance(&mut run, 5);
            }
            _ => {}
        }
    }
    if let Some(mut intent) = world.get_resource_mut::<MoveIntent>() {
        intent.direction = direction;
        intent.sprint = false;
        intent.jump = false;
    }
    world.insert_resource(run);
}
fn observe(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let window = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(|w| Vec2::new(w.width(), w.height()));
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
                if (center - half).min_element() < -1. || (center + half - size).max_element() > 1.
                {
                    run.invalid_layouts += 1;
                }
            }
        }
    }
    if run.phase == 5 {
        let ui = world.resource::<Ui>();
        let bag = ui.bag.as_ref().expect("Committed native bag");
        let desired = if run.variant_b { 2 } else { 1 };
        let passed = run.initial_closed == Some(true)
            && bag.revision() == desired
            && run.invalid_layouts == 0
            && run.layout_samples > 10
            && run.key_inputs == 2
            && !ui.pending
            && !ui.vendor_open;
        let report = serde_json::json!({"passed":passed,"phase":run.phase,"initial_closed":run.initial_closed,"initial_revision":run.initial_revision,"revision":bag.revision(),"snapshot":serde_json::from_slice::<serde_json::Value>(&bag.to_json()).unwrap(),"ui":ui,"pointer_inputs":run.pointer_inputs,"key_inputs":run.key_inputs,"layout_samples":run.layout_samples,"invalid_layouts":run.invalid_layouts,"window_size":window.map(|s|[s.x as u32,s.y as u32])});
        std::fs::write(
            run.output.join("vendor.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    world.insert_resource(run);
}
