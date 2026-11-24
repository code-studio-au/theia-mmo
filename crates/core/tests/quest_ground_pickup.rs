use serde_json::{Value, json};
use theia_core::{inventory, quests::catalog::Catalog as Quests, rewards::Catalog};
fn items() -> inventory::Catalog {
    inventory::Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-coin-items.json"
    ))
    .unwrap()
}
fn rewards() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-starter-loot.json"
    ))
    .unwrap()
}
#[test]
fn ground_item_is_bound_to_one_consuming_delivery_quest() {
    let items = items();
    let rewards = Catalog::from_json(&serde_json::to_vec(&rewards()).unwrap(), &items).unwrap();
    let quests = Quests::from_json(include_bytes!(
        "../../../content/quests/development-starter-quests.json"
    ))
    .unwrap();
    rewards.validate_quest_items(&quests).unwrap();
    let ground = rewards
        .definitions()
        .find(|d| d.ground().is_some())
        .unwrap();
    assert_eq!(ground.ground().unwrap().area, "Northern Meadow");
    assert_eq!(ground.grants()[0].count(), 1);
    let quest = quests.quest(&ground.ground().unwrap().quest).unwrap();
    assert_eq!(quest.ready_label(), Some("Deliver Lost Compass to Rowan"));
    assert_eq!(
        quest.stages().next().unwrap().objectives()[0].label(),
        "Collect Lost Compass from Northern Meadow"
    );
    let mut wrong: Value = serde_json::from_slice(include_bytes!(
        "../../../content/quests/development-starter-quests.json"
    ))
    .unwrap();
    wrong["quests"].as_array_mut().unwrap().last_mut().unwrap()["stages"][0]["objectives"][0]["consume_on_turn_in"] =
        json!(false);
    assert!(
        rewards
            .validate_quest_items(&Quests::from_json(&serde_json::to_vec(&wrong).unwrap()).unwrap())
            .is_err()
    );
}
#[test]
fn ground_reward_rejects_ambiguous_sources_probability_and_duplicate_grants() {
    for (field, value) in [
        ("creature", json!("theia.dev.training-creature")),
        ("scope", json!("per_creature_life")),
        ("quest_required", json!(false)),
        ("chance_basis_points", json!(5000)),
        (
            "grants",
            json!([{"item":"theia.dev.lost-compass","count":2}]),
        ),
    ] {
        let mut raw = rewards();
        raw["rewards"].as_array_mut().unwrap().last_mut().unwrap()[field] = value;
        assert!(
            Catalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items()).is_err(),
            "accepted {field}"
        );
    }
    let mut raw = rewards();
    raw["schema"] = json!(4);
    assert!(Catalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items()).is_err());
}
