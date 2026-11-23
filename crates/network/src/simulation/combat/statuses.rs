//! Bounded transient periodic effects. Cast completion precedes due ticks, then
//! the parent cancels all casts against retired target lives in the same step.
use super::{Authority, Cast, Health, Hit, Reason};
use crate::simulation::World;
use std::collections::BTreeMap;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::EntityKey,
    statuses::{Catalog, Effect, Schedule},
    time::TickRate,
};
pub const MAX_ACTIVE_STATUSES: usize = 16;
pub const MAX_TARGET_STATUSES: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub source: EntityKey,
    pub target: EntityKey,
    pub status: ContentId,
    pub expires_at: u64,
    pub next_tick: Option<u64>,
}
type Key = (u64, ContentId, u64); // deterministic target, definition, source order
struct Instance {
    source: EntityKey,
    target: EntityKey,
    status: ContentId,
    ability: ContentId,
    sequence: u64,
    schedule: Schedule,
}
pub(super) struct Registry {
    pub(super) catalog: Catalog,
    active: BTreeMap<Key, Instance>,
}
fn key(source: EntityKey, target: EntityKey, status: &ContentId) -> Key {
    (target.entity.value(), status.clone(), source.entity.value())
}
impl Registry {
    pub(super) fn new(catalog: Catalog) -> Self {
        Self {
            catalog,
            active: BTreeMap::new(),
        }
    }
    pub(super) fn can_apply(
        &self,
        source: EntityKey,
        target: EntityKey,
        status: &ContentId,
    ) -> Result<(), Reason> {
        let effect = self
            .catalog
            .definition(status)
            .ok_or(Reason::Unavailable)?
            .effect();
        if effect.periodic() == (source == target) {
            return Err(Reason::UnknownTarget);
        }
        if let Some(instance) = self.active.get(&key(source, target, status)) {
            return if instance.source == source && instance.target == target {
                Ok(())
            } else {
                Err(Reason::UnknownTarget)
            };
        }
        if self.active.len() >= MAX_ACTIVE_STATUSES
            || self.active.values().filter(|s| s.target == target).count() >= MAX_TARGET_STATUSES
        {
            return Err(Reason::StatusCapacity);
        }
        Ok(())
    }
    pub(super) fn apply(
        &mut self,
        now: u64,
        rate: TickRate,
        source: EntityKey,
        cast: &Cast,
        status: &ContentId,
    ) -> Result<(), Reason> {
        self.can_apply(source, cast.target, status)?;
        let key = key(source, cast.target, status);
        if let Some(instance) = self.active.get_mut(&key) {
            instance
                .schedule
                .refresh(now)
                .map_err(|_| Reason::TimeExhausted)?;
            instance.ability = cast.ability.clone();
            instance.sequence = cast.sequence;
        } else {
            let timing = self.catalog.definition(status).unwrap().timing(rate);
            let schedule = Schedule::start(now, timing).map_err(|_| Reason::TimeExhausted)?;
            self.active.insert(
                key,
                Instance {
                    source,
                    target: cast.target,
                    status: status.clone(),
                    ability: cast.ability.clone(),
                    sequence: cast.sequence,
                    schedule,
                },
            );
        }
        Ok(())
    }
    pub(super) fn remove_source(&mut self, id: u64) {
        self.active.retain(|_, s| s.source.entity.value() != id);
    }
    pub(super) fn remove_target(&mut self, key: EntityKey) -> usize {
        let before = self.active.len();
        self.active.retain(|_, s| s.target != key);
        before - self.active.len()
    }
    fn effect(&self, instance: &Instance) -> Effect {
        self.catalog.definition(&instance.status).unwrap().effect()
    }
    pub(super) fn views(
        &self,
        owner: Option<EntityKey>,
        focus: GlobalPosition,
        creatures: &[Health],
    ) -> Vec<Status> {
        self.active
            .values()
            .filter(|s| {
                Some(s.target) == owner
                    || creatures.iter().any(|c| {
                        c.key == s.target
                            && c.current > 0
                            && crate::targeting::relevant(focus, c.position)
                    })
            })
            .map(|s| Status {
                source: s.source,
                target: s.target,
                status: s.status.clone(),
                expires_at: s.schedule.expires_at(),
                next_tick: self
                    .effect(s)
                    .periodic()
                    .then(|| s.schedule.next_tick())
                    .flatten(),
            })
            .collect()
    }
}
impl Authority {
    pub(super) fn advance_statuses(&mut self, world: &mut World) {
        let Some(mut statuses) = self.statuses.take() else {
            return;
        };
        let keys: Vec<_> = statuses.active.keys().cloned().collect();
        for key in keys {
            let Some(mut instance) = statuses.active.remove(&key) else {
                continue;
            };
            let effect = statuses.effect(&instance);
            let target_alive = if effect.periodic() {
                self.creatures
                    .iter()
                    .any(|c| c.key == instance.target && c.current > 0)
            } else {
                self.player_alive(instance.target.entity.value())
            };
            if world.entity_key(instance.source.entity.value()) != Some(instance.source)
                || !target_alive
            {
                continue;
            }
            let step = instance
                .schedule
                .advance(world.tick)
                .expect("Fixed steps visit every bounded status deadline");
            if step.damage_due
                && let Effect::PeriodicDamage { amount } = effect
            {
                self.damage(
                    world,
                    Hit {
                        credit_encounter: None,
                        ability: instance.ability.clone(),
                        actor: instance.source,
                        sequence: instance.sequence,
                        target: instance.target,
                        amount,
                    },
                );
            }
            if effect.periodic()
                && !self
                    .creatures
                    .iter()
                    .any(|c| c.key == instance.target && c.current > 0)
            {
                statuses.remove_target(instance.target);
            } else if !step.expired {
                statuses.active.insert(key, instance);
            }
        }
        self.statuses = Some(statuses);
    }
}
