//! Offline authored-reference check; never opens a session or mutates saved data.
use std::{fs, io::Read, path::Path, path::PathBuf};
use theia_core::{
    documents, inventory,
    quests::{
        areas,
        catalog::{self, EventKey},
    },
};
use theia_network::{Result, terrain::sha256};
const USAGE: &str =
    "Usage: validate_quest_documents ITEMS.json DOCUMENTS.json QUESTS.json [AREAS.json]";
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
// The same regular-file and bounded-read policy as the other authoring tools.
fn bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "Document authoring input unavailable")?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("Document authoring requires regular bounded input files".into());
    }
    let file = fs::File::open(path).map_err(|_| "Document authoring input unavailable")?;
    if !file.metadata()?.is_file() {
        return Err("Document authoring input changed from a regular file".into());
    }
    let mut raw = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut raw)?;
    if raw.len() > limit {
        return Err("Document authoring input exceeds byte budget".into());
    }
    Ok(raw)
}
fn inspect(
    items: &Path,
    documents: &Path,
    quests: &Path,
    areas: Option<&Path>,
) -> Result<serde_json::Value> {
    let item_raw = bounded(items, inventory::MAX_CATALOG_BYTES)?;
    let document_raw = bounded(documents, documents::MAX_CATALOG_BYTES)?;
    let quest_raw = bounded(quests, catalog::MAX_CATALOG_BYTES)?;
    let items = theia_network::renet_runtime::inventory::Profile::from_json(&item_raw)?;
    let documents = theia_network::documents::Profile::from_json(&document_raw, &items)?;
    let quests = catalog::Catalog::from_json(&quest_raw)?;
    if !quests.quests().any(|q| q.has_document_objectives()) {
        return Err("Quest catalogue has no read-document objectives".into());
    }
    documents.catalog().validate_quests(&quests)?;
    let has_areas = quests
        .quests()
        .flat_map(|q| q.stages())
        .flat_map(|s| s.objectives())
        .any(|o| matches!(o.event(), EventKey::ReachArea(_)));
    let area_report = if let Some(path) = areas {
        let raw = bounded(path, areas::MAX_CATALOG_BYTES)?;
        let areas = areas::Catalog::from_json(&raw)?;
        areas.validate_quests(&quests)?;
        documents.catalog().validate_areas(&areas)?;
        Some(
            serde_json::json!({"id":areas.id().as_str(),"bytes":raw.len(),"sha256":hex(&sha256(&raw))}),
        )
    } else {
        if has_areas {
            return Err("Quest area objectives require AREAS.json".into());
        }
        None
    };
    Ok(serde_json::json!({
        "schema":1,"validated":true,
        "scope":"Offline item, document, quest and optional area references only",
        "world_references_validated":false,"saved_state_admitted":false,
        "quest_journal_profile_bound":false,"runtime_pack_loading":false,
        "items":{"id":items.catalog().id().as_str(),"bytes":item_raw.len(),"sha256":hex(&sha256(&item_raw))},
        "documents":{"id":documents.catalog().id().as_str(),"bytes":document_raw.len(),"sha256":hex(&sha256(&document_raw)),"profile_sha256":hex(&documents.hash()),"count":documents.catalog().documents().len()},
        "quests":{"id":quests.id().as_str(),"bytes":quest_raw.len(),"sha256":hex(&sha256(&quest_raw)),"count":quests.quests().count()},
        "areas":area_report
    }))
}
fn run() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if !(3..=4).contains(&args.len()) {
        return Err(USAGE.into());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&inspect(
            &args[0],
            &args[1],
            &args[2],
            args.get(3).map(PathBuf::as_path)
        )?)?
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Quest document validation failed: {error}");
        std::process::exit(1);
    }
}
#[cfg(test)]
#[path = "validate_quest_documents/tests.rs"]
mod tests;
