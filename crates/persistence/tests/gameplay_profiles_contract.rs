mod support;
use theia_core::{coordinates::GlobalPosition, inventory::Bag};
use theia_persistence::{
    Binding, CharacterName, Error, GameplayProfiles, InventoryCatalog, ProfileCompatibility as C,
    ProgressionCatalog, QuestCatalog, QuestLifecycleCatalog, RealmId, Store, Worker,
};
const WORLD: [u8; 32] = [151; 32];
const QUEST: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
const XP: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
const RECIPE: &[u8] = include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json");
async fn character(store: &mut Store, name: &str) -> Binding {
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/gameplay-profiles", name)
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
fn lifecycle(xp: &ProgressionCatalog) -> (QuestLifecycleCatalog, QuestLifecycleCatalog) {
    (
        QuestLifecycleCatalog::from_json(QUEST).unwrap(),
        QuestLifecycleCatalog::from_json_with_turn_in_xp(QUEST, RECIPE, xp).unwrap(),
    )
}
#[test]
fn admission_profiles_reject_zero_and_conflicting_quest_modes() {
    assert!(GameplayProfiles::new(Some([0; 32]), None, None, None).is_err());
    assert!(GameplayProfiles::new(None, Some([0; 32]), None, None).is_err());
    assert!(GameplayProfiles::new(None, None, Some([1; 32]), Some([2; 32])).is_err());
    assert!(GameplayProfiles::new(None, None, None, None).is_ok());
}
#[tokio::test]
#[ignore = "requires owned THEIA_TEST_DATABASE_URL/THEIA_TEST_DATABASE_OWNED=1"]
async fn saved_profiles_are_checked_without_initialization_or_migration() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000091").unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Profileone").await;
    let b = character(&mut store, "Profiletwo").await;
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let (journal, rewarded) = lifecycle(&xp);
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let enabled = GameplayProfiles::new(
        Some(items.hash()),
        Some(xp.hash()),
        None,
        Some(rewarded.hash()),
    )
    .unwrap();
    assert_eq!(
        store
            .check_gameplay_profiles(a, WORLD, enabled)
            .await
            .unwrap(),
        C::Compatible
    );
    let count:i64=admin.query_one("SELECT (SELECT count(*) FROM theia.backpacks WHERE realm_id=$1)+(SELECT count(*) FROM theia.progression WHERE realm_id=$1)+(SELECT count(*) FROM theia.quest_lifecycles WHERE realm_id=$1)",&[&realm.value()]).await.unwrap().get(0);
    assert_eq!(count, 0, "preflight must not initialize missing state");
    store
        .ensure_inventory(a, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    store.ensure_progression(a, WORLD, &xp).await.unwrap();
    store
        .ensure_quest_lifecycle(a, WORLD, &journal)
        .await
        .unwrap();
    let before = admin
        .query(
            "SELECT row_to_json(q)::text FROM theia.quest_lifecycles q WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()[0]
        .get::<_, String>(0);
    assert_eq!(
        store
            .check_gameplay_profiles(a, WORLD, enabled)
            .await
            .unwrap(),
        C::Incompatible
    );
    let matching = GameplayProfiles::new(
        Some(items.hash()),
        Some(xp.hash()),
        None,
        Some(journal.hash()),
    )
    .unwrap();
    assert_eq!(
        store
            .check_gameplay_profiles(a, WORLD, matching)
            .await
            .unwrap(),
        C::Compatible
    );
    assert_eq!(
        store
            .check_gameplay_profiles(a, WORLD, GameplayProfiles::default())
            .await
            .unwrap(),
        C::Compatible
    );
    for profiles in [
        GameplayProfiles::new(Some([9; 32]), None, None, None).unwrap(),
        GameplayProfiles::new(None, Some([9; 32]), None, None).unwrap(),
    ] {
        assert_eq!(
            store
                .check_gameplay_profiles(a, WORLD, profiles)
                .await
                .unwrap(),
            C::Incompatible
        );
    }
    let objective = QuestCatalog::from_json(QUEST).unwrap();
    store.ensure_quest(a, WORLD, &objective).await.unwrap();
    assert_eq!(
        store
            .check_gameplay_profiles(
                a,
                WORLD,
                GameplayProfiles::new(None, None, Some(objective.hash()), None).unwrap()
            )
            .await
            .unwrap(),
        C::Compatible
    );
    assert_eq!(
        store
            .check_gameplay_profiles(
                a,
                WORLD,
                GameplayProfiles::new(None, None, Some([9; 32]), None).unwrap()
            )
            .await
            .unwrap(),
        C::Incompatible
    );
    assert!(matches!(
        store
            .check_gameplay_profiles(
                Binding {
                    owner: b.owner,
                    ..a
                },
                WORLD,
                enabled
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.check_gameplay_profiles(a, [152; 32], enabled).await,
        Err(Error::WorldMismatch)
    ));
    let after = admin
        .query(
            "SELECT row_to_json(q)::text FROM theia.quest_lifecycles q WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()[0]
        .get::<_, String>(0);
    assert_eq!(before, after);
    let receipts: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(receipts, 0);
    drop(store);
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned THEIA_TEST_DATABASE_URL/THEIA_TEST_DATABASE_OWNED=1"]
async fn expected_incompatible_character_keeps_worker_available() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000092").unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Profileworker").await;
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let (journal, rewarded) = lifecycle(&xp);
    store.ensure_progression(a, WORLD, &xp).await.unwrap();
    store
        .ensure_quest_lifecycle(a, WORLD, &journal)
        .await
        .unwrap();
    let changed =
        GameplayProfiles::new(None, Some(xp.hash()), None, Some(rewarded.hash())).unwrap();
    let (publisher, worker) = Worker::start(store);
    assert_eq!(
        publisher
            .check_gameplay_profiles(a, WORLD, changed)
            .await
            .unwrap(),
        C::Incompatible
    );
    assert!(publisher.select_character(a, WORLD).await.is_ok());
    assert_eq!(
        publisher
            .check_gameplay_profiles(a, WORLD, GameplayProfiles::default())
            .await
            .unwrap(),
        C::Compatible
    );
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert_eq!(status.gameplay_profile_checks, 2);
    assert_eq!(status.incompatible_gameplay_profiles, 1);
    drop(publisher);
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned THEIA_TEST_DATABASE_URL/THEIA_TEST_DATABASE_OWNED=1"]
async fn read_only_preflight_survives_cancelled_answer_and_bounded_flood() {
    use std::time::Duration;
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000093").unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let a = character(&mut store, "Profilecancel").await;
    let xp = ProgressionCatalog::from_json(XP).unwrap();
    let (journal, rewarded) = lifecycle(&xp);
    store
        .ensure_quest_lifecycle(a, WORLD, &journal)
        .await
        .unwrap();
    let changed = GameplayProfiles::new(None, None, None, Some(rewarded.hash())).unwrap();
    let (publisher, worker) = Worker::start(store);
    admin
        .batch_execute("BEGIN; LOCK TABLE theia.quest_lifecycles IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();
    let p = publisher.clone();
    let first = tokio::spawn(async move { p.check_gameplay_profiles(a, WORLD, changed).await });
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let waiting: bool = admin.query_one("SELECT EXISTS(SELECT 1 FROM pg_stat_activity WHERE application_name='theia-development-persistence' AND wait_event_type='Lock' AND query LIKE 'SELECT 0::int AS role%')", &[]).await.unwrap().get(0);
            if waiting {break}
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.expect("preflight must be executing before answer cancellation");
    first.abort();
    let _ = first.await;
    let mut flood = tokio::task::JoinSet::new();
    for _ in 0..40 {
        let p = publisher.clone();
        flood.spawn(async move { p.check_gameplay_profiles(a, WORLD, changed).await });
    }
    // Let all producers hit the bounded queue while the one SQL reader waits.
    tokio::time::sleep(Duration::from_millis(30)).await;
    admin.batch_execute("COMMIT").await.unwrap();
    let (mut accepted, mut refused) = (0, 0);
    while let Some(result) = flood.join_next().await {
        match result.unwrap() {
            Ok(C::Incompatible) => accepted += 1,
            Err(Error::Invalid(_)) => refused += 1,
            other => panic!("Unexpected flood outcome: {other:?}"),
        }
    }
    assert_eq!(accepted, 16);
    assert_eq!(refused, 24);
    assert_eq!(
        publisher
            .check_gameplay_profiles(a, WORLD, GameplayProfiles::default())
            .await
            .unwrap(),
        C::Compatible
    );
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert_eq!(status.gameplay_profile_checks, 18);
    assert_eq!(status.incompatible_gameplay_profiles, 17);
    let state: Vec<u8> = admin
        .query_one(
            "SELECT profile_hash FROM theia.quest_lifecycles WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(state, journal.hash());
    drop(publisher);
    drop(admin);
    driver.await.unwrap().unwrap();
}
