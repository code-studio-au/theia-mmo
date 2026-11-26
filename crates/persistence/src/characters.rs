//! Initial account-owned character records. Names are display/lookup data, never
//! ownership; the owner must come from an authenticated server session.
use crate::{CharacterId, SavedCharacter};
pub use theia_core::character_name::CharacterName;
pub const MAX_CHARACTERS_PER_OWNER: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: CharacterName,
    pub appearance: theia_core::appearance::Appearance,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebsiteCharacterSummary {
    pub name: CharacterName,
    pub progression: Option<theia_core::progression::Progress>,
    pub last_saved_unix: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub saved: SavedCharacter,
    pub name: CharacterName,
    pub appearance: theia_core::appearance::Appearance,
}
