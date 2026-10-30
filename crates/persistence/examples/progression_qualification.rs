//! Owned clean database/process restart probe; not gameplay earned-credit proof.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{KillEvent, KillId},
};
use theia_persistence::{Binding, CharacterName, Error, ProgressionCatalog, RealmId, Store};
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
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000009").unwrap();
    let world = [123; 32];
    let profile = ProgressionCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/progression-restart-fixture",
            "synthetic-progression-subject",
        )
        .await?;
    let mut people = Vec::new();
    for (name, lives, base) in [("Experiencelevel", 4, 100), ("Experiencefirst", 1, 200)] {
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
        if mode == "write" {
            for id in base..base + lives {
                state = store
                    .award_experience(binding, world, &profile, &event(id))
                    .await?
                    .state;
            }
        }
        let before = state.clone();
        let old = store
            .award_experience(binding, world, &profile, &event(base))
            .await?;
        if !old.replayed
            || old.granted != 0
            || old.state != before
            || state.total() != lives as u32 * 25
            || state.revision() != lives
            || store.ensure_progression(binding, world, &profile).await? != state
        {
            return Err(Error::Invalid(
                "Progression restart value/receipt/reseed mismatch",
            ));
        }
        people.push(serde_json::json!({"name":name,"character":binding.character.value().to_string(),"revision":state.revision(),"progress":profile.profile().progress(&state).map_err(Error::ProgressionCommand)?}));
    }
    println!(
        "progression_probe {}",
        serde_json::json!({"owner":owner.value().to_string(),"profile":profile.hash(),"characters":people})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
