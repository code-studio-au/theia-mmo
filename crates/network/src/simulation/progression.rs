//! Bounded fatal-hit facts; durable award work occurs outside the fixed step.
use super::{World, combat::Event};
use crate::{Result, progression_wire::Profile};
use std::sync::Arc;
use theia_core::{
    identity::EntityKey,
    progression::{KillEvent, KillId},
};
pub const FACT_CAPACITY: usize = 16;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opportunity {
    pub actor: EntityKey,
    pub event: KillEvent,
}
impl World {
    pub fn enable_progression(&mut self, profile: Arc<Profile>) -> Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_none()
            || self.progression.is_some()
        {
            return Err("Progression requires inactive combat world".into());
        }
        let definition = self
            .patrol
            .as_ref()
            .map(|patrol| patrol.definition())
            .or_else(|| {
                self.population_binding
                    .as_ref()
                    .map(|binding| binding.definition().id())
            })
            .ok_or("Progression requires authoritative creature")?;
        if profile
            .profile()
            .kill_rewards()
            .any(|(id, _)| id != definition)
        {
            return Err("Progression creature differs from installed creature definition".into());
        }
        self.progression = Some(profile);
        Ok(())
    }
    pub fn progression_profile(&self) -> Option<&Arc<Profile>> {
        self.progression.as_ref()
    }
    pub fn experience_opportunities(&self, events: &[Event]) -> Result<Vec<Opportunity>> {
        let Some(profile) = &self.progression else {
            return Ok(Vec::new());
        };
        let mut facts = Vec::new();
        for event in events {
            let Some((actor, target)) = self.kill_recipient(event)? else {
                continue;
            };
            let Some(definition) = self.combat_creature_definition(target) else {
                continue;
            };
            if self.entity_key(actor.entity.value()) != Some(actor)
                || profile.profile().kill_experience(definition.id()).is_none()
            {
                continue;
            }
            let fact = Opportunity {
                actor,
                event: KillEvent {
                    id: KillId::from_victim(target),
                    creature: definition.id().clone(),
                },
            };
            if facts.contains(&fact) {
                continue;
            }
            if facts.len() == FACT_CAPACITY {
                return Err("XP fact budget exceeded".into());
            }
            facts.push(fact);
        }
        Ok(facts)
    }
}
