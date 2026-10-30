//! Actual owned PostgreSQL only. No native, OIDC or replicated backpack claim.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, Operation, Stack},
};
use theia_persistence::{Binding, CharacterName, Error, InventoryCatalog, RealmId, Store, Worker};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-items.json");
const WORLD: [u8; 32] = [91; 32];
fn profile() -> InventoryCatalog {
    InventoryCatalog::from_json(DATA).unwrap()
}
fn seed(profile: &InventoryCatalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, suffix, count) in [(0, "stone", 13), (1, "stone", 15), (2, "token", 99)] {
        slots[slot] = Stack::new(
            profile.catalog(),
            ContentId::new(&format!("theia.dev.training-{suffix}")).unwrap(),
            count,
        );
    }
    Bag::from_slots(profile.catalog(), 0, slots).unwrap()
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/inventory-contract", name)
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
async fn owned_backpacks_commit_once_bind_catalog_and_preserve_state_across_owner_reconnect() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let binding = character(&mut store, "Bagstore").await;
    let other = character(&mut store, "Bagother").await;
    let profile = profile();
    let initial = seed(&profile);
    assert!(matches!(
        store.load_inventory(binding, WORLD, &profile).await,
        Err(Error::InventoryMissing)
    ));
    assert_eq!(
        store
            .ensure_inventory(binding, WORLD, &profile, &initial)
            .await
            .unwrap(),
        initial
    );
    let request = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 0, to: 1 },
    };
    let applied = store
        .mutate_inventory(binding, WORLD, &profile, request)
        .await
        .unwrap();
    assert!(!applied.replayed);
    assert_eq!(applied.bag.revision(), 1);
    let bytes: Vec<u8> = admin
        .query_one(
            "SELECT state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &binding.character.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        Bag::from_json(profile.catalog(), &bytes).unwrap(),
        applied.bag,
        "Independent connection observes COMMIT before success"
    );
    assert_eq!(
        store
            .ensure_inventory(binding, WORLD, &profile, &Bag::empty(profile.catalog()))
            .await
            .unwrap(),
        applied.bag,
        "Re-entry cannot reseed contents"
    );
    let replay = store
        .mutate_inventory(binding, WORLD, &profile, request)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag, applied.bag);
    let conflict = Command {
        operation: Operation::Move { from: 0, to: 3 },
        ..request
    };
    assert!(matches!(
        store
            .mutate_inventory(binding, WORLD, &profile, conflict)
            .await,
        Err(Error::InventoryConflict)
    ));
    let invalid = Command {
        expected_revision: 1,
        operation: Operation::Split {
            from: 1,
            to: 3,
            amount: 0,
        },
    };
    assert!(matches!(
        store
            .mutate_inventory(binding, WORLD, &profile, invalid)
            .await,
        Err(Error::InventoryCommand(_))
    ));
    let foreign = Binding {
        owner: other.owner,
        ..binding
    };
    assert!(matches!(
        store.load_inventory(foreign, WORLD, &profile).await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .ensure_inventory(foreign, WORLD, &profile, &initial)
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .mutate_inventory(foreign, WORLD, &profile, request)
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.load_inventory(binding, [92; 32], &profile).await,
        Err(Error::WorldMismatch)
    ));
    // Whitespace changes the byte identity even when all logical definitions match.
    let mut changed = DATA.to_vec();
    changed.push(b' ');
    let changed = InventoryCatalog::from_json(&changed).unwrap();
    assert_eq!(profile.catalog(), changed.catalog());
    assert_ne!(profile.hash(), changed.hash());
    assert!(matches!(
        store.load_inventory(binding, WORLD, &changed).await,
        Err(Error::InventoryCatalogMismatch)
    ));
    let second = Command {
        expected_revision: 1,
        operation: Operation::Split {
            from: 1,
            to: 15,
            amount: 7,
        },
    };
    let final_bag = store
        .mutate_inventory(binding, WORLD, &profile, second)
        .await
        .unwrap()
        .bag;
    assert!(matches!(
        store
            .mutate_inventory(binding, WORLD, &profile, request)
            .await,
        Err(Error::InventoryStale)
    ));
    store.close().await.unwrap();
    let mut reopened = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        reopened
            .load_inventory(binding, WORLD, &profile)
            .await
            .unwrap(),
        final_bag
    );
    assert!(
        reopened
            .mutate_inventory(binding, WORLD, &profile, second)
            .await
            .unwrap()
            .replayed
    );
    // Database constraints and the checked reader fail closed on corrupt state.
    assert!(
        admin
            .execute(
                "UPDATE theia.backpacks SET revision=-1 WHERE realm_id=$1 AND character_id=$2",
                &[&realm.value(), &binding.character.value()]
            )
            .await
            .is_err()
    );
    admin
        .execute(
            "UPDATE theia.backpacks SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[
                &b"{}".as_slice(),
                &realm.value(),
                &binding.character.value(),
            ],
        )
        .await
        .unwrap();
    assert!(matches!(
        reopened.load_inventory(binding, WORLD, &profile).await,
        Err(Error::InventoryData)
    ));
    admin
        .execute(
            "UPDATE theia.backpacks SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[
                &final_bag.to_json(),
                &realm.value(),
                &binding.character.value(),
            ],
        )
        .await
        .unwrap();
    reopened.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn locked_backend(transaction: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_millis(300),async {
        loop {
            transaction.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
            if let Some(row) = transaction.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT catalog_hash,%'",&[]).await.unwrap() {return row.get(0)}
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }).await.expect("Owned inventory transaction never waited on fixture row")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn worker_serializes_conflicts_recovers_a_dropped_response_and_fails_closed_on_backend_loss()
{
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let binding = character(&mut store, "Bagworker").await;
    let profile = profile();
    store
        .ensure_inventory(binding, WORLD, &profile, &seed(&profile))
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    let a = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 0, to: 3 },
    };
    let b = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 0, to: 4 },
    };
    let (ra, rb) = tokio::join!(
        publisher.mutate_inventory(binding, WORLD, &profile, a),
        publisher.mutate_inventory(binding, WORLD, &profile, b)
    );
    assert_ne!(ra.is_ok(), rb.is_ok());
    let (winner, rejected) = if ra.is_ok() { (a, rb) } else { (b, ra) };
    assert!(matches!(rejected, Err(Error::InventoryConflict)));
    assert!(
        publisher
            .mutate_inventory(binding, WORLD, &profile, winner)
            .await
            .unwrap()
            .replayed
    );
    // Drop the caller only after the real mutation is executing in PostgreSQL.
    let receipt_lost = Command {
        expected_revision: 1,
        operation: Operation::Split {
            from: 1,
            to: 6,
            amount: 3,
        },
    };
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &binding.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let c = profile.clone();
    let pending =
        tokio::spawn(async move { p.mutate_inventory(binding, WORLD, &c, receipt_lost).await });
    locked_backend(&guard).await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    guard.commit().await.unwrap();
    let committed = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let bag = publisher
                .load_inventory(binding, WORLD, &profile)
                .await
                .unwrap();
            if bag.revision() == 2 {
                break bag;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let recovered = publisher
        .mutate_inventory(binding, WORLD, &profile, receipt_lost)
        .await
        .unwrap();
    assert!(recovered.replayed);
    assert_eq!(recovered.bag, committed);
    assert_eq!(committed.slots()[6].as_ref().unwrap().count(), 3);
    // Block the next actual mutation, saturate the fixed queue, then terminate
    // only this owned worker backend before it can commit or report success.
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &binding.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let c = profile.clone();
    let pending = tokio::spawn(async move {
        p.mutate_inventory(
            binding,
            WORLD,
            &c,
            Command {
                expected_revision: 2,
                operation: Operation::Move { from: 6, to: 7 },
            },
        )
        .await
    });
    let backend = locked_backend(&guard).await;
    let mut queued = Vec::new();
    for _ in 0..64 {
        let p = publisher.clone();
        let c = profile.clone();
        queued.push(tokio::spawn(async move {
            p.load_inventory(binding, WORLD, &c).await
        }));
    }
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(
        queued.iter().any(|job| job.is_finished()),
        "Full queue must reject work immediately"
    );
    let terminated: bool = guard
        .query_one("SELECT pg_terminate_backend($1)", &[&backend])
        .await
        .unwrap()
        .get(0);
    assert!(terminated);
    assert!(
        pending.await.unwrap().is_err(),
        "No acknowledgement before COMMIT"
    );
    guard.rollback().await.unwrap();
    let mut busy = 0;
    for job in queued {
        let result = job.await.unwrap();
        busy += usize::from(matches!(result, Err(Error::InventoryBusy)));
        assert!(result.is_err());
    }
    assert!(busy > 0);
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .load_inventory(binding, WORLD, &profile)
            .await
            .unwrap(),
        committed
    );
    assert!(
        store
            .mutate_inventory(binding, WORLD, &profile, receipt_lost)
            .await
            .unwrap()
            .replayed
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
