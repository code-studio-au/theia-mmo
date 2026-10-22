//! Self-contained, data-only source projection profiles in glTF material extras.
//! Package visual bytes bind these parameters; canonical shared-image manifests
//! bind all four PNGs. No shader paths, source scripts or arbitrary file URLs.
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use theia_core::coordinates::GlobalPosition;

pub const EXTRA: &str = "theia_triplanar_profile";
pub const SAMPLER: [u32; 4] = [10497, 10497, 9729, 9729];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Textures {
    top: String,
    side: String,
    top_normal: String,
    side_normal: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    profile: String,
    id: u16,
    tiling: f32,
    falloff: f32,
    /// Global f64 anchor corresponding to authored source world zero.
    source_anchor: [f64; 3],
    textures: Textures,
}

/// Constructed only after numeric, source-frame and canonical URI validation.
#[derive(Clone, Debug, PartialEq)]
pub struct TriplanarProfile(Document);
impl TriplanarProfile {
    pub fn from_extras(extras: &Value) -> Result<Option<Self>> {
        let Some(value) = extras.get(EXTRA) else {
            // Existing inspector-only integer marks remain provenance. Normal
            // rendering must separately reject marks without a bound profile.
            return Ok(None);
        };
        let mut data: Document = serde_json::from_value(value.clone())?;
        if data.schema != 1
            || data.profile != "synty-triplanar-world-v1"
            || !(1..=16).contains(&data.id)
            || extras.get("theia_triplanar").and_then(Value::as_u64) != Some(u64::from(data.id))
            || !data.tiling.is_finite()
            || !(0. ..=1000.).contains(&data.tiling)
            || data.tiling == 0.
            || !data.falloff.is_finite()
            || !(0. ..=100.).contains(&data.falloff)
            || data.falloff == 0.
            || GlobalPosition::new(data.source_anchor).is_none()
        {
            return Err("Invalid bound triplanar parameters/source anchor".into());
        }
        for value in &mut data.source_anchor {
            if *value == 0. {
                *value = 0.;
            }
        }
        let result = Self(data);
        for (uri, srgb) in result.textures() {
            let parts = uri
                .strip_prefix("../shared/")
                .and_then(|v| v.strip_suffix(".png"))
                .map(|v| v.split('-').collect::<Vec<_>>())
                .ok_or("Noncanonical triplanar image URI")?;
            if parts.len() != 6
                || parts[0].len() != 64
                || !parts[0]
                    .bytes()
                    .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
                || parts[1] != if srgb { "s" } else { "l" }
                || parts[2..] != ["10497", "10497", "9729", "9729"]
            {
                return Err("Triplanar image role/sampler/URI differs".into());
            }
        }
        Ok(Some(result))
    }
    pub fn id(&self) -> u16 {
        self.0.id
    }
    pub fn tiling(&self) -> f32 {
        self.0.tiling
    }
    pub fn falloff(&self) -> f32 {
        self.0.falloff
    }
    pub fn source_anchor(&self) -> GlobalPosition {
        GlobalPosition::new(self.0.source_anchor).expect("Validated source anchor")
    }
    pub fn textures(&self) -> [(&str, bool); 4] {
        [
            (&self.0.textures.top, true),
            (&self.0.textures.side, true),
            (&self.0.textures.top_normal, false),
            (&self.0.textures.side_normal, false),
        ]
    }
    /// Content keys avoid collisions between source-local numeric IDs from
    /// different packages. Equal bound parameters/resources may share a material.
    pub fn key(&self) -> Result<[u8; 32]> {
        let mut bytes = b"theia-bound-triplanar-v1\0".to_vec();
        bytes.extend(serde_json::to_vec(&(
            self.0.tiling,
            self.0.falloff,
            self.0.source_anchor,
            &self.0.textures,
        ))?);
        Ok(crate::terrain::sha256(&bytes))
    }
}
