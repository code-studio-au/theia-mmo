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
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Progression requires authoritative creature")?;
        if profile
            .profile()
            .kill_rewards()
            .any(|(id, _)| id != patrol.definition())
        {
            return Err("Progression creature differs from installed patrol".into());
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
        let patrol = self.patrol.as_ref().ok_or("Progression creature missing")?;
        let mut facts = Vec::new();
        for event in events {
            let Event::Damage {
                actor,
                target,
                remaining: 0,
                ..
            } = event
            else {
                continue;
            };
            if *target != patrol.key
                || self.entity_key(actor.entity.value()) != Some(*actor)
                || profile
                    .profile()
                    .kill_experience(patrol.definition())
                    .is_none()
            {
                continue;
            }
            let fact = Opportunity {
                actor: *actor,
                event: KillEvent {
                    id: KillId::from_victim(*target),
                    creature: patrol.definition().clone(),
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
