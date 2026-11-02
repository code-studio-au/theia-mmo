//! Owned clean process/database restart; synthetic facts, no NPC/gameplay proof.
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
            epoch: WorldEpoch::new(73).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
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
    let realm = RealmId::parse("74686569-6100-0000-0000-00000000000c").unwrap();
    let world = [143; 32];
    let mut store = Store::connect_loopback(&url, realm).await?;
    let q = QuestLifecycleCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))?;
    let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    data["level_thresholds"] = serde_json::json!([0, 25]);
    let xp = ProgressionCatalog::from_json(&serde_json::to_vec(&data).unwrap())?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/lifecycle-restart-fixture",
            "synthetic-lifecycle-subject",
        )
        .await?;
    let accept = Command {
        expected_revision: 0,
        action: Action::Accept,
    };
    let turn_in = Command {
        expected_revision: 4,
        action: Action::TurnIn,
    };
    let mut people = Vec::new();
    for (name, phase, base, count) in [
        ("Lifeavailable", Phase::Available, 100, 0),
        ("Lifeactive", Phase::Active, 200, 1),
        ("Lifecompleted", Phase::TurnedIn, 300, 3),
    ] {
        let binding = if mode == "write" {
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
            let records = store.list_characters(owner, world).await?;
            let saved = records
                .iter()
                .find(|p| p.name.as_str() == name)
                .ok_or(Error::Invalid("Missing lifecycle restart character"))?;
            Binding {
                owner,
                character: saved.id,
            }
        };
        store.ensure_progression(binding, world, &xp).await?;
        store.ensure_quest_lifecycle(binding, world, &q).await?;
        if mode == "write" {
            store
                .award_progression_and_quest_lifecycle(binding, world, &xp, &q, &event(base))
                .await?;
            if phase == Phase::Available {
                store
                    .award_progression_and_quest_lifecycle(
                        binding,
                        world,
                        &xp,
                        &q,
                        &event(base + 1),
                    )
                    .await?;
            } else {
                store
                    .command_quest_lifecycle(binding, world, &q, accept)
                    .await?;
                for id in 1..=count {
                    store
                        .award_progression_and_quest_lifecycle(
                            binding,
                            world,
                            &xp,
                            &q,
                            &event(base + u64::from(id)),
                        )
                        .await?;
                }
                if phase == Phase::TurnedIn {
                    store
                        .command_quest_lifecycle(binding, world, &q, turn_in)
                        .await?;
                }
            }
        }
        let state = store.load_quest_lifecycle(binding, world, &q).await?;
        if state
            .progress(q.profile())
            .map_err(Error::QuestLifecycleCommand)?
            .phase
            != phase
            || state.count() != count
        {
            return Err(Error::Invalid("Lifecycle restart phase/count mismatch"));
        }
        for id in [
            base,
            if phase == Phase::Available {
                base + 1
            } else {
                base + u64::from(count)
            },
        ] {
            let replay = store
                .award_progression_and_quest_lifecycle(binding, world, &xp, &q, &event(id))
                .await?;
            if !replay.quest.replayed
                || replay.quest.changed
                || replay.quest.state != state
                || replay.experience.granted != 0
            {
                return Err(Error::Invalid("Lifecycle restart kill receipt mismatch"));
            }
        }
        for command in if phase == Phase::TurnedIn {
            vec![accept, turn_in]
        } else if phase == Phase::Active {
            vec![accept]
        } else {
            vec![]
        } {
            let replay = store
                .command_quest_lifecycle(binding, world, &q, command)
                .await?;
            if !replay.replayed || replay.changed || replay.turned_in_now || replay.state != state {
                return Err(Error::Invalid("Lifecycle restart action receipt mismatch"));
            }
        }
        let experience = store.load_progression(binding, world, &xp).await?;
        if experience.total() != 25 || experience.revision() != 1 {
            return Err(Error::Invalid("Lifecycle restart XP mismatch"));
        }
        let receipts=admin.query_one("SELECT (SELECT count(*) FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2),(SELECT count(*) FROM theia.quest_lifecycle_kills WHERE realm_id=$1 AND character_id=$2)",&[&realm.value(),&binding.character.value()]).await.unwrap();
        people.push(serde_json::json!({"name":name,"character":binding.character.value().to_string(),"progress":state.progress(q.profile()).map_err(Error::QuestLifecycleCommand)?,"experience":experience.total(),"xp_revision":experience.revision(),"action_receipts":receipts.get::<_,i64>(0),"kill_receipts":receipts.get::<_,i64>(1)}));
    }
    println!(
        "quest_lifecycle_probe {}",
        serde_json::json!({"owner":owner.value().to_string(),"profile_hash":q.hash(),"xp_profile_hash":xp.hash(),"characters":people})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
