//! Bounded authority-only damage threat arithmetic. Callers own exactly-once
//! committed damage, encounter generations and eligibility; no wire input is
//! accepted here. The opt-in World adapter owns those committed-fact guards.
use crate::{content::ContentId, identity::EntityKey, player_life::LifeKey};
use serde::Deserialize;

pub const MAX_PROFILE_BYTES: usize = 4096;
pub const MAX_PARTICIPANTS: usize = 32;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: String,
    maximum_participants: usize,
    melee_transfer_basis_points: u32,
    ranged_transfer_basis_points: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    maximum_participants: usize,
    melee_transfer_basis_points: u32,
    ranged_transfer_basis_points: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Threat policy exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid threat policy")?;
        if raw.schema != 1
            || raw.ruleset != "classic_damage_threat_empirical_v1"
            || !(1..=MAX_PARTICIPANTS).contains(&raw.maximum_participants)
            || raw.melee_transfer_basis_points != 11000
            || raw.ranged_transfer_basis_points != 13000
        {
            return Err("Unsupported threat policy or bounds");
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid threat policy ID")?,
            maximum_participants: raw.maximum_participants,
            melee_transfer_basis_points: raw.melee_transfer_basis_points,
            ranged_transfer_basis_points: raw.ranged_transfer_basis_points,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn maximum_participants(&self) -> usize {
        self.maximum_participants
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidSource,
    DifferentLife,
    Capacity,
    Exhausted,
    InvalidEligibility,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub life: LifeKey,
    pub damage_threat: u64,
}
/// Supplied from current authoritative reach/life/session checks. Being outside
/// melee uses the ranged threshold even when the damage came from a melee skill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eligible {
    pub life: LifeKey,
    pub in_melee: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    profile: Profile,
    creature: EntityKey,
    entries: Vec<Entry>,
    current: Option<LifeKey>,
}
impl Table {
    pub fn new(profile: Profile, creature: EntityKey) -> Self {
        Self {
            entries: Vec::with_capacity(profile.maximum_participants),
            profile,
            creature,
            current: None,
        }
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn current(&self) -> Option<LifeKey> {
        self.current
    }
    fn source(&self, life: LifeKey) -> Result<Option<usize>, Error> {
        if life.actor.epoch != self.creature.epoch || life.actor == self.creature {
            return Err(Error::InvalidSource);
        }
        let index = self.entries.iter().position(|e| e.life.actor == life.actor);
        if index.is_some_and(|i| self.entries[i].life != life) {
            return Err(Error::DifferentLife);
        }
        Ok(index)
    }
    /// Body-pull registration adds zero threat. It does not assign a victim;
    /// selection uses the separately supplied current eligibility list.
    pub fn register(&mut self, life: LifeKey) -> Result<(), Error> {
        if self.source(life)?.is_some() {
            return Ok(());
        }
        if self.entries.len() == self.profile.maximum_participants {
            return Err(Error::Capacity);
        }
        self.entries.push(Entry {
            life,
            damage_threat: 0,
        });
        Ok(())
    }
    /// One threat per point of actual HP lost, after mitigation and overkill.
    /// The caller must supply each committed resolution exactly once. Zero
    /// damage allocates nothing; misses/avoidance do not manufacture threat.
    pub fn add_damage(&mut self, life: LifeKey, health_lost: u32) -> Result<(), Error> {
        let index = self.source(life)?;
        if health_lost == 0 {
            return Ok(());
        }
        if let Some(index) = index {
            let total = self.entries[index]
                .damage_threat
                .checked_add(u64::from(health_lost))
                .ok_or(Error::Exhausted)?;
            self.entries[index].damage_threat = total;
        } else {
            self.register(life)?;
            self.entries.last_mut().unwrap().damage_threat = u64::from(health_lost);
        }
        Ok(())
    }
    /// Exact life removal cannot delete a later incarnation. Death/disconnect,
    /// encounter reset and queued-resolution invalidation belong to the caller.
    pub fn remove(&mut self, life: LifeKey) {
        self.entries.retain(|e| e.life != life);
        if self.current == Some(life) {
            self.current = None;
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current = None;
    }
    /// Strictly exceed the current victim's threshold. Exact equality retains
    /// it. If several challengers qualify, highest threat then lowest stable
    /// life key wins. These equality/tie rules are explicit Theia adaptations.
    /// All eligibility is validated before changing the selected victim.
    pub fn select(&mut self, eligible: &[Eligible]) -> Result<Option<LifeKey>, Error> {
        if eligible.len() > self.profile.maximum_participants {
            return Err(Error::InvalidEligibility);
        }
        for (i, candidate) in eligible.iter().enumerate() {
            if self.source(candidate.life)?.is_none()
                || eligible[..i].iter().any(|e| e.life == candidate.life)
            {
                return Err(Error::InvalidEligibility);
            }
        }
        let current = self
            .current
            .filter(|life| eligible.iter().any(|e| e.life == *life));
        let baseline = current.map(|life| {
            self.entries
                .iter()
                .find(|e| e.life == life)
                .unwrap()
                .damage_threat
        });
        let mut best: Option<Entry> = None;
        for candidate in eligible {
            let entry = *self
                .entries
                .iter()
                .find(|e| e.life == candidate.life)
                .unwrap();
            if current == Some(entry.life) {
                continue;
            }
            let threshold = if candidate.in_melee {
                self.profile.melee_transfer_basis_points
            } else {
                self.profile.ranged_transfer_basis_points
            };
            if baseline.is_some_and(|threat| {
                u128::from(entry.damage_threat) * 10000
                    <= u128::from(threat) * u128::from(threshold)
            }) {
                continue;
            }
            if best.is_none_or(|old| {
                entry.damage_threat > old.damage_threat
                    || (entry.damage_threat == old.damage_threat
                        && order(entry.life) < order(old.life))
            }) {
                best = Some(entry);
            }
        }
        self.current = best.map(|e| e.life).or(current);
        Ok(self.current)
    }
}
fn order(life: LifeKey) -> (u64, u64, u64) {
    (
        life.actor.epoch.value(),
        life.actor.entity.value(),
        life.generation.get(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{EntityId, WorldEpoch};
    use std::num::NonZeroU64;
    #[test]
    fn overflow_refuses_without_partial_mutation_and_threshold_math_does_not_overflow() {
        let profile = Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-creature-threat.json"
        ))
        .unwrap();
        let creature = EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(100).unwrap(),
        };
        let life = |id| LifeKey {
            actor: EntityKey {
                entity: EntityId::new(id).unwrap(),
                ..creature
            },
            generation: NonZeroU64::MIN,
        };
        let mut table = Table::new(profile, creature);
        table.register(life(1)).unwrap();
        table.register(life(2)).unwrap();
        table.entries[0].damage_threat = u64::MAX;
        table.entries[1].damage_threat = u64::MAX;
        let first = [Eligible {
            life: life(1),
            in_melee: true,
        }];
        table.select(&first).unwrap();
        let before = table.clone();
        assert_eq!(table.add_damage(life(1), 1), Err(Error::Exhausted));
        assert_eq!(table, before);
        assert_eq!(
            table.select(&[
                first[0],
                Eligible {
                    life: life(2),
                    in_melee: true
                }
            ]),
            Ok(Some(life(1)))
        );
    }
}
