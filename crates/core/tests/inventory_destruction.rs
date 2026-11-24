use theia_core::{
    content::ContentId,
    inventory::{Bag, Catalog, Command, Error, MAX_REVISION, Operation, Stack},
};
fn items() -> Catalog {
    Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap()
}
fn bag(c: &Catalog, revision: u64) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(c, ContentId::new("theia.dev.training-stone").unwrap(), 3);
    Bag::from_slots(c, revision, slots).unwrap()
}
#[test]
fn destroy_partial_or_whole_stack_is_checked_single_revision_and_leaves_other_slots_alone() {
    let c = items();
    let original = bag(&c, 0);
    let next = original
        .propose(
            &c,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 1 },
            },
        )
        .unwrap();
    assert_eq!(next.slots()[0].as_ref().unwrap().count(), 2);
    assert_eq!(next.revision(), 1);
    assert_eq!(&next.slots()[1..], &original.slots()[1..]);
    let empty = next
        .propose(
            &c,
            Command {
                expected_revision: 1,
                operation: Operation::Destroy { from: 0, amount: 2 },
            },
        )
        .unwrap();
    assert!(empty.slots().iter().all(Option::is_none));
    assert_eq!(empty.revision(), 2);
    assert_eq!(original.slots()[0].as_ref().unwrap().count(), 3);
}
#[test]
fn destroy_refuses_invalid_source_amount_stale_or_exhausted_revision_without_changing_bag() {
    let c = items();
    let b = bag(&c, 0);
    for (from, amount, error) in [
        (0, 0, Error::InvalidAmount),
        (0, 4, Error::InvalidAmount),
        (15, 1, Error::EmptySource),
        (16, 1, Error::InvalidSlot),
    ] {
        assert_eq!(
            b.propose(
                &c,
                Command {
                    expected_revision: 0,
                    operation: Operation::Destroy { from, amount }
                }
            ),
            Err(error)
        );
    }
    assert_eq!(
        b.propose(
            &c,
            Command {
                expected_revision: 1,
                operation: Operation::Destroy { from: 0, amount: 1 }
            }
        ),
        Err(Error::StaleRevision)
    );
    assert_eq!(
        bag(&c, MAX_REVISION).propose(
            &c,
            Command {
                expected_revision: MAX_REVISION,
                operation: Operation::Destroy { from: 0, amount: 1 }
            }
        ),
        Err(Error::RevisionExhausted)
    );
}
