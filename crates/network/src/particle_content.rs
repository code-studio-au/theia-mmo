//! Bounded particle catalog data bound by the schema-4 world manifest.
//! Client playback and dependency admission are not yet implemented.
use crate::{
    Result, content,
    shared_images::{self, ColorSpace},
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs::File, io::Read, path::Path};
use theia_core::{content::ContentId, coordinates::LocalPosition};

pub const MAX_CATALOG_BYTES: u64 = 256 * 1024;
pub const MAX_EFFECT_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_TOTAL_EFFECT_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_PLACEMENTS: usize = 256;
pub const MAX_TEXTURES: usize = 64;
pub const MAX_TEXTURE_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_TEXTURE_RGBA_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    effects: Vec<Effect>,
    placements: Vec<Placement>,
    #[serde(default)]
    textures: Vec<Texture>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    pub sha256: [u8; 32],
    pub bytes: u64,
    pub color_space: ColorSpace,
}
impl Texture {
    pub fn filename(&self) -> String {
        let hash = self
            .sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let color = match self.color_space {
            ColorSpace::Srgb => 's',
            ColorSpace::Linear => 'l',
        };
        format!("texture-{hash}-{color}.png")
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub id: String,
    pub bytes: u64,
    pub sha256: [u8; 32],
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub id: u64,
    pub effect: String,
    pub position: [f32; 3],
    pub yaw_degrees: f32,
    pub scale: f32,
}
/// Validated data only; effect contents still need Sprinkles-specific admission.
#[derive(Debug)]
pub struct Catalog(Document);
impl Catalog {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > MAX_CATALOG_BYTES {
            return Err("Particle catalog exceeds byte budget".into());
        }
        let document: Document = serde_json::from_slice(bytes)?;
        if document.schema != 1
            || document.placements.is_empty()
            || document.placements.len() > MAX_PLACEMENTS
            || document.effects.is_empty()
            || document.effects.len() > MAX_PLACEMENTS
            || document.textures.len() > MAX_TEXTURES
        {
            return Err("Unsupported or over-budget particle catalog".into());
        }
        let mut ids = BTreeSet::new();
        let mut total = 0u64;
        for effect in &document.effects {
            if ContentId::new(&effect.id).is_none()
                || effect.id == "vfx.unassigned"
                || !ids.insert(effect.id.as_str())
                || !(1..=MAX_EFFECT_BYTES).contains(&effect.bytes)
            {
                return Err("Invalid particle effect descriptor".into());
            }
            total = total
                .checked_add(effect.bytes)
                .ok_or("Particle effect size overflow")?;
        }
        if total > MAX_TOTAL_EFFECT_BYTES {
            return Err("Particle effect budget exceeded".into());
        }
        let mut texture_keys = BTreeSet::new();
        let mut texture_bytes = 0u64;
        for texture in &document.textures {
            if !(1..=shared_images::MAX_IMAGE_BYTES).contains(&texture.bytes)
                || !texture_keys.insert((texture.sha256, texture.color_space))
            {
                return Err("Invalid particle texture descriptor".into());
            }
            texture_bytes = texture_bytes
                .checked_add(texture.bytes)
                .ok_or("Particle texture size overflow")?;
        }
        if texture_bytes > MAX_TEXTURE_BYTES {
            return Err("Particle texture byte budget exceeded".into());
        }
        let mut used = BTreeSet::new();
        let mut placement_ids = BTreeSet::new();
        for placement in &document.placements {
            if placement.id == 0
                || !placement_ids.insert(placement.id)
                || !ids.contains(placement.effect.as_str())
                || LocalPosition::new(placement.position).is_none()
                || !placement.yaw_degrees.is_finite()
                || placement.yaw_degrees.abs() > 360_000.
                || !placement.scale.is_finite()
                || !(0.01..=64.).contains(&placement.scale)
            {
                return Err("Invalid particle placement".into());
            }
            used.insert(placement.effect.as_str());
        }
        if used != ids {
            return Err("Unused particle effect descriptor".into());
        }
        Ok(Self(document))
    }
    pub fn effects(&self) -> &[Effect] {
        &self.0.effects
    }
    pub fn placements(&self) -> &[Placement] {
        &self.0.placements
    }
    pub fn textures(&self) -> &[Texture] {
        &self.0.textures
    }
    /// Read the exact effect bytes described by this catalog, in catalog order.
    /// Hashes are checked on the returned buffers, so later file changes cannot
    /// alter those buffers. RON admission, texture verification and resource
    /// budgets remain separate responsibilities.
    pub fn read_effects(&self, directory: &Path) -> Result<Vec<Vec<u8>>> {
        let mut effects = Vec::with_capacity(self.0.effects.len());
        for effect in &self.0.effects {
            let path = directory.join(format!("effect-{}.ron", effect.id));
            if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
                return Err("Particle effect must be a regular file".into());
            }
            let file = File::open(path)?;
            if !file.metadata()?.is_file() {
                return Err("Particle effect must be a regular file".into());
            }
            let mut bytes = Vec::new();
            file.take(MAX_EFFECT_BYTES + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 != effect.bytes || crate::terrain::sha256(&bytes) != effect.sha256
            {
                return Err(format!(
                    "Particle effect {} does not match its descriptor",
                    effect.id
                )
                .into());
            }
            effects.push(bytes);
        }
        Ok(effects)
    }
    /// Checks exact file length and hash; callers must separately validate RON,
    /// dependencies and resource budgets before loading it with a renderer.
    pub fn verify_effects(&self, directory: &Path) -> Result<()> {
        for effect in &self.0.effects {
            let path = directory.join(format!("effect-{}.ron", effect.id));
            if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
                return Err("Particle effect must be a regular file".into());
            }
            content::verify_hash(&path, effect.sha256, Some(effect.bytes), MAX_EFFECT_BYTES)
                .map_err(|_| {
                    format!(
                        "Particle effect {} does not match its descriptor",
                        effect.id
                    )
                })?;
        }
        Ok(())
    }
    /// Verifies fixed-name PNG bytes and bounded headers, not image decoding or
    /// agreement between the catalog and references inside effect RON.
    pub fn verify_textures(&self, directory: &Path) -> Result<()> {
        let mut rgba_bytes = 0u64;
        for texture in &self.0.textures {
            let path = directory.join(texture.filename());
            if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
                return Err("Particle texture must be a regular file".into());
            }
            content::verify_hash(
                &path,
                texture.sha256,
                Some(texture.bytes),
                shared_images::MAX_IMAGE_BYTES,
            )
            .map_err(|_| "Particle texture does not match its descriptor")?;
            let mut header = [0; 33];
            File::open(path)?.read_exact(&mut header)?;
            rgba_bytes = rgba_bytes
                .checked_add(shared_images::png_rgba_bytes(&header)?)
                .ok_or("Particle texture RGBA size overflow")?;
            if rgba_bytes > MAX_TEXTURE_RGBA_BYTES {
                return Err("Particle texture RGBA budget exceeded".into());
            }
        }
        Ok(())
    }

    /// Read and validate each cataloged PNG into an owned buffer, in catalog
    /// order. Header and hash checks apply to those exact buffers; PNG decoding
    /// and renderer admission remain separate responsibilities.
    pub fn read_textures(&self, directory: &Path) -> Result<Vec<Vec<u8>>> {
        self.read_textures_with_rgba_limit(directory, MAX_TEXTURE_RGBA_BYTES)
    }

    /// Apply the caller's decoded RGBA budget to verified headers before any
    /// image decoder can allocate the corresponding pixel buffers.
    pub fn read_textures_with_rgba_limit(
        &self,
        directory: &Path,
        max_rgba_bytes: u64,
    ) -> Result<Vec<Vec<u8>>> {
        let mut textures = Vec::with_capacity(self.0.textures.len());
        let mut rgba_bytes = 0u64;
        for texture in &self.0.textures {
            let path = directory.join(texture.filename());
            if !std::fs::symlink_metadata(&path)?.file_type().is_file() {
                return Err("Particle texture must be a regular file".into());
            }
            let file = File::open(path)?;
            if !file.metadata()?.is_file() {
                return Err("Particle texture must be a regular file".into());
            }
            let mut bytes = Vec::new();
            file.take(shared_images::MAX_IMAGE_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 != texture.bytes
                || crate::terrain::sha256(&bytes) != texture.sha256
            {
                return Err("Particle texture does not match its descriptor".into());
            }
            let header: &[u8; 33] = bytes
                .get(..33)
                .ok_or("Particle texture header is truncated")?
                .try_into()?;
            rgba_bytes = rgba_bytes
                .checked_add(shared_images::png_rgba_bytes(header)?)
                .ok_or("Particle texture RGBA size overflow")?;
            if rgba_bytes > MAX_TEXTURE_RGBA_BYTES.min(max_rgba_bytes) {
                return Err("Particle texture RGBA budget exceeded".into());
            }
            textures.push(bytes);
        }
        Ok(textures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::sha256;
    use serde_json::{Value, json};
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn fixture() -> Value {
        json!({"schema":1,"effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,1.,2.],"yaw_degrees":25.,"scale":1.}]})
    }
    fn decode(value: &Value) -> Result<Catalog> {
        Catalog::decode(&serde_json::to_vec(value).unwrap())
    }
    #[test]
    fn rejects_invalid_unbound_and_over_budget_content() {
        let valid = fixture();
        let catalog = decode(&valid).unwrap();
        assert_eq!(catalog.effects()[0].id, "vfx.fire");
        assert_eq!(catalog.placements()[0].position, [0., 1., 2.]);
        for (pointer, value) in [
            ("/schema", json!(2)),
            ("/effects/0/id", json!("../file")),
            ("/effects/0/id", json!("vfx.unassigned")),
            ("/effects/0/bytes", json!(0)),
            ("/effects/0/bytes", json!(MAX_EFFECT_BYTES + 1)),
            ("/placements/0/id", json!(0)),
            ("/placements/0/effect", json!("vfx.other")),
            ("/placements/0/position", json!([4097., 0., 0.])),
            ("/placements/0/scale", json!(0.)),
            ("/placements/0/yaw_degrees", json!(360001.)),
        ] {
            let mut bad = valid.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(decode(&bad).is_err(), "{pointer}");
        }
        let mut bad = valid.clone();
        bad["effects"]
            .as_array_mut()
            .unwrap()
            .push(valid["effects"][0].clone());
        assert!(decode(&bad).is_err());
        let mut bad = valid.clone();
        bad["placements"]
            .as_array_mut()
            .unwrap()
            .push(valid["placements"][0].clone());
        assert!(decode(&bad).is_err());
        let mut bad = valid.clone();
        bad["effects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"vfx.unused","bytes":4,"sha256":sha256(b"fire")}));
        assert!(decode(&bad).is_err());
        let mut bad = valid.clone();
        bad["extra"] = json!(true);
        assert!(decode(&bad).is_err());
        let effects: Vec<_> = (0..9)
            .map(|i| json!({"id":format!("vfx.e{i}"),"bytes":MAX_EFFECT_BYTES,"sha256":sha256(b"fire")}))
            .collect();
        let placements: Vec<_> = (0..9)
            .map(|i| json!({"id":i + 1,"effect":format!("vfx.e{i}"),"position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}))
            .collect();
        assert!(decode(&json!({"schema":1,"effects":effects,"placements":placements})).is_err());
        assert!(Catalog::decode(&vec![b' '; MAX_CATALOG_BYTES as usize + 1]).is_err());
    }
    #[test]
    fn verifies_effect_bytes_and_rejects_symlinks() {
        let path = std::env::temp_dir().join(format!(
            "theia-particle-catalog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let effect = path.join("effect-vfx.fire.ron");
        let catalog = decode(&fixture()).unwrap();
        assert!(catalog.verify_effects(&path).is_err());
        assert!(catalog.read_effects(&path).is_err());
        fs::write(&effect, b"fire").unwrap();
        catalog.verify_effects(&path).unwrap();
        let retained = catalog.read_effects(&path).unwrap();
        assert_eq!(retained, vec![b"fire".to_vec()]);
        fs::write(&effect, b"fIrE").unwrap();
        assert!(catalog.verify_effects(&path).is_err());
        assert!(catalog.read_effects(&path).is_err());
        assert_eq!(retained, vec![b"fire".to_vec()]);
        fs::write(&effect, vec![0; MAX_EFFECT_BYTES as usize + 1]).unwrap();
        assert!(catalog.read_effects(&path).is_err());
        #[cfg(unix)]
        {
            fs::remove_file(&effect).unwrap();
            let target = path.join("target.ron");
            fs::write(&target, b"fire").unwrap();
            std::os::unix::fs::symlink(target, &effect).unwrap();
            assert!(catalog.verify_effects(&path).is_err());
            assert!(catalog.read_effects(&path).is_err());
        }
        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn texture_descriptors_bind_bounded_png_variants() {
        let png = include_bytes!("../tests/fixtures/shared-image.png");
        let mut document = fixture();
        let srgb = json!({"sha256":sha256(png),"bytes":png.len(),"color_space":"srgb"});
        let linear = json!({"sha256":sha256(png),"bytes":png.len(),"color_space":"linear"});
        document["textures"] = json!([srgb, linear]);
        let catalog = decode(&document).unwrap();
        assert_ne!(
            catalog.textures()[0].filename(),
            catalog.textures()[1].filename()
        );
        let path = std::env::temp_dir().join(format!(
            "theia-particle-textures-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        for texture in catalog.textures() {
            fs::write(path.join(texture.filename()), png).unwrap();
        }
        catalog.verify_textures(&path).unwrap();
        let retained = catalog.read_textures(&path).unwrap();
        assert_eq!(retained, vec![png.to_vec(), png.to_vec()]);
        let header: &[u8; 33] = png[..33].try_into().unwrap();
        let rgba = shared_images::png_rgba_bytes(header).unwrap();
        assert!(
            catalog
                .read_textures_with_rgba_limit(&path, rgba * 2)
                .is_ok()
        );
        assert!(
            catalog
                .read_textures_with_rgba_limit(&path, rgba * 2 - 1)
                .unwrap_err()
                .to_string()
                .contains("RGBA budget")
        );
        fs::write(path.join(catalog.textures()[0].filename()), b"corrupt").unwrap();
        assert!(catalog.verify_textures(&path).is_err());
        assert!(catalog.read_textures(&path).is_err());
        assert_eq!(retained, vec![png.to_vec(), png.to_vec()]);
        let mut bad = document.clone();
        bad["textures"][1] = bad["textures"][0].clone();
        assert!(decode(&bad).is_err());
        bad = document.clone();
        bad["textures"][0]["bytes"] = json!(shared_images::MAX_IMAGE_BYTES + 1);
        assert!(decode(&bad).is_err());
        let too_large: Vec<_> = (0..9u8)
            .map(|i| json!({"sha256":vec![i;32],"bytes":shared_images::MAX_IMAGE_BYTES,"color_space":"srgb"}))
            .collect();
        bad = fixture();
        bad["textures"] = json!(too_large);
        assert!(decode(&bad).is_err());
        let mut malformed = png.to_vec();
        malformed[16..20].copy_from_slice(&9000u32.to_be_bytes());
        let mut bad = fixture();
        bad["textures"] =
            json!([{"sha256":sha256(&malformed),"bytes":malformed.len(),"color_space":"srgb"}]);
        let catalog = decode(&bad).unwrap();
        fs::write(path.join(catalog.textures()[0].filename()), malformed).unwrap();
        assert!(catalog.verify_textures(&path).is_err());
        assert!(catalog.read_textures(&path).is_err());
        #[cfg(unix)]
        {
            let catalog = decode(&document).unwrap();
            let first = path.join(catalog.textures()[0].filename());
            fs::remove_file(&first).unwrap();
            let target = path.join("target.png");
            fs::write(&target, png).unwrap();
            std::os::unix::fs::symlink(target, first).unwrap();
            assert!(catalog.read_textures(&path).is_err());
        }
        fs::remove_dir_all(path).unwrap();
    }
}
