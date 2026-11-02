//! Opt-in ordinary-creature ownership. No XP/quest/loot consumer is changed.
use super::{Authority, Event, World};
use crate::Result;
use std::num::NonZeroU64;
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
    ledger: Ledger,
    last_kill: Option<KillCredit>,
    fatal: Option<Event>,
    sequence: u64,
    reset_mode: bool,
    departure: bool,
    error: Option<String>,
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
    ledger: Ledger,
    kill: Option<KillCredit>,
    sequence: u64,
}
impl Runtime {
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        identity(base, self.hash)
    }
    pub(super) fn leave(&mut self, id: u64) {
        if self.last_kill.is_none()
            && self
                .ledger
                .owner()
                .is_some_and(|owner| owner.entity.value() == id)
        {
            self.departure = true;
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
        self.combat.as_mut().unwrap().credit = Some(Runtime {
            ledger: Ledger::new(profile.clone(), npc),
            profile,
            hash: crate::terrain::sha256(bytes),
            last_kill: None,
            fatal: None,
            sequence: 0,
            reset_mode: false,
            departure: false,
            error: None,
        });
        Ok(())
    }
    pub fn creature_credit_encounter(&self) -> Option<EncounterKey> {
        self.combat.as_ref()?.capture_credit()
    }
    pub fn creature_tag_owner(&self) -> Option<EntityKey> {
        let runtime = self.combat.as_ref()?.credit.as_ref()?;
        if runtime.error.is_some() || runtime.departure {
            return None;
        }
        runtime.ledger.owner()
    }
    /// Retained for the exact current NPC lifetime. A later durable adapter must
    /// consume the encounter identity idempotently; this query is not a grant.
    pub fn creature_kill_credit(&self) -> Option<&KillCredit> {
        let runtime = self.combat.as_ref()?.credit.as_ref()?;
        if runtime.error.is_some() {
            return None;
        }
        runtime.last_kill.as_ref()
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
        if runtime.fatal.as_ref() != Some(event) {
            return Ok(None);
        }
        let Some(credit) = runtime
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
            self.combat = Some(combat);
        }
    }
}
impl Authority {
    pub(super) fn capture_credit(&self) -> Option<EncounterKey> {
        let runtime = self.credit.as_ref()?;
        if runtime.error.is_some()
            || runtime.departure
            || runtime.reset_mode
            || runtime.last_kill.is_some()
        {
            return None;
        }
        Some(runtime.ledger.encounter())
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
        if runtime.error.is_some()
            || runtime.departure
            || runtime.reset_mode
            || runtime.last_kill.is_some()
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
        let sequence = runtime
            .sequence
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or("Creature credit resolution serial exhausted")?;
        let mut ledger = runtime.ledger.clone();
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
        Ok(Some(Planned {
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
            runtime.ledger = plan.ledger;
            runtime.last_kill = plan.kill;
            runtime.fatal = fatal;
            runtime.sequence = plan.sequence;
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
            let resetting = matches!(mode, Mode::Return | Mode::Blocked);
            if runtime.ledger.encounter().creature != npc {
                runtime.ledger = Ledger::new(runtime.profile.clone(), npc);
                runtime.last_kill = None;
                runtime.fatal = None;
                runtime.departure = false;
            } else if runtime.last_kill.is_none()
                && (runtime.departure || (resetting && !runtime.reset_mode))
            {
                runtime
                    .ledger
                    .reset(world.tick, runtime.ledger.encounter())
                    .map_err(|e| format!("Creature credit reset refused: {e:?}"))?;
                runtime.departure = false;
            }
            runtime.reset_mode = resetting;
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.credit = Some(runtime);
    }
}
