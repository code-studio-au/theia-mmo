//! Explicit once-per-character quest lifecycle proposals. These do not authorize
//! NPC interaction, deduplicate kills, commit progress/rewards or acknowledge a
//! client. The existing auto-start training objective remains a separate mode.
use super::{Error as ObjectiveError, Profile};
use crate::content::ContentId;
use serde::Serialize;

pub const PROFILE_DOMAIN: &[u8] = b"theia-quest-lifecycle-profile-v1\0";

/// `Ready` is derived from active progress, never an independently stored flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Available,
    Active,
    Ready,
    TurnedIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Available,
    Accepted,
    TurnedIn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    profile: ContentId,
    revision: u64,
    status: Status,
    count: u32,
}

impl State {
    pub fn available(profile: &Profile) -> Self {
        Self {
            profile: profile.id().clone(),
            revision: 0,
            status: Status::Available,
            count: 0,
        }
    }

    /// Restore checked logical state only. The persistence/wire envelopes must
    /// separately bind the exact profile bytes and owning character/world.
    /// This first lifecycle has no abandonment, repeat or administrative edits:
    /// acceptance adds one revision, each credited kill one, turn-in one.
    pub fn restore(
        profile: &Profile,
        revision: u64,
        status: Status,
        count: u32,
    ) -> Result<Self, Error> {
        let state = Self {
            profile: profile.id().clone(),
            revision,
            status,
            count,
        };
        state.progress(profile)?;
        Ok(state)
    }

    pub fn profile(&self) -> &ContentId {
        &self.profile
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn progress(&self, profile: &Profile) -> Result<Progress, Error> {
        if &self.profile != profile.id() {
            return Err(Error::ProfileMismatch);
        }
        if self.count > profile.required() {
            return Err(Error::InvalidState);
        }
        let valid = match self.status {
            Status::Available => self.revision == 0 && self.count == 0,
            Status::Accepted => self.revision == u64::from(self.count) + 1,
            Status::TurnedIn => {
                self.count == profile.required() && self.revision == u64::from(self.count) + 2
            }
        };
        if !valid {
            return Err(Error::InvalidState);
        }
        let phase = match self.status {
            Status::Available => Phase::Available,
            Status::Accepted if self.count == profile.required() => Phase::Ready,
            Status::Accepted => Phase::Active,
            Status::TurnedIn => Phase::TurnedIn,
        };
        Ok(Progress {
            revision: self.revision,
            count: self.count,
            required: profile.required(),
            phase,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub revision: u64,
    pub count: u32,
    pub required: u32,
    pub phase: Phase,
}

/// The client may request an action/revision, never supply a phase, count or
/// reward. The runtime must authorize the current NPC/session/range before work
/// enters the bounded durable owner; idempotency belongs to that owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub expected_revision: u64,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Accept,
    TurnIn,
}

impl Command {
    pub fn canonical_bytes(self) -> [u8; 9] {
        let mut bytes = [0; 9];
        bytes[..8].copy_from_slice(&self.expected_revision.to_le_bytes());
        bytes[8] = match self.action {
            Action::Accept => 1,
            Action::TurnIn => 2,
        };
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub state: State,
    pub changed: bool,
    pub became_ready: bool,
    /// An atomic durable turn-in may use this to grant its server-owned recipe.
    /// This is not a claim receipt or evidence that any item/XP was awarded.
    pub turned_in_now: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ProfileMismatch,
    InvalidState,
    StaleRevision,
    AlreadyAccepted,
    NotAccepted,
    NotReady,
    AlreadyTurnedIn,
    Objective(ObjectiveError),
}

pub fn propose_action(
    profile: &Profile,
    state: &State,
    command: Command,
) -> Result<Proposal, Error> {
    let progress = state.progress(profile)?;
    if command.expected_revision != state.revision {
        return Err(Error::StaleRevision);
    }
    let status = match (command.action, progress.phase) {
        (_, Phase::TurnedIn) => return Err(Error::AlreadyTurnedIn),
        (Action::Accept, Phase::Available) => Status::Accepted,
        (Action::Accept, _) => return Err(Error::AlreadyAccepted),
        (Action::TurnIn, Phase::Available) => return Err(Error::NotAccepted),
        (Action::TurnIn, Phase::Active) => return Err(Error::NotReady),
        (Action::TurnIn, Phase::Ready) => Status::TurnedIn,
    };
    let next = State::restore(profile, state.revision + 1, status, state.count)?;
    Ok(Proposal {
        state: next,
        changed: true,
        became_ready: false,
        turned_in_now: command.action == Action::TurnIn,
    })
}

/// Trusted current-life kill only. Storage must deduplicate the victim's receipt
/// before proposing against newer state; this function holds no event ledger.
pub fn propose_kill(
    profile: &Profile,
    state: &State,
    creature: &ContentId,
) -> Result<Proposal, Error> {
    let progress = state.progress(profile)?;
    if progress.phase != Phase::Active || creature != profile.creature() {
        return Ok(Proposal {
            state: state.clone(),
            changed: false,
            became_ready: false,
            turned_in_now: false,
        });
    }
    // Reuse the checked objective bounds/arithmetic; lifecycle acceptance owns
    // its extra revision independently from the retained auto-start fixture.
    let objective = super::State::new(profile, u64::from(state.count), state.count)
        .map_err(Error::Objective)?;
    let next = profile
        .propose_kill(&objective, creature)
        .map_err(Error::Objective)?;
    Ok(Proposal {
        state: State::restore(
            profile,
            state.revision + 1,
            Status::Accepted,
            next.state.count(),
        )?,
        changed: next.advanced,
        became_ready: next.completed_now,
        turned_in_now: false,
    })
}
