//! Draft particle overlays for an existing chunk index without publishing it.
use std::{collections::BTreeSet, fs, io::Read, path::Path};
use theia_core::coordinates::{GlobalPosition, LocalFrame};
use theia_network::{
    chunks::Index,
    content::Manifest,
    particle_content::{Catalog, MAX_CATALOG_BYTES},
};

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Expected a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Input exceeds byte limit".into());
    }
    Ok(bytes)
}

pub fn run(source: &Path, chunks: &Path, output: &Path, origin: [f64; 3]) -> Result<(), String> {
    let source = fs::canonicalize(source).map_err(|e| e.to_string())?;
    let chunks = fs::canonicalize(chunks).map_err(|e| e.to_string())?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    if parent.starts_with(&source) || parent.starts_with(&chunks) {
        return Err("Output must be outside source and chunk directories".into());
    }
    let name = output
        .file_name()
        .ok_or("Output requires a directory name")?;
    let destination = parent.join(name);
    if destination.exists() {
        return Err("Output directory already exists".into());
    }
    let origin = GlobalPosition::new(origin).ok_or("Invalid source origin")?;
    let manifest = Manifest::decode(&read_bounded(
        &source.join("world.manifest.json"),
        256 * 1024,
    )?)
    .map_err(|e| e.to_string())?;
    let catalog_bytes = read_bounded(&source.join("particles/catalog.json"), MAX_CATALOG_BYTES)?;
    manifest
        .verify_particle_catalog(&catalog_bytes)
        .map_err(|e| e.to_string())?;
    from_catalog(
        &catalog_bytes,
        &source.join("particles/effects"),
        &source.join("particles/textures"),
        &chunks,
        &destination,
        origin.metres(),
        Some(manifest.identity()),
    )
}

/// Draft a validated catalog and its hash-bound files without requiring a world
/// geometry export. `source_identity` is optional provenance, not authority.
#[allow(clippy::too_many_arguments)]
pub fn from_catalog(
    catalog_bytes: &[u8],
    effects: &Path,
    textures: &Path,
    chunks: &Path,
    output: &Path,
    origin: [f64; 3],
    source_identity: Option<[u8; 32]>,
) -> Result<(), String> {
    let chunks = fs::canonicalize(chunks).map_err(|e| e.to_string())?;
    let effects = fs::canonicalize(effects).map_err(|e| e.to_string())?;
    let textures = fs::canonicalize(textures).map_err(|e| e.to_string())?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    if parent.starts_with(&chunks) || parent.starts_with(&effects) || parent.starts_with(&textures)
    {
        return Err("Output must be outside input directories".into());
    }
    let destination = parent.join(
        output
            .file_name()
            .ok_or("Output requires a directory name")?,
    );
    if destination.exists() {
        return Err("Output directory already exists".into());
    }
    let origin = GlobalPosition::new(origin).ok_or("Invalid source origin")?;
    let catalog = Catalog::decode(catalog_bytes).map_err(|e| e.to_string())?;
    let index = Index::load(&chunks).map_err(|e| e.to_string())?;
    let ids: BTreeSet<_> = index.ids().collect();
    let drafts = crate::draft_particle_chunks(
        &catalog,
        &effects,
        &textures,
        LocalFrame::new(origin),
        &ids,
        256 * 1024 * 1024,
    )?;
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let result = (|| {
        let mut entries = Vec::new();
        for (id, draft) in drafts {
            let directory = destination.join(format!("chunk_{}_{}", id.x, id.z));
            fs::create_dir(&directory).map_err(|e| e.to_string())?;
            let effects = directory.join("effects");
            let textures = directory.join("textures");
            fs::create_dir(&effects).map_err(|e| e.to_string())?;
            fs::create_dir(&textures).map_err(|e| e.to_string())?;
            for (name, data) in draft.effects {
                fs::write(effects.join(name), data).map_err(|e| e.to_string())?;
            }
            for (name, data) in draft.textures {
                fs::write(textures.join(name), data).map_err(|e| e.to_string())?;
            }
            fs::write(directory.join("catalog.json"), &draft.catalog).map_err(|e| e.to_string())?;
            entries.push(serde_json::json!({"id":[id.x,id.z], "catalog_sha256":theia_network::terrain::sha256(&draft.catalog)}));
        }
        let report = serde_json::json!({"schema":1,"source_identity":source_identity,"source_catalog_sha256":theia_network::terrain::sha256(catalog_bytes),"chunk_index_identity":index.identity(),"source_origin":origin.metres(),"chunks":entries,"scope":"unpublished particle overlays; chunk geometry, manifests and index are unchanged"});
        fs::write(
            destination.join("draft.json"),
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&destination);
    }
    result
}

/// Draft an explicit empty replacement, allowing an author to clear all world
/// particles without inventing a catalog containing zero placements.
pub fn empty(chunks: &Path, output: &Path, origin: [f64; 3]) -> Result<(), String> {
    let chunks = fs::canonicalize(chunks).map_err(|e| e.to_string())?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    if parent.starts_with(&chunks) {
        return Err("Output must be outside input directories".into());
    }
    let destination = parent.join(
        output
            .file_name()
            .ok_or("Output requires a directory name")?,
    );
    let origin = GlobalPosition::new(origin).ok_or("Invalid source origin")?;
    let index = Index::load(&chunks).map_err(|e| e.to_string())?;
    fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let report = serde_json::json!({"schema":1,"source_identity":null,"source_catalog_sha256":null,"chunk_index_identity":index.identity(),"source_origin":origin.metres(),"chunks":[],"scope":"unpublished explicit empty particle replacement"});
    let result = fs::write(
        destination.join("draft.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string());
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::default;
    use bevy_sprinkles::asset::ParticlesAsset;
    use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
    use serde_json::json;
    use theia_network::terrain::sha256;

    #[test]
    fn writes_verified_overlay_and_refuses_missing_owner_or_existing_output() {
        let root =
            std::env::temp_dir().join(format!("theia-particle-overlay-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let source = root.join("source");
        let chunks = root.join("chunks");
        fs::create_dir_all(source.join("particles/effects")).unwrap();
        fs::create_dir_all(source.join("particles/textures")).unwrap();
        fs::create_dir(&chunks).unwrap();
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
            source.join("particles/effects/effect-vfx.test.ron"),
            &effect,
        )
        .unwrap();
        let catalog = serde_json::to_vec(&json!({"schema":1,"effects":[{"id":"vfx.test","bytes":effect.len(),"sha256":sha256(&effect)}],"placements":[{"id":1,"effect":"vfx.test","position":[2.,0.,1.],"yaw_degrees":0.,"scale":1.}],"textures":[]})).unwrap();
        fs::write(source.join("particles/catalog.json"), &catalog).unwrap();
        let manifest = json!({"schema":4,"content_id":"theia.test.world","revision":1,"collision_schema":2,"collision_bytes":1,"collision_sha256":vec![0;32],"visual_bytes":1,"visual_sha256":vec![0;32],"particle_bytes":catalog.len(),"particle_sha256":sha256(&catalog)});
        fs::write(
            source.join("world.manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(chunks.join("world.chunks.json"),serde_json::to_vec(&json!({"schema":1,"chunk_metres":128,"query_origin":[0.,0.,0.],"chunks":[{"id":[0,0],"package":vec![0;32]}]})).unwrap()).unwrap();
        let output = root.join("draft");
        run(&source, &chunks, &output, [120., 0., 0.]).unwrap();
        let chunk = theia_network::particle_content::Catalog::decode(
            &fs::read(output.join("chunk_0_0/catalog.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(chunk.placements()[0].position, [122., 0., 1.]);
        assert_eq!(
            fs::read(output.join("chunk_0_0/effects/effect-vfx.test.ron")).unwrap(),
            effect
        );
        assert!(run(&source, &chunks, &output, [120., 0., 0.]).is_err());
        let missing = root.join("missing");
        assert!(run(&source, &chunks, &missing, [1000., 0., 0.]).is_err());
        assert!(!missing.exists());
        fs::remove_file(source.join("world.manifest.json")).unwrap();
        let direct = root.join("direct");
        from_catalog(
            &catalog,
            &source.join("particles/effects"),
            &source.join("particles/textures"),
            &chunks,
            &direct,
            [120., 0., 0.],
            None,
        )
        .unwrap();
        assert_eq!(
            fs::read(direct.join("chunk_0_0/catalog.json")).unwrap(),
            fs::read(output.join("chunk_0_0/catalog.json")).unwrap()
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(direct.join("draft.json")).unwrap()).unwrap();
        assert!(report["source_identity"].is_null());
        assert_eq!(report["source_catalog_sha256"], json!(sha256(&catalog)));
        fs::remove_dir_all(root).unwrap();
    }
}
