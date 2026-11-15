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
        self.install_rewards(profile, false, false)
    }
    /// Explicit headless loot adoption. Default content/manifest admission stays
    /// unchanged until the normal client pickup flow is qualified.
    pub fn enable_creature_loot(&mut self, profile: Arc<Profile>) -> Result<()> {
        self.install_rewards(profile, true, false)
    }
    /// Explicit headless conditional mode. Eligibility remains a locked durable
    /// owner decision; the fixed step only forwards trusted fatal-life facts.
    pub fn enable_quest_loot(&mut self, profile: Arc<Profile>) -> Result<()> {
        if !profile.catalog().has_quest_required()
            || profile.quest_journal_hash().is_none()
            || self.quest_journal.as_ref().is_none_or(|j| {
                Some(j.hash()) != profile.quest_journal_hash()
                    || j.inventory_hash() != Some(profile.item_hash())
            })
        {
            return Err("Quest loot requires the exact installed item journal".into());
        }
        self.install_rewards(profile, true, true)
    }
    fn install_rewards(
        &mut self,
        profile: Arc<Profile>,
        allow_life: bool,
        allow_quest: bool,
    ) -> Result<()> {
        if profile.catalog().has_quest_required() && !allow_quest {
            return Err("Quest-only loot simulation is not installed".into());
        }
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_none()
            || self.rewards.is_some()
        {
            return Err("Rewards require inactive combat world".into());
        }
        if !allow_life
            && profile
                .catalog()
                .definitions()
                .any(|d| d.scope() == theia_core::rewards::Scope::PerCreatureLife)
        {
            return Err("Per-life corpse interaction and pickup are not installed".into());
        }
        let creature = self
            .patrol
            .as_ref()
            .map(|patrol| patrol.definition())
            .or_else(|| {
                self.population_binding
                    .as_ref()
                    .map(|binding| binding.definition().id())
            })
            .ok_or("Rewards require an authoritative creature")?;
        if profile
            .catalog()
            .definitions()
            .any(|d| d.ground().is_none() && d.creature() != creature)
        {
            return Err("Reward definition does not match installed creature".into());
        }
        for ground in profile.catalog().definitions().filter_map(|d| d.ground()) {
            let patch = self
                .procedural
                .as_ref()
                .and_then(|r| r.patch(ground.position))
                .ok_or("Ground item collision is not prepared")?;
            crate::ground_placement::supported(patch, ground.position)?;
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
    pub fn ground_pickup_eligible(
        &self,
        actor: u64,
        position: theia_core::coordinates::GlobalPosition,
    ) -> bool {
        self.player_alive(actor)
            && self
                .global_position(actor)
                .is_some_and(|p| crate::dialogue::in_range(p, position))
            && self.players.get(&actor).is_some_and(|a| {
                self.procedural
                    .as_ref()
                    .and_then(|r| r.patch(a.position))
                    .is_some_and(|p| p.covers(a.position, a.ready.as_ref()))
            })
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
        let mut result = Vec::new();
        for event in events {
            let Some((actor, target)) = self.kill_recipient(event)? else {
                continue;
            };
            let Some(creature) = self.combat_creature_definition(target) else {
                continue;
            };
            if self.entity_key(actor.entity.value()) != Some(actor) {
                continue;
            }
            let Some(position) = self.combat_creature_position(target) else {
                continue;
            };
            for definition in profile
                .catalog()
                .definitions()
                .filter(|d| d.ground().is_none() && d.creature() == creature.id())
            {
                let opportunity = Opportunity {
                    actor,
                    reward: definition.id().clone(),
                    event: KillEvent {
                        id: KillId::from_victim(target),
                        creature: creature.id().clone(),
                    },
                    position,
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
#[path = "rewards/quest_tests.rs"]
mod quest_tests;

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
