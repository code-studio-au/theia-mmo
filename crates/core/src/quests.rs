//! One bounded development kill objective. Pure proposals establish neither
//! credit, receipt deduplication, character ownership nor durable completion.
use crate::content::ContentId;
use serde::{Deserialize, Serialize};
pub const MAX_PROFILE_BYTES: usize = 4096;
pub const MAX_REQUIRED_KILLS: u32 = 100;
pub const MAX_REVISION: u64 = i64::MAX as u64;
pub const PROFILE_DOMAIN: &[u8] = b"theia-quest-objective-profile-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    name: String,
    objective: RawObjective,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObjective {
    id: String,
    kind: String,
    creature: String,
    required: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    name: String,
    objective: ContentId,
    creature: ContentId,
    required: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Quest profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid quest profile")?;
        if raw.schema != 1
            || raw.name.is_empty()
            || raw.name.len() > 64
            || raw.name.trim() != raw.name
            || raw.name.chars().any(char::is_control)
            || raw.objective.kind != "kill"
            || !(1..=MAX_REQUIRED_KILLS).contains(&raw.objective.required)
        {
            return Err("Invalid quest objective bounds");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid quest ID")?;
        let objective = ContentId::new(&raw.objective.id).ok_or("Invalid objective ID")?;
        let creature =
            ContentId::new(&raw.objective.creature).ok_or("Invalid quest creature ID")?;
        if id == objective || id == creature || objective == creature {
            return Err("Quest content IDs alias different definitions");
        }
        Ok(Self {
            id,
            name: raw.name,
            objective,
            creature,
            required: raw.objective.required,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn objective(&self) -> &ContentId {
        &self.objective
    }
    pub fn creature(&self) -> &ContentId {
        &self.creature
    }
    pub fn required(&self) -> u32 {
        self.required
    }
    pub fn progress(&self, state: &State) -> Result<Progress, Error> {
        if state.profile != self.id {
            return Err(Error::ProfileMismatch);
        }
        if state.revision > MAX_REVISION || state.count > self.required {
            return Err(Error::InvalidState);
        }
        Ok(Progress {
            count: state.count,
            required: self.required,
            complete: state.count == self.required,
        })
    }
    /// Trusted proposal only; the durable owner must reject an already credited
    /// victim life before applying this to newer progress. Nonmatching kills and
    /// a completed objective do not mutate state or require a positive receipt.
    pub fn propose_kill(&self, state: &State, creature: &ContentId) -> Result<Proposal, Error> {
        let before = self.progress(state)?;
        if creature != &self.creature || before.complete {
            return Ok(Proposal {
                state: state.clone(),
                advanced: false,
                completed_now: false,
            });
        }
        let revision = state
            .revision
            .checked_add(1)
            .filter(|n| *n <= MAX_REVISION)
            .ok_or(Error::RevisionExhausted)?;
        let count = state.count.checked_add(1).ok_or(Error::InvalidState)?;
        let state = State::new(self, revision, count)?;
        Ok(Proposal {
            completed_now: count == self.required,
            state,
            advanced: true,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    profile: ContentId,
    revision: u64,
    count: u32,
}
impl State {
    /// Logical identity/bounds only. Storage and wire separately bind the exact
    /// immutable profile digest; callers cannot silently swap objective content.
    pub fn new(profile: &Profile, revision: u64, count: u32) -> Result<Self, Error> {
        let state = Self {
            profile: profile.id.clone(),
            revision,
            count,
        };
        profile.progress(&state)?;
        Ok(state)
    }
    pub fn profile(&self) -> &ContentId {
        &self.profile
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub count: u32,
    pub required: u32,
    pub complete: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub state: State,
    pub advanced: bool,
    pub completed_now: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ProfileMismatch,
    InvalidState,
    RevisionExhausted,
}
