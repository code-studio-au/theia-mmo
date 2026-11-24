//! Actual network/store constructors agree; no World admission or SQL claim.
use serde_json::{Value, json};
use theia_network::{
    quest_journal::Profile as Wire, renet_runtime::inventory::Profile as WireItems,
};
use theia_persistence::{InventoryCatalog, ProgressionCatalog, QuestJournalCatalog as Stored};
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
fn quests() -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Bring","description":"Test","prerequisites":[],
        "giver":"guide","turn_in":"guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
            "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.training-stone",
            "required":3,"consume_on_turn_in":true}]}]}]})).unwrap()
}
fn pair(q: &[u8], items: &[u8], areas: Option<&[u8]>) -> (Wire, Stored) {
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let store_xp = ProgressionCatalog::from_json(XP).unwrap();
    let wire = if let Some(areas) = areas {
        Wire::from_json_with_areas(q, areas, &xp).unwrap()
    } else {
        Wire::from_json(q, &xp).unwrap()
    };
    let stored = if let Some(areas) = areas {
        Stored::from_json_with_areas(q, areas, &store_xp).unwrap()
    } else {
        Stored::from_json(q, &store_xp).unwrap()
    };
    (
        wire.with_inventory(&WireItems::from_json(items).unwrap())
            .unwrap(),
        stored
            .with_inventory(&InventoryCatalog::from_json(items).unwrap())
            .unwrap(),
    )
}
#[test]
fn exact_inventory_bytes_bind_both_profiles_and_old_area_free_hash_stays_unchanged() {
    let q = quests();
    let (wire, store) = pair(&q, ITEMS, None);
    assert_eq!(wire.hash(), store.hash());
    assert_eq!(
        wire.inventory_hash(),
        Some(store.inventory().unwrap().hash())
    );
    assert_eq!(
        wire.inventory_catalog().unwrap(),
        store.inventory().unwrap().catalog()
    );
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let old = Wire::from_json(&q, &xp).unwrap();
    let mut expected = theia_core::quests::journal::PROFILE_DOMAIN.to_vec();
    expected.extend(&q);
    expected.extend(xp.hash());
    assert_eq!(old.hash(), theia_network::terrain::sha256(&expected));
    assert!(old.inventory_hash().is_none());
    assert_ne!(old.hash(), wire.hash());
    let mut changed = ITEMS.to_vec();
    changed.push(b' ');
    let (other, stored) = pair(&q, &changed, None);
    assert_eq!(other.hash(), stored.hash());
    assert_ne!(wire.hash(), other.hash());
}
#[test]
fn area_then_inventory_binding_agrees_and_duplicate_or_unused_binding_refuses() {
    let mut q: Value = serde_json::from_slice(&quests()).unwrap();
    q["quests"][0]["stages"][0]["objectives"].as_array_mut().unwrap().push(json!({
        "kind":"reach_area","id":"area-goal","label":"Reach","area":"theia.dev.area.western-overlook"}));
    let q = serde_json::to_vec(&q).unwrap();
    let areas = include_bytes!("../../../content/quests/development-quest-areas.json");
    let (wire, stored) = pair(&q, ITEMS, Some(areas));
    assert_eq!(wire.hash(), stored.hash());
    assert!(
        wire.with_inventory(&WireItems::from_json(ITEMS).unwrap())
            .is_err()
    );
    assert!(
        stored
            .with_inventory(&InventoryCatalog::from_json(ITEMS).unwrap())
            .is_err()
    );
    let old = include_bytes!("../../../content/quests/development-journal.json");
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    assert!(
        Wire::from_json(old, &xp)
            .unwrap()
            .with_inventory(&WireItems::from_json(ITEMS).unwrap())
            .is_err()
    );
}
#[test]
fn missing_impossible_and_contact_alias_item_references_refuse_binding() {
    let xp = theia_network::progression_wire::Profile::from_json(XP).unwrap();
    let store_xp = ProgressionCatalog::from_json(XP).unwrap();
    for (item, count) in [
        ("absent", 3),
        ("theia.dev.training-sword", 17),
        ("guide", 3),
    ] {
        let mut q: Value = serde_json::from_slice(&quests()).unwrap();
        q["quests"][0]["stages"][0]["objectives"][0]["item"] = json!(item);
        q["quests"][0]["stages"][0]["objectives"][0]["required"] = json!(count);
        let q = serde_json::to_vec(&q).unwrap();
        let mut items: Value = serde_json::from_slice(ITEMS).unwrap();
        if item == "guide" {
            items["items"][0]["id"] = json!("guide");
        }
        let items = serde_json::to_vec(&items).unwrap();
        assert!(
            Wire::from_json(&q, &xp)
                .unwrap()
                .with_inventory(&WireItems::from_json(&items).unwrap())
                .is_err()
        );
        assert!(
            Stored::from_json(&q, &store_xp)
                .unwrap()
                .with_inventory(&InventoryCatalog::from_json(&items).unwrap())
                .is_err()
        );
    }
}
