//! Item-conservation prerequisites only; proposals are not durable acknowledgements.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use theia_core::{
    content::ContentId,
    inventory::{
        Bag, Catalog, Command, EquipmentCommand, EquipmentOperation as Op, Error, MAX_BAG_BYTES,
        MAX_REVISION, Operation, SLOT_COUNT, Stack,
    },
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const LEGACY: &[u8] = include_bytes!("../../../content/gameplay/development-items.json");
fn id(suffix: &str) -> ContentId {
    ContentId::new(&format!("theia.dev.training-{suffix}")).unwrap()
}
fn fixture(catalog: &Catalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    for (i, name, count) in [
        (0, "sword", 1),
        (1, "wand", 1),
        (2, "stone", 13),
        (3, "stone", 15),
    ] {
        slots[i] = Stack::new(catalog, id(name), count);
    }
    Bag::from_slots(catalog, 0, slots).unwrap()
}
fn command(bag: &Bag, operation: Op) -> EquipmentCommand {
    EquipmentCommand {
        expected_revision: bag.revision(),
        operation,
    }
}
fn totals(bag: &Bag) -> BTreeMap<ContentId, u64> {
    let mut totals = BTreeMap::new();
    for item in bag.slots().iter().flatten().chain(bag.main_hand()) {
        *totals.entry(item.item().clone()).or_default() += u64::from(item.count());
    }
    totals
}
#[test]
fn schema_two_eligibility_is_checked_and_schema_one_remains_strict() {
    let catalog = Catalog::from_json(DATA).unwrap();
    assert!(catalog.equipment_enabled());
    assert_eq!(
        catalog
            .definition(&id("sword"))
            .unwrap()
            .main_hand_appearance()
            .unwrap()
            .as_str(),
        "theia.dev.simple-sword"
    );
    assert!(
        catalog
            .definition(&id("stone"))
            .unwrap()
            .main_hand_appearance()
            .is_none()
    );
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for (field, value) in [
        ("maximum_stack", json!(2)),
        ("maximum_stack", json!(0)),
        ("main_hand_appearance", json!("../sword.glb")),
        ("main_hand_appearance", json!("")),
        ("main_hand_appearance", json!("a".repeat(97))),
        ("main_hand_appearance", json!(true)),
        ("slot", json!("main_hand")),
    ] {
        let mut bad = original.clone();
        bad["items"][2][field] = value;
        assert!(Catalog::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut old: Value = serde_json::from_slice(LEGACY).unwrap();
    old["items"][0]["main_hand_appearance"] = Value::Null;
    assert!(Catalog::from_json(&serde_json::to_vec(&old).unwrap()).is_err());
    let legacy = Catalog::from_json(LEGACY).unwrap();
    assert!(!legacy.equipment_enabled());
    assert!(!Bag::empty(&legacy).equipment_enabled());
    assert!(
        !String::from_utf8(Bag::empty(&legacy).to_json())
            .unwrap()
            .contains("main_hand")
    );
}
#[test]
fn equip_replace_and_unequip_are_atomic_transfers_with_one_revision() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let bag = fixture(&catalog);
    let initial = totals(&bag);
    let equipped = bag
        .propose_equipment(&catalog, command(&bag, Op::Equip { from: 0 }))
        .unwrap();
    assert_eq!(equipped.main_hand().unwrap().item(), &id("sword"));
    assert!(equipped.slots()[0].is_none());
    assert_eq!(bag.main_hand(), None);
    let replaced = equipped
        .propose_equipment(&catalog, command(&equipped, Op::Equip { from: 1 }))
        .unwrap();
    assert_eq!(replaced.main_hand().unwrap().item(), &id("wand"));
    assert_eq!(replaced.slots()[1].as_ref().unwrap().item(), &id("sword"));
    let moved = replaced
        .propose(
            &catalog,
            Command {
                expected_revision: replaced.revision(),
                operation: Operation::Move { from: 1, to: 0 },
            },
        )
        .unwrap();
    assert_eq!(moved.main_hand(), replaced.main_hand());
    let returned = moved
        .propose_equipment(&catalog, command(&moved, Op::Unequip { to: 1 }))
        .unwrap();
    assert!(returned.main_hand().is_none());
    assert_eq!(returned.slots()[1].as_ref().unwrap().item(), &id("wand"));
    for (i, state) in [&bag, &equipped, &replaced, &moved, &returned]
        .into_iter()
        .enumerate()
    {
        assert_eq!(state.revision(), i as u64);
        assert_eq!(totals(state), initial);
        assert_eq!(
            Bag::from_json(&catalog, &state.to_json()),
            Ok(state.clone())
        );
    }
    assert_eq!(
        returned.propose_equipment(&catalog, command(&bag, Op::Equip { from: 0 })),
        Err(Error::StaleRevision)
    );
}
#[test]
fn rejections_preserve_every_item_and_full_backpack_allows_replacement_only() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let bag = fixture(&catalog);
    for (op, error) in [
        (Op::Equip { from: 2 }, Error::NotEquippable),
        (Op::Equip { from: 15 }, Error::EmptySource),
        (Op::Equip { from: 16 }, Error::InvalidSlot),
        (Op::Equip { from: 255 }, Error::InvalidSlot),
        (Op::Unequip { to: 15 }, Error::EmptyHand),
        (Op::Unequip { to: 255 }, Error::InvalidSlot),
    ] {
        let before = bag.to_json();
        assert_eq!(
            bag.propose_equipment(&catalog, command(&bag, op)),
            Err(error)
        );
        assert_eq!(bag.to_json(), before);
    }
    let legacy = Catalog::from_json(LEGACY).unwrap();
    let old = Bag::empty(&legacy);
    assert_eq!(
        old.propose_equipment(&legacy, command(&old, Op::Equip { from: 0 })),
        Err(Error::EquipmentDisabled)
    );
    let mut slots = std::array::from_fn(|_| Stack::new(&catalog, id("token"), 99));
    slots[0] = Stack::new(&catalog, id("wand"), 1);
    let full = Bag::from_parts(&catalog, 8, slots, Stack::new(&catalog, id("sword"), 1)).unwrap();
    assert_eq!(
        full.propose_equipment(&catalog, command(&full, Op::Unequip { to: 1 })),
        Err(Error::DestinationOccupied)
    );
    let swapped = full
        .propose_equipment(&catalog, command(&full, Op::Equip { from: 0 }))
        .unwrap();
    assert_eq!(totals(&full), totals(&swapped));
    assert_eq!(swapped.slots()[0].as_ref().unwrap().item(), &id("sword"));
    let exhausted = Bag::from_parts(
        &catalog,
        MAX_REVISION,
        swapped.slots().clone(),
        swapped.main_hand().cloned(),
    )
    .unwrap();
    assert_eq!(
        exhausted.propose_equipment(&catalog, command(&exhausted, Op::Equip { from: 0 })),
        Err(Error::RevisionExhausted)
    );
}
#[test]
fn checked_state_rejects_schema_aliases_corrupt_equipment_and_forged_eligibility() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let equipped = fixture(&catalog)
        .propose_equipment(
            &catalog,
            EquipmentCommand {
                expected_revision: 0,
                operation: Op::Equip { from: 0 },
            },
        )
        .unwrap();
    let original: Value = serde_json::from_slice(&equipped.to_json()).unwrap();
    let mut missing_hand = original.clone();
    missing_hand.as_object_mut().unwrap().remove("main_hand");
    assert!(
        Bag::from_json(&catalog, &serde_json::to_vec(&missing_hand).unwrap()).is_err(),
        "Missing equipment field must never mean unequip"
    );
    for (field, value) in [
        ("schema", json!(1)),
        ("schema", json!(3)),
        ("catalog", json!("theia.dev.backpack-items")),
        ("revision", json!(MAX_REVISION + 1)),
        (
            "main_hand",
            json!({"item":"theia.dev.training-stone","count":1}),
        ),
        (
            "main_hand",
            json!({"item":"theia.dev.training-sword","count":2}),
        ),
        (
            "main_hand",
            json!({"item":"theia.dev.training-sword","count":0}),
        ),
        (
            "main_hand",
            json!({"item":"theia.dev.training-foreign","count":1}),
        ),
        (
            "main_hand",
            json!({"item":"theia.dev.training-sword","count":1,"asset":"sword.glb"}),
        ),
        ("unknown", json!(true)),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(Bag::from_json(&catalog, &serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let legacy = Catalog::from_json(LEGACY).unwrap();
    assert!(Bag::from_json(&legacy, &equipped.to_json()).is_err());
    assert!(Bag::from_json(&catalog, &Bag::empty(&legacy).to_json()).is_err());
    let mut forged: Value = serde_json::from_slice(DATA).unwrap();
    forged["items"][0]["maximum_stack"] = json!(1);
    forged["items"][0]["main_hand_appearance"] = json!("theia.dev.simple-sword");
    let forged = Catalog::from_json(&serde_json::to_vec(&forged).unwrap()).unwrap();
    let fake = Stack::new(&forged, id("stone"), 1);
    assert_eq!(
        Bag::from_parts(&catalog, 0, std::array::from_fn(|_| None), fake),
        Err(Error::InvalidState)
    );
    assert_eq!(
        Bag::from_parts(
            &legacy,
            0,
            std::array::from_fn(|_| None),
            equipped.main_hand().cloned()
        ),
        Err(Error::InvalidState)
    );
}
#[test]
fn mixed_backpack_and_equipment_sequences_conserve_contents_and_share_cas_revision() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let mut bag = fixture(&catalog);
    let expected = totals(&bag);
    let mut random = 0x71ea_u64;
    let mut successes = 0;
    for _ in 0..10_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let from = ((random >> 16) % SLOT_COUNT as u64) as u8;
        let to = ((random >> 32) % SLOT_COUNT as u64) as u8;
        let old = bag.clone();
        let next = match random & 3 {
            0 => bag.propose_equipment(&catalog, command(&bag, Op::Equip { from })),
            1 => bag.propose_equipment(&catalog, command(&bag, Op::Unequip { to })),
            2 => bag.propose(
                &catalog,
                Command {
                    expected_revision: bag.revision(),
                    operation: Operation::Move { from, to },
                },
            ),
            _ => bag.propose(
                &catalog,
                Command {
                    expected_revision: bag.revision(),
                    operation: Operation::Split {
                        from,
                        to,
                        amount: 1,
                    },
                },
            ),
        };
        if let Ok(next) = next {
            assert_eq!(next.revision(), old.revision() + 1);
            bag = next;
            successes += 1;
        }
        assert_eq!(totals(&bag), expected);
        if let Some(hand) = bag.main_hand() {
            assert_eq!(hand.count(), 1);
            assert!(
                catalog
                    .definition(hand.item())
                    .unwrap()
                    .main_hand_appearance()
                    .is_some()
            );
        }
    }
    assert!(successes > 1000);
    assert_eq!(bag.revision(), successes);
}
#[test]
fn largest_checked_item_identifiers_with_equipment_fit_existing_durable_byte_budget() {
    let name = format!("a{}", "x".repeat(95));
    let catalog = Catalog::from_json(
        &serde_json::to_vec(&json!({
            "schema":2,"id":name,"items":[{
                "id":name,"name":"Item","maximum_stack":1,"main_hand_appearance":name
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let stack = Stack::new(&catalog, ContentId::new(&name).unwrap(), 1);
    let bag = Bag::from_parts(
        &catalog,
        MAX_REVISION,
        std::array::from_fn(|_| stack.clone()),
        stack,
    )
    .unwrap();
    assert!(bag.to_json().len() <= MAX_BAG_BYTES);
    assert_eq!(Bag::from_json(&catalog, &bag.to_json()), Ok(bag));
}
