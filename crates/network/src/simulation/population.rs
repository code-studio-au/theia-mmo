//! Atomic admission of one checked stationary population before players/combat.
//! This does not bind creature AI, models, group-specific rewards or roaming.
use super::World;
use crate::{
    Result,
    navigation::spawn_preparation::Prepared,
    targeting::{MAX_TARGETS, Target},
};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey},
};

/// One resolved passive creature type and its authored group respawn policy.
/// Presentation/model resolution and multi-creature pursuit remain separate.
#[derive(Clone)]
pub struct Binding {
    definition: super::creatures::Definition,
    respawn_ms: u32,
}
impl Binding {
    pub fn new(
        requested: &theia_core::content::ContentId,
        definition: super::creatures::Definition,
        respawn_seconds: u32,
    ) -> Result<Self> {
        if definition.id() != requested {
            return Err("Population creature reference does not match its definition".into());
        }
        if definition.aggro().is_some() {
            return Err("Population pursuit is not connected; aggressive creature definitions are unsupported".into());
        }
        if !(1..=super::combat::MAX_RESPAWN_MS / 1000).contains(&respawn_seconds) {
            return Err("Population respawn delay must be 1..86400 seconds".into());
        }
        Ok(Self {
            definition,
            respawn_ms: respawn_seconds * 1000,
        })
    }
    pub fn definition(&self) -> &super::creatures::Definition {
        &self.definition
    }
    pub fn respawn_ms(&self) -> u32 {
        self.respawn_ms
    }
    pub fn profile_identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-population-life-world-v1\0".to_vec();
        bytes.extend(self.definition.profile_identity(base));
        bytes.extend(self.respawn_ms.to_le_bytes());
        crate::terrain::sha256(&bytes)
    }
}

impl World {
    /// Resolve one passive creature type for this pool before publishing actors.
    /// Multi-creature pursuit is not installed yet; aggressive definitions fail
    /// explicitly rather than silently losing their behavior.
    pub fn enable_defined_population(
        &mut self,
        prepared: Prepared,
        binding: Binding,
    ) -> Result<Vec<EntityKey>> {
        let keys = self.enable_prepared_population(prepared)?;
        self.population_binding = Some(binding);
        Ok(keys)
    }
    /// Definition for an exact live actor incarnation, independent of its route.
    /// Unbound development targets and retired incarnations return None.
    pub fn creature_definition(&self, key: EntityKey) -> Option<&super::creatures::Definition> {
        if self
            .targets
            .get(&key.entity.value())
            .is_none_or(|t| t.key != key)
        {
            return None;
        }
        if let Some(patrol) = self.patrol.as_ref().filter(|p| p.key == key) {
            return Some(patrol.profile().definition());
        }
        self.population_binding.as_ref().map(Binding::definition)
    }
    /// Consume complete navigation/collision preparation. No file I/O, collision
    /// construction or navigation search occurs here or in the fixed step.
    /// Existing combat installs health and respawn timing afterward.
    pub fn enable_prepared_population(&mut self, prepared: Prepared) -> Result<Vec<EntityKey>> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.combat.is_some()
            || self.patrol.is_some()
            || self.population.is_some()
            || !self.targets.is_empty()
            || prepared.points().is_empty()
            || prepared.points().len() > MAX_TARGETS
        {
            return Err(
                "Population requires inactive empty targeting world and at most 16 checked points"
                    .into(),
            );
        }
        let runtime = self
            .procedural
            .as_ref()
            .ok_or("Population requires authored terrain")?;
        if runtime.source.identity() != prepared.source_identity() {
            return Err("Population prepared for another world".into());
        }
        let next_id = self
            .next_id
            .checked_add(prepared.points().len() as u64)
            .ok_or("Population entity identities exhausted")?;
        let targets = prepared
            .points()
            .iter()
            .enumerate()
            .map(|(index, feet)| {
                let mut position = feet.metres();
                position[1] += f64::from(crate::world::PAWN_Y);
                Ok(Target {
                    key: EntityKey {
                        epoch: self.epoch,
                        entity: EntityId::new(self.next_id + index as u64 + 1).unwrap(),
                    },
                    position: GlobalPosition::new(position)
                        .ok_or("Invalid population body position")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        // All fallible admission checks precede publication of any patch or actor.
        let runtime = self.procedural.as_mut().unwrap();
        for patch in prepared.patches() {
            if runtime.patches.insert(patch.owner, patch.clone()).is_some() {
                runtime.retired = runtime.retired.saturating_add(1);
            }
            runtime.generated = runtime.generated.saturating_add(1);
        }
        let keys = targets.iter().map(|t| t.key).collect();
        for target in targets {
            self.targets.insert(target.key.entity.value(), target);
        }
        self.next_id = next_id;
        self.targeting_enabled = true;
        self.population = Some(prepared);
        Ok(keys)
    }
}
