//! Server-cued sword presentation using standard Bevy skeletal masks.
use bevy::{animation::AnimationTargetId, prelude::*};
use std::{
    collections::{BTreeMap, HashSet},
    time::{Duration, Instant},
};
use theia_core::{
    abilities::{AnimationCue, Catalog},
    identity::EntityKey,
};
use theia_network::simulation::combat::Event;
const PRESENTATION_SECONDS: f32 = 0.8;
struct Cue {
    key: EntityKey,
    sequence: u64,
    ability_sequence: u64,
    melee_sequence: u64,
    basic: bool,
    received: Instant,
    offset: f32,
    cancelled: bool,
}
#[derive(Resource, Default)]
pub(crate) struct Cues(BTreeMap<u64, Cue>);
impl Cues {
    pub(crate) fn offer_white(
        &mut self,
        fact: &theia_network::simulation::combat::white_melee::Fact,
        catalog: &theia_core::damage::catalog::Catalog,
        now: Instant,
    ) {
        if fact.actor_generation.is_some()
            && matches!(catalog.attack(&fact.attack),
            Some(theia_core::damage::catalog::Attack::Weapon {item,..}) if item.as_str()=="theia.dev.training-sword")
        {
            self.0.retain(|_, cue| {
                now.saturating_duration_since(cue.received) < Duration::from_secs(2)
            });
            self.start(fact.actor, fact.sequence, true, 0., now);
        }
    }
    pub(crate) fn offer_physical(
        &mut self,
        fact: &theia_network::simulation::combat::physical::Fact,
        profile: &theia_network::simulation::combat::physical::Profile,
        now: Instant,
    ) {
        if fact.actor_generation.is_some()
            && matches!(profile.catalog().attack(&fact.attack),
                Some(theia_core::damage::catalog::Attack::Weapon { item, .. })
                    if item.as_str() == "theia.dev.training-sword")
        {
            self.0.retain(|_, cue| {
                now.saturating_duration_since(cue.received) < Duration::from_secs(2)
            });
            self.start(fact.actor, fact.sequence, true, 0., now);
        }
    }

    pub(crate) fn offer(
        &mut self,
        event: &Event,
        catalog: &Catalog,
        tick: u64,
        hz: u32,
        now: Instant,
    ) {
        self.0
            .retain(|_, cue| now.saturating_duration_since(cue.received) < Duration::from_secs(2));
        match event {
            Event::Started { actor, cast, .. }
                if catalog
                    .ability(cast.ability.as_str())
                    .is_some_and(|a| a.animation() == Some(AnimationCue::SwordAttack)) =>
            {
                self.start(
                    *actor,
                    cast.sequence,
                    false,
                    tick.saturating_sub(cast.started) as f32 / hz.max(1) as f32,
                    now,
                );
            }
            Event::MeleeDamage {
                actor,
                sequence,
                weapon,
                ..
            } if weapon.as_str() == "theia.dev.training-sword" => {
                self.start(*actor, *sequence, true, 0., now);
            }
            Event::Cancelled {
                actor, sequence, ..
            } => {
                if let Some(cue) = self.0.get_mut(&actor.entity.value())
                    && cue.key == *actor
                    && !cue.basic
                    && cue.ability_sequence == *sequence
                {
                    cue.cancelled = true;
                }
            }
            _ => {}
        }
    }
    fn start(
        &mut self,
        key: EntityKey,
        source_sequence: u64,
        basic: bool,
        offset: f32,
        now: Instant,
    ) {
        let previous = self.0.get(&key.entity.value()).filter(|old| old.key == key);
        let ability_sequence = previous.map_or(0, |c| c.ability_sequence);
        let melee_sequence = previous.map_or(0, |c| c.melee_sequence);
        if source_sequence == 0
            || source_sequence
                <= if basic {
                    melee_sequence
                } else {
                    ability_sequence
                }
        {
            return;
        }
        if !self.0.contains_key(&key.entity.value())
            && self.0.len() >= theia_network::wire::MAX_PLAYERS
        {
            return;
        }
        let Some(sequence) = previous.map_or(Some(1), |c| c.sequence.checked_add(1)) else {
            return;
        };
        self.0.insert(
            key.entity.value(),
            Cue {
                key,
                sequence,
                ability_sequence: if basic {
                    ability_sequence
                } else {
                    source_sequence
                },
                melee_sequence: if basic {
                    source_sequence
                } else {
                    melee_sequence
                },
                basic,
                received: now,
                offset,
                cancelled: false,
            },
        );
    }
    pub(super) fn pose(&self, key: EntityKey, now: Instant) -> Option<(u64, f32)> {
        let cue = self.0.get(&key.entity.value())?;
        let elapsed = cue.offset + now.saturating_duration_since(cue.received).as_secs_f32();
        (cue.key == key && !cue.cancelled && elapsed < PRESENTATION_SECONDS)
            .then_some((cue.sequence, elapsed))
    }
    pub(super) fn invalidate(&mut self, key: EntityKey) {
        if let Some(cue) = self.0.get_mut(&key.entity.value())
            && cue.key == key
        {
            // Retain the sequence tombstone so a duplicate cannot restart it.
            cue.cancelled = true;
        }
    }
}
pub(super) struct Graph {
    pub bases: [AnimationNodeIndex; 4],
    pub directional_bases: [Option<AnimationNodeIndex>; super::locomotion::COUNT],
    pub strike: AnimationNodeIndex,
}
impl Graph {
    pub(super) fn base(&self, clip: usize) -> AnimationNodeIndex {
        if clip < 4 {
            self.bases[clip]
        } else {
            self.directional_bases[clip - super::locomotion::FIRST]
                .expect("Selected attack direction is available")
        }
    }
}
fn mask_graph(
    graph: &mut AnimationGraph,
    sources: [Handle<AnimationClip>; 4],
    directional: [Option<Handle<AnimationClip>>; super::locomotion::COUNT],
    attack: Handle<AnimationClip>,
    upper: &HashSet<AnimationTargetId>,
    targets: &HashSet<AnimationTargetId>,
) -> Graph {
    for id in targets {
        graph.add_target_to_mask_group(*id, if upper.contains(id) { 3 } else { 2 });
    }
    Graph {
        bases: sources.map(|clip| graph.add_clip_with_mask(clip, 1 << 3, 1., graph.root)),
        directional_bases: directional.map(|source| {
            source.map(|clip| graph.add_clip_with_mask(clip, 1 << 3, 1., graph.root))
        }),
        strike: graph.add_clip_with_mask(attack, 1 << 2, 1., graph.root),
    }
}
pub(super) fn prepare(
    world: &World,
    root: Entity,
    graph: &mut AnimationGraph,
    sources: [Handle<AnimationClip>; 4],
    directional: [Option<Handle<AnimationClip>>; super::locomotion::COUNT],
    attack: Handle<AnimationClip>,
    targets: &HashSet<AnimationTargetId>,
) -> Result<Graph, String> {
    let spines: Vec<_> = super::descendants(world, root)
        .into_iter()
        .filter(|e| {
            world
                .get::<Name>(*e)
                .is_some_and(|n| n.as_str() == "Spine_01")
        })
        .collect();
    if spines.len() != 1 {
        return Err("Sword attack requires exactly one Spine_01".into());
    }
    let upper: HashSet<_> = super::descendants(world, spines[0])
        .iter()
        .filter_map(|e| world.get::<AnimationTargetId>(*e).copied())
        .collect();
    if upper.is_empty() || !upper.is_subset(targets) || upper.len() == targets.len() {
        return Err("Invalid upper-body animation partition".into());
    }
    let clips = world.resource::<Assets<AnimationClip>>();
    let clip = clips
        .get(&attack)
        .ok_or("Missing SwordAttack source clip")?;
    if (clip.duration() - PRESENTATION_SECONDS).abs() > 0.01
        || targets
            .iter()
            .any(|id| clip.curves_for_target(*id).is_none())
    {
        return Err("SwordAttack duration or channels differ from qualified source".into());
    }
    Ok(mask_graph(
        graph,
        sources,
        directional,
        attack,
        &upper,
        targets,
    ))
}
#[derive(Resource, Default, serde::Serialize)]
pub(crate) struct Metrics {
    #[serde(skip)]
    pub capture: bool,
    pub available: bool,
    pub local_starts: u64,
    pub remote_starts: u64,
    pub local_frames: u64,
    pub remote_frames: u64,
    pub moving_frames: u64,
    pub samples: Vec<Sample>,
}
#[derive(serde::Serialize)]
pub(crate) struct Sample {
    local: bool,
    clip: &'static str,
    base_time: f32,
    attack_time: f32,
    bones: BTreeMap<String, Pose>,
}
#[derive(serde::Serialize)]
struct Pose {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}
pub(crate) fn observe(world: &mut World) {
    if !world
        .get_resource::<Metrics>()
        .is_some_and(|m| m.available && m.capture)
    {
        return;
    }
    let roots: Vec<_> = world
        .query::<(Entity, &super::Visual)>()
        .iter(world)
        .filter(|(_, v)| v.attack_sequence.is_some())
        .map(|(e, v)| (e, v.owner, v.clip.unwrap()))
        .collect();
    for (root, owner, clip) in roots {
        let local = world.get::<crate::movement::LocalPawn>(owner).is_some();
        let name = super::locomotion::base_name(clip).expect("Attack has a locomotion base");
        let ready = {
            let mut v = world.get_mut::<super::Visual>(root).unwrap();
            v.layer_frames = v.layer_frames.saturating_add(1);
            v.layer_frames >= 3
        };
        if !ready
            || world
                .resource::<Metrics>()
                .samples
                .iter()
                .any(|s| s.local == local && s.clip == name)
        {
            continue;
        }
        let visual = world.get::<super::Visual>(root).unwrap();
        let graph = world
            .resource::<super::Catalog>()
            .grip
            .as_ref()
            .unwrap()
            .attack
            .as_ref()
            .unwrap();
        let times = visual.players.iter().find_map(|e| {
            let player = world.get::<AnimationPlayer>(*e)?;
            Some((
                player.animation(graph.base(clip))?.seek_time(),
                player.animation(graph.strike)?.seek_time(),
            ))
        });
        let Some((base_time, attack_time)) = times else {
            continue;
        };
        let bones = super::descendants(world, root)
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
        if world.resource::<Metrics>().samples.len() < 8 {
            world.resource_mut::<Metrics>().samples.push(Sample {
                local,
                clip: name,
                base_time,
                attack_time,
                bones,
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authoritative_cues_expire_and_duplicate_or_foreign_cancellation_cannot_restart_them() {
        use theia_core::{
            content::ContentId,
            coordinates::GlobalPosition,
            identity::{EntityId, WorldEpoch},
        };
        use theia_network::simulation::combat::{Cast, Reason};
        let catalog = Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-melee-abilities.json"
        ))
        .unwrap();
        let actor = EntityKey {
            entity: EntityId::new(1).unwrap(),
            epoch: WorldEpoch::new(2).unwrap(),
        };
        let event = Event::Started {
            actor,
            cast: Cast {
                sequence: 1,
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: actor,
                started: 30,
                completes: 45,
            },
            position: GlobalPosition::new([0., 0., 0.]).unwrap(),
        };
        let now = Instant::now();
        let mut cues = Cues::default();
        cues.offer(&event, &catalog, 33, 30, now);
        assert!((cues.pose(actor, now).unwrap().1 - 0.1).abs() < 0.0001);
        cues.offer(&event, &catalog, 33, 30, now + Duration::from_millis(200));
        assert!(
            (cues
                .pose(actor, now + Duration::from_millis(200))
                .unwrap()
                .1
                - 0.3)
                .abs()
                < 0.0001
        );
        cues.offer(
            &Event::Cancelled {
                actor,
                sequence: 2,
                reason: Reason::Interrupted,
            },
            &catalog,
            34,
            30,
            now,
        );
        assert!(cues.pose(actor, now).is_some());
        let foreign = EntityKey {
            epoch: WorldEpoch::new(3).unwrap(),
            ..actor
        };
        assert!(cues.pose(foreign, now).is_none());
        assert!(cues.pose(actor, now + Duration::from_millis(701)).is_none());
        cues.offer(
            &Event::Cancelled {
                actor,
                sequence: 1,
                reason: Reason::EquipmentRequired,
            },
            &catalog,
            34,
            30,
            now,
        );
        assert!(cues.pose(actor, now).is_none());
        cues.offer(&event, &catalog, 33, 30, now);
        assert!(cues.pose(actor, now).is_none());
        let mut next = event.clone();
        if let Event::Started { cast, .. } = &mut next {
            cast.sequence = 2;
        }
        cues.offer(&next, &catalog, 33, 30, now);
        cues.invalidate(foreign);
        assert!(cues.pose(actor, now).is_some());
        cues.invalidate(actor);
        assert!(cues.pose(actor, now).is_none());
        cues.offer(&next, &catalog, 33, 30, now);
        assert!(cues.pose(actor, now).is_none());
    }
    #[test]
    fn basic_and_ability_sequences_do_not_suppress_or_cancel_each_other() {
        use theia_core::{
            content::ContentId,
            identity::{EntityId, WorldEpoch},
        };
        use theia_network::simulation::combat::{Cast, Reason};
        let catalog = Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-melee-abilities.json"
        ))
        .unwrap();
        let actor = EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let mut cues = Cues::default();
        let now = Instant::now();
        let spell = Event::Started {
            actor,
            cast: Cast {
                sequence: 1,
                ability: ContentId::new("theia.dev.sword-slash").unwrap(),
                target: actor,
                started: 30,
                completes: 45,
            },
            position: theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap(),
        };
        cues.offer(&spell, &catalog, 30, 30, now);
        let first = cues.pose(actor, now).unwrap().0;
        let basic = Event::MeleeDamage {
            actor,
            sequence: 1,
            target: actor,
            weapon: ContentId::new("theia.dev.training-sword").unwrap(),
            position: theia_core::coordinates::GlobalPosition::new([0.; 3]).unwrap(),
            amount: 15,
            remaining: 85,
        };
        cues.offer(&basic, &catalog, 30, 30, now);
        let second = cues.pose(actor, now).unwrap().0;
        assert!(second > first);
        cues.offer(
            &Event::Cancelled {
                actor,
                sequence: 1,
                reason: Reason::Interrupted,
            },
            &catalog,
            30,
            30,
            now,
        );
        assert_eq!(cues.pose(actor, now).unwrap().0, second);
        cues.offer(&basic, &catalog, 30, 30, now + Duration::from_millis(200));
        assert!(
            cues.pose(actor, now + Duration::from_millis(200))
                .unwrap()
                .1
                >= 0.2
        );
        assert_eq!(cues.pose(actor, now).unwrap().0, second);
        let mut next = spell;
        if let Event::Started { cast, .. } = &mut next {
            cast.sequence = 2;
        }
        cues.offer(&next, &catalog, 30, 30, now);
        assert!(cues.pose(actor, now).unwrap().0 > second);
    }
    #[test]
    fn attack_masks_preserve_locomotion_hips_legs_and_authored_upper_body() {
        let id = |s: &str| AnimationTargetId::from_name(&Name::new(s.to_owned()));
        let hips = id("Hips");
        let leg = id("UpperLeg_R");
        let wrist = id("Hand_R");
        let prop = id("Prop_R");
        let digit = id("Finger_01_1");
        let upper = HashSet::from([wrist, prop, digit]);
        let targets = HashSet::from([hips, leg, wrist, prop, digit]);
        let mut graph = AnimationGraph::default();
        let result = mask_graph(
            &mut graph,
            std::array::from_fn(|_| Handle::default()),
            std::array::from_fn(|_| Some(Handle::default())),
            Handle::default(),
            &upper,
            &targets,
        );
        assert_eq!(graph.mask_groups[&hips], 1 << 2);
        assert_eq!(graph.mask_groups[&leg], 1 << 2);
        for id in upper {
            assert_eq!(graph.mask_groups[&id], 1 << 3)
        }
        for base in result
            .bases
            .into_iter()
            .chain(result.directional_bases.into_iter().flatten())
        {
            assert_eq!(graph.graph[base].mask, 1 << 3)
        }
        assert_eq!(graph.graph[result.strike].mask, 1 << 2);
    }
    #[test]
    fn ordinary_avoided_swings_animate_once_and_use_the_basic_sequence_namespace() {
        let catalog = theia_core::damage::catalog::Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-physical-damage-definitions.json"
        ))
        .unwrap();
        let abilities = Catalog::from_json(include_bytes!(
            "../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap();
        let fact = crate::combat::test_facts::fact(
            theia_core::damage::white_melee::Outcome::Dodge,
            false,
            100,
        );
        let now = Instant::now();
        let mut cues = Cues::default();
        cues.offer_white(&fact, &catalog, now);
        let first = cues.pose(fact.actor, now).unwrap();
        cues.offer_white(&fact, &catalog, now + Duration::from_millis(200));
        let pose = cues
            .pose(fact.actor, now + Duration::from_millis(200))
            .unwrap();
        assert_eq!(pose.0, first.0);
        assert!(pose.1 >= 0.19);
        cues.offer(
            &Event::Cancelled {
                actor: fact.actor,
                sequence: fact.sequence,
                reason: theia_network::simulation::combat::Reason::Interrupted,
            },
            &abilities,
            30,
            30,
            now + Duration::from_millis(200),
        );
        assert!(
            cues.pose(fact.actor, now + Duration::from_millis(300))
                .is_some()
        );
        let incoming = crate::combat::test_facts::fact(
            theia_core::damage::white_melee::Outcome::Hit,
            true,
            100,
        );
        cues.offer_white(&incoming, &catalog, now);
        assert!(cues.pose(incoming.actor, now).is_none());
    }
}
