//! Explicit normal-contact physical configuration and authority. Checked content
//! and identity are shared with CE06 and the schema-6 launcher. Ordinary outcome
//! tables use their separate opt-in profile and fact type.
use crate::{Result, simulation::creatures};
use std::collections::BTreeMap;
use theia_core::{
    abilities,
    content::ContentId,
    damage::{
        MAX_VALUE, Range, Weapon,
        catalog::{Attack, Catalog},
        physical::{Resolved, normal_reference_v1},
        stats,
    },
    inventory, melee,
};
pub struct Bindings<'a> {
    pub weapons: &'a melee::Catalog,
    pub items: &'a inventory::Catalog,
    pub abilities: &'a abilities::Catalog,
    pub creature: &'a creatures::Profile,
    pub player_maximum_health: u32,
    pub creature_maximum_health: u32,
}
#[derive(Clone, Debug)]
pub struct Profile {
    catalog: Catalog,
    stats: stats::Profile,
    weapons: BTreeMap<ContentId, (ContentId, Weapon)>,
    creature_definition: ContentId,
    creature_attack: ContentId,
    creature_damage: Range,
    player_maximum_health: u32,
    creature_maximum_health: u32,
    digest: [u8; 32],
}
impl Profile {
    pub fn new(definitions: &[u8], stat_data: &[u8], bindings: Bindings<'_>) -> Result<Self> {
        if bindings.abilities.schema() != 7
            || !(1..=MAX_VALUE).contains(&bindings.player_maximum_health)
            || !(1..=MAX_VALUE).contains(&bindings.creature_maximum_health)
        {
            return Err("Physical damage requires known ability rules and health bounds".into());
        }
        let catalog = Catalog::from_json(definitions)?;
        let stats = stats::Profile::from_json(stat_data)?;
        if stats.player().spell_bonus_damage() != 0
            || catalog
                .attacks()
                .any(|(_, a)| matches!(a, Attack::Spell { .. }))
        {
            return Err("Physical-result v1 cannot adopt spell definitions or spell bonus".into());
        }
        catalog.validate_references(
            bindings.items,
            bindings.abilities,
            std::slice::from_ref(bindings.creature.id()),
        )?;
        stats.validate_definitions(&catalog)?;
        let mut weapons = BTreeMap::new();
        let mut creature = None;
        for (id, attack) in catalog.attacks() {
            match attack {
                Attack::Weapon { item, damage } => {
                    weapons.insert(item.clone(), (id.clone(), *damage));
                }
                Attack::Creature {
                    creature: definition,
                    damage,
                } => {
                    creature = Some((id.clone(), definition.clone(), *damage));
                }
                Attack::Spell { .. } => unreachable!("Unsupported kind rejected before indexing"),
            }
        }
        if weapons.len() != bindings.weapons.weapons().len()
            || bindings.weapons.weapons().any(|w| {
                weapons
                    .get(w.item())
                    .is_none_or(|(_, damage)| damage.base_speed_ms() != w.period_ms())
            })
        {
            return Err(
                "Physical definitions must cover every melee weapon at its base speed".into(),
            );
        }
        let (creature_attack, creature_definition, creature_damage) =
            creature.ok_or("Physical results require one known creature attack")?;
        let mut identity = b"theia-physical-damage-config-v1\0".to_vec();
        identity.extend(crate::terrain::sha256(definitions));
        identity.extend(crate::terrain::sha256(stat_data));
        identity.extend(bindings.player_maximum_health.to_le_bytes());
        identity.extend(bindings.creature_maximum_health.to_le_bytes());
        Ok(Self {
            catalog,
            stats,
            weapons,
            creature_definition,
            creature_attack,
            creature_damage,
            player_maximum_health: bindings.player_maximum_health,
            creature_maximum_health: bindings.creature_maximum_health,
            digest: crate::terrain::sha256(&identity),
        })
    }
    pub fn context_hash(&self) -> [u8; 32] {
        self.digest
    }
    /// Apply after the existing ability/life/melee identity, before later roles.
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-physical-damage-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.digest);
        crate::terrain::sha256(&bytes)
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn stats(&self) -> &stats::Profile {
        &self.stats
    }
    pub fn player_maximum_health(&self) -> u32 {
        self.player_maximum_health
    }
    pub fn creature_maximum_health(&self) -> u32 {
        self.creature_maximum_health
    }
    pub fn weapon(&self, item: &ContentId) -> Option<(&ContentId, Weapon)> {
        self.weapons.get(item).map(|(id, damage)| (id, *damage))
    }
    pub fn creature_definition(&self) -> &ContentId {
        &self.creature_definition
    }
    pub fn creature_attack(&self) -> (&ContentId, Range) {
        (&self.creature_attack, self.creature_damage)
    }
    pub fn weapon_result(&self, item: &ContentId, roll: u32, health: u32) -> Result<Resolved> {
        if health > self.creature_maximum_health {
            return Err("Victim health exceeds physical creature bounds".into());
        }
        let (_, weapon) = self.weapon(item).ok_or("Unknown physical weapon")?;
        let player = self.stats.player();
        let defender = self
            .stats
            .creature(&self.creature_definition)
            .ok_or("Missing physical creature stats")?;
        Ok(normal_reference_v1(
            weapon.starting_damage(roll, player.attack_power())?,
            player.physical().level(),
            defender.armour(),
            player.physical().armour_penetration(),
            health,
        )?)
    }
    pub fn creature_result(&self, roll: u32, health: u32) -> Result<Resolved> {
        if health > self.player_maximum_health {
            return Err("Victim health exceeds physical player bounds".into());
        }
        let attacker = self
            .stats
            .creature(&self.creature_definition)
            .ok_or("Missing physical creature stats")?;
        let player = self.stats.player().physical();
        Ok(normal_reference_v1(
            self.creature_damage.starting_damage(roll)?,
            attacker.level(),
            player.armour(),
            attacker.armour_penetration(),
            health,
        )?)
    }
}
#[cfg(test)]
mod tests;

use super::{Authority, Event, outcomes};
use crate::simulation::World;
use std::{num::NonZeroU64, sync::Arc};
use theia_core::{
    coordinates::GlobalPosition,
    identity::EntityKey,
    player_life::{self, LifeKey},
};

/// Server-resolved hit facts. A player source/victim carries its life generation;
/// the creature uses its entity incarnation. No client command contains this type.
#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub attack: ContentId,
    pub actor: EntityKey,
    pub actor_generation: Option<NonZeroU64>,
    pub sequence: u64,
    pub target: EntityKey,
    pub target_generation: Option<NonZeroU64>,
    pub position: GlobalPosition,
    pub roll: u32,
    pub result: Resolved,
    pub remaining: u32,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub base_draws: u64,
    pub outgoing_hits: u64,
    pub incoming_hits: u64,
    pub outgoing_damage: u64,
    pub incoming_damage: u64,
}
pub(super) struct Runtime {
    pub(super) profile: Arc<Profile>,
    random: outcomes::Random,
    npc_sequence: u64,
    metrics: Metrics,
    pub(super) error: Option<String>,
}
struct Planned {
    random: outcomes::Random,
    roll: u32,
    result: Resolved,
    npc_sequence: Option<u64>,
}
impl Runtime {
    fn new(profile: Profile) -> Result<Self> {
        Ok(Self {
            profile: Arc::new(profile),
            random: outcomes::Random::new()?,
            npc_sequence: 0,
            metrics: Metrics::default(),
            error: None,
        })
    }
    fn weapon(&self, item: &ContentId, health: u32) -> Result<Planned> {
        if self.error.is_some() || health == 0 || health > self.profile.creature_maximum_health {
            return Err("Physical weapon victim is unavailable".into());
        }
        let (_, weapon) = self.profile.weapon(item).ok_or("Unknown physical weapon")?;
        // Preview on a copy. Rejection, event pressure or a failed life/reset
        // transaction cannot consume the authoritative stream.
        let mut random = self.random.clone();
        let roll = random.base_roll(weapon.range())?;
        let result = self.profile.weapon_result(item, roll, health)?;
        Ok(Planned {
            random,
            roll,
            result,
            npc_sequence: None,
        })
    }
    fn creature(&self, health: u32) -> Result<Planned> {
        if self.error.is_some() || health == 0 || health > self.profile.player_maximum_health {
            return Err("Physical player victim is unavailable".into());
        }
        let npc_sequence = self
            .npc_sequence
            .checked_add(1)
            .ok_or("Physical NPC sequence exhausted")?;
        let mut random = self.random.clone();
        let roll = random.base_roll(self.profile.creature_damage)?;
        let result = self.profile.creature_result(roll, health)?;
        Ok(Planned {
            random,
            roll,
            result,
            npc_sequence: Some(npc_sequence),
        })
    }
    fn commit(&mut self, plan: Planned, incoming: bool) {
        self.random = plan.random;
        if let Some(sequence) = plan.npc_sequence {
            self.npc_sequence = sequence;
        }
        self.metrics.base_draws = self.random.base_draws();
        if incoming {
            self.metrics.incoming_hits = self.metrics.incoming_hits.saturating_add(1);
            self.metrics.incoming_damage = self
                .metrics
                .incoming_damage
                .saturating_add(u64::from(plan.result.health_lost));
        } else {
            self.metrics.outgoing_hits = self.metrics.outgoing_hits.saturating_add(1);
            self.metrics.outgoing_damage = self
                .metrics
                .outgoing_damage
                .saturating_add(u64::from(plan.result.health_lost));
        }
    }
}
impl World {
    /// Install checked normal-contact physical authority before admission.
    /// Its explicit configuration/identity is shared with the CE06 transport.
    pub fn enable_physical_damage(
        &mut self,
        definitions: &[u8],
        stats: &[u8],
        items: &[u8],
    ) -> Result<()> {
        if !self.classic || self.tick != 0 || !self.players.is_empty() {
            return Err("Physical damage requires inactive classic authority".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Physical damage requires combat")?;
        let melee = combat
            .autoattack
            .as_ref()
            .ok_or("Physical damage requires basic melee")?;
        if combat.physical.is_some()
            || combat.pve.is_none()
            || crate::terrain::sha256(items) != melee.items_digest()
        {
            return Err(
                "Physical damage requires matching life/equipment and one configuration".into(),
            );
        }
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Physical damage requires a known patrol")?;
        if combat.creatures.len() != 1 || combat.creatures[0].key != patrol.key {
            return Err("Initial physical damage requires one identified creature".into());
        }
        let items = inventory::Catalog::from_json(items)?;
        let profile = Profile::new(
            definitions,
            stats,
            Bindings {
                weapons: melee.catalog(),
                items: &items,
                abilities: &combat.catalog,
                creature: patrol.profile(),
                player_maximum_health: combat
                    .resources
                    .as_ref()
                    .ok_or("Physical damage requires player vitals")?
                    .maximum_health(),
                creature_maximum_health: combat.creatures[0].maximum,
            },
        )?;
        let runtime = Runtime::new(profile)?;
        self.combat.as_mut().unwrap().physical = Some(runtime);
        Ok(())
    }
    pub fn physical_damage_profile(&self) -> Option<&Arc<Profile>> {
        self.combat.as_ref()?.physical.as_ref().map(|r| &r.profile)
    }
    pub fn physical_damage_metrics(&self) -> Option<Metrics> {
        self.combat.as_ref()?.physical.as_ref().map(|r| r.metrics)
    }
    pub fn physical_damage_error(&self) -> Option<&str> {
        self.combat.as_ref()?.physical.as_ref()?.error.as_deref()
    }
}
impl Authority {
    pub(super) fn physical_melee_event_slots(&self, target: EntityKey) -> usize {
        if self.physical.is_none() {
            return 2;
        }
        // A lethal hit also cancels every pending cast/queue on that creature.
        2 + self
            .actors
            .values()
            .map(|a| {
                usize::from(a.cast.as_ref().is_some_and(|c| c.target == target))
                    + usize::from(
                        a.queued
                            .as_ref()
                            .is_some_and(|q| q.request.target == target),
                    )
            })
            .sum::<usize>()
    }
    pub(super) fn apply_physical_melee(
        &mut self,
        world: &mut World,
        id: u64,
        target: EntityKey,
        weapon: &melee::Weapon,
        sequence: u64,
    ) -> Result<()> {
        if self.events.len() + self.physical_melee_event_slots(target) > self.step_event_budget()
            || world.patrol.as_ref().is_none_or(|p| p.key != target)
        {
            return Err("Physical melee event/creature context unavailable".into());
        }
        let actor = world.entity_key(id).ok_or("Missing physical attacker")?;
        let life = self
            .actors
            .get(&id)
            .and_then(|a| a.life.as_ref())
            .ok_or("Missing attacker life")?
            .state
            .key();
        if life.actor != actor {
            return Err("Physical attacker life is stale".into());
        }
        let victim = self
            .creatures
            .iter()
            .find(|c| c.key == target && c.current > 0)
            .ok_or("Missing physical victim")?;
        let plan = self
            .physical
            .as_ref()
            .ok_or("Missing physical profile")?
            .weapon(weapon.item(), victim.current)?;
        if plan.result.health_lost == victim.current
            && world.tick.checked_add(self.respawn_ticks).is_none()
        {
            return Err("Physical death deadline exhausted".into());
        }
        let attack = self
            .physical
            .as_ref()
            .unwrap()
            .profile
            .weapon(weapon.item())
            .unwrap()
            .0
            .clone();
        let result = plan.result;
        let roll = plan.roll;
        self.apply_damage(
            world,
            actor,
            target,
            result.resolved_damage,
            |amount, remaining, position| {
                debug_assert_eq!(amount, result.health_lost);
                Event::PhysicalDamage(Fact {
                    attack,
                    actor,
                    actor_generation: Some(life.generation),
                    sequence,
                    target,
                    target_generation: None,
                    position,
                    roll,
                    result,
                    remaining,
                })
            },
        );
        self.physical.as_mut().unwrap().commit(plan, false);
        Ok(())
    }
    pub(super) fn apply_physical_retaliation(
        &mut self,
        world: &mut World,
        npc: EntityKey,
        target: LifeKey,
        legacy_amount: u32,
    ) -> Result<Option<player_life::Damage>> {
        if self.physical.is_none() {
            return self.damage_player(world, target, legacy_amount);
        }
        if self.events.len() + 3 > self.step_event_budget() {
            return Err("Physical retaliation event budget exhausted".into());
        }
        let Some(player) = self
            .actors
            .get(&target.actor.entity.value())
            .and_then(|a| a.life.as_ref())
        else {
            return Ok(None);
        };
        if player.state.key() != target
            || !player.state.alive()
            || world.entity_key(target.actor.entity.value()) != Some(target.actor)
            || world.patrol.as_ref().is_none_or(|p| p.key != npc)
            || !self.creatures.iter().any(|c| c.key == npc && c.current > 0)
        {
            return Ok(None);
        }
        let position = world
            .players
            .get(&target.actor.entity.value())
            .ok_or("Missing physical player pose")?
            .position;
        let runtime = self.physical.as_ref().unwrap();
        let plan = runtime.creature(player.state.health())?;
        let attack = runtime.profile.creature_attack.clone();
        let sequence = plan.npc_sequence.unwrap();
        let Some(hit) = self.damage_player(world, target, plan.result.resolved_damage)? else {
            return Ok(None);
        };
        debug_assert_eq!(hit.applied, plan.result.health_lost);
        self.events.push(Event::PhysicalDamage(Fact {
            attack,
            actor: npc,
            actor_generation: None,
            sequence,
            target: target.actor,
            target_generation: Some(target.generation),
            position,
            roll: plan.roll,
            result: plan.result,
            remaining: hit.remaining,
        }));
        self.physical.as_mut().unwrap().commit(plan, true);
        Ok(Some(hit))
    }
}

#[cfg(test)]
mod world_tests;

#[cfg(test)]
mod renet_tests;
