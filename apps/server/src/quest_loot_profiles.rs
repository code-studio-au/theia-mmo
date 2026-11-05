//! Exact constructor agreement and closed World admission; no SQL/native claim.
use std::sync::Arc;
use theia_network::{
    progression_wire, quest_journal, renet_runtime::inventory::Profile as WireItems,
    reward_wire::Profile as Wire,
};
use theia_persistence::{InventoryCatalog, ProgressionCatalog, QuestJournalCatalog, RewardCatalog};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-loot-items.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const QUESTS: &[u8] = include_bytes!("../../../content/quests/development-journal-items.json");
const AREAS: &[u8] = include_bytes!("../../../content/quests/development-quest-areas.json");
const REWARDS: &[u8] =
    include_bytes!("../../../content/gameplay/development-random-creature-loot-rewards.json");
fn rewards() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(REWARDS).unwrap();
    value["schema"] = serde_json::json!(4);
    value["rewards"][1]["quest_required"] = serde_json::json!(true);
    serde_json::to_vec(&value).unwrap()
}
fn journals(
    items: &[u8],
    quests: &[u8],
    xp: &[u8],
    areas: &[u8],
) -> (
    WireItems,
    InventoryCatalog,
    quest_journal::Profile,
    QuestJournalCatalog,
) {
    let wire_items = WireItems::from_json(items).unwrap();
    let stored_items = InventoryCatalog::from_json(items).unwrap();
    let wire = quest_journal::Profile::from_json_with_areas(
        quests,
        areas,
        &progression_wire::Profile::from_json(xp).unwrap(),
    )
    .unwrap()
    .with_inventory(&wire_items)
    .unwrap();
    let stored = QuestJournalCatalog::from_json_with_areas(
        quests,
        areas,
        &ProgressionCatalog::from_json(xp).unwrap(),
    )
    .unwrap()
    .with_inventory(&stored_items)
    .unwrap();
    (wire_items, stored_items, wire, stored)
}
#[test]
fn quest_loot_binds_exact_items_journal_xp_areas_and_reward_bytes_on_both_sides() {
    let source = rewards();
    let (wi, si, wj, sj) = journals(ITEMS, QUESTS, XP, AREAS);
    let wire = Wire::from_json_with_quest_journal(&source, &wi, &wj).unwrap();
    let stored = RewardCatalog::from_json_with_quest_journal(&source, &si, &sj).unwrap();
    assert_eq!(wire.hash(), stored.hash());
    assert_eq!(wire.quest_journal_hash(), Some(wj.hash()));
    assert_eq!(stored.quest_journal().unwrap().hash(), sj.hash());
    assert_eq!(wire.item_hash(), si.hash());
    let mut previous = std::collections::BTreeSet::new();
    previous.insert(wire.hash());
    for index in 0..5 {
        let mut inputs = [
            ITEMS.to_vec(),
            QUESTS.to_vec(),
            XP.to_vec(),
            AREAS.to_vec(),
            source.clone(),
        ];
        inputs[index].push(b' ');
        let (wi, si, wj, sj) = journals(&inputs[0], &inputs[1], &inputs[2], &inputs[3]);
        let wire = Wire::from_json_with_quest_journal(&inputs[4], &wi, &wj).unwrap();
        let stored = RewardCatalog::from_json_with_quest_journal(&inputs[4], &si, &sj).unwrap();
        assert_eq!(wire.hash(), stored.hash());
        assert!(
            previous.insert(wire.hash()),
            "Source {index} did not bind its bytes"
        );
    }
    let legacy = Wire::from_json_with_creature_loot(REWARDS, &wi).unwrap();
    let stored = RewardCatalog::from_json(REWARDS, &si).unwrap();
    let mut expected = theia_core::rewards::PROFILE_DOMAIN.to_vec();
    expected.extend(wi.hash());
    expected.extend(REWARDS);
    assert_eq!(legacy.hash(), theia_network::terrain::sha256(&expected));
    assert_eq!(legacy.hash(), stored.hash());
    assert!(legacy.quest_journal_hash().is_none());
    assert!(stored.quest_journal().is_none());
}
#[test]
fn missing_foreign_unused_and_unreferenced_quest_loot_bindings_refuse() {
    let source = rewards();
    let (wi, si, wj, sj) = journals(ITEMS, QUESTS, XP, AREAS);
    let unbound_wire = quest_journal::Profile::from_json(
        QUESTS,
        &progression_wire::Profile::from_json(XP).unwrap(),
    )
    .unwrap();
    let unbound_store =
        QuestJournalCatalog::from_json(QUESTS, &ProgressionCatalog::from_json(XP).unwrap())
            .unwrap();
    assert!(Wire::from_json_with_quest_journal(&source, &wi, &unbound_wire).is_err());
    assert!(RewardCatalog::from_json_with_quest_journal(&source, &si, &unbound_store).is_err());
    assert!(Wire::from_json_with_quest_journal(REWARDS, &wi, &wj).is_err());
    assert!(RewardCatalog::from_json_with_quest_journal(REWARDS, &si, &sj).is_err());
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    assert!(
        Wire::from_json_with_quest_journal(&source, &WireItems::from_json(&changed).unwrap(), &wj)
            .is_err()
    );
    assert!(
        RewardCatalog::from_json_with_quest_journal(
            &source,
            &InventoryCatalog::from_json(&changed).unwrap(),
            &sj
        )
        .is_err()
    );
    let changed = String::from_utf8(QUESTS.to_vec())
        .unwrap()
        .replace("theia.dev.wolf-pelt\"", "theia.dev.training-token\"");
    let (wi, si, wj, sj) = journals(ITEMS, changed.as_bytes(), XP, AREAS);
    assert!(Wire::from_json_with_quest_journal(&source, &wi, &wj).is_err());
    assert!(RewardCatalog::from_json_with_quest_journal(&source, &si, &sj).is_err());
}
#[test]
fn exact_binding_does_not_open_simulation_admission_or_change_world_identity() {
    let (wi, _, wj, _) = journals(ITEMS, QUESTS, XP, AREAS);
    let profile = Arc::new(Wire::from_json_with_quest_journal(&rewards(), &wi, &wj).unwrap());
    let mut world = theia_network::simulation::World::with_procedural([8000., 8000.], 1.).unwrap();
    let before = world.identity();
    for error in [
        world.enable_rewards(profile.clone()).unwrap_err(),
        world.enable_creature_loot(profile).unwrap_err(),
    ] {
        assert!(
            error
                .to_string()
                .contains("Quest-only loot simulation is not installed")
        );
    }
    assert_eq!(world.identity(), before);
    assert!(world.reward_profile().is_none());
}
