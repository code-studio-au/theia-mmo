//! Pure player display-name value, extracted from persistence's existing policy.
//! Names are display/lookup data; they never establish ownership or permissions.
pub const MAX_BYTES: usize = 20;
pub const RULES: &str =
    "Names require 3–20 ASCII letters with single interior apostrophes or hyphens";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(RULES)
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterName(String);
impl CharacterName {
    /// Existing temporary development policy: 3–20 ASCII characters, letter
    /// ends, apostrophe/hyphen inside, and no adjacent punctuation.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let bytes = value.as_bytes();
        if !(3..=MAX_BYTES).contains(&bytes.len())
            || !bytes[0].is_ascii_alphabetic()
            || !bytes[bytes.len() - 1].is_ascii_alphabetic()
            || bytes
                .iter()
                .any(|b| !b.is_ascii_alphabetic() && !matches!(b, b'\'' | b'-'))
            || bytes
                .windows(2)
                .any(|pair| !pair[0].is_ascii_alphabetic() && !pair[1].is_ascii_alphabetic())
        {
            return Err(Error);
        }
        Ok(Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn development_names_bound_input_and_preserve_display_spelling() {
        // Preserve the original persistence parser's accepted/rejected cases.
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
