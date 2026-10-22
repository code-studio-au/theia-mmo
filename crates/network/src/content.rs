//! Versioned bounded world packages. No renderer or asset-server dependency.
use crate::shared_images::{self, SharedImage};
use crate::{Result, terrain::sha256};
use ring::digest::{Context, SHA256};
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};
use theia_core::content::ContentId;

pub const MANIFEST_FILE: &str = "world.manifest.json";
pub const MAX_COLLISION_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_VISUAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const DOMAIN: &[u8] = b"theia-world-package-v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    content_id: String,
    revision: u32,
    collision_schema: u32,
    collision_bytes: u64,
    collision_sha256: [u8; 32],
    visual_bytes: u64,
    visual_sha256: [u8; 32],
    #[serde(default)]
    shared_images: Vec<SharedImage>,
}
/// Constructed only after schema, logical identity and resource budgets validate.
pub struct Manifest {
    document: Document,
    id: ContentId,
}
impl Manifest {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err("World manifest exceeds byte budget".into());
        }
        let mut document: Document = serde_json::from_slice(bytes)?;
        let id = ContentId::new(&document.content_id).ok_or("Invalid stable content ID")?;
        if !matches!(document.schema, 1..=3)
            || (document.schema == 1 && (bytes.len() > 8192 || !document.shared_images.is_empty()))
            || (document.schema == 3 && document.shared_images.len() < 2)
            || document.revision == 0
            || !matches!(document.collision_schema, 1..=4)
            || !(1..=MAX_COLLISION_BYTES).contains(&document.collision_bytes)
            || !(1..=MAX_VISUAL_BYTES).contains(&document.visual_bytes)
        {
            return Err("Unsupported or out-of-budget world manifest".into());
        }
        shared_images::canonicalize(&mut document.shared_images)?;
        Ok(Self { document, id })
    }
    pub fn load_optional(path: &Path) -> Result<Option<Self>> {
        match read_bounded(path, MAX_MANIFEST_BYTES) {
            Ok(bytes) => Ok(Some(Self::decode(&bytes)?)),
            Err(error)
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    pub fn content_id(&self) -> &ContentId {
        &self.id
    }
    pub fn requires_material_profiles(&self) -> bool {
        self.document.schema == 3
    }
    pub fn revision(&self) -> u32 {
        self.document.revision
    }
    /// Independent of JSON formatting and absolute paths. See the byte contract
    /// in CONTENT_PACKAGES.md; Python's offline publisher uses the same layout.
    pub fn identity(&self) -> [u8; 32] {
        let d = &self.document;
        let id = self.id.as_str().as_bytes();
        let mut context = Context::new(&SHA256);
        for bytes in [
            if d.schema == 1 {
                DOMAIN
            } else if d.schema == 2 {
                b"theia-world-package-v2\0"
            } else {
                b"theia-world-package-v3\0"
            },
            &(id.len() as u16).to_le_bytes(),
            id,
            &d.revision.to_le_bytes(),
            &d.collision_schema.to_le_bytes(),
            &d.collision_bytes.to_le_bytes(),
            &d.collision_sha256,
            &d.visual_bytes.to_le_bytes(),
            &d.visual_sha256,
        ] {
            context.update(bytes);
        }
        if d.schema >= 2 {
            context.update(&(d.shared_images.len() as u16).to_le_bytes());
            for image in &d.shared_images {
                context.update(&image.sha256);
                context.update(&image.bytes.to_le_bytes());
                context.update(&[image.color_space as u8]);
                for value in image.sampler {
                    context.update(&value.to_le_bytes());
                }
            }
        }
        context.finish().as_ref().try_into().unwrap()
    }
    pub fn verify_collision(&self, bytes: &[u8]) -> Result<()> {
        if bytes.len() as u64 != self.document.collision_bytes
            || sha256(bytes) != self.document.collision_sha256
        {
            return Err("Collision bytes do not match world manifest".into());
        }
        Ok(())
    }
    pub fn verify_profile(&self, schema: u32, visual_sha256: [u8; 32]) -> Result<()> {
        if schema != self.document.collision_schema || visual_sha256 != self.document.visual_sha256
        {
            return Err("Collision schema/visual binding disagrees with world manifest".into());
        }
        Ok(())
    }
    pub fn verify_visual(&self, path: &Path) -> Result<()> {
        self.verify_visual_with_root(path, None)
    }
    pub fn verify_visual_in(&self, path: &Path, asset_root: &Path) -> Result<()> {
        self.verify_visual_with_root(path, Some(asset_root))
    }
    pub fn shared_images(&self) -> &[SharedImage] {
        &self.document.shared_images
    }
    fn verify_visual_with_root(&self, path: &Path, root: Option<&Path>) -> Result<()> {
        verify_visual(
            path,
            self.document.visual_sha256,
            Some(self.document.visual_bytes),
        )?;
        shared_images::verify(
            path,
            root,
            &self.document.shared_images,
            self.document.schema,
        )
    }
}

pub(crate) fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Content file exceeds byte budget".into());
    }
    Ok(bytes)
}
/// Stream hashes rather than allocating a second copy of a large visual GLB.
pub(crate) fn verify_visual(path: &Path, expected: [u8; 32], length: Option<u64>) -> Result<()> {
    verify_hash(path, expected, length, MAX_VISUAL_BYTES)
}
pub(crate) fn verify_hash(
    path: &Path,
    expected: [u8; 32],
    length: Option<u64>,
    limit: u64,
) -> Result<()> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    if size > limit || length.is_some_and(|n| n != size) {
        return Err("Visual size does not match content budget/manifest".into());
    }
    let mut file = file.take(limit + 1);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 64 * 1024];
    let mut count = 0u64;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        context.update(&buffer[..n]);
    }
    if count != size || context.finish().as_ref() != expected {
        return Err("Terrain visual does not match collision profile/package".into());
    }
    Ok(())
}
