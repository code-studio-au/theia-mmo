//! Actual PostgreSQL account-owned character APIs; no avatars/native admission.
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
};
use theia_persistence::{
    Binding, CharacterName, Error, MAX_CHARACTERS_PER_OWNER, OwnerId, ProgressionCatalog, RealmId,
    Store, Worker,
};
fn name(value: &str) -> CharacterName {
    CharacterName::parse(value).unwrap()
}
fn spawn() -> GlobalPosition {
    GlobalPosition::new([8060.25, -17.125, 8000.5]).unwrap()
}
#[tokio::test]
#[ignore = "requires owned PostgreSQL; run tools/qualify_persistence.py"]
async fn owned_characters_names_quotas_selection_and_rejections_are_transactional() {
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::random();
    let world = [81; 32];
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let issuer = format!("http://127.0.0.1:40123/characters/{}", realm.value());
    let alice = store.resolve_owner(&issuer, "alice").await.unwrap();
    let bob = store.resolve_owner(&issuer, "bob").await.unwrap();
    assert!(matches!(
        store
            .create_character(OwnerId::random(), world, name("Unknown"), spawn())
            .await,
        Err(Error::AccountMissing)
    ));
    let appearance = theia_core::appearance::Appearance::new([0, 2, 4, 3, 2, 1, 1]).unwrap();
    let arden = store
        .create_character_with_appearance(alice, world, name("Arden"), spawn(), appearance)
        .await
        .unwrap();
    assert_eq!(arden.appearance, appearance);
    assert_eq!(
        store.list_characters(alice, world).await.unwrap()[0].appearance,
        appearance
    );
    for invalid in [vec![0_u8; 6], vec![255; 7], vec![1, 0, 1, 1, 0, 0, 0]] {
        let error = admin
            .execute(
                "UPDATE theia.characters SET appearance=$1 WHERE realm_id=$2 AND character_id=$3",
                &[
                    &invalid,
                    &realm.value(),
                    &arden.saved.binding.character.value(),
                ],
            )
            .await
            .unwrap_err();
        assert_eq!(
            error.code(),
            Some(&tokio_postgres::error::SqlState::CHECK_VIOLATION)
        );
    }
    assert_eq!(arden.saved.position, spawn());
    assert_eq!(arden.saved.sequence, 0);
    assert_eq!(
        store
            .select_character(arden.saved.binding, world)
            .await
            .unwrap(),
        arden
    );
    assert_eq!(store.list_characters(alice, world).await.unwrap().len(), 1);
    assert!(store.list_characters(bob, world).await.unwrap().is_empty());
    let data = include_bytes!("../../../content/gameplay/development-progression.json");
    let catalog = ProgressionCatalog::from_json(data).unwrap();
    let roster = store
        .list_characters_for_identity(&issuer, "alice", world, Some(&catalog))
        .await
        .unwrap();
    assert_eq!(roster.len(), 1);
    assert_eq!(roster[0].name.as_str(), "Arden");
    assert!(roster[0].last_saved_unix > 0);
    let initial = roster[0].progression.unwrap();
    assert_eq!((initial.level, initial.total), (1, 0));
    assert!(matches!(
        store
            .load_progression(arden.saved.binding, world, &catalog)
            .await,
        Err(Error::ProgressionMissing)
    ));
    store
        .ensure_progression(arden.saved.binding, world, &catalog)
        .await
        .unwrap();
    for entity in 1..=4 {
        let event = KillEvent {
            id: KillId::from_victim(EntityKey {
                epoch: WorldEpoch::new(1).unwrap(),
                entity: EntityId::new(entity).unwrap(),
            }),
            creature: ContentId::new("theia.dev.training-creature").unwrap(),
        };
        store
            .award_experience(arden.saved.binding, world, &catalog, &event)
            .await
            .unwrap();
    }
    let roster = store
        .list_characters_for_identity(&issuer, "alice", world, Some(&catalog))
        .await
        .unwrap();
    let saved = roster[0].progression.unwrap();
    assert_eq!((saved.level, saved.total), (2, 100));
    let unconfigured = store
        .list_characters_for_identity(&issuer, "alice", world, None)
        .await
        .unwrap();
    assert_eq!(unconfigured[0].progression, None);
    for subject in ["bob", "new-website-user"] {
        assert!(
            store
                .list_characters_for_identity(&issuer, subject, world, Some(&catalog))
                .await
                .unwrap()
                .is_empty()
        );
    }
    assert!(
        admin
            .query_opt(
                "SELECT owner_id FROM theia.accounts WHERE issuer=$1 AND subject=$2",
                &[&issuer, &"new-website-user"]
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        store
            .select_character(
                Binding {
                    owner: bob,
                    ..arden.saved.binding
                },
                world
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        store.select_character(arden.saved.binding, [82; 32]).await,
        Err(Error::WorldMismatch)
    ));
    assert!(matches!(
        store.list_characters(alice, [82; 32]).await,
        Err(Error::WorldMismatch)
    ));
    let other_realm = RealmId::random();
    let mut other = Store::connect_loopback(&url, other_realm).await.unwrap();
    assert!(
        other
            .list_characters(alice, world)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        other.select_character(arden.saved.binding, world).await,
        Err(Error::CharacterMissing)
    ));
    other
        .create_character(alice, world, name("ARDEN"), spawn())
        .await
        .unwrap();
    other.close().await.unwrap();
    for bad in [
        "Ab",
        "Ar--den",
        "Arden-",
        "Árden",
        "Ar den",
        "Abcdefghijklmnopqrstu",
    ] {
        let error=admin.execute("UPDATE theia.character_names SET display_name=$1 WHERE realm_id=$2 AND character_id=$3", &[&bad, &realm.value(), &arden.saved.binding.character.value()]).await.unwrap_err();
        assert_eq!(
            error.code(),
            Some(&tokio_postgres::error::SqlState::CHECK_VIOLATION)
        );
    }
    let (publisher, worker) = Worker::start(store);
    let (a, b) = tokio::join!(
        publisher.create_character(alice, world, name("Rival"), spawn()),
        publisher.create_character(bob, world, name("rIvAl"), spawn())
    );
    assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
    assert!(matches!(
        (&a, &b),
        (Ok(_), Err(Error::NameUnavailable)) | (Err(Error::NameUnavailable), Ok(_))
    ));
    assert!(matches!(
        publisher
            .create_character(bob, world, name("ARDEN"), spawn())
            .await,
        Err(Error::NameUnavailable)
    ));
    let count: i64 = admin
        .query_one(
            "SELECT count(*) FROM theia.characters WHERE realm_id=$1",
            &[&realm.value()],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(
        count, 2,
        "Rejected names must roll back the earlier character row"
    );
    let current = publisher.list_characters(alice, world).await.unwrap().len();
    for i in 0..MAX_CHARACTERS_PER_OWNER - current {
        let text = format!("Adventurer{}", char::from(b'A' + i as u8));
        publisher
            .create_character(alice, world, name(&text), spawn())
            .await
            .unwrap();
    }
    assert!(matches!(
        publisher
            .create_character(alice, world, name("Overflow"), spawn())
            .await,
        Err(Error::CharacterLimit)
    ));
    let selene = publisher
        .create_character(bob, world, name("Selene"), spawn())
        .await
        .unwrap();
    publisher
        .create_character(bob, world, name("Ar'den"), spawn())
        .await
        .unwrap();
    publisher
        .create_character(bob, world, name("Ar-den"), spawn())
        .await
        .unwrap();
    assert!(matches!(
        publisher
            .select_character(
                Binding {
                    owner: alice,
                    ..selene.saved.binding
                },
                world
            )
            .await,
        Err(Error::Ownership)
    ));
    assert!(matches!(
        publisher.list_characters(alice, [82; 32]).await,
        Err(Error::WorldMismatch)
    ));
    assert!(publisher.list_characters(alice, [0; 32]).await.is_err());
    assert_eq!(
        publisher.resolve_owner(&issuer, "alice").await.unwrap(),
        alice
    );
    assert!(
        !worker.status().failed,
        "Expected user rejections cannot kill healthy persistence"
    );
    assert_eq!(
        publisher.list_characters(alice, world).await.unwrap().len(),
        MAX_CHARACTERS_PER_OWNER
    );
    let mut changed = arden.saved;
    changed.sequence = 9;
    changed.position = GlobalPosition::new([8123.5, -17.125, 8000.5]).unwrap();
    publisher.publish(&[changed]).unwrap();
    let status = worker.shutdown().await.unwrap();
    assert!(!status.failed);
    assert!(status.rejected_account_requests >= 5);
    assert!(status.created_characters >= 8);
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    let selected = store
        .select_character(arden.saved.binding, world)
        .await
        .unwrap();
    assert_eq!(selected.saved, changed);
    assert_eq!(selected.name.as_str(), "Arden");
    assert_eq!(store.resolve_owner(&issuer, "alice").await.unwrap(), alice);
    // Upgrade a genuine version-3 shape while preserving accounts, development
    // bindings and unnamed character rows. Names did not exist in version 3.
    store.close().await.unwrap();
    admin.batch_execute("ALTER TABLE theia.characters DROP COLUMN appearance; DROP TABLE theia.quest_journal_actions; DROP TABLE theia.quest_journal_events; DROP TABLE theia.quest_journal_entries; DROP TABLE theia.quest_journal_heads; DROP TABLE theia.quest_lifecycle_kills; DROP TABLE theia.quest_action_receipts; DROP TABLE theia.quest_lifecycles; DROP TABLE theia.quest_kill_receipts; DROP TABLE theia.quest_objectives; DROP TABLE theia.experience_receipts; DROP TABLE theia.progression; DROP TABLE theia.reward_claims; DROP TABLE theia.backpacks; DROP TABLE theia.character_names; DROP TABLE theia.loot_rolls; DELETE FROM theia.schema_migrations WHERE version>=4;").await.unwrap();
    let mut store = Store::connect_loopback(&url, realm).await.unwrap();
    assert_eq!(store.resolve_owner(&issuer, "alice").await.unwrap(), alice);
    assert_eq!(
        store.load(arden.saved.binding, world).await.unwrap(),
        Some(changed)
    );
    store.close().await.unwrap();
    drop(admin);
    driver.await.unwrap().unwrap();
}
