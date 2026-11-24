//! Narrow static GLB assembly, adapted from tools/merge_static_glb.py.
//! Preserve embedded image bytes and standard material indices; no renderer fork.
use super::*;
use serde_json::{Value, json};
const LIMIT: usize = 64 * 1024 * 1024;
const TABLES: [&str; 7] = [
    "accessors",
    "bufferViews",
    "images",
    "samplers",
    "textures",
    "materials",
    "meshes",
];
pub(super) struct Glb {
    pub document: Value,
    binary: Vec<u8>,
}
fn number(value: &Value) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or("Invalid glTF index/length".into())
}
fn identity(value: &Value, expected: &[f64]) -> bool {
    value.as_array().is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value.as_f64() == Some(*expected))
    })
}
fn shift(value: &mut Value, offset: usize, count: usize) -> Result<(), String> {
    let index = number(value)?;
    if index >= count {
        return Err("Unbound glTF reference".into());
    }
    *value = json!(index.checked_add(offset).ok_or("glTF index overflow")?);
    Ok(())
}
fn word(bytes: &[u8], index: usize) -> Result<usize, String> {
    let word = bytes.get(index..index + 4).ok_or("Truncated GLB")?;
    Ok(u32::from_le_bytes(word.try_into().unwrap()) as usize)
}
impl Glb {
    pub fn read(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > LIMIT
            || bytes.get(..4) != Some(b"glTF")
            || word(bytes, 4)? != 2
            || word(bytes, 8)? != bytes.len()
            || bytes.get(16..20) != Some(b"JSON")
        {
            return Err("Expected bounded complete GLB 2.0".into());
        }
        let end = 20usize
            .checked_add(word(bytes, 12)?)
            .ok_or("GLB size overflow")?;
        if !end.is_multiple_of(4) || bytes.get(end + 4..end + 8) != Some(b"BIN\0") {
            return Err("Expected aligned embedded GLB".into());
        }
        let document: Value =
            serde_json::from_slice(bytes.get(20..end).ok_or("Truncated GLB JSON")?)
                .map_err(|e| e.to_string())?;
        let length = word(bytes, end)?;
        let binary = bytes.get(end + 8..).ok_or("Missing GLB binary")?;
        if length != binary.len()
            || !length.is_multiple_of(4)
            || document["buffers"].as_array().is_none_or(|v| v.len() != 1)
            || !document["buffers"][0]["uri"].is_null()
        {
            return Err("Expected one embedded GLB buffer".into());
        }
        let used = number(&document["buffers"][0]["byteLength"])?;
        if used > length || length - used > 3 {
            return Err("Invalid embedded buffer length".into());
        }
        Ok(Self {
            document,
            binary: binary.to_vec(),
        })
    }
    pub fn append(&mut self, bytes: &[u8]) -> Result<usize, String> {
        let mut source = Self::read(bytes)?;
        let doc = &mut source.document;
        let node = &doc["nodes"][0];
        if doc["nodes"].as_array().is_none_or(|v| v.len() != 1)
            || doc["meshes"].as_array().is_none_or(|v| v.len() != 1)
            || doc["scenes"].as_array().is_none_or(|v| v.len() != 1)
            || doc["scenes"][0]["nodes"] != json!([0])
            || doc["scene"].as_u64().is_some_and(|s| s != 0)
            || node["mesh"] != 0
            || node.get("children").is_some()
            || node.get("skin").is_some()
            || node.get("camera").is_some()
            || node
                .get("translation")
                .is_some_and(|v| !identity(v, &[0., 0., 0.]))
            || node
                .get("scale")
                .is_some_and(|v| !identity(v, &[1., 1., 1.]))
            || node
                .get("rotation")
                .is_some_and(|v| !identity(v, &[0., 0., 0., 1.]))
            || node.get("matrix").is_some_and(|v| {
                !identity(
                    v,
                    &[
                        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                    ],
                )
            })
            || [
                "animations",
                "skins",
                "extensionsUsed",
                "extensionsRequired",
            ]
            .iter()
            .any(|key| doc[*key].as_array().is_some_and(|v| !v.is_empty()))
        {
            return Err("Scenery export needs one canonical static standard-material mesh".into());
        }
        let mut offsets = std::collections::BTreeMap::new();
        let mut counts = std::collections::BTreeMap::new();
        for key in TABLES {
            let destination = self
                .document
                .as_object_mut()
                .ok_or("Invalid base GLB")?
                .entry(key)
                .or_insert(json!([]));
            offsets.insert(
                key,
                destination
                    .as_array()
                    .ok_or("Invalid base GLB table")?
                    .len(),
            );
            let table = doc
                .as_object_mut()
                .ok_or("Invalid scenery GLB")?
                .entry(key)
                .or_insert(json!([]));
            let entries = table.as_array().ok_or("Invalid scenery GLB table")?;
            if entries.iter().any(|v| !v.is_object()) {
                return Err("Invalid scenery GLB table entry".into());
            }
            counts.insert(key, entries.len());
        }
        for view in doc["bufferViews"].as_array_mut().unwrap() {
            let start = view.get("byteOffset").map(number).transpose()?.unwrap_or(0);
            let length = number(&view["byteLength"])?;
            if view["buffer"].as_u64().unwrap_or(0) != 0
                || length == 0
                || start
                    .checked_add(length)
                    .is_none_or(|end| end > source.binary.len())
            {
                return Err("glTF view outside buffer".into());
            }
            view["byteOffset"] = json!(start + self.binary.len());
        }
        for accessor in doc["accessors"].as_array_mut().unwrap() {
            if accessor.get("sparse").is_some() {
                return Err("Sparse scenery accessors unsupported".into());
            }
            shift(
                &mut accessor["bufferView"],
                offsets["bufferViews"],
                counts["bufferViews"],
            )?;
        }
        for image in doc["images"].as_array_mut().unwrap() {
            if image.get("uri").is_some() {
                return Err("External scenery images unsupported".into());
            }
            shift(
                &mut image["bufferView"],
                offsets["bufferViews"],
                counts["bufferViews"],
            )?;
        }
        for texture in doc["textures"].as_array_mut().unwrap() {
            shift(&mut texture["source"], offsets["images"], counts["images"])?;
            if let Some(sampler) = texture.get_mut("sampler") {
                shift(sampler, offsets["samplers"], counts["samplers"])?;
            }
        }
        for material in doc["materials"].as_array_mut().unwrap() {
            if material.get("extensions").is_some() {
                return Err("Extended scenery materials unsupported".into());
            }
            for key in ["normalTexture", "occlusionTexture", "emissiveTexture"] {
                if let Some(texture) = material.get_mut(key) {
                    shift(
                        texture
                            .get_mut("index")
                            .ok_or("Invalid texture reference")?,
                        offsets["textures"],
                        counts["textures"],
                    )?;
                }
            }
            for key in ["baseColorTexture", "metallicRoughnessTexture"] {
                if let Some(texture) = material
                    .get_mut("pbrMetallicRoughness")
                    .and_then(|pbr| pbr.get_mut(key))
                {
                    shift(
                        texture
                            .get_mut("index")
                            .ok_or("Invalid texture reference")?,
                        offsets["textures"],
                        counts["textures"],
                    )?;
                }
            }
        }
        for mesh in doc["meshes"].as_array_mut().unwrap() {
            for primitive in mesh["primitives"]
                .as_array_mut()
                .ok_or("Missing mesh primitives")?
            {
                if !primitive.is_object() {
                    return Err("Invalid mesh primitive".into());
                }
                if primitive.get("targets").is_some()
                    || primitive["mode"].as_u64().is_some_and(|v| v != 4)
                {
                    return Err("Scenery requires static triangles".into());
                }
                for accessor in primitive["attributes"]
                    .as_object_mut()
                    .ok_or("Missing mesh attributes")?
                    .values_mut()
                {
                    shift(accessor, offsets["accessors"], counts["accessors"])?;
                }
                if let Some(indices) = primitive.get_mut("indices") {
                    shift(indices, offsets["accessors"], counts["accessors"])?;
                }
                if let Some(material) = primitive.get_mut("material") {
                    shift(material, offsets["materials"], counts["materials"])?;
                }
            }
        }
        if self
            .binary
            .len()
            .checked_add(source.binary.len())
            .is_none_or(|n| n > LIMIT)
        {
            return Err("Scenery export exceeds GLB byte budget".into());
        }
        for key in TABLES {
            self.document[key]
                .as_array_mut()
                .unwrap()
                .append(doc[key].as_array_mut().unwrap());
        }
        self.binary.extend(source.binary);
        Ok(offsets["meshes"])
    }
    pub fn instance(
        &mut self,
        mesh: usize,
        name: &str,
        transform: Transform,
    ) -> Result<(), String> {
        let nodes = self.document["nodes"]
            .as_array_mut()
            .ok_or("Missing world nodes")?;
        let index = nodes.len();
        nodes.push(json!({"name":name,"mesh":mesh,"translation":transform.translation.to_array(),"rotation":transform.rotation.to_array(),"scale":transform.scale.to_array()}));
        self.document["scenes"][0]["nodes"]
            .as_array_mut()
            .ok_or("Missing world scene")?
            .push(json!(index));
        Ok(())
    }
    pub fn finish(mut self) -> Result<Vec<u8>, String> {
        self.document["buffers"][0]["byteLength"] = json!(self.binary.len());
        let mut json = serde_json::to_vec(&self.document).map_err(|e| e.to_string())?;
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let length = 28 + json.len() + self.binary.len();
        if length > LIMIT {
            return Err("Scenery export exceeds GLB byte budget".into());
        }
        let mut bytes = Vec::with_capacity(length);
        bytes.extend(b"glTF");
        bytes.extend(2u32.to_le_bytes());
        bytes.extend((length as u32).to_le_bytes());
        bytes.extend((json.len() as u32).to_le_bytes());
        bytes.extend(b"JSON");
        bytes.extend(json);
        bytes.extend((self.binary.len() as u32).to_le_bytes());
        bytes.extend(b"BIN\0");
        bytes.extend(self.binary);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Glb {
        Glb {
            document: json!({
                "asset":{"version":"2.0"}, "buffers":[{"byteLength":16}],
                "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":12},{"buffer":0,"byteOffset":12,"byteLength":4}],
                "accessors":[{"bufferView":0,"count":1,"componentType":5126,"type":"VEC3"}],
                "images":[{"bufferView":1,"mimeType":"image/png"}],"samplers":[{}],"textures":[{"source":0,"sampler":0}],
                "materials":[{"pbrMetallicRoughness":{"baseColorTexture":{"index":0}},"normalTexture":{"index":0}}],
                "meshes":[{"primitives":[{"attributes":{"POSITION":0},"material":0}]}],
                "nodes":[{"mesh":0,"translation":[0.0,0.0,0.0],"scale":[1.0,1.0,1.0]}],"scenes":[{"name":"Scene","nodes":[0]}],"scene":0
            }),
            binary: (0u8..16).collect(),
        }
    }
    #[test]
    fn static_scenery_merge_preserves_image_bytes_and_rebases_all_references() {
        let mut base = fixture();
        let source = fixture().finish().unwrap();
        let mesh = base.append(&source).unwrap();
        assert_eq!(mesh, 1);
        let transform = Transform::from_xyz(0.125, 3., -4.).with_scale(Vec3::splat(2.));
        base.instance(mesh, "First", transform).unwrap();
        base.instance(mesh, "Second", Transform::default()).unwrap();
        let bytes = base.finish().unwrap();
        let result = Glb::read(&bytes).unwrap();
        let doc = result.document;
        assert_eq!(doc["nodes"].as_array().unwrap().len(), 3);
        assert_eq!(doc["meshes"].as_array().unwrap().len(), 2);
        assert_eq!(
            doc["meshes"][1]["primitives"][0]["attributes"]["POSITION"],
            1
        );
        assert_eq!(doc["meshes"][1]["primitives"][0]["material"], 1);
        assert_eq!(doc["accessors"][1]["bufferView"], 2);
        assert_eq!(doc["images"][1]["bufferView"], 3);
        assert_eq!(doc["textures"][1]["source"], 1);
        assert_eq!(doc["textures"][1]["sampler"], 1);
        assert_eq!(
            doc["materials"][1]["pbrMetallicRoughness"]["baseColorTexture"]["index"],
            1
        );
        assert_eq!(doc["materials"][1]["normalTexture"]["index"], 1);
        assert_eq!(doc["bufferViews"][3]["byteOffset"], 28);
        assert_eq!(&result.binary[28..32], &[12, 13, 14, 15]);
        assert_eq!(doc["nodes"][1]["translation"], json!([0.125, 3.0, -4.0]));
    }
    #[test]
    fn scenery_glb_rejects_truncation_unbound_refs_noncanonical_and_extended_assets() {
        let valid = fixture().finish().unwrap();
        for end in 0..valid.len() {
            assert!(Glb::read(&valid[..end]).is_err());
        }
        for field in 0..5 {
            let mut changed = fixture();
            match field {
                0 => changed.document["textures"][0]["source"] = json!(7),
                1 => changed.document["materials"][0]["normalTexture"] = json!("invalid"),
                2 => changed.document["nodes"][0]["translation"] = json!([1, 0, 0]),
                3 => changed.document["images"][0]["uri"] = json!("../outside.png"),
                _ => changed.document["materials"][0]["extensions"] = json!({"unknown":{}}),
            }
            assert!(fixture().append(&changed.finish().unwrap()).is_err());
        }
    }
}
