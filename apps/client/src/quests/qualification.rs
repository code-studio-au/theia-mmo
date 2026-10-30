//! Explicit owned native qualification; no automation during ordinary play.
use super::{Definitions, Label};
use crate::{movement::MoveIntent, network::Network};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Resource)]
struct Run {
    output: PathBuf,
    resumed: bool,
    started: Option<Instant>,
    next_cast: Instant,
    next_select: Instant,
    casts: u64,
    revision: Option<u64>,
    samples: u64,
    seen: Vec<serde_json::Value>,
    finish_gate: Option<PathBuf>,
}
pub(super) fn install(app: &mut App) {
    if std::env::var("THEIA_QUESTS_QUALIFY").as_deref() != Ok("1") {
        return;
    }
    assert_eq!(
        std::env::var("THEIA_REWARDS_QUALIFY").as_deref(),
        Ok("1"),
        "Objective native fixture uses the existing reward script for its first kill"
    );
    let output = PathBuf::from(
        std::env::var_os("THEIA_GRAYBOX_QUALIFY").expect("Objective qualification requires output"),
    );
    let resumed = !output
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("first-");
    app.insert_resource(Run {
        output,
        resumed,
        started: None,
        next_cast: Instant::now(),
        next_select: Instant::now(),
        casts: 0,
        revision: None,
        samples: 0,
        seen: Vec::new(),
        finish_gate: std::env::var_os("THEIA_QUEST_FINISH_GATE").map(PathBuf::from),
    })
    .add_systems(PreUpdate, drive.after(crate::graybox::autoplay))
    .add_systems(Last, observe);
}
fn drive(
    network: Res<Network>,
    definitions: Res<Definitions>,
    combat: Res<crate::combat::Config>,
    mut run: ResMut<Run>,
    mut movement: ResMut<MoveIntent>,
) {
    let state = network.state();
    if state.error.is_some() || run.resumed {
        return;
    }
    let Some(view) = &state.quest else {
        return;
    };
    let progress = definitions.0.profile().progress(&view.progress).unwrap();
    let started = *run.started.get_or_insert_with(Instant::now);
    // The ordinary reward widget script earns the first life. Continue later
    // lives through the same normal selection/cast path until objective complete.
    if progress.count == 0 || progress.complete || started.elapsed() < Duration::from_millis(4100) {
        return;
    }
    if progress.count == 1 && run.finish_gate.as_ref().is_some_and(|p| !p.is_file()) {
        return;
    }
    let Some(target) = state.targeting.as_ref().and_then(|v| v.targets.first()) else {
        return;
    };
    movement.sprint = false;
    movement.jump = false;
    if let Some(focus) = state.authority_focus() {
        let a = focus.metres();
        let b = target.position.metres();
        movement.direction =
            (Vec2::new((b[0] - 6. - a[0]) as f32, (b[2] - a[2]) as f32) * 0.4).clamp_length_max(1.);
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
    if Instant::now() >= run.next_cast
        && state.combat.as_ref().is_some_and(|c| {
            c.targets
                .iter()
                .any(|h| h.entity == target.key.entity && h.current > 0)
        })
    {
        if network.request_cast(combat.0.abilities()[0].id().clone(), target.key.entity) {
            run.casts += 1;
        }
        run.next_cast = Instant::now() + Duration::from_millis(1700);
    }
}
fn observe(
    mut commands: Commands,
    network: Res<Network>,
    definitions: Res<Definitions>,
    mut run: ResMut<Run>,
    labels: Query<(&Text, &ComputedNode, &bevy::ui::UiGlobalTransform), With<Label>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let state = network.state();
    if state.error.is_some() {
        return;
    }
    let Some(view) = &state.quest else {
        return;
    };
    let progress = definitions.0.profile().progress(&view.progress).unwrap();
    if run.revision != Some(view.progress.revision()) {
        run.revision = Some(view.progress.revision());
        run.samples = 0;
        run.seen.push(serde_json::json!({"revision":view.progress.revision(),"count":progress.count,"xp_total":state.progression.as_ref().map(|s|s.progress.total())}));
        assert!(run.seen.len() <= definitions.0.profile().required() as usize + 1);
    }
    run.samples += 1;
    if run.samples < 12 || run.samples % 30 != 12 {
        return;
    }
    let Some((text, node, transform)) = labels.iter().next() else {
        return;
    };
    let Some(window) = windows.iter().next() else {
        return;
    };
    let expected = format!(
        "{}\n{}: {} / {}",
        definitions.0.profile().name(),
        if progress.complete {
            "Objective complete"
        } else {
            "Defeat creatures"
        },
        progress.count,
        progress.required
    );
    let size = node.size() * node.inverse_scale_factor();
    let centre = transform.affine().translation * node.inverse_scale_factor();
    let low = centre - size * 0.5;
    let high = centre + size * 0.5;
    let valid = text.0 == expected
        && size.min_element() > 0.
        && size.y < 100.
        && low.min_element() >= 0.
        && high.x <= window.width()
        && high.y <= window.height();
    // Bevy accepts one capture per render target/frame. Other fixture panels
    // can request a screenshot at this same revision, so retry this specific
    // count until its actual file exists while the observation remains current.
    if valid
        && !run
            .output
            .join(format!("quest-count-{}.png", progress.count))
            .is_file()
    {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(
                run.output
                    .join(format!("quest-count-{}.png", progress.count)),
            ));
    }
    let report = serde_json::json!({"passed":valid&&progress.complete,"valid":valid,"resumed":run.resumed,"actor":view.actor.entity.value(),"epoch":view.actor.epoch.value(),"profile_hash":view.profile_hash,"revision":view.progress.revision(),"count":progress.count,"required":progress.required,"complete":progress.complete,"text":text.0,"seen":run.seen,"casts":run.casts,"window_size":[window.physical_width(),window.physical_height()],"scope":"Native committed own-character Bevy objective; scripted ordinary cast/selection, no client credit/count upload"});
    let temporary = run.output.join("quests.json.tmp");
    std::fs::write(&temporary, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    std::fs::rename(temporary, run.output.join("quests.json")).unwrap();
}
