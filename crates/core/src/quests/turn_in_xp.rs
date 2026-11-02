//! Checked XP-only completion recipe and pure combined proposals. Entitlement,
//! exact-byte binding, replay receipts and atomic SQL adoption are separate.
use super::{Profile as Quest, lifecycle};
use crate::{content::ContentId, progression};
use serde::Deserialize;

pub const MAX_PROFILE_BYTES: usize = 4096;
pub const PROFILE_DOMAIN: &[u8] = b"theia-quest-turn-in-xp-profile-v1\0";
/// Exact binding: this domain, journal-only lifecycle digest, recipe digest,
/// progression digest. Each digest is exactly 32 bytes, in that order.
pub const LIFECYCLE_PROFILE_DOMAIN: &[u8] = b"theia-quest-lifecycle-with-xp-profile-v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    quest: String,
    progression: String,
    experience: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    id: ContentId,
    quest: ContentId,
    progression: ContentId,
    experience: u32,
}

impl Recipe {
    pub fn from_json(
        bytes: &[u8],
        quest: &Quest,
        xp: &progression::Profile,
    ) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Quest XP recipe exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid quest XP recipe")?;
        if raw.schema != 1 || !(1..=progression::MAX_GRANT_EXPERIENCE).contains(&raw.experience) {
            return Err("Invalid quest XP recipe schema/amount");
        }
        let recipe = Self {
            id: ContentId::new(&raw.id).ok_or("Invalid quest XP recipe ID")?,
            quest: ContentId::new(&raw.quest).ok_or("Invalid quest XP quest ID")?,
            progression: ContentId::new(&raw.progression)
                .ok_or("Invalid quest XP progression ID")?,
            experience: raw.experience,
        };
        recipe
            .check(quest, xp)
            .map_err(|_| "Quest XP recipe profile mismatch")?;
        Ok(recipe)
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn quest(&self) -> &ContentId {
        &self.quest
    }
    pub fn progression(&self) -> &ContentId {
        &self.progression
    }
    pub fn experience(&self) -> u32 {
        self.experience
    }
    fn check(&self, quest: &Quest, xp: &progression::Profile) -> Result<(), Error> {
        if &self.quest != quest.id()
            || &self.progression != xp.id()
            || xp.kill_experience(quest.creature()).is_none()
        {
            return Err(Error::RecipeMismatch);
        }
        Ok(())
    }
    /// No input is mutated on success or error. A returned proposal grants
    /// nothing until state, XP and one action receipt COMMIT together. Replays
    /// must recover that receipt rather than re-evaluating against fresh state.
    pub fn propose(
        &self,
        quest: &Quest,
        state: &lifecycle::State,
        xp: &progression::Profile,
        experience: &progression::State,
        command: lifecycle::Command,
    ) -> Result<Proposal, Error> {
        self.check(quest, xp)?;
        if command.action != lifecycle::Action::TurnIn {
            return Err(Error::NotTurnIn);
        }
        let quest = lifecycle::propose_action(quest, state, command).map_err(Error::Quest)?;
        let experience = xp
            .propose_grant(experience, self.experience)
            .map_err(Error::Progression)?;
        Ok(Proposal { quest, experience })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub quest: lifecycle::Proposal,
    pub experience: progression::Proposal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    RecipeMismatch,
    NotTurnIn,
    Quest(lifecycle::Error),
    Progression(progression::Error),
}
