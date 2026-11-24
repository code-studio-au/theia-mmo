//! Atomic money/item proposals; durable and live entitlement checks are separate.
use theia_core::{
    content::ContentId,
    inventory::{
        self, Bag, Catalog, Command, EquipmentCommand, EquipmentOperation, Operation, Stack,
    },
    vendors::{Catalog as Vendor, Error, Purchase},
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-coin-items.json");
const VENDOR: &[u8] = include_bytes!("../../../content/gameplay/development-weapons-vendor.json");
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}
fn setup(copper: u64) -> (Catalog, Vendor, Bag) {
    let items = Catalog::from_json(ITEMS).unwrap();
    let vendor = Vendor::from_json(VENDOR, &items).unwrap();
    let bag =
        Bag::from_parts_with_copper(&items, 0, std::array::from_fn(|_| None), None, Some(copper))
            .unwrap();
    (items, vendor, bag)
}
fn trade(vendor: &Vendor, items: &Catalog, bag: &Bag, offer: &str) -> Result<Bag, Error> {
    vendor.propose(
        bag,
        items,
        &Purchase {
            expected_revision: bag.revision(),
            offer: id(&format!("theia.dev.weapons.{offer}")),
        },
    )
}
#[test]
fn buying_selling_repeated_stock_and_equipment_share_one_bounded_money_ledger() {
    let (items, vendor, start) = setup(52550);
    assert_eq!(items.starting_copper(), Some(52550));
    assert_eq!(inventory::format_coins(52550), "5g 25s 50c");
    let bought = trade(&vendor, &items, &start, "buy-sword").unwrap();
    assert_eq!(bought.copper(), Some(51300));
    assert_eq!(start.copper(), Some(52550));
    let equipped = bought
        .propose_equipment(
            &items,
            EquipmentCommand {
                expected_revision: 1,
                operation: EquipmentOperation::Equip { from: 0 },
            },
        )
        .unwrap();
    assert_eq!(equipped.copper(), bought.copper());
    assert_eq!(
        trade(&vendor, &items, &equipped, "sell-training-sword"),
        Err(Error::InsufficientCost)
    );
    let unequipped = equipped
        .propose_equipment(
            &items,
            EquipmentCommand {
                expected_revision: 2,
                operation: EquipmentOperation::Unequip { to: 0 },
            },
        )
        .unwrap();
    let sold = trade(&vendor, &items, &unequipped, "sell-training-sword").unwrap();
    assert_eq!(sold.copper(), Some(51550));
    assert!(sold.slots().iter().all(Option::is_none));
    assert_eq!(sold.revision(), 4);
    let mut again = sold;
    for _ in 0..20 {
        again = trade(&vendor, &items, &again, "buy-sword").unwrap();
        again = trade(&vendor, &items, &again, "sell-training-sword").unwrap();
    }
    assert_eq!(again.copper(), Some(31550)); // no stock or vendor cash depletion
    assert_eq!(Bag::from_json(&items, &again.to_json()).unwrap(), again);
}
#[test]
fn insufficient_funds_capacity_overflow_stale_and_destroy_are_atomic() {
    let (items, vendor, poor) = setup(1249);
    assert_eq!(
        trade(&vendor, &items, &poor, "buy-sword"),
        Err(Error::InsufficientCost)
    );
    let sword = Stack::new(&items, id("theia.dev.training-sword"), 1);
    let full = Bag::from_parts_with_copper(
        &items,
        0,
        std::array::from_fn(|_| sword.clone()),
        None,
        Some(52550),
    )
    .unwrap();
    assert_eq!(
        trade(&vendor, &items, &full, "buy-wand"),
        Err(Error::NoCapacity)
    );
    let capped = Bag::from_parts_with_copper(
        &items,
        0,
        full.slots().clone(),
        None,
        Some(inventory::MAX_COPPER),
    )
    .unwrap();
    assert_eq!(
        trade(&vendor, &items, &capped, "sell-training-sword"),
        Err(Error::CurrencyOverflow)
    );
    let sold = trade(&vendor, &items, &full, "sell-training-sword").unwrap();
    assert_eq!(sold.copper(), Some(52800));
    assert_eq!(
        vendor.propose(
            &sold,
            &items,
            &Purchase {
                expected_revision: 0,
                offer: id("theia.dev.weapons.buy-sword")
            }
        ),
        Err(Error::StaleRevision)
    );
    let destroyed = full
        .propose(
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 1 },
            },
        )
        .unwrap();
    assert_eq!(destroyed.copper(), full.copper()); // deletion is not a sale
    assert_eq!(full.copper(), Some(52550));
}
#[test]
fn currency_schema_is_explicit_and_missing_overflow_or_legacy_money_is_refused() {
    let (items, _, bag) = setup(52550);
    let raw: serde_json::Value = serde_json::from_slice(&bag.to_json()).unwrap();
    assert_eq!(raw["schema"], 3);
    for value in [
        serde_json::json!(-1),
        serde_json::json!(1.5),
        serde_json::json!(null),
        serde_json::json!(u64::MAX),
    ] {
        let mut bad = raw.clone();
        bad["copper"] = value;
        assert!(Bag::from_json(&items, &serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = raw.clone();
    bad.as_object_mut().unwrap().remove("copper");
    assert!(Bag::from_json(&items, &serde_json::to_vec(&bad).unwrap()).is_err());
    let legacy = Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-document-items.json"
    ))
    .unwrap();
    assert!(Bag::from_json(&legacy, &bag.to_json()).is_err());
    assert!(Bag::from_json(&items, &Bag::empty(&legacy).to_json()).is_err());
    assert!(Vendor::from_json(VENDOR, &legacy).is_err());
}
#[test]
fn rewards_preserve_currency() {
    let (items, _, bag) = setup(52550);
    let rewards = theia_core::rewards::Catalog::from_json(
        include_bytes!("../../../content/gameplay/development-document-loot.json"),
        &items,
    )
    .unwrap();
    let reward = rewards.definitions().next().unwrap();
    let next = reward.propose(&bag, &items, 0).unwrap();
    assert_eq!(next.copper(), bag.copper());
}
