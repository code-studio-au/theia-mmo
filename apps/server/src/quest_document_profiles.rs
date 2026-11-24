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

const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-loot-items.json");
const AREAS: &[u8] = include_bytes!("../../../content/quests/development-quest-areas.json");
fn documents() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"schema":1,"id":"documents","revision":1,"item_catalog":"theia.dev.equipment-items","documents":[{"id":"note","item":"theia.dev.training-stone","title":"A note","pages":["Take the road."]}]})).unwrap()
}
fn source_with_area() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&source()).unwrap();
    let areas: serde_json::Value = serde_json::from_slice(AREAS).unwrap();
    value["quests"][0]["stages"][0]["objectives"].as_array_mut().unwrap().push(serde_json::json!({"kind":"reach_area","id":"reach","label":"Reach","area":areas["areas"][0]["id"]}));
    serde_json::to_vec(&value).unwrap()
}
struct Bound {
    items: theia_network::renet_runtime::inventory::Profile,
    wire_documents: theia_network::documents::Profile,
    stored_documents: theia_persistence::DocumentCatalog,
    wire: theia_network::quest_journal::Profile,
    stored: theia_persistence::QuestJournalCatalog,
}
fn bind(items: &[u8], quest: &[u8], xp: &[u8], documents: &[u8], areas: Option<&[u8]>) -> Bound {
    let wi = theia_network::renet_runtime::inventory::Profile::from_json(items).unwrap();
    let si = theia_persistence::InventoryCatalog::from_json(items).unwrap();
    let wd = theia_network::documents::Profile::from_json(documents, &wi).unwrap();
    let sd = theia_persistence::DocumentCatalog::from_json(documents, &si).unwrap();
    let wj = theia_network::quest_journal::Profile::from_json_with_documents(
        quest,
        areas,
        &theia_network::progression_wire::Profile::from_json(xp).unwrap(),
        &wd,
    )
    .unwrap();
    let sj = theia_persistence::QuestJournalCatalog::from_json_with_documents(
        quest,
        areas,
        &theia_persistence::ProgressionCatalog::from_json(xp).unwrap(),
        &sd,
    )
    .unwrap();
    assert_eq!(wd.hash(), sd.hash());
    assert_eq!(wj.hash(), sj.hash());
    assert_eq!(wj.inventory_hash(), Some(wi.hash()));
    assert_eq!(sj.inventory().unwrap().hash(), wi.hash());
    Bound {
        items: wi,
        wire_documents: wd,
        stored_documents: sd,
        wire: wj,
        stored: sj,
    }
}
#[test]
fn exact_document_binding_agrees_and_read_only_journals_still_require_inventory() {
    let source = source_with_area();
    let documents = documents();
    let base = bind(ITEMS, &source, XP, &documents, Some(AREAS));
    assert!(
        !base
            .wire
            .catalog()
            .quests()
            .any(|q| q.has_possession_objectives())
    );
    assert_eq!(
        base.wire.documents().unwrap().hash(),
        base.wire_documents.hash()
    );
    assert_eq!(
        base.stored.documents().unwrap().hash(),
        base.stored_documents.hash()
    );
    let mut hashes = std::collections::BTreeSet::from([base.wire.hash()]);
    for index in 0..5 {
        let mut inputs = [
            ITEMS.to_vec(),
            source.clone(),
            XP.to_vec(),
            documents.clone(),
            AREAS.to_vec(),
        ];
        inputs[index].push(b' ');
        let changed = bind(
            &inputs[0],
            &inputs[1],
            &inputs[2],
            &inputs[3],
            Some(&inputs[4]),
        );
        assert!(
            hashes.insert(changed.wire.hash()),
            "Input {index} did not bind its exact bytes"
        );
        assert_eq!(
            changed.wire_documents.hash() == base.wire_documents.hash(),
            !matches!(index, 0 | 3)
        );
    }
    assert!(
        theia_network::quest_journal::Profile::from_json_with_documents(
            &source,
            None,
            &theia_network::progression_wire::Profile::from_json(XP).unwrap(),
            &base.wire_documents
        )
        .is_err()
    );
    assert!(
        theia_persistence::QuestJournalCatalog::from_json_with_documents(
            &source,
            None,
            &theia_persistence::ProgressionCatalog::from_json(XP).unwrap(),
            &base.stored_documents
        )
        .is_err()
    );
    let without = bind(ITEMS, &self::source(), XP, &documents, None);
    assert_ne!(without.wire.hash(), base.wire.hash());
    assert_eq!(without.wire_documents.hash(), base.wire_documents.hash());
    assert!(base.wire.clone().with_inventory(&base.items).is_err());
    assert!(
        base.stored
            .clone()
            .with_inventory(base.stored_documents.inventory())
            .is_err()
    );
}
#[test]
fn missing_documents_unused_bindings_and_area_definition_aliases_refuse() {
    let source = source();
    let base = bind(ITEMS, &source, XP, &documents(), None);
    let mut raw: serde_json::Value = serde_json::from_slice(&source).unwrap();
    for replacement in [
        serde_json::json!({"kind":"read_document","id":"read","label":"Read","document":"unknown"}),
        serde_json::json!({"kind":"kill","id":"read","label":"Kill","creature":"wolf","required":1}),
    ] {
        raw["quests"][0]["stages"][0]["objectives"][0] = replacement;
        let bytes = serde_json::to_vec(&raw).unwrap();
        assert!(
            theia_network::quest_journal::Profile::from_json_with_documents(
                &bytes,
                None,
                &theia_network::progression_wire::Profile::from_json(XP).unwrap(),
                &base.wire_documents
            )
            .is_err()
        );
        assert!(
            theia_persistence::QuestJournalCatalog::from_json_with_documents(
                &bytes,
                None,
                &theia_persistence::ProgressionCatalog::from_json(XP).unwrap(),
                &base.stored_documents
            )
            .is_err()
        );
    }
    let mut areas: serde_json::Value = serde_json::from_slice(AREAS).unwrap();
    let mut extra = areas["areas"][0].clone();
    extra["id"] = serde_json::json!("unused-area");
    areas["areas"].as_array_mut().unwrap().push(extra);
    let area_bytes = serde_json::to_vec(&areas).unwrap();
    for unused in [false, true] {
        let mut raw: serde_json::Value = serde_json::from_slice(&documents()).unwrap();
        if unused {
            raw["documents"].as_array_mut().unwrap().push(serde_json::json!({"id":"unused-area","item":"theia.dev.training-token","title":"Unused","pages":["Text"]}));
        } else {
            raw["id"] = areas["id"].clone();
        }
        let bytes = serde_json::to_vec(&raw).unwrap();
        let wd = theia_network::documents::Profile::from_json(&bytes, &base.items).unwrap();
        let sd = theia_persistence::DocumentCatalog::from_json(
            &bytes,
            base.stored_documents.inventory(),
        )
        .unwrap();
        assert!(
            theia_network::quest_journal::Profile::from_json_with_documents(
                &source_with_area(),
                Some(&area_bytes),
                &theia_network::progression_wire::Profile::from_json(XP).unwrap(),
                &wd
            )
            .is_err()
        );
        assert!(
            theia_persistence::QuestJournalCatalog::from_json_with_documents(
                &source_with_area(),
                Some(&area_bytes),
                &theia_persistence::ProgressionCatalog::from_json(XP).unwrap(),
                &sd
            )
            .is_err()
        );
    }
}
#[tokio::test]
async fn exact_metadata_does_not_admit_legacy_owner_world_or_gameplay_client() {
    use theia_network::renet_runtime as net;
    use tokio::sync::{mpsc, watch};
    let bound = bind(ITEMS, &source(), XP, &documents(), None);
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (_, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let mut gate = crate::admissions::GameAdmissions::new(
        Arc::new(std::sync::Mutex::new(theia_auth::Sessions::default())),
        issuer,
        [1; 32],
    );
    assert!(
        crate::admissions::GameAdmissions::with_quest_journal(&mut gate, bound.stored)
            .unwrap_err()
            .to_string()
            .contains("Document journal owner")
    );
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
            .enable_item_quest_journal(
                Arc::new(bound.wire.clone()),
                Arc::new(theia_network::progression_wire::Profile::from_json(XP).unwrap()),
                &bound.items
            )
            .is_err()
    );
    assert_eq!(identity, world.identity());
    let (_, input) = watch::channel(theia_network::wire::Intent::default());
    let (state, _) = watch::channel(theia_network::client::State::default());
    let (_, stop) = watch::channel(false);
    let (_, ready) = watch::channel(None);
    let (_, actions) = mpsc::channel(4);
    let (_, commands) = mpsc::channel(1);
    let error = net::run_client_gameplay(
        "127.0.0.1:1".parse().unwrap(),
        &[],
        input,
        state,
        stop,
        [1; 32],
        net::GameplayClient {
            party: None,
            chat: None,
            avatar_life: None,
            public_attacks: None,
            classic_source: None,
            dodge: None,
            player_life: None,
            lifecycle: None,
            dialogue: None,
            choices: None,
            combat: None,
            quest: None,
            progression: None,
            statuses: None,
            commands: None,
            ready,
            journal: Some(net::quest_journal::Client {
                profile: Arc::new(bound.wire),
                actions,
            }),
            inventory: Some(net::inventory::Client {
                profile: bound.items,
                commands,
                pickups: None,
            }),
        },
    )
    .await
    .unwrap_err()
    .to_string();
    assert!(error.contains("Document intents require"), "{error}");
}

#[test]
fn explicit_document_world_installation_requires_exact_items_notes_and_inactive_world() {
    let bound = bind(ITEMS, &source(), XP, &documents(), None);
    let items = bound.items.clone().with_documents(&documents()).unwrap();
    let xp = Arc::new(theia_network::progression_wire::Profile::from_json(XP).unwrap());
    let setup = || {
        let mut world =
            theia_network::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
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
        world
    };
    let mut world = setup();
    let before = world.identity();
    assert!(
        world
            .enable_document_quest_journal(Arc::new(bound.wire.clone()), xp.clone(), &bound.items)
            .is_err()
    );
    assert!(
        world
            .enable_item_quest_journal(Arc::new(bound.wire.clone()), xp.clone(), &items)
            .is_err()
    );
    let mut changed = documents();
    changed.push(b' ');
    let foreign = bound.items.clone().with_documents(&changed).unwrap();
    assert!(
        world
            .enable_document_quest_journal(Arc::new(bound.wire.clone()), xp.clone(), &foreign)
            .is_err()
    );
    assert_eq!(world.identity(), before);
    let mut alias: serde_json::Value = serde_json::from_slice(&documents()).unwrap();
    alias["documents"].as_array_mut().unwrap().push(serde_json::json!({"id":"theia.dev.interaction.road","item":"theia.dev.training-token","title":"Unused note","pages":["Text"]}));
    let alias_bytes = serde_json::to_vec(&alias).unwrap();
    let alias_bound = bind(ITEMS, &source(), XP, &alias_bytes, None);
    let alias_items = bound.items.clone().with_documents(&alias_bytes).unwrap();
    assert!(
        world
            .enable_document_quest_journal(Arc::new(alias_bound.wire), xp.clone(), &alias_items)
            .unwrap_err()
            .to_string()
            .contains("alias installed World")
    );
    assert_eq!(world.identity(), before);
    world
        .enable_document_quest_journal(Arc::new(bound.wire.clone()), xp.clone(), &items)
        .unwrap();
    assert_ne!(world.identity(), before);
    assert_eq!(
        world.quest_journal_profile().unwrap().hash(),
        bound.wire.hash()
    );
    assert!(
        world
            .enable_document_quest_journal(Arc::new(bound.wire.clone()), xp.clone(), &items)
            .is_err()
    );
    let mut active = setup();
    assert!(active.join().is_some());
    let before = active.identity();
    assert!(
        active
            .enable_document_quest_journal(Arc::new(bound.wire), xp, &items)
            .is_err()
    );
    assert_eq!(active.identity(), before);
}

#[test]
fn normal_document_manifest_sources_bind_identically_across_storage_and_transport() {
    let items = include_bytes!("../../../content/gameplay/development-document-items.json");
    let quests = include_bytes!("../../../content/quests/development-journal-documents.json");
    let notes = include_bytes!("../../../content/quests/development-documents.json");
    let areas = include_bytes!("../../../content/quests/development-quest-areas.json");
    let bound = bind(items, quests, XP, notes, Some(areas));
    assert_eq!(bound.wire.hash(), bound.stored.hash());
    assert_eq!(
        bound.wire.documents().unwrap().hash(),
        bound.stored.documents().unwrap().hash()
    );
    assert_eq!(bound.wire.inventory_hash(), Some(bound.items.hash()));
    assert_eq!(bound.wire.catalog().quests().count(), 5);
}
