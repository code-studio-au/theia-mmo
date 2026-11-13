//! Opt-in ordinary-creature ownership. No XP/quest/loot consumer is changed.
use super::{Authority, Event, World};
use crate::Result;
use std::{collections::BTreeMap, num::NonZeroU64};
use theia_core::{
    creature_credit::{Category, DamageFact, EncounterKey, KillCredit, Ledger, Profile},
    identity::EntityKey,
    player_life::LifeKey,
    pursuit::Mode,
};
const DOMAIN: &[u8] = b"theia-creature-credit-world-v1\0";
pub fn profile_identity(base: [u8; 32], bytes: &[u8]) -> Result<[u8; 32]> {
    Profile::from_json(bytes)?;
    Ok(identity(base, crate::terrain::sha256(bytes)))
}

fn identity(base: [u8; 32], hash: [u8; 32]) -> [u8; 32] {
    let mut bytes = DOMAIN.to_vec();
    bytes.extend(base);
    bytes.extend(hash);
    crate::terrain::sha256(&bytes)
}
pub(super) struct Runtime {
    profile: Profile,
    hash: [u8; 32],
    encounters: BTreeMap<u64, Encounter>,
    error: Option<String>,
}
struct Encounter {
    ledger: Ledger,
    last_kill: Option<KillCredit>,
    fatal: Option<Event>,
    sequence: u64,
    reset_mode: bool,
    departure: bool,
}
impl Encounter {
    fn new(profile: Profile, creature: EntityKey) -> Self {
        Self {
            ledger: Ledger::new(profile, creature),
            last_kill: None,
            fatal: None,
            sequence: 0,
            reset_mode: false,
            departure: false,
        }
    }
}
pub(super) struct ResolvedHit {
    pub source: LifeKey,
    pub target: EntityKey,
    pub category: Category,
    pub encounter: Option<EncounterKey>,
    pub health_lost: u32,
    pub fatal: bool,
}
pub(super) struct Planned {
    threat: Option<super::threat::Planned>,
    ledger: Ledger,
    kill: Option<KillCredit>,
    sequence: u64,
}
impl Runtime {
    fn new(profile: Profile, hash: [u8; 32], creatures: &[EntityKey]) -> Result<Self> {
        if creatures.is_empty() || creatures.len() > crate::targeting::MAX_TARGETS {
            return Err("Creature credit encounter count exceeds bounds".into());
        }
        let encounters: BTreeMap<_, _> = creatures
            .iter()
            .map(|&key| (key.entity.value(), Encounter::new(profile.clone(), key)))
            .collect();
        if encounters.len() != creatures.len() {
            return Err("Creature credit encounters contain duplicate identities".into());
        }
        Ok(Self {
            profile,
            hash,
            encounters,
            error: None,
        })
    }
    fn encounter(&self, key: EntityKey) -> Option<&Encounter> {
        self.encounters
            .get(&key.entity.value())
            .filter(|entry| entry.ledger.encounter().creature == key)
    }
    fn single_key(&self) -> Option<EntityKey> {
        (self.encounters.len() == 1).then(|| {
            self.encounters
                .values()
                .next()
                .unwrap()
                .ledger
                .encounter()
                .creature
        })
    }
    fn capture(&self, target: EntityKey) -> Option<EncounterKey> {
        let entry = self.encounter(target)?;
        if self.error.is_some() || entry.departure || entry.reset_mode || entry.last_kill.is_some()
        {
            return None;
        }
        Some(entry.ledger.encounter())
    }
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        identity(base, self.hash)
    }
    pub(super) fn leave(&mut self, id: u64) {
        for entry in self.encounters.values_mut() {
            if entry.last_kill.is_none()
                && entry
                    .ledger
                    .owner()
                    .is_some_and(|owner| owner.entity.value() == id)
            {
                entry.departure = true;
            }
        }
    }
}
impl World {
    pub fn enable_creature_credit(&mut self, bytes: &[u8]) -> Result<()> {
        let profile = Profile::from_json(bytes)?;
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Creature credit requires pre-admission classic World".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Creature credit requires combat")?;
        if combat.membership.is_none() || combat.credit.is_some() {
            return Err(
                "Creature credit requires explicit membership and no prior credit policy".into(),
            );
        }
        let (npc, _, _) = combat
            .membership_encounter()
            .ok_or("Creature credit requires pursuit")?;
        self.combat.as_mut().unwrap().credit = Some(Runtime::new(
            profile,
            crate::terrain::sha256(bytes),
            &[npc],
        )?);
        Ok(())
    }
    pub fn creature_credit_encounter(&self) -> Option<EncounterKey> {
        let target = self.combat.as_ref()?.credit.as_ref()?.single_key()?;
        self.creature_credit_encounter_for(target)
    }
    pub fn creature_credit_encounter_for(&self, target: EntityKey) -> Option<EncounterKey> {
        self.has_combat_incarnation(target).then_some(())?;
        self.combat.as_ref()?.capture_credit(target)
    }
    pub fn creature_tag_owner(&self) -> Option<EntityKey> {
        let target = self.combat.as_ref()?.credit.as_ref()?.single_key()?;
        self.creature_tag_owner_for(target)
    }
    pub fn creature_tag_owner_for(&self, target: EntityKey) -> Option<EntityKey> {
        self.has_combat_incarnation(target).then_some(())?;
        let runtime = self.combat.as_ref()?.credit.as_ref()?;
        let entry = runtime.encounter(target)?;
        if runtime.error.is_some() || entry.departure {
            return None;
        }
        entry.ledger.owner()
    }
    /// Retained for the exact current NPC lifetime. This query is not a grant.
    /// Legacy unaddressed observation is available only for one encounter.
    pub fn creature_kill_credit(&self) -> Option<&KillCredit> {
        let target = self.combat.as_ref()?.credit.as_ref()?.single_key()?;
        self.creature_kill_credit_for(target)
    }
    pub fn creature_kill_credit_for(&self, target: EntityKey) -> Option<&KillCredit> {
        self.has_combat_incarnation(target).then_some(())?;
        let runtime = self.combat.as_ref()?.credit.as_ref()?;
        if runtime.error.is_some() {
            return None;
        }
        runtime.encounter(target)?.last_kill.as_ref()
    }
    /// Explicit solo-recipient fixture. The existing accepted-quest policy and
    /// nonrepeatable training reward recipe still decide their own disposition.
    /// Connected dead owners remain eligible; departed owners have no fallback.
    pub(in crate::simulation) fn kill_recipient(
        &self,
        event: &Event,
    ) -> Result<Option<(EntityKey, EntityKey)>> {
        let Some((last_hitter, victim)) = event.fatal_player_hit() else {
            return Ok(None);
        };
        let Some(runtime) = self
            .combat
            .as_ref()
            .and_then(|combat| combat.credit.as_ref())
            .filter(|runtime| runtime.profile.recipient().is_some())
        else {
            return Ok(Some((last_hitter, victim)));
        };
        if runtime.error.is_some() {
            return Err("Creature credit authority unavailable".into());
        }
        let Some(entry) = runtime.encounter(victim) else {
            return Ok(None);
        };
        if entry.fatal.as_ref() != Some(event) {
            return Ok(None);
        }
        let Some(credit) = entry
            .last_kill
            .as_ref()
            .filter(|credit| credit.encounter.creature == victim)
        else {
            return Ok(None);
        };
        Ok(credit
            .owner
            .filter(|owner| self.entity_key(owner.entity.value()) == Some(*owner))
            .map(|owner| (owner, victim)))
    }
    pub fn creature_credit_error(&self) -> Option<&str> {
        self.combat.as_ref()?.credit.as_ref()?.error.as_deref()
    }
    pub(in crate::simulation) fn advance_creature_credit(&mut self) {
        if self
            .combat
            .as_ref()
            .is_none_or(|combat| combat.credit.is_none())
        {
            return;
        }
        if let Some(mut combat) = self.combat.take() {
            combat.sync_credit(self);
            combat.retire_threat_encounter();
            self.combat = Some(combat);
        }
    }
}
impl Authority {
    pub(super) fn capture_credit(&self, target: EntityKey) -> Option<EncounterKey> {
        self.credit.as_ref()?.capture(target)
    }
    pub(super) fn prepare_credit(
        &self,
        world: &World,
        hit: ResolvedHit,
    ) -> Result<Option<Planned>> {
        let ResolvedHit {
            source,
            target,
            category,
            encounter,
            health_lost,
            fatal,
        } = hit;
        let Some(runtime) = &self.credit else {
            return Ok(None);
        };
        let entry = runtime
            .encounter(target)
            .ok_or("Creature credit target unavailable")?;
        if runtime.error.is_some()
            || entry.departure
            || entry.reset_mode
            || entry.last_kill.is_some()
            || world.entity_key(source.actor.entity.value()) != Some(source.actor)
            || !self
                .actors
                .get(&source.actor.entity.value())
                .and_then(|a| a.life.as_ref())
                .is_some_and(|p| p.state.alive() && p.state.key() == source)
        {
            return Err("Creature credit authority or source unavailable".into());
        }
        let encounter = encounter
            .filter(|key| key.creature == target)
            .ok_or("Creature credit encounter unavailable")?;
        // Allocate a resolution serial, not the cast/request serial: a slow
        // projectile can legitimately land after a later instant ability.
        let sequence = entry
            .sequence
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or("Creature credit resolution serial exhausted")?;
        let mut ledger = entry.ledger.clone();
        ledger
            .record(
                world.tick,
                DamageFact {
                    encounter,
                    source,
                    category,
                    sequence,
                    health_lost,
                },
            )
            .map_err(|e| format!("Creature contribution refused: {e:?}"))?;
        let kill = if fatal {
            Some(
                ledger
                    .finish(world.tick, encounter)
                    .map_err(|e| format!("Creature credit death refused: {e:?}"))?,
            )
        } else {
            None
        };
        let threat = self.prepare_threat(source, health_lost)?;
        Ok(Some(Planned {
            threat,
            ledger,
            kill,
            sequence: sequence.get(),
        }))
    }
    pub(super) fn commit_credit(&mut self, planned: Option<Planned>) {
        if let Some(plan) = planned {
            let fatal = plan.kill.as_ref().map(|kill| {
                self.events
                    .iter()
                    .rev()
                    .find(|event| {
                        event
                            .fatal_player_hit()
                            .is_some_and(|(_, target)| target == kill.encounter.creature)
                    })
                    .expect("Prepared fatal damage publishes its immutable fact")
                    .clone()
            });
            let runtime = self
                .credit
                .as_mut()
                .expect("Prepared creature credit authority");
            let target = plan.ledger.encounter().creature;
            let entry = runtime
                .encounters
                .get_mut(&target.entity.value())
                .filter(|entry| entry.ledger.encounter().creature == target)
                .expect("Prepared exact creature credit encounter");
            entry.ledger = plan.ledger;
            entry.last_kill = plan.kill;
            entry.fatal = fatal;
            entry.sequence = plan.sequence;
            self.commit_threat(plan.threat);
        }
    }
    fn sync_credit(&mut self, world: &World) {
        let Some(mut runtime) = self.credit.take() else {
            return;
        };
        let result = (|| -> Result<()> {
            if runtime.error.is_some() {
                return Ok(());
            }
            let (npc, mode, _) = self
                .membership_encounter()
                .ok_or("Creature credit lost pursuit")?;
            // The installed pursuit adapter remains single-creature. A later
            // population adapter must provide its own complete lifecycle sync.
            let old = runtime
                .single_key()
                .ok_or("Pursuit credit requires one encounter")?;
            if old != npc {
                let sequence = runtime.encounters[&old.entity.value()].sequence;
                runtime.encounters.clear();
                let mut entry = Encounter::new(runtime.profile.clone(), npc);
                entry.sequence = sequence;
                runtime.encounters.insert(npc.entity.value(), entry);
            }
            let entry = runtime.encounters.get_mut(&npc.entity.value()).unwrap();
            let resetting = matches!(mode, Mode::Return | Mode::Blocked);
            if entry.last_kill.is_none() && (entry.departure || (resetting && !entry.reset_mode)) {
                entry
                    .ledger
                    .reset(world.tick, entry.ledger.encounter())
                    .map_err(|e| format!("Creature credit reset refused: {e:?}"))?;
                entry.departure = false;
            }
            entry.reset_mode = resetting;
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.credit = Some(runtime);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    use theia_core::identity::{EntityId, WorldEpoch};
    fn key(n: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(n).unwrap(),
        }
    }
    fn profile() -> Profile {
        Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-creature-credit-solo.json"
        ))
        .unwrap()
    }
    fn tag(runtime: &mut Runtime, target: EntityKey, actor: EntityKey) {
        let entry = runtime.encounters.get_mut(&target.entity.value()).unwrap();
        entry
            .ledger
            .record(
                1,
                DamageFact {
                    encounter: entry.ledger.encounter(),
                    source: LifeKey {
                        actor,
                        generation: NonZeroU64::new(1).unwrap(),
                    },
                    category: Category::AbilityDamage,
                    sequence: NonZeroU64::new(1).unwrap(),
                    health_lost: 20,
                },
            )
            .unwrap();
    }
    #[test]
    fn independent_encounters_keep_tags_departures_and_reset_generations_separate() {
        let mut runtime = Runtime::new(profile(), [0; 32], &[key(20), key(21)]).unwrap();
        assert!(runtime.single_key().is_none());
        let old = runtime.capture(key(20)).unwrap();
        let other = runtime.capture(key(21)).unwrap();
        tag(&mut runtime, key(20), key(1));
        tag(&mut runtime, key(21), key(2));
        runtime.leave(1);
        assert!(runtime.capture(key(20)).is_none());
        assert_eq!(runtime.capture(key(21)), Some(other));
        assert_eq!(
            runtime.encounter(key(21)).unwrap().ledger.owner(),
            Some(key(2))
        );
        let entry = runtime.encounters.get_mut(&20).unwrap();
        entry.ledger.reset(2, old).unwrap();
        entry.departure = false;
        assert_ne!(runtime.capture(key(20)), Some(old));
        assert_eq!(runtime.capture(key(21)), Some(other));
        let mut stale_epoch = key(20);
        stale_epoch.epoch = WorldEpoch::new(2).unwrap();
        assert!(runtime.capture(stale_epoch).is_none());
        assert!(runtime.capture(key(22)).is_none());
        let entry = runtime.encounters.get_mut(&21).unwrap();
        entry.last_kill = Some(entry.ledger.finish(2, other).unwrap());
        runtime.leave(2);
        let entry = runtime.encounter(key(21)).unwrap();
        assert!(!entry.departure);
        assert_eq!(entry.last_kill.as_ref().unwrap().owner, Some(key(2)));
        assert!(runtime.capture(key(21)).is_none());
        assert!(runtime.capture(key(20)).is_some());
    }
    #[test]
    fn encounter_registry_refuses_empty_duplicate_and_over_capacity_inputs() {
        assert!(Runtime::new(profile(), [0; 32], &[]).is_err());
        assert!(Runtime::new(profile(), [0; 32], &[key(1), key(1)]).is_err());
        let mut other_epoch = key(1);
        other_epoch.epoch = WorldEpoch::new(2).unwrap();
        assert!(Runtime::new(profile(), [0; 32], &[key(1), other_epoch]).is_err());
        let keys: Vec<_> = (1..=crate::targeting::MAX_TARGETS as u64 + 1)
            .map(key)
            .collect();
        assert!(Runtime::new(profile(), [0; 32], &keys).is_err());
        assert!(Runtime::new(profile(), [0; 32], &keys[..crate::targeting::MAX_TARGETS]).is_ok());
    }
}
