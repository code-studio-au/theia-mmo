//! Owned PostgreSQL boundaries only. These synthetic earned events establish no
//! gameplay kill credit, replication, native progression or retirement proof.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::Bag,
    progression::{KillEvent, KillId, MAX_REVISION},
};
use theia_persistence::{
    Binding, CharacterName, Error, InventoryCatalog, ProgressionCatalog, RealmId, RewardCatalog,
    Store, Worker,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const WORLD: [u8; 32] = [121; 32];
fn profile() -> ProgressionCatalog {
    ProgressionCatalog::from_json(DATA).unwrap()
}
fn event(entity: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(u64::MAX).unwrap(),
            entity: EntityId::new(entity).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/progression-contract", name)
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
async fn count(admin: &tokio_postgres::Client, realm: RealmId, binding: Binding) -> i64 {
    admin
        .query_one(
            "SELECT count(*) FROM theia.experience_receipts WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &binding.character.value()],
        )
        .await
        .unwrap()
        .get(0)
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn receipts_deduplicate_old_lives_bind_recipient_world_profile_and_preserve_item_revisions() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Experienceone").await;
    let b = character(&mut store, "Experiencetwo").await;
    let mut raw: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    raw["kill_rewards"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"creature":"theia.dev.other-training-creature","experience":10}));
    let bytes = serde_json::to_vec(&raw).unwrap();
    let p = ProgressionCatalog::from_json(&bytes).unwrap();
    assert!(matches!(
        store.load_progression(a, WORLD, &p).await,
        Err(Error::ProgressionMissing)
    ));
    assert_eq!(
        store
            .ensure_progression(a, WORLD, &p)
            .await
            .unwrap()
            .total(),
        0
    );
    store.ensure_progression(b, WORLD, &p).await.unwrap();
    assert!(matches!(
        store
            .load_progression(
                Binding {
                    owner: b.owner,
                    ..a
                },
                WORLD,
                &p
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.award_experience(a, [122; 32], &p, &event(1)).await,
        Err(Error::WorldMismatch)
    ));
    let mut other_bytes = bytes.clone();
    other_bytes.push(b' ');
    let other = ProgressionCatalog::from_json(&other_bytes).unwrap();
    assert!(matches!(
        store.ensure_progression(a, WORLD, &other).await,
        Err(Error::ProgressionCatalogMismatch)
    ));
    assert!(matches!(
        store.award_experience(a, WORLD, &other, &event(1)).await,
        Err(Error::ProgressionCatalogMismatch)
    ));
    let first = store
        .award_experience(a, WORLD, &p, &event(1))
        .await
        .unwrap();
    assert_eq!(
        (
            first.state.total(),
            first.state.revision(),
            first.granted,
            first.replayed
        ),
        (25, 1, 25, false)
    );
    let second = store
        .award_experience(a, WORLD, &p, &event(2))
        .await
        .unwrap();
    assert_eq!((second.state.total(), second.state.revision()), (50, 2));
    let old = store
        .award_experience(a, WORLD, &p, &event(1))
        .await
        .unwrap();
    assert!(old.replayed);
    assert_eq!(old.granted, 0);
    assert_eq!(old.state, second.state);
    assert!(matches!(
        store.award_experience(b, WORLD, &p, &event(1)).await,
        Err(Error::ProgressionConflict)
    ));
    let changed_kind = KillEvent {
        creature: ContentId::new("theia.dev.other-training-creature").unwrap(),
        ..event(1)
    };
    assert!(matches!(
        store.award_experience(a, WORLD, &p, &changed_kind).await,
        Err(Error::ProgressionConflict)
    ));
    // Positive XP does not mutate bag state, consume a claim or require a pickup.
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let rewards = RewardCatalog::from_json(
        include_bytes!("../../../content/gameplay/development-rewards.json"),
        &items,
    )
    .unwrap();
    let bag = store
        .ensure_inventory(a, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    let claim = store
        .offer_reward(
            a,
            WORLD,
            &rewards,
            &ContentId::new("theia.dev.first-training-kill").unwrap(),
        )
        .await
        .unwrap();
    let incarnation = KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(1).unwrap(),
        }),
        ..event(1)
    };
    let third = store
        .award_experience(a, WORLD, &p, &incarnation)
        .await
        .unwrap();
    assert_eq!((third.state.total(), third.state.revision()), (75, 3));
    assert_eq!(store.load_inventory(a, WORLD, &items).await.unwrap(), bag);
    assert_eq!(
        store.list_reward_claims(a, WORLD, &rewards).await.unwrap(),
        vec![claim.clone()]
    );
    let picked = store
        .claim_reward(a, WORLD, &rewards, claim.id, 0)
        .await
        .unwrap();
    assert_eq!(picked.bag.revision(), 1);
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap(),
        third.state
    );
    for id in 3..=9 {
        store
            .award_experience(a, WORLD, &p, &event(id))
            .await
            .unwrap();
    }
    let cap = store.load_progression(a, WORLD, &p).await.unwrap();
    assert_eq!((cap.total(), cap.revision()), (250, 10));
    assert_eq!(count(&admin, realm, a).await, 10);
    for id in 100..120 {
        let result = store
            .award_experience(a, WORLD, &p, &event(id))
            .await
            .unwrap();
        assert_eq!(result.state, cap);
        assert_eq!(result.granted, 0);
        assert!(!result.replayed);
    }
    assert_eq!(count(&admin, realm, a).await, 10);
    assert_eq!(store.ensure_progression(a, WORLD, &p).await.unwrap(), cap);
    assert_eq!(
        store.load_progression(b, WORLD, &p).await.unwrap().total(),
        0
    );
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let old = store
        .award_experience(a, WORLD, &p, &event(1))
        .await
        .unwrap();
    assert!(old.replayed);
    assert_eq!(old.state, cap);
    assert_eq!(old.granted, 0);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn xp_and_receipt_roll_back_together_and_exhaustion_never_records_a_false_award() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Experiencerollback").await;
    let p = profile();
    let initial = store.ensure_progression(a, WORLD, &p).await.unwrap();
    admin.batch_execute("ALTER TABLE theia.progression ADD CONSTRAINT theia_fixture_xp_zero CHECK(experience=0) NOT VALID").await.unwrap();
    assert!(matches!(
        store.award_experience(a, WORLD, &p, &event(1)).await,
        Err(Error::Database(_))
    ));
    assert_eq!(store.load_progression(a, WORLD, &p).await.unwrap(), initial);
    assert_eq!(count(&admin, realm, a).await, 0);
    admin
        .batch_execute("ALTER TABLE theia.progression DROP CONSTRAINT theia_fixture_xp_zero")
        .await
        .unwrap();
    admin
        .execute(
            "UPDATE theia.progression SET revision=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&(MAX_REVISION as i64), &realm.value(), &a.character.value()],
        )
        .await
        .unwrap();
    assert!(matches!(
        store.award_experience(a, WORLD, &p, &event(1)).await,
        Err(Error::ProgressionCommand(
            theia_core::progression::Error::RevisionExhausted
        ))
    ));
    assert_eq!(count(&admin, realm, a).await, 0);
    admin
        .execute(
            "UPDATE theia.progression SET revision=0 WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &a.character.value()],
        )
        .await
        .unwrap();
    let applied = store
        .award_experience(a, WORLD, &p, &event(1))
        .await
        .unwrap();
    assert_eq!(applied.state.total(), 25);
    assert_eq!(count(&admin, realm, a).await, 1);
    let error=admin.execute("INSERT INTO theia.experience_receipts(realm_id,world_id,event_id,character_id,profile_hash,creature_id,granted,revision,experience) VALUES($1,$2,$3,$4,$5,'theia.dev.training-creature',1,2,26)",&[&realm.value(),&WORLD.as_slice(),&[0u8;16].as_slice(),&a.character.value(),&p.hash().as_slice()]).await.unwrap_err();
    assert_eq!(
        error.code(),
        Some(&tokio_postgres::error::SqlState::CHECK_VIOLATION)
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn blocked_backend(transaction: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async {loop {
  transaction.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
  if let Some(row)=transaction.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT world_id,profile_hash,%'",&[]).await.unwrap() {return row.get(0);}
  tokio::time::sleep(Duration::from_millis(2)).await;
 }}).await.expect("Owned XP transaction never reached its progression row lock")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn shared_worker_serializes_competing_awards_recovers_cancelled_answers_and_bounds_failure() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Experienceworker").await;
    let b = character(&mut store, "Experienceother").await;
    let p = profile();
    store.ensure_progression(a, WORLD, &p).await.unwrap();
    store.ensure_progression(b, WORLD, &p).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    let kill = event(1);
    let (first, second) = tokio::join!(
        publisher.award_experience(a, WORLD, &p, &kill),
        publisher.award_experience(b, WORLD, &p, &kill)
    );
    assert_eq!(u8::from(first.is_ok()) + u8::from(second.is_ok()), 1);
    assert!(
        matches!(first, Err(Error::ProgressionConflict))
            || matches!(second, Err(Error::ProgressionConflict))
    );
    assert!(!worker.status().failed);
    let before = publisher.load_progression(a, WORLD, &p).await.unwrap();
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let recipe = p.clone();
    let lost =
        tokio::spawn(async move { cloned.award_experience(a, WORLD, &recipe, &event(2)).await });
    blocked_backend(&guard).await;
    assert!(!lost.is_finished());
    lost.abort();
    let _ = lost.await;
    guard.commit().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while worker.status().experience_committed < 2 {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .unwrap();
    let recovered = publisher
        .award_experience(a, WORLD, &p, &event(2))
        .await
        .unwrap();
    assert!(recovered.replayed);
    assert_eq!(recovered.granted, 0);
    assert_eq!(recovered.state.total(), before.total() + 25);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let recipe = p.clone();
    let pending =
        tokio::spawn(async move { cloned.award_experience(a, WORLD, &recipe, &event(3)).await });
    blocked_backend(&guard).await;
    let mut queued = Vec::new();
    for id in 100..124 {
        let cloned = publisher.clone();
        let recipe = p.clone();
        queued.push(tokio::spawn(async move {
            cloned.award_experience(a, WORLD, &recipe, &event(id)).await
        }));
    }
    tokio::time::sleep(Duration::from_millis(30)).await;
    let mut rejected = 0;
    for task in queued {
        if task.is_finished() {
            assert!(matches!(task.await.unwrap(), Err(Error::InventoryBusy)));
            rejected += 1;
        } else {
            task.abort();
            let _ = task.await;
        }
    }
    assert!(
        rejected >= 8,
        "The existing sixteen-job critical queue must reject overflow"
    );
    guard.commit().await.unwrap();
    let committed = pending.await.unwrap().unwrap().state;
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let recipe = p.clone();
    let pending =
        tokio::spawn(async move { cloned.award_experience(a, WORLD, &recipe, &event(4)).await });
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
        "A terminated backend cannot acknowledge an XP award"
    );
    guard.rollback().await.unwrap();
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap(),
        committed
    );
    let retry = store
        .award_experience(a, WORLD, &p, &event(4))
        .await
        .unwrap();
    assert!(!retry.replayed);
    assert_eq!(retry.granted, 25);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
