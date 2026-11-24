//! Owned SQL fixtures, adapted from the existing possession/document contracts.
//! Bindings and NPC authorization here are synthetic; no live-session claim.
#![allow(dead_code)]
use serde_json::json;
use std::num::NonZeroU128;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Stack},
    quests::{catalog::EventKey, journal::Status},
};
use theia_persistence::*;
pub const WORLD: [u8; 32] = [219; 32];
pub const ITEMS: &[u8] =
    include_bytes!("../../../../content/gameplay/development-document-items.json");
pub const XP: &[u8] = include_bytes!("../../../../content/gameplay/development-progression.json");
pub fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
pub struct Profiles {
    pub items: InventoryCatalog,
    pub xp: ProgressionCatalog,
    pub journal: QuestJournalCatalog,
    pub quest_raw: Vec<u8>,
    pub reward_raw: Vec<u8>,
}
pub fn profiles(delivery: bool, bundle: bool) -> Profiles {
    let goal = if delivery {
        json!({"kind":"possess_item","id":"bring","label":"Bring three","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":true})
    } else {
        json!({"kind":"kill","id":"kill","label":"One wolf","creature":"wolf","required":1})
    };
    let q = json!({"schema":3,"id":"fixed-reward-quests","revision":1,"quests":[
        {"id":"quest","version":1,"title":"Fixed reward","description":"Owned fixture","giver":"guide","turn_in":"guide","prerequisites":[],"repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"stage","stages":[{"id":"stage","next":null,"completion":{"kind":"all"},"objectives":[goal]}]},
        {"id":"watcher","version":1,"title":"Other holdings","description":"Owned fixture","giver":"guide","turn_in":"guide","prerequisites":[],"repeat_policy":"once","credit_scope":"character","experience":0,"start_stage":"watch-stage","stages":[{"id":"watch-stage","next":null,"completion":{"kind":"all"},"objectives":[{"kind":"possess_item","id":"watch-item","label":"Keep three","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":false}]}]}
    ]});
    let mut grants = vec![json!({"item":"theia.dev.training-token","count":3})];
    if bundle {
        grants.push(json!({"item":"theia.dev.training-stone","count":1}));
    }
    let reward = json!({"schema":1,"id":"fixed-rewards","revision":1,"quest_catalog":"fixed-reward-quests","item_catalog":"theia.dev.equipment-items","rewards":[{"quest":"quest","grants":grants}]});
    from_raw(
        &serde_json::to_vec(&q).unwrap(),
        ITEMS,
        XP,
        &serde_json::to_vec(&reward).unwrap(),
    )
}
pub fn from_raw(quest: &[u8], items: &[u8], xp: &[u8], reward: &[u8]) -> Profiles {
    let items = InventoryCatalog::from_json(items).unwrap();
    let xp = ProgressionCatalog::from_json(xp).unwrap();
    let journal = QuestJournalCatalog::from_json(quest, &xp)
        .unwrap()
        .with_item_rewards(reward, &items)
        .unwrap();
    Profiles {
        items,
        xp,
        journal,
        quest_raw: quest.to_vec(),
        reward_raw: reward.to_vec(),
    }
}
pub fn command(n: u128, action: QuestJournalAction, revision: u64) -> QuestJournalCommand {
    QuestJournalCommand {
        request_id: NonZeroU128::new(n).unwrap(),
        quest: id("quest"),
        npc: id("guide"),
        action,
        expected_revision: revision,
    }
}
pub fn bag(p: &Profiles, full: bool, stones: u32) -> Bag {
    let mut slots = std::array::from_fn(|_| {
        if full {
            Stack::new(p.items.catalog(), id("theia.dev.training-sword"), 1)
        } else {
            None
        }
    });
    if stones > 0 {
        slots[0] = Stack::new(p.items.catalog(), id("theia.dev.training-stone"), stones);
    }
    Bag::from_slots(p.items.catalog(), 0, slots).unwrap()
}
pub fn count(bag: &Bag, item: &str) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id(item))
        .map(Stack::count)
        .sum()
}
pub async fn initialize(store: &mut Store, name: &str, p: &Profiles, seed: &Bag) -> Binding {
    let owner = store
        .resolve_owner("https://fixed-rewards.theia.test", name)
        .await
        .unwrap();
    let who = store
        .create_character(
            owner,
            WORLD,
            CharacterName::parse(name).unwrap(),
            GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        )
        .await
        .unwrap()
        .saved
        .binding;
    store
        .ensure_inventory(who, WORLD, &p.items, seed)
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, &p.xp).await.unwrap();
    store
        .ensure_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    who
}
pub async fn ready(store: &mut Store, who: Binding, p: &Profiles) -> QuestJournalApplied {
    let mut result = store
        .command_quest_journal(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            &command(1, QuestJournalAction::Accept, 0),
        )
        .await
        .unwrap();
    if result.journal.progress(&id("quest")).unwrap().status() != Status::Ready {
        result = store
            .record_quest_event(
                who,
                WORLD,
                &p.journal,
                NonZeroU128::new(1).unwrap(),
                &EventKey::Kill(id("wolf")),
            )
            .await
            .unwrap();
    }
    assert_eq!(
        result.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    result
}
pub async fn shutdown(
    store: Store,
    admin: tokio_postgres::Client,
    driver: tokio::task::JoinHandle<std::result::Result<(), tokio_postgres::Error>>,
) {
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
