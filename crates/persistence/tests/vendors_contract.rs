//! Actual owned SQL/worker contracts. No live NPC/range or native purchase claim.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, Operation, Stack},
    vendors::{Error as CommandError, Purchase},
};
use theia_persistence::{
    Binding, CharacterName, Error, InventoryCatalog, RealmId, Store, VendorCatalog, Worker,
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-vendor.json");
const WORLD: [u8; 32] = [141; 32];
fn items() -> InventoryCatalog {
    InventoryCatalog::from_json(ITEMS).unwrap()
}
fn vendor(items: &InventoryCatalog) -> VendorCatalog {
    VendorCatalog::from_json(DATA, items).unwrap()
}
fn purchase(revision: u64) -> Purchase {
    Purchase {
        expected_revision: revision,
        offer: ContentId::new("theia.dev.training-vendor.stones").unwrap(),
    }
}
fn stack(items: &InventoryCatalog, name: &str, count: u32) -> Option<Stack> {
    Stack::new(
        items.catalog(),
        ContentId::new(&format!("theia.dev.training-{name}")).unwrap(),
        count,
    )
}
fn seed(items: &InventoryCatalog, tokens: u32, full: bool) -> Bag {
    let mut slots = std::array::from_fn(|_| if full { stack(items, "sword", 1) } else { None });
    slots[0] = stack(items, "token", tokens);
    if !full {
        slots[2] = stack(items, "stone", 18);
    }
    Bag::from_parts(items.catalog(), 0, slots, stack(items, "wand", 1)).unwrap()
}
fn total(bag: &Bag, name: &str) -> u32 {
    let id = ContentId::new(&format!("theia.dev.training-{name}")).unwrap();
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id)
        .map(Stack::count)
        .sum()
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/vendor-contract", name)
        .await
        .unwrap();
    store
        .create_character(
            owner,
            WORLD,
            CharacterName::parse(name).unwrap(),
            GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        )
        .await
        .unwrap()
        .saved
        .binding
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn purchase_receipts_bind_cost_output_profiles_owner_and_shared_item_revision() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let items = items();
    let vendor = vendor(&items);
    let a = character(&mut store, "Vendorone").await;
    let b = character(&mut store, "Vendortwo").await;
    let initial = seed(&items, 4, false);
    for who in [a, b] {
        store
            .ensure_inventory(who, WORLD, &items, &initial)
            .await
            .unwrap();
    }
    assert!(matches!(
        store
            .purchase_vendor(
                Binding {
                    owner: b.owner,
                    ..a
                },
                WORLD,
                &vendor,
                &purchase(0)
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .purchase_vendor(a, [142; 32], &vendor, &purchase(0))
            .await,
        Err(Error::WorldMismatch)
    ));
    let first = store
        .purchase_vendor(a, WORLD, &vendor, &purchase(0))
        .await
        .unwrap();
    assert!(!first.replayed);
    assert_eq!(first.bag.revision(), 1);
    assert_eq!(total(&first.bag, "token"), 3);
    assert_eq!(total(&first.bag, "stone"), 21);
    assert_eq!(first.bag.main_hand(), initial.main_hand());
    let retry = store
        .purchase_vendor(a, WORLD, &vendor, &purchase(0))
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(retry.bag, first.bag);
    let padded = VendorCatalog::from_json(&[DATA, b"\n"].concat(), &items).unwrap();
    assert_ne!(padded.hash(), vendor.hash());
    assert!(matches!(
        store.purchase_vendor(a, WORLD, &padded, &purchase(0)).await,
        Err(Error::InventoryConflict)
    ));
    let changed_items = InventoryCatalog::from_json(&[ITEMS, b"\n"].concat()).unwrap();
    let changed_vendor = vendor_for(&changed_items);
    assert!(matches!(
        store
            .purchase_vendor(a, WORLD, &changed_vendor, &purchase(1))
            .await,
        Err(Error::InventoryCatalogMismatch)
    ));
    let mut data: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    data["offers"][0]["cost"]["count"] = serde_json::json!(2);
    let expensive = VendorCatalog::from_json(&serde_json::to_vec(&data).unwrap(), &items).unwrap();
    let second = store
        .purchase_vendor(a, WORLD, &expensive, &purchase(1))
        .await
        .unwrap();
    assert_eq!(total(&second.bag, "token"), 1);
    assert_eq!(total(&second.bag, "stone"), 24);
    assert!(matches!(
        store.purchase_vendor(a, WORLD, &vendor, &purchase(0)).await,
        Err(Error::InventoryStale)
    ));
    let movement = Command {
        expected_revision: 2,
        operation: Operation::Move { from: 2, to: 3 },
    };
    store
        .mutate_inventory(a, WORLD, &items, movement)
        .await
        .unwrap();
    assert!(matches!(
        store
            .purchase_vendor(a, WORLD, &expensive, &purchase(1))
            .await,
        Err(Error::InventoryStale)
    ));
    let final_bag = store
        .purchase_vendor(a, WORLD, &vendor, &purchase(3))
        .await
        .unwrap()
        .bag;
    assert_eq!(final_bag.revision(), 4);
    assert_eq!(total(&final_bag, "token"), 0);
    assert_eq!(total(&final_bag, "stone"), 27);
    assert_eq!(
        store.load_inventory(b, WORLD, &items).await.unwrap(),
        initial
    );
    let row=admin.query_one("SELECT revision,octet_length(last_command) FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert_eq!(row.get::<_, i64>(0), 4);
    assert_eq!(row.get::<_, i32>(1), 32);
    let missing = character(&mut store, "Vendormissing").await;
    assert!(matches!(
        store
            .purchase_vendor(missing, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::InventoryMissing)
    ));
    let unknown = Purchase {
        offer: ContentId::new("unknown").unwrap(),
        ..purchase(4)
    };
    assert!(matches!(
        store.purchase_vendor(a, WORLD, &vendor, &unknown).await,
        Err(Error::VendorCommand(CommandError::UnknownOffer))
    ));
    assert!(matches!(
        store
            .purchase_vendor(a, WORLD, &vendor, &purchase(u64::MAX))
            .await,
        Err(Error::InventoryStale)
    ));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
fn vendor_for(items: &InventoryCatalog) -> VendorCatalog {
    vendor(items)
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn no_capacity_and_after_update_failure_rollback_cost_output_and_latest_receipt() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let items = items();
    let vendor = vendor(&items);
    let full = character(&mut store, "Vendorfull").await;
    let initial = seed(&items, 2, true);
    store
        .ensure_inventory(full, WORLD, &items, &initial)
        .await
        .unwrap();
    assert!(matches!(
        store
            .purchase_vendor(full, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::VendorCommand(CommandError::NoCapacity))
    ));
    assert_eq!(
        store.load_inventory(full, WORLD, &items).await.unwrap(),
        initial
    );
    let mut raw: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    raw["offers"][0]["cost"]["count"] = serde_json::json!(2);
    let clears_slot = VendorCatalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items).unwrap();
    let applied = store
        .purchase_vendor(full, WORLD, &clears_slot, &purchase(0))
        .await
        .unwrap();
    assert_eq!(total(&applied.bag, "token"), 0);
    assert_eq!(total(&applied.bag, "stone"), 3);
    assert_eq!(applied.bag.main_hand(), initial.main_hand());
    let who = character(&mut store, "Vendoratomic").await;
    let initial = seed(&items, 4, false);
    store
        .ensure_inventory(who, WORLD, &items, &initial)
        .await
        .unwrap();
    let name = format!("vendor_failure_{}", realm.value().simple());
    admin.batch_execute(&format!("CREATE FUNCTION theia.{name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.realm_id='{}'::uuid AND NEW.character_id='{}'::uuid THEN RAISE EXCEPTION 'owned vendor rollback fixture'; END IF; RETURN NEW; END $$; CREATE TRIGGER {name} AFTER UPDATE ON theia.backpacks FOR EACH ROW EXECUTE FUNCTION theia.{name}();",realm.value(),who.character.value())).await.unwrap();
    assert!(matches!(
        store
            .purchase_vendor(who, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::Database(_))
    ));
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        initial
    );
    let row=admin.query_one("SELECT revision,last_command IS NULL FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&who.character.value()]).await.unwrap();
    assert_eq!(row.get::<_, i64>(0), 0);
    assert!(row.get::<_, bool>(1));
    admin
        .batch_execute(&format!(
            "DROP TRIGGER {name} ON theia.backpacks; DROP FUNCTION theia.{name}();"
        ))
        .await
        .unwrap();
    let applied = store
        .purchase_vendor(who, WORLD, &vendor, &purchase(0))
        .await
        .unwrap();
    assert_eq!(applied.bag.revision(), 1);
    assert_eq!(total(&applied.bag, "token"), 3);
    assert_eq!(total(&applied.bag, "stone"), 21);
    assert!(
        store
            .purchase_vendor(who, WORLD, &vendor, &purchase(0))
            .await
            .unwrap()
            .replayed
    );
    let poor = character(&mut store, "Vendorpoor").await;
    let empty = Bag::empty(items.catalog());
    store
        .ensure_inventory(poor, WORLD, &items, &empty)
        .await
        .unwrap();
    assert!(matches!(
        store
            .purchase_vendor(poor, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::VendorCommand(CommandError::InsufficientCost))
    ));
    assert_eq!(
        store.load_inventory(poor, WORLD, &items).await.unwrap(),
        empty
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn blocked_backend(guard: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async {
        loop {
            guard.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
            if let Some(row)=guard.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT catalog_hash,%'",&[]).await.unwrap(){return row.get(0);}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.unwrap()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn shared_worker_serializes_purchase_races_recovers_lost_answers_and_bounds_backend_failure()
{
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let items = items();
    let vendor = vendor(&items);
    let who = character(&mut store, "Vendorworker").await;
    let initial = seed(&items, 8, false);
    store
        .ensure_inventory(who, WORLD, &items, &initial)
        .await
        .unwrap();
    assert!(matches!(
        Store::connect_loopback(&url, realm).await,
        Err(Error::RealmBusy)
    ));
    let poor = character(&mut store, "Vendorpoorwork").await;
    store
        .ensure_inventory(poor, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    let full = character(&mut store, "Vendorfullwork").await;
    store
        .ensure_inventory(full, WORLD, &items, &seed(&items, 2, true))
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    assert!(matches!(
        publisher
            .purchase_vendor(poor, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::VendorCommand(CommandError::InsufficientCost))
    ));
    assert!(matches!(
        publisher
            .purchase_vendor(full, WORLD, &vendor, &purchase(0))
            .await,
        Err(Error::VendorCommand(CommandError::NoCapacity))
    ));
    assert_eq!(
        publisher.load_inventory(who, WORLD, &items).await.unwrap(),
        initial,
        "Expected refusals must leave the shared worker available"
    );
    let buy = purchase(0);
    let movement = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 0, to: 1 },
    };
    let (bought, moved) = tokio::join!(
        publisher.purchase_vendor(who, WORLD, &vendor, &buy),
        publisher.mutate_inventory(who, WORLD, &items, movement)
    );
    assert_ne!(bought.is_ok(), moved.is_ok());
    let first = if let Ok(applied) = bought {
        assert!(matches!(moved, Err(Error::InventoryConflict)));
        assert!(
            publisher
                .purchase_vendor(who, WORLD, &vendor, &buy)
                .await
                .unwrap()
                .replayed
        );
        applied.bag
    } else {
        assert!(matches!(bought, Err(Error::InventoryConflict)));
        let applied = moved.unwrap();
        assert!(
            publisher
                .mutate_inventory(who, WORLD, &items, movement)
                .await
                .unwrap()
                .replayed
        );
        applied.bag
    };
    assert_eq!(first.revision(), 1);
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let v = vendor.clone();
    let pending =
        tokio::spawn(async move { p.purchase_vendor(who, WORLD, &v, &purchase(1)).await });
    blocked_backend(&guard).await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    guard.commit().await.unwrap();
    let committed = publisher.load_inventory(who, WORLD, &items).await.unwrap();
    assert_eq!(committed.revision(), 2);
    assert_eq!(total(&committed, "token"), total(&first, "token") - 1);
    assert_eq!(total(&committed, "stone"), total(&first, "stone") + 3);
    assert!(
        publisher
            .purchase_vendor(who, WORLD, &vendor, &purchase(1))
            .await
            .unwrap()
            .replayed
    );
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let v = vendor.clone();
    let pending =
        tokio::spawn(async move { p.purchase_vendor(who, WORLD, &v, &purchase(2)).await });
    let backend = blocked_backend(&guard).await;
    let mut queued = Vec::new();
    for _ in 0..17 {
        let p = publisher.clone();
        let v = vendor.clone();
        queued.push(tokio::spawn(async move {
            p.purchase_vendor(who, WORLD, &v, &purchase(2)).await
        }));
    }
    tokio::time::timeout(Duration::from_secs(1), async {
        while !queued.iter().any(tokio::task::JoinHandle::is_finished) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        guard
            .query_one("SELECT pg_terminate_backend($1)", &[&backend])
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    assert!(pending.await.unwrap().is_err());
    guard.rollback().await.unwrap();
    let mut busy = 0;
    for answer in queued {
        let result = answer.await.unwrap();
        if matches!(result, Err(Error::InventoryBusy)) {
            busy += 1;
        }
        assert!(
            result.is_err(),
            "No purchase success without durable commit"
        );
    }
    assert_eq!(
        busy, 1,
        "One executing purchase plus sixteen queued jobs is the bound"
    );
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        committed
    );
    assert!(
        store
            .purchase_vendor(who, WORLD, &vendor, &purchase(1))
            .await
            .unwrap()
            .replayed
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
