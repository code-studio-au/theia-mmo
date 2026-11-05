//! Metadata-bound fixtures only; not saved ownership or authorized item opens.
use crate::*;
use theia_core::{
    content::ContentId,
    progression::{KillEvent, KillId},
    quests::journal::Journal,
};
pub(crate) fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
pub(crate) struct Profiles {
    pub(crate) items: InventoryCatalog,
    pub(crate) xp: ProgressionCatalog,
    pub(crate) journal: QuestJournalCatalog,
    pub(crate) conditional: RewardCatalog,
    pub(crate) ordinary: RewardCatalog,
    pub(crate) vendor: VendorCatalog,
}
pub(crate) fn profiles() -> Profiles {
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-loot-items.json"
    ))
    .unwrap();
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let document=serde_json::to_vec(&serde_json::json!({"schema":1,"id":"documents","revision":1,"item_catalog":"theia.dev.equipment-items","documents":[{"id":"note","item":"theia.dev.training-stone","title":"Note","pages":["Text"]}]})).unwrap();
    let docs = DocumentCatalog::from_json(&document, &items).unwrap();
    let source=serde_json::to_vec(&serde_json::json!({"schema":4,"id":"quests","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Read","description":"Fixture","prerequisites":[],"giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[
            {"kind":"read_document","id":"read","label":"Read","document":"note"},
            {"kind":"possess_item","id":"bring","label":"Bring","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":true},
            {"kind":"reach_area","id":"reach","label":"Reach","area":"theia.dev.area.western-overlook"}
        ]}]}]})).unwrap();
    let journal = QuestJournalCatalog::from_json_with_documents(
        &source,
        Some(include_bytes!(
            "../../../../content/quests/development-quest-areas.json"
        )),
        &xp,
        &docs,
    )
    .unwrap();
    let rewards = include_bytes!("../../../../content/gameplay/development-rewards.json");
    let ordinary = RewardCatalog::from_json(rewards, &items).unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(rewards).unwrap();
    raw["schema"] = serde_json::json!(4);
    raw["rewards"][0]["scope"] = serde_json::json!("per_creature_life");
    raw["rewards"][0]["quest_required"] = serde_json::json!(true);
    raw["rewards"][0]["grants"][0]["item"] = serde_json::json!("theia.dev.training-stone");
    let conditional = RewardCatalog::from_json_with_quest_journal(
        &serde_json::to_vec(&raw).unwrap(),
        &items,
        &journal,
    )
    .unwrap();
    let vendor = VendorCatalog::from_json(
        include_bytes!("../../../../content/gameplay/development-vendor.json"),
        &items,
    )
    .unwrap();
    Profiles {
        items,
        xp,
        journal,
        conditional,
        ordinary,
        vendor,
    }
}
pub(crate) fn binding() -> Binding {
    Binding {
        owner: OwnerId::new(uuid::Uuid::from_bytes([1; 16])).unwrap(),
        character: CharacterId::new(uuid::Uuid::from_bytes([2; 16])).unwrap(),
    }
}
pub(crate) fn event() -> KillEvent {
    KillEvent {
        id: KillId::from_bytes([4; 16]).unwrap(),
        creature: id("theia.dev.training-creature"),
    }
}
pub(crate) fn command() -> QuestJournalCommand {
    QuestJournalCommand {
        request_id: std::num::NonZeroU128::new(1).unwrap(),
        quest: id("quest"),
        npc: id("theia.dev.guide"),
        expected_revision: 0,
        action: QuestJournalAction::Accept,
    }
}
pub(crate) fn area(journal: &QuestJournalCatalog) -> theia_core::quests::journal::ReachedAreas {
    Journal::default()
        .propose_accept(journal.catalog(), &id("quest"), &id("theia.dev.guide"))
        .unwrap()
        .journal
        .prepare_reached_areas(
            journal.catalog(),
            &std::collections::BTreeSet::from([id("theia.dev.area.western-overlook")]),
        )
        .unwrap()
}
pub(crate) fn refused<T>(result: Result<T>) {
    assert!(matches!(
        result,
        Err(Error::Invalid("Document journal runtime is not installed"))
    ));
}
