//! Opt-in one-creature damage threat. No direct network damage input or rewards.
use super::{Authority, World};
use crate::Result;
use theia_core::{
    identity::EntityKey,
    player_life::LifeKey,
    pursuit::Mode,
    threat::{Entry, Profile, Table},
};
const DOMAIN: &[u8] = b"theia-creature-damage-threat-world-v1\0";
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
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub damage_facts: u64,
    pub health_lost: u64,
    pub transfers: u64,
    pub removed_lives: u64,
    pub clears: u64,
}
pub(super) struct Runtime {
    pub(super) profile: Profile,
    pub(super) npc: EntityKey,
    pub(super) table: Table,
    pub(super) error: Option<String>,
    pub(super) metrics: Metrics,
    hash: [u8; 32],
}
pub(super) struct Planned {
    table: Table,
    metrics: Metrics,
}
impl Runtime {
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        identity(base, self.hash)
    }
    pub(super) fn ready(&self) -> bool {
        self.error.is_none()
    }
    fn sync_encounter(&mut self, npc: EntityKey, mode: Mode, alive: bool) -> bool {
        if npc != self.npc {
            if !self.table.entries().is_empty() {
                self.metrics.clears = self.metrics.clears.saturating_add(1);
            }
            self.table = Table::new(self.profile.clone(), npc);
            self.npc = npc;
        }
        if !alive || matches!(mode, Mode::Return | Mode::Blocked | Mode::Dead) {
            if !self.table.entries().is_empty() {
                self.table.clear();
                self.metrics.clears = self.metrics.clears.saturating_add(1);
            }
            return false;
        }
        true
    }
    pub(super) fn contains(&self, life: LifeKey) -> bool {
        self.ready() && self.table.entries().iter().any(|e| e.life == life)
    }
}
impl World {
    pub fn enable_creature_threat(&mut self, bytes: &[u8]) -> Result<()> {
        let profile = Profile::from_json(bytes)?;
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Threat requires pre-admission classic World".into());
        }
        let combat = self.combat.as_ref().ok_or("Threat requires combat")?;
        if combat.threat.is_some()
            || combat.reset.is_none()
            || combat.credit.is_none()
            || !combat.ordinary_target_ready()
        {
            return Err("Threat requires explicit typed credit and return reset".into());
        }
        let (npc, _, _) = combat.checked_reset_encounter()?;
        self.combat.as_mut().unwrap().threat = Some(Runtime {
            table: Table::new(profile.clone(), npc),
            profile,
            npc,
            hash: crate::terrain::sha256(bytes),
            error: None,
            metrics: Metrics::default(),
        });
        Ok(())
    }
    pub fn creature_threat_entries(&self) -> Option<&[Entry]> {
        Some(self.combat.as_ref()?.threat.as_ref()?.table.entries())
    }
    pub fn creature_threat_metrics(&self) -> Option<Metrics> {
        Some(self.combat.as_ref()?.threat.as_ref()?.metrics)
    }
    pub fn creature_threat_error(&self) -> Option<&str> {
        self.combat.as_ref()?.threat.as_ref()?.error.as_deref()
    }
    pub(in crate::simulation) fn advance_creature_threat(&mut self) {
        if self.combat.as_ref().is_none_or(|c| c.threat.is_none()) {
            return;
        }
        if let Some(mut combat) = self.combat.take() {
            combat.sync_threat(self);
            self.combat = Some(combat);
        }
    }
}
impl Authority {
    pub(super) fn prepare_threat(
        &self,
        source: LifeKey,
        health_lost: u32,
    ) -> Result<Option<Planned>> {
        let Some(runtime) = &self.threat else {
            return Ok(None);
        };
        if !runtime.ready() || self.checked_reset_encounter()?.0 != runtime.npc {
            return Err("Threat authority unavailable".into());
        }
        let mut table = runtime.table.clone();
        if let Some(victim) = self.threat_current_victim() {
            table
                .register(victim)
                .map_err(|e| format!("Threat victim registration refused: {e:?}"))?;
        }
        // Explicit valid hostile attempts can register zero threat. The caller
        // has already checked the exact life, encounter and committed resolution.
        table
            .register(source)
            .map_err(|e| format!("Threat participant refused: {e:?}"))?;
        table
            .add_damage(source, health_lost)
            .map_err(|e| format!("Damage threat refused: {e:?}"))?;
        let mut metrics = runtime.metrics;
        metrics.damage_facts = metrics.damage_facts.saturating_add(1);
        metrics.health_lost = metrics.health_lost.saturating_add(u64::from(health_lost));
        Ok(Some(Planned { table, metrics }))
    }
    pub(super) fn commit_threat(&mut self, plan: Option<Planned>) {
        if let Some(plan) = plan {
            let runtime = self.threat.as_mut().expect("Prepared threat authority");
            runtime.table = plan.table;
            runtime.metrics = plan.metrics;
        }
    }
    /// Lifecycle-only cleanup after pursuit/reset/death. Never select or transfer
    /// here: retaliation may already have evaluated this tick. Early-frame
    /// selection still owns victim changes and their prepared path invalidation.
    pub(super) fn retire_threat_encounter(&mut self) {
        if self.threat.is_none() {
            return;
        }
        let encounter = self.checked_reset_encounter();
        let Some(runtime) = self.threat.as_mut().filter(|r| r.ready()) else {
            return;
        };
        match encounter {
            Ok((npc, mode, _)) => {
                runtime.sync_encounter(
                    npc,
                    mode,
                    self.creatures.iter().any(|c| c.key == npc && c.current > 0),
                );
            }
            Err(error) => runtime.error = Some(error.to_string()),
        }
    }
    pub(in crate::simulation::combat) fn sync_threat(&mut self, world: &World) {
        let Some(mut runtime) = self.threat.take() else {
            return;
        };
        let result = (|| -> Result<()> {
            if !runtime.ready() {
                return Ok(());
            }
            let (npc, mode, current) = self.checked_reset_encounter()?;
            if !runtime.sync_encounter(
                npc,
                mode,
                self.creatures.iter().any(|c| c.key == npc && c.current > 0),
            ) {
                return Ok(());
            }
            let mut table = runtime.table.clone();
            let retired: Vec<_> = table
                .entries()
                .iter()
                .filter(|entry| {
                    !self
                        .actors
                        .get(&entry.life.actor.entity.value())
                        .and_then(|a| a.life.as_ref())
                        .is_some_and(|p| {
                            p.state.alive()
                                && p.state.key() == entry.life
                                && world.entity_key(entry.life.actor.entity.value())
                                    == Some(entry.life.actor)
                        })
                })
                .map(|e| e.life)
                .collect();
            for life in &retired {
                table.remove(*life);
            }
            if !matches!(mode, Mode::Chase | Mode::Attack) {
                runtime.table = table;
                runtime.metrics.removed_lives = runtime
                    .metrics
                    .removed_lives
                    .saturating_add(retired.len() as u64);
                return Ok(());
            }
            if let Some(target) = current.and_then(|life| self.threat_candidate(world, life)) {
                table
                    .register(target.key)
                    .map_err(|e| format!("Threat body pull refused: {e:?}"))?;
                if table.current().is_none() {
                    table
                        .select(&[theia_core::threat::Eligible {
                            life: target.key,
                            in_melee: target.in_melee,
                        }])
                        .map_err(|e| format!("Threat victim seed refused: {e:?}"))?;
                }
            }
            let candidates: Vec<_> = table
                .entries()
                .iter()
                .filter_map(|entry| self.threat_candidate(world, entry.life))
                .collect();
            let eligible: Vec<_> = candidates
                .iter()
                .map(|t| theia_core::threat::Eligible {
                    life: t.key,
                    in_melee: t.in_melee,
                })
                .collect();
            let next = table
                .select(&eligible)
                .map_err(|e| format!("Threat selection refused: {e:?}"))?;
            let transfer =
                if next.is_some() && (next != current || self.threat_policy_target() != next) {
                    let target = candidates
                        .iter()
                        .find(|t| Some(t.key) == next)
                        .copied()
                        .ok_or("Threat lost its prepared victim")?;
                    Some(self.prepare_threat_transfer(world, target)?)
                } else {
                    None
                };
            if let Some(transfer) = transfer {
                self.commit_threat_transfer(transfer);
                runtime.metrics.transfers = runtime
                    .metrics
                    .transfers
                    .saturating_add(u64::from(next != current));
            }
            runtime.table = table;
            runtime.metrics.removed_lives = runtime
                .metrics
                .removed_lives
                .saturating_add(retired.len() as u64);
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.threat = Some(runtime);
    }
}
