use super::*;
use std::collections::BTreeSet;
fn area_profiles() -> (ProgressionCatalog, QuestJournalCatalog, Vec<u8>, Vec<u8>) {
    let mut q: Value = serde_json::from_slice(DATA).unwrap();
    q["schema"] = json!(2);
    q["quests"].as_array_mut().unwrap().truncate(1);
    for (index, stage) in q["quests"][0]["stages"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        stage["objectives"] = json!([{"kind":"reach_area","id":format!("theia.dev.goal.{index}"),
            "label":"Reach","area":"theia.dev.area.test"}]);
    }
    let areas = json!({"schema":1,"id":"theia.dev.areas","revision":1,"areas":[{
        "id":"theia.dev.area.test","name":"Test","min":[7990.,-64.,7990.],"max":[8010.,64.,8010.]}]});
    let q = serde_json::to_vec(&q).unwrap();
    let areas = serde_json::to_vec(&areas).unwrap();
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let catalog = QuestJournalCatalog::from_json_with_areas(&q, &areas, &xp).unwrap();
    (xp, catalog, q, areas)
}
fn observe(
    q: &QuestJournalCatalog,
    j: &theia_core::quests::journal::Journal,
) -> theia_core::quests::journal::ReachedAreas {
    j.prepare_reached_areas(q.catalog(), &BTreeSet::from([id("theia.dev.area.test")]))
        .unwrap()
}
#[test]
fn area_profile_hash_matches_the_network_domain_and_exact_bytes() {
    let (xp, q, bytes, areas) = area_profiles();
    let old = QuestJournalCatalog::from_json(&bytes, &xp).unwrap();
    let mut expected = theia_core::quests::areas::PROFILE_DOMAIN.to_vec();
    expected.extend(old.hash());
    expected.extend(&areas);
    assert_eq!(
        q.hash(),
        ring::digest::digest(&ring::digest::SHA256, &expected).as_ref()
    );
    let mut other = areas;
    other.push(b' ');
    assert_ne!(
        q.hash(),
        QuestJournalCatalog::from_json_with_areas(&bytes, &other, &xp)
            .unwrap()
            .hash()
    );
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn area_receipts_seal_stages_rollback_atomically_and_reopen_without_duplicate_xp() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q, _, _) = area_profiles();
    let b = character(&mut store, "Reachone").await;
    initialize(&mut store, b, &xp, &q).await;
    let j = store
        .command_quest_journal(b, WORLD, &q, &xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    let first_batch = observe(&q, &j.journal);
    assert_eq!(first_batch.len(), 1);
    admin.batch_execute("CREATE FUNCTION theia.reject_area_receipt() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned area receipt failure'; END $$; CREATE TRIGGER reject_area_receipt BEFORE INSERT ON theia.quest_journal_events FOR EACH ROW EXECUTE FUNCTION theia.reject_area_receipt();").await.unwrap();
    assert!(
        store
            .record_reached_areas(b, WORLD, &q, nz(100), &first_batch)
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_area_receipt ON theia.quest_journal_events; DROP FUNCTION theia.reject_area_receipt();").await.unwrap();
    let untouched = store.load_quest_journal(b, WORLD, &q).await.unwrap();
    assert_eq!(untouched.journal, j.journal);
    assert_eq!(untouched.revision, j.revision);
    let next = store
        .record_reached_areas(b, WORLD, &q, nz(100), &first_batch)
        .await
        .unwrap();
    assert_eq!(
        next.journal
            .progress(&first())
            .unwrap()
            .count(&id("theia.dev.goal.0")),
        Some(1)
    );
    assert_eq!(
        next.journal
            .progress(&first())
            .unwrap()
            .count(&id("theia.dev.goal.1")),
        Some(0)
    );
    let replay = store
        .record_reached_areas(b, WORLD, &q, nz(100), &first_batch)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.revision, next.revision);
    let fresh = observe(&q, &next.journal);
    assert!(matches!(
        store
            .record_reached_areas(b, WORLD, &q, nz(100), &fresh)
            .await,
        Err(Error::QuestJournalConflict)
    ));
    // Even a new receipt cannot apply the old sample to a later identical area.
    let stale = store
        .record_reached_areas(b, WORLD, &q, nz(101), &first_batch)
        .await
        .unwrap();
    assert!(stale.changed.is_empty());
    assert_eq!(stale.journal, next.journal);
    let ready = store
        .record_reached_areas(b, WORLD, &q, nz(102), &fresh)
        .await
        .unwrap();
    assert_eq!(
        ready.journal.progress(&first()).unwrap().status(),
        Status::Ready
    );
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        0
    );
    drop(store);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .journal,
        ready.journal
    );
    assert!(
        store
            .record_reached_areas(b, WORLD, &q, nz(102), &fresh)
            .await
            .unwrap()
            .replayed
    );
    let revision = ready.journal.progress(&first()).unwrap().revision();
    let turn = command(2, first(), Action::TurnIn, revision);
    let done = store
        .command_quest_journal(b, WORLD, &q, &xp, &turn)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    assert_eq!(
        store
            .command_quest_journal(b, WORLD, &q, &xp, &turn)
            .await
            .unwrap()
            .experience_granted,
        0
    );
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        25
    );
    drop(store);
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn accepted_area_job_survives_dropped_response_and_shutdown() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, q, _, _) = area_profiles();
    let b = character(&mut store, "Reachtwo").await;
    initialize(&mut store, b, &xp, &q).await;
    let j = store
        .command_quest_journal(b, WORLD, &q, &xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    let batch = observe(&q, &j.journal);
    let guard = admin.transaction().await.unwrap();
    guard
        .batch_execute("LOCK TABLE theia.quest_journal_heads IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();
    let (publisher, worker) = theia_persistence::Worker::start(store);
    let mut answer = publisher
        .enqueue_reached_areas(b, WORLD, &q, nz(200), batch.clone())
        .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut answer)
            .await
            .is_err()
    );
    drop(answer);
    let mut ending = tokio::spawn(worker.shutdown());
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut ending)
            .await
            .is_err()
    );
    guard.commit().await.unwrap();
    let status = ending.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_events, 1);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let replay = store
        .record_reached_areas(b, WORLD, &q, nz(200), &batch)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(
        replay
            .journal
            .progress(&first())
            .unwrap()
            .count(&id("theia.dev.goal.0")),
        Some(1)
    );
    assert_eq!(
        replay
            .journal
            .progress(&first())
            .unwrap()
            .count(&id("theia.dev.goal.1")),
        Some(0)
    );
    drop(store);
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn delayed_area_batch_cannot_credit_new_acceptance_and_fresh_batch_updates_two_quests() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, _, bytes, areas) = area_profiles();
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    let copy = serde_json::to_string(&value["quests"][0])
        .unwrap()
        .replace("theia.dev.quest-introduction", "theia.dev.quest-other")
        .replace("theia.dev.goal.", "theia.dev.other-goal.");
    value["quests"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::from_str(&copy).unwrap());
    let bytes = serde_json::to_vec(&value).unwrap();
    let q = QuestJournalCatalog::from_json_with_areas(&bytes, &areas, &xp).unwrap();
    let b = character(&mut store, "Reachthree").await;
    initialize(&mut store, b, &xp, &q).await;
    let first_state = store
        .command_quest_journal(b, WORLD, &q, &xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    let old = observe(&q, &first_state.journal);
    store
        .command_quest_journal(
            b,
            WORLD,
            &q,
            &xp,
            &command(2, id("theia.dev.quest-other"), Action::Accept, 0),
        )
        .await
        .unwrap();
    assert!(matches!(
        store
            .record_reached_areas(b, [174; 32], &q, nz(300), &old)
            .await,
        Err(Error::WorldMismatch)
    ));
    let unbound = QuestJournalCatalog::from_json(&bytes, &xp).unwrap();
    assert!(
        store
            .record_reached_areas(b, WORLD, &unbound, nz(300), &old)
            .await
            .is_err()
    );
    assert!(
        store
            .record_quest_event(
                b,
                WORLD,
                &q,
                nz(300),
                &EventKey::ReachArea(id("theia.dev.area.test"))
            )
            .await
            .is_err()
    );
    let advanced = store
        .record_reached_areas(b, WORLD, &q, nz(300), &old)
        .await
        .unwrap();
    assert_eq!(advanced.changed, vec![first()]);
    assert_eq!(
        advanced
            .journal
            .progress(&id("theia.dev.quest-other"))
            .unwrap()
            .count(&id("theia.dev.other-goal.0")),
        Some(0)
    );
    let fresh = observe(&q, &advanced.journal);
    assert_eq!(fresh.len(), 2);
    let both = store
        .record_reached_areas(b, WORLD, &q, nz(301), &fresh)
        .await
        .unwrap();
    assert_eq!(both.changed.len(), 2);
    assert_eq!(
        both.journal.progress(&first()).unwrap().status(),
        Status::Ready
    );
    let other = both.journal.progress(&id("theia.dev.quest-other")).unwrap();
    assert_eq!(other.count(&id("theia.dev.other-goal.0")), Some(1));
    assert_eq!(other.count(&id("theia.dev.other-goal.1")), Some(0));
    let replay = store
        .record_reached_areas(b, WORLD, &q, nz(301), &fresh)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.journal, both.journal);
    assert_eq!(replay.revision, both.revision);
    drop(store);
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn overlapping_multi_objective_batch_keeps_journal_head_above_every_quest_revision() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, _, bytes, areas) = area_profiles();
    let mut q: Value = serde_json::from_slice(&bytes).unwrap();
    let mut areas: Value = serde_json::from_slice(&areas).unwrap();
    for n in [1, 2] {
        let mut area = areas["areas"][0].clone();
        area["id"] = json!(format!("theia.dev.area.overlap{n}"));
        areas["areas"].as_array_mut().unwrap().push(area);
        q["quests"][0]["stages"][0]["objectives"]
            .as_array_mut()
            .unwrap()
            .push(json!({
            "kind":"reach_area","id":format!("theia.dev.overlap-goal{n}"),
            "label":"Reach","area":format!("theia.dev.area.overlap{n}")}));
    }
    let q = QuestJournalCatalog::from_json_with_areas(
        &serde_json::to_vec(&q).unwrap(),
        &serde_json::to_vec(&areas).unwrap(),
        &xp,
    )
    .unwrap();
    let b = character(&mut store, "Reachfour").await;
    initialize(&mut store, b, &xp, &q).await;
    let accepted = store
        .command_quest_journal(b, WORLD, &q, &xp, &command(1, first(), Action::Accept, 0))
        .await
        .unwrap();
    let inside = BTreeSet::from([
        id("theia.dev.area.test"),
        id("theia.dev.area.overlap1"),
        id("theia.dev.area.overlap2"),
    ]);
    let batch = accepted
        .journal
        .prepare_reached_areas(q.catalog(), &inside)
        .unwrap();
    assert_eq!(batch.len(), 3);
    let applied = store
        .record_reached_areas(b, WORLD, &q, nz(400), &batch)
        .await
        .unwrap();
    assert_eq!(applied.journal.progress(&first()).unwrap().revision(), 4);
    let loaded = store.load_quest_journal(b, WORLD, &q).await.unwrap();
    assert_eq!(loaded.journal, applied.journal);
    assert_eq!(loaded.revision, 4);
    assert!(
        store
            .record_reached_areas(b, WORLD, &q, nz(400), &batch)
            .await
            .unwrap()
            .replayed
    );
    let next = observe(&q, &loaded.journal);
    let ready = store
        .record_reached_areas(b, WORLD, &q, nz(401), &next)
        .await
        .unwrap();
    assert_eq!(ready.revision, 5);
    assert_eq!(
        ready.journal.progress(&first()).unwrap().status(),
        Status::Ready
    );
    drop(store);
    drop(admin);
    driver.await.unwrap().unwrap();
}
