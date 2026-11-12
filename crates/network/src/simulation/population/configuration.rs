//! Bounded startup configuration shared by authority and client identity setup.
use super::{Binding, authoring};
use crate::{Result, navigation, procedural, simulation::creatures};
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

const MAX_BYTES: u64 = 4096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    zone: String,
    group: u64,
    creature: String,
    navigation: String,
    minimum_spacing_metres: f64,
    access_from: [f64; 3],
}

/// Fully prepared before simulation or network admission; never deserialized.
pub struct Loaded {
    prepared: navigation::spawn_preparation::Prepared,
    binding: Binding,
}
impl Loaded {
    pub fn open(path: &Path, source: &procedural::Source) -> Result<Self> {
        let procedural::Source::Authored(store) = source else {
            return Err("Population configuration requires an authored world".into());
        };
        let path = path.canonicalize()?;
        let root = path
            .parent()
            .ok_or("Population configuration has no directory")?;
        let document: Document =
            serde_json::from_slice(&crate::content::read_bounded(&path, MAX_BYTES)?)?;
        if document.schema != 1 {
            return Err("Unsupported population configuration schema".into());
        }
        let zone = theia_zone::Zone::load(&contained(root, &document.zone)?)?;
        let profile = creatures::Profile::from_json(&crate::content::read_bounded(
            &contained(root, &document.creature)?,
            creatures::MAX_PROFILE_BYTES as u64,
        )?)?;
        // Refuse unresolved behavior/model/count before opening navigation.
        authoring::binding(&zone, document.group, profile.definition().clone())?;
        let nav =
            navigation::Store::open(&contained(root, &document.navigation)?, store.identity())?;
        let (prepared, binding) = authoring::prepare(
            &zone,
            document.group,
            profile.definition().clone(),
            store,
            &nav,
            document.minimum_spacing_metres,
            document.access_from,
        )?;
        Ok(Self { prepared, binding })
    }
    pub fn profile_identity(&self, base: [u8; 32]) -> [u8; 32] {
        self.binding
            .profile_identity(self.prepared.profile_identity(base))
    }
    pub fn respawn_ms(&self) -> u32 {
        self.binding.respawn_ms()
    }
    pub fn count(&self) -> usize {
        self.prepared.points().len()
    }
    pub fn into_parts(self) -> (navigation::spawn_preparation::Prepared, Binding) {
        (self.prepared, self.binding)
    }
}

fn contained(root: &Path, value: &str) -> Result<PathBuf> {
    if value.is_empty()
        || value.len() > 256
        || value.contains('\\')
        || !Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(
            "Population input must be a bounded relative path inside its configuration directory"
                .into(),
        );
    }
    let path = root.join(value).canonicalize()?;
    if !path.starts_with(root) {
        return Err("Population input escapes configuration directory".into());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_paths_reject_escape_and_invalid_forms_before_reading() {
        for path in [
            "",
            "../zone.json",
            "/tmp/zone.json",
            "a/../../zone.json",
            "C:\\zone.json",
        ] {
            assert!(
                contained(Path::new("/nonexistent"), path)
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("bounded relative path")
            );
        }
        assert!(contained(Path::new("/nonexistent"), &"a".repeat(257)).is_err());
    }
    #[test]
    fn procedural_worlds_refuse_before_loading_configuration() {
        assert!(
            Loaded::open(Path::new("/nonexistent"), &procedural::Source::Procedural)
                .err()
                .unwrap()
                .to_string()
                .contains("authored")
        );
    }
}
