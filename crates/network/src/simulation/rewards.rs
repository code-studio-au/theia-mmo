//! Bounded trusted kill facts only. No grants, persistent IDs or awaited work in
//! the fixed step. Once-per-character consumption belongs to the durable owner.
use super::{World, combat::Event};
use crate::{Result, reward_wire::Profile};
use std::sync::Arc;
use theia_core::{content::ContentId, identity::EntityKey, rewards::MAX_DEFINITIONS};
pub fn profile_identity(base: [u8; 32], rewards: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-reward-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(rewards);
    crate::terrain::sha256(&bytes)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opportunity {
    pub actor: EntityKey,
    pub reward: ContentId,
}
impl World {
    pub fn enable_rewards(&mut self, profile: Arc<Profile>) -> Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_none()
            || self.rewards.is_some()
        {
            return Err("Rewards require inactive combat world".into());
        }
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Rewards require an authoritative creature")?;
        if profile
            .catalog()
            .definitions()
            .any(|d| d.creature() != patrol.definition())
        {
            return Err("Reward definition does not match installed creature".into());
        }
        self.rewards = Some(profile);
        Ok(())
    }
    pub fn reward_profile(&self) -> Option<&Arc<Profile>> {
        self.rewards.as_ref()
    }
    /// Last-hit credit is an explicit development policy. The source must still
    /// be a live admitted player; static targets and stale creature lives reject.
    pub fn reward_opportunities(&self, events: &[Event]) -> Result<Vec<Opportunity>> {
        let Some(profile) = &self.rewards else {
            return Ok(Vec::new());
        };
        let Some(patrol) = &self.patrol else {
            return Err("Reward creature missing".into());
        };
        let mut result = Vec::new();
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
            if *target != patrol.key || self.entity_key(actor.entity.value()) != Some(*actor) {
                continue;
            }
            for definition in profile
                .catalog()
                .definitions()
                .filter(|d| d.creature() == patrol.definition())
            {
                let opportunity = Opportunity {
                    actor: *actor,
                    reward: definition.id().clone(),
                };
                if result.contains(&opportunity) {
                    continue;
                }
                if result.len() == MAX_DEFINITIONS {
                    return Err("Reward opportunity budget exceeded".into());
                }
                result.push(opportunity);
            }
        }
        Ok(result)
    }
}
