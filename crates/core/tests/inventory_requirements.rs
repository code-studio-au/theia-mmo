//! Pure possession/consumption only, not a live quest, deletion UI or SQL commit.
use theia_core::{
    content::ContentId,
    inventory::{
        Bag, Catalog, Command, MAX_REVISION, Operation, SLOT_COUNT, Stack,
        requirements::{Error, MAX_REQUIRED_COUNT, MAX_REQUIREMENTS, Requirement, Requirements},
    },
};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn catalog() -> Catalog {
    Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap()
}
fn stone() -> ContentId {
    id("theia.dev.training-stone")
}
fn token() -> ContentId {
    id("theia.dev.training-token")
}
fn bag(c: &Catalog, revision: u64, counts: &[(usize, ContentId, u32)]) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, item, count) in counts {
        slots[*slot] = Stack::new(c, item.clone(), *count);
    }
    Bag::from_slots(c, revision, slots).unwrap()
}
fn requirements(c: &Catalog, values: &[(ContentId, u32, bool)]) -> Requirements {
    Requirements::new(
        c,
        values
            .iter()
            .map(|(item, n, consume)| Requirement::new(item.clone(), *n, *consume).unwrap())
            .collect(),
    )
    .unwrap()
}
#[test]
fn split_merge_and_slot_order_preserve_total_and_one_of_three_missing_is_incomplete() {
    let c = catalog();
    let r = requirements(&c, &[(stone(), 3, true)]);
    let original = bag(&c, 0, &[(0, stone(), 3)]);
    let split = original
        .propose(
            &c,
            Command {
                expected_revision: 0,
                operation: Operation::Split {
                    from: 0,
                    to: 15,
                    amount: 1,
                },
            },
        )
        .unwrap();
    for b in [&original, &split] {
        let view = r.observe(&c, b).unwrap();
        assert!(view.satisfied());
        assert_eq!(
            (
                view.holdings()[0].held(),
                view.holdings()[0].displayed_count()
            ),
            (3, 3)
        );
        assert_eq!(view.inventory_revision(), b.revision());
    }
    let merged = split
        .propose(
            &c,
            Command {
                expected_revision: 1,
                operation: Operation::Move { from: 15, to: 0 },
            },
        )
        .unwrap();
    assert_eq!(
        r.observe(&c, &merged).unwrap().holdings(),
        r.observe(&c, &original).unwrap().holdings()
    );
    // An authoritative two-item fixture models a subsequent removal; this test
    // does not claim that a live deletion command is implemented.
    let reduced = bag(&c, 3, &[(0, stone(), 2)]);
    assert!(!r.observe(&c, &reduced).unwrap().satisfied());
    assert_eq!(
        r.observe(&c, &reduced).unwrap().holdings()[0].displayed_count(),
        2
    );
    assert_eq!(
        r.propose_consumption(&c, &reduced),
        Err(Error::MissingItems)
    );
    assert!(r.observe(&c, &original).unwrap().satisfied());
}
#[test]
fn consumption_rechecks_current_bag_and_uses_all_stacks_once_in_slot_order() {
    let c = catalog();
    let r = requirements(&c, &[(stone(), 3, true)]);
    let b = bag(
        &c,
        8,
        &[
            (0, stone(), 2),
            (5, stone(), 2),
            (10, stone(), 4),
            (15, token(), 7),
        ],
    );
    let view = r.observe(&c, &b).unwrap();
    assert_eq!(
        (
            view.holdings()[0].held(),
            view.holdings()[0].displayed_count()
        ),
        (8, 3)
    );
    let next = r.propose_consumption(&c, &b).unwrap();
    assert_eq!(next.revision(), 9);
    assert!(next.slots()[0].is_none());
    assert_eq!(next.slots()[5].as_ref().unwrap().count(), 1);
    assert_eq!(next.slots()[10].as_ref().unwrap().count(), 4);
    assert_eq!(next.slots()[15], b.slots()[15]);
    assert_eq!(b.revision(), 8);
    // A formerly sufficient observation is not a capability to consume a
    // different, insufficient current bag.
    let missing = bag(&c, 9, &[(0, stone(), 2)]);
    assert!(view.satisfied());
    assert_eq!(
        r.propose_consumption(&c, &missing),
        Err(Error::MissingItems)
    );
}
#[test]
fn mixed_consume_and_keep_requirements_are_atomic_and_nonconsuming_is_unchanged() {
    let c = catalog();
    let r = requirements(&c, &[(stone(), 3, true), (token(), 2, false)]);
    let missing = bag(&c, 0, &[(0, stone(), 3), (1, token(), 1)]);
    assert_eq!(
        r.propose_consumption(&c, &missing),
        Err(Error::MissingItems)
    );
    assert_eq!(missing.slots()[0].as_ref().unwrap().count(), 3);
    let b = bag(&c, 0, &[(0, stone(), 3), (1, token(), 2)]);
    let next = r.propose_consumption(&c, &b).unwrap();
    assert!(next.slots()[0].is_none());
    assert_eq!(next.slots()[1], b.slots()[1]);
    let keep = requirements(&c, &[(token(), 2, false)]);
    assert_eq!(keep.propose_consumption(&c, &b).unwrap(), b);
}
#[test]
fn duplicate_unknown_zero_impossible_and_over_budget_requirements_refuse() {
    let c = catalog();
    assert!(Requirement::new(stone(), 0, true).is_none());
    assert!(Requirement::new(stone(), MAX_REQUIRED_COUNT + 1, true).is_none());
    assert!(Requirements::new(&c, vec![]).is_err());
    assert!(Requirements::new(&c, vec![Requirement::new(stone(), 1, true).unwrap(); 2]).is_err());
    assert!(Requirements::new(&c, vec![Requirement::new(id("absent"), 1, true).unwrap()]).is_err());
    let max = c.definition(&stone()).unwrap().maximum_stack() * SLOT_COUNT as u32;
    assert!(
        Requirements::new(&c, vec![Requirement::new(stone(), max + 1, true).unwrap()]).is_err()
    );
    assert!(
        Requirements::new(
            &c,
            vec![Requirement::new(stone(), 1, true).unwrap(); MAX_REQUIREMENTS + 1]
        )
        .is_err()
    );
}
#[test]
fn equipped_item_is_not_a_backpack_holding_and_consumption_preserves_equipment() {
    let c = catalog();
    let sword = id("theia.dev.training-sword");
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(&c, stone(), 3);
    let hand = Stack::new(&c, sword.clone(), 1);
    let b = Bag::from_parts(&c, 0, slots, hand.clone()).unwrap();
    assert!(
        !requirements(&c, &[(sword, 1, true)])
            .observe(&c, &b)
            .unwrap()
            .satisfied()
    );
    let next = requirements(&c, &[(stone(), 3, true)])
        .propose_consumption(&c, &b)
        .unwrap();
    assert_eq!(next.main_hand(), hand.as_ref());
}
#[test]
fn exhaustion_refuses_mutation_but_read_and_keep_checks_still_work() {
    let c = catalog();
    let b = bag(&c, MAX_REVISION, &[(0, stone(), 3)]);
    let r = requirements(&c, &[(stone(), 3, true)]);
    assert!(r.observe(&c, &b).unwrap().satisfied());
    assert_eq!(
        r.propose_consumption(&c, &b),
        Err(Error::Inventory(
            theia_core::inventory::Error::RevisionExhausted
        ))
    );
    assert_eq!(
        requirements(&c, &[(stone(), 3, false)])
            .propose_consumption(&c, &b)
            .unwrap(),
        b
    );
}
#[test]
fn exact_maximum_bag_quantity_and_requirement_count_remain_bounded() {
    let source = serde_json::json!({"schema":1,"id":"test.items","items":(0..MAX_REQUIREMENTS).map(|n|
        serde_json::json!({"id":format!("item{n}"),"name":format!("Item {n}"),"maximum_stack":10000})).collect::<Vec<_>>()});
    let c = Catalog::from_json(&serde_json::to_vec(&source).unwrap()).unwrap();
    let all = Requirements::new(
        &c,
        (0..MAX_REQUIREMENTS)
            .map(|n| Requirement::new(id(&format!("item{n}")), 1, false).unwrap())
            .collect(),
    )
    .unwrap();
    assert_eq!(all.entries().count(), MAX_REQUIREMENTS);
    let full = bag(
        &c,
        0,
        &(0..SLOT_COUNT)
            .map(|s| (s, id("item0"), 10000))
            .collect::<Vec<_>>(),
    );
    let r = requirements(&c, &[(id("item0"), MAX_REQUIRED_COUNT, true)]);
    let view = r.observe(&c, &full).unwrap();
    assert_eq!(view.holdings()[0].held(), MAX_REQUIRED_COUNT);
    assert!(view.satisfied());
    let empty = r.propose_consumption(&c, &full).unwrap();
    assert!(empty.slots().iter().all(Option::is_none));
    assert_eq!(empty.revision(), 1);
    assert!(!all.observe(&c, &full).unwrap().satisfied());
}
