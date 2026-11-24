//! In-memory portable PNG variants for particle export drafts; never publishes files.
use bevy_sprinkles::TextureRef;
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};
use theia_network::{
    particle_content::{MAX_TEXTURE_BYTES, MAX_TEXTURE_RGBA_BYTES, MAX_TEXTURES, Texture},
    shared_images::{self, ColorSpace},
    terrain::sha256,
};

type Key = ([u8; 32], ColorSpace);
#[derive(Default)]
pub(super) struct Textures {
    variants: BTreeMap<Key, Vec<u8>>,
    sources: BTreeMap<(String, ColorSpace), Key>,
    bytes: u64,
    rgba_bytes: u64,
}
impl Textures {
    pub fn rewrite(
        &mut self,
        texture: &mut TextureRef,
        folders: &[String],
        color: ColorSpace,
    ) -> Result<(), String> {
        if matches!(texture, TextureRef::Preset(_)) {
            return Ok(());
        }
        let source = texture.resolve_path(folders);
        if source.is_empty() || !Path::new(&source).is_file() {
            return Err("Particle texture file is missing".into());
        }
        let source_key = (source.clone(), color);
        let key = if let Some(key) = self.sources.get(&source_key) {
            *key
        } else {
            let mut bytes = Vec::new();
            File::open(&source)
                .map_err(|e| e.to_string())?
                .take(shared_images::MAX_IMAGE_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() as u64 > shared_images::MAX_IMAGE_BYTES || bytes.len() < 33 {
                return Err(
                    "Particle texture exceeds size limit or has an incomplete PNG header".into(),
                );
            }
            let header: &[u8; 33] = bytes[..33].try_into().unwrap();
            let rgba = shared_images::png_rgba_bytes(header).map_err(|e| e.to_string())?;
            let key = (sha256(&bytes), color);
            if !self.variants.contains_key(&key) {
                let total = self
                    .bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or("Particle texture byte overflow")?;
                let rgba_total = self
                    .rgba_bytes
                    .checked_add(rgba)
                    .ok_or("Particle texture RGBA overflow")?;
                if self.variants.len() >= MAX_TEXTURES
                    || total > MAX_TEXTURE_BYTES
                    || rgba_total > MAX_TEXTURE_RGBA_BYTES
                {
                    return Err("Particle texture budget exceeded".into());
                }
                self.bytes = total;
                self.rgba_bytes = rgba_total;
                self.variants.insert(key, bytes);
            }
            self.sources.insert(source_key, key);
            key
        };
        let name = Texture {
            sha256: key.0,
            bytes: self.variants[&key].len() as u64,
            color_space: key.1,
        }
        .filename();
        *texture = TextureRef::Asset(format!("textures/{name}"));
        Ok(())
    }
    pub fn into_files(self) -> Vec<(String, Vec<u8>)> {
        self.variants
            .into_iter()
            .map(|((hash, color_space), bytes)| {
                let filename = Texture {
                    sha256: hash,
                    bytes: bytes.len() as u64,
                    color_space,
                }
                .filename();
                (filename, bytes)
            })
            .collect()
    }
    pub fn descriptors(&self) -> Vec<serde_json::Value> {
        self.variants.iter().map(|((hash, color), bytes)| serde_json::json!({
            "sha256":hash,"bytes":bytes.len(),"color_space": match color { ColorSpace::Srgb => "srgb", ColorSpace::Linear => "linear" }
        })).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    #[test]
    fn relative_asset_resolution_deduplicates_and_rejects_non_png() {
        let path = std::env::temp_dir().join(format!(
            "theia-vfx-texture-draft-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let source = path.join("particle.png");
        fs::write(
            &source,
            include_bytes!("../../../crates/network/tests/fixtures/shared-image.png"),
        )
        .unwrap();
        let folder = format!("{}/", path.display());
        let mut textures = Textures::default();
        let mut first = TextureRef::Asset("particle.png".into());
        textures
            .rewrite(&mut first, std::slice::from_ref(&folder), ColorSpace::Srgb)
            .unwrap();
        let mut duplicate = TextureRef::Asset("particle.png".into());
        textures
            .rewrite(
                &mut duplicate,
                std::slice::from_ref(&folder),
                ColorSpace::Srgb,
            )
            .unwrap();
        assert_eq!(first, duplicate);
        assert_eq!(textures.descriptors().len(), 1);
        fs::write(&source, b"not a PNG").unwrap();
        let mut other = TextureRef::Asset("particle.png".into());
        assert!(
            Textures::default()
                .rewrite(&mut other, &[folder], ColorSpace::Srgb)
                .is_err()
        );
        fs::remove_dir_all(path).unwrap();
    }
}
