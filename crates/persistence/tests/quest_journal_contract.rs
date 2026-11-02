//! Owned SQL with synthetic already-authorized NPC/events. No live World,
//! shared-party entitlement, transport or native quest adoption claim.
mod support;
use serde_json::{Value, json};
use std::{num::NonZeroU128, time::Duration};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    quests::{catalog::EventKey, journal::Status},
};
use theia_persistence::{
    Binding, CharacterName, Error, ProgressionCatalog, QuestJournalAction as Action,
    QuestJournalCatalog, QuestJournalCommand, RealmId, Store,
};
const DATA: &[u8] = include_bytes!("../../../content/quests/development-journal.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const WORLD: [u8; 32] = [173; 32];
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn first() -> ContentId {
    id("theia.dev.quest-introduction")
}
fn second() -> ContentId {
    id("theia.dev.quest-follow-up")
}
fn third() -> ContentId {
    id("theia.dev.quest-follow-up-b")
}
fn nz(n: u128) -> NonZeroU128 {
    NonZeroU128::new(n).unwrap()
}
fn event(name: &str) -> EventKey {
    EventKey::NpcInteraction {
        npc: id("theia.dev.guide"),
        interaction: id(&format!("theia.dev.interaction.{name}")),
    }
}
fn command(n: u128, q: ContentId, action: Action, revision: u64) -> QuestJournalCommand {
    QuestJournalCommand {
        request_id: nz(n),
        quest: q,
        npc: id("theia.dev.guide"),
        expected_revision: revision,
        action,
    }
}
fn quest_bytes() -> Vec<u8> {
    let mut v: Value = serde_json::from_slice(DATA).unwrap();
    // This kill has no XP recipe: journal credit must be independent.
    v["quests"][1]["stages"][0]["objectives"][0]["creature"] = json!("theia.dev.no-xp-creature");
    let copy = serde_json::to_string(&v["quests"][1])
        .unwrap()
        .replace("theia.dev.quest-follow-up", "theia.dev.quest-follow-up-b");
    v["quests"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::from_str(&copy).unwrap());
    serde_json::to_vec(&v).unwrap()
}
fn profiles() -> (ProgressionCatalog, QuestJournalCatalog) {
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let q = QuestJournalCatalog::from_json(&quest_bytes(), &xp).unwrap();
    (xp, q)
}
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/quest-journal-contract", name)
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
async fn initialize(
    store: &mut Store,
    b: Binding,
    xp: &ProgressionCatalog,
    q: &QuestJournalCatalog,
) {
    store.ensure_progression(b, WORLD, xp).await.unwrap();
    store.ensure_quest_journal(b, WORLD, q).await.unwrap();
}
async fn ready(store: &mut Store, b: Binding, xp: &ProgressionCatalog, q: &QuestJournalCatalog) {
    store
        .command_quest_journal(b, WORLD, q, xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    for (n, name) in [(1, "road"), (2, "supplies"), (3, "report")] {
        store
            .record_quest_event(b, WORLD, q, nz(n), &event(name))
            .await
            .unwrap();
    }
    assert_eq!(
        store
            .load_quest_journal(b, WORLD, q)
            .await
            .unwrap()
            .journal
            .progress(&first())
            .unwrap()
            .status(),
        Status::Ready
    );
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn independent_event_receipts_cover_multiple_quests_and_recipients_without_xp_kill_recipes() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles();
    let a = character(&mut store, "Journalone").await;
    let b = character(&mut store, "Journaltwo").await;
    let kill = EventKey::Kill(id("theia.dev.no-xp-creature"));
    for binding in [a, b] {
        initialize(&mut store, binding, &xp, &q).await;
        store
            .record_quest_event(binding, WORLD, &q, nz(99), &kill)
            .await
            .unwrap();
        assert!(matches!(
            store
                .command_quest_journal(
                    binding,
                    WORLD,
                    &q,
                    &xp,
                    &command(20, second(), Action::Accept, 0)
                )
                .await,
            Err(Error::QuestJournalCommand(_))
        ));
        ready(&mut store, binding, &xp, &q).await;
        let r = store
            .command_quest_journal(
                binding,
                WORLD,
                &q,
                &xp,
                &command(2, first(), Action::TurnIn, 4),
            )
            .await
            .unwrap();
        assert_eq!(r.experience_granted, 25);
        for (n, quest) in [(3, second()), (4, third())] {
            store
                .command_quest_journal(
                    binding,
                    WORLD,
                    &q,
                    &xp,
                    &command(n, quest, Action::Accept, 0),
                )
                .await
                .unwrap();
        }
        let old = store
            .record_quest_event(binding, WORLD, &q, nz(99), &kill)
            .await
            .unwrap();
        assert!(old.replayed);
        assert!(old.changed.is_empty());
        let r = store
            .record_quest_event(binding, WORLD, &q, nz(100), &kill)
            .await
            .unwrap();
        assert_eq!(r.changed.len(), 2);
        let repeat = store
            .record_quest_event(binding, WORLD, &q, nz(100), &kill)
            .await
            .unwrap();
        assert!(repeat.replayed);
        assert_eq!(repeat.journal, r.journal);
        assert!(matches!(
            store
                .record_quest_event(binding, WORLD, &q, nz(100), &event("training"))
                .await,
            Err(Error::QuestJournalConflict)
        ));
        assert_eq!(
            store
                .load_progression(binding, WORLD, &xp)
                .await
                .unwrap()
                .total(),
            25
        );
    }
    for n in [101, 102] {
        store
            .record_quest_event(a, WORLD, &q, nz(n), &kill)
            .await
            .unwrap();
    }
    store
        .record_quest_event(a, WORLD, &q, nz(103), &event("training"))
        .await
        .unwrap();
    for (n, quest) in [(5, second()), (6, third())] {
        let revision = store
            .load_quest_journal(a, WORLD, &q)
            .await
            .unwrap()
            .journal
            .progress(&quest)
            .unwrap()
            .revision();
        let request = command(n, quest, Action::TurnIn, revision);
        let r = store
            .command_quest_journal(a, WORLD, &q, &xp, &request)
            .await
            .unwrap();
        assert_eq!(r.experience_granted, 25);
        assert_eq!(
            store
                .command_quest_journal(a, WORLD, &q, &xp, &request)
                .await
                .unwrap()
                .experience_granted,
            0
        );
    }
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        75
    );
    let replay = store
        .command_quest_journal(a, WORLD, &q, &xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(
        replay.journal.progress(&third()).unwrap().status(),
        Status::Completed
    );
    assert!(matches!(
        store
            .command_quest_journal(a, WORLD, &q, &xp, &command(1, second(), Action::Accept, 0))
            .await,
        Err(Error::QuestJournalConflict)
    ));
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_journal_events WHERE realm_id=$1 AND event_id=$2",
            &[&realm.value(), &100u128.to_le_bytes().as_slice()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 2);
    let saved = store.load_quest_journal(a, WORLD, &q).await.unwrap();
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .load_quest_journal(a, WORLD, &q)
            .await
            .unwrap()
            .journal,
        saved.journal
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn completion_rolls_back_state_xp_and_receipt_and_waits_for_durable_lock_release() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles();
    let a = character(&mut store, "Journalatomic").await;
    initialize(&mut store, a, &xp, &q).await;
    ready(&mut store, a, &xp, &q).await;
    admin.batch_execute("CREATE FUNCTION theia.reject_journal_turnin() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.action=2 THEN RAISE EXCEPTION 'owned quest journal failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_journal_turnin BEFORE INSERT ON theia.quest_journal_actions FOR EACH ROW EXECUTE FUNCTION theia.reject_journal_turnin();").await.unwrap();
    let request = command(2, first(), Action::TurnIn, 4);
    assert!(
        store
            .command_quest_journal(a, WORLD, &q, &xp, &request)
            .await
            .is_err()
    );
    assert_eq!(
        store.load_progression(a, WORLD, &xp).await.unwrap().total(),
        0
    );
    assert_eq!(
        store
            .load_quest_journal(a, WORLD, &q)
            .await
            .unwrap()
            .journal
            .progress(&first())
            .unwrap()
            .status(),
        Status::Ready
    );
    admin.batch_execute("DROP TRIGGER reject_journal_turnin ON theia.quest_journal_actions; DROP FUNCTION theia.reject_journal_turnin();").await.unwrap();
    let lock = admin.transaction().await.unwrap();
    lock.query_one(
        "SELECT revision FROM theia.progression WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
        &[&realm.value(), &a.character.value()],
    )
    .await
    .unwrap();
    let task_xp = xp.clone();
    let task_q = q.clone();
    let task_request = request.clone();
    let mut task = tokio::spawn(async move {
        let result = store
            .command_quest_journal(a, WORLD, &task_q, &task_xp, &task_request)
            .await;
        (store, result)
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(60), &mut task)
            .await
            .is_err()
    );
    lock.commit().await.unwrap();
    let (mut store, result) = task.await.unwrap();
    assert_eq!(result.unwrap().experience_granted, 25);
    assert_eq!(
        store
            .command_quest_journal(a, WORLD, &q, &xp, &request)
            .await
            .unwrap()
            .experience_granted,
        0
    );
    let row=admin.query_one("SELECT count(*),sum(xp_granted) FROM theia.quest_journal_actions WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&a.character.value()]).await.unwrap();
    assert_eq!(row.get::<_, i64>(0), 2);
    assert_eq!(row.get::<_, Option<i64>>(1), Some(25));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn exact_profile_owner_world_and_per_quest_bytes_cannot_be_silently_rebound() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q) = profiles();
    let a = character(&mut store, "Journalguard").await;
    initialize(&mut store, a, &xp, &q).await;
    ready(&mut store, a, &xp, &q).await;
    assert!(matches!(
        store.load_quest_journal(a, [174; 32], &q).await,
        Err(Error::WorldMismatch)
    ));
    let mut wrong = a;
    wrong.owner = theia_persistence::OwnerId::random();
    assert!(matches!(
        store.load_quest_journal(wrong, WORLD, &q).await,
        Err(Error::Ownership)
    ));
    let mut bytes = quest_bytes();
    bytes.push(b' ');
    let padded = QuestJournalCatalog::from_json(&bytes, &xp).unwrap();
    assert!(matches!(
        store.load_quest_journal(a, WORLD, &padded).await,
        Err(Error::QuestJournalCatalogMismatch)
    ));
    let state:Vec<u8>=admin.query_one("SELECT state FROM theia.quest_journal_entries WHERE realm_id=$1 AND character_id=$2 AND quest_id=$3",&[&realm.value(),&a.character.value(),&first().as_str()]).await.unwrap().get(0);
    admin
        .execute(
            "UPDATE theia.quest_journal_entries SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&b"null".as_slice(), &realm.value(), &a.character.value()],
        )
        .await
        .unwrap();
    assert!(matches!(
        store.load_quest_journal(a, WORLD, &q).await,
        Err(Error::QuestJournalData)
    ));
    admin
        .execute(
            "UPDATE theia.quest_journal_entries SET state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&state, &realm.value(), &a.character.value()],
        )
        .await
        .unwrap();
    store.load_quest_journal(a, WORLD, &q).await.unwrap();
    // A logically valid earlier quest row must still fail the journal-head
    // digest; checking only row-local counts/revisions would permit rollback.
    let earlier = theia_core::quests::journal::Journal::default()
        .propose_accept(q.catalog(), &first(), &id("theia.dev.guide"))
        .unwrap()
        .journal
        .quest_snapshots(q.catalog())
        .unwrap()
        .remove(0);
    admin.execute("UPDATE theia.quest_journal_entries SET state=$1,revision=1,status=0 WHERE realm_id=$2 AND character_id=$3",&[&earlier.bytes,&realm.value(),&a.character.value()]).await.unwrap();
    assert!(matches!(
        store.load_quest_journal(a, WORLD, &q).await,
        Err(Error::QuestJournalData)
    ));
    admin.execute("UPDATE theia.quest_journal_entries SET state=$1,revision=4,status=1 WHERE realm_id=$2 AND character_id=$3",&[&state,&realm.value(),&a.character.value()]).await.unwrap();
    store.load_quest_journal(a, WORLD, &q).await.unwrap();
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn zero_reward_and_capped_xp_still_complete_with_idempotent_receipts() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let mut xp_raw: Value = serde_json::from_slice(XP).unwrap();
    xp_raw["level_thresholds"] = json!([0, 25]);
    let xp = ProgressionCatalog::from_json(&serde_json::to_vec(&xp_raw).unwrap()).unwrap();
    for (name, amount) in [("Journalzero", 0), ("Journalcap", 25)] {
        let mut raw: Value = serde_json::from_slice(DATA).unwrap();
        raw["quests"][0]["experience"] = json!(amount);
        let q = QuestJournalCatalog::from_json(&serde_json::to_vec(&raw).unwrap(), &xp).unwrap();
        let a = character(&mut store, name).await;
        initialize(&mut store, a, &xp, &q).await;
        if amount > 0 {
            store
                .award_experience(
                    a,
                    WORLD,
                    &xp,
                    &theia_core::progression::KillEvent {
                        id: theia_core::progression::KillId::from_victim(
                            theia_core::identity::EntityKey {
                                epoch: theia_core::identity::WorldEpoch::new(1).unwrap(),
                                entity: theia_core::identity::EntityId::new(987).unwrap(),
                            },
                        ),
                        creature: id("theia.dev.training-creature"),
                    },
                )
                .await
                .unwrap();
        }
        ready(&mut store, a, &xp, &q).await;
        let request = command(2, first(), Action::TurnIn, 4);
        let result = store
            .command_quest_journal(a, WORLD, &q, &xp, &request)
            .await
            .unwrap();
        assert_eq!(result.experience_granted, 0);
        assert_eq!(
            result.journal.progress(&first()).unwrap().status(),
            Status::Completed
        );
        assert!(
            store
                .command_quest_journal(a, WORLD, &q, &xp, &request)
                .await
                .unwrap()
                .replayed
        );
    }
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn journal_worker_retains_cancelled_jobs_bounds_queue_and_drains_before_shutdown() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let b = character(&mut store, "Queuejournal").await;
    let (xp, q) = profiles();
    initialize(&mut store, b, &xp, &q).await;
    let (publisher, worker) = theia_persistence::Worker::start(store);
    assert_eq!(
        publisher
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .revision,
        0
    );

    let tx = admin.transaction().await.unwrap();
    tx.query_one("SELECT revision FROM theia.quest_journal_heads WHERE realm_id=$1 AND character_id=$2 FOR UPDATE", &[&realm.value(), &b.character.value()]).await.unwrap();
    let mut accepted = publisher
        .enqueue_quest_journal_command(b, WORLD, &q, &xp, command(200, first(), Action::Accept, 0))
        .unwrap();
    // Establish that the job has left the queue and is waiting for our lock.
    let (observer, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls)
        .await
        .unwrap();
    let observer_driver = tokio::spawn(connection);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let waiting: bool = observer.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE wait_event_type='Lock' AND query LIKE '%quest_journal_heads%' AND pid<>pg_backend_pid())", &[]).await.unwrap().get(0);
            if waiting { break; }
            tokio::task::yield_now().await;
        }
    }).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut accepted)
            .await
            .is_err()
    );
    // Dropped answers cannot erase accepted mutations, including queued jobs.
    drop(accepted);
    for n in 1..=16 {
        drop(
            publisher
                .enqueue_quest_event(b, WORLD, &q, nz(200 + n), event("road"))
                .unwrap(),
        );
    }
    assert!(matches!(
        publisher.enqueue_quest_event(b, WORLD, &q, nz(999), event("road")),
        Err(Error::InventoryBusy)
    ));
    let mut ending = tokio::spawn(worker.shutdown());
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut ending)
            .await
            .is_err()
    );
    tx.commit().await.unwrap();
    let status = ending.await.unwrap().unwrap();
    assert_eq!(status.journal_commands, 1);
    assert_eq!(status.journal_events, 16);
    assert!(!status.failed);
    assert!(
        publisher
            .enqueue_quest_event(b, WORLD, &q, nz(999), event("road"))
            .is_err()
    );

    let mut reopened = Store::connect_loopback(&url, realm).await.unwrap();
    let saved = reopened.load_quest_journal(b, WORLD, &q).await.unwrap();
    assert_eq!(saved.revision, 17);
    assert_eq!(saved.journal.progress(&first()).unwrap().revision(), 2);
    let (publisher, worker) = theia_persistence::Worker::start(reopened);
    // Refusals and receipt conflicts must not kill the shared SQL worker.
    let refused = publisher
        .enqueue_quest_journal_command(b, WORLD, &q, &xp, command(201, first(), Action::TurnIn, 1))
        .unwrap()
        .await
        .unwrap();
    assert!(matches!(refused, Err(Error::QuestJournalCommand(_))));
    let conflict = publisher
        .enqueue_quest_event(b, WORLD, &q, nz(201), event("supplies"))
        .unwrap()
        .await
        .unwrap();
    assert!(matches!(conflict, Err(Error::QuestJournalConflict)));
    for (n, action) in [(301, "supplies"), (302, "report")] {
        drop(
            publisher
                .enqueue_quest_event(b, WORLD, &q, nz(n), event(action))
                .unwrap(),
        );
    }
    drop(
        publisher
            .enqueue_quest_journal_command(
                b,
                WORLD,
                &q,
                &xp,
                command(202, first(), Action::TurnIn, 4),
            )
            .unwrap(),
    );
    let status = worker.shutdown().await.unwrap();
    assert_eq!(status.journal_rejected, 2);
    assert_eq!(status.journal_experience_units, 25);
    let mut reopened = Store::connect_loopback(&url, realm).await.unwrap();
    let saved = reopened.load_quest_journal(b, WORLD, &q).await.unwrap();
    assert_eq!(
        saved.journal.progress(&first()).unwrap().status(),
        Status::Completed
    );
    assert_eq!(
        reopened
            .load_progression(b, WORLD, &xp)
            .await
            .unwrap()
            .total(),
        25
    );
    reopened.close().await.unwrap();
    drop(observer);
    observer_driver.await.unwrap().unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn journal_admission_profile_preflight_is_read_only_and_checks_exact_binding() {
    use theia_persistence::{GameplayProfiles, ProfileCompatibility as Compatibility};
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let b = character(&mut store, "Preflightjournal").await;
    let (xp, q) = profiles();
    let profiles = GameplayProfiles::new(None, Some(xp.hash()), None, None)
        .unwrap()
        .with_quest_journal(q.hash())
        .unwrap();
    assert!(
        GameplayProfiles::default()
            .with_quest_journal(q.hash())
            .is_err()
    );
    assert!(
        GameplayProfiles::new(None, Some(xp.hash()), Some([1; 32]), None)
            .unwrap()
            .with_quest_journal(q.hash())
            .is_err()
    );
    assert!(profiles.with_quest_journal(q.hash()).is_err());
    assert_eq!(
        store
            .check_gameplay_profiles(b, WORLD, profiles)
            .await
            .unwrap(),
        Compatibility::Compatible
    );
    // Preflight must not initialize any saved journal or progression rows.
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_journal_heads WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 0);
    initialize(&mut store, b, &xp, &q).await;
    let (publisher, worker) = theia_persistence::Worker::start(store);
    assert_eq!(
        publisher
            .check_gameplay_profiles(b, WORLD, profiles)
            .await
            .unwrap(),
        Compatibility::Compatible
    );
    let changed = GameplayProfiles::new(None, Some(xp.hash()), None, None)
        .unwrap()
        .with_quest_journal([19; 32])
        .unwrap();
    assert_eq!(
        publisher
            .check_gameplay_profiles(b, WORLD, changed)
            .await
            .unwrap(),
        Compatibility::Incompatible
    );
    assert_eq!(
        publisher
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .revision,
        0
    );
    let status = worker.shutdown().await.unwrap();
    assert_eq!(status.incompatible_gameplay_profiles, 1);
    assert!(!status.failed);
    drop(admin);
    driver.await.unwrap().unwrap();
}
