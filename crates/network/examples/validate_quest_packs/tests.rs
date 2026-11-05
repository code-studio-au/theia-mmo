use super::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "theia-quest-packs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let f = Self(base);
        fs::create_dir_all(f.root().join("quests")).unwrap();
        let raw: Value = serde_json::from_slice(include_bytes!(
            "../../../../content/quests/development-journal.json"
        ))
        .unwrap();
        for (n, name) in ["intro", "follow-up"].into_iter().enumerate() {
            let pack = json!({"schema":1,"id":format!("theia.test.pack.{name}"),"revision":1,"quests":[raw["quests"][n].clone()]});
            fs::write(f.pack(name), serde_json::to_vec(&pack).unwrap()).unwrap();
        }
        fs::write(f.index(),serde_json::to_vec(&json!({"schema":1,"id":"theia.test.quest-index","revision":1,"catalogues":["quests/intro.json","quests/follow-up.json"]})).unwrap()).unwrap();
        f
    }
    fn root(&self) -> PathBuf {
        self.0.join("content")
    }
    fn index(&self) -> PathBuf {
        self.root().join("index.json")
    }
    fn pack(&self, name: &str) -> PathBuf {
        self.root().join("quests").join(format!("{name}.json"))
    }
    fn edit(&self, path: &Path, change: impl FnOnce(&mut Value)) {
        let mut raw: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        change(&mut raw);
        fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn actual_files_link_split_canonical_content_and_report_exact_read_only_hashes() {
    let f = Fixture::new();
    let before = fs::read(f.pack("follow-up")).unwrap();
    let first = validate(&f.index()).unwrap();
    assert!(first.validated);
    assert!(!first.runtime_pack_loading);
    assert!(!first.world_references_validated);
    assert_eq!((first.quests, first.catalogues.len()), (2, 2));
    assert_eq!(first.catalogues[1].path, "quests/follow-up.json");
    assert_eq!(first.catalogues[1].sha256, hex(&sha256(&before)));
    assert_eq!(first, validate(&f.index()).unwrap());
    assert_eq!(fs::read(f.pack("follow-up")).unwrap(), before);
    let mut changed = before;
    changed.push(b' ');
    fs::write(f.pack("follow-up"), changed).unwrap();
    let second = validate(&f.index()).unwrap();
    assert_eq!(first.index_sha256, second.index_sha256);
    assert_ne!(
        first.validation_registry_sha256,
        second.validation_registry_sha256
    );
    let mut changed = fs::read(f.index()).unwrap();
    changed.push(b' ');
    fs::write(f.index(), changed).unwrap();
    assert_ne!(
        second.index_sha256,
        validate(&f.index()).unwrap().index_sha256
    );
}
#[test]
fn index_rejects_planning_fields_unsupported_versions_nulls_and_counts() {
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("id", json!("H1-01")),
        ("planning", json!(true)),
        ("catalogues", Value::Null),
        ("catalogues", json!([])),
        (
            "catalogues",
            json!(vec!["quests/intro.json"; MAX_CATALOGUES + 1]),
        ),
    ] {
        let f = Fixture::new();
        f.edit(&f.index(), |v| v[field] = value);
        assert!(validate(&f.index()).is_err(), "{field}");
    }
}
#[test]
fn existing_external_and_normalized_alias_paths_refuse_before_loading() {
    for value in [
        "../outside.json",
        "quests/../quests/intro.json",
        "quests/./intro.json",
        "quests//intro.json",
        "quests\\intro.json",
        "quests/intro.json?edit",
        "./quests/intro.json",
    ] {
        let f = Fixture::new();
        fs::copy(f.pack("intro"), f.0.join("outside.json")).unwrap();
        f.edit(&f.index(), |v| v["catalogues"][0] = json!(value));
        assert!(validate(&f.index()).is_err(), "{value}");
    }
    let f = Fixture::new();
    let external = f.0.join("outside.json");
    fs::copy(f.pack("intro"), &external).unwrap();
    f.edit(&f.index(), |v| {
        v["catalogues"][0] = json!(external.to_str().unwrap())
    });
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    f.edit(&f.index(), |v| {
        v["catalogues"][1] = v["catalogues"][0].clone()
    });
    assert!(validate(&f.index()).is_err());
}
#[test]
fn bounded_inputs_refuse_missing_nonregular_and_oversized_sources() {
    let f = Fixture::new();
    fs::remove_file(f.pack("intro")).unwrap();
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    fs::remove_file(f.pack("intro")).unwrap();
    fs::create_dir(f.pack("intro")).unwrap();
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    fs::write(f.pack("intro"), vec![b' '; MAX_CATALOG_BYTES + 1]).unwrap();
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    fs::write(f.index(), vec![b' '; MAX_INDEX_BYTES + 1]).unwrap();
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    f.edit(&f.index(), |v| {
        v["catalogues"][0] = json!("a".repeat(MAX_PATH_BYTES) + ".json")
    });
    assert!(validate(&f.index()).is_err());
}
#[cfg(unix)]
#[test]
fn symlinks_cannot_escape_the_index_root_or_duplicate_a_source() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let external = f.0.join("outside.json");
    fs::copy(f.pack("follow-up"), &external).unwrap();
    fs::remove_file(f.pack("follow-up")).unwrap();
    symlink(external, f.pack("follow-up")).unwrap();
    assert!(validate(&f.index()).is_err());
    let f = Fixture::new();
    fs::remove_file(f.pack("follow-up")).unwrap();
    symlink(f.pack("intro"), f.pack("follow-up")).unwrap();
    assert!(validate(&f.index()).is_err());
}
#[test]
fn invalid_prerequisites_report_file_and_ids_without_echoing_narrative() {
    let f = Fixture::new();
    f.edit(&f.pack("follow-up"), |v| {
        v["quests"][0]["prerequisites"] = json!(["theia.test.absent"])
    });
    let error = validate(&f.index()).unwrap_err().to_string();
    assert!(error.contains("quests/follow-up.json"));
    assert!(error.contains("theia.dev.quest-follow-up"));
    assert!(error.contains("theia.test.absent"));
    let f = Fixture::new();
    f.edit(&f.pack("intro"), |v| {
        v["quests"][0]["script"] = json!("private prose not echoed")
    });
    let error = validate(&f.index()).unwrap_err().to_string();
    assert!(error.contains("quests/intro.json"));
    assert!(!error.contains("private prose"));
    let f = Fixture::new();
    f.edit(&f.pack("intro"), |v| {
        v["quests"][0]["description"] = json!("Private narrative stays in authored data.")
    });
    let report = serde_json::to_string(&validate(&f.index()).unwrap()).unwrap();
    assert!(!report.contains("Private narrative"));
    assert!(report.len() <= MAX_REPORT_BYTES);
}

#[test]
fn duplicate_owner_and_cycle_errors_name_the_relevant_files() {
    let f = Fixture::new();
    f.edit(&f.pack("follow-up"), |v| {
        v["id"] = json!("theia.test.pack.intro")
    });
    let error = validate(&f.index()).unwrap_err().to_string();
    for required in [
        "quests/intro.json",
        "quests/follow-up.json",
        "theia.test.pack.intro",
    ] {
        assert!(error.contains(required), "{error}");
    }
    let f = Fixture::new();
    f.edit(&f.pack("intro"), |v| {
        v["quests"][0]["prerequisites"] = json!(["theia.dev.quest-follow-up"])
    });
    let error = validate(&f.index()).unwrap_err().to_string();
    assert!(error.contains("cycle"), "{error}");
    assert!(error.contains("quests/follow-up.json"), "{error}");
}

fn make_selection(f: &Fixture) -> PathBuf {
    let path = f.root().join("selection.json");
    fs::write(&path,serde_json::to_vec(&json!({"schema":1,"id":"theia.test.compiled","revision":3,"quests":["theia.dev.quest-follow-up"]})).unwrap()).unwrap();
    path
}
#[test]
fn compile_mode_produces_a_closed_catalogue_with_bound_source_selection_and_output_hashes() {
    use theia_core::quests::catalog::Catalog;
    let f = Fixture::new();
    let selection = make_selection(&f);
    let original = fs::read(f.pack("follow-up")).unwrap();
    let result = compile(&f.index(), &selection).unwrap();
    let expected = Catalog::from_json(include_bytes!(
        "../../../../content/quests/development-journal.json"
    ))
    .unwrap();
    let output = result.catalogue.catalogue();
    assert_eq!(output.quests().count(), 2);
    for quest in expected.quests() {
        assert_eq!(output.quest(quest.id()), Some(quest));
    }
    assert_eq!(output.revision(), 3);
    assert_eq!(result.summary.included_quests, 2);
    assert_eq!(
        result.summary.source_registry_sha256,
        validate(&f.index()).unwrap().validation_registry_sha256
    );
    assert_eq!(
        result.summary.catalogue_sha256,
        hex(&sha256(result.catalogue.as_bytes()))
    );
    assert_eq!(
        result.summary.selection_sha256,
        hex(&sha256(&fs::read(&selection).unwrap()))
    );
    assert!(result.summary.closed_catalogue_validated);
    assert!(!result.summary.world_references_validated);
    assert_eq!(fs::read(f.pack("follow-up")).unwrap(), original);
}
#[test]
fn semantic_source_whitespace_changes_provenance_but_not_the_deterministic_compiled_catalogue() {
    let f = Fixture::new();
    let selection = make_selection(&f);
    let first = compile(&f.index(), &selection).unwrap();
    let mut changed = fs::read(f.pack("follow-up")).unwrap();
    changed.push(b' ');
    fs::write(f.pack("follow-up"), changed).unwrap();
    let second = compile(&f.index(), &selection).unwrap();
    assert_eq!(first.catalogue.as_bytes(), second.catalogue.as_bytes());
    assert_ne!(
        first.summary.source_registry_sha256,
        second.summary.source_registry_sha256
    );
}
#[test]
fn compile_selection_refuses_unsupported_fields_bad_ids_duplicates_and_unknown_quests() {
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("id", json!("../bad")),
        ("script", json!("private prose not echoed")),
        ("quests", json!([])),
        ("quests", json!(["bad ID"])),
        ("quests", json!(["theia.test.missing"])),
        (
            "quests",
            json!(["theia.dev.quest-follow-up", "theia.dev.quest-follow-up"]),
        ),
        (
            "quests",
            json!(vec!["theia.dev.quest-follow-up"; MAX_QUESTS + 1]),
        ),
    ] {
        let f = Fixture::new();
        let selection = make_selection(&f);
        f.edit(&selection, |v| v[field] = value);
        let error = compile(&f.index(), &selection).unwrap_err().to_string();
        assert!(!error.contains("private prose"), "{error}");
    }
}
#[test]
fn compile_selection_files_are_bounded_regular_and_source_graph_must_still_validate() {
    let f = Fixture::new();
    let selection = make_selection(&f);
    fs::write(&selection, vec![b' '; MAX_SELECTION_BYTES + 1]).unwrap();
    assert!(compile(&f.index(), &selection).is_err());
    let f = Fixture::new();
    let selection = make_selection(&f);
    fs::remove_file(&selection).unwrap();
    fs::create_dir(&selection).unwrap();
    assert!(compile(&f.index(), &selection).is_err());
    let f = Fixture::new();
    let selection = make_selection(&f);
    f.edit(&f.pack("follow-up"), |v| {
        v["quests"][0]["prerequisites"] = json!(["theia.test.missing"])
    });
    let error = compile(&f.index(), &selection).unwrap_err().to_string();
    assert!(error.contains("quests/follow-up.json"));
    assert!(error.contains("theia.test.missing"));
}
