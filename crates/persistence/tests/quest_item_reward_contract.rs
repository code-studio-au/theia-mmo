//! Real owned SQL; NPC/session entitlement is a synthetic trusted fixture.
#[path = "support/item_rewards.rs"]
mod fixtures;
mod support;
use fixtures::*;
use std::time::Duration;
use theia_core::{
    inventory::{Command, ItemCommand, Operation},
    quests::journal::Status,
};
use theia_persistence::*;
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn fixed_grants_delivery_xp_completion_and_receipt_rollback_and_retry_atomically() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(true, false);
    let who = initialize(&mut store, "Fixedatomic", &p, &bag(&p, false, 3)).await;
    ready(&mut store, who, &p).await;
    let mut accept = command(2, QuestJournalAction::Accept, 0);
    accept.quest = id("watcher");
    let before = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &accept)
        .await
        .unwrap();
    let cmd = command(
        20,
        QuestJournalAction::TurnIn,
        before.journal.progress(&id("quest")).unwrap().revision(),
    );
    admin.batch_execute("CREATE FUNCTION theia.reject_fixed_reward_action() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned fixed reward rollback'; END $$; CREATE TRIGGER reject_fixed_reward_action BEFORE INSERT ON theia.quest_journal_actions FOR EACH ROW EXECUTE FUNCTION theia.reject_fixed_reward_action();").await.unwrap();
    assert!(
        store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_fixed_reward_action ON theia.quest_journal_actions; DROP FUNCTION theia.reject_fixed_reward_action();").await.unwrap();
    let restored = store
        .load_quest_journal(who, WORLD, &p.journal)
        .await
        .unwrap();
    assert_eq!(restored.journal, before.journal);
    assert_eq!(restored.inventory, before.inventory);
    assert_eq!(
        store
            .load_progression(who, WORLD, &p.xp)
            .await
            .unwrap()
            .total(),
        0
    );
    let done = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    let b = done.inventory.as_ref().unwrap();
    assert_eq!(b.revision(), 1);
    assert_eq!(count(b, "theia.dev.training-stone"), 0);
    assert_eq!(count(b, "theia.dev.training-token"), 3);
    assert_eq!(
        done.journal.progress(&id("quest")).unwrap().status(),
        Status::Completed
    );
    assert_eq!(
        done.journal.progress(&id("watcher")).unwrap().status(),
        Status::Active
    );
    let moved = store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            ItemCommand::Backpack(Command {
                expected_revision: 1,
                operation: Operation::Move { from: 0, to: 15 },
            }),
            &p.journal,
        )
        .await
        .unwrap();
    let replay = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.experience_granted, 0);
    assert_eq!(replay.inventory, Some(moved.bag.clone()));
    let mut conflict = cmd.clone();
    conflict.npc = id("other-guide");
    assert!(matches!(
        store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &conflict)
            .await,
        Err(Error::QuestJournalConflict)
    ));
    store.close().await.unwrap();
    let mut reopened = Store::connect_loopback(&url, realm).await.unwrap();
    let again = reopened
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert!(again.replayed);
    assert_eq!(again.inventory, Some(moved.bag));
    assert_eq!(again.experience.unwrap().total(), 25);
    shutdown(reopened, admin, driver).await;
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn full_bag_bundle_failure_and_freed_delivery_slots_preserve_or_commit_whole_rewards() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    for (name, delivery, bundle, fits) in [
        ("Fixedfreed", true, false, true),
        ("Fixedordinary", false, false, false),
        ("Fixedbundle", true, true, false),
    ] {
        let p = profiles(delivery, bundle);
        let seed = bag(&p, true, if delivery { 3 } else { 0 });
        let who = initialize(&mut store, name, &p, &seed).await;
        let before = ready(&mut store, who, &p).await;
        let cmd = command(
            20,
            QuestJournalAction::TurnIn,
            before.journal.progress(&id("quest")).unwrap().revision(),
        );
        let result = store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
            .await;
        let done = if fits {
            result.unwrap()
        } else {
            assert!(matches!(
                result,
                Err(Error::RewardCommand(theia_core::rewards::Error::NoCapacity))
            ));
            assert_eq!(
                store.load_inventory(who, WORLD, &p.items).await.unwrap(),
                seed
            );
            assert_eq!(
                store
                    .load_quest_journal(who, WORLD, &p.journal)
                    .await
                    .unwrap()
                    .journal,
                before.journal
            );
            assert_eq!(
                store
                    .load_progression(who, WORLD, &p.xp)
                    .await
                    .unwrap()
                    .total(),
                0
            );
            store
                .mutate_items_with_quest_journal(
                    who,
                    WORLD,
                    &p.items,
                    ItemCommand::Backpack(Command {
                        expected_revision: 0,
                        operation: Operation::Destroy {
                            from: 15,
                            amount: 1,
                        },
                    }),
                    &p.journal,
                )
                .await
                .unwrap();
            store
                .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
                .await
                .unwrap()
        };
        assert_eq!(done.experience_granted, 25);
        let b = done.inventory.unwrap();
        assert_eq!(b.revision(), if fits { 1 } else { 2 });
        assert_eq!(count(&b, "theia.dev.training-token"), 3);
        assert_eq!(count(&b, "theia.dev.training-stone"), u32::from(bundle));
    }
    shutdown(store, admin, driver).await;
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn reward_actions_recheck_owner_world_exact_content_stale_intent_and_current_delivery_items()
{
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(true, false);
    let who = initialize(&mut store, "Fixedcontext", &p, &bag(&p, false, 3)).await;
    let foreign = initialize(&mut store, "Fixedforeign", &p, &bag(&p, false, 3)).await;
    let before = ready(&mut store, who, &p).await;
    let cmd = command(
        20,
        QuestJournalAction::TurnIn,
        before.journal.progress(&id("quest")).unwrap().revision(),
    );
    assert!(
        store
            .command_quest_journal(
                Binding {
                    owner: foreign.owner,
                    ..who
                },
                WORLD,
                &p.journal,
                &p.xp,
                &cmd
            )
            .await
            .is_err()
    );
    assert!(
        store
            .command_quest_journal(who, [220; 32], &p.journal, &p.xp, &cmd)
            .await
            .is_err()
    );
    for index in 0..4 {
        let mut raw = [
            p.quest_raw.clone(),
            ITEMS.to_vec(),
            XP.to_vec(),
            p.reward_raw.clone(),
        ];
        raw[index].push(b' ');
        let changed = from_raw(&raw[0], &raw[1], &raw[2], &raw[3]);
        assert!(
            store
                .command_quest_journal(who, WORLD, &changed.journal, &changed.xp, &cmd)
                .await
                .is_err()
        );
    }
    let mut stale = cmd.clone();
    stale.expected_revision += 1;
    assert!(
        store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &stale)
            .await
            .is_err()
    );
    let mut wrong = cmd.clone();
    wrong.npc = id("wrong-guide");
    assert!(
        store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &wrong)
            .await
            .is_err()
    );
    assert_eq!(
        store
            .load_quest_journal(who, WORLD, &p.journal)
            .await
            .unwrap()
            .journal,
        before.journal
    );
    store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            ItemCommand::Backpack(Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 1 },
            }),
            &p.journal,
        )
        .await
        .unwrap();
    assert!(matches!(
        store
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
            .await,
        Err(Error::QuestJournalCommand(_))
    ));
    let b = store.load_inventory(who, WORLD, &p.items).await.unwrap();
    assert_eq!(count(&b, "theia.dev.training-stone"), 2);
    assert_eq!(count(&b, "theia.dev.training-token"), 0);
    assert_eq!(
        store
            .load_progression(who, WORLD, &p.xp)
            .await
            .unwrap()
            .total(),
        0
    );
    shutdown(store, admin, driver).await;
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn accepted_turn_in_survives_lost_reply_shutdown_duplicates_and_following_bag_mutation() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(true, false);
    let who = initialize(&mut store, "Fixedworker", &p, &bag(&p, false, 3)).await;
    let ready = ready(&mut store, who, &p).await;
    let cmd = command(
        20,
        QuestJournalAction::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    let answer = publisher
        .enqueue_quest_journal_command(who, WORLD, &p.journal, &p.xp, cmd.clone())
        .unwrap();
    tokio::time::timeout(Duration::from_secs(3),async {
        loop {let waiting:bool=guard.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE pid<>pg_backend_pid() AND wait_event_type='Lock' AND query LIKE '%theia.backpacks%')",&[]).await.unwrap().get(0);if waiting {break;}tokio::time::sleep(Duration::from_millis(5)).await;}
    }).await.unwrap();
    drop(answer);
    let retry = publisher
        .enqueue_quest_journal_command(who, WORLD, &p.journal, &p.xp, cmd.clone())
        .unwrap();
    let moved = publisher
        .enqueue_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            ItemCommand::Backpack(Command {
                expected_revision: 1,
                operation: Operation::Move { from: 0, to: 15 },
            }),
            &p.journal,
        )
        .unwrap();
    let mut ending = tokio::spawn(worker.shutdown());
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut ending)
            .await
            .is_err()
    );
    guard.commit().await.unwrap();
    let status = ending.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_commands, 2);
    assert_eq!(status.journal_replayed, 1);
    assert_eq!(status.inventory_committed, 1);
    assert!(retry.await.unwrap().unwrap().replayed);
    assert_eq!(moved.await.unwrap().unwrap().bag.revision(), 2);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let replay = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.experience.unwrap().total(), 25);
    let b = replay.inventory.unwrap();
    assert_eq!(b.revision(), 2);
    assert!(b.slots()[0].is_none());
    assert_eq!(count(&b, "theia.dev.training-token"), 3);
    shutdown(store, admin, driver).await;
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn worker_capacity_refusal_is_nonfatal_and_original_failed_intent_can_succeed_after_space_is_freed()
 {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let p = profiles(false, false);
    let who = initialize(&mut store, "Fixednonfatal", &p, &bag(&p, true, 0)).await;
    let ready = ready(&mut store, who, &p).await;
    let cmd = command(
        20,
        QuestJournalAction::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    let (publisher, worker) = Worker::start(store);
    assert!(matches!(
        publisher
            .command_quest_journal(who, WORLD, &p.journal, &p.xp, cmd.clone())
            .await,
        Err(Error::RewardCommand(theia_core::rewards::Error::NoCapacity))
    ));
    publisher
        .enqueue_items_with_quest_journal(
            who,
            WORLD,
            &p.items,
            ItemCommand::Backpack(Command {
                expected_revision: 0,
                operation: Operation::Destroy {
                    from: 15,
                    amount: 1,
                },
            }),
            &p.journal,
        )
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    let done = publisher
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, cmd)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    assert_eq!(
        count(done.inventory.as_ref().unwrap(), "theia.dev.training-token"),
        3
    );
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert_eq!(status.journal_rejected, 1);
    assert_eq!(status.journal_commands, 1);
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[path = "support/documents.rs"]
mod documents;
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn historical_read_receipts_survive_rewarded_delivery_consumption_without_extra_grants() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let mut p = documents::profiles(true);
    let r=serde_json::to_vec(&serde_json::json!({"schema":1,"id":"fixed-document-rewards","revision":1,"quest_catalog":p.journal.catalog().id().as_str(),"item_catalog":p.items.catalog().id().as_str(),"rewards":[{"quest":"quest","grants":[{"item":"theia.dev.training-token","count":3}]}]})).unwrap();
    p.journal = p.journal.with_item_rewards(&r, &p.items).unwrap();
    let who = documents::initialize(&mut store, "Fixedread", 2, &p).await;
    documents::accept(&mut store, who, &p).await;
    store
        .open_quest_document(
            who,
            documents::WORLD,
            &p.journal,
            &documents::open(&p, 100, 0, 0),
        )
        .await
        .unwrap();
    let before = store
        .open_quest_document(
            who,
            documents::WORLD,
            &p.journal,
            &documents::open(&p, 101, 0, 0),
        )
        .await
        .unwrap();
    let cmd = documents::command(
        20,
        QuestJournalAction::TurnIn,
        before.journal.progress(&id("quest")).unwrap().revision(),
    );
    let done = store
        .command_quest_journal(who, documents::WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    let b = done.inventory.as_ref().unwrap();
    assert_eq!(b.revision(), 1);
    assert_eq!(count(b, "theia.dev.training-stone"), 0);
    assert_eq!(count(b, "theia.dev.training-token"), 4);
    let replay = store
        .open_quest_document(
            who,
            documents::WORLD,
            &p.journal,
            &documents::open(&p, 100, 0, 0),
        )
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.inventory, done.inventory);
    assert_eq!(replay.journal, done.journal);
    assert_eq!(
        store
            .load_progression(who, documents::WORLD, &p.xp)
            .await
            .unwrap()
            .total(),
        25
    );
    shutdown(store, admin, driver).await;
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn capped_experience_still_commits_fixed_items_and_once_only_completion() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let base = profiles(true, false);
    let mut xp: serde_json::Value = serde_json::from_slice(XP).unwrap();
    xp["level_thresholds"] = serde_json::json!([0, 25]);
    let p = from_raw(
        &base.quest_raw,
        ITEMS,
        &serde_json::to_vec(&xp).unwrap(),
        &base.reward_raw,
    );
    let who = initialize(&mut store, "Fixedcapped", &p, &bag(&p, false, 3)).await;
    store
        .award_experience(
            who,
            WORLD,
            &p.xp,
            &theia_core::progression::KillEvent {
                id: theia_core::progression::KillId::from_victim(theia_core::identity::EntityKey {
                    epoch: theia_core::identity::WorldEpoch::new(1).unwrap(),
                    entity: theia_core::identity::EntityId::new(99).unwrap(),
                }),
                creature: id("theia.dev.training-creature"),
            },
        )
        .await
        .unwrap();
    let ready = ready(&mut store, who, &p).await;
    let cmd = command(
        20,
        QuestJournalAction::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    let done = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 0);
    assert_eq!(done.experience.unwrap().total(), 25);
    assert_eq!(
        count(done.inventory.as_ref().unwrap(), "theia.dev.training-token"),
        3
    );
    let replay = store
        .command_quest_journal(who, WORLD, &p.journal, &p.xp, &cmd)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.experience_granted, 0);
    assert_eq!(replay.inventory, done.inventory);
    shutdown(store, admin, driver).await;
}
