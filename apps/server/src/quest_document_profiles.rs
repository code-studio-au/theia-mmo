//! New pure read kind must not acquire unbound wire/storage/World admission.
use std::sync::Arc;
use theia_core::quests::catalog::Catalog;
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
fn source() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"schema":4,"id":"quests","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Read","description":"Fixture","prerequisites":[],
        "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
            "kind":"read_document","id":"read","label":"Read","document":"note"}]}]}]})).unwrap()
}
#[test]
fn read_metadata_does_not_open_unbound_runtime_constructors_or_raw_world_preparation() {
    let source = source();
    let catalog = Arc::new(Catalog::from_json(&source).unwrap());
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let stored_xp = theia_persistence::ProgressionCatalog::from_json(XP).unwrap();
    let areas = include_bytes!("../../../content/quests/development-quest-areas.json");
    for error in [
        theia_network::quest_journal::Profile::from_json(&source, &xp)
            .unwrap_err()
            .to_string(),
        theia_network::quest_journal::Profile::from_json_with_areas(&source, areas, &xp)
            .unwrap_err()
            .to_string(),
        theia_persistence::QuestJournalCatalog::from_json(&source, &stored_xp)
            .unwrap_err()
            .to_string(),
        theia_persistence::QuestJournalCatalog::from_json_with_areas(&source, areas, &stored_xp)
            .unwrap_err()
            .to_string(),
    ] {
        assert!(error.contains("Document-read"), "{error}");
    }
    let mut world = theia_network::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
    world
        .spawn_development_guide(
            Arc::new(
                theia_network::dialogue::Profile::from_json(include_bytes!(
                    "../../../content/quests/development-journal-dialogue.json"
                ))
                .unwrap(),
            ),
            [8004., 8000.],
        )
        .unwrap();
    let identity = world.identity();
    assert!(
        world
            .validate_quest_journal_references(&catalog)
            .unwrap_err()
            .to_string()
            .contains("Document-read")
    );
    assert!(world.prepare_quest_journal(catalog).is_err());
    assert_eq!(world.identity(), identity);
    assert!(world.quest_journal_profile().is_none());
}
