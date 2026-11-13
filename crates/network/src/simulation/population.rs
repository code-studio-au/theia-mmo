//! Atomic admission of checked creature placements before players/combat.
//! Optional prepared pursuit reuses the ordinary AI and static-world controller.
pub mod authoring;
pub mod configuration;
pub(super) mod motion;
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

/// One resolved creature type and its authored group respawn policy.
/// Presentation/model resolution remains separate.
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
    /// Content references are type IDs, independent of live/dead actor instances.
    pub(super) fn has_creature_definition(&self, id: &theia_core::content::ContentId) -> bool {
        self.patrol
            .as_ref()
            .is_some_and(|patrol| patrol.definition() == id)
            || self
                .population_binding
                .as_ref()
                .is_some_and(|binding| binding.definition().id() == id)
    }
    /// Resolve trusted combat facts even after targetability ends on death.
    /// Retired lives after respawn, unbound targets and player IDs do not resolve.
    pub(super) fn combat_creature_definition(
        &self,
        key: EntityKey,
    ) -> Option<&super::creatures::Definition> {
        if !self.has_combat_incarnation(key) {
            return None;
        }
        if let Some(patrol) = self.patrol.as_ref().filter(|p| p.key == key) {
            return Some(patrol.profile().definition());
        }
        self.population_binding.as_ref().map(Binding::definition)
    }
    /// Exact live creature orientation. Stationary definitions spawn facing -Z;
    /// pursuing creatures face their movement or their own current attack target.
    pub fn creature_facing(&self, key: EntityKey) -> Option<theia_core::facing::Facing> {
        self.creature_definition(key)?;
        if self.patrol.is_some() {
            self.patrol_facing(key)
        } else {
            self.population.as_ref()?;
            Some(
                self.population_motion
                    .iter()
                    .find(|p| p.key == key)
                    .map_or_else(theia_core::facing::Facing::default, |p| p.facing),
            )
        }
    }
    /// Publish stationary definitions. Pursuing definitions must use the
    /// prepared-navigation installer rather than silently losing their behavior.
    pub fn enable_defined_population(
        &mut self,
        prepared: Prepared,
        binding: Binding,
    ) -> Result<Vec<EntityKey>> {
        if binding.definition().aggro().is_some() {
            return Err("Aggressive population requires prepared pursuit navigation".into());
        }
        let keys = self.enable_prepared_population(prepared)?;
        self.population_binding = Some(binding);
        Ok(keys)
    }
    /// Prepare all collision bodies and one shared worker before publishing actors.
    /// Matching player-life installation subsequently binds the ordinary AI policy.
    pub fn enable_navigating_population(
        &mut self,
        prepared: Prepared,
        binding: Binding,
        worker: crate::navigation::worker::Worker,
    ) -> Result<Vec<EntityKey>> {
        if binding.definition().aggro().is_none()
            || worker.navigation() != prepared.navigation_identity()
        {
            return Err("Population pursuit requires matching settings and navigation".into());
        }
        let mut motion = Vec::with_capacity(prepared.points().len());
        for (index, feet) in prepared.points().iter().enumerate() {
            let owner = crate::procedural::owner(*feet).ok_or("Invalid population patch owner")?;
            let patch = prepared
                .patches()
                .iter()
                .find(|p| p.owner == owner)
                .ok_or("Missing prepared population collision")?
                .clone();
            let mut position = feet.metres();
            position[1] += f64::from(crate::world::PAWN_Y);
            let key = EntityKey {
                epoch: self.epoch,
                entity: EntityId::new(
                    self.next_id
                        .checked_add(index as u64 + 1)
                        .ok_or("Population IDs exhausted")?,
                )
                .ok_or("Invalid population ID")?,
            };
            motion.push(motion::Actor::new(
                key,
                GlobalPosition::new(position).ok_or("Invalid population position")?,
                patch,
            )?);
        }
        let keys = self.enable_prepared_population(prepared)?;
        self.population_binding = Some(binding);
        self.population_motion = motion;
        self.population_navigation = Some(worker);
        Ok(keys)
    }
    pub(super) fn advance_population_pursuit(&mut self) {
        if self.population_motion.is_empty() {
            return;
        }
        let Some(mut combat) = self.combat.take() else {
            return;
        };
        let directives = combat.advance_population_pursuit(self);
        self.combat = Some(combat);
        let mut actors = std::mem::take(&mut self.population_motion);
        for actor in &mut actors {
            if let Some((_, directive)) = directives.iter().find(|(key, _)| *key == actor.key) {
                actor.advance(self, *directive);
            }
        }
        self.population_motion = actors;
    }
    pub(super) fn population_respawn(
        &mut self,
        previous: EntityKey,
        next: EntityKey,
    ) -> Option<GlobalPosition> {
        self.population_motion
            .iter_mut()
            .find(|p| p.key == previous)
            .map(|p| p.respawn(next))
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
