//! Bound shared PNG dependencies for the standard Bevy glTF/image loaders.
use crate::{Result, content};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

pub const MAX_IMAGES: usize = 512;
pub const MAX_IMAGE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_RGBA_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_GLB_JSON: u32 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    Srgb,
    Linear,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedImage {
    pub sha256: [u8; 32],
    pub bytes: u64,
    pub color_space: ColorSpace,
    /// glTF wrapS, wrapT, minFilter, magFilter; zero means unspecified filter.
    pub sampler: [u32; 4],
}
impl SharedImage {
    pub fn key(&self) -> ([u8; 32], ColorSpace, [u32; 4]) {
        (self.sha256, self.color_space, self.sampler)
    }
    pub fn filename(&self) -> String {
        let hash = self
            .sha256
            .iter()
            .map(|v| format!("{v:02x}"))
            .collect::<String>();
        let color = match self.color_space {
            ColorSpace::Srgb => 's',
            ColorSpace::Linear => 'l',
        };
        let [s, t, min, mag] = self.sampler;
        format!("{hash}-{color}-{s}-{t}-{min}-{mag}.png")
    }
    pub fn uri(&self) -> String {
        format!("../shared/{}", self.filename())
    }
    fn valid(&self) -> bool {
        (1..=MAX_IMAGE_BYTES).contains(&self.bytes) && valid_sampler(self.sampler)
    }
}
fn valid_sampler([s, t, min, mag]: [u32; 4]) -> bool {
    [s, t]
        .into_iter()
        .all(|v| matches!(v, 10497 | 33071 | 33648))
        && matches!(min, 0 | 9728 | 9729 | 9984..=9987)
        && matches!(mag, 0 | 9728 | 9729)
}
pub(crate) fn canonicalize(images: &mut [SharedImage]) -> Result<()> {
    if images.len() > MAX_IMAGES || images.iter().any(|i| !i.valid()) {
        return Err("Invalid or over-budget shared images".into());
    }
    images.sort_by_key(SharedImage::key);
    if images.windows(2).any(|pair| pair[0].key() == pair[1].key())
        || images.iter().map(|i| i.bytes).sum::<u64>() > MAX_TOTAL_BYTES
    {
        return Err("Duplicate or over-budget shared images".into());
    }
    Ok(())
}
fn array<'a>(document: &'a Value, key: &str) -> Result<&'a [Value]> {
    match document.get(key) {
        None => Ok(&[]),
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| "Invalid GLB dependency array".into()),
    }
}
/// The two standard material payloads already exercised by native Bevy. They
/// introduce no image/geometry indices; texture transforms and other extension
/// locations still require a separate dependency contract.
fn material_extensions(document: &Value) -> Result<()> {
    let mut declarations = Vec::new();
    for key in ["extensionsUsed", "extensionsRequired"] {
        let mut names = BTreeSet::new();
        for entry in array(document, key)? {
            let name = entry.as_str().ok_or("Invalid material extension name")?;
            if !matches!(
                name,
                "KHR_materials_unlit" | "KHR_materials_emissive_strength"
            ) || !names.insert(name)
            {
                return Err("Unsupported or repeated static material extension".into());
            }
        }
        declarations.push(names);
    }
    if !declarations[1].is_subset(&declarations[0]) {
        return Err("Required material extension is undeclared".into());
    }
    let mut pending = vec![(document, false)];
    while let Some((value, material)) = pending.pop() {
        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    if key == "extensions" {
                        let extensions = child.as_object().ok_or("Invalid extension object")?;
                        if !material && !extensions.is_empty() {
                            return Err("Only static material extensions are supported".into());
                        }
                        for (name, payload) in extensions {
                            if !declarations[0].contains(name.as_str()) {
                                return Err("Undeclared material extension payload".into());
                            }
                            let fields = payload.as_object().ok_or("Invalid material payload")?;
                            match name.as_str() {
                                "KHR_materials_unlit" if fields.is_empty() => {}
                                "KHR_materials_emissive_strength" if fields.len() == 1 => {
                                    let strength = fields
                                        .get("emissiveStrength")
                                        .and_then(Value::as_f64)
                                        .ok_or("Missing numeric emissive strength")?;
                                    if !strength.is_finite() || !(0. ..=16.).contains(&strength) {
                                        return Err(
                                            "Emissive strength exceeds static profile".into()
                                        );
                                    }
                                }
                                _ => return Err("Unsupported static material payload".into()),
                            }
                        }
                    } else if std::ptr::eq(value, document) && key == "materials" {
                        pending.extend(array(document, "materials")?.iter().map(|v| (v, true)));
                    } else {
                        pending.push((child, false));
                    }
                }
            }
            Value::Array(values) => pending.extend(values.iter().map(|v| (v, false))),
            _ => {}
        }
    }
    Ok(())
}
fn index(value: Option<&Value>) -> Result<usize> {
    value
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| "Invalid GLB dependency index".into())
}
fn sampler(document: &Value, texture: &Value) -> Result<[u32; 4]> {
    let empty = Value::Null;
    let settings = if texture.get("sampler").is_some() {
        array(document, "samplers")?
            .get(index(texture.get("sampler"))?)
            .ok_or("Missing glTF sampler")?
    } else {
        &empty
    };
    let mut result = [10497, 10497, 0, 0];
    for (n, key) in ["wrapS", "wrapT", "minFilter", "magFilter"]
        .iter()
        .enumerate()
    {
        if let Some(value) = settings.get(key) {
            result[n] = u32::try_from(index(Some(value))?)?;
        }
    }
    if !valid_sampler(result) {
        return Err("Invalid glTF shared image sampler".into());
    }
    Ok(result)
}
fn glb_json(path: &Path, required: bool) -> Result<Option<Value>> {
    let mut file = File::open(path)?;
    let mut magic = [0; 4];
    if file.read(&mut magic)? != 4 || &magic != b"glTF" {
        return if required {
            Err("Shared-image packages require GLB 2.0".into())
        } else {
            Ok(None)
        };
    }
    let mut header = [0; 16];
    file.read_exact(&mut header)?;
    let version = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let total = u32::from_le_bytes(header[4..8].try_into().unwrap());
    let size = u32::from_le_bytes(header[8..12].try_into().unwrap());
    if version != 2
        || u64::from(total) != file.metadata()?.len()
        || &header[12..16] != b"JSON"
        || size == 0
        || size > MAX_GLB_JSON
        || size % 4 != 0
    {
        return Err("Invalid or over-budget GLB JSON dependency header".into());
    }
    let mut json = vec![0; size as usize];
    file.read_exact(&mut json)?;
    let document: Value = serde_json::from_slice(&json)?;
    if !document.is_object() {
        return Err("Invalid GLB dependency document".into());
    }
    Ok(Some(document))
}
pub(crate) fn verify(
    path: &Path,
    root: Option<&Path>,
    images: &[SharedImage],
    schema: u32,
) -> Result<()> {
    let Some(document) = glb_json(path, schema >= 2)? else {
        return Ok(());
    };
    let descriptors = images
        .iter()
        .map(|image| (image.uri(), image))
        .collect::<BTreeMap<_, _>>();
    let declared = descriptors.keys().cloned().collect::<BTreeSet<_>>();
    let mut referenced = BTreeSet::new();
    let mut projected_images = BTreeSet::new();
    let mut projected_profiles = BTreeSet::new();
    for material in array(&document, "materials")? {
        let Some(profile) = crate::material_profile::TriplanarProfile::from_extras(
            material.get("extras").unwrap_or(&Value::Null),
        )?
        else {
            continue;
        };
        if schema != 3 {
            return Err("Bound material profiles require package schema 3".into());
        }
        projected_profiles.insert(profile.key()?);
        if projected_profiles.len() > 16 {
            return Err("Bound material profile budget exceeded".into());
        }
        for (uri, srgb) in profile.textures() {
            let descriptor = descriptors
                .get(uri)
                .ok_or("Unbound projected material image")?;
            let color = if srgb {
                ColorSpace::Srgb
            } else {
                ColorSpace::Linear
            };
            if descriptor.color_space != color
                || descriptor.sampler != crate::material_profile::SAMPLER
            {
                return Err("Projected material image role/sampler differs".into());
            }
            referenced.insert(uri.to_owned());
            projected_images.insert(uri.to_owned());
        }
    }
    for buffer in array(&document, "buffers")? {
        if buffer.get("uri").is_some() {
            return Err("External GLB buffers are not bound".into());
        }
    }
    for image in array(&document, "images")? {
        if let Some(uri) = image.get("uri") {
            let uri = uri.as_str().ok_or("Invalid image URI")?;
            if !declared.contains(uri)
                || image.get("bufferView").is_some()
                || image
                    .get("mimeType")
                    .is_some_and(|v| v.as_str() != Some("image/png"))
            {
                return Err("Unbound or noncanonical shared image URI".into());
            }
            referenced.insert(uri.to_owned());
        }
    }
    if schema == 3 && projected_profiles.is_empty() {
        return Err("Package schema 3 requires bound material profiles".into());
    }
    if referenced != declared {
        return Err("Unused shared image declaration".into());
    }
    if schema == 1 {
        return Ok(());
    }
    material_extensions(&document)?;
    let textures = array(&document, "textures")?;
    let mut linear = BTreeSet::new();
    let mut colors = BTreeSet::new();
    for material in array(&document, "materials")? {
        for (pointer, is_linear) in [
            ("/normalTexture/index", true),
            ("/occlusionTexture/index", true),
            ("/pbrMetallicRoughness/metallicRoughnessTexture/index", true),
            ("/pbrMetallicRoughness/baseColorTexture/index", false),
            ("/emissiveTexture/index", false),
        ] {
            if let Some(value) = material.pointer(pointer) {
                let id = index(Some(value))?;
                if id >= textures.len() {
                    return Err("Unknown material texture".into());
                }
                if is_linear {
                    linear.insert(id);
                } else {
                    colors.insert(id);
                }
            }
        }
    }
    if !linear.is_disjoint(&colors) {
        return Err("Shared texture must separate linear and sRGB uses".into());
    }
    for (n, texture) in textures.iter().enumerate() {
        let image = array(&document, "images")?
            .get(index(texture.get("source"))?)
            .ok_or("Unknown texture image")?;
        if let Some(uri) = image.get("uri").and_then(Value::as_str) {
            let descriptor = descriptors
                .get(uri)
                .ok_or("Missing shared image descriptor")?;
            let color = if linear.contains(&n) {
                ColorSpace::Linear
            } else {
                ColorSpace::Srgb
            };
            if descriptor.color_space != color || descriptor.sampler != sampler(&document, texture)?
            {
                return Err("Shared image variant disagrees with glTF loader settings".into());
            }
        }
    }
    if images.is_empty() {
        return Ok(());
    }
    let root = root
        .ok_or("Shared images require an explicit world asset root")?
        .canonicalize()?;
    if path
        .parent()
        .and_then(Path::parent)
        .ok_or("Missing package parent")?
        .canonicalize()?
        != root
    {
        return Err("Shared-image package is outside the world asset root".into());
    }
    let mut rgba_bytes = 0u64;
    for image in images {
        let expected = root.join("shared").join(image.filename());
        if expected.canonicalize()? != expected {
            return Err("Shared image path alias is not allowed".into());
        }
        content::verify_hash(&expected, image.sha256, Some(image.bytes), MAX_IMAGE_BYTES).map_err(
            |error| {
                format!(
                    "Shared image {} failed verification: {error}",
                    image.filename()
                )
            },
        )?;
        let mut header = [0; 33];
        File::open(expected)?.read_exact(&mut header)?;
        let width = u32::from_be_bytes(header[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(header[20..24].try_into().unwrap());
        let pixels = u64::from(width) * u64::from(height);
        if projected_images.contains(&image.uri())
            && (width > 4096 || height > 4096 || !matches!(header[25], 2 | 6) || header[28] != 0)
        {
            return Err("Unsupported bounded projected-material PNG".into());
        }
        if &header[..8] != b"\x89PNG\r\n\x1a\n"
            || &header[12..16] != b"IHDR"
            || u32::from_be_bytes(header[8..12].try_into().unwrap()) != 13
            || header[24] != 8
            || !matches!(header[25], 0 | 2 | 3 | 4 | 6)
            || header[26] != 0
            || header[27] != 0
            || header[28] > 1
            || width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || pixels > 16 * 1024 * 1024
        {
            return Err("Unsupported or over-budget shared PNG header".into());
        }
        rgba_bytes += pixels * 4;
        if rgba_bytes > MAX_RGBA_BYTES {
            return Err("Shared PNG base-level budget exceeded".into());
        }
    }
    Ok(())
}
