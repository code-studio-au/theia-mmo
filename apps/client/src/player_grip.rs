//! Standard Bevy masks: locomotion owns wrist/socket/body, SwordIdle owns digits.
use super::{Catalog, Visual, descendants};
use bevy::{
    animation::{AnimationTargetId, graph::AnimationNodeType},
    prelude::*,
};
use std::collections::{BTreeMap, HashSet};
const DIGITS: [&str; 11] = [
    "Thumb_01_1",
    "Thumb_02_1",
    "Thumb_03_1",
    "IndexFinger_01_1",
    "IndexFinger_02_1",
    "IndexFinger_03_1",
    "IndexFinger_04_1",
    "Finger_01_1",
    "Finger_02_1",
    "Finger_03_1",
    "Finger_04_1",
];
pub(super) struct Graph {
    pub handle: Handle<AnimationGraph>,
    pub bases: [AnimationNodeIndex; 4],
    pub dodge_bases: [Option<AnimationNodeIndex>; 4],
    pub hold: AnimationNodeIndex,
    pub attack: Option<super::attack::Graph>,
}
impl Graph {
    pub fn base(&self, clip: usize) -> AnimationNodeIndex {
        if clip < 4 {
            self.bases[clip]
        } else {
            self.dodge_bases[clip - 6].expect("Selected masked dodge is available")
        }
    }
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    pub available: bool,
    #[serde(skip)]
    pub(super) capture: bool,
    pub local_frames: [u64; 4],
    pub remote_frames: [u64; 4],
    pub dodge_local_frames: [u64; 4],
    pub samples: Vec<Sample>,
}
#[derive(serde::Serialize)]
pub(crate) struct Sample {
    local: bool,
    clip: &'static str,
    time: f32,
    bones: BTreeMap<String, Pose>,
}
#[derive(serde::Serialize)]
struct Pose {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}
fn mask_graph(
    graph: &mut AnimationGraph,
    sources: [Handle<AnimationClip>; 4],
    hold: Handle<AnimationClip>,
    digits: &HashSet<AnimationTargetId>,
    targets: &HashSet<AnimationTargetId>,
) -> ([AnimationNodeIndex; 4], AnimationNodeIndex) {
    for id in targets {
        graph.add_target_to_mask_group(*id, u32::from(!digits.contains(id)));
    }
    let bases = sources.map(|clip| graph.add_clip_with_mask(clip, 1, 1., graph.root));
    let hold = graph.add_clip_with_mask(hold, 2, 1., graph.root);
    (bases, hold)
}
pub(super) fn prepare(world: &mut World, root: Entity) -> Result<(), String> {
    if world.resource::<Catalog>().grip.is_some() {
        return Ok(());
    }
    let descendants = descendants(world, root);
    let hands: Vec<_> = descendants
        .iter()
        .copied()
        .filter(|e| {
            world
                .get::<Name>(*e)
                .is_some_and(|n| n.as_str() == "Hand_R")
        })
        .collect();
    if hands.len() != 1 {
        return Err("Sword grip requires exactly one Hand_R".into());
    }
    let hand_children = super::descendants(world, hands[0]);
    let mut digits = HashSet::new();
    for name in DIGITS {
        let matching: Vec<_> = hand_children
            .iter()
            .filter(|e| world.get::<Name>(**e).is_some_and(|n| n.as_str() == name))
            .collect();
        if matching.len() != 1 {
            return Err(format!("Sword grip requires one right-hand digit {name}"));
        }
        let id = world
            .get::<AnimationTargetId>(*matching[0])
            .ok_or("Missing digit animation target")?;
        digits.insert(*id);
    }
    if digits.len() != 11 {
        return Err("Sword grip digit targets are not unique".into());
    }
    let catalog = world.resource::<Catalog>();
    let source_graph = world
        .resource::<Assets<AnimationGraph>>()
        .get(&catalog.graphs[0])
        .ok_or("Missing body animation graph")?;
    let mut graph = source_graph.clone();
    let clip = |slot: usize| match &graph.graph[catalog.indices[0][slot]].node_type {
        AnimationNodeType::Clip(handle) => Ok(handle.clone()),
        _ => Err("Expected source clip node".to_string()),
    };
    let sources = [clip(0)?, clip(1)?, clip(2)?, clip(3)?];
    let hold = clip(catalog.optional_slots[1].ok_or("Missing SwordIdle clip")?)?;
    let attack = catalog.attack_slot.map(clip).transpose()?;
    let dodge = |i: usize| catalog.dodge_slots[i].map(clip).transpose();
    let dodge_sources = [dodge(0)?, dodge(1)?, dodge(2)?, dodge(3)?];
    let clips = world.resource::<Assets<AnimationClip>>();
    let mut targets = HashSet::new();
    for handle in sources.iter().chain(std::iter::once(&hold)) {
        let source = clips.get(handle).ok_or("Missing grip source clip")?;
        if !digits
            .iter()
            .all(|id| source.curves_for_target(*id).is_some())
        {
            return Err("Source clip omits right-hand digit channels".into());
        }
        targets.extend(source.curves().keys().copied());
    }
    let attack_available = attack.is_some();
    let strike = attack
        .map(|attack| {
            super::attack::prepare(world, root, &mut graph, sources.clone(), attack, &targets)
        })
        .transpose()?;
    let (bases, hold) = mask_graph(&mut graph, sources, hold, &digits, &targets);
    let dodge_bases = dodge_sources
        .map(|source| source.map(|clip| graph.add_clip_with_mask(clip, 1, 1., graph.root)));
    let handle = world.resource_mut::<Assets<AnimationGraph>>().add(graph);
    world.resource_mut::<Catalog>().grip = Some(Graph {
        handle,
        bases,
        dodge_bases,
        hold,
        attack: strike,
    });
    world.resource_mut::<Metrics>().available = true;
    if let Some(mut metrics) = world.get_resource_mut::<super::attack::Metrics>() {
        metrics.available = attack_available;
    }
    Ok(())
}
pub(crate) fn observe(world: &mut World) {
    let qualification = world.get_resource::<Metrics>().is_some_and(|m| m.capture);
    let roots: Vec<_> = world
        .query::<(Entity, &Visual)>()
        .iter(world)
        .filter(|(_, v)| v.grip_active)
        .map(|(e, v)| (e, v.owner, v.clip.unwrap()))
        .collect();
    for (root, owner, clip) in roots {
        let local = world.get::<crate::movement::LocalPawn>(owner).is_some();
        {
            let mut metrics = world.resource_mut::<Metrics>();
            if clip >= 6 {
                metrics.dodge_local_frames[clip - 6] += 1;
            } else if local {
                metrics.local_frames[clip] += 1;
            } else {
                metrics.remote_frames[clip] += 1;
            }
        }
        let mut visual = world.get_mut::<Visual>(root).unwrap();
        visual.layer_frames = visual.layer_frames.saturating_add(1);
        if !qualification || visual.layer_frames < 3 {
            continue;
        }
        let name = if clip < 4 {
            ["Idle", "Walk", "Run", "Jump"][clip]
        } else {
            crate::dodge::animation::NAMES[clip - 6]
        };
        if world
            .resource::<Metrics>()
            .samples
            .iter()
            .any(|s| s.local == local && s.clip == name)
        {
            continue;
        }
        let visual = world.get::<Visual>(root).unwrap();
        let graph = world.resource::<Catalog>().grip.as_ref().unwrap();
        let time = visual.players.iter().find_map(|e| {
            world
                .get::<AnimationPlayer>(*e)
                .and_then(|p| p.animation(graph.base(clip)))
                .map(|a| a.seek_time())
        });
        let Some(time) = time else {
            continue;
        };
        let bones = descendants(world, root)
            .into_iter()
            .filter_map(|e| {
                world.get::<AnimationTargetId>(e)?;
                let name = world.get::<Name>(e)?.as_str().to_owned();
                let t = world.get::<Transform>(e)?;
                Some((
                    name,
                    Pose {
                        translation: t.translation.to_array(),
                        rotation: t.rotation.to_array(),
                        scale: t.scale.to_array(),
                    },
                ))
            })
            .collect();
        world.resource_mut::<Metrics>().samples.push(Sample {
            local,
            clip: name,
            time,
            bones,
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn masks_keep_wrist_socket_and_body_on_locomotion_and_only_digits_on_grip() {
        let id = |s: &str| AnimationTargetId::from_name(&Name::new(s.to_owned()));
        let finger = id("Finger_01_1");
        let wrist = id("Hand_R");
        let socket = id("Prop_R");
        let hips = id("Hips");
        let digits = HashSet::from([finger]);
        let targets = HashSet::from([finger, wrist, socket, hips]);
        let mut graph = AnimationGraph::default();
        let (bases, hold) = mask_graph(
            &mut graph,
            std::array::from_fn(|_| Handle::default()),
            Handle::default(),
            &digits,
            &targets,
        );
        assert_eq!(graph.mask_groups[&finger], 1);
        for id in [wrist, socket, hips] {
            assert_eq!(graph.mask_groups[&id], 2);
        }
        for index in bases {
            assert_eq!(graph.graph[index].mask, 1);
        }
        assert_eq!(graph.graph[hold].mask, 2);
        assert_eq!(graph.graph[graph.root].mask, 0);
    }
}
