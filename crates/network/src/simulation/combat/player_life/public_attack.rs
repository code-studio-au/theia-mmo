//! Opt-in authority observation; transport and Bevy adoption are separate slices.
use crate::{
    Result,
    creature_attack_wire::{Observation, Profile, Windup},
    simulation::World,
};
use std::sync::Arc;
impl World {
    pub fn enable_public_creature_attacks(&mut self) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Public creature attacks require pre-admission classic World".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Public creature attacks require combat")?;
        if combat.public_attack.is_some()
            || combat.public_population_attack.is_some()
            || !combat.ordinary_target_ready()
        {
            return Err(
                "Public creature attacks require ready pursuit/life and no prior adoption".into(),
            );
        }
        let profile = Arc::new(Profile::new(combat.pve.as_ref().unwrap().profile.clone()));
        self.combat.as_mut().unwrap().public_attack = Some(profile);
        Ok(())
    }
    pub fn public_creature_attack_profile(&self) -> Option<&Arc<Profile>> {
        self.combat.as_ref()?.public_attack.as_ref()
    }
    pub fn public_creature_attack_snapshot(&self) -> Result<Option<Observation>> {
        let Some(combat) = self.combat.as_ref() else {
            return Ok(None);
        };
        let Some(profile) = &combat.public_attack else {
            return Ok(None);
        };
        let pve = combat
            .pve
            .as_ref()
            .ok_or("Public creature attacks lost life authority")?;
        self.creature_attack_observation(combat, pve, profile)
            .map(Some)
    }
    fn creature_attack_observation(
        &self,
        combat: &super::Combat,
        pve: &super::Authority,
        profile: &Profile,
    ) -> Result<Observation> {
        if pve.error.is_some()
            || pve.profile.hash() != profile.life().hash()
            || self.creature_threat_error().is_some()
            || self.creature_reset_error().is_some()
        {
            return Err("Public creature attack authority unavailable".into());
        }
        let alive = combat
            .creatures
            .iter()
            .any(|c| c.key == pve.npc && c.current > 0);
        let can_attack = pve
            .pursuit
            .as_ref()
            .is_some_and(|p| p.directive().can_attack);
        let target = pve.policy.target().filter(|key| {
            self.entity_key(key.actor.entity.value()) == Some(key.actor)
                && combat
                    .actors
                    .get(&key.actor.entity.value())
                    .and_then(|a| a.life.as_ref())
                    .is_some_and(|p| p.state.alive() && p.state.key() == *key)
        });
        let windup = if alive && can_attack {
            match target.zip(pve.policy.windup_until()) {
                Some((target, completes)) => {
                    let duration = profile
                        .windup_ticks(self.rate)
                        .ok_or("Public creature attack clock exhausted")?;
                    let starts = completes
                        .checked_sub(duration)
                        .ok_or("Invalid public creature attack start")?;
                    Some(Windup {
                        target: target.actor,
                        starts,
                        completes,
                    })
                }
                None => None,
            }
        } else {
            None
        };
        let value = Observation {
            profile: profile.hash(),
            creature: pve.npc,
            tick: self.tick,
            windup,
        };
        if !crate::creature_attack_wire::valid(profile, self.rate, &value) {
            return Err("Invalid public creature authority observation".into());
        }
        Ok(value)
    }
    /// Authority-only opt-in. Normal launch remains gated until priority
    /// transport and multi-NPC presentation are connected and qualified.
    pub fn enable_public_population_attacks(&mut self) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || !self.classic || self.population.is_none()
        {
            return Err("Public population attacks require inactive classic population".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Public population attacks require combat")?;
        combat.checked_reset_encounters()?;
        if combat.pve.is_some()
            || combat.retaliators.is_empty()
            || combat.public_attack.is_some()
            || combat.public_population_attack.is_some()
            || combat.lives.as_ref().is_none_or(|l| !l.ready())
        {
            return Err(
                "Public population attacks require matching pursuit/life authorities".into(),
            );
        }
        let profile = Arc::new(
            crate::creature_attack_wire::population::PopulationProfile::new(Arc::new(
                Profile::new(combat.retaliators[0].profile.clone()),
            )),
        );
        self.combat.as_mut().unwrap().public_population_attack = Some(profile);
        Ok(())
    }
    pub fn public_population_attack_profile(
        &self,
    ) -> Option<&Arc<crate::creature_attack_wire::population::PopulationProfile>> {
        self.combat.as_ref()?.public_population_attack.as_ref()
    }
    pub fn public_population_attack_snapshot(
        &self,
    ) -> Result<Option<crate::creature_attack_wire::population::Frame>> {
        let Some(combat) = &self.combat else {
            return Ok(None);
        };
        let Some(profile) = &combat.public_population_attack else {
            return Ok(None);
        };
        if combat.retaliators.len() != combat.creatures.len()
            || combat.retaliators.len() > crate::creature_attack_wire::population::MAX_ACTORS
        {
            return Err("Public population attack identities unavailable".into());
        }
        let mut actors = combat
            .retaliators
            .iter()
            .map(|pve| self.creature_attack_observation(combat, pve, profile.source()))
            .collect::<Result<Vec<_>>>()?;
        actors.sort_by_key(|v| v.creature.entity);
        let frame = crate::creature_attack_wire::population::Frame {
            epoch: self.epoch,
            tick: self.tick,
            actors,
        };
        crate::creature_attack_wire::population::encode(profile, self.rate, &frame)?;
        Ok(Some(frame))
    }
}
