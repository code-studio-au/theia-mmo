//! Bounded creature membership for checked pursuit or passive populations.
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
    npcs: Vec<EntityKey>,
    states: BTreeMap<u64, State>,
    error: Option<String>,
}
struct Encounter {
    npc: EntityKey,
    mode: Mode,
    target: Option<LifeKey>,
    alive: bool,
}
pub(super) struct Planned {
    owner: LifeKey,
    state: State,
}
impl Runtime {
    fn new(
        profile: Arc<crate::combat_membership_wire::Profile>,
        npcs: &[EntityKey],
    ) -> Result<Self> {
        Self::validate_npcs(&profile, npcs)?;
        Ok(Self {
            profile,
            npcs: npcs.to_vec(),
            states: BTreeMap::new(),
            error: None,
        })
    }
    fn validate_npcs(
        profile: &crate::combat_membership_wire::Profile,
        npcs: &[EntityKey],
    ) -> Result<()> {
        if npcs.is_empty()
            || npcs.len() > crate::targeting::MAX_TARGETS
            || npcs.len() > profile.policy().maximum_opponents()
            || npcs.iter().enumerate().any(|(i, key)| {
                npcs[..i].iter().any(|prior| prior.entity == key.entity)
                    || key.epoch != npcs[0].epoch
            })
        {
            return Err(
                "Membership creature registry exceeds bounds or contains invalid identities".into(),
            );
        }
        Ok(())
    }
    fn contains(&self, npc: EntityKey) -> bool {
        self.npcs.contains(&npc)
    }
    /// Retire exact old lives before applying current observations. Work on a
    /// copy so a refused observation cannot partially alter the player's state.
    fn observe(
        &self,
        state: &State,
        tick: u64,
        current: &[(EntityKey, bool, bool)],
    ) -> Result<State> {
        let keys: Vec<_> = current.iter().map(|entry| entry.0).collect();
        Self::validate_npcs(&self.profile, &keys)?;
        let mut next = state.clone();
        let owner = next.owner();
        for old in &self.npcs {
            if !keys.contains(old) {
                next.forget(tick, owner, *old)
                    .map_err(|e| format!("Membership retirement refused: {e:?}"))?;
            }
        }
        for &(npc, resolved, active) in current {
            if resolved {
                next.forget(tick, owner, npc)
                    .map_err(|e| format!("Membership resolution refused: {e:?}"))?;
            } else {
                next.encounter(tick, owner, npc, active)
                    .map_err(|e| format!("Membership encounter refused: {e:?}"))?;
            }
        }
        Ok(next)
    }
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
        let encounters = combat.membership_encounters(self)?;
        if combat.membership.is_some()
            || (!combat.ordinary_target_ready() && !combat.population_physical_ready(self))
            || combat.ability_damage.is_none()
            || combat.white_melee.is_none()
            || combat
                .catalog
                .abilities()
                .iter()
                .any(|a| !matches!(a.effect(), theia_core::abilities::Effect::Damage { .. }))
        {
            return Err("Membership requires checked typed-damage creature encounters".into());
        }
        let keys: Vec<_> = encounters.iter().map(|e| e.npc).collect();
        self.combat.as_mut().unwrap().membership = Some(Runtime::new(profile, &keys)?);
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
    fn membership_encounters(&self, world: &World) -> Result<Vec<Encounter>> {
        if let Some((npc, mode, target)) = self.membership_encounter() {
            if self.creatures.len() != 1 || self.creatures[0].key != npc {
                return Err("Membership pursuit creature identity is unavailable".into());
            }
            return Ok(vec![Encounter {
                npc,
                mode,
                target,
                alive: self.creatures[0].current > 0,
            }]);
        }
        if !self.population_physical_ready(world) {
            return Err("Membership lost checked creature authority".into());
        }
        Ok(self
            .creatures
            .iter()
            .map(|creature| Encounter {
                npc: creature.key,
                mode: self
                    .population_encounter(creature.key)
                    .map_or(Mode::Patrol, |e| e.0),
                target: self.population_encounter(creature.key).and_then(|e| e.1),
                alive: creature.current > 0,
            })
            .collect())
    }
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
            || !runtime.contains(npc)
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
            let encounters = self.membership_encounters(world)?;
            let keys: Vec<_> = encounters.iter().map(|e| e.npc).collect();
            Runtime::validate_npcs(&runtime.profile, &keys)?;
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
                let state = runtime.states.get(&id).cloned().unwrap_or_else(|| {
                    State::new(runtime.profile.policy().clone(), world.rate, owner)
                });
                let current: Vec<_> = encounters
                    .iter()
                    .map(|e| {
                        let resolved =
                            !e.alive || matches!(e.mode, Mode::Return | Mode::Blocked | Mode::Dead);
                        let active = e.alive
                            && matches!(e.mode, Mode::Chase | Mode::Attack)
                            && (e.target == Some(owner)
                                || self
                                    .threat
                                    .iter()
                                    .chain(&self.population_threat)
                                    .any(|t| t.npc == e.npc && t.contains(owner)));
                        (e.npc, resolved, active)
                    })
                    .collect();
                let next = runtime.observe(&state, world.tick, &current)?;
                runtime.states.insert(id, next);
            }
            runtime.npcs = keys;
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.membership = Some(runtime);
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use std::num::NonZeroU64;
    use theia_core::{
        identity::{EntityId, WorldEpoch},
        time::TickRate,
    };
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn profile() -> Arc<crate::combat_membership_wire::Profile> {
        Arc::new(
            crate::combat_membership_wire::Profile::from_json(include_bytes!(
                "../../../../../content/gameplay/development-combat-membership.json"
            ))
            .unwrap(),
        )
    }
    #[test]
    fn retirement_preserves_other_opponents_and_new_lives_start_without_old_hostility() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let runtime = Runtime::new(profile(), &[key(20), key(21)]).unwrap();
            let owner = LifeKey {
                actor: key(1),
                generation: NonZeroU64::new(1).unwrap(),
            };
            let mut state = State::new(runtime.profile.policy().clone(), rate, owner);
            state
                .hostility(0, owner, key(20), Hostility::Outgoing)
                .unwrap();
            state
                .hostility(0, owner, key(21), Hostility::Incoming)
                .unwrap();
            let active = runtime
                .observe(&state, 1, &[(key(20), false, true), (key(21), false, true)])
                .unwrap();
            assert_eq!(active.summary().opponents, 2);
            let retired = runtime
                .observe(
                    &active,
                    2,
                    &[(key(22), false, false), (key(21), false, true)],
                )
                .unwrap();
            assert_eq!(retired.summary().opponents, 1);
            assert!(retired.summary().incoming_hostility);
            assert!(!retired.summary().outgoing_hostility);
            assert!(retired.summary().active_encounter);
            let resolved = runtime
                .observe(&active, 2, &[(key(20), true, true), (key(21), false, true)])
                .unwrap();
            assert_eq!(resolved, retired);
            let expired = runtime
                .observe(
                    &retired,
                    4000,
                    &[(key(22), false, false), (key(21), false, false)],
                )
                .unwrap();
            assert!(!expired.summary().in_combat());
            let mut foreign = key(21);
            foreign.epoch = WorldEpoch::new(2).unwrap();
            assert!(!runtime.contains(foreign));
            assert!(
                runtime
                    .observe(
                        &active,
                        2,
                        &[(key(20), true, false), (foreign, false, true)]
                    )
                    .is_err()
            );
            assert_eq!(active.summary().opponents, 2);
            assert!(
                runtime
                    .observe(
                        &active,
                        0,
                        &[(key(20), true, false), (key(21), false, true)]
                    )
                    .is_err()
            );
        }
    }
    #[test]
    fn registry_checks_actor_capacity_and_exact_identity() {
        assert!(Runtime::new(profile(), &[]).is_err());
        assert!(Runtime::new(profile(), &[key(20), key(20)]).is_err());
        let keys: Vec<_> = (20..=20 + crate::targeting::MAX_TARGETS as u64)
            .map(key)
            .collect();
        assert!(Runtime::new(profile(), &keys).is_err());
        assert!(Runtime::new(profile(), &keys[..crate::targeting::MAX_TARGETS]).is_ok());
        let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-combat-membership.json"
        ))
        .unwrap();
        raw["maximum_opponents"] = 1.into();
        let limited = Arc::new(
            crate::combat_membership_wire::Profile::from_json(&serde_json::to_vec(&raw).unwrap())
                .unwrap(),
        );
        assert!(Runtime::new(limited, &[key(20), key(21)]).is_err());
    }
}
