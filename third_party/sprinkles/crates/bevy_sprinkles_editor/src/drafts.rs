//! Storage foundation for recoverable particle drafts. Not yet connected to the UI.
//!
//! Drafts are separate from project files. A caller owns an ID for a document
//! session and may replace only that session's draft; other drafts are retained.
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_DRAFT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DRAFTS: usize = 128;
const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draft {
    version: u32,
    /// The original file is informational; recovery must never overwrite it.
    pub original_path: Option<PathBuf>,
    /// A serialized Sprinkles asset, validated when recovery is requested.
    pub snapshot: String,
}

impl Draft {
    pub fn new(original_path: Option<PathBuf>, snapshot: String) -> Self {
        Self {
            version: VERSION,
            original_path,
            snapshot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftId(String);

impl DraftId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated snapshot. The source path is informational; restoring must
/// open the asset as an unsaved project until the user chooses Save As.
pub struct RecoveredDraft {
    pub original_path: Option<PathBuf>,
    pub asset: bevy_sprinkles::prelude::ParticlesAsset,
}

#[derive(Debug)]
pub enum RecoveryError {
    Storage(io::Error),
    InvalidProject,
    UnknownVersion,
    UnsupportedMaterial,
}

#[derive(Debug, Clone)]
pub struct DraftStore {
    directory: PathBuf,
}

impl DraftStore {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    /// Reserve a fresh draft without replacing any existing draft.
    pub fn create(&self, draft: &Draft) -> io::Result<DraftId> {
        std::fs::create_dir_all(&self.directory)?;
        if self.list()?.len() >= MAX_DRAFTS {
            return Err(io::Error::other(
                "Draft storage is full; existing drafts were preserved",
            ));
        }
        let bytes = encode(draft)?;
        static NONCE: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let id = DraftId(format!(
                "{time:x}-{:x}-{:x}",
                std::process::id(),
                NONCE.fetch_add(1, Ordering::Relaxed)
            ));
            match crate::project::atomic_write(&self.path(&id), &bytes, true) {
                Ok(()) => return Ok(id),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Cannot allocate draft ID",
        ))
    }

    /// Replace a known draft. A missing draft is an error rather than a new file.
    pub fn update(&self, id: &DraftId, draft: &Draft) -> io::Result<()> {
        let path = self.path(id);
        if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Draft is not a regular file",
            ));
        }
        crate::project::atomic_write(&path, &encode(draft)?, false)
    }

    /// List IDs, including corrupt drafts, so they are never silently discarded.
    pub fn list(&self) -> io::Result<Vec<DraftId>> {
        let entries = match std::fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name
                .to_str()
                .and_then(|name| name.strip_suffix(".draft.ron"))
            else {
                continue;
            };
            if !name.is_empty() && name.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-') {
                ids.push(DraftId(name.to_owned()));
            }
        }
        ids.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(ids)
    }

    pub fn load(&self, id: &DraftId) -> io::Result<Draft> {
        let path = self.path(id);
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_DRAFT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid draft file",
            ));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(MAX_DRAFT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_DRAFT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Draft is too large",
            ));
        }
        let draft: Draft = ron::de::from_bytes(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if draft.version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported draft version",
            ));
        }
        Ok(draft)
    }

    /// Validate the complete asset before callers replace the current project.
    pub fn recover(&self, id: &DraftId) -> Result<RecoveredDraft, RecoveryError> {
        let draft = self.load(id).map_err(RecoveryError::Storage)?;
        let result =
            crate::project::parse_project(&draft.snapshot).map_err(|error| match error {
                crate::project::LoadProjectError::UnknownVersion => RecoveryError::UnknownVersion,
                crate::project::LoadProjectError::UnsupportedMaterial => {
                    RecoveryError::UnsupportedMaterial
                }
                crate::project::LoadProjectError::Parse
                | crate::project::LoadProjectError::Read(_) => RecoveryError::InvalidProject,
            })?;
        Ok(RecoveredDraft {
            original_path: draft.original_path,
            asset: result.asset,
        })
    }

    fn path(&self, id: &DraftId) -> PathBuf {
        self.directory.join(format!("{}.draft.ron", id.0))
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

fn encode(draft: &Draft) -> io::Result<Vec<u8>> {
    let bytes = ron::ser::to_string(draft)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .into_bytes();
    if bytes.len() as u64 > MAX_DRAFT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Draft is too large",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> DraftStore {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        DraftStore::new(std::env::temp_dir().join(format!(
            "theia-draft-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }

    #[test]
    fn drafts_remain_separate_and_updates_never_touch_originals() {
        let store = store();
        std::fs::create_dir_all(store.directory()).unwrap();
        let original = store.directory().join("project.ron");
        std::fs::write(&original, b"original project").unwrap();
        let first = Draft::new(Some(original.clone()), "first".into());
        let second = Draft::new(None, "untitled".into());
        let first_id = store.create(&first).unwrap();
        let second_id = store.create(&second).unwrap();
        assert_ne!(first_id, second_id);
        assert_eq!(store.list().unwrap().len(), 2);
        store
            .update(
                &first_id,
                &Draft::new(Some(original.clone()), "updated".into()),
            )
            .unwrap();
        assert_eq!(store.load(&first_id).unwrap().snapshot, "updated");
        assert_eq!(store.load(&second_id).unwrap(), second);
        assert_eq!(std::fs::read(original).unwrap(), b"original project");
        std::fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn corrupt_and_oversized_drafts_are_reported_and_preserved() {
        let store = store();
        let id = store.create(&Draft::new(None, "good".into())).unwrap();
        std::fs::write(store.path(&id), b"corrupt").unwrap();
        assert_eq!(
            store.load(&id).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(store.list().unwrap(), vec![id.clone()]);
        let huge = Draft::new(None, "x".repeat(MAX_DRAFT_BYTES as usize));
        assert!(store.update(&id, &huge).is_err());
        assert_eq!(std::fs::read(store.path(&id)).unwrap(), b"corrupt");
        std::fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn recovery_validates_project_before_returning_an_unsaved_asset() {
        use bevy::prelude::default;
        use bevy_sprinkles::prelude::*;
        let store = store();
        let original = store.directory().join("original.ron");
        std::fs::create_dir_all(store.directory()).unwrap();
        std::fs::write(&original, b"original").unwrap();
        let mut asset = ParticlesAsset::new(
            "Recovered".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let id = store
            .create(&Draft::new(
                Some(original.clone()),
                ron::to_string(&asset).unwrap(),
            ))
            .unwrap();
        let recovered = store.recover(&id).unwrap();
        assert_eq!(recovered.asset.name, "Recovered");
        assert_eq!(recovered.original_path.as_deref(), Some(original.as_path()));
        asset.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
            vertex_shader: None,
            fragment_shader: None,
        };
        store
            .update(&id, &Draft::new(None, ron::to_string(&asset).unwrap()))
            .unwrap();
        assert!(matches!(
            store.recover(&id),
            Err(RecoveryError::UnsupportedMaterial)
        ));
        store
            .update(&id, &Draft::new(None, "broken project".into()))
            .unwrap();
        assert!(matches!(
            store.recover(&id),
            Err(RecoveryError::InvalidProject)
        ));
        assert_eq!(std::fs::read(original).unwrap(), b"original");
        assert_eq!(store.list().unwrap(), vec![id]);
        std::fs::remove_dir_all(store.directory()).unwrap();
    }

    #[test]
    fn full_storage_and_missing_drafts_never_replace_existing_data() {
        let store = store();
        std::fs::create_dir_all(store.directory()).unwrap();
        let draft = Draft::new(None, "data".into());
        for index in 0..MAX_DRAFTS {
            std::fs::write(
                store.directory().join(format!("{index:x}.draft.ron")),
                b"keep",
            )
            .unwrap();
        }
        assert!(store.create(&draft).is_err());
        assert_eq!(store.list().unwrap().len(), MAX_DRAFTS);
        assert!(store.update(&DraftId("ffff".into()), &draft).is_err());
        assert_eq!(
            std::fs::read(store.directory().join("0.draft.ron")).unwrap(),
            b"keep"
        );
        std::fs::remove_dir_all(store.directory()).unwrap();
    }
}
