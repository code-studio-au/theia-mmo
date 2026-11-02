//! Fixed-step, bounded travel. No I/O or transport work occurs here.
use super::{Authority, Event, Hit, ProjectileEnd, World};
use theia_core::{
    coordinates::GlobalPosition,
    player_life::LifeKey,
    projectiles::{Flight, Profile, State},
};
pub(super) const MAX_ACTIVE: usize = 16;
const MAX_PER_ACTOR: usize = 4;
pub(super) struct Active {
    hit: Hit,
    source_life: Option<LifeKey>,
    flight: Flight,
    pub(super) clear_position: GlobalPosition,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub launched: u64,
    pub impacts: u64,
    pub expired: u64,
    pub obstructed: u64,
    pub unavailable: u64,
    pub invalid_sources: u64,
    pub invalid_targets: u64,
    pub capacity_refusals: u64,
    pub peak_active: usize,
}
enum Step {
    Continue,
    Impact,
    End(ProjectileEnd),
}
fn anchor(position: GlobalPosition) -> Option<GlobalPosition> {
    let mut point = position.metres();
    point[1] += 0.6;
    GlobalPosition::new(point)
}
fn advance_one(
    active: &mut Active,
    world: &World,
    source: Option<Option<LifeKey>>,
    target: Option<GlobalPosition>,
) -> Step {
    if source != Some(active.source_life) {
        return Step::End(ProjectileEnd::SourceGone);
    }
    let Some(target) = target.and_then(anchor) else {
        return Step::End(ProjectileEnd::TargetGone);
    };
    let Some(segment) = active.flight.advance(target) else {
        return Step::End(ProjectileEnd::Expired);
    };
    let Some(patch) = world
        .procedural
        .as_ref()
        .and_then(|r| r.patch(segment.from))
    else {
        return Step::End(ProjectileEnd::Unavailable);
    };
    if !patch.covers(segment.to, None) {
        return Step::End(ProjectileEnd::Unavailable);
    }
    let (Some(from), Some(to)) = (
        patch.frame.to_local(segment.from),
        patch.frame.to_local(segment.to),
    ) else {
        return Step::End(ProjectileEnd::Unavailable);
    };
    if !patch.scene.projectile_clear(
        bevy::prelude::Vec3::from_array(from.metres()),
        bevy::prelude::Vec3::from_array(to.metres()),
    ) {
        return Step::End(ProjectileEnd::Obstructed);
    }
    active.clear_position = segment.to;
    match segment.state {
        State::Travelling => Step::Continue,
        State::Arrived => Step::Impact,
        State::Expired => Step::End(ProjectileEnd::Expired),
    }
}
impl Authority {
    pub(super) fn step_event_budget(&self) -> usize {
        if self.catalog.schema() >= 6 {
            super::MAX_PROJECTILE_STEP_EVENTS
        } else {
            super::MAX_STEP_EVENTS
        }
    }
    pub(super) fn projectile_capacity(&self, id: u64) -> bool {
        let reserved = self.actors.iter().filter(|(_, actor)| {
            actor.cast.as_ref().is_some_and(|cast| {
                self.catalog
                    .ability(cast.ability.as_str())
                    .is_some_and(|ability| ability.projectile().is_some())
            })
        });
        let total = self.projectiles.len() + reserved.clone().count();
        let owned = self
            .projectiles
            .iter()
            .filter(|p| p.hit.actor.entity.value() == id)
            .count()
            + reserved.filter(|(owner, _)| **owner == id).count();
        total < MAX_ACTIVE && owned < MAX_PER_ACTOR
    }
    pub(super) fn launch_projectile(&mut self, world: &World, hit: Hit, profile: Profile) {
        let positions = world
            .global_position(hit.actor.entity.value())
            .and_then(anchor)
            .zip(
                self.creatures
                    .iter()
                    .find(|c| c.key == hit.target && c.current > 0)
                    .and_then(|c| anchor(c.position)),
            );
        let Some((position, target_position)) = positions else {
            self.cancel(hit.actor, hit.sequence, super::Reason::Unavailable);
            return;
        };
        // Start reserved this slot before spending mana. Fail closed if an
        // invariant ever breaks; never grow the authoritative collection.
        if self.projectiles.len() >= MAX_ACTIVE {
            self.cancel(hit.actor, hit.sequence, super::Reason::Unavailable);
            return;
        }
        self.events.push(Event::ProjectileLaunched {
            ability: hit.ability.clone(),
            actor: hit.actor,
            sequence: hit.sequence,
            target: hit.target,
            position,
            target_position,
        });
        let source_life = self
            .actors
            .get(&hit.actor.entity.value())
            .and_then(|a| a.life.as_ref())
            .map(|p| p.state.key());
        self.projectiles.push(Active {
            hit,
            source_life,
            flight: Flight::launch(profile, position, world.rate),
            clear_position: position,
        });
        self.projectile_metrics.launched += 1;
        self.projectile_metrics.peak_active = self
            .projectile_metrics
            .peak_active
            .max(self.projectiles.len());
    }
    pub(super) fn advance_projectiles(&mut self, world: &mut World) {
        for index in (0..self.projectiles.len()).rev() {
            let active = &mut self.projectiles[index];
            let id = active.hit.actor.entity.value();
            let source = (world.entity_key(id) == Some(active.hit.actor)
                && (self.pve.is_none()
                    || self
                        .actors
                        .get(&id)
                        .and_then(|a| a.life.as_ref())
                        .is_some_and(|p| p.state.alive())))
            .then(|| {
                self.actors
                    .get(&id)
                    .and_then(|a| a.life.as_ref())
                    .map(|p| p.state.key())
            });
            let target = self
                .creatures
                .iter()
                .find(|c| c.key == active.hit.target && c.current > 0)
                .map(|c| c.position);
            match advance_one(active, world, source, target) {
                Step::Continue => {}
                Step::Impact => {
                    let active = self.projectiles.swap_remove(index);
                    self.projectile_metrics.impacts += 1;
                    let table = self
                        .catalog
                        .ability(active.hit.ability.as_str())
                        .and_then(|a| a.hit_table());
                    self.resolve_damage(world, active.hit, table);
                }
                Step::End(reason) => {
                    let active = self.projectiles.swap_remove(index);
                    let count = match reason {
                        ProjectileEnd::Expired => &mut self.projectile_metrics.expired,
                        ProjectileEnd::Obstructed => &mut self.projectile_metrics.obstructed,
                        ProjectileEnd::Unavailable => &mut self.projectile_metrics.unavailable,
                        ProjectileEnd::SourceGone => &mut self.projectile_metrics.invalid_sources,
                        ProjectileEnd::TargetGone => &mut self.projectile_metrics.invalid_targets,
                    };
                    *count += 1;
                    self.events.push(Event::ProjectileEnded {
                        ability: active.hit.ability,
                        actor: active.hit.actor,
                        sequence: active.hit.sequence,
                        target: active.hit.target,
                        position: active.clear_position,
                        reason,
                    });
                }
            }
        }
    }
}
impl World {
    pub fn projectile_metrics(&self) -> Option<Metrics> {
        self.combat
            .as_ref()
            .filter(|c| c.catalog.schema() >= 6)
            .map(|c| c.projectile_metrics.clone())
    }
    pub fn combat_step_event_budget(&self) -> usize {
        self.combat
            .as_ref()
            .map_or(super::MAX_STEP_EVENTS, Authority::step_event_budget)
    }
}
