//! Bounded particle catalog data for a future versioned world package.
//! This decoder is not yet admitted by a world manifest or used by clients.
use crate::{Result, content};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};
use theia_core::{content::ContentId, coordinates::LocalPosition};

pub const MAX_CATALOG_BYTES: u64 = 256 * 1024;
pub const MAX_EFFECT_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_TOTAL_EFFECT_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_PLACEMENTS: usize = 256;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    effects: Vec<Effect>,
    placements: Vec<Placement>,
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
        fs::write(&effect, b"fire").unwrap();
        catalog.verify_effects(&path).unwrap();
        fs::write(&effect, b"fIrE").unwrap();
        assert!(catalog.verify_effects(&path).is_err());
        #[cfg(unix)]
        {
            fs::remove_file(&effect).unwrap();
            let target = path.join("target.ron");
            fs::write(&target, b"fire").unwrap();
            std::os::unix::fs::symlink(target, &effect).unwrap();
            assert!(catalog.verify_effects(&path).is_err());
        }
        fs::remove_dir_all(&path).unwrap();
    }
}
