//! Pure tentative exchanges; no live vendor, receipt or purchase acknowledgement.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    inventory::{self, Bag, Catalog as Items, Stack},
    vendors::{self, Catalog, Error, Purchase},
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-vendor.json");
fn items() -> Items {
    Items::from_json(ITEMS).unwrap()
}
fn raw() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn load(value: &Value, items: &Items) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap(), items)
}
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn stack(items: &Items, name: &str, count: u32) -> Option<Stack> {
    Stack::new(items, id(&format!("theia.dev.training-{name}")), count)
}
fn purchase(revision: u64) -> Purchase {
    Purchase {
        expected_revision: revision,
        offer: id("theia.dev.training-vendor.stones"),
    }
}
fn count(bag: &Bag, name: &str) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id(&format!("theia.dev.training-{name}")))
        .map(Stack::count)
        .sum()
}

#[test]
fn checked_profiles_reject_unknown_free_ambiguous_or_unsupported_prices() {
    let items = items();
    for (key, value) in [
        ("schema", json!(2)),
        ("npc", json!("../npc")),
        ("item_catalog", json!("foreign")),
        ("offers", json!([])),
        ("currency", json!("gold")),
    ] {
        let mut bad = raw();
        bad[key] = value;
        assert!(load(&bad, &items).is_err(), "{key}");
    }
    for (key, value) in [
        ("name", json!("")),
        ("name", json!(" leading")),
        ("name", json!("trailing ")),
        ("name", json!("line\nbreak")),
        ("name", json!("é".repeat(33))),
        ("quantity", json!(2)),
        ("scope", json!("free")),
    ] {
        let mut bad = raw();
        bad["offers"][0][key] = value;
        assert!(load(&bad, &items).is_err(), "{key}");
    }
    for leg in ["cost", "grant"] {
        for (key, value) in [
            ("count", json!(0)),
            ("count", json!(-1)),
            ("count", json!(1.5)),
            ("count", json!(vendors::MAX_TRADE_UNITS + 1)),
            ("item", json!("missing")),
            ("item", json!("../item")),
            ("price", json!(1)),
        ] {
            let mut bad = raw();
            bad["offers"][0][leg][key] = value;
            assert!(load(&bad, &items).is_err(), "{leg}.{key}");
        }
    }
    let mut same = raw();
    same["offers"][0]["grant"]["item"] = same["offers"][0]["cost"]["item"].clone();
    assert!(load(&same, &items).is_err());
    let mut equipment = raw();
    equipment["offers"][0]["cost"]["item"] = json!("theia.dev.training-sword");
    assert!(load(&equipment, &items).is_err());
    let mut impossible = raw();
    impossible["offers"][0]["grant"]["item"] = json!("theia.dev.training-sword");
    impossible["offers"][0]["grant"]["count"] = json!(17);
    assert!(load(&impossible, &items).is_err());
    let data = String::from_utf8(DATA.to_vec()).unwrap();
    for duplicate in [
        data.replacen("\"schema\": 1", "\"schema\": 1, \"schema\": 1", 1),
        data.replacen("\"count\": 1", "\"count\": 1, \"count\": 1", 1),
    ] {
        assert!(Catalog::from_json(duplicate.as_bytes(), &items).is_err());
    }
}

#[test]
fn definition_aliases_and_catalogue_budget_fail_without_silent_offer_replacement() {
    let items = items();
    for alias in [
        "theia.dev.training-vendor",
        "theia.dev.guide",
        "theia.dev.equipment-items",
        "theia.dev.training-token",
    ] {
        let mut bad = raw();
        bad["offers"][0]["id"] = json!(alias);
        assert!(load(&bad, &items).is_err());
    }
    let mut bad = raw();
    bad["npc"] = bad["id"].clone();
    assert!(load(&bad, &items).is_err());
    let mut bad = raw();
    let duplicate = bad["offers"][0].clone();
    bad["offers"].as_array_mut().unwrap().push(duplicate);
    assert!(load(&bad, &items).is_err());
    let mut maximum = raw();
    let offers: Vec<_> = (0..vendors::MAX_OFFERS)
        .map(|index| {
            let mut offer = maximum["offers"][0].clone();
            offer["id"] = json!(format!("offer-{index}"));
            offer
        })
        .collect();
    maximum["offers"] = json!(offers);
    maximum["offers"][0]["name"] = json!("é".repeat(32));
    assert_eq!(
        load(&maximum, &items).unwrap().offers().len(),
        vendors::MAX_OFFERS
    );
    maximum["offers"].as_array_mut().unwrap().push(json!({"id":"extra","name":"Extra","cost":{"item":"theia.dev.training-token","count":1},"grant":{"item":"theia.dev.training-stone","count":1}}));
    assert!(load(&maximum, &items).is_err());
    let mut padded = DATA.to_vec();
    padded.resize(vendors::MAX_CATALOG_BYTES, b' ');
    assert!(Catalog::from_json(&padded, &items).is_ok());
    padded.push(b' ');
    assert!(Catalog::from_json(&padded, &items).is_err());
}

#[test]
fn exchanges_consume_in_slot_order_then_merge_outputs_and_preserve_equipment() {
    let items = items();
    let mut data = raw();
    data["offers"][0]["cost"]["count"] = json!(4);
    data["offers"][0]["grant"]["count"] = json!(5);
    let vendor = load(&data, &items).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = stack(&items, "token", 2);
    slots[2] = stack(&items, "stone", 18);
    slots[3] = stack(&items, "token", 3);
    let bag = Bag::from_parts(&items, 9, slots, stack(&items, "sword", 1)).unwrap();
    let original = bag.clone();
    let next = vendor.propose(&bag, &items, &purchase(9)).unwrap();
    assert_eq!(vendor.npc().as_str(), "theia.dev.guide");
    assert_eq!(vendor.item_catalog(), items.id());
    assert_eq!(vendor.id().as_str(), "theia.dev.training-vendor");
    let offer = vendor.offer(&purchase(9).offer).unwrap();
    assert_eq!(offer.name(), "Training stones");
    assert_eq!(offer.cost().count(), 4);
    assert_eq!(offer.grant().count(), 5);
    assert_eq!(next.revision(), 10);
    assert_eq!(next.slots()[0], stack(&items, "stone", 3));
    assert_eq!(next.slots()[2], stack(&items, "stone", 20));
    assert_eq!(next.slots()[3], stack(&items, "token", 1));
    assert_eq!(count(&next, "token"), count(&bag, "token") - 4);
    assert_eq!(count(&next, "stone"), count(&bag, "stone") + 5);
    assert_eq!(next.main_hand(), bag.main_hand());
    assert_eq!(bag, original);
}

#[test]
fn full_bag_capacity_is_checked_after_cost_removal_and_failures_leave_no_partial_cost() {
    let items = items();
    let vendor = Catalog::from_json(DATA, &items).unwrap();
    let mut slots = std::array::from_fn(|_| stack(&items, "sword", 1));
    slots[0] = stack(&items, "token", 2);
    let bag = Bag::from_parts(&items, 4, slots.clone(), stack(&items, "wand", 1)).unwrap();
    let original = bag.clone();
    assert_eq!(
        vendor.propose(&bag, &items, &purchase(4)),
        Err(Error::NoCapacity)
    );
    assert_eq!(bag, original);
    slots[0] = stack(&items, "token", 1);
    let bag = Bag::from_parts(&items, 4, slots, stack(&items, "wand", 1)).unwrap();
    let next = vendor.propose(&bag, &items, &purchase(4)).unwrap();
    assert_eq!(next.slots()[0], stack(&items, "stone", 3));
    assert_eq!(next.main_hand(), bag.main_hand());
    assert_eq!(count(&next, "token"), 0);
    assert_eq!(next.slots().iter().flatten().count(), inventory::SLOT_COUNT);
}

#[test]
fn insufficient_cost_unknown_offer_stale_and_exhausted_state_cannot_mutate_the_source() {
    let items = items();
    let vendor = Catalog::from_json(DATA, &items).unwrap();
    let empty = Bag::empty(&items);
    let original = empty.clone();
    assert_eq!(
        vendor.propose(&empty, &items, &purchase(0)),
        Err(Error::InsufficientCost)
    );
    assert_eq!(empty, original);
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = stack(&items, "token", 2);
    let bag = Bag::from_slots(&items, 6, slots.clone()).unwrap();
    assert_eq!(
        vendor.propose(&bag, &items, &purchase(5)),
        Err(Error::StaleRevision)
    );
    assert_eq!(
        vendor.propose(&bag, &items, &purchase(7)),
        Err(Error::StaleRevision)
    );
    let missing = Purchase {
        offer: id("unknown"),
        ..purchase(6)
    };
    assert_eq!(
        vendor.propose(&bag, &items, &missing),
        Err(Error::UnknownOffer)
    );
    let exhausted = Bag::from_slots(&items, inventory::MAX_REVISION, slots).unwrap();
    assert_eq!(
        vendor.propose(&exhausted, &items, &purchase(inventory::MAX_REVISION)),
        Err(Error::RevisionExhausted)
    );
    let first = vendor.propose(&bag, &items, &purchase(6)).unwrap();
    let repeated = vendor.propose(&bag, &items, &purchase(6)).unwrap();
    assert_eq!(
        first, repeated,
        "Pure proposals do not enforce durable receipt idempotency"
    );
    assert_eq!(bag.revision(), 6);
    assert_eq!(count(&bag, "token"), 2);
}

#[test]
fn changed_item_definitions_even_with_the_same_logical_name_do_not_rebind_a_price() {
    let items = items();
    let vendor = Catalog::from_json(DATA, &items).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = stack(&items, "token", 2);
    let bag = Bag::from_slots(&items, 0, slots).unwrap();
    let mut raw: Value = serde_json::from_slice(ITEMS).unwrap();
    raw["items"][0]["maximum_stack"] = json!(21);
    let changed = Items::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert_eq!(changed.id(), items.id());
    assert_eq!(
        vendor.propose(&bag, &changed, &purchase(0)),
        Err(Error::CatalogMismatch)
    );
    raw["id"] = json!("foreign");
    let foreign = Items::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let foreign_bag = Bag::empty(&foreign);
    assert_eq!(
        vendor.propose(&foreign_bag, &items, &purchase(0)),
        Err(Error::CatalogMismatch)
    );
    assert_eq!(count(&bag, "token"), 2);
    assert_eq!(bag.revision(), 0);
}

#[test]
fn maximum_unit_counts_stay_bounded_by_slots_and_equippable_outputs_do_not_auto_equip() {
    let mut raw_items: Value = serde_json::from_slice(ITEMS).unwrap();
    raw_items["items"][0]["maximum_stack"] = json!(10_000);
    raw_items["items"][1]["maximum_stack"] = json!(10_000);
    let items = Items::from_json(&serde_json::to_vec(&raw_items).unwrap()).unwrap();
    let mut data = raw();
    data["offers"][0]["cost"]["count"] = json!(vendors::MAX_TRADE_UNITS);
    data["offers"][0]["grant"]["count"] = json!(vendors::MAX_TRADE_UNITS);
    let vendor = load(&data, &items).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = stack(&items, "token", vendors::MAX_TRADE_UNITS);
    let bag = Bag::from_parts(&items, 0, slots, stack(&items, "wand", 1)).unwrap();
    let next = vendor.propose(&bag, &items, &purchase(0)).unwrap();
    assert_eq!(
        next.slots()[0],
        stack(&items, "stone", vendors::MAX_TRADE_UNITS)
    );
    assert_eq!(next.slots().iter().flatten().count(), 1);
    let mut data = raw();
    data["offers"][0]["grant"]["item"] = json!("theia.dev.training-sword");
    data["offers"][0]["grant"]["count"] = json!(1);
    let vendor = load(&data, &items).unwrap();
    let next = vendor.propose(&bag, &items, &purchase(0)).unwrap();
    assert_eq!(count(&next, "sword"), 1);
    assert_eq!(next.main_hand(), bag.main_hand());
    assert_eq!(next.main_hand(), stack(&items, "wand", 1).as_ref());
}

#[test]
fn legacy_backpack_schema_can_barter_without_creating_an_equipment_slot() {
    let items = Items::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))
    .unwrap();
    let mut data = raw();
    data["item_catalog"] = json!(items.id().as_str());
    let vendor = load(&data, &items).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = stack(&items, "token", 1);
    let bag = Bag::from_slots(&items, 0, slots).unwrap();
    let next = vendor.propose(&bag, &items, &purchase(0)).unwrap();
    assert!(!next.equipment_enabled());
    assert!(next.main_hand().is_none());
    assert_eq!(count(&next, "stone"), 3);
    let value: Value = serde_json::from_slice(&next.to_json()).unwrap();
    assert_eq!(value["schema"], 1);
    assert!(value.get("main_hand").is_none());
}
