//! Explicit native qualification only; ordinary play installs no automation.
use super::{Assets as TargetAssets, TargetVisual};
use crate::{floating_origin::Frame, network::Network};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Run {
    output: PathBuf,
    phase: u8,
    changed: Instant,
    written: Instant,
    samples: u64,
    visible: u64,
    selected: u64,
    cleared: u64,
    empty_after_selection: u64,
    returned: u64,
    max_error: f64,
    visible_moving_samples: u64,
    steady_follow_direction: Option<Vec2>,
    previous_displayed: std::collections::BTreeMap<u64, theia_core::coordinates::GlobalPosition>,
    physics_bodies: u64,
    max_visuals: usize,
    revisions: std::collections::BTreeSet<u64>,
    combat: bool,
    feedback_rebased: bool,
    next_cast: Instant,
    action_ui: bool,
    input_count: u64,
    pointer_pending: bool,
    pointer_release: bool,
    pointer_inputs: u64,
    last_select: Instant,
    combat_shots: std::collections::BTreeSet<&'static str>,
}
pub(super) fn install(app: &mut App) {
    if std::env::var("THEIA_TARGETING_QUALIFY").as_deref() != Ok("1") {
        return;
    }
    let output = std::env::var_os("THEIA_GRAYBOX_QUALIFY")
        .map(PathBuf::from)
        .expect("Target qualification requires graybox output");
    app.insert_resource(Run {
        output,
        phase: 0,
        changed: Instant::now(),
        written: Instant::now(),
        samples: 0,
        visible: 0,
        selected: 0,
        cleared: 0,
        empty_after_selection: 0,
        returned: 0,
        max_error: 0.,
        visible_moving_samples: 0,
        steady_follow_direction: None,
        previous_displayed: std::collections::BTreeMap::new(),
        physics_bodies: 0,
        max_visuals: 0,
        revisions: Default::default(),
        combat: std::env::var("THEIA_COMBAT_QUALIFY").as_deref() == Ok("1"),
        feedback_rebased: false,
        next_cast: Instant::now(),
        action_ui: std::env::var("THEIA_COMBAT_UI_QUALIFY").as_deref() == Ok("1"),
        input_count: 0,
        pointer_pending: false,
        pointer_release: false,
        pointer_inputs: 0,
        last_select: Instant::now(),
        combat_shots: Default::default(),
    })
    .add_systems(
        First,
        pointer_input
            .after(bevy::picking::PickingSystems::Input)
            .before(bevy::picking::PickingSystems::PostInput),
    )
    .add_systems(PreUpdate, input.after(crate::graybox::autoplay))
    .add_systems(Last, observe);
}
// Synthetic native pointer messages run through Bevy picking, UI hit testing,
// widget observers and the ordinary request path. This is not OS mouse delivery.
fn pointer_input(world: &mut World) {
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    let mut run = world.remove_resource::<Run>().unwrap();
    if run.action_ui && (run.pointer_pending || run.pointer_release) {
        let button = world.query_filtered::<(&ComputedNode, &bevy::ui::UiGlobalTransform), With<crate::combat::ui::ActionButton>>()
            .iter(world).next().map(|(node, transform)| (node.size(), transform.affine().translation * node.inverse_scale_factor()));
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
            let action = if run.pointer_release {
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
            if run.pointer_release {
                run.pointer_release = false;
            } else {
                run.pointer_pending = false;
                run.pointer_release = true;
                run.pointer_inputs += 1;
            }
        }
    }
    world.insert_resource(run);
}
fn input(
    network: Res<Network>,
    mut run: ResMut<Run>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut movement: ResMut<crate::movement::MoveIntent>,
    mut frame: ResMut<Frame>,
    sinks: Query<&SpatialAudioSink>,
) {
    // Keep the screenshot subject in view while the actual selection round trips.
    // Continue the ordinary traversal/tour fixture once selection is verified.
    if run.phase < 6 {
        movement.direction = Vec2::ZERO;
        movement.jump = false;
        movement.sprint = false;
    }
    keys.release(KeyCode::Tab);
    keys.release(KeyCode::Backspace);
    let state = network.state();
    if !run.feedback_rebased
        && std::env::var("THEIA_FEEDBACK_QUALIFY").as_deref() == Ok("1")
        && !sinks.is_empty()
        && let Some(focus) = state.authority_focus()
    {
        let mut origin = frame.render.origin().metres();
        origin[0] += if focus.metres()[0] >= origin[0] {
            128.
        } else {
            -128.
        };
        frame.render = theia_core::coordinates::LocalFrame::new(
            theia_core::coordinates::GlobalPosition::new(origin).unwrap(),
        );
        frame.revision += 1;
        run.feedback_rebased = true;
    }
    let Some(view) = &state.targeting else {
        return;
    };
    let ready = view.targets.iter().any(|t| {
        t.position.chunk(128).is_some_and(|id| {
            state
                .chunk_report
                .as_ref()
                .is_some_and(|r| r.chunks.contains(&[id.x, id.z]))
        })
    });
    let key = match run.phase {
        0 if ready => Some(KeyCode::Tab),
        2 if run.changed.elapsed().as_secs_f32() > 1. => Some(KeyCode::Backspace),
        4 if run.changed.elapsed().as_secs_f32() > 1. => Some(KeyCode::Tab),
        _ => None,
    };
    if let Some(key) = key {
        keys.press(key);
        run.phase += 1;
        run.changed = Instant::now();
    }
    if run.combat {
        keys.release(KeyCode::Digit1);
        if run.phase >= 6 {
            // A balanced, bounded route keeps the NPC in range while proving
            // walking/running/jumping casts through the real movement owner.
            let t = run.changed.elapsed().as_secs_f64() % 8.;
            movement.direction = match t {
                t if t < 2. => Vec2::X,
                t if t < 4. => -Vec2::X,
                t if t < 5. => Vec2::X,
                t if t < 6. => -Vec2::X,
                _ => Vec2::ZERO,
            };
            movement.sprint = (4. ..6.).contains(&t);
            movement.jump = (6. ..6.2).contains(&t);
            if std::env::var("THEIA_CREATURE_QUALIFY").as_deref() == Ok("1") {
                movement.direction = Vec2::ZERO;
                if let Some(focus) = state.authority_focus()
                    && let Some(target) = view
                        .targets
                        .iter()
                        .find(|target| Some(target.key.entity) == view.selected)
                        .or_else(|| view.targets.first())
                {
                    let a = focus.metres();
                    let b = target.position.metres();
                    let offset = Vec2::new((b[0] - 6. - a[0]) as f32, (b[2] - a[2]) as f32);
                    movement.direction = (offset * 0.4).clamp_length_max(1.);
                    // A short constant-input segment retains the existing
                    // steady-motion checks before returning to closed-loop follow.
                    let elapsed = run.changed.elapsed().as_secs_f64();
                    if (8. ..12.).contains(&elapsed) {
                        let direction = *run
                            .steady_follow_direction
                            .get_or_insert(movement.direction);
                        movement.direction = direction;
                        movement.sprint = false;
                        movement.jump = false;
                    }
                    // Exercise an actual streamed boundary and return while
                    // the home creature continues its independent patrol.
                    if (20. ..30.).contains(&elapsed) {
                        movement.direction = Vec2::X;
                        movement.sprint = true;
                        movement.jump = false;
                    } else if (30. ..40.).contains(&elapsed) {
                        movement.direction = -Vec2::X;
                        movement.sprint = true;
                        movement.jump = false;
                    }
                } else {
                    movement.jump = false;
                    movement.sprint = false;
                }
            }
            if ready && view.selected.is_none() && run.last_select.elapsed().as_secs_f64() > 0.5 {
                keys.press(KeyCode::Tab);
                run.last_select = Instant::now();
            }
            let alive = state.combat.as_ref().is_some_and(|c| {
                c.targets
                    .iter()
                    .any(|h| Some(h.entity) == view.selected && h.current > 0)
            });
            // Allow the actual baked loop/waypoint waits before damaging it.
            let patrol_ready = std::env::var("THEIA_CREATURE_QUALIFY").as_deref() != Ok("1")
                || run.changed.elapsed().as_secs_f64() >= 72.;
            if alive && patrol_ready && Instant::now() >= run.next_cast {
                if run.action_ui && run.input_count % 2 == 1 {
                    run.pointer_pending = true;
                } else {
                    keys.press(KeyCode::Digit1);
                }
                run.input_count += 1;
                run.next_cast = Instant::now() + std::time::Duration::from_millis(2200);
            }
        }
    }
}
fn observe(world: &mut World) {
    let mut run = world.remove_resource::<Run>().unwrap();
    let state = world.resource::<Network>().state();
    let render = world.resource::<Frame>().render;
    let revision = world.resource::<Frame>().revision;
    let selected_material = world.resource::<TargetAssets>().selected.clone();
    run.previous_displayed.retain(|id, _| {
        state
            .targeting
            .as_ref()
            .is_some_and(|v| v.targets.iter().any(|t| t.key.entity.value() == *id))
    });

    let mut visible_selected = false;
    let mut visible_clear = false;
    let mut count = 0;
    if let Some(view) = &state.targeting {
        for (visual, transform, shown, visible, material, body, motion) in world
            .query::<(
                &TargetVisual,
                &Transform,
                &Visibility,
                &ViewVisibility,
                &MeshMaterial3d<StandardMaterial>,
                Option<&avian3d::prelude::Collider>,
                Option<&super::motion::TargetMotion>,
            )>()
            .iter(world)
        {
            count += 1;
            run.physics_bodies += u64::from(body.is_some());
            let Some(target) = view.targets.iter().find(|t| t.key == visual.key) else {
                continue;
            };
            if *shown == Visibility::Hidden {
                continue;
            }
            let expected = render
                .to_local(motion.map_or(target.position, |m| m.global()))
                .expect("Visible target in bounded frame");
            let error = transform
                .translation
                .distance(Vec3::from_array(expected.metres()));
            run.max_error = run.max_error.max(f64::from(error));
            run.samples += 1;
            run.revisions.insert(revision);
            let displayed = render
                .to_global(
                    theia_core::coordinates::LocalPosition::new(transform.translation.to_array())
                        .expect("Bounded displayed target"),
                )
                .expect("Checked displayed target global");
            let previous = run
                .previous_displayed
                .insert(visual.key.entity.value(), displayed);
            if visible.get() {
                if motion.is_some()
                    && previous.is_some_and(|p| {
                        let a = p.metres();
                        let b = displayed.metres();
                        (a[0] - b[0]).hypot(a[2] - b[2]) > 0.00001
                    })
                {
                    run.visible_moving_samples += 1;
                }
                run.visible += 1;
                if view.selected == Some(visual.key.entity) && material.0 == selected_material {
                    run.selected += 1;
                    visible_selected = true;
                }
                if view.selected.is_none() && material.0 != selected_material {
                    run.cleared += 1;
                    visible_clear = true;
                }
            }
        }
        if run.phase >= 6 {
            if view.targets.is_empty() && count == 0 {
                run.empty_after_selection += 1;
            } else if !view.targets.is_empty() && count > 0 && run.empty_after_selection > 0 {
                run.returned += 1;
            }
        }
    }
    run.max_visuals = run.max_visuals.max(count);
    let ready = world
        .resource::<crate::capture_ready::CaptureReady>()
        .ready();
    let shot = match run.phase {
        1 if ready && visible_selected => Some("selected.png"),
        3 if ready && visible_clear => Some("cleared.png"),
        5 if ready && visible_selected => Some("reselected.png"),
        _ => None,
    };
    if let Some(name) = shot {
        std::fs::create_dir_all(&run.output).unwrap();
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(run.output.join(name)));
        run.phase += 1;
        run.changed = Instant::now();
    }
    let mut combat_shots = Vec::new();
    if ready
        && run.combat
        && let Some(metrics) = world.get_resource::<crate::combat::Metrics>()
    {
        for (name, shown) in [
            (
                "mana-depleted.png",
                metrics.mana_rejections > 0 && metrics.last_mana.is_some_and(|m| m < 25),
            ),
            (
                "mana-recovered.png",
                metrics.mana_rejections > 0 && metrics.last_mana.is_some_and(|m| m >= 25),
            ),
            (
                "damaged.png",
                visible_selected
                    && metrics
                        .last_health
                        .is_some_and(|hp| hp > 0 && hp < metrics.max_health),
            ),
            (
                "defeated.png",
                count == 0 && metrics.deaths > 0 && metrics.defeated_frames > 0,
            ),
            (
                "respawned.png",
                visible_selected
                    && metrics.respawns > 0
                    && metrics.last_health == Some(metrics.max_health),
            ),
        ] {
            if shown && !run.combat_shots.contains(name) {
                combat_shots.push(name);
            }
        }
    }
    if ready && let Some(metrics) = world.get_resource::<crate::status_effects::Metrics>() {
        let active = world.get_resource::<Network>().is_some_and(|network| {
            let state = network.state();
            let selected = state.targeting.as_ref().and_then(|v| v.selected);
            state
                .statuses
                .as_ref()
                .is_some_and(|v| v.statuses.iter().any(|s| Some(s.target.entity) == selected))
        });
        for (name, shown) in [
            ("statuses-active.png", active),
            (
                "statuses-cleared.png",
                !active && metrics.active_frames > 100,
            ),
        ] {
            if shown && !run.combat_shots.contains(name) {
                combat_shots.push(name);
            }
        }
    }
    if ready
        && crate::spell_feedback::critical_visible(world)
        && !run.combat_shots.contains("feedback.png")
    {
        combat_shots.push("feedback.png");
    }
    // Bevy accepts one screenshot per render target per frame. Leave other
    // conditions pending so they can capture a later visible frame.
    for name in combat_shots.into_iter().take(1) {
        run.combat_shots.insert(name);
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(run.output.join(name)));
    }
    if run.written.elapsed().as_secs_f32() >= 1. || shot.is_some() {
        std::fs::create_dir_all(&run.output).unwrap();
        let report = serde_json::json!({"phase":run.phase,"samples":run.samples,
            "visible_samples":run.visible,"selected_samples":run.selected,
            "cleared_samples":run.cleared,"empty_after_selection":run.empty_after_selection,
            "returned_samples":run.returned,"max_position_error_m":run.max_error,
            "physics_body_samples":run.physics_bodies,"max_visuals":run.max_visuals,
            "observed_revisions":run.revisions,"combat_shots":run.combat_shots,
            "visible_moving_samples":run.visible_moving_samples,
            "target_motion":world.get_resource::<super::motion::Metrics>(),
            "statuses":world.get_resource::<crate::status_effects::Metrics>(),
            "action_ui":world.get_resource::<crate::combat::ui::Model>(),
            "ui_metrics":world.get_resource::<crate::combat::ui::Metrics>(),
            "pointer_inputs":run.pointer_inputs,
            "feedback_rebased":run.feedback_rebased,
            "feedback":world.get_resource::<crate::spell_feedback::Metrics>(),
            "combat":world.get_resource::<crate::combat::Metrics>()});
        std::fs::write(
            run.output.join("targeting.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        run.written = Instant::now();
    }
    world.insert_resource(run);
}
