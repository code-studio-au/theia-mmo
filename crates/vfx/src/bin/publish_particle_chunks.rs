//! Publish drafted particle overlays into a new, independently verified chunk world.
use bevy::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use theia_core::coordinates::{ChunkId, INITIAL_CHUNK_METRES, LocalFrame};
use theia_network::{
    chunks::Index,
    content::Manifest,
    particle_content::Catalog,
    terrain::{Terrain, sha256},
};
use theia_vfx::CatalogCounts;
const COUNTS: CatalogCounts = CatalogCounts {
    placements: 32,
    emitters: 64,
    vertices: 1_000_000,
    indices: 3_000_000,
    particle_slots: 8192,
    trail_history_entries: 65_536,
    sub_emitter_words: 100_000,
};
const IMAGE_LIMIT: usize = 32 * 1024 * 1024;

fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Expected a regular file".into());
    }
    let mut data = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() as u64 > limit {
        return Err("Input exceeds byte limit".into());
    }
    Ok(data)
}
fn copy_tree(source: &Path, dest: &Path, total: &mut u64, files: &mut usize) -> Result<(), String> {
    let meta = fs::symlink_metadata(source).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Symbolic links are not supported in chunk inputs".into());
    }
    if meta.is_dir() {
        fs::create_dir(dest).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            copy_tree(&entry.path(), &dest.join(entry.file_name()), total, files)?;
        }
    } else if meta.is_file() {
        *files = files.checked_add(1).ok_or("File count overflow")?;
        *total = total.checked_add(meta.len()).ok_or("File size overflow")?;
        if *files > 100_000 || *total > 16 * 1024 * 1024 * 1024 {
            return Err("Chunk copy exceeds bounded size".into());
        }
        fs::copy(source, dest).map_err(|e| e.to_string())?;
    } else {
        return Err("Unsupported chunk input file type".into());
    }
    Ok(())
}
fn parse_id(value: &serde_json::Value) -> Result<ChunkId, String> {
    let pair = value
        .as_array()
        .filter(|v| v.len() == 2)
        .ok_or("Invalid chunk id")?;
    Ok(ChunkId {
        x: pair[0].as_i64().ok_or("Invalid chunk x")?,
        z: pair[1].as_i64().ok_or("Invalid chunk z")?,
    })
}
fn parse_hash(value: &serde_json::Value) -> Result<[u8; 32], String> {
    let list = value
        .as_array()
        .filter(|v| v.len() == 32)
        .ok_or("Invalid hash")?;
    let mut result = [0; 32];
    for (out, v) in result.iter_mut().zip(list) {
        *out = u8::try_from(v.as_u64().ok_or("Invalid hash byte")?)
            .map_err(|_| "Invalid hash byte")?;
    }
    Ok(result)
}
fn run(chunks: &Path, overlays: &Path, output: &Path) -> Result<(), String> {
    let chunks = fs::canonicalize(chunks).map_err(|e| e.to_string())?;
    let overlays = fs::canonicalize(overlays).map_err(|e| e.to_string())?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    if parent.starts_with(&chunks) || parent.starts_with(&overlays) {
        return Err("Output must be outside input directories".into());
    }
    let destination = parent.join(output.file_name().ok_or("Output needs a name")?);
    if destination.exists() {
        return Err("Output already exists".into());
    }
    let index = Index::load(&chunks).map_err(|e| e.to_string())?;
    let overlay: serde_json::Value =
        serde_json::from_slice(&read(&overlays.join("draft.json"), 256 * 1024)?)
            .map_err(|e| e.to_string())?;
    if overlay["schema"].as_u64() != Some(1)
        || parse_hash(&overlay["chunk_index_identity"])? != index.identity()
    {
        return Err("Overlay does not match chunk index".into());
    }
    let mut selected = BTreeMap::new();
    for item in overlay["chunks"]
        .as_array()
        .ok_or("Invalid overlay chunks")?
    {
        let id = parse_id(&item["id"])?;
        if !index.contains(id)
            || selected
                .insert(id, parse_hash(&item["catalog_sha256"])?)
                .is_some()
        {
            return Err("Invalid or duplicate overlay chunk".into());
        }
    }
    if selected.is_empty() {
        return Err("Overlay contains no chunks".into());
    }
    let index_bytes = read(&chunks.join("world.chunks.json"), 32 * 1024)?;
    let mut index_json: serde_json::Value =
        serde_json::from_slice(&index_bytes).map_err(|e| e.to_string())?;
    let mut expected = BTreeMap::new();
    for entry in index_json["chunks"]
        .as_array()
        .ok_or("Invalid index chunks")?
    {
        let id = parse_id(&entry["id"])?;
        if expected
            .insert(id, parse_hash(&entry["package"])?)
            .is_some()
        {
            return Err("Duplicate chunk index entry".into());
        }
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = parent.join(format!(".particle-world-{}-{nonce}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let mut reserved = false;
    let result = (|| {
        let mut total = 0;
        let mut files = 0;
        for id in index.ids() {
            let name = format!("chunk_{}_{}", id.x, id.z);
            copy_tree(
                &chunks.join(&name),
                &stage.join(&name),
                &mut total,
                &mut files,
            )?;
        }
        if chunks.join("shared").exists() {
            copy_tree(
                &chunks.join("shared"),
                &stage.join("shared"),
                &mut total,
                &mut files,
            )?;
        }
        fs::write(
            stage.join("world.session.json"),
            read(&chunks.join("world.session.json"), 256 * 1024)?,
        )
        .map_err(|e| e.to_string())?;
        let mut replacements = BTreeMap::new();
        for id in index.ids() {
            let directory = stage.join(format!("chunk_{}_{}", id.x, id.z));
            let manifest_path = directory.join("world.manifest.json");
            let original_bytes = read(&manifest_path, 256 * 1024)?;
            let manifest = Manifest::decode(&original_bytes).map_err(|e| e.to_string())?;
            if expected.get(&id) != Some(&manifest.identity()) {
                return Err("Chunk manifest does not match index".into());
            }
            if manifest.has_particle_catalog() {
                return Err("Input chunk already has particles".into());
            }
            Terrain::load(&directory.join("terrain.json")).map_err(|e| e.to_string())?;
            manifest
                .verify_visual_in(&directory.join("terrain.glb"), &stage)
                .map_err(|e| e.to_string())?;
            let Some(wanted) = selected.get(&id) else {
                continue;
            };
            let draft = overlays.join(format!("chunk_{}_{}", id.x, id.z));
            let catalog_bytes = read(
                &draft.join("catalog.json"),
                theia_network::particle_content::MAX_CATALOG_BYTES,
            )?;
            if sha256(&catalog_bytes) != *wanted {
                return Err("Overlay catalog hash mismatch".into());
            }
            let catalog = Catalog::decode(&catalog_bytes).map_err(|e| e.to_string())?;
            let frame = LocalFrame::new(
                id.origin(INITIAL_CHUNK_METRES)
                    .ok_or("Invalid chunk origin")?,
            );
            let snapshots = theia_vfx::draft_particle_chunks(
                &catalog,
                &draft.join("effects"),
                &draft.join("textures"),
                frame,
                &[id].into_iter().collect::<BTreeSet<_>>(),
                256 * 1024 * 1024,
            )?;
            let snapshot = snapshots.get(&id).ok_or("Empty chunk overlay")?;
            let particles = directory.join("particles");
            fs::create_dir(&particles).map_err(|e| e.to_string())?;
            fs::create_dir(particles.join("effects")).map_err(|e| e.to_string())?;
            fs::create_dir(particles.join("textures")).map_err(|e| e.to_string())?;
            for (name, data) in &snapshot.effects {
                fs::write(particles.join("effects").join(name), data).map_err(|e| e.to_string())?;
            }
            for (name, data) in &snapshot.textures {
                fs::write(particles.join("textures").join(name), data)
                    .map_err(|e| e.to_string())?;
            }
            fs::write(particles.join("catalog.json"), &snapshot.catalog)
                .map_err(|e| e.to_string())?;
            let checked = Catalog::decode(&snapshot.catalog).map_err(|e| e.to_string())?;
            theia_vfx::prepare_catalog_with_image_limit(
                &checked,
                &particles.join("effects"),
                &particles.join("textures"),
                Path::new("particles"),
                frame,
                frame,
                COUNTS,
                IMAGE_LIMIT,
            )?;
            let (updated, identity) =
                theia_vfx::bind_chunk_catalog(&original_bytes, &snapshot.catalog)?;
            fs::write(manifest_path, updated).map_err(|e| e.to_string())?;
            let revised = Manifest::load_optional(&directory.join("world.manifest.json"))
                .map_err(|e| e.to_string())?
                .ok_or("Revised manifest missing")?;
            revised
                .verify_visual_in_for_particle_catalog(
                    &directory.join("terrain.glb"),
                    &stage,
                    &snapshot.catalog,
                )
                .map_err(|e| e.to_string())?;
            Terrain::load(&directory.join("terrain.json")).map_err(|e| e.to_string())?;
            replacements.insert(id, identity);
        }
        for entry in index_json["chunks"]
            .as_array_mut()
            .ok_or("Invalid index chunks")?
        {
            let id = parse_id(&entry["id"])?;
            if let Some(identity) = replacements.get(&id) {
                entry["package"] = serde_json::json!(identity);
            }
        }
        fs::write(
            stage.join("world.chunks.json"),
            serde_json::to_vec(&index_json).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let output_index = Index::load(&stage).map_err(|e| e.to_string())?;
        if output_index.ids().count() != expected.len() {
            return Err("Output index count changed".into());
        }
        let config = theia_network::chunk_world::Config::load(&stage).map_err(|e| e.to_string())?;
        let (startup, _) = config.prepare_startup().map_err(|e| e.to_string())?;
        theia_vfx::prepare_resident_particles(&startup, COUNTS, IMAGE_LIMIT)?;
        fs::create_dir(&destination).map_err(|e| e.to_string())?;
        reserved = true;
        for entry in fs::read_dir(&stage).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_name() == "world.chunks.json" {
                continue;
            }
            fs::rename(entry.path(), destination.join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
        fs::rename(
            stage.join("world.chunks.json"),
            destination.join("world.chunks.json"),
        )
        .map_err(|e| e.to_string())?;
        fs::remove_dir(&stage).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
        if reserved {
            let _ = fs::remove_dir_all(&destination);
        }
    }
    result
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: publish_particle_chunks CHUNK_WORLD OVERLAY_DRAFT NEW_OUTPUT".into());
    }
    run(
        &PathBuf::from(&args[0]),
        &PathBuf::from(&args[1]),
        &PathBuf::from(&args[2]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_sprinkles::{
        asset::ParticlesAsset,
        prelude::{EmitterData, ParticlesDimension},
    };
    use serde_json::json;

    #[test]
    fn publishes_new_world_and_rejects_tampered_overlay_without_output() {
        let root =
            std::env::temp_dir().join(format!("theia-publish-particles-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let source = root.join("source");
        let overlay = root.join("overlay");
        let chunk = source.join("chunk_0_0");
        fs::create_dir_all(&chunk).unwrap();
        fs::create_dir_all(overlay.join("chunk_0_0/effects")).unwrap();
        fs::create_dir(overlay.join("chunk_0_0/textures")).unwrap();
        let mut gltf_json = br#"{"asset":{"version":"2.0"}}"#.to_vec();
        while !gltf_json.len().is_multiple_of(4) {
            gltf_json.push(b' ');
        }
        let mut visual = b"glTF".to_vec();
        visual.extend(2u32.to_le_bytes());
        visual.extend((20u32 + gltf_json.len() as u32).to_le_bytes());
        visual.extend((gltf_json.len() as u32).to_le_bytes());
        visual.extend(b"JSON");
        visual.extend(gltf_json);
        fs::write(chunk.join("terrain.glb"), &visual).unwrap();
        let profile=serde_json::to_vec(&json!({"schema":2,"bounds":[[-1.,-1.],[129.,129.]],"source_origin":[0.,0.,0.],"visual_sha256":sha256(&visual),"triangles":[[[-1.,0.,-1.],[-1.,0.,129.],[129.,0.,-1.]],[[129.,0.,-1.],[-1.,0.,129.],[129.,0.,129.]]]})).unwrap();
        fs::write(chunk.join("terrain.json"), &profile).unwrap();
        let manifest=serde_json::to_vec(&json!({"schema":1,"content_id":"theia.test.chunk","revision":1,"collision_schema":2,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(&visual)})).unwrap();
        fs::write(chunk.join("world.manifest.json"), &manifest).unwrap();
        let index = json!({"schema":1,"chunk_metres":128,"query_origin":[0.,0.,0.],"chunks":[{"id":[0,0],"package":Manifest::decode(&manifest).unwrap().identity()}]});
        fs::write(
            source.join("world.chunks.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        fs::write(source.join("world.session.json"),serde_json::to_vec(&json!({"schema":1,"resident_budget":1,"initial_chunks":[[0,0]],"spawns":[[4.,0.,4.]],"fall_plane":-20.})).unwrap()).unwrap();
        let effect = ParticlesAsset::new(
            "test".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let effect = ron::to_string(&effect).unwrap().into_bytes();
        fs::write(
            overlay.join("chunk_0_0/effects/effect-vfx.test.ron"),
            &effect,
        )
        .unwrap();
        let catalog=serde_json::to_vec(&json!({"schema":1,"effects":[{"id":"vfx.test","bytes":effect.len(),"sha256":sha256(&effect)}],"placements":[{"id":1,"effect":"vfx.test","position":[3.,0.,3.],"yaw_degrees":0.,"scale":1.}],"textures":[]})).unwrap();
        fs::write(overlay.join("chunk_0_0/catalog.json"), &catalog).unwrap();
        let report = json!({"schema":1,"chunk_index_identity":Index::load(&source).unwrap().identity(),"chunks":[{"id":[0,0],"catalog_sha256":sha256(&catalog)}]});
        fs::write(
            overlay.join("draft.json"),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        let output = root.join("output");
        run(&source, &overlay, &output).unwrap();
        assert_eq!(
            fs::read(chunk.join("world.manifest.json")).unwrap(),
            manifest
        );
        let revised = Manifest::load_optional(&output.join("chunk_0_0/world.manifest.json"))
            .unwrap()
            .unwrap();
        assert_eq!(revised.revision(), 2);
        assert!(revised.has_particle_catalog());
        let config = theia_network::chunk_world::Config::load(&output).unwrap();
        let (active, _) = config.prepare_startup().unwrap();
        assert_eq!(
            theia_vfx::prepare_resident_particles(&active, COUNTS, IMAGE_LIMIT)
                .unwrap()
                .counts
                .placements,
            1
        );
        assert!(run(&source, &overlay, &output).is_err());
        fs::write(overlay.join("chunk_0_0/catalog.json"), b"corrupt").unwrap();
        let failed = root.join("failed");
        assert!(run(&source, &overlay, &failed).is_err());
        assert!(!failed.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
