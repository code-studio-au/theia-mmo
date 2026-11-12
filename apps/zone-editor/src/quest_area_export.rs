//! Explicit, reference-checked area-catalog export. Does not activate a server bundle.
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use theia_core::quests::{areas, catalog};
use theia_zone::Zone;

pub(super) fn prepare(
    zone: &Zone,
    quests: &[u8],
    id: &str,
    revision: u32,
) -> Result<Vec<u8>, String> {
    let quests = catalog::Catalog::from_json(quests).map_err(|e| e.to_string())?;
    let bytes = theia_zone::quest_areas::draft_catalog(zone, id, revision)?;
    areas::Catalog::from_json(&bytes)
        .map_err(str::to_owned)?
        .validate_quests(&quests)
        .map_err(str::to_owned)?;
    Ok(bytes)
}

pub(super) fn publish(
    zone: &Zone,
    quests: &Path,
    output: &Path,
    id: &str,
    revision: u32,
) -> Result<serde_json::Value, String> {
    let file = fs::File::open(quests).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Quest catalog must be a regular file".into());
    }
    let mut source = Vec::new();
    file.take(catalog::MAX_CATALOG_BYTES as u64 + 1)
        .read_to_end(&mut source)
        .map_err(|e| e.to_string())?;
    let bytes = prepare(zone, &source, id, revision)?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let candidate = parent.join(format!(".quest-areas-{}-{nonce}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&candidate)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        // Same-directory hard link publishes complete bytes and refuses replacement.
        fs::hard_link(&candidate, output).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(candidate);
    result?;
    Ok(
        serde_json::json!({"areas":zone.quest_areas.len(),"id":id,"revision":revision,"quest_references_valid":true,"scope":"standalone catalog; gameplay bundle and authoritative world admission remain separate"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn zone() -> Zone {
        Zone {
            schema: 9,
            origin: [8000., 0., 8000.],
            spawn: [8000., 0., 8000.],
            quest_areas: vec![theia_zone::quest_areas::QuestArea {
                id: 1,
                definition: "theia.dev.area.western-overlook".into(),
                name: "Western overlook".into(),
                min: [7856., -64., 7984.],
                max: [7872., 64., 8016.],
                map_objectives: vec!["theia.dev.quest-overlook.objective.reach".into()],
            }],
            ..Zone::default()
        }
    }
    const QUESTS: &[u8] = include_bytes!("../../../content/quests/development-journal-areas.json");
    #[test]
    fn authored_catalog_resolves_runtime_quest_and_minimap_links() {
        let mut zone = zone();
        let bytes = prepare(&zone, QUESTS, "theia.areas.starter", 1).unwrap();
        let areas = areas::Catalog::from_json(&bytes).unwrap();
        assert_eq!(
            areas.areas().next().unwrap().min().metres(),
            zone.quest_areas[0].min
        );
        zone.quest_areas[0].map_objectives = vec!["theia.missing.objective".into()];
        assert!(prepare(&zone, QUESTS, "theia.areas.starter", 1).is_err());
        zone.quest_areas[0].map_objectives.clear();
        zone.quest_areas[0].definition = "theia.wrong.area".into();
        assert!(prepare(&zone, QUESTS, "theia.areas.starter", 1).is_err());
    }
    #[test]
    fn export_is_complete_refuses_replacement_and_keeps_inputs() {
        let root = std::env::temp_dir().join(format!("theia-area-export-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("quests.json");
        let output = root.join("areas.json");
        fs::write(&source, QUESTS).unwrap();
        publish(&zone(), &source, &output, "theia.areas.starter", 1).unwrap();
        let original = fs::read(&output).unwrap();
        assert!(publish(&zone(), &source, &output, "theia.areas.starter", 2).is_err());
        assert_eq!(fs::read(&output).unwrap(), original);
        assert_eq!(fs::read(&source).unwrap(), QUESTS);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
