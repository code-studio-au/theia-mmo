//! Bounded self-contained GLB envelope for the current six-clip creature adapter.
//! Bevy still loads and validates the actual meshes, skin and animation data.
use std::{collections::BTreeSet, io::Read, path::Path};

pub const MAX_BYTES: usize = 8 * 1024 * 1024;
pub const NAMES: [&str; 6] = ["Idle", "Walk", "Run", "Attack", "Hit", "Death"];

pub fn read(path: &Path) -> Result<Vec<u8>, String> {
    if path.extension().and_then(|s| s.to_str()) != Some("glb") {
        return Err("Creature appearance must be a .glb file".into());
    }
    if !path.is_file() || path.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES as u64 {
        return Err("Creature appearance must be a regular file of at most 8 MiB".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    validate(&bytes)?;
    Ok(bytes)
}
fn validate(bytes: &[u8]) -> Result<(), String> {
    let word = |offset: usize| -> Option<u32> {
        Some(u32::from_le_bytes(
            bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
        ))
    };
    if bytes.len() > MAX_BYTES
        || word(0) != Some(0x46546c67)
        || word(4) != Some(2)
        || word(8).map(|n| n as usize) != Some(bytes.len())
        || word(16) != Some(0x4e4f534a)
    {
        return Err("Creature appearance requires a complete GLB 2 file of at most 8 MiB".into());
    }
    let json_len = word(12).ok_or("Missing GLB JSON length")? as usize;
    if json_len > 2 * 1024 * 1024 || !json_len.is_multiple_of(4) {
        return Err("Creature GLB JSON exceeds its bounded aligned envelope".into());
    }
    let end = 20usize.checked_add(json_len).ok_or("Invalid GLB length")?;
    let json: serde_json::Value =
        serde_json::from_slice(bytes.get(20..end).ok_or("Truncated GLB JSON")?)
            .map_err(|e| e.to_string())?;
    let bin_len = word(end).ok_or("Missing GLB binary chunk")? as usize;
    if word(end + 4) != Some(0x004e4942)
        || bin_len == 0
        || !bin_len.is_multiple_of(4)
        || end.checked_add(8).and_then(|n| n.checked_add(bin_len)) != Some(bytes.len())
    {
        return Err("Creature GLB requires one bounded embedded binary chunk".into());
    }
    let array = |key: &str| json.get(key).and_then(serde_json::Value::as_array);
    if json["asset"]["version"] != "2.0"
        || json["scene"] != 0
        || ["scenes", "meshes", "materials", "skins", "buffers"]
            .iter()
            .any(|key| array(key).is_none_or(|v| v.len() != 1))
    {
        return Err(
            "Creature appearance needs one scene, mesh, material, skin and embedded buffer".into(),
        );
    }
    let buffers = array("buffers").unwrap();
    let declared = buffers[0]["byteLength"]
        .as_u64()
        .ok_or("Invalid embedded buffer length")?;
    if buffers[0].get("uri").is_some()
        || declared == 0
        || declared > bin_len as u64
        || bin_len as u64 - declared > 3
        || array("images").is_some_and(|images| images.iter().any(|i| i.get("uri").is_some()))
    {
        return Err("Creature appearance must embed all buffers and images in its GLB".into());
    }
    let animations = array("animations").ok_or("Missing creature animation clips")?;
    let names: BTreeSet<_> = animations
        .iter()
        .filter_map(|a| a["name"].as_str())
        .collect();
    if animations.len() != NAMES.len() || names != BTreeSet::from(NAMES) {
        return Err(
            "Creature appearance needs exactly Idle, Walk, Run, Attack, Hit and Death clips".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> serde_json::Value {
        serde_json::json!({"asset":{"version":"2.0"},"scene":0,"scenes":[{}],"meshes":[{}],
            "materials":[{}],"skins":[{}],"buffers":[{"byteLength":4}],"images":[],
            "animations":NAMES.map(|name|serde_json::json!({"name":name}))})
    }
    fn glb(document: &serde_json::Value) -> Vec<u8> {
        let mut json = serde_json::to_vec(document).unwrap();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let mut bytes = Vec::new();
        for word in [
            0x46546c67,
            2,
            (json.len() + 32) as u32,
            json.len() as u32,
            0x4e4f534a,
        ] {
            bytes.extend(word.to_le_bytes());
        }
        bytes.extend(json);
        bytes.extend(4u32.to_le_bytes());
        bytes.extend(0x004e4942u32.to_le_bytes());
        bytes.extend([0; 4]);
        bytes
    }
    #[test]
    fn bounded_envelope_refuses_truncation_external_files_and_wrong_adapter_contract() {
        let good = glb(&document());
        validate(&good).unwrap();
        for length in [0, 12, 19, good.len() - 1] {
            assert!(validate(&good[..length]).is_err());
        }
        assert!(validate(&vec![0; MAX_BYTES + 1]).is_err());
        for (field, value) in [
            (
                "buffers",
                serde_json::json!([{"byteLength":4,"uri":"outside.bin"}]),
            ),
            ("images", serde_json::json!([{"uri":"outside.png"}])),
            ("scene", serde_json::json!(1)),
            ("skins", serde_json::json!([])),
            (
                "animations",
                serde_json::json!(vec![serde_json::json!({"name":"Idle"}); 6]),
            ),
        ] {
            let mut bad = document();
            bad[field] = value;
            assert!(validate(&glb(&bad)).is_err(), "{field}");
        }
        let mut bad = document();
        bad["buffers"][0]["byteLength"] = serde_json::json!(5);
        assert!(validate(&glb(&bad)).is_err());
    }
}
