//! Opt-in bounded per-creature damage threat. No direct network damage input or rewards.
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
    npc: EntityKey,
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
    pub(super) fn respawned(&mut self, previous: EntityKey, next: EntityKey) {
        if self.npc == previous {
            self.sync_encounter(next, Mode::Patrol, true);
        }
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
            || !combat.population_threat.is_empty()
            || combat.reset.is_none()
            || combat.credit.is_none()
            || (!combat.ordinary_target_ready() && !combat.population_physical_ready(self))
        {
            return Err("Threat requires explicit typed credit and return reset".into());
        }
        let encounters = combat.checked_reset_encounters()?;
        let runtimes: Vec<_> = encounters
            .into_iter()
            .map(|(npc, _, _)| Runtime {
                table: Table::new(profile.clone(), npc),
                profile: profile.clone(),
                npc,
                hash: crate::terrain::sha256(bytes),
                error: None,
                metrics: Metrics::default(),
            })
            .collect();
        let combat = self.combat.as_mut().unwrap();
        if combat.pve.is_some() {
            combat.threat = runtimes.into_iter().next();
        } else {
            combat.population_threat = runtimes;
        }
        Ok(())
    }
    pub fn creature_threat_entries(&self) -> Option<&[Entry]> {
        Some(self.combat.as_ref()?.threat.as_ref()?.table.entries())
    }
    pub fn creature_threat_entries_for(&self, npc: EntityKey) -> Option<&[Entry]> {
        let combat = self.combat.as_ref()?;
        Some(
            combat
                .threat
                .iter()
                .chain(&combat.population_threat)
                .find(|t| t.npc == npc)?
                .table
                .entries(),
        )
    }
    pub fn creature_threat_metrics(&self) -> Option<Metrics> {
        let combat = self.combat.as_ref()?;
        let mut total = Metrics::default();
        let mut found = false;
        for runtime in combat.threat.iter().chain(&combat.population_threat) {
            found = true;
            total.damage_facts += runtime.metrics.damage_facts;
            total.health_lost += runtime.metrics.health_lost;
            total.transfers += runtime.metrics.transfers;
            total.removed_lives += runtime.metrics.removed_lives;
            total.clears += runtime.metrics.clears;
        }
        found.then_some(total)
    }
    pub fn creature_threat_error(&self) -> Option<&str> {
        let combat = self.combat.as_ref()?;
        combat
            .threat
            .iter()
            .chain(&combat.population_threat)
            .find_map(|t| t.error.as_deref())
    }
    pub(in crate::simulation) fn advance_creature_threat(&mut self) {
        if self
            .combat
            .as_ref()
            .is_none_or(|c| c.threat.is_none() && c.population_threat.is_empty())
        {
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
        npc: EntityKey,
        source: LifeKey,
        health_lost: u32,
    ) -> Result<Option<Planned>> {
        if self.threat.is_none() && self.population_threat.is_empty() {
            return Ok(None);
        }
        let runtime = self
            .threat
            .iter()
            .chain(&self.population_threat)
            .find(|t| t.npc == npc)
            .ok_or("Threat victim encounter unavailable")?;
        if !runtime.ready() || !self.checked_reset_encounters()?.iter().any(|e| e.0 == npc) {
            return Err("Threat authority unavailable".into());
        }
        let mut table = runtime.table.clone();
        if let Some(victim) = self.threat_current_victim_for(npc) {
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
        Ok(Some(Planned {
            npc,
            table,
            metrics,
        }))
    }
    pub(super) fn commit_threat(&mut self, plan: Option<Planned>) {
        if let Some(plan) = plan {
            let runtime = self
                .threat
                .iter_mut()
                .chain(&mut self.population_threat)
                .find(|t| t.npc == plan.npc)
                .expect("Prepared threat authority");
            assert_eq!(runtime.npc, plan.npc, "Prepared exact threat encounter");
            runtime.table = plan.table;
            runtime.metrics = plan.metrics;
        }
    }
    /// Lifecycle-only cleanup after pursuit/reset/death. Never select or transfer
    /// here: retaliation may already have evaluated this tick. Early-frame
    /// selection still owns victim changes and their prepared path invalidation.
    pub(super) fn retire_threat_encounter(&mut self) {
        if self.threat.is_none() && self.population_threat.is_empty() {
            return;
        }
        let encounters = self.checked_reset_encounters();
        for runtime in self
            .threat
            .iter_mut()
            .chain(&mut self.population_threat)
            .filter(|r| r.ready())
        {
            match &encounters {
                Ok(encounters) => {
                    if let Some((npc, mode, _)) = encounters.iter().find(|e| e.0 == runtime.npc) {
                        runtime.sync_encounter(
                            *npc,
                            *mode,
                            self.creatures
                                .iter()
                                .any(|c| c.key == *npc && c.current > 0),
                        );
                    } else {
                        runtime.table.clear();
                    }
                }
                Err(error) => runtime.error = Some(error.to_string()),
            }
        }
    }
    pub(in crate::simulation::combat) fn sync_threat(&mut self, world: &World) {
        if let Some(mut runtime) = self.threat.take() {
            let npc = self.pve.as_ref().map_or(runtime.npc, |p| p.npc);
            self.sync_one_threat(world, &mut runtime, npc);
            self.threat = Some(runtime);
        }
        let mut population = std::mem::take(&mut self.population_threat);
        for runtime in &mut population {
            self.sync_one_threat(world, runtime, runtime.npc);
        }
        self.population_threat = population;
    }
    fn sync_one_threat(&mut self, world: &World, runtime: &mut Runtime, npc: EntityKey) {
        let result = (|| -> Result<()> {
            if !runtime.ready() {
                return Ok(());
            }
            let (_, mode, current) = self
                .checked_reset_encounters()?
                .into_iter()
                .find(|e| e.0 == npc)
                .ok_or("Threat lost exact encounter")?;
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
            if let Some(target) =
                current.and_then(|life| self.threat_candidate_for(world, npc, life))
            {
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
                .filter_map(|entry| self.threat_candidate_for(world, npc, entry.life))
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
            let transfer = if next.is_some()
                && (next != current || self.threat_policy_target_for(npc) != next)
            {
                let target = candidates
                    .iter()
                    .find(|t| Some(t.key) == next)
                    .copied()
                    .ok_or("Threat lost its prepared victim")?;
                Some(self.prepare_threat_transfer_for(world, npc, target)?)
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
    }
}
