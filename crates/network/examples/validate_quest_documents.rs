//! Offline authored-reference check; never opens a session or mutates saved data.
use std::{fs, io::Read, path::Path, path::PathBuf};
use theia_core::{
    documents, inventory,
    quests::{
        areas,
        catalog::{self, EventKey},
        item_rewards,
    },
};
use theia_network::{Result, terrain::sha256};
const USAGE: &str = "Usage: validate_quest_documents ITEMS.json DOCUMENTS.json QUESTS.json [AREAS.json] [--item-rewards REWARDS.json]";
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
    reward_file: Option<&Path>,
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
    let (area_catalog, area_report) = if let Some(path) = areas {
        let raw = bounded(path, areas::MAX_CATALOG_BYTES)?;
        let areas = areas::Catalog::from_json(&raw)?;
        areas.validate_quests(&quests)?;
        documents.catalog().validate_areas(&areas)?;
        let report = serde_json::json!({"id":areas.id().as_str(),"bytes":raw.len(),"sha256":hex(&sha256(&raw))});
        (Some(areas), Some(report))
    } else {
        if has_areas {
            return Err("Quest area objectives require AREAS.json".into());
        }
        (None, None)
    };
    let mut report = serde_json::json!({
        "schema":1,"validated":true,
        "scope":"Offline item, document, quest and optional area references only",
        "world_references_validated":false,"saved_state_admitted":false,
        "quest_journal_profile_bound":false,"runtime_pack_loading":false,
        "items":{"id":items.catalog().id().as_str(),"bytes":item_raw.len(),"sha256":hex(&sha256(&item_raw))},
        "documents":{"id":documents.catalog().id().as_str(),"bytes":document_raw.len(),"sha256":hex(&sha256(&document_raw)),"profile_sha256":hex(&documents.hash()),"count":documents.catalog().documents().len()},
        "quests":{"id":quests.id().as_str(),"bytes":quest_raw.len(),"sha256":hex(&sha256(&quest_raw)),"count":quests.quests().count()},
        "areas":area_report
    });
    if let Some(path) = reward_file {
        let raw = bounded(path, item_rewards::MAX_CATALOG_BYTES)?;
        let fixed = item_rewards::Catalog::from_json(&raw, &quests, items.catalog())?;
        fixed.validate_context(area_catalog.as_ref(), Some(documents.catalog()))?;
        report["scope"] =
            "Offline item, document, quest, optional area and fixed reward references only".into();
        report["fixed_rewards"] = serde_json::json!({"id":fixed.id().as_str(),"bytes":raw.len(),"sha256":hex(&sha256(&raw)),"quests":fixed.rewards().len(),"grants":fixed.rewards().map(|(_,g)|g.len()).sum::<usize>()});
    }
    Ok(report)
}
fn optional_inputs(args: &[PathBuf]) -> Result<(Option<&Path>, Option<&Path>)> {
    let mut area = None;
    let mut reward = None;
    let mut i = 0;
    while i < args.len() {
        if args[i].as_os_str() == "--item-rewards" {
            let path = args.get(i + 1).ok_or(USAGE)?;
            if reward.is_some() || path.as_os_str().to_string_lossy().starts_with('-') {
                return Err(USAGE.into());
            }
            reward = Some(path.as_path());
            i += 2;
        } else {
            if area.is_some() || args[i].as_os_str().to_string_lossy().starts_with('-') {
                return Err(USAGE.into());
            }
            area = Some(args[i].as_path());
            i += 1;
        }
    }
    Ok((area, reward))
}
fn run() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if !(3..=6).contains(&args.len()) {
        return Err(USAGE.into());
    }
    let (areas, rewards) = optional_inputs(&args[3..])?;
    println!(
        "{}",
        serde_json::to_string_pretty(&inspect(&args[0], &args[1], &args[2], areas, rewards)?)?
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
