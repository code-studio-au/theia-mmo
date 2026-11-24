//! Read-only review of two closed executable catalogues. No saved-state access,
//! migration, content installation or World-reference compatibility claim.
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Write},
    path::Path,
};
use theia_core::quests::catalog::{Catalog, MAX_CATALOG_BYTES};
use theia_network::{Result, terrain::sha256};
const MAX_REPORT_BYTES: usize = 32768;
const GAMEPLAY_FIELDS: [&str; 8] = [
    "prerequisites",
    "giver",
    "turn_in",
    "repeat_policy",
    "credit_scope",
    "experience",
    "start_stage",
    "stages",
];
const USAGE: &str = "Usage: review_quest_update PREVIOUS.json CANDIDATE.json";
struct Input {
    catalog: Catalog,
    raw: Value,
    digest: String,
    bytes: usize,
}
fn digest(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|b| format!("{b:02x}")).collect()
}
impl Input {
    fn parse(bytes: &[u8]) -> Result<Self> {
        let catalog =
            Catalog::from_json(bytes).map_err(|_| "Invalid executable quest catalogue")?;
        Ok(Self {
            catalog,
            raw: serde_json::from_slice(bytes).map_err(|_| "Invalid executable quest JSON")?,
            digest: digest(bytes),
            bytes: bytes.len(),
        })
    }
    fn read(path: &Path) -> Result<Self> {
        if !path
            .metadata()
            .is_ok_and(|m| m.is_file() && m.len() <= MAX_CATALOG_BYTES as u64)
        {
            return Err("Quest input must be a bounded regular file".into());
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|_| "Quest input unavailable")?
            .take(MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::parse(&bytes)
    }
    fn quests(&self) -> BTreeMap<&str, &Value> {
        // The typed catalogue already validated all required fields and IDs.
        self.raw["quests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|q| (q["id"].as_str().unwrap(), q))
            .collect()
    }
    fn metadata(&self) -> Metadata {
        Metadata {
            id: self.catalog.id().as_str().to_owned(),
            revision: self.catalog.revision(),
            sha256: self.digest.clone(),
            bytes: self.bytes,
            quests: self.catalog.quests().count(),
        }
    }
}
#[derive(Serialize)]
struct Metadata {
    id: String,
    revision: u32,
    sha256: String,
    bytes: usize,
    quests: usize,
}
#[derive(Serialize)]
struct QuestReview {
    id: String,
    change: &'static str,
    previous_version: Option<u32>,
    candidate_version: Option<u32>,
    gameplay_fields: Vec<&'static str>,
    presentation_changed: bool,
    version_review: Option<&'static str>,
    previous_gameplay_sha256: Option<String>,
    candidate_gameplay_sha256: Option<String>,
}
#[derive(Serialize)]
struct Report {
    schema: u32,
    review_only: bool,
    validated_executable_catalogues: bool,
    previous: Metadata,
    candidate: Metadata,
    exact_bytes_unchanged: bool,
    catalogue_revision_review: Option<&'static str>,
    quest_version_review_issues: usize,
    saved_state_compatibility: &'static str,
    automatic_migration: bool,
    world_reference_validation: bool,
    quests: Vec<QuestReview>,
}
/// Sort object keys explicitly even if serde_json's preserve_order feature is
/// enabled elsewhere. Only schema-1's unordered identity-indexed arrays sort.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .map(|(k, v)| (k, canonical(v)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(canonical).collect()),
        other => other,
    }
}
fn gameplay(quest: &Value) -> Value {
    let mut quest = quest.clone();
    let object = quest.as_object_mut().unwrap();
    for key in ["title", "description", "version"] {
        object.remove(key);
    }
    object
        .get_mut("prerequisites")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    let stages = object.get_mut("stages").unwrap().as_array_mut().unwrap();
    for stage in stages.iter_mut() {
        let stage = stage.as_object_mut().unwrap();
        stage.entry("next").or_insert(Value::Null);
        let objectives = stage.get_mut("objectives").unwrap().as_array_mut().unwrap();
        for objective in objectives.iter_mut() {
            objective.as_object_mut().unwrap().remove("label");
        }
        objectives.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    }
    stages.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    canonical(quest)
}
fn labels(quest: &Value) -> BTreeMap<&str, &str> {
    quest["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["objectives"].as_array().unwrap())
        .map(|o| (o["id"].as_str().unwrap(), o["label"].as_str().unwrap()))
        .collect()
}
fn gameplay_digest(value: &Value) -> String {
    let mut bytes = b"theia-quest-gameplay-review-v1\0".to_vec();
    bytes.extend(serde_json::to_vec(value).unwrap());
    digest(&bytes)
}
fn quest_review(id: &str, previous: Option<&Value>, candidate: Option<&Value>) -> QuestReview {
    let old = previous.map(gameplay);
    let new = candidate.map(gameplay);
    let previous_version = previous.map(|q| q["version"].as_u64().unwrap() as u32);
    let candidate_version = candidate.map(|q| q["version"].as_u64().unwrap() as u32);
    let mut review = QuestReview {
        id: id.to_owned(),
        change: "unchanged",
        previous_version,
        candidate_version,
        gameplay_fields: Vec::new(),
        presentation_changed: false,
        version_review: None,
        previous_gameplay_sha256: old.as_ref().map(gameplay_digest),
        candidate_gameplay_sha256: new.as_ref().map(gameplay_digest),
    };
    match (previous, candidate) {
        (None, Some(_)) => review.change = "added",
        (Some(_), None) => review.change = "removed",
        (Some(a), Some(b)) => {
            let old = old.as_ref().unwrap();
            let new = new.as_ref().unwrap();
            review.gameplay_fields = GAMEPLAY_FIELDS
                .into_iter()
                .filter(|field| old[*field] != new[*field])
                .collect();
            review.presentation_changed = a["title"] != b["title"]
                || a["description"] != b["description"]
                || labels(a) != labels(b);
            review.change = if !review.gameplay_fields.is_empty() {
                "gameplay"
            } else if review.presentation_changed {
                "presentation"
            } else if previous_version != candidate_version {
                "version"
            } else if a != b {
                "layout"
            } else {
                "unchanged"
            };
            if candidate_version < previous_version {
                review.version_review = Some("version_regressed");
            } else if (!review.gameplay_fields.is_empty() || review.presentation_changed)
                && candidate_version == previous_version
            {
                review.version_review = Some("definition_changed_without_version_bump");
            }
        }
        (None, None) => unreachable!("Review IDs come from the union of both catalogues"),
    }
    review
}
fn compare(previous: &Input, candidate: &Input) -> Report {
    let old = previous.quests();
    let new = candidate.quests();
    let ids: BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    let quests: Vec<_> = ids
        .into_iter()
        .map(|id| quest_review(id, old.get(id).copied(), new.get(id).copied()))
        .collect();
    let exact_bytes_unchanged = previous.digest == candidate.digest;
    let catalogue_revision_review = if previous.catalog.id() != candidate.catalog.id() {
        Some("different_catalogue_identity")
    } else if candidate.catalog.revision() < previous.catalog.revision() {
        Some("revision_regressed")
    } else if !exact_bytes_unchanged && candidate.catalog.revision() == previous.catalog.revision()
    {
        Some("digest_changed_without_revision_bump")
    } else {
        None
    };
    Report {
        schema: 1,
        review_only: true,
        validated_executable_catalogues: true,
        previous: previous.metadata(),
        candidate: candidate.metadata(),
        exact_bytes_unchanged,
        catalogue_revision_review,
        quest_version_review_issues: quests.iter().filter(|q| q.version_review.is_some()).count(),
        saved_state_compatibility: "not_assessed",
        automatic_migration: false,
        world_reference_validation: false,
        quests,
    }
}
fn report_bytes(previous: &Input, candidate: &Input) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(&compare(previous, candidate))?;
    if bytes.len() > MAX_REPORT_BYTES {
        return Err("Quest update report exceeds output budget".into());
    }
    Ok(bytes)
}
fn run() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let previous = args.next().ok_or(USAGE)?;
    let candidate = args.next().ok_or(USAGE)?;
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    let previous = Input::read(Path::new(&previous))?;
    let candidate = Input::read(Path::new(&candidate))?;
    let bytes = report_bytes(&previous, &candidate)?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(&bytes)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Quest update review failed: {error}");
        std::process::exit(1);
    }
}
#[cfg(test)]
#[path = "review_quest_update/tests.rs"]
mod tests;
