//! Actual SQL generation rights only; runtime paging/pickup remain closed.
use super::*;
use serde_json::json;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
    quests::journal::Status,
};
use theia_persistence::{
    ProgressionCatalog, QuestJournalAction, QuestJournalCatalog, QuestJournalCommand,
};
pub(super) fn cid(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
pub(super) fn life(n: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(707).unwrap(),
            entity: EntityId::new(n).unwrap(),
        }),
        creature: cid("theia.dev.training-creature"),
    }
}
pub(super) fn profiles() -> (
    InventoryCatalog,
    ProgressionCatalog,
    QuestJournalCatalog,
    RewardCatalog,
) {
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-loot-items.json"
    ))
    .unwrap();
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    let quests=serde_json::to_vec(&json!({"schema":3,"id":"catalog","revision":1,"quests":[{
        "id":"quest","version":1,"title":"Bring","description":"Test","prerequisites":[],
        "giver":"theia.dev.guide","turn_in":"theia.dev.guide","repeat_policy":"once","credit_scope":"character","experience":25,
        "start_stage":"stage","stages":[{"id":"stage","completion":{"kind":"all"},"next":null,"objectives":[{
            "kind":"possess_item","id":"goal","label":"Bring","item":"theia.dev.wolf-pelt","required":3,"consume_on_turn_in":true}]}]}]})).unwrap();
    let journal = QuestJournalCatalog::from_json(&quests, &xp)
        .unwrap()
        .with_inventory(&items)
        .unwrap();
    let source=serde_json::to_vec(&json!({"schema":4,"id":"theia.test.quest-loot","item_catalog":items.catalog().id().as_str(),"rewards":[
        {"id":"theia.test.pelt","name":"Pelt","scope":"per_creature_life","creature":"theia.dev.training-creature","grants":[{"item":"theia.dev.wolf-pelt","count":1}],"chance_basis_points":10000,"quest_required":true},
        {"id":"theia.test.miss","name":"Miss","scope":"per_creature_life","creature":"theia.dev.training-creature","grants":[{"item":"theia.dev.wolf-pelt","count":1}],"chance_basis_points":0,"quest_required":true},
        {"id":"theia.test.token","name":"Token","scope":"per_creature_life","creature":"theia.dev.training-creature","grants":[{"item":"theia.dev.training-token","count":1}],"chance_basis_points":10000},
        {"id":"theia.test.once","name":"Once","scope":"once_per_character","creature":"theia.dev.training-creature","grants":[{"item":"theia.dev.training-token","count":1}]}
    ]})).unwrap();
    let rewards = RewardCatalog::from_json_with_quest_journal(&source, &items, &journal).unwrap();
    (items, xp, journal, rewards)
}
pub(super) async fn initialize(
    store: &mut Store,
    name: &str,
    count: u32,
    items: &InventoryCatalog,
    xp: &ProgressionCatalog,
    journal: &QuestJournalCatalog,
) -> Binding {
    let who = character(store, name).await;
    let mut slots = std::array::from_fn(|_| None);
    if count > 0 {
        slots[0] = Stack::new(items.catalog(), cid("theia.dev.wolf-pelt"), count);
    }
    let bag = Bag::from_parts(items.catalog(), 0, slots, None).unwrap();
    store
        .ensure_inventory(who, WORLD, items, &bag)
        .await
        .unwrap();
    store.ensure_progression(who, WORLD, xp).await.unwrap();
    store
        .ensure_quest_journal(who, WORLD, journal)
        .await
        .unwrap();
    who
}
pub(super) async fn accept(
    store: &mut Store,
    who: Binding,
    journal: &QuestJournalCatalog,
    xp: &ProgressionCatalog,
) -> Status {
    store
        .command_quest_journal(
            who,
            WORLD,
            journal,
            xp,
            &QuestJournalCommand {
                request_id: std::num::NonZeroU128::new(1).unwrap(),
                quest: cid("quest"),
                npc: cid("theia.dev.guide"),
                expected_revision: 0,
                action: QuestJournalAction::Accept,
            },
        )
        .await
        .unwrap()
        .journal
        .progress(&cid("quest"))
        .unwrap()
        .status()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn first_generation_rights_survive_acceptance_removal_retry_and_reconnect_without_granting() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let (items, xp, journal, rewards) = profiles();
    let position = GlobalPosition::new([8008., -12., 8000.]).unwrap();
    let win = cid("theia.test.pelt");
    let miss = cid("theia.test.miss");
    let missing = character(&mut store, "Questneedmissing").await;
    store
        .ensure_inventory(missing, WORLD, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    assert!(matches!(
        store
            .generate_quest_loot_for_kill(missing, WORLD, &rewards, &win, &life(1), position)
            .await,
        Err(Error::QuestJournalMissing)
    ));
    let missing_rolls: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2",
            &[&realm.value(), &missing.character.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        missing_rolls, 0,
        "Missing journal must not freeze a false eligibility decision"
    );
    let empty = initialize(&mut store, "Questneedempty", 0, &items, &xp, &journal).await;
    assert!(
        store
            .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(1), position)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        accept(&mut store, empty, &journal, &xp).await,
        Status::Active
    );
    assert!(
        store
            .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(1), position)
            .await
            .unwrap()
            .is_none(),
        "Later acceptance cannot unlock an ineligible generation"
    );
    let trigger = format!("quest_generation_guard_{}", realm.value().simple());
    admin.batch_execute(&format!("CREATE FUNCTION theia.{trigger}() RETURNS TRIGGER AS $$ BEGIN IF NEW.realm_id='{}'::uuid AND NEW.reward_id='theia.test.pelt' THEN RAISE EXCEPTION 'owned generation rollback fixture'; END IF; RETURN NEW; END; $$ LANGUAGE plpgsql; CREATE TRIGGER {trigger} BEFORE INSERT ON theia.reward_claims FOR EACH ROW EXECUTE FUNCTION theia.{trigger}();",realm.value())).await.unwrap();
    assert!(
        store
            .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(2), position)
            .await
            .is_err()
    );
    let partial:i64=admin.query_one("SELECT count(*) FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2 AND victim_life=$3",&[&realm.value(),&empty.character.value(),&life(2).id.bytes().as_slice()]).await.unwrap().get(0);
    assert_eq!(
        partial, 0,
        "A failed winning claim must roll back its ticket and rights snapshot"
    );
    admin
        .batch_execute(&format!(
            "DROP TRIGGER {trigger} ON theia.reward_claims; DROP FUNCTION theia.{trigger}();"
        ))
        .await
        .unwrap();
    let won = store
        .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(2), position)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        store
            .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(2), position)
            .await
            .unwrap(),
        Some(won.clone())
    );
    for _ in 0..3 {
        assert!(
            store
                .generate_quest_loot_for_kill(empty, WORLD, &rewards, &miss, &life(3), position)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert!(
        store
            .load_inventory(empty, WORLD, &items)
            .await
            .unwrap()
            .slots()
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(
        store
            .load_quest_journal(empty, WORLD, &journal)
            .await
            .unwrap()
            .revision,
        1
    );
    assert!(
        store
            .claim_reward_with_quest_journal(empty, WORLD, &rewards, won.id, 0, &journal)
            .await
            .is_err(),
        "Generation alone must not open pickup admission"
    );
    assert!(
        store
            .generate_quest_loot_for_kill(
                empty,
                WORLD,
                &rewards,
                &win,
                &life(1),
                GlobalPosition::new([8009., -12., 8000.]).unwrap()
            )
            .await
            .is_err()
    );
    let full = initialize(&mut store, "Questneedfull", 3, &items, &xp, &journal).await;
    assert_eq!(accept(&mut store, full, &journal, &xp).await, Status::Ready);
    assert!(
        store
            .generate_quest_loot_for_kill(full, WORLD, &rewards, &win, &life(1), position)
            .await
            .unwrap()
            .is_none()
    );
    let removed = store
        .mutate_items_with_quest_journal(
            full,
            WORLD,
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Destroy { from: 0, amount: 1 },
            }
            .into(),
            &journal,
        )
        .await
        .unwrap();
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
    assert!(
        store
            .generate_quest_loot_for_kill(full, WORLD, &rewards, &win, &life(1), position)
            .await
            .unwrap()
            .is_none(),
        "Removal cannot create rights for an originally ineligible life"
    );
    let replacement = store
        .generate_quest_loot_for_kill(full, WORLD, &rewards, &win, &life(2), position)
        .await
        .unwrap()
        .unwrap();
    let rows=admin.query("SELECT generation_eligible,generation_bag_revision,generation_journal_revision FROM theia.loot_rolls WHERE realm_id=$1 ORDER BY character_id,roll_key,victim_life",&[&realm.value()]).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.iter().filter(|r| !r.get::<_, bool>(0)).count(), 2);
    let expected = [
        (empty, 1, 0, 0, false),
        (empty, 2, 0, 1, true),
        (full, 1, 0, 2, false),
        (full, 2, 1, 3, true),
    ];
    for (who, n, b, j, eligible) in expected {
        let row=admin.query_one("SELECT generation_eligible,generation_bag_revision,generation_journal_revision FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2 AND roll_key='entry:theia.test.pelt' AND victim_life=$3",&[&realm.value(),&who.character.value(),&life(n).id.bytes().as_slice()]).await.unwrap();
        assert_eq!(
            (
                row.get::<_, bool>(0),
                row.get::<_, Option<i64>>(1),
                row.get::<_, Option<i64>>(2)
            ),
            (eligible, Some(b), Some(j))
        );
    }
    let before:String=admin.query_one("SELECT md5(string_agg(to_jsonb(r)::text,'|' ORDER BY character_id,roll_key,victim_life)) FROM theia.loot_rolls r WHERE realm_id=$1",&[&realm.value()]).await.unwrap().get(0);
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert!(
        store
            .generate_quest_loot_for_kill(empty, WORLD, &rewards, &win, &life(1), position)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .generate_quest_loot_for_kill(full, WORLD, &rewards, &win, &life(2), position)
            .await
            .unwrap(),
        Some(replacement)
    );
    let after:String=admin.query_one("SELECT md5(string_agg(to_jsonb(r)::text,'|' ORDER BY character_id,roll_key,victim_life)) FROM theia.loot_rolls r WHERE realm_id=$1",&[&realm.value()]).await.unwrap().get(0);
    assert_eq!(
        before, after,
        "Replay changed ticket or original generation rights"
    );
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.reward_claims WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(count, 2);
    store.close().await.unwrap();
    driver.abort();
}
