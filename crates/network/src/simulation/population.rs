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

impl World {
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
