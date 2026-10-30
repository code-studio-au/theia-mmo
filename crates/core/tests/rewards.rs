//! Checked grant arithmetic only; no claim authorization, SQL or award success.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use theia_core::{
    content::ContentId,
    inventory::{Bag, Catalog as Items, Command, MAX_REVISION, Operation, SLOT_COUNT, Stack},
    rewards::{
        Catalog, Error, MAX_CATALOG_BYTES, MAX_DEFINITIONS, MAX_GRANT_UNITS, MAX_GRANTS, Scope,
    },
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-rewards.json");
#[test]
fn claim_identifiers_are_checked_and_never_authorize_a_stateless_item_grant() {
    use theia_core::{
        inventory::{Error as ItemError, ItemCommand},
        rewards::{ClaimCommand, ClaimId},
    };
    assert!(ClaimId::new([0; 16]).is_none());
    let claim = ClaimId::new([17; 16]).unwrap();
    let command = ClaimCommand {
        claim,
        expected_revision: 7,
    };
    let bytes = command.canonical_bytes();
    assert_eq!(&bytes[..16], &[17; 16]);
    assert_eq!(&bytes[16..], &7_u64.to_le_bytes());
    let items = Items::from_json(ITEMS).unwrap();
    let intent: ItemCommand = command.into();
    assert_eq!(intent.expected_revision(), 7);
    assert_eq!(
        intent.propose(&Bag::empty(&items), &items),
        Err(ItemError::ClaimAuthorityRequired)
    );
}
fn id(name: &str) -> ContentId {
    ContentId::new(&format!("theia.dev.training-{name}")).unwrap()
}
fn reward_id() -> ContentId {
    ContentId::new("theia.dev.first-training-kill").unwrap()
}
fn recipe(items: &Items, grants: Value) -> Catalog {
    let mut value: Value = serde_json::from_slice(DATA).unwrap();
    value["item_catalog"] = json!(items.id().as_str());
    value["rewards"][0]["grants"] = grants;
    Catalog::from_json(&serde_json::to_vec(&value).unwrap(), items).unwrap()
}
fn totals(b: &Bag) -> BTreeMap<ContentId, u64> {
    let mut map = BTreeMap::new();
    for s in b.slots().iter().flatten().chain(b.main_hand()) {
        *map.entry(s.item().clone()).or_default() += u64::from(s.count());
    }
    map
}
#[test]
fn catalogs_reject_unbounded_unknown_duplicate_or_client_like_reward_definitions() {
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = Catalog::from_json(DATA, &items).unwrap();
    let reward = catalog.definition(&reward_id()).unwrap();
    assert_eq!(catalog.definitions().len(), 1);
    assert_eq!(catalog.item_catalog(), items.id());
    assert_eq!(reward.scope(), Scope::OncePerCharacter);
    assert_eq!(reward.creature().as_str(), "theia.dev.training-creature");
    assert_eq!(reward.grants()[0].item(), &id("token"));
    assert_eq!(reward.grants()[0].count(), 1);
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for (key, value) in [
        ("schema", json!(2)),
        ("id", json!("../rewards")),
        ("item_catalog", json!("theia.other.items")),
        ("rewards", json!([])),
        ("unknown", json!(true)),
    ] {
        let mut bad = original.clone();
        bad[key] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap(), &items).is_err());
    }
    for (key, value) in [
        ("id", json!("../reward")),
        ("name", json!(" leading")),
        ("name", json!("a".repeat(49))),
        ("name", json!(" ")),
        ("name", json!("Name\n")),
        ("scope", json!("repeatable")),
        ("creature", json!("../creature")),
        ("grants", json!([])),
        ("owner", json!("client-chosen")),
    ] {
        let mut bad = original.clone();
        bad["rewards"][0][key] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap(), &items).is_err());
    }
    for grant in [
        json!({"item":"theia.dev.training-token","count":0}),
        json!({"item":"theia.dev.training-token","count":-1}),
        json!({"item":"theia.dev.training-token","count":1.5}),
        json!({"item":"theia.dev.training-token","count":MAX_GRANT_UNITS+1}),
        json!({"item":"theia.dev.unknown","count":1}),
        json!({"item":"../token","count":1}),
        json!({"item":"theia.dev.training-token","count":1,"slot":0}),
    ] {
        let mut bad = original.clone();
        bad["rewards"][0]["grants"] = json!([grant]);
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap(), &items).is_err());
    }
    let mut duplicate = original.clone();
    duplicate["rewards"][0]["grants"] = json!([{"item":"theia.dev.training-token","count":1},{"item":"theia.dev.training-token","count":1}]);
    assert!(Catalog::from_json(&serde_json::to_vec(&duplicate).unwrap(), &items).is_err());
    duplicate = original.clone();
    duplicate["rewards"] = json!([original["rewards"][0], original["rewards"][0]]);
    assert!(Catalog::from_json(&serde_json::to_vec(&duplicate).unwrap(), &items).is_err());
    let mut oversized = original.clone();
    oversized["rewards"] = json!(vec![original["rewards"][0].clone(); MAX_DEFINITIONS + 1]);
    assert!(Catalog::from_json(&serde_json::to_vec(&oversized).unwrap(), &items).is_err());
    oversized = original;
    oversized["rewards"][0]["grants"] = json!(vec![
        json!({"item":"theia.dev.training-token","count":1});
        MAX_GRANTS + 1
    ]);
    assert!(Catalog::from_json(&serde_json::to_vec(&oversized).unwrap(), &items).is_err());
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1], &items).is_err());
}
#[test]
fn grants_merge_then_fill_in_slot_order_without_changing_equipment_or_published_state() {
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = recipe(
        &items,
        json!([{"item":"theia.dev.training-token","count":5}]),
    );
    let reward = catalog.definition(&reward_id()).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[2] = Stack::new(&items, id("token"), 98);
    slots[4] = Stack::new(&items, id("token"), 97);
    let bag = Bag::from_parts(&items, 7, slots, Stack::new(&items, id("sword"), 1)).unwrap();
    let before = bag.to_json();
    let next = reward.propose(&bag, &items, 7).unwrap();
    assert_eq!(next.revision(), 8);
    assert_eq!(next.main_hand(), bag.main_hand());
    assert_eq!(bag.to_json(), before);
    assert_eq!(next.slots()[2].as_ref().unwrap().count(), 99);
    assert_eq!(next.slots()[4].as_ref().unwrap().count(), 99);
    assert_eq!(next.slots()[0].as_ref().unwrap().count(), 2);
    assert!(next.slots()[1].is_none());
    let mut expected = totals(&bag);
    *expected.entry(id("token")).or_default() += 5;
    assert_eq!(totals(&next), expected);
    assert_eq!(Bag::from_json(&items, &next.to_json()).unwrap(), next);
}
#[test]
fn full_backpack_and_multi_item_failure_are_atomic_even_after_tentative_merges() {
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = recipe(
        &items,
        json!([{"item":"theia.dev.training-token","count":2},{"item":"theia.dev.training-stone","count":1}]),
    );
    let reward = catalog.definition(&reward_id()).unwrap();
    let mut slots = std::array::from_fn(|_| Stack::new(&items, id("wand"), 1));
    slots[0] = Stack::new(&items, id("token"), 98);
    slots[15] = None;
    let bag =
        Bag::from_parts(&items, 0, slots.clone(), Stack::new(&items, id("sword"), 1)).unwrap();
    let before = bag.to_json();
    assert_eq!(reward.propose(&bag, &items, 0), Err(Error::NoCapacity));
    assert_eq!(bag.to_json(), before);
    slots[14] = None;
    let room = Bag::from_parts(&items, 0, slots, bag.main_hand().cloned()).unwrap();
    let next = reward.propose(&room, &items, 0).unwrap();
    let mut expected = totals(&room);
    *expected.entry(id("token")).or_default() += 2;
    *expected.entry(id("stone")).or_default() += 1;
    assert_eq!(totals(&next), expected);
    assert_eq!(next.main_hand(), room.main_hand());
    let full = Bag::from_slots(
        &items,
        0,
        std::array::from_fn(|_| Stack::new(&items, id("token"), 99)),
    )
    .unwrap();
    let single = Catalog::from_json(DATA, &items).unwrap();
    assert_eq!(
        single
            .definition(&reward_id())
            .unwrap()
            .propose(&full, &items, 0),
        Err(Error::NoCapacity)
    );
}
#[test]
fn stale_exhausted_or_foreign_proposals_cannot_mint_or_rebind_item_state() {
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = Catalog::from_json(DATA, &items).unwrap();
    let reward = catalog.definition(&reward_id()).unwrap();
    let bag = Bag::empty(&items);
    assert_eq!(reward.propose(&bag, &items, 1), Err(Error::StaleRevision));
    let exhausted = Bag::from_slots(&items, MAX_REVISION, bag.slots().clone()).unwrap();
    assert_eq!(
        reward.propose(&exhausted, &items, MAX_REVISION),
        Err(Error::RevisionExhausted)
    );
    let mut raw: Value = serde_json::from_slice(ITEMS).unwrap();
    raw["id"] = json!("theia.other.items");
    let foreign = Items::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert_eq!(
        reward.propose(&bag, &foreign, 0),
        Err(Error::CatalogMismatch)
    );
    raw = serde_json::from_slice(ITEMS).unwrap();
    raw["items"]
        .as_array_mut()
        .unwrap()
        .retain(|d| d["id"] != "theia.dev.training-token");
    let missing = Items::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert_eq!(reward.propose(&bag, &missing, 0), Err(Error::InvalidGrant));
    assert_eq!(bag.revision(), 0);
    assert!(totals(&bag).is_empty());
}
#[test]
fn grant_order_is_canonical_and_total_quantity_is_bounded_across_item_types() {
    let items = Items::from_json(ITEMS).unwrap();
    let a = recipe(
        &items,
        json!([{"item":"theia.dev.training-token","count":1},{"item":"theia.dev.training-stone","count":1}]),
    );
    let b = recipe(
        &items,
        json!([{"item":"theia.dev.training-stone","count":1},{"item":"theia.dev.training-token","count":1}]),
    );
    assert_eq!(a, b);
    let bag = Bag::empty(&items);
    assert_eq!(
        a.definition(&reward_id()).unwrap().propose(&bag, &items, 0),
        b.definition(&reward_id()).unwrap().propose(&bag, &items, 0)
    );
    let mut raw: Value = serde_json::from_slice(DATA).unwrap();
    raw["rewards"][0]["grants"] = json!([{"item":"theia.dev.training-token","count":MAX_GRANT_UNITS},{"item":"theia.dev.training-stone","count":1}]);
    assert!(Catalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items).is_err());
}
#[test]
fn largest_quantity_uses_stack_capacity_without_work_per_granted_unit() {
    let items=Items::from_json(br#"{"schema":1,"id":"theia.test.items","items":[{"id":"theia.dev.training-token","name":"Token","maximum_stack":10000}]}"#).unwrap();
    let catalog = recipe(
        &items,
        json!([{"item":"theia.dev.training-token","count":MAX_GRANT_UNITS}]),
    );
    let reward = catalog.definition(&reward_id()).unwrap();
    let next = reward.propose(&Bag::empty(&items), &items, 0).unwrap();
    assert_eq!(next.slots().iter().flatten().count(), 1);
    assert_eq!(next.slots()[0].as_ref().unwrap().count(), MAX_GRANT_UNITS);
    assert!(!next.equipment_enabled());
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = recipe(
        &items,
        json!([{"item":"theia.dev.training-sword","count":MAX_GRANT_UNITS}]),
    );
    assert_eq!(
        catalog
            .definition(&reward_id())
            .unwrap()
            .propose(&Bag::empty(&items), &items, 0),
        Err(Error::NoCapacity)
    );
}
#[test]
fn repeated_trusted_proposals_mint_only_recipe_quantities_and_share_transfer_revision() {
    let items = Items::from_json(ITEMS).unwrap();
    let catalog = Catalog::from_json(DATA, &items).unwrap();
    let reward = catalog.definition(&reward_id()).unwrap();
    let mut bag = Bag::from_parts(
        &items,
        0,
        std::array::from_fn(|_| None),
        Stack::new(&items, id("sword"), 1),
    )
    .unwrap();
    let mut random = 37_u64;
    let mut granted = 0;
    for _ in 0..3000 {
        let previous = bag.clone();
        match reward.propose(&bag, &items, bag.revision()) {
            Ok(next) => {
                assert_eq!(next.revision(), bag.revision() + 1);
                bag = next;
                granted += 1;
            }
            Err(Error::NoCapacity) => assert_eq!(bag, previous),
            Err(e) => panic!("Unexpected checked grant failure: {e:?}"),
        }
        assert_eq!(
            totals(&bag).get(&id("token")).copied().unwrap_or(0),
            granted
        );
        assert_eq!(bag.main_hand(), previous.main_hand());
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let from = (random % SLOT_COUNT as u64) as u8;
        let to = ((random >> 32) % SLOT_COUNT as u64) as u8;
        if let Ok(next) = bag.propose(
            &items,
            Command {
                expected_revision: bag.revision(),
                operation: Operation::Move { from, to },
            },
        ) {
            assert_eq!(totals(&bag), totals(&next));
            bag = next;
        }
    }
    assert_eq!(granted, 99 * SLOT_COUNT as u64);
    assert_eq!(totals(&bag).get(&id("sword")), Some(&1));
}
