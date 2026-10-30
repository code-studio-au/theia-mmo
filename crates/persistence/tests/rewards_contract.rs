//! Actual owned PostgreSQL only; qualifying kills/network/claim UI are not tested.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, EquipmentCommand, EquipmentOperation, Operation, Stack},
    rewards,
};
use theia_persistence::{
    Binding, CharacterName, ClaimId, Error, InventoryCatalog, RealmId, RewardCatalog, Store, Worker,
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-rewards.json");
const WORLD: [u8; 32] = [117; 32];
fn profiles() -> (InventoryCatalog, RewardCatalog) {
    let items = InventoryCatalog::from_json(ITEMS).unwrap();
    let rewards = RewardCatalog::from_json(DATA, &items).unwrap();
    (items, rewards)
}
fn reward() -> ContentId {
    ContentId::new("theia.dev.first-training-kill").unwrap()
}
fn id(name: &str) -> ContentId {
    ContentId::new(&format!("theia.dev.training-{name}")).unwrap()
}
fn seed(p: &InventoryCatalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(p.catalog(), id("token"), 99);
    slots[1] = Stack::new(p.catalog(), id("stone"), 13);
    Bag::from_parts(
        p.catalog(),
        0,
        slots,
        Stack::new(p.catalog(), id("sword"), 1),
    )
    .unwrap()
}
fn tokens(b: &Bag) -> u32 {
    b.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id("token"))
        .map(Stack::count)
        .sum()
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/reward-contract", name)
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
async fn claims_commit_once_bind_owner_world_profiles_and_survive_reconnect_without_reoffering() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let who = character(&mut store, "Rewardstore").await;
    let other = character(&mut store, "Rewardother").await;
    let (items, profile) = profiles();
    let initial = seed(&items);
    assert!(matches!(
        store.offer_reward(who, WORLD, &profile, &reward()).await,
        Err(Error::InventoryMissing)
    ));
    store
        .ensure_inventory(who, WORLD, &items, &initial)
        .await
        .unwrap();
    store
        .ensure_inventory(other, WORLD, &items, &initial)
        .await
        .unwrap();
    let claim = store
        .offer_reward(who, WORLD, &profile, &reward())
        .await
        .unwrap();
    assert!(claim.consumed_revision.is_none());
    assert_eq!(
        store
            .offer_reward(who, WORLD, &profile, &reward())
            .await
            .unwrap(),
        claim
    );
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &profile)
            .await
            .unwrap(),
        vec![claim.clone()]
    );
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        initial
    );
    let other_claim = store
        .offer_reward(other, WORLD, &profile, &reward())
        .await
        .unwrap();
    assert_ne!(claim.id, other_claim.id);
    let row = admin
        .query_one(
            "SELECT count(*) FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    assert_eq!(
        row.get::<_, i64>(0),
        1,
        "Availability is already visible on an independent connection"
    );
    assert!(matches!(
        store
            .claim_reward(
                Binding {
                    owner: other.owner,
                    character: who.character
                },
                WORLD,
                &profile,
                claim.id,
                0
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .claim_reward(who, WORLD, &profile, other_claim.id, 0)
            .await,
        Err(Error::RewardMissing)
    ));
    assert!(matches!(
        store
            .claim_reward(who, [118; 32], &profile, claim.id, 0)
            .await,
        Err(Error::WorldMismatch)
    ));
    assert!(matches!(
        store
            .claim_reward(who, WORLD, &profile, ClaimId::new([91; 16]).unwrap(), 0)
            .await,
        Err(Error::RewardMissing)
    ));
    let mut changed: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    changed["rewards"][0]["grants"][0]["count"] = serde_json::json!(2);
    let changed = RewardCatalog::from_json(&serde_json::to_vec(&changed).unwrap(), &items).unwrap();
    assert!(matches!(
        store.offer_reward(who, WORLD, &changed, &reward()).await,
        Err(Error::RewardCatalogMismatch)
    ));
    assert!(matches!(
        store.list_reward_claims(who, WORLD, &changed).await,
        Err(Error::RewardCatalogMismatch)
    ));
    assert!(matches!(
        store.claim_reward(who, WORLD, &changed, claim.id, 0).await,
        Err(Error::RewardCatalogMismatch)
    ));
    drop(
        store
            .claim_reward(who, WORLD, &profile, claim.id, 0)
            .await
            .unwrap(),
    );
    let row=admin.query_one("SELECT b.state,c.consumed_revision,b.last_command=c.consumed_command FROM theia.backpacks b JOIN theia.reward_claims c USING(realm_id,character_id) WHERE b.realm_id=$1 AND b.character_id=$2",&[&realm.value(),&who.character.value()]).await.unwrap();
    let committed = Bag::from_json(items.catalog(), &row.get::<_, Vec<u8>>(0)).unwrap();
    assert_eq!(tokens(&committed), 100);
    assert_eq!(committed.main_hand(), initial.main_hand());
    assert_eq!(row.get::<_, Option<i64>>(1), Some(1));
    assert!(row.get::<_, bool>(2));
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let replay = store
        .claim_reward(who, WORLD, &profile, claim.id, 0)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag, committed);
    let same = store
        .offer_reward(who, WORLD, &profile, &reward())
        .await
        .unwrap();
    assert_eq!(same.id, claim.id);
    assert_eq!(same.consumed_revision, Some(1));
    assert!(matches!(
        store.claim_reward(who, WORLD, &profile, claim.id, 1).await,
        Err(Error::RewardConsumed)
    ));
    let moved = store
        .mutate_equipment(
            who,
            WORLD,
            &items,
            EquipmentCommand {
                expected_revision: 1,
                operation: EquipmentOperation::Unequip { to: 4 },
            },
        )
        .await
        .unwrap()
        .bag;
    assert_eq!(tokens(&moved), 100);
    assert_eq!(moved.revision(), 2);
    assert!(matches!(
        store.claim_reward(who, WORLD, &profile, claim.id, 0).await,
        Err(Error::InventoryStale)
    ));
    assert!(matches!(
        store.claim_reward(who, WORLD, &profile, claim.id, 2).await,
        Err(Error::RewardConsumed)
    ));
    assert_eq!(
        store.load_inventory(other, WORLD, &items).await.unwrap(),
        initial
    );
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &profile)
            .await
            .unwrap()[0]
            .consumed_revision,
        Some(1)
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn full_bags_preserve_available_claims_and_both_database_updates_roll_back_together() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let who = character(&mut store, "Rewardfull").await;
    let (items, profile) = profiles();
    let mut slots = std::array::from_fn(|_| Stack::new(items.catalog(), id("token"), 99));
    slots[0] = Stack::new(items.catalog(), id("sword"), 1);
    let full = Bag::from_slots(items.catalog(), 0, slots).unwrap();
    store
        .ensure_inventory(who, WORLD, &items, &full)
        .await
        .unwrap();
    let claim = store
        .offer_reward(who, WORLD, &profile, &reward())
        .await
        .unwrap();
    assert!(matches!(
        store.claim_reward(who, WORLD, &profile, claim.id, 0).await,
        Err(Error::RewardCommand(rewards::Error::NoCapacity))
    ));
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        full
    );
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &profile)
            .await
            .unwrap(),
        vec![claim.clone()]
    );
    let room = store
        .mutate_equipment(
            who,
            WORLD,
            &items,
            EquipmentCommand {
                expected_revision: 0,
                operation: EquipmentOperation::Equip { from: 0 },
            },
        )
        .await
        .unwrap()
        .bag;
    // An owned test constraint fails the second SQL update, after the tentative
    // bag write, proving transaction rollback instead of a half-committed award.
    admin.batch_execute("ALTER TABLE theia.reward_claims ADD CONSTRAINT theia_fixture_no_reward_consume CHECK(consumed_revision IS NULL) NOT VALID").await.unwrap();
    assert!(matches!(
        store.claim_reward(who, WORLD, &profile, claim.id, 1).await,
        Err(Error::Database(_))
    ));
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        room
    );
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &profile)
            .await
            .unwrap(),
        vec![claim.clone()]
    );
    admin
        .batch_execute(
            "ALTER TABLE theia.reward_claims DROP CONSTRAINT theia_fixture_no_reward_consume",
        )
        .await
        .unwrap();
    let granted = store
        .claim_reward(who, WORLD, &profile, claim.id, 1)
        .await
        .unwrap();
    assert!(!granted.replayed);
    assert_eq!(granted.bag.revision(), 2);
    assert_eq!(tokens(&granted.bag), tokens(&room) + 1);
    assert_eq!(granted.bag.main_hand(), room.main_hand());
    assert!(
        store
            .claim_reward(who, WORLD, &profile, claim.id, 1)
            .await
            .unwrap()
            .replayed
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn blocked_backend(transaction: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async {
  loop {
   transaction.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
   if let Some(row)=transaction.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT catalog_hash,%'",&[]).await.unwrap(){return row.get(0);}
   tokio::time::sleep(Duration::from_millis(2)).await;
  }
 }).await.expect("Owned claim transaction never reached its bag lock")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn shared_worker_serializes_claims_recovers_cancelled_answers_and_fails_closed_before_commit()
{
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, profile) = profiles();
    let initial = seed(&items);
    let mut people = Vec::new();
    for name in ["Rewardrace", "Rewardcancel", "Rewardfailure"] {
        let who = character(&mut store, name).await;
        store
            .ensure_inventory(who, WORLD, &items, &initial)
            .await
            .unwrap();
        let claim = store
            .offer_reward(who, WORLD, &profile, &reward())
            .await
            .unwrap();
        people.push((who, claim.id));
    }
    let (publisher, worker) = Worker::start(store);
    let (a, claim_a) = people[0];
    let movement = Command {
        expected_revision: 0,
        operation: Operation::Move { from: 1, to: 4 },
    };
    let (claimed, moved) = tokio::join!(
        publisher.claim_reward(a, WORLD, &profile, claim_a, 0),
        publisher.mutate_inventory(a, WORLD, &items, movement)
    );
    assert_ne!(claimed.is_ok(), moved.is_ok());
    if claimed.is_ok() {
        assert!(matches!(moved, Err(Error::InventoryConflict)));
        assert!(
            publisher
                .claim_reward(a, WORLD, &profile, claim_a, 0)
                .await
                .unwrap()
                .replayed
        );
    } else {
        assert!(matches!(claimed, Err(Error::InventoryConflict)));
        assert!(
            publisher
                .mutate_inventory(a, WORLD, &items, movement)
                .await
                .unwrap()
                .replayed
        );
    }
    let (b, claim_b) = people[1];
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &b.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let c = profile.clone();
    let pending = tokio::spawn(async move { p.claim_reward(b, WORLD, &c, claim_b, 0).await });
    blocked_backend(&guard).await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    guard.commit().await.unwrap();
    let committed = publisher.load_inventory(b, WORLD, &items).await.unwrap();
    assert_eq!(tokens(&committed), 100);
    assert_eq!(committed.revision(), 1);
    assert!(
        publisher
            .claim_reward(b, WORLD, &profile, claim_b, 0)
            .await
            .unwrap()
            .replayed
    );
    assert_eq!(
        publisher
            .list_reward_claims(b, WORLD, &profile)
            .await
            .unwrap()[0]
            .consumed_revision,
        Some(1)
    );
    let (c, claim_c) = people[2];
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &c.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let recipe = profile.clone();
    let pending = tokio::spawn(async move { p.claim_reward(c, WORLD, &recipe, claim_c, 0).await });
    let backend = blocked_backend(&guard).await;
    let mut queued = Vec::new();
    for _ in 0..64 {
        let p = publisher.clone();
        let recipe = profile.clone();
        queued.push(tokio::spawn(async move {
            p.list_reward_claims(c, WORLD, &recipe).await
        }));
    }
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(
        queued.iter().any(|t| t.is_finished()),
        "Bounded reward queue must reject overflow immediately"
    );
    assert!(
        guard
            .query_one("SELECT pg_terminate_backend($1)", &[&backend])
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    assert!(
        pending.await.unwrap().is_err(),
        "No claim success before COMMIT"
    );
    guard.rollback().await.unwrap();
    let mut busy = 0;
    for task in queued {
        let result = task.await.unwrap();
        busy += usize::from(matches!(result, Err(Error::InventoryBusy)));
        assert!(result.is_err());
    }
    assert!(busy > 0);
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_inventory(c, WORLD, &items).await.unwrap(),
        initial
    );
    assert_eq!(
        store.list_reward_claims(c, WORLD, &profile).await.unwrap()[0].consumed_revision,
        None
    );
    assert_eq!(
        store.load_inventory(b, WORLD, &items).await.unwrap(),
        committed
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
