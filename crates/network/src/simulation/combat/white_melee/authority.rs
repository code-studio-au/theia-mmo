//! Explicit headless ordinary-melee authority. Legacy profiles/codecs remain
//! separate; source randomness is private and previewed before state commits.
use super::super::{Authority, Event, World, outcomes};
use super::*;
use std::num::NonZeroU64;
use theia_core::{
    coordinates::GlobalPosition, facing::Arc as FacingArc, identity::EntityKey, melee,
    player_life::LifeKey,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub attack: ContentId,
    pub actor: EntityKey,
    pub actor_generation: Option<NonZeroU64>,
    pub sequence: u64,
    pub target: EntityKey,
    pub target_generation: Option<NonZeroU64>,
    pub position: GlobalPosition,
    pub defender: DefenderState,
    pub draws: Draws,
    pub result: Resolved,
    pub remaining: u32,
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Counts {
    pub attempts: u64,
    pub misses: u64,
    pub dodges: u64,
    pub parries: u64,
    pub blocks: u64,
    pub glancing: u64,
    pub criticals: u64,
    pub crushing: u64,
    pub normal: u64,
    pub resolved_damage: u64,
    pub health_lost: u64,
}
impl Counts {
    fn observe(&mut self, result: Resolved) {
        self.attempts = self.attempts.saturating_add(1);
        let counter = match result.outcome {
            Outcome::Miss => &mut self.misses,
            Outcome::Dodge => &mut self.dodges,
            Outcome::Parry => &mut self.parries,
            Outcome::Block => &mut self.blocks,
            Outcome::Glancing => &mut self.glancing,
            Outcome::Critical => &mut self.criticals,
            Outcome::Crushing => &mut self.crushing,
            Outcome::Hit => &mut self.normal,
        };
        *counter = counter.saturating_add(1);
        self.resolved_damage = self
            .resolved_damage
            .saturating_add(u64::from(result.resolved_damage));
        self.health_lost = self
            .health_lost
            .saturating_add(u64::from(result.health_lost));
    }
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub base_draws: u64,
    pub outcome_draws: u64,
    pub glance_draws: u64,
    pub outgoing: Counts,
    pub incoming: Counts,
}
pub(in crate::simulation::combat) struct Runtime {
    pub(in crate::simulation::combat) profile: Arc<Profile>,
    random: outcomes::Random,
    npc_sequence: u64,
    metrics: Metrics,
    pub(in crate::simulation::combat) error: Option<String>,
}
struct Planned {
    random: outcomes::Random,
    draws: Draws,
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
    fn weapon(&self, item: &ContentId, state: DefenderState, health: u32) -> Result<Planned> {
        if self.error.is_some() {
            return Err("Ordinary authority is halted".into());
        }
        let weapon = self
            .profile
            .weapons
            .get(item)
            .ok_or("Unknown ordinary weapon")?;
        let mut random = self.random.clone();
        let base = random.base_roll(weapon.damage.range())?;
        let outcome = random.ordinary_roll()?;
        let selected = self
            .profile
            .outgoing_distribution(item, state)?
            .resolve(outcome);
        let glancing = if selected == Outcome::Glancing {
            Some(random.glance_roll(weapon.glance)?)
        } else {
            None
        };
        let draws = Draws {
            outcome,
            base,
            glancing,
        };
        let result = self.profile.evaluate_outgoing(item, state, draws, health)?;
        Ok(Planned {
            random,
            draws,
            result,
            npc_sequence: None,
        })
    }
    fn creature(&self, state: DefenderState, health: u32) -> Result<Planned> {
        if self.error.is_some() {
            return Err("Ordinary authority is halted".into());
        }
        let sequence = self
            .npc_sequence
            .checked_add(1)
            .ok_or("Ordinary NPC sequence exhausted")?;
        let mut random = self.random.clone();
        let base = random.base_roll(self.profile.physical.creature_attack().1)?;
        let outcome = random.ordinary_roll()?;
        let draws = Draws {
            outcome,
            base,
            glancing: None,
        };
        let result = self.profile.evaluate_incoming(state, draws, health)?;
        Ok(Planned {
            random,
            draws,
            result,
            npc_sequence: Some(sequence),
        })
    }
    fn commit(&mut self, plan: Planned, incoming: bool) {
        self.random = plan.random;
        if let Some(sequence) = plan.npc_sequence {
            self.npc_sequence = sequence;
        }
        self.metrics.base_draws = self.random.base_draws();
        self.metrics.outcome_draws = self.random.draws();
        self.metrics.glance_draws = self.random.glance_draws();
        if incoming {
            self.metrics.incoming.observe(plan.result);
        } else {
            self.metrics.outgoing.observe(plan.result);
        }
    }
}
impl World {
    /// Explicit experimental authority opt-in before admission. Normal apps and
    /// older codecs do not select or accept its facts yet.
    pub fn enable_white_melee(&mut self, bytes: &[u8]) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Ordinary outcomes require inactive classic authority".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Ordinary outcomes require combat")?;
        if combat.white_melee.is_some() || !combat.supports_white_melee() {
            return Err("Ordinary outcomes require one pursuit configuration".into());
        }
        let physical = combat
            .physical
            .as_ref()
            .ok_or("Ordinary outcomes require physical damage")?;
        let profile = Profile::new(bytes, physical.profile.clone())?;
        profile.check_supported_abilities(&combat.catalog)?;
        let runtime = Runtime::new(profile)?;
        self.combat.as_mut().unwrap().white_melee = Some(runtime);
        Ok(())
    }
    pub fn white_melee_profile(&self) -> Option<&Arc<Profile>> {
        self.combat
            .as_ref()?
            .white_melee
            .as_ref()
            .map(|r| &r.profile)
    }
    pub fn white_melee_metrics(&self) -> Option<Metrics> {
        self.combat
            .as_ref()?
            .white_melee
            .as_ref()
            .map(|r| r.metrics)
    }
    pub fn white_melee_error(&self) -> Option<&str> {
        self.combat.as_ref()?.white_melee.as_ref()?.error.as_deref()
    }
}
fn front(
    facing: theia_core::facing::Facing,
    defender: GlobalPosition,
    attacker: GlobalPosition,
) -> bool {
    FacingArc::from_degrees(180.)
        .expect("Reference defensive arc")
        .contains(facing, defender, attacker)
}
impl Authority {
    pub(in crate::simulation::combat) fn ordinary_creature_state(
        &self,
        world: &World,
        npc: EntityKey,
        attacker: EntityKey,
    ) -> Result<DefenderState> {
        let creature = self
            .creatures
            .iter()
            .find(|c| c.key == npc && c.current > 0)
            .ok_or("Missing ordinary creature")?;
        let position = world
            .global_position(attacker.entity.value())
            .ok_or("Missing ordinary attacker position")?;
        let facing = world
            .patrol_facing(npc)
            .ok_or("Missing authoritative creature facing")?;
        Ok(DefenderState {
            standing: true,
            can_react: true,
            in_front: front(facing, creature.position, position),
            ..Default::default()
        })
    }
    fn ordinary_player_state(
        &self,
        world: &World,
        npc: EntityKey,
        target: LifeKey,
    ) -> Result<DefenderState> {
        let creature = self
            .creatures
            .iter()
            .find(|c| c.key == npc && c.current > 0)
            .ok_or("Missing ordinary creature")?;
        let id = target.actor.entity.value();
        let actor = self.actors.get(&id).ok_or("Missing ordinary player")?;
        let player = world
            .players
            .get(&id)
            .ok_or("Missing ordinary player pose")?;
        let facing = player.facing.ok_or("Missing classic player facing")?;
        let casting_non_melee = actor.cast.as_ref().is_some_and(|cast| {
            self.catalog
                .timing(cast.ability.as_str(), world.rate)
                .is_some_and(|t| t.cast_ticks > 0)
        });
        Ok(DefenderState {
            standing: true,
            can_react: true,
            casting_non_melee,
            in_front: front(facing, player.position, creature.position),
            usable_melee_weapon: self.melee_weapon(id).is_ok(),
            usable_shield: false,
            disarmed: false,
        })
    }
    pub(in crate::simulation::combat) fn apply_white_melee(
        &mut self,
        world: &mut World,
        id: u64,
        target: EntityKey,
        weapon: &melee::Weapon,
        sequence: u64,
    ) -> Result<()> {
        if !self.ordinary_target_ready()
            || self.events.len() + self.physical_melee_event_slots(target)
                > self.step_event_budget()
            || world.patrol.as_ref().is_none_or(|p| p.key != target)
        {
            return Err("Ordinary event/creature context unavailable".into());
        }
        let actor = world.entity_key(id).ok_or("Missing ordinary attacker")?;
        let player = self
            .actors
            .get(&id)
            .and_then(|a| a.life.as_ref())
            .filter(|p| p.state.alive())
            .ok_or("Missing ordinary attacker life")?;
        let life = player.state.key();
        if life.actor != actor {
            return Err("Ordinary attacker life is stale".into());
        }
        let victim = self
            .creatures
            .iter()
            .find(|c| c.key == target && c.current > 0)
            .ok_or("Missing ordinary victim")?;
        let health = victim.current;
        let state = self.ordinary_creature_state(world, target, actor)?;
        let runtime = self
            .white_melee
            .as_ref()
            .ok_or("Missing ordinary profile")?;
        let plan = runtime.weapon(weapon.item(), state, health)?;
        if plan.result.health_lost == health && world.tick.checked_add(self.respawn_ticks).is_none()
        {
            return Err("Ordinary death deadline exhausted".into());
        }
        let membership = self.prepare_membership(
            world,
            life,
            target,
            theia_core::combat_membership::Hostility::Outgoing,
        )?;
        let credit = self.prepare_credit(
            world,
            super::super::credit::ResolvedHit {
                source: life,
                target,
                category: theia_core::creature_credit::Category::WhiteMelee,
                encounter: self.capture_credit(),
                health_lost: plan.result.health_lost,
                fatal: plan.result.health_lost == health,
            },
        )?;
        let retaliation = self.prepare_retaliation_attempt(
            world,
            actor,
            target,
            health - plan.result.health_lost,
        )?;
        let attack = runtime
            .profile
            .physical
            .weapon(weapon.item())
            .unwrap()
            .0
            .clone();
        let result = plan.result;
        let draws = plan.draws;
        self.apply_damage(
            world,
            actor,
            target,
            result.resolved_damage,
            |applied, remaining, position| {
                debug_assert_eq!(applied, result.health_lost);
                Event::WhiteMelee(Fact {
                    attack,
                    actor,
                    actor_generation: Some(life.generation),
                    sequence,
                    target,
                    target_generation: None,
                    position,
                    defender: state,
                    draws,
                    result,
                    remaining,
                })
            },
        );
        // Valid misses/dodges/full blocks can provoke too. The previewed policy
        // also guarantees a clock failure cannot follow a committed damage roll.
        self.commit_retaliation_attempt(retaliation);
        self.commit_membership(membership);
        self.commit_credit(credit);
        self.white_melee.as_mut().unwrap().commit(plan, false);
        Ok(())
    }
    pub(in crate::simulation::combat) fn apply_white_retaliation(
        &mut self,
        world: &mut World,
        npc: EntityKey,
        target: LifeKey,
    ) -> Result<Option<theia_core::player_life::Damage>> {
        if self.reset.as_ref().is_some_and(|runtime| !runtime.ready()) {
            return Err("Creature reset authority unavailable".into());
        }
        if self.events.len() + 3 > self.step_event_budget() {
            return Err("Ordinary retaliation event budget exhausted".into());
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
        let health = player.state.health();
        let position = world
            .global_position(target.actor.entity.value())
            .ok_or("Missing ordinary player position")?;
        let state = self.ordinary_player_state(world, npc, target)?;
        let runtime = self
            .white_melee
            .as_ref()
            .ok_or("Missing ordinary profile")?;
        let plan = runtime.creature(state, health)?;
        let attack = runtime.profile.physical.creature_attack().0.clone();
        let sequence = plan.npc_sequence.unwrap();
        let membership = self.prepare_membership(
            world,
            target,
            npc,
            theia_core::combat_membership::Hostility::Incoming,
        )?;
        let Some(hit) = self.damage_player(world, target, plan.result.resolved_damage)? else {
            return Ok(None);
        };
        debug_assert_eq!(hit.applied, plan.result.health_lost);
        self.events.push(Event::WhiteMelee(Fact {
            attack,
            actor: npc,
            actor_generation: None,
            sequence,
            target: target.actor,
            target_generation: Some(target.generation),
            position,
            defender: state,
            draws: plan.draws,
            result: plan.result,
            remaining: hit.remaining,
        }));
        self.commit_membership(membership);
        self.white_melee.as_mut().unwrap().commit(plan, true);
        Ok(Some(hit))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod renet_tests;
