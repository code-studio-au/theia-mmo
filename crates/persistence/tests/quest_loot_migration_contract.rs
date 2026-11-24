//! Owned, serialized pre-loot phase: upgrade an ordinary version-13 roll in place.
mod support;
use serde_json::json;
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::Bag,
    progression::{KillEvent, KillId},
};
use theia_persistence::{CharacterName, InventoryCatalog, RealmId, RewardCatalog, Store};
#[tokio::test]
#[ignore = "requires fresh owned PostgreSQL before conditional generations; run tools/qualify_persistence.py"]
async fn version_thirteen_roll_and_claim_upgrade_without_changing_their_result() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let world = [142; 32];
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let owner = store
        .resolve_owner("http://127.0.0.1:40123/upgrade", "ordinary")
        .await
        .unwrap();
    let binding = store
        .create_character(
            owner,
            world,
            CharacterName::parse("Upgradeordinary").unwrap(),
            GlobalPosition::new([8000., 0., 8000.]).unwrap(),
        )
        .await
        .unwrap()
        .saved
        .binding;
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let mut source: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    source["schema"] = json!(3);
    source["rewards"][0]["scope"] = json!("per_creature_life");
    let profile = RewardCatalog::from_json(&serde_json::to_vec(&source).unwrap(), &items).unwrap();
    let reward = profile.catalog().definitions().next().unwrap().id().clone();
    store
        .ensure_inventory(binding, world, &items, &Bag::empty(items.catalog()))
        .await
        .unwrap();
    let event = KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(708).unwrap(),
            entity: EntityId::new(1).unwrap(),
        }),
        creature: profile
            .catalog()
            .definitions()
            .next()
            .unwrap()
            .creature()
            .clone(),
    };
    let position = GlobalPosition::new([8008., 0., 8000.]).unwrap();
    let before = store
        .roll_reward_for_kill(binding, world, &profile, &reward, &event, position)
        .await
        .unwrap()
        .unwrap();
    store.close().await.unwrap();
    // Never discard conditional audit metadata, even in the disposable fixture.
    let conditional: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.loot_rolls WHERE generation_bag_revision IS NOT NULL",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(conditional, 0);
    let ticket: i32 = admin
        .query_one(
            "SELECT ticket FROM theia.loot_rolls WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    admin.batch_execute("ALTER TABLE theia.loot_rolls DROP CONSTRAINT loot_generation_snapshot, DROP COLUMN generation_eligible, DROP COLUMN generation_bag_revision, DROP COLUMN generation_journal_revision; DELETE FROM theia.schema_migrations WHERE version=14;").await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let row=admin.query_one("SELECT ticket,generation_eligible,generation_bag_revision,generation_journal_revision FROM theia.loot_rolls WHERE realm_id=$1",&[&realm.value()]).await.unwrap();
    assert_eq!(row.get::<_, i32>(0), ticket);
    assert!(row.get::<_, bool>(1));
    assert_eq!(row.get::<_, Option<i64>>(2), None);
    assert_eq!(row.get::<_, Option<i64>>(3), None);
    assert_eq!(
        store
            .roll_reward_for_kill(binding, world, &profile, &reward, &event, position)
            .await
            .unwrap(),
        Some(before)
    );
    for assignment in [
        "generation_eligible=false",
        "generation_bag_revision=0",
        "generation_journal_revision=0",
        "generation_bag_revision=-1,generation_journal_revision=0",
    ] {
        let error = admin
            .execute(
                &format!("UPDATE theia.loot_rolls SET {assignment} WHERE realm_id=$1"),
                &[&realm.value()],
            )
            .await
            .unwrap_err();
        assert_eq!(
            error.code(),
            Some(&tokio_postgres::error::SqlState::CHECK_VIOLATION)
        );
    }
    store.close().await.unwrap();
    driver.abort();
}
