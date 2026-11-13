//! Explicit per-creature return reset. No respawn, grants, paths or I/O.
use super::{Authority, World};
use crate::Result;
use serde::Deserialize;
use theia_core::{content::ContentId, identity::EntityKey, pursuit::Mode};
pub const MAX_PROFILE_BYTES: usize = 4096;
const DOMAIN: &[u8] = b"theia-creature-return-reset-world-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: String,
    creature: String,
}
#[derive(Clone, Debug)]
pub struct Profile {
    id: ContentId,
    creature: ContentId,
    hash: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Creature reset profile exceeds byte budget".into());
        }
        let raw: Raw =
            serde_json::from_slice(bytes).map_err(|_| "Invalid creature reset policy")?;
        if raw.schema != 1 || raw.ruleset != "creature_return_reset_fixture_v1" {
            return Err("Unsupported creature reset policy".into());
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid reset policy ID")?,
            creature: ContentId::new(&raw.creature).ok_or("Invalid reset creature ID")?,
            hash: crate::terrain::sha256(bytes),
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = DOMAIN.to_vec();
        bytes.extend(base);
        bytes.extend(self.hash);
        crate::terrain::sha256(&bytes)
    }
}
#[derive(Clone, Copy, Debug, Default, serde::Serialize, PartialEq, Eq)]
pub struct Metrics {
    pub resets: u64,
    pub health_restored: u64,
    pub statuses_removed: u64,
}
struct Encounter {
    npc: EntityKey,
    in_reset: bool,
}
pub(super) struct Runtime {
    profile: Profile,
    encounters: Vec<Encounter>,
    error: Option<String>,
    metrics: Metrics,
}
impl Runtime {
    pub(super) fn identity(&self, base: [u8; 32]) -> [u8; 32] {
        self.profile.identity(base)
    }
    pub(super) fn ready(&self) -> bool {
        self.error.is_none()
    }
}
impl World {
    pub fn enable_creature_reset(&mut self, bytes: &[u8]) -> Result<()> {
        let profile = Profile::from_json(bytes)?;
        if self.tick != 0 || !self.players.is_empty() || !self.classic {
            return Err("Creature reset requires pre-admission classic World".into());
        }
        let combat = self
            .combat
            .as_ref()
            .ok_or("Creature reset requires combat")?;
        let encounters = combat.checked_reset_encounters()?;
        if combat.reset.is_some()
            || combat.credit.is_none()
            || combat.membership.is_none()
            || (!combat.ordinary_target_ready() && !combat.population_physical_ready(self))
            || encounters.iter().any(|(npc, _, _)| {
                self.creature_definition(*npc)
                    .is_none_or(|d| profile.creature() != d.id())
            })
        {
            return Err("Creature reset requires matching explicit typed credit encounters".into());
        }
        self.combat.as_mut().unwrap().reset = Some(Runtime {
            profile,
            encounters: encounters
                .into_iter()
                .map(|(npc, _, _)| Encounter {
                    npc,
                    in_reset: false,
                })
                .collect(),
            error: None,
            metrics: Metrics::default(),
        });
        Ok(())
    }
    pub fn creature_reset_metrics(&self) -> Option<Metrics> {
        Some(self.combat.as_ref()?.reset.as_ref()?.metrics)
    }
    pub fn creature_reset_error(&self) -> Option<&str> {
        self.combat.as_ref()?.reset.as_ref()?.error.as_deref()
    }
    pub(in crate::simulation) fn advance_creature_reset(&mut self) {
        if self
            .combat
            .as_ref()
            .is_none_or(|combat| combat.reset.is_none())
        {
            return;
        }
        if let Some(mut combat) = self.combat.take() {
            combat.sync_reset();
            self.combat = Some(combat);
        }
    }
}
impl Authority {
    fn sync_reset(&mut self) {
        let Some(mut runtime) = self.reset.take() else {
            return;
        };
        let result = (|| -> Result<()> {
            if runtime.error.is_some() {
                return Ok(());
            }
            let encounters = self.checked_reset_encounters()?;
            let mut next = Vec::with_capacity(encounters.len());
            let mut resets = Vec::with_capacity(encounters.len());
            // Validate every transition before changing health, effects or
            // lifecycle state. A new entity starts a fresh reset latch.
            for (npc, mode, _) in encounters {
                let returning = matches!(mode, Mode::Return | Mode::Blocked);
                let previous = runtime
                    .encounters
                    .iter()
                    .find(|e| e.npc == npc)
                    .is_some_and(|e| e.in_reset);
                if returning && !previous {
                    let creature = self
                        .creatures
                        .iter()
                        .find(|c| c.key == npc)
                        .ok_or("Reset creature lifetime unavailable")?;
                    if creature.current == 0 || creature.respawn_at.is_some() {
                        return Err("Dead creature cannot enter return reset".into());
                    }
                    let restored = creature
                        .maximum
                        .checked_sub(creature.current)
                        .ok_or("Invalid reset creature health")?;
                    resets.push((npc, restored));
                }
                next.push(Encounter {
                    npc,
                    in_reset: returning,
                });
            }
            for (npc, restored) in resets {
                let creature = self
                    .creatures
                    .iter_mut()
                    .find(|c| c.key == npc)
                    .expect("Validated reset creature");
                creature.current = creature.maximum;
                let removed = self
                    .statuses
                    .as_mut()
                    .map_or(0, |statuses| statuses.remove_target(npc));
                runtime.metrics.resets = runtime.metrics.resets.saturating_add(1);
                runtime.metrics.health_restored = runtime
                    .metrics
                    .health_restored
                    .saturating_add(u64::from(restored));
                runtime.metrics.statuses_removed = runtime
                    .metrics
                    .statuses_removed
                    .saturating_add(removed as u64);
            }
            runtime.encounters = next;
            Ok(())
        })();
        if let Err(error) = result {
            runtime.error = Some(error.to_string());
        }
        self.reset = Some(runtime);
    }
}
