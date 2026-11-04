//! Owned SQL; source/recipient and pickup entitlement are trusted fixtures.
use super::*;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
    quests::journal::Status,
};
use theia_persistence::{
    LIFE_CLAIM_PAGE_SIZE, ProgressionCatalog, QuestJournalAction, QuestJournalCatalog,
    QuestJournalCommand,
};
fn loot_id() -> ContentId {
    ContentId::new("theia.dev.repeat-training-kill").unwrap()
}
fn loot_profiles() -> (InventoryCatalog, RewardCatalog) {
    let items = InventoryCatalog::from_json(ITEMS).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    value["schema"] = serde_json::json!(2);
    value["rewards"][0]["id"] = serde_json::json!(loot_id().as_str());
    value["rewards"][0]["scope"] = serde_json::json!("per_creature_life");
    value["rewards"][0]["grants"] =
        serde_json::json!([{"item":"theia.dev.training-stone","count":1}]);
    let rewards = RewardCatalog::from_json(&serde_json::to_vec(&value).unwrap(), &items).unwrap();
    (items, rewards)
}
fn kill(n: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(43).unwrap(),
            entity: EntityId::new(n).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
fn position() -> GlobalPosition {
    GlobalPosition::new([8000., 0., 8000.]).unwrap()
}
fn stones(bag: &Bag) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &id("stone"))
        .map(Stack::count)
        .sum()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn death_creates_only_availability_and_life_replays_never_grant_twice() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, rewards) = loot_profiles();
    let who = character(&mut store, "Lootlife").await;
    let other = character(&mut store, "Lootother").await;
    let empty = Bag::empty(items.catalog());
    for b in [who, other] {
        store
            .ensure_inventory(b, WORLD, &items, &empty)
            .await
            .unwrap();
    }
    assert!(
        store
            .offer_reward(who, WORLD, &rewards, &loot_id())
            .await
            .is_err()
    );
    let first = store
        .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(1), position())
        .await
        .unwrap();
    assert_eq!(first.corpse.unwrap().life, kill(1).id);
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        empty
    );
    assert_eq!(
        store
            .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(1), position())
            .await
            .unwrap(),
        first
    );
    assert!(matches!(
        store
            .offer_reward_for_kill(
                who,
                WORLD,
                &rewards,
                &loot_id(),
                &kill(1),
                GlobalPosition::new([8001., 0., 8000.]).unwrap()
            )
            .await,
        Err(Error::RewardData)
    ));
    let mut wrong = kill(1);
    wrong.creature = ContentId::new("theia.dev.other-creature").unwrap();
    assert!(matches!(
        store
            .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &wrong, position())
            .await,
        Err(Error::RewardCommand(rewards::Error::CreatureMismatch))
    ));
    assert!(
        store
            .claim_reward(other, WORLD, &rewards, first.id, 0)
            .await
            .is_err()
    );
    let own_other = store
        .offer_reward_for_kill(other, WORLD, &rewards, &loot_id(), &kill(1), position())
        .await
        .unwrap();
    assert_ne!(own_other.id, first.id);
    let taken = store
        .claim_reward(who, WORLD, &rewards, first.id, 0)
        .await
        .unwrap();
    assert_eq!(stones(&taken.bag), 1);
    assert!(
        store
            .claim_reward(who, WORLD, &rewards, first.id, 0)
            .await
            .unwrap()
            .replayed
    );
    let replay = store
        .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(1), position())
        .await
        .unwrap();
    assert_eq!(replay.id, first.id);
    assert_eq!(replay.consumed_revision, Some(1));
    assert!(
        store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap()
            .is_empty()
    );
    let second = store
        .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(2), position())
        .await
        .unwrap();
    assert_ne!(second.id, first.id);
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap(),
        vec![second.clone()]
    );
    assert_eq!(
        stones(&store.load_inventory(who, WORLD, &items).await.unwrap()),
        1
    );
    let next = store
        .claim_reward(who, WORLD, &rewards, second.id, 1)
        .await
        .unwrap();
    assert_eq!(stones(&next.bag), 2);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn pending_loot_is_paged_without_losing_unseen_or_consumed_life_receipts() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, rewards) = loot_profiles();
    let who = character(&mut store, "Lootpages").await;
    store
        .ensure_inventory(who, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    let mut offered = Vec::new();
    for n in 1..=(LIFE_CLAIM_PAGE_SIZE as u64 + 3) {
        offered.push(
            store
                .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(n), position())
                .await
                .unwrap(),
        );
    }
    let page = store
        .list_reward_claims(who, WORLD, &rewards)
        .await
        .unwrap();
    assert_eq!(page.len(), LIFE_CLAIM_PAGE_SIZE);
    let mut collected = Vec::new();
    loop {
        let page = store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap();
        assert!(page.len() <= LIFE_CLAIM_PAGE_SIZE);
        let Some(claim) = page.first() else { break };
        assert!(!collected.contains(&claim.id));
        store
            .claim_reward(who, WORLD, &rewards, claim.id, collected.len() as u64)
            .await
            .unwrap();
        collected.push(claim.id);
    }
    assert_eq!(collected.len(), offered.len());
    assert_eq!(
        stones(&store.load_inventory(who, WORLD, &items).await.unwrap()),
        offered.len() as u32
    );
    for (n, first) in offered.iter().enumerate() {
        let replay = store
            .offer_reward_for_kill(
                who,
                WORLD,
                &rewards,
                &loot_id(),
                &kill(n as u64 + 1),
                position(),
            )
            .await
            .unwrap();
        assert_eq!(replay.id, first.id);
        assert!(replay.consumed_revision.is_some());
    }
    assert!(
        store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap()
            .is_empty()
    );
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.reward_claims WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, offered.len() as i64);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
fn quest(items: &InventoryCatalog) -> (ProgressionCatalog, QuestJournalCatalog) {
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let bytes=serde_json::to_vec(&serde_json::json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Bring","description":"Test","prerequisites":[],"giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.training-stone","required":3,"consume_on_turn_in":true}]}]}]})).unwrap();
    let q = QuestJournalCatalog::from_json(&bytes, &xp)
        .unwrap()
        .with_inventory(items)
        .unwrap();
    (xp, q)
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn only_explicit_pickup_changes_possession_and_pickup_failure_rolls_back_both() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, rewards) = loot_profiles();
    let (xp, q) = quest(&items);
    let who = character(&mut store, "Lootquest").await;
    store
        .ensure_inventory(who, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, &xp).await.unwrap();
    store.ensure_quest_journal(who, WORLD, &q).await.unwrap();
    let qid = ContentId::new("quest").unwrap();
    store
        .command_quest_journal(
            who,
            WORLD,
            &q,
            &xp,
            &QuestJournalCommand {
                request_id: std::num::NonZeroU128::new(1).unwrap(),
                quest: qid.clone(),
                npc: ContentId::new("theia.dev.guide").unwrap(),
                expected_revision: 0,
                action: QuestJournalAction::Accept,
            },
        )
        .await
        .unwrap();
    for n in 1..=3 {
        let before = store.load_quest_journal(who, WORLD, &q).await.unwrap();
        let bag = store.load_inventory(who, WORLD, &items).await.unwrap();
        let claim = store
            .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(n), position())
            .await
            .unwrap();
        assert_eq!(store.load_inventory(who, WORLD, &items).await.unwrap(), bag);
        assert_eq!(
            store
                .load_quest_journal(who, WORLD, &q)
                .await
                .unwrap()
                .journal,
            before.journal
        );
        if n == 3 {
            admin.batch_execute("CREATE FUNCTION theia.reject_loot_journal() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned pickup journal failure'; END $$; CREATE TRIGGER reject_loot_journal BEFORE UPDATE ON theia.quest_journal_heads FOR EACH ROW EXECUTE FUNCTION theia.reject_loot_journal();").await.unwrap();
            assert!(
                store
                    .claim_reward_with_quest_journal(
                        who,
                        WORLD,
                        &rewards,
                        claim.id,
                        bag.revision(),
                        &q
                    )
                    .await
                    .is_err()
            );
            admin.batch_execute("DROP TRIGGER reject_loot_journal ON theia.quest_journal_heads; DROP FUNCTION theia.reject_loot_journal();").await.unwrap();
            assert_eq!(store.load_inventory(who, WORLD, &items).await.unwrap(), bag);
            assert!(
                store
                    .list_reward_claims(who, WORLD, &rewards)
                    .await
                    .unwrap()
                    .iter()
                    .any(|c| c.id == claim.id)
            );
        }
        let picked = store
            .claim_reward_with_quest_journal(who, WORLD, &rewards, claim.id, bag.revision(), &q)
            .await
            .unwrap();
        assert_eq!(stones(&picked.bag), n as u32);
        assert_eq!(
            picked
                .journal
                .as_ref()
                .unwrap()
                .journal
                .progress(&qid)
                .unwrap()
                .status(),
            if n == 3 {
                Status::Ready
            } else {
                Status::Active
            }
        );
    }
    let bag = store.load_inventory(who, WORLD, &items).await.unwrap();
    let removed = store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &items,
            Command {
                expected_revision: bag.revision(),
                operation: Operation::Destroy { from: 0, amount: 1 },
            }
            .into(),
            &q,
        )
        .await
        .unwrap();
    assert_eq!(
        removed
            .journal
            .as_ref()
            .unwrap()
            .journal
            .progress(&qid)
            .unwrap()
            .status(),
        Status::Active
    );
    let claim = store
        .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(4), position())
        .await
        .unwrap();
    assert_eq!(
        store
            .load_quest_journal(who, WORLD, &q)
            .await
            .unwrap()
            .journal
            .progress(&qid)
            .unwrap()
            .status(),
        Status::Active
    );
    let recovered = store
        .claim_reward_with_quest_journal(who, WORLD, &rewards, claim.id, removed.bag.revision(), &q)
        .await
        .unwrap();
    assert_eq!(
        recovered
            .journal
            .as_ref()
            .unwrap()
            .journal
            .progress(&qid)
            .unwrap()
            .status(),
        Status::Ready
    );
    assert_eq!(stones(&recovered.bag), 3);
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn accepted_loot_offers_drain_after_response_loss_and_full_bags_leave_loot_uncollected() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, rewards) = loot_profiles();
    let who = character(&mut store, "Lootretained").await;
    let full = Bag::from_slots(
        items.catalog(),
        0,
        std::array::from_fn(|_| Stack::new(items.catalog(), id("token"), 99)),
    )
    .unwrap();
    store
        .ensure_inventory(who, WORLD, &items, &full)
        .await
        .unwrap();
    let (publisher, worker) = Worker::start(store);
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let p = publisher.clone();
    let profile = rewards.clone();
    let first = tokio::spawn(async move {
        p.offer_reward_for_kill(who, WORLD, &profile, &loot_id(), &kill(1), position())
            .await
    });
    blocked_backend(&guard).await;
    first.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    let event = kill(2);
    let recipe = loot_id();
    let mut second = Box::pin(publisher.offer_reward_for_kill(
        who,
        WORLD,
        &rewards,
        &recipe,
        &event,
        position(),
    ));
    assert!(
        tokio::time::timeout(Duration::from_millis(50), second.as_mut())
            .await
            .is_err()
    );
    drop(second); // Accepted but still queued behind the deliberately blocked first offer.
    drop(publisher);
    let draining = tokio::spawn(worker.shutdown());
    guard.commit().await.unwrap();
    let status = draining.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.reward_offers, 2);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let offered = store
        .list_reward_claims(who, WORLD, &rewards)
        .await
        .unwrap();
    assert_eq!(offered.len(), 2);
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        full
    );
    assert!(matches!(
        store
            .claim_reward(who, WORLD, &rewards, offered[0].id, 0)
            .await,
        Err(Error::RewardCommand(rewards::Error::NoCapacity))
    ));
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap(),
        offered
    );
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        full
    );
    let room = store
        .mutate_inventory(
            who,
            WORLD,
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy {
                    from: 0,
                    amount: 99,
                },
            },
        )
        .await
        .unwrap();
    let picked = store
        .claim_reward(who, WORLD, &rewards, offered[0].id, room.bag.revision())
        .await
        .unwrap();
    assert_eq!(stones(&picked.bag), 1);
    assert!(
        store
            .claim_reward(who, WORLD, &rewards, offered[0].id, room.bag.revision())
            .await
            .unwrap()
            .replayed
    );
    assert_eq!(
        store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap()
            .len(),
        1
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}

#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn nearby_loot_follows_trusted_pose_and_distant_history_cannot_hide_fresh_drop() {
    let (url, _admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let items = InventoryCatalog::from_json(ITEMS).unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(DATA).unwrap();
    raw["schema"] = 2.into();
    let mut life = raw["rewards"][0].clone();
    life["id"] = loot_id().as_str().into();
    life["scope"] = "per_creature_life".into();
    raw["rewards"].as_array_mut().unwrap().push(life);
    let rewards = RewardCatalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items).unwrap();
    let who = character(&mut store, "Lootnear").await;
    store
        .ensure_inventory(who, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    let once_id = rewards
        .catalog()
        .definitions()
        .find(|d| d.scope() == theia_core::rewards::Scope::OncePerCharacter)
        .unwrap()
        .id()
        .clone();
    let once = store
        .offer_reward(who, WORLD, &rewards, &once_id)
        .await
        .unwrap();
    let far = position();
    let near = GlobalPosition::new([9000., 0., 9000.]).unwrap();
    for n in 1..=34 {
        let p = if n <= 17 {
            far
        } else {
            GlobalPosition::new([9020., 0., 9000.]).unwrap()
        };
        store
            .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(n), p)
            .await
            .unwrap();
    }
    let fresh = store
        .offer_reward_for_kill(who, WORLD, &rewards, &loot_id(), &kill(35), near)
        .await
        .unwrap();
    let boundary = GlobalPosition::new([10000., 0., 10000.]).unwrap();
    let mut boundary_claim = None;
    for (n, p) in [
        (36, [10000., 64., 10000.]),
        (37, [10000., 64.001, 10000.]),
        (38, [10064., 64., 10000.]),
    ] {
        let offered = store
            .offer_reward_for_kill(
                who,
                WORLD,
                &rewards,
                &loot_id(),
                &kill(n),
                GlobalPosition::new(p).unwrap(),
            )
            .await
            .unwrap();
        if n == 36 {
            boundary_claim = Some(offered.id);
        }
    }
    // The old global storage page contains only older distant drops.
    assert!(
        !store
            .list_reward_claims(who, WORLD, &rewards)
            .await
            .unwrap()
            .iter()
            .any(|c| c.id == fresh.id)
    );
    let nearby = store
        .list_reward_claims_nearby(who, WORLD, &rewards, near)
        .await
        .unwrap();
    assert_eq!(nearby.len(), LIFE_CLAIM_PAGE_SIZE + 1);
    assert!(nearby.iter().any(|c| c.id == fresh.id));
    assert!(nearby.iter().any(|c| c.id == once.id));
    assert!(
        nearby
            .iter()
            .filter_map(|c| c.corpse)
            .all(|c| c.position.metres()[0] >= 9000.)
    );
    let edge = store
        .list_reward_claims_nearby(who, WORLD, &rewards, boundary)
        .await
        .unwrap();
    assert_eq!(edge.len(), 2);
    assert!(edge.iter().any(|c| Some(c.id) == boundary_claim));
    assert!(edge.iter().any(|c| c.id == once.id));
    let (publisher, worker) = Worker::start(store);
    assert_eq!(
        publisher
            .list_reward_claims_nearby(who, WORLD, &rewards, near)
            .await
            .unwrap(),
        nearby
    );
    publisher
        .claim_reward(who, WORLD, &rewards, fresh.id, 0)
        .await
        .unwrap();
    let after = publisher
        .list_reward_claims_nearby(who, WORLD, &rewards, near)
        .await
        .unwrap();
    assert_eq!(after.len(), LIFE_CLAIM_PAGE_SIZE + 1);
    assert!(!after.iter().any(|c| c.id == fresh.id));
    let restored_far = publisher
        .list_reward_claims_nearby(who, WORLD, &rewards, far)
        .await
        .unwrap();
    assert_eq!(restored_far.len(), LIFE_CLAIM_PAGE_SIZE + 1);
    assert!(
        restored_far
            .iter()
            .filter_map(|c| c.corpse)
            .all(|c| c.position == far)
    );
    assert_eq!(
        publisher
            .load_inventory(who, WORLD, &items)
            .await
            .unwrap()
            .revision(),
        1
    );
    worker.shutdown().await.unwrap();
    drop(_admin);
    driver.await.unwrap().unwrap();
}
