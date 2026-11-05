use super::*;
use serde_json::json;
const SOURCE: &[u8] = include_bytes!("../../../../content/quests/development-journal.json");
fn raw() -> Value {
    serde_json::from_slice(SOURCE).unwrap()
}
fn input(value: &Value) -> Input {
    Input::parse(&serde_json::to_vec(value).unwrap()).unwrap()
}
#[test]
fn exact_and_reformatted_catalogues_have_distinct_bytes_but_identical_definitions() {
    let old = Input::parse(SOURCE).unwrap();
    let same = compare(&old, &old);
    assert!(same.exact_bytes_unchanged);
    assert!(same.quests.iter().all(|q| q.change == "unchanged"));
    let new = input(&raw());
    let report = compare(&old, &new);
    assert!(!report.exact_bytes_unchanged);
    assert_eq!(
        report.catalogue_revision_review,
        Some("digest_changed_without_revision_bump")
    );
    assert_eq!(report.quest_version_review_issues, 0);
    assert!(
        report
            .quests
            .iter()
            .all(|q| q.previous_gameplay_sha256 == q.candidate_gameplay_sha256)
    );
    assert_eq!(report.saved_state_compatibility, "not_assessed");
    assert!(!report.automatic_migration && !report.world_reference_validation);
}
#[test]
fn prose_and_labels_are_presentation_changes_without_narrative_in_report() {
    let old = input(&raw());
    let mut new = raw();
    new["quests"][0]["title"] = json!("Private narrative title");
    new["quests"][0]["description"] = json!("Private narrative description");
    new["quests"][0]["stages"][0]["objectives"][0]["label"] = json!("Private objective label");
    let candidate = input(&new);
    let report = compare(&old, &candidate);
    let q = report
        .quests
        .iter()
        .find(|q| q.id == new["quests"][0]["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(q.change, "presentation");
    assert!(q.gameplay_fields.is_empty());
    assert!(q.presentation_changed);
    assert_eq!(
        q.version_review,
        Some("definition_changed_without_version_bump")
    );
    assert_eq!(q.previous_gameplay_sha256, q.candidate_gameplay_sha256);
    let bytes = report_bytes(&old, &candidate).unwrap();
    assert!(!String::from_utf8(bytes).unwrap().contains("Private"));
}
#[test]
fn changed_gameplay_and_version_regressions_are_advisory_not_compatibility() {
    let mut previous = raw();
    previous["quests"][0]["version"] = json!(2);
    let old = input(&previous);
    let mut candidate = previous.clone();
    candidate["quests"][0]["experience"] = json!(30);
    let new = input(&candidate);
    let report = compare(&old, &new);
    let q = report
        .quests
        .iter()
        .find(|q| q.change == "gameplay")
        .unwrap();
    assert_eq!(q.gameplay_fields, vec!["experience"]);
    assert_eq!(
        q.version_review,
        Some("definition_changed_without_version_bump")
    );
    assert_ne!(q.previous_gameplay_sha256, q.candidate_gameplay_sha256);
    candidate["quests"][0]["version"] = json!(3);
    candidate["revision"] = json!(2);
    let new = input(&candidate);
    assert_eq!(compare(&old, &new).quest_version_review_issues, 0);
    assert!(compare(&old, &new).catalogue_revision_review.is_none());
    candidate["quests"][0]["version"] = json!(1);
    assert_eq!(
        compare(&old, &input(&candidate)).quest_version_review_issues,
        1
    );
    assert_eq!(
        compare(&old, &input(&candidate))
            .quests
            .iter()
            .find_map(|q| q.version_review),
        Some("version_regressed")
    );
}
#[test]
fn unordered_identity_arrays_and_missing_terminal_next_do_not_change_gameplay() {
    let old = input(&raw());
    let mut value = raw();
    for q in value["quests"].as_array_mut().unwrap() {
        q["prerequisites"].as_array_mut().unwrap().reverse();
        q["stages"].as_array_mut().unwrap().reverse();
        for s in q["stages"].as_array_mut().unwrap() {
            s["objectives"].as_array_mut().unwrap().reverse();
            if s["next"].is_null() {
                s.as_object_mut().unwrap().remove("next");
            }
        }
    }
    let report = compare(&old, &input(&value));
    assert!(report.quests.iter().all(|q| q.gameplay_fields.is_empty()
        && q.previous_gameplay_sha256 == q.candidate_gameplay_sha256
        && q.version_review.is_none()));
}
#[test]
fn added_removed_ids_and_version_only_changes_are_distinct_and_sorted() {
    let mut value = raw();
    let old = input(&value);
    value["quests"][0]["version"] = json!(2);
    let report = compare(&old, &input(&value));
    assert_eq!(
        report
            .quests
            .iter()
            .filter(|q| q.change == "version")
            .count(),
        1
    );
    // Remove the dependent quest, preserving a closed prerequisite graph.
    value["quests"].as_array_mut().unwrap().pop();
    let new = input(&value);
    let report = compare(&old, &new);
    assert_eq!(
        report
            .quests
            .iter()
            .filter(|q| q.change == "removed")
            .count(),
        1
    );
    assert_eq!(
        compare(&new, &old)
            .quests
            .iter()
            .filter(|q| q.change == "added")
            .count(),
        1
    );
    assert!(report.quests.windows(2).all(|q| q[0].id < q[1].id));
}
#[test]
fn invalid_planning_or_oversized_input_cannot_be_reviewed() {
    assert!(Input::parse(b"# planning document").is_err());
    let mut value = raw();
    value["future_field"] = json!(true);
    assert!(Input::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(Input::parse(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    assert!(Input::read(Path::new(env!("CARGO_MANIFEST_DIR"))).is_err());
}
#[test]
fn maximum_union_report_is_bounded_and_deterministic() {
    fn catalogue(prefix: &str) -> Input {
        let quests:Vec<_>=(0..32).map(|n|{let prefix_id=format!("theia.test.{prefix}.{n:02}.");let id=format!("{prefix_id}{}","a".repeat(96-prefix_id.len()));json!({"id":id,"version":1,"title":"Title","description":"Description","prerequisites":[],"giver":"theia.test.guide","turn_in":"theia.test.guide","repeat_policy":"once","credit_scope":"character","experience":0,"start_stage":format!("theia.test.{prefix}.stage.{n}"),"stages":[{"id":format!("theia.test.{prefix}.stage.{n}"),"next":null,"completion":{"kind":"all"},"objectives":[{"kind":"npc_interaction","id":format!("theia.test.{prefix}.objective.{n}"),"label":"Label","npc":"theia.test.guide","interaction":"theia.test.greeting"}]}]})}).collect();
        input(&json!({"schema":1,"id":"theia.test.catalogue","revision":1,"quests":quests}))
    }
    let a = catalogue("old");
    let b = catalogue("new");
    let bytes = report_bytes(&a, &b).unwrap();
    assert_eq!(compare(&a, &b).quests.len(), 64);
    assert!(bytes.len() <= MAX_REPORT_BYTES);
    assert_eq!(bytes, report_bytes(&a, &b).unwrap());
}
#[test]
fn area_selector_is_gameplay_and_requires_version_review() {
    let mut old = raw();
    old["schema"] = json!(2);
    old["quests"][0]["stages"][0]["objectives"][0] = json!({
        "kind":"reach_area","id":"theia.dev.quest-introduction.objective.road",
        "label":"Reach","area":"theia.dev.area.first"});
    let mut new = old.clone();
    new["quests"][0]["stages"][0]["objectives"][0]["area"] = json!("theia.dev.area.second");
    let report = compare(&input(&old), &input(&new));
    let q = report
        .quests
        .iter()
        .find(|q| q.change == "gameplay")
        .unwrap();
    assert_eq!(
        q.version_review,
        Some("definition_changed_without_version_bump")
    );
    assert_ne!(q.previous_gameplay_sha256, q.candidate_gameplay_sha256);
    assert!(!report.world_reference_validation && !report.automatic_migration);
}
#[test]
fn possession_consumption_policy_is_gameplay_and_requires_version_review() {
    let mut old = raw();
    old["schema"] = json!(3);
    old["quests"][0]["stages"][1]["objectives"][0] = json!({
        "kind":"possess_item","id":"theia.dev.quest-introduction.objective.report",
        "label":"Bring","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":true});
    let mut new = old.clone();
    new["quests"][0]["stages"][1]["objectives"][0]["consume_on_turn_in"] = json!(false);
    let report = compare(&input(&old), &input(&new));
    let quest = report
        .quests
        .iter()
        .find(|q| q.change == "gameplay")
        .unwrap();
    assert_eq!(
        quest.version_review,
        Some("definition_changed_without_version_bump")
    );
    assert_ne!(
        quest.previous_gameplay_sha256,
        quest.candidate_gameplay_sha256
    );
    assert!(!report.automatic_migration && !report.world_reference_validation);
}
