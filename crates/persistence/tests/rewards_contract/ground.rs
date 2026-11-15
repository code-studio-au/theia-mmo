//! Durable ground availability, atomic pickup/possession and exact delivery.
use super::*;
use serde_json::json;
use theia_core::quests::journal::Status;
use theia_persistence::{
    ProgressionCatalog, QuestJournalAction, QuestJournalCatalog, QuestJournalCommand,
};

#[tokio::test]
#[ignore = "requires owned PostgreSQL"]
async fn ground_pickup_requires_acceptance_commits_possession_and_delivers_once() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-coin-items.json"
    ))
    .unwrap();
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/quests/development-starter-quests.json"
    ))
    .unwrap();
    let quest = raw["quests"].as_array().unwrap().last().unwrap().clone();
    raw["quests"] = json!([quest]);
    let journal = QuestJournalCatalog::from_json(&serde_json::to_vec(&raw).unwrap(), &xp)
        .unwrap()
        .with_inventory(&items)
        .unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/gameplay/development-starter-loot.json"
    ))
    .unwrap();
    let reward = raw["rewards"].as_array().unwrap().last().unwrap().clone();
    raw["rewards"] = json!([reward]);
    let rewards = RewardCatalog::from_json_with_quest_journal(
        &serde_json::to_vec(&raw).unwrap(),
        &items,
        &journal,
    )
    .unwrap();
    let cid = |s| ContentId::new(s).unwrap();
    let item = cid("theia.dev.lost-compass");
    let reward = cid("theia.dev.compass-ground");
    let quest = cid("theia.dev.quest-lost-compass");
    let npc = cid("theia.dev.villager");
    let who = character(&mut store, "Groundcollector").await;
    store
        .ensure_inventory(who, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, &xp).await.unwrap();
    store
        .ensure_quest_journal(who, WORLD, &journal)
        .await
        .unwrap();
    assert!(matches!(
        store.offer_reward(who, WORLD, &rewards, &reward).await,
        Err(Error::RewardNotNeeded)
    ));
    let action = |request, revision, action| QuestJournalCommand {
        request_id: std::num::NonZeroU128::new(request).unwrap(),
        quest: quest.clone(),
        npc: npc.clone(),
        expected_revision: revision,
        action,
    };
    store
        .command_quest_journal(
            who,
            WORLD,
            &journal,
            &xp,
            &action(1, 0, QuestJournalAction::Accept),
        )
        .await
        .unwrap();
    let claim = store
        .offer_reward(who, WORLD, &rewards, &reward)
        .await
        .unwrap();
    assert_eq!(
        store
            .offer_reward(who, WORLD, &rewards, &reward)
            .await
            .unwrap(),
        claim
    );
    let picked = store
        .pickup_quest_reward(who, WORLD, &rewards, claim.id, 0)
        .await
        .unwrap();
    assert_eq!(
        picked
            .bag
            .slots()
            .iter()
            .flatten()
            .filter(|s| s.item() == &item)
            .map(Stack::count)
            .sum::<u32>(),
        1
    );
    assert_eq!(
        picked
            .journal
            .as_ref()
            .unwrap()
            .journal
            .progress(&quest)
            .unwrap()
            .status(),
        Status::Ready
    );
    assert!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, claim.id, 0)
            .await
            .unwrap()
            .replayed
    );
    let turn_in = action(2, 2, QuestJournalAction::TurnIn);
    let done = store
        .command_quest_journal(who, WORLD, &journal, &xp, &turn_in)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    assert_eq!(
        done.journal.progress(&quest).unwrap().status(),
        Status::Completed
    );
    assert!(
        !store
            .load_inventory(who, WORLD, &items)
            .await
            .unwrap()
            .slots()
            .iter()
            .flatten()
            .any(|s| s.item() == &item)
    );
    assert!(matches!(
        store.offer_reward(who, WORLD, &rewards, &reward).await,
        Err(Error::RewardNotNeeded)
    ));
    drop(store);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert!(
        store
            .command_quest_journal(who, WORLD, &journal, &xp, &turn_in)
            .await
            .unwrap()
            .replayed
    );
    drop(store);
    drop(admin);
    driver.abort();
}
