//! Initial account-owned character records. Names are display/lookup data, never
//! ownership; the owner must come from an authenticated server session.
use crate::{CharacterId, SavedCharacter};
pub use theia_core::character_name::CharacterName;
pub const MAX_CHARACTERS_PER_OWNER: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: CharacterName,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub saved: SavedCharacter,
    pub name: CharacterName,
}
