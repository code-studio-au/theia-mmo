//! Read-only public visual life projection. No gameplay or transport adoption.
use crate::{
    Result,
    avatar_life_wire::{self as wire, Actor, Observation, Profile},
    simulation::World,
};
use std::sync::Arc;
impl World {
    pub fn enable_public_avatar_life(&mut self) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Public avatar life requires pre-admission Classic World".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Public avatar life requires life authority")?;
        let life = combat
            .pve
            .as_ref()
            .filter(|p| p.error.is_none())
            .ok_or("Public avatar life requires available life authority")?;
        if combat.public_life.is_some() {
            return Err("Public avatar life already enabled".into());
        }
        let profile = Arc::new(Profile::new(life.profile.clone()));
        self.combat.as_mut().unwrap().public_life = Some(profile);
        Ok(())
    }
    pub fn public_avatar_life_profile(&self) -> Option<&Arc<Profile>> {
        self.combat.as_ref()?.public_life.as_ref()
    }
    pub fn public_avatar_life_snapshot(&self) -> Result<Option<Observation>> {
        let Some(profile) = self.public_avatar_life_profile() else {
            return Ok(None);
        };
        if self.player_life_error().is_some()
            || self.creature_threat_error().is_some()
            || self.creature_reset_error().is_some()
            || self.combat_membership_error().is_some()
            || self
                .player_life_profile()
                .is_none_or(|p| p.hash() != profile.life().hash())
        {
            return Err("Public avatar life authority unavailable".into());
        }
        if self.players.len() > wire::MAX_ACTORS {
            return Err("Public avatar roster capacity exceeded".into());
        }
        let mut actors = Vec::with_capacity(self.players.len());
        for &id in self.players.keys() {
            let private = self
                .player_life_snapshot(id)
                .ok_or("Connected public avatar has no life authority")?;
            if self.entity_key(id) != Some(private.key.actor) {
                return Err("Public avatar actor incarnation mismatch".into());
            }
            actors.push(Actor {
                key: private.key,
                dead: private.health == 0,
            });
        }
        let observation = Observation {
            profile: profile.hash(),
            epoch: theia_core::identity::WorldEpoch::new(self.epoch())
                .ok_or("Missing public avatar World incarnation")?,
            tick: self.tick,
            actors,
        };
        if !wire::valid(profile, &observation) {
            return Err("Invalid public avatar authority projection".into());
        }
        Ok(Some(observation))
    }
}
