//! Real-time movement/animation capture for the optional local reference scene.
use crate::{
    movement::{Gait, LocalPawn},
    reference_scene::{self, Visual},
};
use bevy::{
    prelude::*,
    render::view::window::screenshot::{Screenshot, ScreenshotCaptured},
};
use std::{collections::BTreeMap, path::PathBuf, time::Instant};

#[derive(Resource)]
pub struct Run {
    output: PathBuf,
    started: Instant,
    ready: Option<Instant>,
    samples: BTreeMap<String, Vec<(Entity, Transform)>>,
    changes: BTreeMap<String, usize>,
    closest_wolf: f32,
    requested: bool,
}
impl Run {
    pub fn new(output: PathBuf) -> Self {
        Self {
            output,
            started: Instant::now(),
            ready: None,
            samples: BTreeMap::new(),
            changes: BTreeMap::new(),
            closest_wolf: f32::INFINITY,
            requested: false,
        }
    }
    pub fn intent(&self) -> (Vec2, bool) {
        let elapsed = self.ready.map_or(0., |t| t.elapsed().as_secs_f32());
        match elapsed {
            t if (1.0..2.4).contains(&t) => (Vec2::new(0.6, -0.8), false),
            t if (2.4..3.2).contains(&t) => (Vec2::new(-0.6, 0.8), true),
            _ => (Vec2::ZERO, false),
        }
    }
}

pub fn observe(world: &mut World) {
    let Some(mut run) = world.remove_resource::<Run>() else {
        return;
    };
    if run.requested {
        world.insert_resource(run);
        return;
    }
    assert!(
        run.started.elapsed().as_secs() < 80,
        "Reference scene capture timed out"
    );
    if !world.contains_resource::<reference_scene::Ready>() {
        world.insert_resource(run);
        return;
    }
    let ready = *run.ready.get_or_insert_with(Instant::now);
    let elapsed = ready.elapsed().as_secs_f32();
    let visuals: Vec<_> = world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .map(|(e, v)| (e, v.kind, v.gait))
        .collect();
    for (root, kind, gait) in visuals {
        let key = format!(
            "{}:{:?}",
            if kind == 0 { "human" } else { "wolf" },
            gait.unwrap_or(Gait::Stand)
        );
        let descendants = reference_scene::descendants(world, root);
        if let Some(before) = run.samples.get(&key) {
            let changes = before
                .iter()
                .filter(|(e, t)| {
                    world.get::<Transform>(*e).is_some_and(|now| {
                        now.translation.distance_squared(t.translation) > 1e-10
                            || 1. - now.rotation.dot(t.rotation).abs() > 1e-7
                    })
                })
                .count();
            let value = run.changes.entry(key).or_default();
            *value = (*value).max(changes);
        } else {
            run.samples.insert(
                key,
                descendants
                    .iter()
                    .filter_map(|e| world.get::<Transform>(*e).map(|t| (*e, *t)))
                    .collect(),
            );
        }
    }
    let position = world
        .query_filtered::<&Transform, With<LocalPawn>>()
        .single(world)
        .unwrap()
        .translation;
    run.closest_wolf = run
        .closest_wolf
        .min(Vec2::new(position.x - 3., position.z + 4.).length());
    if elapsed > 4.5 {
        for key in ["human:Stand", "human:Walk", "human:Run", "wolf:Stand"] {
            assert!(
                run.changes.get(key).is_some_and(|n| *n > 0),
                "No sampled animation motion: {key}"
            );
        }
        assert!(
            run.closest_wolf < 0.5,
            "Player must pass through nonblocking wolf spawn"
        );
        let houses = world
            .query_filtered::<Entity, With<reference_scene::House>>()
            .iter(world)
            .count();
        assert_eq!(houses, 1);
        let adapter = world.resource::<bevy::render::renderer::RenderAdapterInfo>();
        let report = serde_json::json!({"completed":true,"schema":1,"animation_transform_changes":run.changes,"closest_player_to_wolf":run.closest_wolf,"house_instances":houses,"gpu":adapter.name,"position":position.to_array(),"scope":"local art integration, actual locomotion animation and nonblocking wolf presentation; no NPC authority, AI, combat or housing gameplay"});
        let output = run.output.clone();
        world.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .unwrap()
                    .save(output.join("scene.png"))
                    .unwrap();
                std::fs::write(
                    output.join("reference.json"),
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .unwrap();
                exit.write(AppExit::Success);
            },
        );
        run.requested = true;
    }
    world.insert_resource(run);
}
