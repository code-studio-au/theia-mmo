//! Owned clean database/process restart probe; not gameplay earned-credit proof.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
};
use theia_persistence::{
    Binding, CharacterName, Error, ProgressionCatalog, QuestCatalog, RealmId, Store,
};
fn event(entity: u64) -> KillEvent {
    KillEvent {
        id: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(u64::MAX).unwrap(),
            entity: EntityId::new(entity).unwrap(),
        }),
        creature: ContentId::new("theia.dev.training-creature").unwrap(),
    }
}
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-00000000000a").unwrap();
    let world = [133; 32];
    let profile = ProgressionCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))?;
    let quest = QuestCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/quest-restart-fixture",
            "synthetic-quest-subject",
        )
        .await?;
    let mut people = Vec::new();
    for (name, lives, base) in [("Questcomplete", 4, 100), ("Questactive", 1, 200)] {
        let binding = match mode.as_str() {
            "write" => {
                store
                    .create_character(
                        owner,
                        world,
                        CharacterName::parse(name)?,
                        GlobalPosition::new([8060., -17., 8000.]).unwrap(),
                    )
                    .await?
                    .saved
                    .binding
            }
            "read" => {
                let records = store.list_characters(owner, world).await?;
                let saved = records
                    .iter()
                    .find(|p| p.name.as_str() == name)
                    .ok_or(Error::Invalid("Missing progression restart character"))?;
                Binding {
                    owner,
                    character: saved.id,
                }
            }
            _ => return Err(Error::Invalid("Expected write or read")),
        };
        let mut state = store.ensure_progression(binding, world, &profile).await?;
        let mut objective = store.ensure_quest(binding, world, &quest).await?;
        if mode == "write" {
            for id in base..base + lives {
                let earned = store
                    .award_progression_and_quest(binding, world, &profile, &quest, &event(id))
                    .await?;
                state = earned.experience.state;
                objective = earned.quest.state;
            }
        }
        let before = state.clone();
        let old = store
            .award_progression_and_quest(binding, world, &profile, &quest, &event(base))
            .await?;
        if !old.experience.replayed
            || !old.quest.replayed
            || old.experience.granted != 0
            || old.experience.state != before
            || old.quest.state != objective
            || old.quest.advanced
            || old.quest.completed_now
            || objective.count() != lives.min(3) as u32
            || objective.revision() != lives.min(3)
            || store.ensure_quest(binding, world, &quest).await? != objective
            || state.total() != lives as u32 * 25
            || state.revision() != lives
            || store.ensure_progression(binding, world, &profile).await? != state
        {
            return Err(Error::Invalid(
                "Progression restart value/receipt/reseed mismatch",
            ));
        }
        people.push(serde_json::json!({"objective_revision":objective.revision(),"objective":quest.profile().progress(&objective).map_err(Error::QuestCommand)?,"name":name,"character":binding.character.value().to_string(),"revision":state.revision(),"progress":profile.profile().progress(&state).map_err(Error::ProgressionCommand)?}));
    }
    println!(
        "quest_probe {}",
        serde_json::json!({"owner":owner.value().to_string(),"profile":profile.hash(),"quest_profile":quest.hash(),"characters":people})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
