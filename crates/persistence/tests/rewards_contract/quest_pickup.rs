//! Actual conditional storage views/pickups; no actor range or network proof.
use super::quest_generation::{accept, cid, initialize, life, profiles};
use super::*;
use theia_core::quests::journal::Status;
use theia_persistence::{QuestJournalAction, QuestJournalCommand};
fn pelts(bag: &Bag) -> u32 {
    bag.slots()
        .iter()
        .flatten()
        .filter(|s| s.item() == &cid("theia.dev.wolf-pelt"))
        .map(Stack::count)
        .sum()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn needed_pages_precede_limit_and_pickup_rechecks_need_with_latest_receipt_replay() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, xp, journal, rewards) = profiles();
    let who = initialize(&mut store, "Questpage", 0, &items, &xp, &journal).await;
    assert_eq!(accept(&mut store, who, &journal, &xp).await, Status::Active);
    let win = cid("theia.test.pelt");
    let token = cid("theia.test.token");
    let center = GlobalPosition::new([8000., 0., 8000.]).unwrap();
    let once = store
        .generate_quest_loot_for_kill(
            who,
            WORLD,
            &rewards,
            &cid("theia.test.once"),
            &life(500),
            center,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .generate_quest_loot_for_kill(
                who,
                WORLD,
                &rewards,
                &cid("theia.test.once"),
                &life(501),
                center
            )
            .await
            .unwrap(),
        Some(once.clone())
    );
    let mut candidates = Vec::new();
    for n in 1..=20 {
        let position = GlobalPosition::new([8000. + n as f64 * 0.1, 0., 8000.]).unwrap();
        candidates.push(
            store
                .generate_quest_loot_for_kill(who, WORLD, &rewards, &win, &life(100 + n), position)
                .await
                .unwrap()
                .unwrap(),
        );
    }
    let initial = store
        .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
        .await
        .unwrap();
    assert_eq!(initial.len(), 17);
    assert_eq!(initial.iter().filter(|c| c.reward == win).count(), 16);
    assert!(initial.contains(&once));
    for (n, claim) in candidates.iter().take(3).enumerate() {
        let applied = store
            .pickup_quest_reward(who, WORLD, &rewards, claim.id, n as u64)
            .await
            .unwrap();
        assert_eq!(pelts(&applied.bag), n as u32 + 1);
        assert_eq!(
            applied
                .journal
                .unwrap()
                .journal
                .progress(&cid("quest"))
                .unwrap()
                .status(),
            if n == 2 {
                Status::Ready
            } else {
                Status::Active
            }
        );
    }
    let ordinary = store
        .generate_quest_loot_for_kill(
            who,
            WORLD,
            &rewards,
            &token,
            &life(1001),
            GlobalPosition::new([8020., 0., 8000.]).unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    let page = store
        .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
        .await
        .unwrap();
    assert_eq!(
        page,
        vec![once.clone(), ordinary.clone()],
        "Seventeen hidden nearer quest claims must not starve ordinary loot"
    );
    let hidden:i64=admin.query_one("SELECT count(*) FROM theia.reward_claims WHERE realm_id=$1 AND reward_id='theia.test.pelt' AND consumed_revision IS NULL",&[&realm.value()]).await.unwrap().get(0);
    assert_eq!(hidden, 17);
    assert!(matches!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, candidates[3].id, 3)
            .await,
        Err(Error::RewardNotNeeded)
    ));
    let replay = store
        .pickup_quest_reward(who, WORLD, &rewards, candidates[2].id, 2)
        .await
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.bag.revision(), 3);
    assert_eq!(pelts(&replay.bag), 3);
    assert_eq!(
        replay
            .journal
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .status(),
        Status::Ready
    );
    assert!(matches!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, candidates[0].id, 0)
            .await,
        Err(Error::InventoryStale)
    ));
    let removed = store
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
    assert_eq!(pelts(&removed.bag), 2);
    assert_eq!(
        removed
            .journal
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .status(),
        Status::Active
    );
    let page = store
        .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
        .await
        .unwrap();
    assert_eq!(page.len(), 17);
    assert_eq!(page.iter().filter(|c| c.reward == win).count(), 16);
    assert!(page.contains(&once));
    let target = candidates[3].corpse.unwrap();
    admin.execute("UPDATE theia.loot_rolls SET generation_eligible=false WHERE realm_id=$1 AND character_id=$2 AND roll_key='entry:theia.test.pelt' AND victim_life=$3",&[&realm.value(),&who.character.value(),&target.life.bytes().as_slice()]).await.unwrap();
    assert!(matches!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
            .await,
        Err(Error::RewardData)
    ));
    assert!(matches!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, candidates[3].id, 4)
            .await,
        Err(Error::RewardData)
    ));
    admin.execute("UPDATE theia.loot_rolls SET generation_eligible=true WHERE realm_id=$1 AND character_id=$2 AND roll_key='entry:theia.test.pelt' AND victim_life=$3",&[&realm.value(),&who.character.value(),&target.life.bytes().as_slice()]).await.unwrap();
    let replacement = store
        .pickup_quest_reward(who, WORLD, &rewards, candidates[3].id, 4)
        .await
        .unwrap();
    assert_eq!(pelts(&replacement.bag), 3);
    let current = replacement.journal.unwrap();
    assert_eq!(
        current.journal.progress(&cid("quest")).unwrap().status(),
        Status::Ready
    );
    assert_eq!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
            .await
            .unwrap(),
        vec![once.clone(), ordinary.clone()]
    );
    let completed = store
        .command_quest_journal(
            who,
            WORLD,
            &journal,
            &xp,
            &QuestJournalCommand {
                request_id: std::num::NonZeroU128::new(2).unwrap(),
                quest: cid("quest"),
                npc: cid("theia.dev.guide"),
                expected_revision: current.journal.progress(&cid("quest")).unwrap().revision(),
                action: QuestJournalAction::TurnIn,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        completed.journal.progress(&cid("quest")).unwrap().status(),
        Status::Completed
    );
    let bag = completed.inventory.unwrap();
    assert_eq!(pelts(&bag), 0);
    assert!(matches!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, candidates[4].id, bag.revision())
            .await,
        Err(Error::RewardNotNeeded)
    ));
    assert_eq!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
            .await
            .unwrap(),
        vec![once.clone(), ordinary.clone()]
    );
    // Unknown IDs still fail closed rather than disappearing in the SQL filter.
    let unknown = uuid::Uuid::new_v4();
    let unknown_life = life(2000).id.bytes();
    admin.execute("INSERT INTO theia.reward_claims(realm_id,claim_id,character_id,reward_id,world_id,profile_hash,claim_scope,victim_life,corpse_x,corpse_y,corpse_z) VALUES($1,$2,$3,'theia.test.unknown',$4,$5,1,$6,8000,0,8000)",&[&realm.value(),&unknown,&who.character.value(),&WORLD.as_slice(),&rewards.hash().as_slice(),&unknown_life.as_slice()]).await.unwrap();
    assert!(matches!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
            .await,
        Err(Error::RewardData)
    ));
    admin
        .execute(
            "DELETE FROM theia.reward_claims WHERE realm_id=$1 AND claim_id=$2",
            &[&realm.value(), &unknown],
        )
        .await
        .unwrap();
    assert!(
        store
            .list_reward_claims_nearby(who, WORLD, &rewards, center)
            .await
            .is_err()
    );
    let ordinary_taken = store
        .pickup_quest_reward(who, WORLD, &rewards, ordinary.id, bag.revision())
        .await
        .unwrap();
    assert_eq!(pelts(&ordinary_taken.bag), 0);
    let once_taken = store
        .pickup_quest_reward(who, WORLD, &rewards, once.id, ordinary_taken.bag.revision())
        .await
        .unwrap();
    assert_eq!(tokens(&once_taken.bag), 2);
    assert_eq!(
        once_taken
            .journal
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .status(),
        Status::Completed
    );
    let mut once_receipt = once.clone();
    once_receipt.consumed_revision = Some(once_taken.bag.revision());
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, center)
            .await
            .unwrap(),
        vec![once_receipt]
    );
    assert_eq!(
        store
            .load_progression(who, WORLD, &xp)
            .await
            .unwrap()
            .total(),
        25
    );
    store.close().await.unwrap();
    driver.abort();
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn full_conditional_pickup_preserves_claim_bag_and_journal_until_space_is_freed() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, xp, journal, rewards) = profiles();
    let who = character(&mut store, "Questcapacity").await;
    let slots =
        std::array::from_fn(|_| Stack::new(items.catalog(), cid("theia.dev.training-token"), 99));
    let initial = Bag::from_parts(items.catalog(), 0, slots, None).unwrap();
    store
        .ensure_inventory(who, WORLD, &items, &initial)
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, &xp).await.unwrap();
    store
        .ensure_quest_journal(who, WORLD, &journal)
        .await
        .unwrap();
    accept(&mut store, who, &journal, &xp).await;
    let position = GlobalPosition::new([8001., 0., 8000.]).unwrap();
    let claim = store
        .generate_quest_loot_for_kill(
            who,
            WORLD,
            &rewards,
            &cid("theia.test.pelt"),
            &life(3001),
            position,
        )
        .await
        .unwrap()
        .unwrap();
    let before = store
        .load_quest_journal(who, WORLD, &journal)
        .await
        .unwrap();
    assert!(matches!(
        store
            .pickup_quest_reward(who, WORLD, &rewards, claim.id, 0)
            .await,
        Err(Error::RewardCommand(theia_core::rewards::Error::NoCapacity))
    ));
    assert_eq!(
        store.load_inventory(who, WORLD, &items).await.unwrap(),
        initial
    );
    let after = store
        .load_quest_journal(who, WORLD, &journal)
        .await
        .unwrap();
    assert_eq!(before.revision, after.revision);
    assert_eq!(before.journal, after.journal);
    assert_eq!(
        store
            .list_quest_reward_claims_nearby(who, WORLD, &rewards, position)
            .await
            .unwrap(),
        vec![claim.clone()]
    );
    store
        .mutate_items_with_quest_journal(
            who,
            WORLD,
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy {
                    from: 15,
                    amount: 99,
                },
            }
            .into(),
            &journal,
        )
        .await
        .unwrap();
    let taken = store
        .pickup_quest_reward(who, WORLD, &rewards, claim.id, 1)
        .await
        .unwrap();
    assert_eq!(taken.bag.revision(), 2);
    assert_eq!(pelts(&taken.bag), 1);
    assert_eq!(
        taken
            .journal
            .unwrap()
            .journal
            .progress(&cid("quest"))
            .unwrap()
            .count(&cid("goal")),
        Some(1)
    );
    let consumed: Option<i64> = admin
        .query_one(
            "SELECT consumed_revision FROM theia.reward_claims WHERE realm_id=$1 AND claim_id=$2",
            &[&realm.value(), &uuid::Uuid::from_bytes(claim.id.bytes())],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(consumed, Some(2));
    store.close().await.unwrap();
    driver.abort();
}
