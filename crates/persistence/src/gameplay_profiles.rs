//! Exact saved-content bindings checked before issuing an admission ticket.
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Default)]
pub struct GameplayProfiles {
    pub(crate) hashes: [Option<[u8; 32]>; 4],
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
        let hashes = [inventory, progression, objective, lifecycle];
        if hashes.iter().flatten().any(|h| *h == [0; 32])
            || (objective.is_some() && lifecycle.is_some())
        {
            return Err(Error::Invalid("Invalid admission gameplay profiles"));
        }
        Ok(Self { hashes })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileCompatibility {
    Compatible,
    Incompatible,
}
