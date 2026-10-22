//! Stable logical names are separate from byte digests and world incarnations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentId(String);
impl ContentId {
    pub fn new(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        (!bytes.is_empty()
            && bytes.len() <= 96
            && bytes[0].is_ascii_lowercase()
            && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
            && bytes
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(b))
            && !value.contains(".."))
        .then(|| Self(value.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_names_are_bounded_and_not_paths_or_export_locations() {
        assert!(ContentId::new("theia.development.lordaeron-town").is_some());
        for invalid in [
            "",
            "../world",
            "C:\\world",
            "World",
            "world/terrain",
            "world..tile",
            "world.",
            "é",
        ] {
            assert!(ContentId::new(invalid).is_none(), "{invalid}");
        }
        assert!(ContentId::new(&"a".repeat(97)).is_none());
    }
}
