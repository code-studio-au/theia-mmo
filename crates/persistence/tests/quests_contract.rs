//! Real owned SQL/worker contract. Synthetic facts do not establish live credit.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
};
use theia_persistence::{
    Binding, CharacterName, Error, ProgressionCatalog, QuestCatalog, RealmId, Store, Worker,
};
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const QUEST: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
const WORLD: [u8; 32] = [131; 32];
fn event(id: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(u64::MAX).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
fn xp() -> ProgressionCatalog {
    ProgressionCatalog::from_json(XP).unwrap()
}
fn quest() -> QuestCatalog {
    QuestCatalog::from_json(QUEST).unwrap()
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/quest-contract", name)
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
async fn receipts(admin: &tokio_postgres::Client, realm: RealmId, binding: Binding) -> (i64, i64) {
    let row=admin.query_one("SELECT (SELECT count(*) FROM theia.experience_receipts WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.quest_kill_receipts WHERE realm_id=$1 AND character_id=$2)",&[&realm.value(),&binding.character.value()]).await.unwrap();
    (row.get(0), row.get(1))
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn objective_receipts_bind_current_profile_recipient_and_survive_xp_cap_and_later_kills() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Questone").await;
    let b = character(&mut store, "Questtwo").await;
    let mut raw: serde_json::Value = serde_json::from_slice(XP).unwrap();
    raw["kill_rewards"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"creature":"theia.dev.other-creature","experience":10}));
    let p = ProgressionCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    let q = quest();
    assert!(matches!(
        store.load_quest(a, WORLD, &q).await,
        Err(Error::QuestMissing)
    ));
    for binding in [a, b] {
        store.ensure_progression(binding, WORLD, &p).await.unwrap();
        assert_eq!(
            store
                .ensure_quest(binding, WORLD, &q)
                .await
                .unwrap()
                .count(),
            0
        );
    }
    assert!(matches!(
        store
            .load_quest(
                Binding {
                    owner: b.owner,
                    ..a
                },
                WORLD,
                &q
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .award_progression_and_quest(a, [132; 32], &p, &q, &event(1))
            .await,
        Err(Error::WorldMismatch)
    ));
    let padded = QuestCatalog::from_json(&[QUEST, b"\n"].concat()).unwrap();
    assert!(matches!(
        store
            .award_progression_and_quest(a, WORLD, &p, &padded, &event(1))
            .await,
        Err(Error::QuestCatalogMismatch)
    ));
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap().total(),
        0
    );
    assert_eq!(receipts(&admin, realm, a).await, (0, 0));
    for id in 1..=2 {
        let r = store
            .award_progression_and_quest(a, WORLD, &p, &q, &event(id))
            .await
            .unwrap();
        assert_eq!(r.experience.state.total(), id as u32 * 25);
        assert_eq!(r.quest.state.count(), id as u32);
        assert!(r.quest.advanced);
        assert!(!r.quest.completed_now);
    }
    let old = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(1))
        .await
        .unwrap();
    assert!(old.experience.replayed && old.quest.replayed);
    assert!(!old.quest.advanced);
    assert_eq!(old.experience.granted, 0);
    assert_eq!(old.quest.state.count(), 2);
    assert!(matches!(
        store
            .award_progression_and_quest(b, WORLD, &p, &q, &event(1))
            .await,
        Err(Error::ProgressionConflict)
    ));
    let unrelated = KillEvent {
        creature: ContentId::new("theia.dev.other-creature").unwrap(),
        ..event(20)
    };
    let r = store
        .award_progression_and_quest(a, WORLD, &p, &q, &unrelated)
        .await
        .unwrap();
    assert_eq!(r.experience.granted, 10);
    assert!(!r.quest.advanced);
    assert_eq!(r.quest.state.count(), 2);
    let r = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(3))
        .await
        .unwrap();
    assert!(r.quest.completed_now);
    assert_eq!(r.quest.state.revision(), 3);
    assert_eq!(r.experience.state.total(), 85);
    store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(4))
        .await
        .unwrap();
    assert_eq!(receipts(&admin, realm, a).await, (5, 3));
    assert_eq!(
        store.ensure_quest(a, WORLD, &q).await.unwrap(),
        r.quest.state
    );
    let c = character(&mut store, "Questcapped").await;
    raw["level_thresholds"] = serde_json::json!([0, 25]);
    let cap = ProgressionCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    store.ensure_progression(c, WORLD, &cap).await.unwrap();
    store.ensure_quest(c, WORLD, &q).await.unwrap();
    for id in 100..103 {
        let r = store
            .award_progression_and_quest(c, WORLD, &cap, &q, &event(id))
            .await
            .unwrap();
        assert_eq!(r.experience.state.total(), 25);
        assert_eq!(r.experience.state.revision(), 1);
        assert!(r.quest.advanced);
        assert_eq!(r.quest.completed_now, id == 102);
    }
    for id in 103..123 {
        let r = store
            .award_progression_and_quest(c, WORLD, &cap, &q, &event(id))
            .await
            .unwrap();
        assert_eq!(r.experience.granted, 0);
        assert!(!r.quest.advanced);
        assert_eq!(r.quest.state.count(), 3);
    }
    assert_eq!(receipts(&admin, realm, c).await, (1, 3));
    let old = store
        .award_progression_and_quest(c, WORLD, &cap, &q, &event(100))
        .await
        .unwrap();
    assert!(old.experience.replayed && old.quest.replayed);
    assert_eq!(old.quest.state.count(), 3);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn either_effect_failure_rolls_back_both_receipts_and_both_progression_states() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Questatomic").await;
    let p = xp();
    let q = quest();
    store.ensure_progression(a, WORLD, &p).await.unwrap();
    store.ensure_quest(a, WORLD, &q).await.unwrap();
    admin.batch_execute("ALTER TABLE theia.quest_objectives ADD CONSTRAINT theia_test_no_quest CHECK(kill_count=0)").await.unwrap();
    assert!(
        store
            .award_progression_and_quest(a, WORLD, &p, &q, &event(1))
            .await
            .is_err()
    );
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap().total(),
        0
    );
    assert_eq!(store.load_quest(a, WORLD, &q).await.unwrap().count(), 0);
    assert_eq!(receipts(&admin, realm, a).await, (0, 0));
    admin
        .batch_execute("ALTER TABLE theia.quest_objectives DROP CONSTRAINT theia_test_no_quest")
        .await
        .unwrap();
    let first = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(1))
        .await
        .unwrap();
    assert!(!first.experience.replayed && !first.quest.replayed);
    admin
        .batch_execute(
            "ALTER TABLE theia.progression ADD CONSTRAINT theia_test_no_xp CHECK(experience=25)",
        )
        .await
        .unwrap();
    assert!(
        store
            .award_progression_and_quest(a, WORLD, &p, &q, &event(2))
            .await
            .is_err()
    );
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap(),
        first.experience.state
    );
    assert_eq!(
        store.load_quest(a, WORLD, &q).await.unwrap(),
        first.quest.state
    );
    assert_eq!(receipts(&admin, realm, a).await, (1, 1));
    admin
        .batch_execute("ALTER TABLE theia.progression DROP CONSTRAINT theia_test_no_xp")
        .await
        .unwrap();
    let second = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(2))
        .await
        .unwrap();
    assert_eq!(second.quest.state.count(), 2);
    let b = character(&mut store, "Questmissing").await;
    store.ensure_progression(b, WORLD, &p).await.unwrap();
    assert!(matches!(
        store
            .award_progression_and_quest(b, WORLD, &p, &q, &event(5))
            .await,
        Err(Error::QuestMissing)
    ));
    assert_eq!(
        store.load_progression(b, WORLD, &p).await.unwrap().total(),
        0
    );
    assert_eq!(receipts(&admin, realm, b).await, (0, 0));
    store.ensure_quest(b, WORLD, &q).await.unwrap();
    store
        .award_progression_and_quest(b, WORLD, &p, &q, &event(5))
        .await
        .unwrap();
    admin.execute("UPDATE theia.quest_kill_receipts SET kill_count=1 WHERE realm_id=$1 AND character_id=$2 AND revision=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert!(matches!(
        store
            .award_progression_and_quest(a, WORLD, &p, &q, &event(2))
            .await,
        Err(Error::QuestData)
    ));
    admin.execute("UPDATE theia.quest_kill_receipts SET kill_count=2 WHERE realm_id=$1 AND character_id=$2 AND revision=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    let retry = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(2))
        .await
        .unwrap();
    assert!(retry.experience.replayed && retry.quest.replayed);
    let error=admin.execute("INSERT INTO theia.quest_kill_receipts(realm_id,world_id,event_id,character_id,profile_hash,creature_id,revision,kill_count) VALUES($1,$2,$3,$4,$5,'theia.dev.training-creature',3,3)",&[&realm.value(),&WORLD.as_slice(),&[0u8;16].as_slice(),&a.character.value(),&q.hash().as_slice()]).await.unwrap_err();
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
async fn shared_worker_bundles_competing_credit_recovers_lost_answers_and_fails_backend_closed() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Questworker").await;
    let b = character(&mut store, "Questother").await;
    let p = xp();
    let mut raw: serde_json::Value = serde_json::from_slice(QUEST).unwrap();
    raw["objective"]["required"] = serde_json::json!(100);
    let q = QuestCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    store.ensure_progression(a, WORLD, &p).await.unwrap();
    store.ensure_progression(b, WORLD, &p).await.unwrap();
    store.ensure_quest(a, WORLD, &q).await.unwrap();
    store.ensure_quest(b, WORLD, &q).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    let kill = event(1);
    let (first, second) = tokio::join!(
        publisher.award_progression_and_quest(a, WORLD, &p, &q, &kill),
        publisher.award_progression_and_quest(b, WORLD, &p, &q, &kill)
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
    let objective = q.clone();
    let lost = tokio::spawn(async move {
        cloned
            .award_progression_and_quest(a, WORLD, &recipe, &objective, &event(2))
            .await
    });
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
        .award_progression_and_quest(a, WORLD, &p, &q, &event(2))
        .await
        .unwrap();
    assert!(recovered.experience.replayed && recovered.quest.replayed);
    assert_eq!(recovered.experience.granted, 0);
    assert_eq!(recovered.experience.state.total(), before.total() + 25);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let recipe = p.clone();
    let objective = q.clone();
    let pending = tokio::spawn(async move {
        cloned
            .award_progression_and_quest(a, WORLD, &recipe, &objective, &event(3))
            .await
    });
    blocked_backend(&guard).await;
    let mut queued = Vec::new();
    for id in 100..124 {
        let cloned = publisher.clone();
        let recipe = p.clone();
        let objective = q.clone();
        queued.push(tokio::spawn(async move {
            cloned
                .award_progression_and_quest(a, WORLD, &recipe, &objective, &event(id))
                .await
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
    let committed = pending.await.unwrap().unwrap();
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let recipe = p.clone();
    let objective = q.clone();
    let pending = tokio::spawn(async move {
        cloned
            .award_progression_and_quest(a, WORLD, &recipe, &objective, &event(4))
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
        "A terminated backend cannot acknowledge an XP award"
    );
    guard.rollback().await.unwrap();
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap(),
        committed.experience.state
    );
    assert_eq!(
        store.load_quest(a, WORLD, &q).await.unwrap(),
        committed.quest.state
    );
    let retry = store
        .award_progression_and_quest(a, WORLD, &p, &q, &event(4))
        .await
        .unwrap();
    assert!(!retry.experience.replayed && !retry.quest.replayed);
    assert_eq!(retry.experience.granted, 25);
    assert_eq!(retry.quest.state.count(), committed.quest.state.count() + 1);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
