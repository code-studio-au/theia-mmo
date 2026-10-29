//! Initial account-owned character records. Names are display/lookup data, never
//! ownership; the owner must come from an authenticated server session.
use crate::{CharacterId, Error, Result, SavedCharacter};
pub const MAX_CHARACTERS_PER_OWNER: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterName(String);
impl CharacterName {
    /// Temporary development policy: 3–20 ASCII characters, letter ends,
    /// apostrophe/hyphen inside, and no adjacent punctuation.
    pub fn parse(value: &str) -> Result<Self> {
        let bytes = value.as_bytes();
        if !(3..=20).contains(&bytes.len())
            || !bytes[0].is_ascii_alphabetic()
            || !bytes[bytes.len() - 1].is_ascii_alphabetic()
            || bytes
                .iter()
                .any(|b| !b.is_ascii_alphabetic() && !matches!(b, b'\'' | b'-'))
            || bytes
                .windows(2)
                .any(|pair| !pair[0].is_ascii_alphabetic() && !pair[1].is_ascii_alphabetic())
        {
            return Err(Error::Invalid(
                "Names require 3–20 ASCII letters with single interior apostrophes or hyphens",
            ));
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn development_names_bound_input_and_preserve_display_spelling() {
        for name in [
            "Arden",
            "Ar'den",
            "Ar-den",
            "Abcdefghijklmnopqrst",
            "A-b",
            "Ab-C'd",
        ] {
            assert_eq!(CharacterName::parse(name).unwrap().as_str(), name);
        }
        for name in [
            "",
            "Ab",
            "-Arden",
            "Arden-",
            "Ar--den",
            "Ar'-den",
            "Arden Smith",
            "Arden2",
            "Árden",
            "Ar\nden",
        ] {
            assert!(CharacterName::parse(name).is_err());
        }
        assert!(CharacterName::parse(&"x".repeat(100000)).is_err());
    }
}
