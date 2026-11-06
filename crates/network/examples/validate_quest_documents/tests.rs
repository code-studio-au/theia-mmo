use super::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-document-authoring-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let f = Self(root);
        for (name, raw) in [
            (
                "items",
                include_bytes!("../../../../content/gameplay/development-document-items.json")
                    .as_slice(),
            ),
            (
                "documents",
                include_bytes!("../../../../content/quests/development-documents.json").as_slice(),
            ),
            (
                "quests",
                include_bytes!("../../../../content/quests/development-journal-documents.json")
                    .as_slice(),
            ),
            (
                "areas",
                include_bytes!("../../../../content/quests/development-quest-areas.json")
                    .as_slice(),
            ),
        ] {
            fs::write(f.path(name), raw).unwrap();
        }
        f
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(format!("{name}.json"))
    }
    fn inspect(&self, areas: bool) -> Result<Value> {
        inspect(
            &self.path("items"),
            &self.path("documents"),
            &self.path("quests"),
            areas.then(|| self.path("areas")).as_deref(),
        )
    }
    fn edit(&self, name: &str, edit: impl FnOnce(&mut Value)) {
        let path = self.path(name);
        let mut value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        edit(&mut value);
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn actual_files_validate_exact_hashes_without_touching_sources_or_echoing_pages() {
    let f = Fixture::new();
    let before = fs::read(f.path("documents")).unwrap();
    let first = f.inspect(true).unwrap();
    assert_eq!(first["validated"], true);
    assert_eq!(first["documents"]["sha256"], hex(&sha256(&before)));
    assert_eq!(first["world_references_validated"], false);
    assert_eq!(first["quest_journal_profile_bound"], false);
    assert_eq!(first["saved_state_admitted"], false);
    assert_eq!(first["documents"]["count"], 1);
    assert_eq!(fs::read(f.path("documents")).unwrap(), before);
    assert!(
        !serde_json::to_string(&first)
            .unwrap()
            .contains("Keep watch")
    );
    let mut changed = before;
    changed.push(b' ');
    fs::write(f.path("documents"), changed).unwrap();
    let second = f.inspect(true).unwrap();
    assert_ne!(first["documents"]["sha256"], second["documents"]["sha256"]);
    assert_ne!(
        first["documents"]["profile_sha256"],
        second["documents"]["profile_sha256"]
    );
    assert_eq!(first["quests"], second["quests"]);
}
#[test]
fn area_context_is_required_exactly_when_used_and_checked_for_namespace_aliases() {
    let f = Fixture::new();
    assert!(
        f.inspect(false)
            .unwrap_err()
            .to_string()
            .contains("AREAS.json")
    );
    // Use the actual area's catalogue ID; do not assume a naming convention.
    let area: Value = serde_json::from_slice(&fs::read(f.path("areas")).unwrap()).unwrap();
    f.edit("documents", |v| v["id"] = area["id"].clone());
    assert!(f.inspect(true).is_err());
    let f = Fixture::new();
    f.edit("quests", |v| {
        v["quests"]
            .as_array_mut()
            .unwrap()
            .retain(|q| q["id"] == "theia.dev.quest-roadside-orders")
    });
    assert!(f.inspect(false).unwrap()["areas"].is_null());
    assert!(
        f.inspect(true).is_err(),
        "Unused area context must not look validated"
    );
}
#[test]
fn dangling_documents_unknown_items_and_missing_delivery_items_refuse() {
    let f = Fixture::new();
    f.edit("documents", |v| {
        v["documents"][0]["item"] = json!("theia.missing.item")
    });
    assert!(f.inspect(true).is_err());
    let f = Fixture::new();
    f.edit("quests", |v| {
        for q in v["quests"].as_array_mut().unwrap() {
            for stage in q["stages"].as_array_mut().unwrap() {
                for o in stage["objectives"].as_array_mut().unwrap() {
                    if o["kind"] == "read_document" {
                        o["document"] = json!("theia.missing.document");
                    }
                }
            }
        }
    });
    assert!(f.inspect(true).is_err());
    let f = Fixture::new();
    f.edit("quests", |v| {
        for q in v["quests"].as_array_mut().unwrap() {
            for stage in q["stages"].as_array_mut().unwrap() {
                for o in stage["objectives"].as_array_mut().unwrap() {
                    if o["kind"] == "possess_item" {
                        o["item"] = json!("theia.missing.delivery-item");
                    }
                }
            }
        }
    });
    assert!(f.inspect(true).is_err());
}
#[test]
fn each_file_role_enforces_its_budget_and_nonregular_inputs_fail_before_reading() {
    for (name, limit) in [
        ("items", inventory::MAX_CATALOG_BYTES),
        ("documents", documents::MAX_CATALOG_BYTES),
        ("quests", catalog::MAX_CATALOG_BYTES),
        ("areas", areas::MAX_CATALOG_BYTES),
    ] {
        let f = Fixture::new();
        fs::write(f.path(name), vec![b' '; limit + 1]).unwrap();
        assert!(f.inspect(true).is_err());
    }
    let f = Fixture::new();
    fs::remove_file(f.path("documents")).unwrap();
    fs::create_dir(f.path("documents")).unwrap();
    assert!(f.inspect(true).is_err());
    let f = Fixture::new();
    fs::remove_file(f.path("documents")).unwrap();
    assert!(f.inspect(true).is_err());
}
#[cfg(unix)]
#[test]
fn symlinked_document_input_is_refused() {
    let f = Fixture::new();
    fs::rename(f.path("documents"), f.path("target")).unwrap();
    std::os::unix::fs::symlink(f.path("target"), f.path("documents")).unwrap();
    assert!(f.inspect(true).is_err());
}
