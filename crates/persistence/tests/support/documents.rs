//! Trusted owned-database document fixtures; no live session/NPC/UI proof.
#![allow(dead_code)]
use std::num::NonZeroU128;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Stack},
};
use theia_persistence::*;
pub const WORLD: [u8; 32] = [211; 32];
pub fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
pub fn nz(n: u128) -> NonZeroU128 {
    NonZeroU128::new(n).unwrap()
}
pub struct Profiles {
    pub items: InventoryCatalog,
    pub xp: ProgressionCatalog,
    pub documents: DocumentCatalog,
    pub journal: QuestJournalCatalog,
    pub quest_bytes: Vec<u8>,
}
pub fn profiles(possession: bool) -> Profiles {
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-loot-items.json"
    ))
    .unwrap();
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let raw=serde_json::to_vec(&serde_json::json!({"schema":1,"id":"documents","revision":1,"item_catalog":"theia.dev.equipment-items","documents":[
        {"id":"note","item":"theia.dev.training-stone","title":"A note","pages":["Take the eastern road.","The river is quiet."]},
        {"id":"other-note","item":"theia.dev.training-token","title":"Other note","pages":["Unrelated text."]}
    ]})).unwrap();
    let documents = DocumentCatalog::from_json(&raw, &items).unwrap();
    let mut quest = serde_json::json!({"schema":4,"id":"document-quests","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Read the note","description":"Owned fixture","prerequisites":[],"giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"first",
        "stages":[
            {"id":"first","next":"second","completion":{"kind":"all"},"objectives":[{"kind":"read_document","id":"read-first","label":"Read","document":"note"}]},
            {"id":"second","next":null,"completion":{"kind":"all"},"objectives":[{"kind":"read_document","id":"read-final","label":"Read again","document":"note"}]}
        ]
    }]});
    if possession {
        quest["quests"][0]["stages"][1]["objectives"].as_array_mut().unwrap().push(serde_json::json!({"kind":"possess_item","id":"bring","label":"Bring two notes","item":"theia.dev.training-stone","required":2,"consume_on_turn_in":true}));
    }
    let quest_bytes = serde_json::to_vec(&quest).unwrap();
    let journal =
        QuestJournalCatalog::from_json_with_documents(&quest_bytes, None, &xp, &documents).unwrap();
    Profiles {
        items,
        xp,
        documents,
        journal,
        quest_bytes,
    }
}
pub fn open(p: &Profiles, request: u128, revision: u64, slot: u8) -> DocumentOpenCommand {
    DocumentOpenCommand {
        request_id: nz(request),
        profile_hash: p.documents.hash(),
        document: id("note"),
        expected_revision: revision,
        slot,
    }
}
pub fn command(request: u128, action: QuestJournalAction, revision: u64) -> QuestJournalCommand {
    QuestJournalCommand {
        request_id: nz(request),
        quest: id("quest"),
        npc: id("theia.dev.guide"),
        action,
        expected_revision: revision,
    }
}
pub async fn initialize(store: &mut Store, name: &str, count: u32, p: &Profiles) -> Binding {
    let owner = store
        .resolve_owner("https://documents.theia.test", name)
        .await
        .unwrap();
    let who = store
        .create_character(
            owner,
            WORLD,
            CharacterName::parse(name).unwrap(),
            GlobalPosition::new([8002., 0., 8002.]).unwrap(),
        )
        .await
        .unwrap()
        .saved
        .binding;
    let mut slots = std::array::from_fn(|_| None);
    if count > 0 {
        slots[0] = Stack::new(p.items.catalog(), id("theia.dev.training-stone"), count);
    }
    slots[1] = Stack::new(p.items.catalog(), id("theia.dev.training-token"), 1);
    let bag = Bag::from_slots(p.items.catalog(), 0, slots).unwrap();
    store
        .ensure_inventory(who, WORLD, &p.items, &bag)
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, &p.xp).await.unwrap();
    store
        .ensure_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    who
}
pub async fn accept(store: &mut Store, who: Binding, p: &Profiles) -> QuestJournalApplied {
    store
        .command_quest_journal(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            &command(1, QuestJournalAction::Accept, 0),
        )
        .await
        .unwrap()
}
