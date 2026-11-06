#[path = "support/documents.rs"]
mod documents;
mod support;
use documents::*;
use std::time::Duration;
use theia_core::{
    inventory::{Command, Operation},
    quests::{
        catalog::EventKey,
        journal::{Error as Rule, Status},
    },
};
use theia_persistence::*;

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn original_open_receipts_preserve_stage_eligibility_history_and_ownership_on_retry() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false);
    let who = initialize(&mut store, "Documentread", 2, &p).await;
    let original = open(&p, 100, 0, 0);
    let preaccept = store
        .open_quest_document(who, WORLD, &p.journal, &original)
        .await
        .unwrap();
    assert!(preaccept.journal.quests().next().is_none());
    assert!(preaccept.changed.is_empty());
    assert_eq!(preaccept.revision, 1);
    accept(&mut store, who, &p).await;
    let replay = store
        .open_quest_document(who, WORLD, &p.journal, &original)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(
        replay
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-first")),
        Some(0)
    );
    let first = open(&p, 101, 0, 0);
    let result = store
        .open_quest_document(who, WORLD, &p.journal, &first)
        .await
        .unwrap();
    assert_eq!(
        result.journal.progress(&id("quest")).unwrap().stage(),
        &id("second")
    );
    assert_eq!(
        result
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-final")),
        Some(0)
    );
    let replay = store
        .open_quest_document(who, WORLD, &p.journal, &first)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.journal, result.journal);
    let final_read = open(&p, 102, 0, 0);
    let ready = store
        .open_quest_document(who, WORLD, &p.journal, &final_read)
        .await
        .unwrap();
    assert_eq!(
        ready.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    assert_eq!(ready.inventory.as_ref().unwrap().revision(), 0);
    assert_eq!(
        store
            .load_progression(who, WORLD, &p.xp)
            .await
            .unwrap()
            .total(),
        0
    );
    let removed = store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 2 },
            }
            .into(),
            &p.journal,
        )
        .await
        .unwrap();
    assert_eq!(removed.bag.revision(), 1);
    assert!(removed.bag.slots()[0].is_none());
    let retry = store
        .open_quest_document(who, WORLD, &p.journal, &final_read)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(
        retry.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &open(&p, 103, 1, 0))
            .await,
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::MissingItem
        ))
    ));
    let mut changed = first.clone();
    changed.slot = 1;
    changed.document = id("other-note");
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &changed)
            .await,
        Err(Error::QuestJournalConflict)
    ));
    let revision = retry.journal.progress(&id("quest")).unwrap().revision();
    let turn = command(2, QuestJournalAction::TurnIn, revision);
    let done = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &turn)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    let same = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &turn)
        .await
        .unwrap();
    assert!(same.replayed);
    assert_eq!(same.experience_granted, 0);
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let restored = store
        .open_quest_document(who, WORLD, &p.journal, &first)
        .await
        .unwrap();
    assert!(restored.replayed);
    assert_eq!(restored.journal, done.journal);
    assert_eq!(restored.inventory.unwrap().revision(), 1);
    assert_eq!(
        store
            .load_progression(who, WORLD, &p.xp)
            .await
            .unwrap()
            .total(),
        25
    );
    let n: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_journal_events WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(n, 3);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn read_history_survives_loss_and_possession_turn_in_remains_atomic() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(true);
    let who = initialize(&mut store, "Documentmix", 2, &p).await;
    accept(&mut store, who, &p).await;
    store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 1, 0, 0))
        .await
        .unwrap();
    let ready = store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 2, 0, 0))
        .await
        .unwrap();
    assert_eq!(
        ready.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    let removed = store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 1 },
            }
            .into(),
            &p.journal,
        )
        .await
        .unwrap();
    let journal = removed.journal.as_ref().unwrap();
    let progress = journal.journal.progress(&id("quest")).unwrap();
    assert_eq!(progress.status(), Status::Active);
    assert_eq!(progress.count(&id("read-final")), Some(1));
    assert_eq!(progress.count(&id("bring")), Some(1));
    let failed = store
        .command_quest_journal(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            &command(2, QuestJournalAction::TurnIn, progress.revision()),
        )
        .await;
    assert!(matches!(
        failed,
        Err(Error::QuestJournalCommand(Rule::NotReady))
    ));
    // Existing ordinary reward transactions must retain the read journal too.
    let mut reward: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    reward["rewards"][0]["grants"][0]["item"] = serde_json::json!("theia.dev.training-stone");
    let rewards =
        RewardCatalog::from_json(&serde_json::to_vec(&reward).unwrap(), &p.items).unwrap();
    let recipe = id("theia.dev.first-training-kill");
    let offer = store
        .offer_reward(who, WORLD, &rewards, &recipe)
        .await
        .unwrap();
    let regained = store
        .claim_reward_with_quest_journal(who, WORLD, &rewards, offer.id, 1, &p.journal)
        .await
        .unwrap();
    let q = regained
        .journal
        .as_ref()
        .unwrap()
        .journal
        .progress(&id("quest"))
        .unwrap();
    assert_eq!(q.status(), Status::Ready);
    assert_eq!(q.count(&id("read-final")), Some(1));
    let done = store
        .command_quest_journal(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            &command(2, QuestJournalAction::TurnIn, q.revision()),
        )
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    assert!(done.inventory.unwrap().slots()[0].is_none());
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn forged_moved_and_failed_opens_do_not_commit_credit_or_receipts() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false);
    let who = initialize(&mut store, "Documentguard", 2, &p).await;
    let initial = accept(&mut store, who, &p).await;
    let read = open(&p, 1, 0, 0);
    let mut wrong = read.clone();
    wrong.profile_hash[0] ^= 1;
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &wrong)
            .await,
        Err(Error::DocumentProfileMismatch)
    ));
    wrong = read.clone();
    wrong.document = id("unknown");
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &wrong)
            .await,
        Err(Error::DocumentCommand(_))
    ));
    wrong = read.clone();
    wrong.slot = 1;
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &wrong)
            .await,
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::WrongItem
        ))
    ));
    let foreign = Binding {
        owner: OwnerId::new(uuid::Uuid::from_bytes([99; 16])).unwrap(),
        ..who
    };
    assert!(matches!(
        store
            .open_quest_document(foreign, WORLD, &p.journal, &read)
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store
            .open_quest_document(who, [212; 32], &p.journal, &read)
            .await,
        Err(Error::WorldMismatch)
    ));
    assert!(matches!(
        store
            .record_quest_event(
                who,
                WORLD,
                &p.journal,
                nz(987),
                &EventKey::ReadDocument(id("note"))
            )
            .await,
        Err(Error::QuestJournalCommand(Rule::InvalidProgress))
    ));
    let mut changed_xp =
        include_bytes!("../../../content/gameplay/development-progression.json").to_vec();
    changed_xp.push(b' ');
    let changed = QuestJournalCatalog::from_json_with_documents(
        &p.quest_bytes,
        None,
        &ProgressionCatalog::from_json(&changed_xp).unwrap(),
        &p.documents,
    )
    .unwrap();
    assert!(matches!(
        store.open_quest_document(who, WORLD, &changed, &read).await,
        Err(Error::QuestJournalCatalogMismatch)
    ));
    assert!(matches!(
        store.ensure_quest_journal(who, WORLD, &changed).await,
        Err(Error::QuestJournalCatalogMismatch)
    ));
    let moved = store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            Command {
                expected_revision: 0,
                operation: Operation::Move { from: 0, to: 2 },
            }
            .into(),
            &p.journal,
        )
        .await
        .unwrap();
    assert_eq!(moved.bag.revision(), 1);
    assert!(matches!(
        store
            .open_quest_document(who, WORLD, &p.journal, &read)
            .await,
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::StaleRevision
        ))
    ));
    let unchanged = store
        .load_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    assert_eq!(unchanged.journal, initial.journal);
    assert_eq!(unchanged.revision, initial.revision);
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_journal_events WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 0);
    admin.batch_execute(&format!("CREATE FUNCTION theia.document_fail() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.realm_id='{}'::uuid THEN RAISE EXCEPTION 'owned document rollback fixture'; END IF; RETURN NEW; END $$; CREATE TRIGGER document_fail BEFORE INSERT ON theia.quest_journal_events FOR EACH ROW EXECUTE FUNCTION theia.document_fail();",realm.value())).await.unwrap();
    let new = open(&p, 1, 1, 2);
    let failed = store
        .open_quest_document(who, WORLD, &p.journal, &new)
        .await;
    admin.batch_execute("DROP TRIGGER document_fail ON theia.quest_journal_events; DROP FUNCTION theia.document_fail();").await.unwrap();
    assert!(matches!(failed, Err(Error::Database(_))));
    let unchanged = store
        .load_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    assert_eq!(unchanged.journal, initial.journal);
    assert_eq!(unchanged.revision, initial.revision);
    let valid = store
        .open_quest_document(who, WORLD, &p.journal, &new)
        .await
        .unwrap();
    assert!(!valid.replayed);
    assert_eq!(
        valid.journal.progress(&id("quest")).unwrap().stage(),
        &id("second")
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

async fn wait_bag_lock(tx: &tokio_postgres::Transaction<'_>) {
    tokio::time::timeout(Duration::from_millis(300),async {loop {
        tx.query_one("SELECT pg_stat_clear_snapshot()",&[]).await.unwrap();
        let waiting:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE '%theia.backpacks%')",&[]).await.unwrap().get(0);
        if waiting {break;}tokio::time::sleep(Duration::from_millis(2)).await;
    }}).await.unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn accepted_open_survives_lost_answers_shutdown_and_later_stage_acceptance() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false);
    let who = initialize(&mut store, "Documentworker", 2, &p).await;
    let (publisher, worker) = Worker::start(store);
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let original = open(&p, 1, 0, 0);
    drop(
        publisher
            .enqueue_document_open(who, WORLD, &p.journal, original.clone())
            .unwrap(),
    );
    wait_bag_lock(&guard).await;
    let accept = publisher
        .enqueue_quest_journal_command(
            who,
            WORLD,
            &p.journal,
            &p.xp,
            command(1, QuestJournalAction::Accept, 0),
        )
        .unwrap();
    let stopping = tokio::spawn(worker.shutdown());
    tokio::task::yield_now().await;
    guard.commit().await.unwrap();
    let status = stopping.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_events, 1);
    assert_eq!(status.journal_commands, 1);
    assert!(accept.await.unwrap().is_ok());
    assert!(
        publisher
            .enqueue_document_open(who, WORLD, &p.journal, original.clone())
            .is_err()
    );
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let retry = store
        .open_quest_document(who, WORLD, &p.journal, &original)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(
        retry
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-first")),
        Some(0)
    );
    store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 2, 0, 0))
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    let missing = open(&p, 9, 0, 15);
    assert!(matches!(
        publisher
            .open_quest_document(who, WORLD, &p.journal, missing)
            .await,
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::MissingItem
        ))
    ));
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let final_read = open(&p, 3, 0, 0);
    drop(
        publisher
            .enqueue_document_open(who, WORLD, &p.journal, final_read.clone())
            .unwrap(),
    );
    wait_bag_lock(&guard).await;
    let stopping = tokio::spawn(worker.shutdown());
    tokio::task::yield_now().await;
    guard.commit().await.unwrap();
    let status = stopping.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_rejected, 1);
    assert_eq!(status.journal_events, 1);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let retry = store
        .open_quest_document(who, WORLD, &p.journal, &final_read)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(
        retry.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn delayed_open_rechecks_the_changed_locked_bag_and_worker_recovers_after_reacquisition() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false);
    let who = initialize(&mut store, "Documentrace", 2, &p).await;
    accept(&mut store, who, &p).await;
    let before = store.load_inventory(who, WORLD, &p.items).await.unwrap();
    let removed = before
        .propose(
            p.items.catalog(),
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 2 },
            },
        )
        .unwrap();
    let guard = admin.transaction().await.unwrap();
    // Valid authority-shaped bag update is held uncommitted to force the read
    // to wait. The product mutator uses the same backpack-before-journal order.
    guard.execute("UPDATE theia.backpacks SET revision=1,state=$3,last_command=$4 WHERE realm_id=$1 AND character_id=$2",&[&realm.value(),&who.character.value(),&removed.to_json(),&&[73u8;32][..]]).await.unwrap();
    let (publisher, worker) = Worker::start(store);
    let answer = publisher
        .enqueue_document_open(who, WORLD, &p.journal, open(&p, 1, 0, 0))
        .unwrap();
    wait_bag_lock(&guard).await;
    guard.commit().await.unwrap();
    assert!(matches!(
        answer.await.unwrap(),
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::StaleRevision
        ))
    ));
    assert!(matches!(
        publisher
            .open_quest_document(who, WORLD, &p.journal, open(&p, 2, 1, 0))
            .await,
        Err(Error::DocumentCommand(
            theia_core::documents::OpenError::MissingItem
        ))
    ));
    let saved = publisher
        .load_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    assert_eq!(
        saved
            .journal
            .progress(&id("quest"))
            .unwrap()
            .count(&id("read-first")),
        Some(0)
    );
    assert_eq!(saved.revision, 1);
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    raw["rewards"][0]["grants"][0]["item"] = serde_json::json!("theia.dev.training-stone");
    raw["rewards"][0]["grants"][0]["count"] = serde_json::json!(2);
    let rewards = RewardCatalog::from_json(&serde_json::to_vec(&raw).unwrap(), &p.items).unwrap();
    let available = publisher
        .offer_reward(who, WORLD, &rewards, &id("theia.dev.first-training-kill"))
        .await
        .unwrap();
    publisher
        .claim_reward_with_quest_journal(who, WORLD, &rewards, available.id, 1, &p.journal)
        .await
        .unwrap();
    let read = publisher
        .open_quest_document(who, WORLD, &p.journal, open(&p, 3, 2, 0))
        .await
        .unwrap();
    assert_eq!(
        read.journal.progress(&id("quest")).unwrap().stage(),
        &id("second")
    );
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_rejected, 2);
    assert_eq!(status.journal_events, 1);
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn a_full_backpack_can_read_an_owned_split_note_without_item_mutation() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false);
    let who = initialize(&mut store, "Documentfull", 20, &p).await;
    for to in 2..16 {
        store
            .mutate_items_with_quest_journal(
                who,
                WORLD,
                &p.items,
                Command {
                    expected_revision: u64::from(to - 2),
                    operation: Operation::Split {
                        from: 0,
                        to,
                        amount: 1,
                    },
                }
                .into(),
                &p.journal,
            )
            .await
            .unwrap();
    }
    let bag = store.load_inventory(who, WORLD, &p.items).await.unwrap();
    assert!(bag.slots().iter().all(Option::is_some));
    assert_eq!(bag.revision(), 14);
    accept(&mut store, who, &p).await;
    let first = store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 1, 14, 15))
        .await
        .unwrap();
    assert_eq!(first.inventory.as_ref(), Some(&bag));
    let second = store
        .open_quest_document(who, WORLD, &p.journal, &open(&p, 2, 14, 2))
        .await
        .unwrap();
    assert_eq!(second.inventory.as_ref(), Some(&bag));
    assert_eq!(
        second.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[test]
fn document_search_metadata_is_exactly_bound_without_changing_existing_profiles() {
    let p = profiles(false);
    let mut a = serde_json::json!({"schema":1,"id":"map.areas","revision":1,"areas":[{
        "id":"map.search","name":"Orders search","min":[7990.,-64.,7990.],"max":[8010.,64.,8010.],"map_objectives":["read-first"]
    }]});
    let bytes = serde_json::to_vec(&a).unwrap();
    let bound = QuestJournalCatalog::from_json_with_documents(
        &p.quest_bytes,
        Some(&bytes),
        &p.xp,
        &p.documents,
    )
    .unwrap();
    assert_ne!(bound.hash(), p.journal.hash());
    let mut changed = bytes.clone();
    changed.push(b' ');
    assert_ne!(
        bound.hash(),
        QuestJournalCatalog::from_json_with_documents(
            &p.quest_bytes,
            Some(&changed),
            &p.xp,
            &p.documents
        )
        .unwrap()
        .hash()
    );
    let mut expected = theia_core::quests::journal::PROFILE_DOMAIN.to_vec();
    expected.extend(&p.quest_bytes);
    expected.extend(p.xp.hash());
    let hash = |bytes: &[u8]| -> [u8; 32] {
        ring::digest::digest(&ring::digest::SHA256, bytes)
            .as_ref()
            .try_into()
            .unwrap()
    };
    let mut areas = theia_core::quests::areas::PROFILE_DOMAIN.to_vec();
    areas.extend(hash(&expected));
    areas.extend(&bytes);
    let mut items = theia_core::quests::journal::ITEM_PROFILE_DOMAIN.to_vec();
    items.extend(hash(&areas));
    items.extend(p.items.hash());
    let mut documents = theia_core::documents::JOURNAL_PROFILE_DOMAIN.to_vec();
    documents.extend(hash(&items));
    documents.extend(p.documents.hash());
    assert_eq!(bound.hash(), hash(&documents));
    for links in [serde_json::json!(["unknown"]), serde_json::json!([])] {
        a["areas"][0]["map_objectives"] = links;
        assert!(
            QuestJournalCatalog::from_json_with_documents(
                &p.quest_bytes,
                Some(&serde_json::to_vec(&a).unwrap()),
                &p.xp,
                &p.documents
            )
            .is_err()
        );
    }
    assert_eq!(
        p.journal.hash(),
        QuestJournalCatalog::from_json_with_documents(&p.quest_bytes, None, &p.xp, &p.documents)
            .unwrap()
            .hash()
    );
}
