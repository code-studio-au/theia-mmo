//! Actual owned SQL, trusted synthetic facts; not live NPC/kill-credit evidence.
mod support;
use std::time::Duration;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
    quests::lifecycle::{Action, Command, Phase},
};
use theia_persistence::{
    Binding, CharacterName, Error, ProgressionCatalog, QuestLifecycleCatalog, RealmId, Store,
    Worker,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const WORLD: [u8; 32] = [141; 32];
fn profile() -> QuestLifecycleCatalog {
    QuestLifecycleCatalog::from_json(DATA).unwrap()
}
fn xp(capped: bool) -> ProgressionCatalog {
    let mut raw: serde_json::Value = serde_json::from_slice(XP).unwrap();
    if capped {
        raw["level_thresholds"] = serde_json::json!([0, 25]);
    }
    ProgressionCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap()
}
fn event(id: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(71).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
fn accept() -> Command {
    Command {
        expected_revision: 0,
        action: Action::Accept,
    }
}
fn turn_in() -> Command {
    Command {
        expected_revision: 4,
        action: Action::TurnIn,
    }
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/lifecycle-contract", name)
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
async fn counts(
    admin: &tokio_postgres::Client,
    realm: RealmId,
    binding: Binding,
) -> (i64, i64, i64) {
    let r=admin.query_one("SELECT (SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.experience_receipts WHERE realm_id=$1 AND character_id=$2)",&[&realm.value(),&binding.character.value()]).await.unwrap();
    (r.get(0), r.get(1), r.get(2))
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn capped_preaccept_kills_cannot_retroactively_count_and_action_receipts_survive_later_progress()
 {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Lifecycleone").await;
    let b = character(&mut store, "Lifecycletwo").await;
    let p = xp(true);
    let q = profile();
    for binding in [a, b] {
        store.ensure_progression(binding, WORLD, &p).await.unwrap();
        store
            .ensure_quest_lifecycle(binding, WORLD, &q)
            .await
            .unwrap();
    }
    for id in 1..=2 {
        let r = store
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(id))
            .await
            .unwrap();
        assert_eq!(
            r.quest.state.progress(q.profile()).unwrap().phase,
            Phase::Available
        );
        assert!(!r.quest.changed);
        assert_eq!(r.experience.state.total(), 25);
    }
    assert_eq!(counts(&admin, realm, a).await, (0, 2, 1));
    let accepted = store
        .command_quest_lifecycle(a, WORLD, &q, accept())
        .await
        .unwrap();
    assert_eq!(accepted.state.revision(), 1);
    for id in 1..=2 {
        let r = store
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(id))
            .await
            .unwrap();
        assert!(r.quest.replayed);
        assert_eq!(r.quest.state.count(), 0);
        assert_eq!(r.experience.granted, 0);
    }
    for id in 3..=5 {
        let r = store
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(id))
            .await
            .unwrap();
        assert!(r.quest.changed);
        assert_eq!(r.quest.state.count(), (id - 2) as u32);
        assert_eq!(r.experience.granted, 0);
        assert_eq!(r.quest.became_ready, id == 5);
    }
    let done = store
        .command_quest_lifecycle(a, WORLD, &q, turn_in())
        .await
        .unwrap();
    assert!(done.turned_in_now);
    assert_eq!(done.state.revision(), 5);
    for command in [accept(), turn_in()] {
        let retry = store
            .command_quest_lifecycle(a, WORLD, &q, command)
            .await
            .unwrap();
        assert!(retry.replayed);
        assert!(!retry.changed && !retry.turned_in_now);
        assert_eq!(retry.state, done.state);
    }
    let later = store
        .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(6))
        .await
        .unwrap();
    assert!(!later.quest.changed);
    assert_eq!(later.quest.state, done.state);
    assert_eq!(counts(&admin, realm, a).await, (2, 6, 1));
    assert_eq!(
        store.ensure_quest_lifecycle(a, WORLD, &q).await.unwrap(),
        done.state
    );
    assert_eq!(
        store
            .load_quest_lifecycle(b, WORLD, &q)
            .await
            .unwrap()
            .revision(),
        0
    );
    // XP at its cap does not supply a receipt, but lifecycle's globally owned
    // victim disposition still prevents cross-character credit.
    assert!(matches!(
        store
            .award_progression_and_quest_lifecycle(b, WORLD, &p, &q, &event(2))
            .await,
        Err(Error::QuestConflict)
    ));
    assert_eq!(
        store.load_progression(b, WORLD, &p).await.unwrap().total(),
        0
    );
    assert_eq!(counts(&admin, realm, b).await, (0, 0, 0));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn exact_bindings_rollback_and_older_xp_receipts_preserve_lifecycle_integrity() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Lifebinding").await;
    let b = character(&mut store, "Lifeother").await;
    let p = xp(false);
    let q = profile();
    store.ensure_progression(a, WORLD, &p).await.unwrap();
    assert!(matches!(
        store.load_quest_lifecycle(a, WORLD, &q).await,
        Err(Error::QuestMissing)
    ));
    // Failure to find lifecycle state rolls back the preceding XP work.
    assert!(matches!(
        store
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(1))
            .await,
        Err(Error::QuestMissing)
    ));
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap().total(),
        0
    );
    store
        .award_experience(a, WORLD, &p, &event(1))
        .await
        .unwrap();
    store.ensure_quest_lifecycle(a, WORLD, &q).await.unwrap();
    assert!(matches!(
        store
            .command_quest_lifecycle(
                Binding {
                    owner: b.owner,
                    ..a
                },
                WORLD,
                &q,
                accept()
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .command_quest_lifecycle(a, [142; 32], &q, accept())
            .await,
        Err(Error::WorldMismatch)
    ));
    let changed = QuestLifecycleCatalog::from_json(&[DATA, b"\n"].concat()).unwrap();
    assert_ne!(q.hash(), changed.hash());
    assert_ne!(
        q.hash(),
        theia_persistence::QuestCatalog::from_json(DATA)
            .unwrap()
            .hash()
    );
    assert!(matches!(
        store
            .command_quest_lifecycle(a, WORLD, &changed, accept())
            .await,
        Err(Error::QuestCatalogMismatch)
    ));
    assert!(matches!(
        store
            .command_quest_lifecycle(
                a,
                WORLD,
                &q,
                Command {
                    expected_revision: 0,
                    action: Action::TurnIn
                }
            )
            .await,
        Err(Error::QuestLifecycleCommand(_))
    ));
    assert_eq!(counts(&admin, realm, a).await, (0, 0, 1));
    store
        .command_quest_lifecycle(a, WORLD, &q, accept())
        .await
        .unwrap();
    let old = store
        .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(1))
        .await
        .unwrap();
    assert!(old.experience.replayed);
    assert!(!old.quest.changed);
    assert_eq!(old.quest.state.count(), 0);
    assert!(matches!(
        store
            .command_quest_lifecycle(
                a,
                WORLD,
                &q,
                Command {
                    expected_revision: 0,
                    action: Action::TurnIn
                }
            )
            .await,
        Err(Error::QuestConflict)
    ));
    admin.batch_execute("CREATE FUNCTION theia.reject_lifecycle_update() RETURNS trigger LANGUAGE plpgsql AS $$BEGIN RAISE EXCEPTION 'owned lifecycle rollback'; END$$; CREATE TRIGGER reject_lifecycle_update BEFORE UPDATE ON theia.quest_lifecycles FOR EACH ROW EXECUTE FUNCTION theia.reject_lifecycle_update();").await.unwrap();
    assert!(
        store
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(2))
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_lifecycle_update ON theia.quest_lifecycles; DROP FUNCTION theia.reject_lifecycle_update();").await.unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &p).await.unwrap().total(),
        25
    );
    assert_eq!(
        store
            .load_quest_lifecycle(a, WORLD, &q)
            .await
            .unwrap()
            .count(),
        0
    );
    assert_eq!(counts(&admin, realm, a).await, (1, 1, 1));
    let new = store
        .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(2))
        .await
        .unwrap();
    assert_eq!(new.quest.state.count(), 1);
    assert_eq!(new.experience.state.total(), 50);
    // SQL alone cannot encode the declared goal; restore must reject a forged
    // turned-in count below this exact profile's requirement.
    admin.execute("UPDATE theia.quest_lifecycles SET revision=4,status=2,kill_count=2 WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert!(matches!(
        store.load_quest_lifecycle(a, WORLD, &q).await,
        Err(Error::QuestData)
    ));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

async fn blocked(tx: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async{loop{
        tx.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
        if let Some(row)=tx.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT world_id,profile_hash,%'",&[]).await.unwrap(){return row.get(0);}
        tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await.expect("Lifecycle command never reached its owned row lock")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn shared_worker_preserves_fifo_recovers_executing_lost_answers_and_fails_backend_closed() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Lifeworker").await;
    let p = xp(false);
    let q = profile();
    store.ensure_progression(a, WORLD, &p).await.unwrap();
    store.ensure_quest_lifecycle(a, WORLD, &q).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    assert!(matches!(
        publisher
            .command_quest_lifecycle(
                a,
                WORLD,
                &q,
                Command {
                    expected_revision: 0,
                    action: Action::TurnIn
                }
            )
            .await,
        Err(Error::QuestLifecycleCommand(_))
    ));
    assert!(!worker.status().failed);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let profile = q.clone();
    let lost = tokio::spawn(async move {
        cloned
            .command_quest_lifecycle(a, WORLD, &profile, accept())
            .await
    });
    blocked(&guard).await;
    let first_kill = event(1);
    let mut queued_kill =
        Box::pin(publisher.award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &first_kill));
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut queued_kill)
            .await
            .is_err()
    );
    lost.abort();
    let _ = lost.await;
    guard.commit().await.unwrap();
    let earned = queued_kill.await.unwrap();
    assert_eq!(
        earned.quest.state.count(),
        1,
        "FIFO must commit acceptance before the queued kill"
    );
    let retry = publisher
        .command_quest_lifecycle(a, WORLD, &q, accept())
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(retry.state.revision(), 2);
    for id in 2..=3 {
        publisher
            .award_progression_and_quest_lifecycle(a, WORLD, &p, &q, &event(id))
            .await
            .unwrap();
    }
    let ready = publisher.load_quest_lifecycle(a, WORLD, &q).await.unwrap();
    assert_eq!(ready.count(), 3);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let profile = q.clone();
    let pending = tokio::spawn(async move {
        cloned
            .command_quest_lifecycle(a, WORLD, &profile, turn_in())
            .await
    });
    let pid = blocked(&guard).await;
    let mut queued = Vec::new();
    for _ in 0..24 {
        let cloned = publisher.clone();
        let profile = q.clone();
        queued.push(tokio::spawn(async move {
            cloned.load_quest_lifecycle(a, WORLD, &profile).await
        }));
    }
    tokio::time::sleep(Duration::from_millis(30)).await;
    let mut refused = 0;
    for task in queued {
        if task.is_finished() {
            assert!(matches!(task.await.unwrap(), Err(Error::InventoryBusy)));
            refused += 1;
        } else {
            task.abort();
            let _ = task.await;
        }
    }
    assert!(
        refused >= 8,
        "The shared sixteen-job queue must refuse overflow"
    );
    assert!(
        guard
            .query_one("SELECT pg_terminate_backend($1)", &[&pid])
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    assert!(pending.await.unwrap().is_err());
    guard.rollback().await.unwrap();
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_quest_lifecycle(a, WORLD, &q).await.unwrap(),
        ready
    );
    let done = store
        .command_quest_lifecycle(a, WORLD, &q, turn_in())
        .await
        .unwrap();
    assert!(done.turned_in_now);
    assert_eq!(counts(&admin, realm, a).await, (2, 3, 3));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
