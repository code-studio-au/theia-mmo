//! Authority-only, bounded ordinary-creature ownership prerequisite. These
//! facts grant no XP, quest or loot entitlement until their separate policies
//! and durable adapters accept them. No client request belongs at this boundary.
use crate::{content::ContentId, identity::EntityKey, player_life::LifeKey};
use serde::Deserialize;
use std::num::NonZeroU64;

pub const MAX_PROFILE_BYTES: usize = 4096;
pub const MAX_CONTRIBUTORS: usize = 32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: String,
    maximum_contributors: usize,
    #[serde(default, deserialize_with = "recipient_field")]
    recipient: Option<RecipientPolicy>,
}
fn recipient_field<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<RecipientPolicy>, D::Error> {
    RecipientPolicy::deserialize(deserializer).map(Some)
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipientPolicy {
    SoloTagOwner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    maximum_contributors: usize,
    recipient: Option<RecipientPolicy>,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Creature credit profile exceeds byte budget");
        }
        let raw: Raw =
            serde_json::from_slice(bytes).map_err(|_| "Invalid creature credit profile")?;
        let supported = match raw.schema {
            1 => raw.ruleset == "creature_credit_fixture_v1" && raw.recipient.is_none(),
            2 => {
                raw.ruleset == "solo_tag_credit_fixture_v1"
                    && raw.recipient == Some(RecipientPolicy::SoloTagOwner)
            }
            _ => false,
        };
        if !supported || !(1..=MAX_CONTRIBUTORS).contains(&raw.maximum_contributors) {
            return Err("Unsupported creature credit policy or bounds");
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid creature credit profile ID")?,
            maximum_contributors: raw.maximum_contributors,
            recipient: raw.recipient,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn recipient(&self) -> Option<RecipientPolicy> {
        self.recipient
    }
    pub fn maximum_contributors(&self) -> usize {
        self.maximum_contributors
    }
}
/// A fresh NPC key is required on respawn. Encounter generation also protects
/// the same NPC lifetime from delayed facts after evade or owner abandonment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncounterKey {
    pub creature: EntityKey,
    pub generation: NonZeroU64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    WhiteMelee,
    AbilityDamage,
}
impl Category {
    fn index(self) -> usize {
        match self {
            Self::WhiteMelee => 0,
            Self::AbilityDamage => 1,
        }
    }
}
/// Only immutable, resolved authoritative damage may be converted to this
/// input. Health lost excludes overkill and prevented damage. Sequence belongs
/// to the source life and category, not the client command number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageFact {
    pub encounter: EncounterKey,
    pub source: LifeKey,
    pub category: Category,
    pub sequence: NonZeroU64,
    pub health_lost: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    StaleEncounter,
    InvalidSource,
    StaleTick,
    StaleLife,
    Replay,
    Capacity,
    Exhausted,
    Resolved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contribution {
    pub actor: EntityKey,
    pub health_lost: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Participant {
    contribution: Contribution,
    life: NonZeroU64,
    sequences: [u64; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KillCredit {
    pub encounter: EncounterKey,
    /// Session identity; durable character mapping remains outside simulation.
    /// Death alone does not transfer this claim to the final hitter.
    pub owner: Option<EntityKey>,
    pub contributions: Vec<Contribution>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ledger {
    profile: Profile,
    encounter: EncounterKey,
    observed: Option<u64>,
    owner: Option<EntityKey>,
    participants: Vec<Participant>,
    resolved: bool,
}
impl Ledger {
    pub fn new(profile: Profile, creature: EntityKey) -> Self {
        Self {
            participants: Vec::with_capacity(profile.maximum_contributors),
            profile,
            encounter: EncounterKey {
                creature,
                generation: NonZeroU64::MIN,
            },
            observed: None,
            owner: None,
            resolved: false,
        }
    }
    pub fn encounter(&self) -> EncounterKey {
        self.encounter
    }
    pub fn owner(&self) -> Option<EntityKey> {
        self.owner
    }
    pub fn contributions(&self) -> impl Iterator<Item = Contribution> + '_ {
        self.participants.iter().map(|p| p.contribution)
    }
    fn check(&self, now: u64, encounter: EncounterKey) -> Result<(), Error> {
        if encounter != self.encounter {
            return Err(Error::StaleEncounter);
        }
        if self.observed.is_some_and(|last| now < last) {
            return Err(Error::StaleTick);
        }
        if self.resolved {
            return Err(Error::Resolved);
        }
        Ok(())
    }
    /// All errors leave the ledger unchanged. Future World integration must
    /// preflight a candidate before committing health/RNG or outgoing facts.
    /// Returns false for zero effective damage, which never allocates a slot.
    pub fn record(&mut self, now: u64, fact: DamageFact) -> Result<bool, Error> {
        self.check(now, fact.encounter)?;
        if fact.source.actor.epoch != self.encounter.creature.epoch
            || fact.source.actor == self.encounter.creature
        {
            return Err(Error::InvalidSource);
        }
        let index = self
            .participants
            .iter()
            .position(|p| p.contribution.actor == fact.source.actor);
        if let Some(i) = index {
            let p = &self.participants[i];
            if fact.source.generation < p.life {
                return Err(Error::StaleLife);
            }
            if fact.source.generation == p.life
                && fact.sequence.get() <= p.sequences[fact.category.index()]
            {
                return Err(Error::Replay);
            }
        }
        if fact.health_lost == 0 {
            return Ok(false);
        }
        let total = if let Some(i) = index {
            self.participants[i]
                .contribution
                .health_lost
                .checked_add(u64::from(fact.health_lost))
                .ok_or(Error::Exhausted)?
        } else {
            if self.participants.len() == self.profile.maximum_contributors {
                return Err(Error::Capacity);
            }
            u64::from(fact.health_lost)
        };
        let participant = if let Some(i) = index {
            &mut self.participants[i]
        } else {
            self.participants.push(Participant {
                contribution: Contribution {
                    actor: fact.source.actor,
                    health_lost: 0,
                },
                life: fact.source.generation,
                sequences: [0; 2],
            });
            self.participants
                .last_mut()
                .expect("Inserted bounded participant")
        };
        if participant.life != fact.source.generation {
            participant.life = fact.source.generation;
            participant.sequences = [0; 2];
        }
        participant.sequences[fact.category.index()] = fact.sequence.get();
        participant.contribution.health_lost = total;
        self.owner.get_or_insert(fact.source.actor);
        self.observed = Some(now);
        Ok(true)
    }
    /// Authority-confirmed evade or tag-owner departure abandons this encounter.
    /// Incrementing first makes exhaustion a refusal without partial clearing.
    /// Connected player death is not an abandonment signal.
    pub fn reset(&mut self, now: u64, encounter: EncounterKey) -> Result<EncounterKey, Error> {
        self.check(now, encounter)?;
        let generation = self
            .encounter
            .generation
            .checked_add(1)
            .ok_or(Error::Exhausted)?;
        self.encounter.generation = generation;
        self.owner = None;
        self.participants.clear();
        self.observed = Some(now);
        Ok(self.encounter)
    }
    /// Call only after an authoritative creature death, once. This is a trusted
    /// ownership/contribution fact, not an economy success acknowledgement.
    pub fn finish(&mut self, now: u64, encounter: EncounterKey) -> Result<KillCredit, Error> {
        self.check(now, encounter)?;
        let credit = KillCredit {
            encounter,
            owner: self.owner,
            contributions: self.contributions().collect(),
        };
        self.resolved = true;
        self.observed = Some(now);
        Ok(credit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{EntityId, WorldEpoch};

    #[test]
    fn exhausted_damage_and_encounter_counters_refuse_atomically() {
        let creature = EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(1).unwrap(),
        };
        let source = LifeKey {
            actor: EntityKey {
                entity: EntityId::new(2).unwrap(),
                ..creature
            },
            generation: NonZeroU64::MIN,
        };
        let profile = Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-creature-credit.json"
        ))
        .unwrap();
        let mut ledger = Ledger::new(profile, creature);
        let fact = DamageFact {
            encounter: ledger.encounter(),
            source,
            category: Category::WhiteMelee,
            sequence: NonZeroU64::MIN,
            health_lost: 1,
        };
        ledger.record(1, fact).unwrap();
        ledger.participants[0].contribution.health_lost = u64::MAX;
        let before = ledger.clone();
        assert_eq!(
            ledger.record(
                2,
                DamageFact {
                    sequence: NonZeroU64::new(2).unwrap(),
                    ..fact
                }
            ),
            Err(Error::Exhausted)
        );
        assert_eq!(ledger, before);
        ledger.encounter.generation = NonZeroU64::MAX;
        let before = ledger.clone();
        assert_eq!(ledger.reset(2, ledger.encounter()), Err(Error::Exhausted));
        assert_eq!(ledger, before);
    }
}
