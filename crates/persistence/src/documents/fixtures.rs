//! Metadata-bound fixtures only; not saved ownership or authorized item opens.
use crate::*;
use theia_core::content::ContentId;
pub(crate) fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
pub(crate) struct Profiles {
    pub(crate) journal: QuestJournalCatalog,
    pub(crate) legacy: QuestJournalCatalog,
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
    let legacy = QuestJournalCatalog::from_json(
        include_bytes!("../../../../content/quests/development-journal.json"),
        &xp,
    )
    .unwrap();
    Profiles { journal, legacy }
}
pub(crate) fn binding() -> Binding {
    Binding {
        owner: OwnerId::new(uuid::Uuid::from_bytes([1; 16])).unwrap(),
        character: CharacterId::new(uuid::Uuid::from_bytes([2; 16])).unwrap(),
    }
}
pub(crate) fn open(journal: &QuestJournalCatalog) -> DocumentOpenCommand {
    DocumentOpenCommand {
        request_id: std::num::NonZeroU128::new(1).unwrap(),
        profile_hash: journal.documents().unwrap().hash(),
        document: id("note"),
        expected_revision: 0,
        slot: 0,
    }
}
pub(crate) fn invalid(journal: &QuestJournalCatalog) -> Vec<DocumentOpenCommand> {
    let valid = open(journal);
    let mut wrong = valid.clone();
    wrong.profile_hash[0] ^= 1;
    let mut unknown = valid.clone();
    unknown.document = id("unknown");
    let mut slot = valid.clone();
    slot.slot = 16;
    let mut revision = valid;
    revision.expected_revision = u64::MAX;
    vec![wrong, unknown, slot, revision]
}
