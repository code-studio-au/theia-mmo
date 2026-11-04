//! Bounded trusted kill facts only. No grants, persistent IDs or awaited work in
//! the fixed step. Once-per-character consumption belongs to the durable owner.
use super::{World, combat::Event};
use crate::{Result, reward_wire::Profile};
use std::sync::Arc;
use theia_core::{
    content::ContentId,
    identity::EntityKey,
    progression::{KillEvent, KillId},
    rewards::MAX_DEFINITIONS,
};
pub fn profile_identity(base: [u8; 32], rewards: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-reward-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(rewards);
    crate::terrain::sha256(&bytes)
}
#[derive(Clone, Debug, PartialEq)]
pub struct Opportunity {
    pub actor: EntityKey,
    pub reward: ContentId,
    /// Original server-owned fatal life and position, never supplied by a client.
    pub event: KillEvent,
    pub position: theia_core::coordinates::GlobalPosition,
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
        if profile
            .catalog()
            .definitions()
            .any(|d| d.scope() == theia_core::rewards::Scope::PerCreatureLife)
        {
            return Err("Per-life corpse interaction and pickup are not installed".into());
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
    /// Rechecked at pickup acceptance, using only private server-owned source
    /// metadata. Three metres is the current Theia interaction policy, not an
    /// assertion about an exact Classic corpse-loot distance.
    pub fn corpse_pickup_eligible(
        &self,
        id: u64,
        profile_hash: [u8; 32],
        source: theia_core::rewards::CorpseSource,
    ) -> bool {
        let Some(actor) = self.players.get(&id) else {
            return false;
        };
        self.rewards
            .as_ref()
            .is_some_and(|p| p.hash() == profile_hash)
            && self.player_alive(id)
            && (!self.client_chunks_required || actor.ready.is_some())
            && self
                .procedural
                .as_ref()
                .and_then(|r| r.patch(actor.position))
                .is_some_and(|p| p.covers(actor.position, actor.ready.as_ref()))
            && crate::dialogue::in_range(actor.position, source.position)
    }
    pub fn reward_profile(&self) -> Option<&Arc<Profile>> {
        self.rewards.as_ref()
    }
    /// Older modes retain last-hit development credit. Explicit solo credit
    /// resolves the retained tag owner; static/stale creature lives refuse.
    /// These remain nonrepeatable training offers, not general corpse loot.
    pub fn reward_opportunities(&self, events: &[Event]) -> Result<Vec<Opportunity>> {
        let Some(profile) = &self.rewards else {
            return Ok(Vec::new());
        };
        let Some(patrol) = &self.patrol else {
            return Err("Reward creature missing".into());
        };
        let mut result = Vec::new();
        for event in events {
            let Some((actor, target)) = self.kill_recipient(event)? else {
                continue;
            };
            if target != patrol.key || self.entity_key(actor.entity.value()) != Some(actor) {
                continue;
            }
            for definition in profile
                .catalog()
                .definitions()
                .filter(|d| d.creature() == patrol.definition())
            {
                let opportunity = Opportunity {
                    actor,
                    reward: definition.id().clone(),
                    event: KillEvent {
                        id: KillId::from_victim(target),
                        creature: patrol.definition().clone(),
                    },
                    position: patrol.position(),
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

#[cfg(test)]
mod pickup_tests {
    use super::*;
    use theia_core::{
        coordinates::GlobalPosition,
        identity::{EntityId, WorldEpoch},
        rewards::CorpseSource,
    };
    #[test]
    fn pickup_requires_current_loaded_actor_matching_profile_and_three_dimensional_range() {
        let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/gameplay/development-rewards.json"
        ))
        .unwrap();
        value["schema"] = serde_json::json!(2);
        value["rewards"][0]["scope"] = serde_json::json!("per_creature_life");
        let profile = Arc::new(
            Profile::from_json_with_creature_loot(&serde_json::to_vec(&value).unwrap(), &items)
                .unwrap(),
        );
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        let id = world.join().unwrap();
        let position = world.global_position(id).unwrap();
        let source = CorpseSource {
            // A durable corpse from an older world remains a valid personal
            // entitlement; it must not alias the current live creature entity.
            life: KillId::from_victim(EntityKey {
                epoch: WorldEpoch::new(17).unwrap(),
                entity: EntityId::new(88).unwrap(),
            }),
            position,
        };
        assert!(!world.corpse_pickup_eligible(id, profile.hash(), source));
        // Synthetic guard fixture only: ordinary World admission remains gated.
        world.rewards = Some(profile.clone());
        assert!(!world.corpse_pickup_eligible(id, profile.hash(), source));
        let actor = world.players.get_mut(&id).unwrap();
        actor.ready = Some(std::collections::BTreeSet::new());
        assert!(!world.corpse_pickup_eligible(id, profile.hash(), source));
        let ids = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(position)
            .unwrap()
            .ids
            .iter()
            .copied()
            .collect();
        world.players.get_mut(&id).unwrap().ready = Some(ids);
        assert!(world.corpse_pickup_eligible(id, profile.hash(), source));
        assert!(!world.corpse_pickup_eligible(id, [0; 32], source));
        let at_height = |offset| {
            let mut xyz = position.metres();
            xyz[1] += offset;
            CorpseSource {
                position: GlobalPosition::new(xyz).unwrap(),
                ..source
            }
        };
        assert!(world.corpse_pickup_eligible(id, profile.hash(), at_height(3.)));
        assert!(!world.corpse_pickup_eligible(id, profile.hash(), at_height(3.001)));
        world.leave(id);
        assert!(!world.corpse_pickup_eligible(id, profile.hash(), source));
    }
}
