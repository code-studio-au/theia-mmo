//! Pure backpack invariants only; not durable item transactions or multiplayer.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use theia_core::{
    content::ContentId,
    inventory::{
        Bag, Catalog, Command, Error, MAX_BAG_BYTES, MAX_CATALOG_BYTES, MAX_DEFINITIONS,
        MAX_REVISION, MAX_STACK, Operation, SLOT_COUNT, Stack,
    },
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-items.json");
fn id(name: &str) -> ContentId {
    ContentId::new(&format!("theia.dev.training-{name}")).unwrap()
}
fn fixture(catalog: &Catalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(catalog, id("stone"), 13);
    slots[1] = Stack::new(catalog, id("stone"), 15);
    slots[2] = Stack::new(catalog, id("token"), 99);
    Bag::from_slots(catalog, 0, slots).unwrap()
}
fn command(bag: &Bag, operation: Operation) -> Command {
    Command {
        expected_revision: bag.revision(),
        operation,
    }
}
fn totals(bag: &Bag) -> BTreeMap<ContentId, u64> {
    let mut result = BTreeMap::new();
    for stack in bag.slots().iter().flatten() {
        *result.entry(stack.item().clone()).or_default() += u64::from(stack.count());
    }
    result
}
#[test]
fn catalogs_are_checked_bounded_and_not_arbitrary_client_item_data() {
    let catalog = Catalog::from_json(DATA).unwrap();
    assert_eq!(catalog.definitions().len(), 2);
    let stone = catalog.definition(&id("stone")).unwrap();
    assert_eq!(
        (stone.name(), stone.maximum_stack()),
        ("Training Stone", 20)
    );
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for (field, value) in [
        ("schema", json!(2)),
        ("id", json!("../items")),
        ("items", json!([])),
        ("unknown", json!(true)),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for (field, value) in [
        ("id", json!("../item")),
        ("name", json!("")),
        ("name", json!(" ")),
        ("name", json!("Training\nStone")),
        ("name", json!(" leading space")),
        ("name", json!("a".repeat(49))),
        ("maximum_stack", json!(0)),
        ("maximum_stack", json!(MAX_STACK + 1)),
        ("maximum_stack", json!(-1)),
        ("maximum_stack", json!(1.5)),
        ("unknown", json!(1)),
    ] {
        let mut bad = original.clone();
        bad["items"][0][field] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut duplicate = original.clone();
    duplicate["items"][1] = duplicate["items"][0].clone();
    assert!(Catalog::from_json(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    let mut oversized = original.clone();
    oversized["items"] = json!(
        (0..=MAX_DEFINITIONS)
            .map(|i| json!({"id":format!("theia.test.item-{i}"),"name":"Item","maximum_stack":1}))
            .collect::<Vec<_>>()
    );
    assert!(Catalog::from_json(&serde_json::to_vec(&oversized).unwrap()).is_err());
    assert!(Catalog::from_json(&vec![b' '; MAX_CATALOG_BYTES + 1]).is_err());
    assert!(Stack::new(&catalog, id("stone"), 0).is_none());
    assert!(Stack::new(&catalog, id("stone"), 21).is_none());
    assert!(Stack::new(&catalog, id("unknown"), 1).is_none());
}
#[test]
fn checked_snapshot_roundtrips_and_rejects_corrupt_or_incompatible_state() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let bag = fixture(&catalog);
    assert_eq!(Bag::from_json(&catalog, &bag.to_json()), Ok(bag.clone()));
    let original: Value = serde_json::from_slice(&bag.to_json()).unwrap();
    for (field, value) in [
        ("schema", json!(2)),
        ("catalog", json!("theia.different.catalog")),
        ("revision", json!(MAX_REVISION + 1)),
        ("revision", json!(-1)),
        ("slots", json!([])),
        ("slots", json!(vec![Value::Null; SLOT_COUNT + 1])),
        ("unknown", json!(1)),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(Bag::from_json(&catalog, &serde_json::to_vec(&bad).unwrap()).is_err());
    }
    for stack in [
        json!({"item":"theia.dev.training-stone","count":0}),
        json!({"item":"theia.dev.training-stone","count":21}),
        json!({"item":"theia.dev.training-stone","count":1.5}),
        json!({"item":"theia.dev.training-unknown","count":1}),
        json!({"item":"../stone","count":1}),
        json!({"item":"theia.dev.training-stone","count":1,"unknown":1}),
    ] {
        let mut bad = original.clone();
        bad["slots"][0] = stack;
        assert!(Bag::from_json(&catalog, &serde_json::to_vec(&bad).unwrap()).is_err());
    }
    assert!(Bag::from_json(&catalog, &vec![b' '; MAX_BAG_BYTES + 1]).is_err());
    let mut other: Value = serde_json::from_slice(DATA).unwrap();
    other["items"][0]["maximum_stack"] = json!(1);
    let other = Catalog::from_json(&serde_json::to_vec(&other).unwrap()).unwrap();
    assert_eq!(
        Bag::from_slots(&other, 0, bag.slots().clone()),
        Err(Error::InvalidState)
    );
    assert!(Bag::from_json(&other, &bag.to_json()).is_err());
    let full = Bag::from_slots(
        &catalog,
        MAX_REVISION,
        std::array::from_fn(|_| Stack::new(&catalog, id("token"), 99)),
    )
    .unwrap();
    assert!(full.to_json().len() <= MAX_BAG_BYTES);
    assert_eq!(Bag::from_json(&catalog, &full.to_json()), Ok(full));
}
#[test]
fn moves_partial_merges_swaps_and_splits_preserve_items_without_mutating_published_state() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let bag = fixture(&catalog);
    let merge = bag
        .propose(&catalog, command(&bag, Operation::Move { from: 0, to: 1 }))
        .unwrap();
    assert_eq!(merge.slots()[0].as_ref().unwrap().count(), 8);
    assert_eq!(merge.slots()[1].as_ref().unwrap().count(), 20);
    assert_eq!(merge.revision(), 1);
    assert_eq!(bag, fixture(&catalog));
    let split = merge
        .propose(
            &catalog,
            command(
                &merge,
                Operation::Split {
                    from: 1,
                    to: 15,
                    amount: 7,
                },
            ),
        )
        .unwrap();
    assert_eq!(split.slots()[1].as_ref().unwrap().count(), 13);
    assert_eq!(split.slots()[15].as_ref().unwrap().count(), 7);
    let swap = split
        .propose(
            &catalog,
            command(&split, Operation::Move { from: 2, to: 15 }),
        )
        .unwrap();
    assert_eq!(swap.slots()[15].as_ref().unwrap().item(), &id("token"));
    assert_eq!(swap.slots()[2].as_ref().unwrap().item(), &id("stone"));
    let moved = swap
        .propose(
            &catalog,
            command(&swap, Operation::Move { from: 15, to: 14 }),
        )
        .unwrap();
    assert!(moved.slots()[15].is_none());
    for next in [&merge, &split, &swap, &moved] {
        assert_eq!(totals(next), totals(&bag));
        assert_eq!(Bag::from_json(&catalog, &next.to_json()), Ok(next.clone()));
    }
}
#[test]
fn failures_and_stale_replays_cannot_advance_revision_or_lose_items() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let bag = fixture(&catalog);
    for (operation, error) in [
        (Operation::Move { from: 16, to: 0 }, Error::InvalidSlot),
        (Operation::Move { from: 0, to: 255 }, Error::InvalidSlot),
        (Operation::Move { from: 0, to: 0 }, Error::SameSlot),
        (Operation::Move { from: 15, to: 0 }, Error::EmptySource),
        (
            Operation::Split {
                from: 0,
                to: 15,
                amount: 0,
            },
            Error::InvalidAmount,
        ),
        (
            Operation::Split {
                from: 0,
                to: 15,
                amount: 13,
            },
            Error::InvalidAmount,
        ),
        (
            Operation::Split {
                from: 0,
                to: 15,
                amount: u32::MAX,
            },
            Error::InvalidAmount,
        ),
        (
            Operation::Split {
                from: 0,
                to: 1,
                amount: 1,
            },
            Error::DestinationOccupied,
        ),
    ] {
        assert_eq!(bag.propose(&catalog, command(&bag, operation)), Err(error));
        assert_eq!(bag, fixture(&catalog));
    }
    let request = command(&bag, Operation::Move { from: 0, to: 1 });
    let next = bag.propose(&catalog, request).unwrap();
    assert_eq!(next.propose(&catalog, request), Err(Error::StaleRevision));
    assert_eq!(
        next.propose(&catalog, command(&next, request.operation)),
        Err(Error::DestinationFull)
    );
    let exhausted = Bag::from_slots(&catalog, MAX_REVISION, bag.slots().clone()).unwrap();
    assert_eq!(
        exhausted.propose(
            &catalog,
            command(&exhausted, Operation::Move { from: 0, to: 15 })
        ),
        Err(Error::RevisionExhausted)
    );
    assert_eq!(totals(&exhausted), totals(&bag));
}
#[test]
fn long_mixed_sequences_conserve_every_item_and_always_remain_valid() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let mut bag = fixture(&catalog);
    let expected = totals(&bag);
    let mut seed = 0x1234_5678_u64;
    let mut successes = 0;
    for i in 0..10_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let from = ((seed >> 32) % 18) as u8;
        let to = ((seed >> 40) % 18) as u8;
        let operation = if i % 3 == 0 {
            Operation::Split {
                from,
                to,
                amount: ((seed >> 48) % 25) as u32,
            }
        } else {
            Operation::Move { from, to }
        };
        let before = bag.clone();
        let proposal = bag.propose(&catalog, command(&bag, operation));
        assert_eq!(bag, before, "A proposal cannot publish itself");
        if let Ok(next) = proposal {
            assert_eq!(next.revision(), bag.revision() + 1);
            bag = next;
            successes += 1;
        }
        assert_eq!(totals(&bag), expected);
        assert_eq!(Bag::from_json(&catalog, &bag.to_json()), Ok(bag.clone()));
    }
    assert!(
        successes > 1000,
        "Mixed test must exercise successful mutations"
    );
}
