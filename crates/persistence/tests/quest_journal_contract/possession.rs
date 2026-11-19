//! Owned SQL; synthetic authorization and administrative removal fixture only.
use super::*;
use theia_core::inventory::{Bag, Command, Operation, Stack};
use theia_persistence::InventoryCatalog;
const ITEMS: &[u8] =
    include_bytes!("../../../../content/gameplay/development-equipment-items.json");
fn profiles() -> (ProgressionCatalog, InventoryCatalog, QuestJournalCatalog) {
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let items = InventoryCatalog::from_json(ITEMS).unwrap();
    let bytes = serde_json::to_vec(&json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Bring","description":"Test","prerequisites":[],
        "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
            "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.training-stone",
            "required":3,"consume_on_turn_in":true}]}]}]})).unwrap();
    let q = QuestJournalCatalog::from_json(&bytes, &xp)
        .unwrap()
        .with_inventory(&items)
        .unwrap();
    (xp, items, q)
}
fn bag(items: &InventoryCatalog, revision: u64, count: u32) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    if count > 0 {
        slots[0] = Stack::new(items.catalog(), id("theia.dev.training-stone"), count);
    }
    Bag::from_slots(items.catalog(), revision, slots).unwrap()
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn item_turn_in_consumption_xp_receipt_rollback_replay_and_reopen_share_one_commit() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, items, q) = profiles();
    let b = character(&mut store, "Itematomic").await;
    assert!(matches!(
        store.ensure_quest_journal(b, WORLD, &q).await,
        Err(Error::InventoryMissing)
    ));
    store
        .ensure_inventory(b, WORLD, &items, &bag(&items, 0, 3))
        .await
        .unwrap();
    initialize(&mut store, b, &xp, &q).await;
    let accepted = store
        .command_quest_journal(
            b,
            WORLD,
            &q,
            &xp,
            &command(1, id("quest"), Action::Accept, 0),
        )
        .await
        .unwrap();
    assert_eq!(
        accepted.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    let split = store
        .mutate_inventory(
            b,
            WORLD,
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Split {
                    from: 0,
                    to: 15,
                    amount: 1,
                },
            },
        )
        .await
        .unwrap();
    let ready = store.load_quest_journal(b, WORLD, &q).await.unwrap();
    assert_eq!(ready.journal, accepted.journal);
    assert_eq!(ready.inventory, Some(split.bag.clone()));
    let turn_in = command(
        2,
        id("quest"),
        Action::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    admin.batch_execute("CREATE FUNCTION theia.reject_item_action() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned item receipt failure'; END $$; CREATE TRIGGER reject_item_action BEFORE INSERT ON theia.quest_journal_actions FOR EACH ROW EXECUTE FUNCTION theia.reject_item_action();").await.unwrap();
    assert!(
        store
            .command_quest_journal(b, WORLD, &q, &xp, &turn_in)
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_item_action ON theia.quest_journal_actions; DROP FUNCTION theia.reject_item_action();").await.unwrap();
    assert_eq!(
        store.load_inventory(b, WORLD, &items).await.unwrap(),
        split.bag
    );
    assert_eq!(
        store
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .journal,
        ready.journal
    );
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        0
    );
    let done = store
        .command_quest_journal(b, WORLD, &q, &xp, &turn_in)
        .await
        .unwrap();
    assert_eq!(done.experience_granted, 25);
    let empty = done.inventory.as_ref().unwrap();
    assert_eq!(empty.revision(), 2);
    assert!(empty.slots().iter().all(Option::is_none));
    assert_eq!(
        done.journal.progress(&id("quest")).unwrap().status(),
        Status::Completed
    );
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let retry = store
        .command_quest_journal(b, WORLD, &q, &xp, &turn_in)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(retry.experience_granted, 0);
    assert_eq!(retry.inventory, done.inventory);
    assert_eq!(retry.journal, done.journal);
    assert_eq!(retry.experience.unwrap().total(), 25);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn current_locked_bag_overrules_cached_ready_and_reconciles_on_load_and_accept_replay() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, items, q) = profiles();
    let b = character(&mut store, "Itemmissing").await;
    store
        .ensure_inventory(b, WORLD, &items, &bag(&items, 0, 3))
        .await
        .unwrap();
    initialize(&mut store, b, &xp, &q).await;
    let accept = command(1, id("quest"), Action::Accept, 0);
    let ready = store
        .command_quest_journal(b, WORLD, &q, &xp, &accept)
        .await
        .unwrap();
    let turn = command(
        2,
        id("quest"),
        Action::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    // Explicit SQL fixture simulates an older inventory-only writer, not a player deletion path.
    let reduced = bag(&items, 1, 2);
    admin.execute("UPDATE theia.backpacks SET revision=1,state=$1,last_command=decode(repeat('00',32),'hex') WHERE realm_id=$2 AND character_id=$3", &[&reduced.to_json(),&realm.value(),&b.character.value()]).await.unwrap();
    assert!(matches!(
        store.command_quest_journal(b, WORLD, &q, &xp, &turn).await,
        Err(Error::QuestJournalCommand(
            theia_core::quests::journal::Error::NotReady
        ))
    ));
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        0
    );
    let active = store.load_quest_journal(b, WORLD, &q).await.unwrap();
    let state = active.journal.progress(&id("quest")).unwrap();
    assert_eq!(state.status(), Status::Active);
    assert_eq!(state.count(&id("goal")), Some(2));
    assert!(active.revision > ready.revision);
    let restored = bag(&items, 2, 3);
    admin
        .execute(
            "UPDATE theia.backpacks SET revision=2,state=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&restored.to_json(), &realm.value(), &b.character.value()],
        )
        .await
        .unwrap();
    let retry = store
        .command_quest_journal(b, WORLD, &q, &xp, &accept)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(
        retry.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    assert_eq!(retry.inventory, Some(restored));
    assert!(retry.revision > active.revision);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn vendor_acquisition_and_journal_rollback_together_then_exact_retry_returns_both_views() {
    use theia_core::vendors::Purchase;
    use theia_persistence::VendorCatalog;
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, items, q) = profiles();
    let vendor = VendorCatalog::from_json(
        include_bytes!("../../../../content/gameplay/development-vendor.json"),
        &items,
    )
    .unwrap();
    let b = character(&mut store, "Itemvendor").await;
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(items.catalog(), id("theia.dev.training-token"), 2);
    let seed = Bag::from_slots(items.catalog(), 0, slots).unwrap();
    store
        .ensure_inventory(b, WORLD, &items, &seed)
        .await
        .unwrap();
    initialize(&mut store, b, &xp, &q).await;
    let active = store
        .command_quest_journal(
            b,
            WORLD,
            &q,
            &xp,
            &command(1, id("quest"), Action::Accept, 0),
        )
        .await
        .unwrap();
    assert_eq!(
        active.journal.progress(&id("quest")).unwrap().status(),
        Status::Active
    );
    let purchase = Purchase {
        expected_revision: 0,
        offer: id("theia.dev.training-vendor.stones"),
    };
    admin.batch_execute("CREATE FUNCTION theia.reject_item_projection() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned projection failure'; END $$; CREATE TRIGGER reject_item_projection BEFORE UPDATE ON theia.quest_journal_heads FOR EACH ROW EXECUTE FUNCTION theia.reject_item_projection();").await.unwrap();
    assert!(
        store
            .purchase_vendor_with_quest_journal(b, WORLD, &vendor, &purchase, &q)
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_item_projection ON theia.quest_journal_heads; DROP FUNCTION theia.reject_item_projection();").await.unwrap();
    assert_eq!(store.load_inventory(b, WORLD, &items).await.unwrap(), seed);
    assert_eq!(
        store
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .journal,
        active.journal
    );
    let bought = store
        .purchase_vendor_with_quest_journal(b, WORLD, &vendor, &purchase, &q)
        .await
        .unwrap();
    let journal = bought.journal.as_ref().unwrap();
    assert_eq!(journal.inventory.as_ref(), Some(&bought.bag));
    assert_eq!(
        journal.journal.progress(&id("quest")).unwrap().status(),
        Status::Ready
    );
    let replay = store
        .purchase_vendor_with_quest_journal(b, WORLD, &vendor, &purchase, &q)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag, bought.bag);
    assert_eq!(replay.journal.unwrap().journal, journal.journal);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn accepted_compound_item_job_drains_after_response_loss_and_preserves_split_progress() {
    use theia_core::inventory::ItemCommand;
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, items, q) = profiles();
    let b = character(&mut store, "Itemretained").await;
    store
        .ensure_inventory(b, WORLD, &items, &bag(&items, 0, 3))
        .await
        .unwrap();
    initialize(&mut store, b, &xp, &q).await;
    let accepted = store
        .command_quest_journal(
            b,
            WORLD,
            &q,
            &xp,
            &command(1, id("quest"), Action::Accept, 0),
        )
        .await
        .unwrap();
    let guard = admin.transaction().await.unwrap();
    guard
        .batch_execute("LOCK TABLE theia.backpacks IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();
    let (publisher, worker) = theia_persistence::Worker::start(store);
    let cmd = ItemCommand::Backpack(Command {
        expected_revision: 0,
        operation: Operation::Split {
            from: 0,
            to: 15,
            amount: 1,
        },
    });
    let mut answer = publisher
        .enqueue_items_with_quest_journal(b, WORLD, &items, cmd, &q)
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
    assert_eq!(status.inventory_committed, 1);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let retry = store
        .mutate_items_with_quest_journal(b, WORLD, &items, cmd, &q)
        .await
        .unwrap();
    assert!(retry.replayed);
    assert_eq!(retry.bag.revision(), 1);
    assert_eq!(retry.journal.unwrap().journal, accepted.journal);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires disposable owned PostgreSQL; run tools/qualify_persistence.py"]
async fn actual_destroy_command_reverses_ready_with_atomic_rollback_and_never_replays_removal() {
    use theia_core::inventory::ItemCommand;
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (xp, items, q) = profiles();
    let b = character(&mut store, "Itemdestroy").await;
    store
        .ensure_inventory(b, WORLD, &items, &bag(&items, 0, 3))
        .await
        .unwrap();
    initialize(&mut store, b, &xp, &q).await;
    let ready = store
        .command_quest_journal(
            b,
            WORLD,
            &q,
            &xp,
            &command(1, id("quest"), Action::Accept, 0),
        )
        .await
        .unwrap();
    let split = ItemCommand::Backpack(Command {
        expected_revision: 0,
        operation: Operation::Split {
            from: 0,
            to: 15,
            amount: 1,
        },
    });
    store
        .mutate_items_with_quest_journal(b, WORLD, &items, split, &q)
        .await
        .unwrap();
    let destroy = ItemCommand::Backpack(Command {
        expected_revision: 1,
        operation: Operation::Destroy {
            from: 15,
            amount: 1,
        },
    });
    admin.batch_execute("CREATE FUNCTION theia.reject_destroy_projection() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned destroy projection failure'; END $$; CREATE TRIGGER reject_destroy_projection BEFORE UPDATE ON theia.quest_journal_heads FOR EACH ROW EXECUTE FUNCTION theia.reject_destroy_projection();").await.unwrap();
    assert!(
        store
            .mutate_items_with_quest_journal(b, WORLD, &items, destroy, &q)
            .await
            .is_err()
    );
    admin.batch_execute("DROP TRIGGER reject_destroy_projection ON theia.quest_journal_heads; DROP FUNCTION theia.reject_destroy_projection();").await.unwrap();
    assert_eq!(
        store
            .load_inventory(b, WORLD, &items)
            .await
            .unwrap()
            .revision(),
        1
    );
    assert_eq!(
        store
            .load_quest_journal(b, WORLD, &q)
            .await
            .unwrap()
            .journal,
        ready.journal
    );
    let removed = store
        .mutate_items_with_quest_journal(b, WORLD, &items, destroy, &q)
        .await
        .unwrap();
    let state = removed
        .journal
        .as_ref()
        .unwrap()
        .journal
        .progress(&id("quest"))
        .unwrap();
    assert_eq!(state.status(), Status::Active);
    assert_eq!(state.count(&id("goal")), Some(2));
    assert_eq!(removed.bag.revision(), 2);
    assert!(removed.bag.slots()[15].is_none());
    let replay = store
        .mutate_items_with_quest_journal(b, WORLD, &items, destroy, &q)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag, removed.bag);
    let stale_turn = command(
        2,
        id("quest"),
        Action::TurnIn,
        ready.journal.progress(&id("quest")).unwrap().revision(),
    );
    assert!(
        store
            .command_quest_journal(b, WORLD, &q, &xp, &stale_turn)
            .await
            .is_err()
    );
    assert_eq!(
        store.load_progression(b, WORLD, &xp).await.unwrap().total(),
        0
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
