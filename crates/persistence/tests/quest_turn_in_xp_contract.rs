//! Actual owned SQL and trusted inputs only; no runtime NPC entitlement proof.
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
const QUEST: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const RECIPE: &[u8] = include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json");
const WORLD: [u8; 32] = [145; 32];
fn profiles(max: Option<u32>) -> (ProgressionCatalog, QuestLifecycleCatalog) {
    let mut data: serde_json::Value = serde_json::from_slice(XP).unwrap();
    if let Some(max) = max {
        data["level_thresholds"] = serde_json::json!([0, max]);
    }
    let xp = ProgressionCatalog::from_json(&serde_json::to_vec(&data).unwrap()).unwrap();
    let q = QuestLifecycleCatalog::from_json_with_turn_in_xp(QUEST, RECIPE, &xp).unwrap();
    (xp, q)
}
fn event(id: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(79).unwrap(),
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
fn turn() -> Command {
    Command {
        expected_revision: 4,
        action: Action::TurnIn,
    }
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/turn-in-xp-contract", name)
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
async fn ready(
    store: &mut Store,
    a: Binding,
    xp: &ProgressionCatalog,
    q: &QuestLifecycleCatalog,
    base: u64,
) {
    store.ensure_progression(a, WORLD, xp).await.unwrap();
    store.ensure_quest_lifecycle(a, WORLD, q).await.unwrap();
    store
        .command_quest_lifecycle_with_experience(a, WORLD, xp, q, accept())
        .await
        .unwrap();
    for id in base..base + 3 {
        store
            .award_progression_and_quest_lifecycle(a, WORLD, xp, q, &event(id))
            .await
            .unwrap();
    }
}
async fn receipt_count(admin: &tokio_postgres::Client, realm: RealmId, a: Binding) -> i64 {
    admin.query_one("SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&a.character.value()]).await.unwrap().get(0)
}
#[test]
fn exact_reward_mode_binding_differs_from_journal_and_all_component_byte_changes() {
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let q = QuestLifecycleCatalog::from_json_with_turn_in_xp(QUEST, RECIPE, &xp).unwrap();
    let old = QuestLifecycleCatalog::from_json(QUEST).unwrap();
    assert_ne!(q.hash(), old.hash());
    assert_eq!(q.turn_in_xp().unwrap().experience(), 25);
    assert!(old.turn_in_xp().is_none());
    let changed = [RECIPE, b" "].concat();
    assert_ne!(
        q.hash(),
        QuestLifecycleCatalog::from_json_with_turn_in_xp(QUEST, &changed, &xp)
            .unwrap()
            .hash()
    );
    let changed = [QUEST, b" "].concat();
    assert_ne!(
        q.hash(),
        QuestLifecycleCatalog::from_json_with_turn_in_xp(&changed, RECIPE, &xp)
            .unwrap()
            .hash()
    );
    let changed = ProgressionCatalog::from_json(&[XP, b" "].concat()).unwrap();
    assert_ne!(
        q.hash(),
        QuestLifecycleCatalog::from_json_with_turn_in_xp(QUEST, RECIPE, &changed)
            .unwrap()
            .hash()
    );
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn positive_clipped_and_capped_turn_ins_commit_once_with_exact_snapshot_receipts() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles(None);
    let a = character(&mut store, "Turnpositive").await;
    let b = character(&mut store, "Turnprivate").await;
    ready(&mut store, a, &xp, &q, 1).await;
    store.ensure_progression(b, WORLD, &xp).await.unwrap();
    store.ensure_quest_lifecycle(b, WORLD, &q).await.unwrap();
    assert!(matches!(
        store.command_quest_lifecycle(a, WORLD, &q, turn()).await,
        Err(Error::Invalid(_))
    ));
    assert_eq!(receipt_count(&admin, realm, a).await, 1);
    let done = store
        .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
        .await
        .unwrap();
    assert_eq!(
        (
            done.experience.granted,
            done.experience.state.total(),
            done.experience.state.revision()
        ),
        (25, 100, 4)
    );
    assert!(done.quest.turned_in_now);
    assert_eq!(done.quest.state.revision(), 5);
    let r=admin.query_one("SELECT xp_profile_hash,xp_granted,xp_revision,xp_total FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert_eq!(r.get::<_, Vec<u8>>(0), xp.hash());
    assert_eq!(
        (r.get::<_, i32>(1), r.get::<_, i64>(2), r.get::<_, i32>(3)),
        (25, 4, 100)
    );
    store
        .award_progression_and_quest_lifecycle(a, WORLD, &xp, &q, &event(4))
        .await
        .unwrap();
    for command in [accept(), turn()] {
        let r = store
            .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, command)
            .await
            .unwrap();
        assert!(r.quest.replayed && r.experience.replayed);
        assert_eq!(r.experience.granted, 0);
        assert_eq!(r.experience.state.total(), 125);
        assert_eq!(r.experience.state.revision(), 5);
        assert_eq!(r.quest.state, done.quest.state);
    }
    assert_eq!(receipt_count(&admin, realm, a).await, 2);
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(
                a,
                WORLD,
                &xp,
                &q,
                Command {
                    expected_revision: 4,
                    action: Action::Accept
                }
            )
            .await,
        Err(Error::QuestConflict)
    ));
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        0
    );
    assert_eq!(
        store
            .load_quest_lifecycle(b, WORLD, &q)
            .await
            .unwrap()
            .revision(),
        0
    );
    for (max, name, base, grant, total) in [
        (80, "Turnclipped", 100, 5, 80),
        (50, "Turncapped", 200, 0, 50),
    ] {
        let (x, q) = profiles(Some(max));
        let who = character(&mut store, name).await;
        ready(&mut store, who, &x, &q, base).await;
        let before = store.load_progression(who, WORLD, &x).await.unwrap();
        let r = store
            .command_quest_lifecycle_with_experience(who, WORLD, &x, &q, turn())
            .await
            .unwrap();
        assert_eq!(r.experience.granted, grant);
        assert_eq!(r.experience.state.total(), total);
        assert_eq!(
            r.experience.state.revision(),
            before.revision() + u64::from(grant > 0)
        );
        assert!(r.quest.turned_in_now);
        let retry = store
            .command_quest_lifecycle_with_experience(who, WORLD, &x, &q, turn())
            .await
            .unwrap();
        assert!(retry.quest.replayed);
        assert_eq!(retry.experience.granted, 0);
        assert_eq!(receipt_count(&admin, realm, who).await, 2);
    }
    assert_eq!(
        admin
            .query_one(
                "SELECT count(*) FROM theia.quest_objectives WHERE realm_id=$1",
                &[&realm.value()]
            )
            .await
            .unwrap()
            .get::<_, i64>(0),
        0
    );
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        125
    );
    assert!(matches!(
        store
            .ensure_quest_lifecycle(a, WORLD, &QuestLifecycleCatalog::from_json(QUEST).unwrap())
            .await,
        Err(Error::QuestCatalogMismatch)
    ));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn rollback_exhaustion_ownership_binding_and_corrupt_receipts_never_partially_award() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles(None);
    let a = character(&mut store, "Turnrollback").await;
    let b = character(&mut store, "Turnforeign").await;
    ready(&mut store, a, &xp, &q, 1).await;
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(
                Binding {
                    owner: b.owner,
                    character: a.character
                },
                WORLD,
                &xp,
                &q,
                turn()
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, [1; 32], &xp, &q, turn())
            .await,
        Err(Error::WorldMismatch)
    ));
    let altered = ProgressionCatalog::from_json(
        &[
            serde_json::to_vec(&serde_json::from_slice::<serde_json::Value>(XP).unwrap()).unwrap(),
            b" ".to_vec(),
        ]
        .concat(),
    )
    .unwrap();
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, WORLD, &altered, &q, turn())
            .await,
        Err(Error::ProgressionCatalogMismatch)
    ));
    admin.batch_execute("CREATE FUNCTION theia.reject_completion_xp() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned completion rollback fixture'; END $$; CREATE TRIGGER reject_completion_xp BEFORE UPDATE ON theia.progression FOR EACH ROW EXECUTE FUNCTION theia.reject_completion_xp();").await.unwrap();
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
            .await,
        Err(Error::Database(_))
    ));
    admin.batch_execute("DROP TRIGGER reject_completion_xp ON theia.progression; DROP FUNCTION theia.reject_completion_xp();").await.unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        75
    );
    assert_eq!(
        store
            .load_quest_lifecycle(a, WORLD, &q)
            .await
            .unwrap()
            .progress(q.profile())
            .unwrap()
            .phase,
        Phase::Ready
    );
    assert_eq!(receipt_count(&admin, realm, a).await, 1);
    admin
        .execute(
            "UPDATE theia.progression SET revision=$1 WHERE realm_id=$2 AND character_id=$3",
            &[
                &(theia_core::progression::MAX_REVISION as i64),
                &realm.value(),
                &a.character.value(),
            ],
        )
        .await
        .unwrap();
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
            .await,
        Err(Error::ProgressionCommand(
            theia_core::progression::Error::RevisionExhausted
        ))
    ));
    assert_eq!(receipt_count(&admin, realm, a).await, 1);
    assert_eq!(
        store
            .load_quest_lifecycle(a, WORLD, &q)
            .await
            .unwrap()
            .revision(),
        4
    );
    admin
        .execute(
            "UPDATE theia.progression SET revision=3 WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &a.character.value()],
        )
        .await
        .unwrap();
    store
        .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
        .await
        .unwrap();
    for clause in [
        "xp_granted=24",
        "xp_granted=26",
        "xp_total=101",
        "xp_revision=5",
    ] {
        admin.execute(&format!("UPDATE theia.quest_action_receipts SET {clause} WHERE realm_id=$1 AND character_id=$2 AND action=2"),&[&realm.value(),&a.character.value()]).await.unwrap();
        assert!(matches!(
            store
                .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
                .await,
            Err(Error::ProgressionData)
        ));
        admin.execute("UPDATE theia.quest_action_receipts SET xp_granted=25,xp_total=100,xp_revision=4 WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    }
    assert!(admin.execute("UPDATE theia.quest_action_receipts SET xp_total=NULL WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&a.character.value()]).await.is_err());
    assert!(admin.execute("UPDATE theia.quest_action_receipts SET xp_granted=1 WHERE realm_id=$1 AND character_id=$2 AND action=1",&[&realm.value(),&a.character.value()]).await.is_err());
    admin.execute("UPDATE theia.quest_action_receipts SET xp_profile_hash=NULL,xp_granted=NULL,xp_revision=NULL,xp_total=NULL WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
            .await,
        Err(Error::QuestData)
    ));
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        100
    );
    assert_eq!(
        store
            .load_quest_lifecycle(a, WORLD, &q)
            .await
            .unwrap()
            .revision(),
        5
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
async fn blocked(tx: &tokio_postgres::Transaction<'_>) -> i32 {
    tokio::time::timeout(Duration::from_secs(1),async{loop{tx.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();if let Some(row)=tx.query_opt("SELECT pid FROM pg_stat_activity WHERE datname=current_database() AND application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT world_id,profile_hash,%'",&[]).await.unwrap(){return row.get(0);}tokio::time::sleep(Duration::from_millis(2)).await;}}).await.expect("Owned rewarded action never reached XP lock")
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn worker_retains_executing_lost_answers_orders_later_kills_and_bounds_backend_failure() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles(None);
    let a = character(&mut store, "Turnworker").await;
    let b = character(&mut store, "Turnbackend").await;
    ready(&mut store, a, &xp, &q, 1).await;
    ready(&mut store, b, &xp, &q, 100).await;
    let (publisher, worker) = Worker::start(store);
    assert!(matches!(
        publisher
            .command_quest_lifecycle_with_experience(
                a,
                WORLD,
                &xp,
                &q,
                Command {
                    expected_revision: 3,
                    action: Action::TurnIn
                }
            )
            .await,
        Err(Error::QuestLifecycleCommand(_))
    ));
    assert!(!worker.status().failed);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&a.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let (x, c) = (xp.clone(), q.clone());
    let lost = tokio::spawn(async move {
        cloned
            .command_quest_lifecycle_with_experience(a, WORLD, &x, &c, turn())
            .await
    });
    blocked(&guard).await;
    lost.abort();
    let _ = lost.await;
    let fact = event(4);
    let mut next =
        Box::pin(publisher.award_progression_and_quest_lifecycle(a, WORLD, &xp, &q, &fact));
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut next)
            .await
            .is_err()
    );
    guard.commit().await.unwrap();
    let earned = next.await.unwrap();
    assert_eq!(earned.experience.state.total(), 125);
    assert_eq!(earned.quest.state.revision(), 5);
    let committed=admin.query_one("SELECT xp_total,xp_revision FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND action=2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert_eq!(
        (committed.get::<_, i32>(0), committed.get::<_, i64>(1)),
        (100, 4),
        "The accepted turn-in must precede the queued kill"
    );
    let retry = publisher
        .command_quest_lifecycle_with_experience(a, WORLD, &xp, &q, turn())
        .await
        .unwrap();
    assert!(retry.quest.replayed);
    assert_eq!(retry.experience.granted, 0);
    assert_eq!(worker.status().lifecycle_xp_awards, 1);
    assert_eq!(worker.status().lifecycle_xp_units, 25);
    let guard = admin.transaction().await.unwrap();
    guard.query_one("SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&realm.value(),&b.character.value()]).await.unwrap();
    let cloned = publisher.clone();
    let (x, c) = (xp.clone(), q.clone());
    let pending = tokio::spawn(async move {
        cloned
            .command_quest_lifecycle_with_experience(b, WORLD, &x, &c, turn())
            .await
    });
    let pid = blocked(&guard).await;
    let mut queued = Vec::new();
    for _ in 0..24 {
        let cloned = publisher.clone();
        let c = q.clone();
        queued.push(tokio::spawn(async move {
            cloned.load_quest_lifecycle(b, WORLD, &c).await
        }));
    }
    tokio::time::sleep(Duration::from_millis(30)).await;
    let mut refused = 0;
    let mut waiting = Vec::new();
    for task in queued {
        if task.is_finished() {
            assert!(matches!(task.await.unwrap(), Err(Error::InventoryBusy)));
            refused += 1;
        } else {
            waiting.push(task);
        }
    }
    assert!(
        refused >= 8,
        "The sixteen-job queue must refuse excess requests"
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
    for task in waiting {
        assert!(task.await.unwrap().is_err());
    }
    assert!(worker.shutdown().await.is_err());
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        75
    );
    assert_eq!(
        store
            .load_quest_lifecycle(b, WORLD, &q)
            .await
            .unwrap()
            .revision(),
        4
    );
    assert_eq!(receipt_count(&admin, realm, b).await, 1);
    let r = store
        .command_quest_lifecycle_with_experience(b, WORLD, &xp, &q, turn())
        .await
        .unwrap();
    assert_eq!(r.experience.granted, 25);
    assert_eq!(r.experience.state.total(), 100);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn migration_nine_journal_rows_and_receipts_upgrade_without_completion_rewards() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, rewarded) = profiles(None);
    let journal = QuestLifecycleCatalog::from_json(QUEST).unwrap();
    let a = character(&mut store, "Turnupgrade").await;
    store.ensure_progression(a, WORLD, &xp).await.unwrap();
    store
        .ensure_quest_lifecycle(a, WORLD, &journal)
        .await
        .unwrap();
    store
        .command_quest_lifecycle(a, WORLD, &journal, accept())
        .await
        .unwrap();
    for id in 1..=3 {
        store
            .award_progression_and_quest_lifecycle(a, WORLD, &xp, &journal, &event(id))
            .await
            .unwrap();
    }
    let done = store
        .command_quest_lifecycle(a, WORLD, &journal, turn())
        .await
        .unwrap();
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        75
    );
    store.close().await.unwrap();
    admin.batch_execute("DROP TABLE theia.quest_journal_actions; DROP TABLE theia.quest_journal_events; DROP TABLE theia.quest_journal_entries; DROP TABLE theia.quest_journal_heads; ALTER TABLE theia.quest_action_receipts DROP CONSTRAINT quest_action_xp_metadata, DROP COLUMN xp_profile_hash, DROP COLUMN xp_granted, DROP COLUMN xp_revision, DROP COLUMN xp_total; DELETE FROM theia.schema_migrations WHERE version>=10;").await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let count: i64 = admin
        .query_one("SELECT count(*) FROM theia.schema_migrations", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 11);
    assert_eq!(
        store
            .load_quest_lifecycle(a, WORLD, &journal)
            .await
            .unwrap(),
        done.state
    );
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        75
    );
    for command in [accept(), turn()] {
        let r = store
            .command_quest_lifecycle(a, WORLD, &journal, command)
            .await
            .unwrap();
        assert!(r.replayed);
        assert_eq!(r.state, done.state);
    }
    let old:i64=admin.query_one("SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND xp_profile_hash IS NULL AND xp_granted IS NULL AND xp_revision IS NULL AND xp_total IS NULL",&[&realm.value(),&a.character.value()]).await.unwrap().get(0);
    assert_eq!(old, 2);
    assert!(matches!(
        store.ensure_quest_lifecycle(a, WORLD, &rewarded).await,
        Err(Error::QuestCatalogMismatch)
    ));
    assert!(matches!(
        store
            .command_quest_lifecycle_with_experience(a, WORLD, &xp, &rewarded, turn())
            .await,
        Err(Error::QuestCatalogMismatch)
    ));
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        75
    );
    let fresh = character(&mut store, "Turnupgradenew").await;
    ready(&mut store, fresh, &xp, &rewarded, 100).await;
    let r = store
        .command_quest_lifecycle_with_experience(fresh, WORLD, &xp, &rewarded, turn())
        .await
        .unwrap();
    assert_eq!(r.experience.granted, 25);
    assert_eq!(r.experience.state.total(), 100);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
