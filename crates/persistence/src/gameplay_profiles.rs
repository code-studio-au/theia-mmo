//! Exact saved-content bindings checked before issuing an admission ticket.
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Default)]
pub struct GameplayProfiles {
    pub(crate) hashes: [Option<[u8; 32]>; 5],
}
impl GameplayProfiles {
    /// Disabled profiles are ignored; absent saved state is initialized later by
    /// its normal retained owner. Automatic and explicit quests are exclusive.
    pub fn new(
        inventory: Option<[u8; 32]>,
        progression: Option<[u8; 32]>,
        objective: Option<[u8; 32]>,
        lifecycle: Option<[u8; 32]>,
    ) -> Result<Self> {
        let hashes = [inventory, progression, objective, lifecycle, None];
        if hashes.iter().flatten().any(|h| *h == [0; 32])
            || (objective.is_some() && lifecycle.is_some())
        {
            return Err(Error::Invalid("Invalid admission gameplay profiles"));
        }
        Ok(Self { hashes })
    }
    /// Expanded journals are exclusive with the legacy objective/lifecycle modes.
    /// Missing rows remain compatible; their retained owner initializes later.
    pub fn with_quest_journal(mut self, journal: [u8; 32]) -> Result<Self> {
        if journal == [0; 32]
            || self.hashes[1].is_none()
            || self.hashes[2].is_some()
            || self.hashes[3].is_some()
            || self.hashes[4].is_some()
        {
            return Err(Error::Invalid("Invalid expanded journal admission profile"));
        }
        self.hashes[4] = Some(journal);
        Ok(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileCompatibility {
    Compatible,
    Incompatible,
}
