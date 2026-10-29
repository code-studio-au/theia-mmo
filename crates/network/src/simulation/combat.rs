//! Optional bounded server combat. No transport, renderer, file I/O or wall clock.
use super::World;
use std::{collections::BTreeMap, time::Duration};
use theia_core::{
    abilities::{Ability, Catalog, Effect},
    content::ContentId,
    coordinates::GlobalPosition,
    identity::EntityKey,
};

pub const MAX_STEP_EVENTS: usize = 64;
pub const DEVELOPMENT_HEALTH: u32 = 100;
pub const DEVELOPMENT_RESPAWN_MS: u32 = 5000;
fn configuration(bytes: &[u8], health: u32, respawn_ms: u32) -> crate::Result<(Catalog, [u8; 32])> {
    if health == 0 || health > 1_000_000 || !(100..=60_000).contains(&respawn_ms) {
        return Err("Combat health/respawn configuration exceeds bounded prototype".into());
    }
    let catalog = Catalog::from_json(bytes)?;
    let mut identity = b"theia-combat-profile-v2\0".to_vec();
    identity.extend(crate::terrain::sha256(bytes));
    identity.extend(health.to_le_bytes());
    identity.extend(respawn_ms.to_le_bytes());
    Ok((catalog, crate::terrain::sha256(&identity)))
}
fn world_digest(base: [u8; 32], config: [u8; 32]) -> [u8; 32] {
    let mut identity = b"theia-combat-world-v2\0".to_vec();
    identity.extend(base);
    identity.extend(config);
    crate::terrain::sha256(&identity)
}
/// Bind the validated catalog/configuration on both ends before admission.
/// `base` is the targeting-profile world identity, not the raw terrain digest.
pub fn profile_identity(
    base: [u8; 32],
    bytes: &[u8],
    health: u32,
    respawn_ms: u32,
) -> crate::Result<[u8; 32]> {
    let (_, config) = configuration(bytes, health, respawn_ms)?;
    Ok(world_digest(base, config))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub epoch: u64,
    pub sequence: u64,
    pub ability: ContentId,
    pub target: EntityKey,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    UnknownAbility,
    UnknownTarget,
    Dead,
    OutOfRange,
    NotReady,
    Obstructed,
    Moving,
    Interrupted,
    Unavailable,
    Superseded,
    TimeExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignored,
    Rejected(Reason),
    Started { completes: u64 },
    Queued { at: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub sequence: u64,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cast {
    pub sequence: u64,
    pub ability: ContentId,
    pub target: EntityKey,
    pub started: u64,
    pub completes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    pub request: Request,
    pub at: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Started {
        actor: EntityKey,
        cast: Cast,
        position: GlobalPosition,
    },
    Queued {
        actor: EntityKey,
        queued: Queued,
    },
    Cancelled {
        actor: EntityKey,
        sequence: u64,
        reason: Reason,
    },
    Damage {
        ability: ContentId,
        actor: EntityKey,
        sequence: u64,
        target: EntityKey,
        position: GlobalPosition,
        amount: u32,
        remaining: u32,
    },
    Died {
        target: EntityKey,
        position: GlobalPosition,
        respawn_at: u64,
    },
    Respawned {
        previous: EntityKey,
        target: EntityKey,
        position: GlobalPosition,
    },
}
#[derive(Clone, Debug)]
pub struct CastResult {
    pub reply: Reply,
    /// Immediate command outcomes, at most cancellation + replacement/start.
    pub events: Vec<Event>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Health {
    pub key: EntityKey,
    pub position: GlobalPosition,
    pub current: u32,
    pub maximum: u32,
    pub respawn_at: Option<u64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub tick: u64,
    pub reply: Option<Reply>,
    pub cast: Option<Cast>,
    pub queued: Option<Queued>,
    pub gcd_until: u64,
    pub cooldowns: Vec<(ContentId, u64)>,
    pub targets: Vec<Health>,
}
#[derive(Default)]
struct Actor {
    sequence: u64,
    reply: Option<Reply>,
    cast: Option<Cast>,
    queued: Option<Queued>,
    gcd_until: u64,
    cooldowns: BTreeMap<ContentId, u64>,
    interrupted: bool,
}
pub(super) struct Authority {
    catalog: Catalog,
    digest: [u8; 32],
    respawn_ticks: u64,
    creatures: Vec<Health>,
    actors: BTreeMap<u64, Actor>,
    events: Vec<Event>,
}
impl Authority {
    fn create(world: &World, bytes: &[u8], health: u32, respawn_ms: u32) -> crate::Result<Self> {
        let (catalog, digest) = configuration(bytes, health, respawn_ms)?;
        Ok(Self {
            catalog,
            digest,
            respawn_ticks: world
                .rate
                .ticks_for(Duration::from_millis(u64::from(respawn_ms)))
                .unwrap(),
            creatures: world
                .targets
                .values()
                .map(|t| Health {
                    key: t.key,
                    position: t.position,
                    current: health,
                    maximum: health,
                    respawn_at: None,
                })
                .collect(),
            actors: BTreeMap::new(),
            events: Vec::with_capacity(MAX_STEP_EVENTS),
        })
    }
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        world_digest(base, self.digest)
    }
    pub(super) fn leave(&mut self, id: u64) {
        self.actors.remove(&id);
    }
    pub(super) fn interrupt(&mut self, id: u64) {
        if let Some(actor) = self.actors.get_mut(&id) {
            actor.interrupted = true;
        }
    }
    pub(super) fn forget(&mut self, key: EntityKey) -> bool {
        let count = self.creatures.len();
        self.creatures.retain(|c| c.key != key);
        self.creatures.len() != count
    }
    fn ready(world: &World, id: u64) -> bool {
        let Some(actor) = world.players.get(&id) else {
            return false;
        };
        world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(actor.position))
            .is_some_and(|p| p.covers(actor.position, actor.ready.as_ref()))
    }
    fn validate(
        &self,
        world: &World,
        id: u64,
        target: EntityKey,
        ability: &Ability,
        moved: bool,
    ) -> std::result::Result<(), Reason> {
        let actor = world.players.get(&id).ok_or(Reason::Interrupted)?;
        let health = self
            .creatures
            .iter()
            .find(|c| c.key == target)
            .ok_or(Reason::UnknownTarget)?;
        if health.current == 0 {
            return Err(Reason::Dead);
        }
        if !Self::ready(world, id) {
            return Err(Reason::NotReady);
        }
        if !ability.cast_while_moving()
            && (moved || actor.intent.axes != [0, 0] || actor.body.vertical_speed().abs() > 0.001)
        {
            return Err(Reason::Moving);
        }
        let a = actor.position.metres();
        let b = health.position.metres();
        let distance2: f64 = a.into_iter().zip(b).map(|(a, b)| (a - b) * (a - b)).sum();
        if distance2 > ability.range_metres().powi(2) {
            return Err(Reason::OutOfRange);
        }
        let patch = world
            .procedural
            .as_ref()
            .and_then(|r| r.patch(actor.position))
            .ok_or(Reason::NotReady)?;
        if !patch.covers(health.position, None) {
            return Err(Reason::NotReady);
        }
        let from = patch
            .frame
            .to_local(actor.position)
            .ok_or(Reason::NotReady)?;
        let to = patch
            .frame
            .to_local(health.position)
            .ok_or(Reason::NotReady)?;
        if !patch.scene.line_clear(
            bevy::prelude::Vec3::from_array(from.metres()),
            bevy::prelude::Vec3::from_array(to.metres()),
        ) {
            return Err(Reason::Obstructed);
        }
        Ok(())
    }
    fn available(&self, id: u64, ability: &Ability, now: u64) -> u64 {
        self.actors.get(&id).map_or(now, |actor| {
            now.max(actor.cast.as_ref().map_or(now, |c| c.completes))
                .max(if ability.uses_global_cooldown() {
                    actor.gcd_until
                } else {
                    now
                })
                .max(actor.cooldowns.get(ability.id()).copied().unwrap_or(now))
        })
    }
    fn start(
        &mut self,
        world: &World,
        id: u64,
        request: Request,
    ) -> std::result::Result<Event, Reason> {
        let timing = self
            .catalog
            .timing(request.ability.as_str(), world.rate)
            .unwrap();
        let completes = world
            .tick
            .checked_add(timing.cast_ticks.max(1))
            .ok_or(Reason::TimeExhausted)?;
        let gcd = world
            .tick
            .checked_add(timing.global_cooldown_ticks)
            .ok_or(Reason::TimeExhausted)?;
        let cooldown = world
            .tick
            .checked_add(timing.cooldown_ticks)
            .ok_or(Reason::TimeExhausted)?;
        let cast = Cast {
            sequence: request.sequence,
            ability: request.ability,
            target: request.target,
            started: world.tick,
            completes,
        };
        let actor = self.actors.entry(id).or_default();
        actor.cast = Some(cast.clone());
        if timing.global_cooldown_ticks > 0 {
            actor.gcd_until = gcd;
        }
        if timing.cooldown_ticks > 0 {
            actor.cooldowns.insert(cast.ability.clone(), cooldown);
        }
        Ok(Event::Started {
            actor: world.entity_key(id).unwrap(),
            cast,
            position: world.players[&id].position,
        })
    }
    fn request(&mut self, world: &World, id: u64, request: Request) -> CastResult {
        let ignored = CastResult {
            reply: Reply {
                sequence: request.sequence,
                outcome: Outcome::Ignored,
            },
            events: vec![],
        };
        let Some(actor_key) = world.entity_key(id) else {
            return ignored;
        };
        let previous = self.actors.get(&id).map_or(0, |a| a.sequence);
        if request.epoch != world.epoch()
            || request.target.epoch.value() != world.epoch()
            || request.sequence <= previous
            || request.sequence.saturating_sub(previous) > 128
        {
            return ignored;
        }
        self.actors.entry(id).or_default().sequence = request.sequence;
        let mut events = Vec::with_capacity(2);
        let outcome = (|| {
            if self.actors[&id].interrupted {
                return Err(Reason::Interrupted);
            }
            let ability = self
                .catalog
                .ability(request.ability.as_str())
                .ok_or(Reason::UnknownAbility)?
                .clone();
            self.validate(world, id, request.target, &ability, false)?;
            let at = self.available(id, &ability, world.tick);
            if at > world.tick {
                let window = self
                    .catalog
                    .timing(request.ability.as_str(), world.rate)
                    .unwrap()
                    .queue_window_ticks;
                if at - world.tick > window {
                    return Err(Reason::Unavailable);
                }
                let actor = self.actors.get_mut(&id).unwrap();
                if let Some(old) = actor.queued.take() {
                    events.push(Event::Cancelled {
                        actor: actor_key,
                        sequence: old.request.sequence,
                        reason: Reason::Superseded,
                    });
                }
                let queued = Queued { request, at };
                actor.queued = Some(queued.clone());
                events.push(Event::Queued {
                    actor: actor_key,
                    queued,
                });
                Ok(Outcome::Queued { at })
            } else {
                let event = self.start(world, id, request)?;
                let completes = self.actors[&id].cast.as_ref().unwrap().completes;
                events.push(event);
                Ok(Outcome::Started { completes })
            }
        })()
        .unwrap_or_else(Outcome::Rejected);
        let reply = Reply {
            sequence: self.actors[&id].sequence,
            outcome,
        };
        self.actors.get_mut(&id).unwrap().reply = Some(reply);
        CastResult { reply, events }
    }
    fn cancel(&mut self, key: EntityKey, sequence: u64, reason: Reason) {
        self.events.push(Event::Cancelled {
            actor: key,
            sequence,
            reason,
        });
    }
    fn advance(&mut self, world: &mut World, before: &BTreeMap<u64, GlobalPosition>) {
        self.events.clear();
        for creature in &mut self.creatures {
            if creature.respawn_at.is_some_and(|at| at <= world.tick) {
                let Some(next) = world.next_id.checked_add(1) else {
                    continue;
                };
                let previous = creature.key;
                creature.key.entity = theia_core::identity::EntityId::new(next).unwrap();
                creature.current = creature.maximum;
                creature.respawn_at = None;
                world.next_id = next;
                world.targets.insert(
                    next,
                    crate::targeting::Target {
                        key: creature.key,
                        position: creature.position,
                    },
                );
                self.events.push(Event::Respawned {
                    previous,
                    target: creature.key,
                    position: creature.position,
                });
            }
        }
        let ids: Vec<_> = self.actors.keys().copied().collect();
        for id in ids {
            let Some(key) = world.entity_key(id) else {
                self.actors.remove(&id);
                continue;
            };
            let moved = before.get(&id).is_some_and(|old| {
                let a = old.metres();
                let b = world.players[&id].position.metres();
                a.into_iter()
                    .zip(b)
                    .map(|(a, b)| (a - b) * (a - b))
                    .sum::<f64>()
                    > 0.000001
            });
            if self.actors[&id].interrupted {
                let actor = self.actors.get_mut(&id).unwrap();
                let cast = actor.cast.take();
                let queued = actor.queued.take();
                actor.interrupted = false;
                if let Some(cast) = cast {
                    self.cancel(key, cast.sequence, Reason::Interrupted);
                }
                if let Some(queued) = queued {
                    self.cancel(key, queued.request.sequence, Reason::Interrupted);
                }
                continue;
            }
            if let Some(cast) = self.actors[&id].cast.clone() {
                let ability = self.catalog.ability(cast.ability.as_str()).unwrap();
                let failure = if !Self::ready(world, id) {
                    Some(Reason::NotReady)
                } else if !self
                    .creatures
                    .iter()
                    .any(|c| c.key == cast.target && c.current > 0)
                {
                    Some(Reason::Dead)
                } else if !ability.cast_while_moving()
                    && (moved
                        || world.players[&id].intent.axes != [0, 0]
                        || world.players[&id].body.vertical_speed().abs() > 0.001)
                {
                    Some(Reason::Moving)
                } else if world.tick >= cast.completes {
                    self.validate(world, id, cast.target, ability, moved).err()
                } else {
                    None
                };
                if let Some(reason) = failure {
                    self.actors.get_mut(&id).unwrap().cast = None;
                    self.cancel(key, cast.sequence, reason);
                } else if world.tick >= cast.completes {
                    let Effect::Damage { amount } = ability.effect();
                    let creature = self
                        .creatures
                        .iter_mut()
                        .find(|c| c.key == cast.target)
                        .unwrap();
                    let applied = amount.min(creature.current);
                    creature.current -= applied;
                    self.events.push(Event::Damage {
                        ability: cast.ability.clone(),
                        actor: key,
                        sequence: cast.sequence,
                        target: cast.target,
                        position: creature.position,
                        amount: applied,
                        remaining: creature.current,
                    });
                    if creature.current == 0 {
                        let at = world.tick.saturating_add(self.respawn_ticks);
                        creature.respawn_at = Some(at);
                        world.retire_target(cast.target);
                        self.events.push(Event::Died {
                            target: cast.target,
                            position: creature.position,
                            respawn_at: at,
                        });
                    }
                    self.actors.get_mut(&id).unwrap().cast = None;
                }
            }
            if let Some(queued) = self.actors[&id].queued.clone() {
                let ability = self
                    .catalog
                    .ability(queued.request.ability.as_str())
                    .unwrap();
                let dead = !self
                    .creatures
                    .iter()
                    .any(|c| c.key == queued.request.target && c.current > 0);
                if dead || !Self::ready(world, id) {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    self.cancel(
                        key,
                        queued.request.sequence,
                        if dead { Reason::Dead } else { Reason::NotReady },
                    );
                } else if self.available(id, ability, world.tick) <= world.tick {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    match self.validate(world, id, queued.request.target, ability, moved) {
                        Err(reason) => self.cancel(key, queued.request.sequence, reason),
                        Ok(()) => {
                            let sequence = queued.request.sequence;
                            match self.start(world, id, queued.request) {
                                Ok(event) => self.events.push(event),
                                Err(reason) => self.cancel(key, sequence, reason),
                            }
                        }
                    }
                } else if world.tick >= queued.at {
                    self.actors.get_mut(&id).unwrap().queued = None;
                    self.cancel(key, queued.request.sequence, Reason::Unavailable);
                }
            }
            self.actors
                .get_mut(&id)
                .unwrap()
                .cooldowns
                .retain(|_, at| *at > world.tick);
        }
        // A later actor may kill a target after an earlier actor was already
        // visited. Publish one coherent completed-step state, with every cast
        // on that retired life cancelled in this same step.
        for id in self.actors.keys().copied().collect::<Vec<_>>() {
            let actor = self.actors.get_mut(&id).unwrap();
            let dead_cast = actor.cast.as_ref().is_some_and(|cast| {
                !self
                    .creatures
                    .iter()
                    .any(|c| c.key == cast.target && c.current > 0)
            });
            let dead_queue = actor.queued.as_ref().is_some_and(|queued| {
                !self
                    .creatures
                    .iter()
                    .any(|c| c.key == queued.request.target && c.current > 0)
            });
            let cast = if dead_cast { actor.cast.take() } else { None };
            let queued = if dead_queue {
                actor.queued.take()
            } else {
                None
            };
            let key = world.entity_key(id).unwrap();
            if let Some(cast) = cast {
                self.cancel(key, cast.sequence, Reason::Dead);
            }
            if let Some(queued) = queued {
                self.cancel(key, queued.request.sequence, Reason::Dead);
            }
        }
        debug_assert!(self.events.len() <= MAX_STEP_EVENTS);
    }
}
impl World {
    pub fn enable_combat(
        &mut self,
        catalog: &[u8],
        health: u32,
        respawn_ms: u32,
    ) -> crate::Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.procedural.is_none()
            || self.targets.is_empty()
            || self.combat.is_some()
        {
            return Err(
                "Combat requires inactive graybox world with prepared development targets".into(),
            );
        }
        self.combat = Some(Authority::create(self, catalog, health, respawn_ms)?);
        Ok(())
    }
    pub fn combat_enabled(&self) -> bool {
        self.combat.is_some()
    }
    pub fn combat_catalog(&self) -> Option<&Catalog> {
        self.combat.as_ref().map(|c| &c.catalog)
    }
    pub fn cast(&mut self, actor: u64, request: Request) -> CastResult {
        let Some(mut authority) = self.combat.take() else {
            return CastResult {
                reply: Reply {
                    sequence: request.sequence,
                    outcome: Outcome::Ignored,
                },
                events: vec![],
            };
        };
        let result = authority.request(self, actor, request);
        self.combat = Some(authority);
        result
    }
    /// Events from the most recently completed fixed step; not a reliable outbox.
    /// A transport adopting combat must publish them before advancing again.
    pub fn combat_events(&self) -> &[Event] {
        self.combat.as_ref().map_or(&[], |c| &c.events)
    }
    pub fn combat_snapshot(&self, actor: u64) -> Option<Snapshot> {
        let focus = self.players.get(&actor)?.position;
        let authority = self.combat.as_ref()?;
        let state = authority.actors.get(&actor);
        let mut targets: Vec<_> = authority
            .creatures
            .iter()
            .filter(|c| crate::targeting::relevant(focus, c.position))
            .cloned()
            .collect();
        targets.sort_by_key(|c| c.key.entity);
        Some(Snapshot {
            tick: self.tick,
            reply: state.and_then(|a| a.reply),
            cast: state.and_then(|a| a.cast.clone()),
            queued: state.and_then(|a| a.queued.clone()),
            gcd_until: state.map_or(0, |a| a.gcd_until),
            cooldowns: state.map_or_else(Vec::new, |a| {
                a.cooldowns
                    .iter()
                    .map(|(id, at)| (id.clone(), *at))
                    .collect()
            }),
            targets,
        })
    }
    pub(super) fn advance_combat(&mut self, before: BTreeMap<u64, GlobalPosition>) {
        if let Some(mut authority) = self.combat.take() {
            authority.advance(self, &before);
            self.combat = Some(authority);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;
    use std::sync::Arc;
    #[test]
    fn actual_world_sight_rejects_a_wall_at_start_and_rechecks_it_at_completion() {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let target = world.spawn_development_target([8008., 8000.]).unwrap();
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        let actor = world.join().unwrap();
        let position = world.global_position(actor).unwrap();
        let original = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(position)
            .unwrap()
            .clone();
        assert!(world.client_ready(
            actor,
            &crate::wire::ChunkReady {
                version: crate::wire::VERSION,
                epoch: world.epoch(),
                sequence: 1,
                chunks: original.ids.iter().map(|id| [id.x, id.z]).collect()
            }
        ));
        let from = Vec3::from_array(original.frame.to_local(position).unwrap().metres());
        let to = Vec3::from_array(
            original
                .frame
                .to_local(world.targets[&target.entity.value()].position)
                .unwrap()
                .metres(),
        );
        let mut geometry = original.scene.colliders.clone();
        geometry.push(crate::physics::PlacedCollider {
            shape: avian3d::prelude::Collider::cuboid(0.1, 8., 16.),
            center: (from + to) / 2.,
            layer: crate::physics::SOLID,
        });
        let wall = Arc::new(crate::procedural::Patch {
            owner: original.owner,
            frame: original.frame,
            ids: original.ids.clone(),
            scene: crate::physics::StaticScene::new(geometry),
        });
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, wall.clone());
        let request = |world: &World, sequence| Request {
            epoch: world.epoch(),
            sequence,
            ability: ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target,
        };
        assert_eq!(
            world.cast(actor, request(&world, 1)).reply.outcome,
            Outcome::Rejected(Reason::Obstructed)
        );
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, original.clone());
        assert!(matches!(
            world.cast(actor, request(&world, 2)).reply.outcome,
            Outcome::Started { .. }
        ));
        world
            .procedural
            .as_mut()
            .unwrap()
            .patches
            .insert(original.owner, wall);
        for _ in 0..30 {
            world.step();
        }
        assert!(matches!(
            world.combat_events(),
            [Event::Cancelled {
                reason: Reason::Obstructed,
                ..
            }]
        ));
        assert_eq!(
            world.combat_snapshot(actor).unwrap().targets[0].current,
            100
        );
    }
}
