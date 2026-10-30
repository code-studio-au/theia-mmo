//! Actual owned PostgreSQL; no equipment network, rendering or gameplay claim.
mod support;
use std::{collections::BTreeMap, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, EquipmentCommand, EquipmentOperation as Op, Operation, Stack},
};
use theia_persistence::{Binding, CharacterName, Error, InventoryCatalog, RealmId, Store, Worker};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const WORLD: [u8; 32] = [107; 32];
fn profile() -> InventoryCatalog {
    InventoryCatalog::from_json(DATA).unwrap()
}
fn id(name: &str) -> ContentId {
    ContentId::new(&format!("theia.dev.training-{name}")).unwrap()
}
fn seed(p: &InventoryCatalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, name, count) in [(0, "sword", 1), (1, "wand", 1), (2, "stone", 13)] {
        slots[slot] = Stack::new(p.catalog(), id(name), count);
    }
    Bag::from_slots(p.catalog(), 0, slots).unwrap()
}
fn totals(b: &Bag) -> BTreeMap<ContentId, u64> {
    let mut totals = BTreeMap::new();
    for s in b.slots().iter().flatten().chain(b.main_hand()) {
        *totals.entry(s.item().clone()).or_default() += u64::from(s.count());
    }
    totals
}
async fn character(s: &mut Store, name: &str) -> Binding {
    let owner = s
        .resolve_owner("http://127.0.0.1:40123/equipment-contract", name)
        .await
        .unwrap();
    s.create_character(
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
async fn equipment_commits_once_and_shares_backpack_receipts_ownership_and_revision() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let who = character(&mut store, "Equipstore").await;
    let other = character(&mut store, "Equipother").await;
    let profile = profile();
    let initial = seed(&profile);
    let original = totals(&initial);
    store
        .ensure_inventory(who, WORLD, &profile, &initial)
        .await
        .unwrap();
    let equip = EquipmentCommand {
        expected_revision: 0,
        operation: Op::Equip { from: 0 },
    };
    // Intentionally discard success; an independent connection observes COMMIT.
    drop(
        store
            .mutate_equipment(who, WORLD, &profile, equip)
            .await
            .unwrap(),
    );
    let row = admin
        .query_one(
            "SELECT revision,state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, i64>(0), 1);
    let committed = Bag::from_json(profile.catalog(), &row.get::<_, Vec<u8>>(1)).unwrap();
    assert_eq!(committed.main_hand().unwrap().item(), &id("sword"));
    assert_eq!(totals(&committed), original);
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let replay = store
        .mutate_equipment(who, WORLD, &profile, equip)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag, committed);
    assert!(matches!(
        store
            .mutate_equipment(
                who,
                WORLD,
                &profile,
                EquipmentCommand {
                    expected_revision: 0,
                    operation: Op::Unequip { to: 0 }
                }
            )
            .await,
        Err(Error::InventoryConflict)
    ));
    assert!(matches!(
        store
            .mutate_inventory(
                who,
                WORLD,
                &profile,
                Command {
                    expected_revision: 0,
                    operation: Operation::Move { from: 0, to: 1 }
                }
            )
            .await,
        Err(Error::InventoryConflict)
    ));
    let moved = store
        .mutate_inventory(
            who,
            WORLD,
            &profile,
            Command {
                expected_revision: 1,
                operation: Operation::Move { from: 1, to: 0 },
            },
        )
        .await
        .unwrap()
        .bag;
    assert_eq!(moved.main_hand(), committed.main_hand());
    assert_eq!(moved.revision(), 2);
    assert!(matches!(
        store.mutate_equipment(who, WORLD, &profile, equip).await,
        Err(Error::InventoryStale)
    ));
    let replaced = store
        .mutate_equipment(
            who,
            WORLD,
            &profile,
            EquipmentCommand {
                expected_revision: 2,
                operation: Op::Equip { from: 0 },
            },
        )
        .await
        .unwrap()
        .bag;
    assert_eq!(replaced.main_hand().unwrap().item(), &id("wand"));
    assert_eq!(replaced.slots()[0].as_ref().unwrap().item(), &id("sword"));
    for op in [Op::Equip { from: 2 }, Op::Unequip { to: 2 }] {
        assert!(matches!(
            store
                .mutate_equipment(
                    who,
                    WORLD,
                    &profile,
                    EquipmentCommand {
                        expected_revision: 3,
                        operation: op
                    }
                )
                .await,
            Err(Error::InventoryCommand(_))
        ));
        assert_eq!(
            store.load_inventory(who, WORLD, &profile).await.unwrap(),
            replaced
        );
    }
    let returned = store
        .mutate_equipment(
            who,
            WORLD,
            &profile,
            EquipmentCommand {
                expected_revision: 3,
                operation: Op::Unequip { to: 4 },
            },
        )
        .await
        .unwrap()
        .bag;
    assert!(returned.main_hand().is_none());
    assert_eq!(totals(&returned), original);
    assert!(matches!(
        store
            .mutate_equipment(
                Binding {
                    owner: other.owner,
                    character: who.character
                },
                WORLD,
                &profile,
                equip
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.load_inventory(who, [108; 32], &profile).await,
        Err(Error::WorldMismatch)
    ));
    let mut changed: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    changed["items"][2]["name"] = serde_json::json!("Changed Sword");
    let changed = InventoryCatalog::from_json(&serde_json::to_vec(&changed).unwrap()).unwrap();
    assert!(matches!(
        store.load_inventory(who, WORLD, &changed).await,
        Err(Error::InventoryCatalogMismatch)
    ));
    assert_eq!(
        store
            .ensure_inventory(who, WORLD, &profile, &Bag::empty(profile.catalog()))
            .await
            .unwrap(),
        returned,
        "Existing state cannot be reseeded"
    );
    store
        .ensure_inventory(other, WORLD, &profile, &initial)
        .await
        .unwrap();
    assert_eq!(
        store.load_inventory(other, WORLD, &profile).await.unwrap(),
        initial
    );
    assert_eq!(
        store.load_inventory(who, WORLD, &profile).await.unwrap(),
        returned
    );
    let mut corrupt: serde_json::Value = serde_json::from_slice(&returned.to_json()).unwrap();
    corrupt.as_object_mut().unwrap().remove("main_hand");
    admin
        .execute(
            "UPDATE theia.backpacks SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[
                &serde_json::to_vec(&corrupt).unwrap(),
                &realm.value(),
                &who.character.value(),
            ],
        )
        .await
        .unwrap();
    assert!(matches!(
        store.load_inventory(who, WORLD, &profile).await,
        Err(Error::InventoryData)
    ));
    admin
        .execute(
            "UPDATE theia.backpacks SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&returned.to_json(), &realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn concurrent_backpack_and_equipment_intents_cannot_duplicate_the_same_item() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut a = Store::connect_loopback(&url, realm).await.unwrap();
    let who = character(&mut a, "Equiprace").await;
    let profile = profile();
    let initial = seed(&profile);
    a.ensure_inventory(who, WORLD, &profile, &initial)
        .await
        .unwrap();
    assert!(matches!(
        Store::connect_loopback(&url, realm).await,
        Err(Error::RealmBusy)
    ));
    let (publisher, worker) = Worker::start(a);
    let equip = EquipmentCommand {
        expected_revision: 0,
        operation: Op::Equip { from: 0 },
    };
    let movement = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 0, to: 5 },
    };
    let (ea, mb) = tokio::join!(
        publisher.mutate_equipment(who, WORLD, &profile, equip),
        publisher.mutate_inventory(who, WORLD, &profile, movement)
    );
    assert_ne!(ea.is_ok(), mb.is_ok());
    let state = if let Ok(outcome) = ea {
        assert!(matches!(mb, Err(Error::InventoryConflict)));
        assert!(
            publisher
                .mutate_equipment(who, WORLD, &profile, equip)
                .await
                .unwrap()
                .replayed
        );
        outcome.bag
    } else {
        assert!(matches!(ea, Err(Error::InventoryConflict)));
        let outcome = mb.unwrap();
        assert!(
            publisher
                .mutate_inventory(who, WORLD, &profile, movement)
                .await
                .unwrap()
                .replayed
        );
        outcome.bag
    };
    assert_eq!(state.revision(), 1);
    assert_eq!(totals(&state), totals(&initial));
    assert_eq!(
        publisher
            .load_inventory(who, WORLD, &profile)
            .await
            .unwrap(),
        state
    );
    let status = worker.shutdown().await.unwrap();
    assert_eq!(status.inventory_committed, 1);
    assert_eq!(status.inventory_replayed, 1);
    assert_eq!(status.inventory_rejected, 1);
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn blocked_backend(t: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async {
  loop {
   t.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
   if let Some(r)=t.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT catalog_hash,%'",&[]).await.unwrap() { return r.get(0); }
   tokio::time::sleep(Duration::from_millis(2)).await;
  }
 }).await.expect("Owned equipment mutation did not reach its SQL lock")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn worker_recovers_cancelled_equipment_response_and_never_acknowledges_failed_commit() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let who = character(&mut store, "Equipworker").await;
    let profile = profile();
    let initial = seed(&profile);
    store
        .ensure_inventory(who, WORLD, &profile, &initial)
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    let equip = EquipmentCommand {
        expected_revision: 0,
        operation: Op::Equip { from: 0 },
    };
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let c = profile.clone();
    let pending = tokio::spawn(async move { p.mutate_equipment(who, WORLD, &c, equip).await });
    blocked_backend(&guard).await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    guard.commit().await.unwrap();
    // A queued load waits behind executing SQL, despite the caller dropping its answer.
    let committed = publisher
        .load_inventory(who, WORLD, &profile)
        .await
        .unwrap();
    assert_eq!(committed.revision(), 1);
    assert_eq!(totals(&committed), totals(&initial));
    let recovered = publisher
        .mutate_equipment(who, WORLD, &profile, equip)
        .await
        .unwrap();
    assert!(recovered.replayed);
    assert_eq!(recovered.bag, committed);
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let c = profile.clone();
    let pending = tokio::spawn(async move {
        p.mutate_equipment(
            who,
            WORLD,
            &c,
            EquipmentCommand {
                expected_revision: 1,
                operation: Op::Unequip { to: 0 },
            },
        )
        .await
    });
    let backend = blocked_backend(&guard).await;
    assert!(
        guard
            .query_one("SELECT pg_terminate_backend($1)", &[&backend])
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    assert!(
        pending.await.unwrap().is_err(),
        "No equipment success before durable COMMIT"
    );
    guard.rollback().await.unwrap();
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_inventory(who, WORLD, &profile).await.unwrap(),
        committed
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
