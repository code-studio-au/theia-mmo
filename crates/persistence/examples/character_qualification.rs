//! Named-character clean-restart probe against the owned fixture only.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::coordinates::GlobalPosition;
use theia_persistence::{CharacterName, Error, RealmId, Store, Worker};
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000005").unwrap();
    let world = [83; 32];
    let position = GlobalPosition::new([8123.5, -17.125, 8000.5]).unwrap();
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/named-restart-fixture",
            "synthetic-character-subject",
        )
        .await?;
    let character = match mode.as_str() {
        "write" => {
            let mut character = store
                .create_character(
                    owner,
                    world,
                    CharacterName::parse("Restarted")?,
                    GlobalPosition::new([8060., -17., 8000.]).unwrap(),
                )
                .await?;
            character.saved.sequence = 9;
            character.saved.position = position;
            let (publisher, worker) = Worker::start(store);
            publisher.publish(&[character.saved])?;
            let status = worker.shutdown().await?;
            if status.failed || status.committed_records != 1 {
                return Err(Error::Invalid("Named character checkpoint not committed"));
            }
            character
        }
        "read" => {
            let entries = store.list_characters(owner, world).await?;
            if entries.len() != 1 {
                return Err(Error::Invalid("Named restart character list mismatch"));
            }
            let character = store
                .select_character(
                    theia_persistence::Binding {
                        owner,
                        character: entries[0].id,
                    },
                    world,
                )
                .await?;
            if character.name.as_str() != "Restarted"
                || character.saved.position != position
                || character.saved.sequence != 9
            {
                return Err(Error::Invalid("Named character restart state mismatch"));
            }
            store.close().await?;
            character
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    println!(
        "character_probe {}",
        serde_json::json!({"owner":owner.value().to_string(),"character":character.saved.binding.character.value().to_string(),"name":character.name.as_str(),"position":character.saved.position.metres(),"sequence":character.saved.sequence})
    );
    drop(admin);
    driver
        .await
        .map_err(|_| Error::Invalid("Owned fixture driver stopped"))??;
    Ok(())
}
