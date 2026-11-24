//! Read-only authoring validation and closed-catalogue compilation.
//! No World, runtime pack loading or saved-state migration.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use theia_core::{
    content::ContentId,
    quests::catalog::{
        MAX_CATALOG_BYTES, MAX_QUESTS,
        registry::{CompiledCatalogue, Error as RegistryError, MAX_CATALOGUES, Registry},
    },
};
use theia_network::{Result, terrain::sha256};
const MAX_INDEX_BYTES: usize = 16384;
const MAX_PATH_BYTES: usize = 256;
const MAX_REPORT_BYTES: usize = 32768;
const MAX_SELECTION_BYTES: usize = 8192;
const USAGE: &str = "Usage: validate_quest_packs INDEX.json [--compile-selection SELECTION.json]";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    schema: u32,
    id: String,
    revision: u32,
    catalogues: Vec<String>,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
struct Record {
    path: String,
    id: String,
    revision: u32,
    quests: usize,
    bytes: usize,
    sha256: String,
}
#[derive(Debug, Serialize, PartialEq, Eq)]
struct Report {
    schema: u32,
    validated: bool,
    scope: &'static str,
    runtime_pack_loading: bool,
    world_references_validated: bool,
    index_schema: u32,
    id: String,
    revision: u32,
    index_sha256: String,
    validation_registry_sha256: String,
    quests: usize,
    definitions: usize,
    catalogues: Vec<Record>,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    if !fs::metadata(path)
        .map_err(|_| "Quest content unavailable")?
        .is_file()
    {
        return Err("Quest input must be a regular file".into());
    }
    let file = fs::File::open(path).map_err(|_| "Quest content unavailable")?;
    if !file.metadata()?.is_file() {
        return Err("Quest input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Quest content read failed")?;
    if bytes.len() > limit {
        return Err("Quest content exceeds byte budget".into());
    }
    Ok(bytes)
}
fn relative(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_PATH_BYTES
        && name.ends_with(".json")
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        && name.split('/').all(|part| !matches!(part, "" | "." | ".."))
        && !Path::new(name).is_absolute()
}
fn context(error: RegistryError, index: &Index) -> String {
    let positions = match &error {
        RegistryError::CatalogueBytes { index } | RegistryError::Catalogue { index, .. } => {
            vec![*index]
        }
        RegistryError::DuplicateDefinition { first, second, .. } => vec![*first, *second],
        RegistryError::TargetAlias {
            catalogue,
            definition_catalogue,
            ..
        } => vec![*catalogue, *definition_catalogue],
        RegistryError::UnknownPrerequisite { catalogue, .. }
        | RegistryError::Cycle { catalogue, .. } => vec![*catalogue],
        RegistryError::CatalogueCount | RegistryError::RegistryBytes => vec![],
    };
    let names = positions
        .into_iter()
        .filter_map(|i| index.catalogues.get(i))
        .cloned()
        .collect::<Vec<_>>();
    if names.is_empty() {
        error.to_string()
    } else {
        format!("{}: {error}", names.join(", "))
    }
}
struct Loaded {
    registry: Registry,
    report: Report,
}
fn inspect(path: &Path) -> Result<Loaded> {
    let path = path.canonicalize().map_err(|_| "Quest index unavailable")?;
    let raw = bounded(&path, MAX_INDEX_BYTES)?;
    let index: Index = serde_json::from_slice(&raw).map_err(|_| "Invalid quest pack index")?;
    if index.schema != 1
        || index.revision == 0
        || ContentId::new(&index.id).is_none()
        || index.catalogues.is_empty()
        || index.catalogues.len() > MAX_CATALOGUES
    {
        return Err("Invalid quest pack index identity/version/bounds".into());
    }
    let root = path.parent().ok_or("Quest index requires a directory")?;
    let mut seen = BTreeSet::from([path.clone()]);
    let mut sources = Vec::with_capacity(index.catalogues.len());
    for name in &index.catalogues {
        if !relative(name) {
            return Err("Unsafe quest catalogue path".into());
        }
        let source = root
            .join(name)
            .canonicalize()
            .map_err(|_| format!("Missing quest catalogue {name}"))?;
        if !source.starts_with(root) || !seen.insert(source.clone()) {
            return Err(format!("Outside-root or aliased quest catalogue {name}").into());
        }
        sources.push(
            bounded(&source, MAX_CATALOG_BYTES).map_err(|_| {
                format!("Unreadable, nonregular or oversized quest catalogue {name}")
            })?,
        );
    }
    let registry =
        Registry::from_catalogues(&sources.iter().map(Vec::as_slice).collect::<Vec<_>>())
            .map_err(|error| context(error, &index))?;
    let mut identity = b"theia-quest-pack-validation-v1\0".to_vec();
    identity.extend(sha256(&raw));
    let mut catalogues = Vec::with_capacity(sources.len());
    for ((name, bytes), metadata) in index
        .catalogues
        .iter()
        .zip(&sources)
        .zip(registry.catalogues())
    {
        let hash = sha256(bytes);
        identity.extend((name.len() as u16).to_le_bytes());
        identity.extend(name.as_bytes());
        identity.extend(hash);
        catalogues.push(Record {
            path: name.clone(),
            id: metadata.id.as_str().into(),
            revision: metadata.revision,
            quests: metadata.quests,
            bytes: bytes.len(),
            sha256: hex(&hash),
        });
    }
    let report = Report {
        schema: 1,
        validated: true,
        scope: "Offline executable quest definitions, namespace and prerequisite graph",
        runtime_pack_loading: false,
        world_references_validated: false,
        index_schema: index.schema,
        id: index.id,
        revision: index.revision,
        index_sha256: hex(&sha256(&raw)),
        validation_registry_sha256: hex(&sha256(&identity)),
        quests: registry.quest_count(),
        definitions: registry.definition_count(),
        catalogues,
    };
    Ok(Loaded { registry, report })
}
fn validate(path: &Path) -> Result<Report> {
    Ok(inspect(path)?.report)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    schema: u32,
    id: String,
    revision: u32,
    quests: Vec<String>,
}
#[derive(Debug, Serialize)]
struct CompileSummary {
    schema: u32,
    closed_catalogue_validated: bool,
    world_references_validated: bool,
    id: String,
    revision: u32,
    selected_quests: Vec<String>,
    included_quests: usize,
    source_registry_sha256: String,
    selection_sha256: String,
    catalogue_sha256: String,
}
#[derive(Debug)]
struct Compilation {
    catalogue: CompiledCatalogue,
    summary: CompileSummary,
}
fn compile(index: &Path, selection: &Path) -> Result<Compilation> {
    let raw = bounded(selection, MAX_SELECTION_BYTES)?;
    let selection: Selection =
        serde_json::from_slice(&raw).map_err(|_| "Invalid quest compilation selection")?;
    if selection.schema != 1 || selection.quests.len() > MAX_QUESTS {
        return Err("Unsupported quest selection schema or count".into());
    }
    let id = ContentId::new(&selection.id).ok_or("Invalid compiled catalogue ID")?;
    let roots = selection
        .quests
        .iter()
        .map(|id| ContentId::new(id).ok_or("Invalid selected quest ID"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let loaded = inspect(index)?;
    let catalogue = loaded
        .registry
        .compile_catalogue(&id, selection.revision, &roots)?;
    let summary = CompileSummary {
        schema: 1,
        closed_catalogue_validated: true,
        world_references_validated: false,
        id: selection.id,
        revision: selection.revision,
        selected_quests: selection.quests,
        included_quests: catalogue.catalogue().quests().count(),
        source_registry_sha256: loaded.report.validation_registry_sha256,
        selection_sha256: hex(&sha256(&raw)),
        catalogue_sha256: hex(&sha256(catalogue.as_bytes())),
    };
    Ok(Compilation { catalogue, summary })
}
fn run() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let index = PathBuf::from(args.next().ok_or(USAGE)?);
    match args.next() {
        None => {
            let report = serde_json::to_string_pretty(&validate(&index)?)?;
            if report.len() > MAX_REPORT_BYTES {
                return Err("Quest validation report exceeds byte budget".into());
            }
            println!("{report}");
        }
        Some(flag) if flag == "--compile-selection" => {
            let selection = PathBuf::from(args.next().ok_or(USAGE)?);
            if args.next().is_some() {
                return Err(USAGE.into());
            }
            let compiled = compile(&index, &selection)?;
            let summary = serde_json::to_string(
                &serde_json::json!({"quest_catalogue_compile":compiled.summary}),
            )?;
            if summary.len() > MAX_SELECTION_BYTES {
                return Err("Quest compilation summary exceeds byte budget".into());
            }
            std::io::stdout()
                .lock()
                .write_all(compiled.catalogue.as_bytes())?;
            eprintln!("{summary}");
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Quest pack validation failed: {error}");
        std::process::exit(1);
    }
}
#[cfg(test)]
#[path = "validate_quest_packs/tests.rs"]
mod tests;
