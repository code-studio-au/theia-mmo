//! Explicit opt-in World authority. All entropy is acquired before admission;
//! each effect previews private draws, eligibility and commit prerequisites.
use super::*;
use crate::simulation::{
    World,
    combat::{Authority, Event, Hit, Reason, outcomes},
};
use std::num::NonZeroU64;
use theia_core::{coordinates::GlobalPosition, identity::EntityKey};
#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub ability: ContentId,
    pub actor: EntityKey,
    pub actor_generation: NonZeroU64,
    pub sequence: u64,
    pub target: EntityKey,
    pub position: GlobalPosition,
    pub defender: white_melee::DefenderState,
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
    pub hits: u64,
    pub criticals: u64,
    pub resolved_damage: u64,
    pub health_lost: u64,
}
impl Counts {
    fn observe(&mut self, result: Resolved) {
        self.attempts = self.attempts.saturating_add(1);
        let count = match result.outcome {
            Outcome::MeleeSpecial(melee_special::Outcome::Miss)
            | Outcome::Arcane(spell::Outcome::Miss) => &mut self.misses,
            Outcome::MeleeSpecial(melee_special::Outcome::Dodge) => &mut self.dodges,
            Outcome::MeleeSpecial(melee_special::Outcome::Parry) => &mut self.parries,
            Outcome::MeleeSpecial(melee_special::Outcome::Hit)
            | Outcome::Arcane(spell::Outcome::Hit) => &mut self.hits,
            Outcome::MeleeSpecial(melee_special::Outcome::Critical)
            | Outcome::Arcane(spell::Outcome::Critical) => &mut self.criticals,
        };
        *count = count.saturating_add(1);
        self.resolved_damage = self
            .resolved_damage
            .saturating_add(u64::from(result.damage.resolved_damage));
        self.health_lost = self
            .health_lost
            .saturating_add(u64::from(result.damage.health_lost));
    }
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub base_draws: u64,
    pub landing_draws: u64,
    pub critical_draws: u64,
    pub refusals: u64,
    pub special: Counts,
    pub arcane: Counts,
}
pub(in crate::simulation::combat) struct Runtime {
    pub(in crate::simulation::combat) profile: Arc<Profile>,
    random: outcomes::Random,
    metrics: Metrics,
    last_error: Option<String>,
}
struct Planned {
    random: outcomes::Random,
    draws: Draws,
    result: Resolved,
    landing_draws: u64,
    critical_draws: u64,
}
impl Runtime {
    fn new(profile: Profile) -> Result<Self> {
        Ok(Self {
            profile: Arc::new(profile),
            random: outcomes::Random::new()?,
            metrics: Metrics::default(),
            last_error: None,
        })
    }
    fn preview(
        &self,
        ability: &ContentId,
        state: white_melee::DefenderState,
        health: u32,
    ) -> Result<Planned> {
        let landing_draws = self
            .metrics
            .landing_draws
            .checked_add(1)
            .ok_or("Ability landing counter exhausted")?;
        let mut random = self.random.clone();
        let base = random.base_roll(self.profile.range(ability)?)?;
        let landing = random.ordinary_roll()?;
        let needed = self.profile.needs_critical_roll(ability, state, landing)?;
        let critical_draws = self
            .metrics
            .critical_draws
            .checked_add(u64::from(needed))
            .ok_or("Ability critical counter exhausted")?;
        let critical = if needed {
            Some(random.ordinary_roll()?)
        } else {
            None
        };
        let draws = Draws {
            base,
            landing,
            critical,
        };
        let result = self.profile.evaluate(ability, state, draws, health)?;
        Ok(Planned {
            random,
            draws,
            result,
            landing_draws,
            critical_draws,
        })
    }
    fn commit(&mut self, plan: Planned) {
        self.random = plan.random;
        self.metrics.base_draws = self.random.base_draws();
        self.metrics.landing_draws = plan.landing_draws;
        self.metrics.critical_draws = plan.critical_draws;
        match plan.result.outcome {
            Outcome::MeleeSpecial(_) => self.metrics.special.observe(plan.result),
            Outcome::Arcane(_) => self.metrics.arcane.observe(plan.result),
        }
    }
}
impl World {
    /// Before admission only. Normal applications/codecs do not opt into this
    /// candidate until separately versioned transport and native checks pass.
    pub fn enable_ability_damage(
        &mut self,
        starting: &[u8],
        policy: &[u8],
        abilities: &[u8],
    ) -> Result<()> {
        if !self.classic || self.tick != 0 || !self.players.is_empty() {
            return Err("Ability damage requires inactive classic authority".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Ability damage requires combat")?;
        if combat.ability_damage.is_some()
            || crate::terrain::sha256(abilities) != combat.catalog_digest
        {
            return Err("Ability damage requires one exact active ability catalogue".into());
        }
        let ordinary = combat
            .white_melee
            .as_ref()
            .ok_or("Ability damage requires ordinary physical/life/pursuit context")?;
        let runtime = Runtime::new(Profile::new(
            starting,
            policy,
            abilities,
            ordinary.profile.clone(),
        )?)?;
        self.combat.as_mut().unwrap().ability_damage = Some(runtime);
        Ok(())
    }
    pub fn ability_damage_profile(&self) -> Option<&Arc<Profile>> {
        self.combat
            .as_ref()?
            .ability_damage
            .as_ref()
            .map(|r| &r.profile)
    }
    pub fn ability_damage_metrics(&self) -> Option<Metrics> {
        self.combat
            .as_ref()?
            .ability_damage
            .as_ref()
            .map(|r| r.metrics)
    }
    pub fn ability_damage_error(&self) -> Option<&str> {
        self.combat
            .as_ref()?
            .ability_damage
            .as_ref()?
            .last_error
            .as_deref()
    }
}
impl Authority {
    pub(in crate::simulation::combat) fn apply_ability_damage(
        &mut self,
        world: &mut World,
        hit: &Hit,
    ) -> Result<()> {
        if hit.sequence == 0
            || !self.ordinary_target_ready()
            || world.entity_key(hit.actor.entity.value()) != Some(hit.actor)
            || self.events.len() + self.physical_melee_event_slots(hit.target)
                > self.step_event_budget()
            || world.patrol.as_ref().is_none_or(|p| p.key != hit.target)
        {
            return Err("Ability-damage actor, target or event capacity unavailable".into());
        }
        let player = self
            .actors
            .get(&hit.actor.entity.value())
            .and_then(|a| a.life.as_ref())
            .filter(|l| l.state.alive())
            .ok_or("Ability-damage source life unavailable")?;
        let life = player.state.key();
        if life.actor != hit.actor {
            return Err("Ability-damage source incarnation is stale".into());
        }
        let victim = self
            .creatures
            .iter()
            .find(|c| c.key == hit.target && c.current > 0)
            .ok_or("Ability-damage target is unavailable")?;
        let health = victim.current;
        let state = self.ordinary_creature_state(world, hit.target, hit.actor)?;
        let runtime = self
            .ability_damage
            .as_ref()
            .ok_or("Missing ability-damage profile")?;
        let plan = runtime.preview(&hit.ability, state, health)?;
        let result = plan.result;
        if result.damage.health_lost == health
            && world.tick.checked_add(self.respawn_ticks).is_none()
        {
            return Err("Ability-damage death deadline exhausted".into());
        }
        let membership = self.prepare_membership(
            world,
            life,
            hit.target,
            theia_core::combat_membership::Hostility::Outgoing,
        )?;
        let credit = self.prepare_credit(
            world,
            super::super::credit::ResolvedHit {
                source: life,
                target: hit.target,
                category: theia_core::creature_credit::Category::AbilityDamage,
                encounter: hit.credit_encounter,
                health_lost: result.damage.health_lost,
                fatal: result.damage.health_lost == health,
            },
        )?;
        let retaliation = self.prepare_retaliation_attempt(
            world,
            hit.actor,
            hit.target,
            health - result.damage.health_lost,
        )?;
        let draws = plan.draws;
        let actor = hit.actor;
        let ability = hit.ability.clone();
        let sequence = hit.sequence;
        let target = hit.target;
        self.apply_damage(
            world,
            actor,
            target,
            result.damage.resolved_damage,
            |applied, remaining, position| {
                debug_assert_eq!(applied, result.damage.health_lost);
                Event::AbilityDamage(Fact {
                    ability,
                    actor,
                    actor_generation: life.generation,
                    sequence,
                    target,
                    position,
                    defender: state,
                    draws,
                    result,
                    remaining,
                })
            },
        );
        self.commit_retaliation_attempt(retaliation);
        self.commit_membership(membership);
        self.commit_credit(credit);
        self.ability_damage.as_mut().unwrap().commit(plan);
        Ok(())
    }
    pub(in crate::simulation::combat) fn resolve_ability_damage(
        &mut self,
        world: &mut World,
        hit: Hit,
    ) {
        if let Err(error) = self.apply_ability_damage(world, &hit) {
            let runtime = self.ability_damage.as_mut().unwrap();
            runtime.metrics.refusals = runtime.metrics.refusals.saturating_add(1);
            runtime.last_error = Some(error.to_string());
            if self.events.len() < self.step_event_budget() {
                self.cancel(hit.actor, hit.sequence, Reason::Unavailable);
            }
        }
    }
}
#[cfg(test)]
mod renet_tests;
#[cfg(test)]
mod tests;
