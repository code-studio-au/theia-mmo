//! Real bounded worker/SQL, deliberately dropped replies; no gameplay-owner proof.
use super::quest_generation::{cid, initialize, life, profiles};
use super::*;
use theia_core::quests::journal::Status;
use theia_persistence::{QuestJournalAction, QuestJournalCommand};
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn conditional_jobs_are_bounded_retain_queued_lost_replies_and_drain_before_shutdown() {
    let (url, mut admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, xp, journal, rewards) = profiles();
    let who = initialize(&mut store, "Questworker", 0, &items, &xp, &journal).await;
    let (publisher, worker) = Worker::start(store);
    let reward = cid("theia.test.pelt");
    let position = GlobalPosition::new([8002., 0., 8000.]).unwrap();
    // Unsupported old routes must reject before poisoning the shared SQL owner.
    assert!(
        publisher
            .roll_reward_for_kill(who, WORLD, &rewards, &reward, &life(4001), position)
            .await
            .is_err()
    );
    assert!(
        publisher
            .claim_reward_with_quest_journal(
                who,
                WORLD,
                &rewards,
                ClaimId::new([91; 16]).unwrap(),
                0,
                &journal
            )
            .await
            .is_err()
    );
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    drop(
        publisher
            .enqueue_quest_loot_generation(who, WORLD, &rewards, &reward, &life(4001), position)
            .unwrap(),
    );
    blocked_backend(&guard).await;
    let accepted = publisher
        .enqueue_quest_journal_command(
            who,
            WORLD,
            &journal,
            &xp,
            QuestJournalCommand {
                request_id: std::num::NonZeroU128::new(1).unwrap(),
                quest: cid("quest"),
                npc: cid("theia.dev.guide"),
                expected_revision: 0,
                action: QuestJournalAction::Accept,
            },
        )
        .unwrap();
    let mut queued = 0;
    let mut busy = 0;
    for _ in 0..32 {
        match publisher.enqueue_quest_loot_generation(
            who,
            WORLD,
            &rewards,
            &reward,
            &life(4001),
            position,
        ) {
            Ok(answer) => {
                queued += 1;
                drop(answer);
            }
            Err(Error::InventoryBusy) => busy += 1,
            Err(error) => panic!("Unexpected queue result: {error}"),
        }
    }
    assert_eq!(queued, 15);
    assert_eq!(
        busy, 17,
        "Critical FIFO remains bounded at sixteen pending requests"
    );
    let pending_publisher = publisher.clone();
    let pending_profile = rewards.clone();
    let pending_reward = reward.clone();
    let capacity_wait = tokio::spawn(async move {
        pending_publisher
            .generate_quest_loot_for_kill(
                who,
                WORLD,
                &pending_profile,
                &pending_reward,
                &life(4001),
                position,
            )
            .await
    });
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(
        !capacity_wait.is_finished(),
        "Async critical producer must wait for bounded queue capacity"
    );
    guard.commit().await.unwrap();
    assert_eq!(
        accepted
            .await
            .unwrap()
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .status(),
        Status::Active
    );
    assert!(capacity_wait.await.unwrap().unwrap().is_none());
    assert!(
        publisher
            .generate_quest_loot_for_kill(who, WORLD, &rewards, &reward, &life(4001), position)
            .await
            .unwrap()
            .is_none(),
        "Dropped preaccept generation must commit before the queued acceptance"
    );
    let claim = publisher
        .generate_quest_loot_for_kill(who, WORLD, &rewards, &reward, &life(4002), position)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        publisher
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, position)
            .await
            .unwrap(),
        vec![claim.clone()]
    );
    // Block a replay so the pickup is accepted but cannot start before its
    // answer is dropped and shutdown closes intake. It must still commit.
    let guard = admin.transaction().await.unwrap();
    guard
        .query_one(
            "SELECT revision FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",
            &[&realm.value(), &who.character.value()],
        )
        .await
        .unwrap();
    let blocker = publisher
        .enqueue_quest_loot_generation(who, WORLD, &rewards, &reward, &life(4002), position)
        .unwrap();
    blocked_backend(&guard).await;
    drop(
        publisher
            .enqueue_quest_loot_pickup(who, WORLD, &rewards, claim.id, 0)
            .unwrap(),
    );
    let stopping = tokio::spawn(worker.shutdown());
    tokio::task::yield_now().await;
    guard.commit().await.unwrap();
    assert_eq!(blocker.await.unwrap().unwrap(), Some(claim.clone()));
    let status = stopping.await.unwrap().unwrap();
    assert!(!status.failed);
    assert_eq!(status.reward_offers, queued + 5);
    assert_eq!(status.reward_committed, 1);
    assert!(
        publisher
            .enqueue_quest_loot_pickup(who, WORLD, &rewards, claim.id, 0)
            .is_err()
    );
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let bag = store.load_inventory(who, WORLD, &items).await.unwrap();
    assert_eq!(bag.revision(), 1);
    assert_eq!(
        store
            .load_quest_journal(who, WORLD, &journal)
            .await
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .count(&cid("goal")),
        Some(1)
    );
    assert!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, claim.id, 0)
            .await
            .unwrap()
            .replayed
    );
    let (publisher, worker) = Worker::start(store);
    let mut more = Vec::new();
    for n in 4003..=4005 {
        more.push(
            publisher
                .generate_quest_loot_for_kill(who, WORLD, &rewards, &reward, &life(n), position)
                .await
                .unwrap()
                .unwrap(),
        );
    }
    publisher
        .pickup_quest_reward(who, WORLD, &rewards, more[0].id, 1)
        .await
        .unwrap();
    let ready = publisher
        .pickup_quest_reward(who, WORLD, &rewards, more[1].id, 2)
        .await
        .unwrap();
    assert_eq!(
        ready
            .journal
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .status(),
        Status::Ready
    );
    assert!(matches!(
        publisher
            .pickup_quest_reward(who, WORLD, &rewards, more[2].id, 3)
            .await,
        Err(Error::RewardNotNeeded)
    ));
    assert!(
        publisher
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, position)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        publisher
            .pickup_quest_reward(who, WORLD, &rewards, more[1].id, 2)
            .await
            .unwrap()
            .replayed
    );
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert_eq!(status.reward_committed, 2);
    assert_eq!(status.reward_replayed, 1);
    assert_eq!(status.reward_rejected, 1);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &items,
            Command {
                expected_revision: 3,
                operation: Operation::Destroy { from: 0, amount: 1 },
            }
            .into(),
            &journal,
        )
        .await
        .unwrap();
    assert_eq!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, position)
            .await
            .unwrap(),
        vec![more[2].clone()]
    );
    let rows=admin.query("SELECT generation_eligible,generation_journal_revision FROM theia.loot_rolls WHERE realm_id=$1",&[&realm.value()]).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.iter().filter(|r| !r.get::<_, bool>(0)).count(), 1);
    store.close().await.unwrap();
    driver.abort();
}
