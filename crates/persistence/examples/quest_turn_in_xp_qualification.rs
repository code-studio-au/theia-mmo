//! Actual clean process/database restart; trusted facts, no native/NPC evidence.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
    quests::lifecycle::{Action, Command, Phase},
};
use theia_persistence::{
    Binding, CharacterName, Error, ProgressionCatalog, QuestLifecycleCatalog, RealmId, Store,
};
fn event(id: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(83).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
fn command(revision: u64, action: Action) -> Command {
    Command {
        expected_revision: revision,
        action,
    }
}
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    if !matches!(mode.as_str(), "write" | "read") {
        return Err(Error::Invalid("Expected write or read"));
    }
    let (url, admin, driver) = support::owned().await;
    let realm = RealmId::parse("74686569-6100-0000-0000-00000000000d").unwrap();
    let world = [147; 32];
    let mut store = Store::connect_loopback(&url, realm).await?;
    let xp = ProgressionCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))?;
    let q = QuestLifecycleCatalog::from_json_with_turn_in_xp(
        include_bytes!("../../../content/gameplay/development-quest.json"),
        include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json"),
        &xp,
    )?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/turn-in-xp-restart-fixture",
            "synthetic-completion-xp-subject",
        )
        .await?;
    let mut people = Vec::new();
    for (name, phase, base, total, revision) in [
        ("Rewardavailable", Phase::Available, 100, 0, 0),
        ("Rewardready", Phase::Ready, 200, 75, 3),
        ("Rewardcomplete", Phase::TurnedIn, 300, 125, 5),
    ] {
        let a = if mode == "write" {
            store
                .create_character(
                    owner,
                    world,
                    CharacterName::parse(name)?,
                    GlobalPosition::new([8000., 0., 8000.]).unwrap(),
                )
                .await?
                .saved
                .binding
        } else {
            let id = store
                .list_characters(owner, world)
                .await?
                .into_iter()
                .find(|c| c.name.as_str() == name)
                .ok_or(Error::CharacterMissing)?
                .id;
            Binding {
                owner,
                character: id,
            }
        };
        store.ensure_progression(a, world, &xp).await?;
        store.ensure_quest_lifecycle(a, world, &q).await?;
        if mode == "write" && phase != Phase::Available {
            store
                .command_quest_lifecycle_with_experience(
                    a,
                    world,
                    &xp,
                    &q,
                    command(0, Action::Accept),
                )
                .await?;
            for id in base..base + 3 {
                store
                    .award_progression_and_quest_lifecycle(a, world, &xp, &q, &event(id))
                    .await?;
            }
            if phase == Phase::TurnedIn {
                let done = store
                    .command_quest_lifecycle_with_experience(
                        a,
                        world,
                        &xp,
                        &q,
                        command(4, Action::TurnIn),
                    )
                    .await?;
                assert_eq!(done.experience.granted, 25);
                assert_eq!(done.experience.state.total(), 100);
                store
                    .award_progression_and_quest_lifecycle(a, world, &xp, &q, &event(base + 3))
                    .await?;
            }
        }
        if phase != Phase::Available {
            let replay = store
                .command_quest_lifecycle_with_experience(
                    a,
                    world,
                    &xp,
                    &q,
                    command(0, Action::Accept),
                )
                .await?;
            assert!(replay.quest.replayed);
            assert_eq!(replay.experience.granted, 0);
            if phase == Phase::TurnedIn {
                let replay = store
                    .command_quest_lifecycle_with_experience(
                        a,
                        world,
                        &xp,
                        &q,
                        command(4, Action::TurnIn),
                    )
                    .await?;
                assert!(replay.quest.replayed);
                assert_eq!(replay.experience.granted, 0);
                assert_eq!(replay.experience.state.total(), 125);
            }
        }
        let state = store.load_quest_lifecycle(a, world, &q).await?;
        let experience = store.load_progression(a, world, &xp).await?;
        assert_eq!(state.progress(q.profile()).unwrap().phase, phase);
        assert_eq!(
            (experience.total(), experience.revision()),
            (total, revision)
        );
        let rows=admin.query("SELECT expected_revision,action,encode(command_hash,'hex'),encode(xp_profile_hash,'hex'),xp_granted,xp_revision,xp_total FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 ORDER BY expected_revision",&[&realm.value(),&a.character.value()]).await?;
        let receipts:Vec<_>=rows.into_iter().map(|r|serde_json::json!({"expected_revision":r.get::<_,i64>(0),"action":r.get::<_,i16>(1),"command_hash":r.get::<_,String>(2),"xp_profile_hash":r.get::<_,String>(3),"granted":r.get::<_,i32>(4),"xp_revision":r.get::<_,i64>(5),"xp_total":r.get::<_,i32>(6)})).collect();
        assert_eq!(
            receipts.len(),
            match phase {
                Phase::Available => 0,
                Phase::Ready => 1,
                Phase::TurnedIn => 2,
                _ => unreachable!(),
            }
        );
        if phase == Phase::TurnedIn {
            assert_eq!(receipts[1]["granted"], 25);
            assert_eq!(receipts[1]["xp_total"], 100);
            assert_eq!(receipts[1]["xp_revision"], 4);
        }
        people.push(serde_json::json!({"name":name,"character":a.character.value().to_string(),"phase":phase,"quest_revision":state.revision(),"count":state.count(),"xp_total":experience.total(),"xp_revision":experience.revision(),"receipts":receipts}));
    }
    println!(
        "quest_turn_in_xp_probe {}",
        serde_json::json!({"quest_hash":q.hash(),"xp_hash":xp.hash(),"people":people})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
