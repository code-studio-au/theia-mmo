//! Explicit opt-in single-creature membership. Permission consumers are separate.
use super::{Authority, World};
use crate::Result;
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    combat_membership::{Hostility, State, Summary},
    identity::EntityKey,
    player_life::LifeKey,
    pursuit::Mode,
};
const DOMAIN: &[u8] = b"theia-combat-membership-world-v1\0";
fn identity(base: [u8; 32], hash: [u8; 32]) -> [u8; 32] {
    let mut bytes = DOMAIN.to_vec();
    bytes.extend(base);
    bytes.extend(hash);
    crate::terrain::sha256(&bytes)
}
/// Bind before later reward/progression profile wrappers, just after ability
/// damage. Normal applications bind this only through the explicit profile.
pub fn profile_identity(base: [u8; 32], bytes: &[u8]) -> Result<[u8; 32]> {
    crate::combat_membership_wire::Profile::from_json(bytes)?;
    Ok(identity(base, crate::terrain::sha256(bytes)))
}
pub fn identity_for_profile(
    base: [u8; 32],
    profile: &crate::combat_membership_wire::Profile,
) -> [u8; 32] {
    identity(base, profile.hash())
}
pub(super) struct Runtime {
    profile: Arc<crate::combat_membership_wire::Profile>,
    npc: EntityKey,
    states: BTreeMap<u64, State>,
    error: Option<String>,
}
pub(super) struct Planned {
    owner: LifeKey,
    state: State,
}
impl Runtime {
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        identity(base, self.profile.hash())
    }
    pub(super) fn leave(&mut self, id: u64) {
        self.states.remove(&id);
    }
}
impl World {
    pub fn enable_combat_membership(&mut self, bytes: &[u8]) -> Result<()> {
        if !self.classic || self.tick != 0 || !self.players.is_empty() {
            return Err("Membership requires pre-admission classic World".into());
        }
        let profile = Arc::new(crate::combat_membership_wire::Profile::from_json(bytes)?);
        let combat = self.combat.as_ref().ok_or("Membership requires combat")?;
        let (npc, _, _) = combat
            .membership_encounter()
            .ok_or("Membership requires creature pursuit")?;
        if combat.membership.is_some()
            || !combat.ordinary_target_ready()
            || combat.ability_damage.is_none()
            || combat.white_melee.is_none()
            || combat.creatures.len() != 1
            || combat.creatures[0].key != npc
            || combat
                .catalog
                .abilities()
                .iter()
                .any(|a| !matches!(a.effect(), theia_core::abilities::Effect::Damage { .. }))
        {
            return Err("Membership supports one opt-in typed-damage creature encounter".into());
        }
        self.combat.as_mut().unwrap().membership = Some(Runtime {
            profile,
            npc,
            states: BTreeMap::new(),
            error: None,
        });
        Ok(())
    }
    pub fn combat_membership_profile(
        &self,
    ) -> Option<&Arc<crate::combat_membership_wire::Profile>> {
        self.combat
            .as_ref()?
            .membership
            .as_ref()
            .map(|runtime| &runtime.profile)
    }
    /// Authority observation. Client presentation and permission consumers adopt
    /// this only through their explicit profile/versioned transport.
    pub fn combat_membership(&self, id: u64) -> Option<Summary> {
        let combat = self.combat.as_ref()?;
        let runtime = combat.membership.as_ref()?;
        if runtime.error.is_some() {
            return None;
        }
        let player = combat.actors.get(&id)?.life.as_ref()?;
        if !player.state.alive() || self.entity_key(id) != Some(player.state.key().actor) {
            return None;
        }
        Some(
            runtime
                .states
                .get(&id)
                .filter(|state| state.owner() == player.state.key())
                .map_or_else(Summary::default, State::summary),
        )
    }
    pub fn combat_membership_error(&self) -> Option<&str> {
        self.combat.as_ref()?.membership.as_ref()?.error.as_deref()
    }
    pub(in crate::simulation) fn advance_combat_membership(&mut self) {
        if self.combat.as_ref().is_none_or(|c| c.membership.is_none()) {
            return;
        }
        if let Some(mut combat) = self.combat.take() {
            combat.sync_membership(self);
            self.combat = Some(combat);
        }
    }
}
impl Authority {
    /// Preview before health, RNG or retaliation commits. The exact current
    /// life is supplied only after existing attack eligibility checks.
    pub(super) fn prepare_membership(
        &self,
        world: &World,
        owner: LifeKey,
        npc: EntityKey,
        direction: Hostility,
    ) -> Result<Option<Planned>> {
        let Some(runtime) = &self.membership else {
            return Ok(None);
        };
        if runtime.error.is_some()
            || runtime.npc != npc
            || world.entity_key(owner.actor.entity.value()) != Some(owner.actor)
            || !self
                .actors
                .get(&owner.actor.entity.value())
                .and_then(|a| a.life.as_ref())
                .is_some_and(|p| p.state.alive() && p.state.key() == owner)
        {
            return Err("Membership life or encounter unavailable".into());
        }
        let mut state = runtime
            .states
            .get(&owner.actor.entity.value())
            .filter(|state| state.owner() == owner)
            .cloned()
            .unwrap_or_else(|| State::new(runtime.profile.policy().clone(), world.rate, owner));
        state
            .hostility(world.tick, owner, npc, direction)
            .map_err(|e| format!("Membership hostility refused: {e:?}"))?;
        Ok(Some(Planned { owner, state }))
    }
    pub(super) fn commit_membership(&mut self, plan: Option<Planned>) {
        if let Some(plan) = plan {
            self.membership
                .as_mut()
                .expect("Prepared membership authority")
                .states
                .insert(plan.owner.actor.entity.value(), plan.state);
        }
    }
    fn sync_membership(&mut self, world: &World) {
        let Some(mut runtime) = self.membership.take() else {
            return;
        };
        let result = (|| -> Result<()> {
            if runtime.error.is_some() {
                return Ok(());
            }
            let (npc, mode, target) = self
                .membership_encounter()
                .ok_or("Membership lost its pursuit authority")?;
            let alive = self.creatures.iter().any(|c| c.key == npc && c.current > 0);
            let resolved = !alive || matches!(mode, Mode::Return | Mode::Blocked | Mode::Dead);
            runtime.states.retain(|id, state| {
                world.players.contains_key(id)
                    && self
                        .actors
                        .get(id)
                        .and_then(|a| a.life.as_ref())
                        .is_some_and(|p| p.state.alive() && p.state.key() == state.owner())
            });
            for (&id, actor) in &self.actors {
                let Some(player) = actor.life.as_ref().filter(|p| p.state.alive()) else {
                    continue;
                };
                let owner = player.state.key();
                if world.entity_key(id) != Some(owner.actor) {
                    continue;
                }
                let state = runtime.states.entry(id).or_insert_with(|| {
                    State::new(runtime.profile.policy().clone(), world.rate, owner)
                });
                if runtime.npc != npc || resolved {
                    state
                        .forget(world.tick, owner, runtime.npc)
                        .map_err(|e| format!("Membership retirement refused: {e:?}"))?;
                }
                let active =
                    alive && matches!(mode, Mode::Chase | Mode::Attack) && target == Some(owner);
                state
                    .encounter(world.tick, owner, npc, active)
                    .map_err(|e| format!("Membership encounter refused: {e:?}"))?;
            }
            runtime.npc = npc;
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.membership = Some(runtime);
    }
}
