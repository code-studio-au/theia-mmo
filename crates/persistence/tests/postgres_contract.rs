//! Actual PostgreSQL only; an owned isolated cluster is required by the runner.
mod support;
use theia_core::coordinates::{GlobalPosition, MAX_GLOBAL_METRES};
use theia_persistence::{Binding, CharacterId, Error, OwnerId, RealmId, Saved, Store};
use tokio_postgres::error::SqlState;
use uuid::Uuid;
fn point(x: f64) -> GlobalPosition {
    GlobalPosition::new([x, -17.125, 8000.5]).unwrap()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn real_postgres_migrations_ownership_monotonicity_atomicity_and_sql_boundaries() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let binding = Binding {
        character: CharacterId::random(),
        owner: OwnerId::random(),
    };
    let other = Binding {
        character: CharacterId::random(),
        owner: OwnerId::random(),
    };
    let world = [31; 32];
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let issuer = "http://127.0.0.1:40123/realm-one";
    let account = store
        .resolve_owner(issuer, "case-Sensitive-subject")
        .await
        .unwrap();
    assert_eq!(
        store
            .resolve_owner(issuer, "case-Sensitive-subject")
            .await
            .unwrap(),
        account
    );
    assert_ne!(
        store
            .resolve_owner("http://127.0.0.1:40123/realm-two", "case-Sensitive-subject")
            .await
            .unwrap(),
        account
    );
    assert_ne!(
        store
            .resolve_owner(issuer, "case-sensitive-subject")
            .await
            .unwrap(),
        account
    );
    for (a, b) in [("", "subject"), (issuer, ""), (issuer, "bad\nsubject")] {
        assert!(store.resolve_owner(a, b).await.is_err());
    }
    assert!(
        store
            .resolve_owner(&"x".repeat(2049), "subject")
            .await
            .is_err()
    );
    assert!(store.resolve_owner(issuer, &"x".repeat(256)).await.is_err());
    let development = store.development_binding(1).await.unwrap();
    assert_eq!(store.development_binding(1).await.unwrap(), development);
    assert_ne!(store.development_binding(2).await.unwrap(), development);
    assert!(store.development_binding(0).await.is_err());
    assert!(store.development_binding(9).await.is_err());
    let initial = store.seed(binding, world, point(8060.)).await.unwrap();
    assert_eq!(initial.sequence, 0);
    assert!(matches!(
        Store::connect_loopback(&url, realm).await,
        Err(Error::RealmBusy)
    ));
    let mut another = Store::connect_loopback(&url, RealmId::random())
        .await
        .unwrap();
    assert_eq!(
        another
            .resolve_owner(issuer, "case-Sensitive-subject")
            .await
            .unwrap(),
        account
    );
    another.close().await.unwrap();
    assert_eq!(
        store.seed(binding, world, point(9999.)).await.unwrap(),
        initial
    );
    let wrong = Binding {
        owner: OwnerId::random(),
        ..binding
    };
    assert!(matches!(
        store.load(wrong, world).await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.seed(wrong, world, point(1.)).await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.load(binding, [32; 32]).await,
        Err(Error::WorldMismatch)
    ));
    let mut saved = initial;
    saved.sequence = 2;
    saved.position = point(8068.25);
    assert_eq!(
        store.checkpoint(&[saved]).await.unwrap(),
        vec![Saved::Committed]
    );
    let mut stale = saved;
    stale.sequence = 1;
    stale.position = point(1.);
    assert_eq!(
        store.checkpoint(&[stale]).await.unwrap(),
        vec![Saved::Stale]
    );
    assert_eq!(
        store.checkpoint(&[saved]).await.unwrap(),
        vec![Saved::Stale]
    );
    assert_eq!(store.load(binding, world).await.unwrap(), Some(saved));
    assert!(store.checkpoint(&[]).await.is_err());
    assert!(store.checkpoint(&[saved; 9]).await.is_err());
    assert!(store.checkpoint(&[saved, saved]).await.is_err());
    let mut invalid = saved;
    invalid.sequence = 0;
    assert!(store.checkpoint(&[invalid]).await.is_err());
    invalid.sequence = u64::MAX;
    assert!(store.checkpoint(&[invalid]).await.is_err());
    let second = store.seed(other, world, point(8070.)).await.unwrap();
    let mut changed = saved;
    changed.sequence = 3;
    changed.position = point(8080.);
    let mut forbidden = second;
    forbidden.sequence = 1;
    forbidden.binding.owner = OwnerId::random();
    assert!(matches!(
        store.checkpoint(&[changed, forbidden]).await,
        Err(Error::Ownership)
    ));
    assert_eq!(
        store.load(binding, world).await.unwrap(),
        Some(saved),
        "Batch must roll back its earlier update"
    );
    store.close().await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store
            .resolve_owner(issuer, "case-Sensitive-subject")
            .await
            .unwrap(),
        account
    );
    assert_eq!(store.load(binding, world).await.unwrap(), Some(saved));
    assert_eq!(
        store.checkpoint(&[stale]).await.unwrap(),
        vec![Saved::Stale],
        "Restart cannot authorize an older sequence"
    );
    assert_eq!(
        store.checkpoint(&[changed]).await.unwrap(),
        vec![Saved::Committed]
    );
    assert_eq!(store.load(binding, world).await.unwrap(), Some(changed));
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        MAX_GLOBAL_METRES + 1.,
    ] {
        let error = admin
            .execute(
                "UPDATE theia.characters SET x=$1 WHERE realm_id=$2 AND character_id=$3",
                &[&value, &realm.value(), &binding.character.value()],
            )
            .await
            .unwrap_err();
        assert_eq!(error.code(), Some(&SqlState::CHECK_VIOLATION));
    }
    let nil = Uuid::nil();
    let error = admin
        .execute(
            "UPDATE theia.characters SET owner_id=$1 WHERE realm_id=$2 AND character_id=$3",
            &[&nil, &realm.value(), &binding.character.value()],
        )
        .await
        .unwrap_err();
    assert_eq!(error.code(), Some(&SqlState::CHECK_VIOLATION));
    for bytes in [vec![1u8; 31], vec![0u8; 32]] {
        let error = admin
            .execute(
                "UPDATE theia.characters SET world_id=$1 WHERE realm_id=$2 AND character_id=$3",
                &[&bytes, &realm.value(), &binding.character.value()],
            )
            .await
            .unwrap_err();
        assert_eq!(error.code(), Some(&SqlState::CHECK_VIOLATION));
    }
    store.close().await.unwrap();
    admin.batch_execute("DROP TABLE theia.quest_kill_receipts; DROP TABLE theia.quest_objectives; DROP TABLE theia.experience_receipts; DROP TABLE theia.progression; DROP TABLE theia.reward_claims; DROP TABLE theia.backpacks; DROP TABLE theia.character_names; DROP TABLE theia.accounts; DROP TABLE theia.development_bindings; DELETE FROM theia.schema_migrations WHERE version>=2;").await.unwrap();
    let store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.load(binding, world).await.unwrap(),
        Some(changed),
        "Migration upgrade must retain character data"
    );
    let versions: i64 = admin
        .query_one("SELECT count(*) FROM theia.schema_migrations", &[])
        .await
        .unwrap()
        .get(0);
    assert_eq!(versions, 8);
    let upgraded_binding = store.development_binding(1).await.unwrap();
    store.close().await.unwrap();
    admin
        .batch_execute(
            "DROP TABLE theia.quest_kill_receipts; DROP TABLE theia.quest_objectives; DROP TABLE theia.experience_receipts; DROP TABLE theia.progression; DROP TABLE theia.reward_claims; DROP TABLE theia.backpacks; DROP TABLE theia.character_names; DROP TABLE theia.accounts; DELETE FROM theia.schema_migrations WHERE version>=3;",
        )
        .await
        .unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(
        store.development_binding(1).await.unwrap(),
        upgraded_binding
    );
    assert_eq!(store.load(binding, world).await.unwrap(), Some(changed));
    let _owner = store
        .resolve_owner(issuer, "after-version-two-upgrade")
        .await
        .unwrap();
    for (issuer, subject) in [("", "subject"), ("issuer", ""), ("issuer", "bad\nsubject")] {
        let error = admin
            .execute(
                "INSERT INTO theia.accounts(owner_id,issuer,subject) VALUES($1,$2,$3)",
                &[&Uuid::new_v4(), &issuer, &subject],
            )
            .await
            .unwrap_err();
        assert_eq!(error.code(), Some(&SqlState::CHECK_VIOLATION));
    }

    let original: Vec<u8> = admin
        .query_one(
            "SELECT checksum FROM theia.schema_migrations WHERE version=1",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    admin
        .execute(
            "UPDATE theia.schema_migrations SET checksum=$1 WHERE version=1",
            &[&vec![0u8; 32]],
        )
        .await
        .unwrap();
    assert!(matches!(
        Store::connect_loopback(&url, RealmId::random()).await,
        Err(Error::Migration)
    ));
    admin
        .execute(
            "UPDATE theia.schema_migrations SET checksum=$1 WHERE version=1",
            &[&original],
        )
        .await
        .unwrap();
    admin
        .execute(
            "INSERT INTO theia.schema_migrations(version,checksum) VALUES(99,$1)",
            &[&original],
        )
        .await
        .unwrap();
    assert!(matches!(
        Store::connect_loopback(&url, RealmId::random()).await,
        Err(Error::Migration)
    ));
    admin
        .execute("DELETE FROM theia.schema_migrations WHERE version=99", &[])
        .await
        .unwrap();
    let error = Store::connect_loopback(
        "host=203.0.113.1 password=never-log-this",
        RealmId::random(),
    )
    .await
    .err()
    .unwrap();
    assert!(matches!(error, Error::Invalid(_)));
    assert!(!format!("{error:?}").contains("never-log-this"));
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
