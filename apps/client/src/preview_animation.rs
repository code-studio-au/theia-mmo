//! Standard Bevy animation graphs, confined to the asset inspection mode.
use super::{Hud, Preview};
use bevy::prelude::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct Playback {
    requested: Option<String>,
    active: Option<Active>,
}
struct Active {
    name: String,
    duration: f32,
    player_count: usize,
    started: Instant,
    baseline: Option<Vec<(Entity, Transform)>>,
    changed: Option<usize>,
}
impl Playback {
    pub fn new(requested: Option<String>) -> Self {
        Self {
            requested,
            active: None,
        }
    }
    pub fn ready_to_capture(&self) -> bool {
        self.requested.is_none()
            || self
                .active
                .as_ref()
                .is_some_and(|a| a.changed.is_some_and(|n| n > 0))
    }
    pub fn report(&self) -> Value {
        self.active.as_ref().map_or(Value::Null,|a|json!({"name":a.name,"duration":a.duration,"players":a.player_count,"changed_transforms":a.changed,"elapsed":a.started.elapsed().as_secs_f32()}))
    }
}
fn descendants(world: &World, root: Entity) -> Vec<Entity> {
    let mut stack = vec![root];
    let mut result = Vec::new();
    while let Some(entity) = stack.pop() {
        result.push(entity);
        if let Some(children) = world.get::<Children>(entity) {
            stack.extend(children.iter());
        }
    }
    result
}
pub fn start(world: &mut World) {
    let playback = world.resource::<Playback>();
    let Some(name) = playback.requested.clone() else {
        return;
    };
    if playback.active.is_some() || world.resource::<Preview>().stats.is_none() {
        return;
    }
    let preview = world.resource::<Preview>();
    let root = preview.root.unwrap();
    let gltf = world
        .resource::<Assets<Gltf>>()
        .get(&preview.handle)
        .unwrap();
    let Some(clip) = gltf.named_animations.get(name.as_str()).cloned() else {
        error!("Animation clip not found: {name}");
        world.write_message(AppExit::error());
        return;
    };
    let duration = world
        .resource::<Assets<AnimationClip>>()
        .get(&clip)
        .expect("Loaded animation")
        .duration();
    if duration <= 0. {
        error!("Animation clip is empty: {name}");
        world.write_message(AppExit::error());
        return;
    }
    let (graph, index) = AnimationGraph::from_clip(clip);
    let handle = world.resource_mut::<Assets<AnimationGraph>>().add(graph);
    let mut count = 0;
    for entity in descendants(world, root) {
        if let Some(mut player) = world.get_mut::<AnimationPlayer>(entity) {
            player.play(index).repeat();
            count += 1;
            world
                .entity_mut(entity)
                .insert(AnimationGraphHandle(handle.clone()));
        }
    }
    if count == 0 {
        error!("No animation player instantiated");
        world.write_message(AppExit::error());
        return;
    }
    info!("Playing {name}: duration={duration}, players={count}");
    let mut hud = world.query_filtered::<&mut Text, With<Hud>>();
    for mut text in hud.iter_mut(world) {
        text.0.push_str(&format!("\nPlaying: {name}"));
    }
    world.resource_mut::<Playback>().active = Some(Active {
        name,
        duration,
        player_count: count,
        started: Instant::now(),
        baseline: None,
        changed: None,
    });
}
pub fn sample_motion(world: &mut World) {
    let Some(active) = world.resource::<Playback>().active.as_ref() else {
        return;
    };
    if active.changed.is_some() || active.started.elapsed() < Duration::from_millis(500) {
        return;
    }
    if active.baseline.is_none() {
        let root = world.resource::<Preview>().root.unwrap();
        let baseline = descendants(world, root)
            .into_iter()
            .filter_map(|e| world.get::<Transform>(e).copied().map(|t| (e, t)))
            .collect();
        world
            .resource_mut::<Playback>()
            .active
            .as_mut()
            .unwrap()
            .baseline = Some(baseline);
    } else if let Some(baseline) = &active.baseline
        && active.started.elapsed() > Duration::from_millis(900)
    {
        let changed = baseline
            .iter()
            .filter(|(entity, before)| {
                world.get::<Transform>(*entity).is_some_and(|after| {
                    after.translation.distance_squared(before.translation) > 1e-10
                        || 1. - after.rotation.dot(before.rotation).abs() > 1e-7
                        || after.scale.distance_squared(before.scale) > 1e-10
                })
            })
            .count();
        if changed == 0 {
            error!("Animation clock produced no sampled pose change");
            world.write_message(AppExit::error());
        }
        world
            .resource_mut::<Playback>()
            .active
            .as_mut()
            .unwrap()
            .changed = Some(changed);
    }
}
