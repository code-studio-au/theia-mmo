//! Store-only restart probe. The runner owns PostgreSQL and its credential env.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::coordinates::GlobalPosition;
use theia_persistence::{Binding, CharacterId, Error, OwnerId, RealmId, Saved, Store};
use uuid::Uuid;
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::new(Uuid::from_u128(0x74686569610000000000000000000001)).unwrap();
    let mut binding = Binding {
        character: CharacterId::new(Uuid::from_u128(0x74686569610000000000000000000002)).unwrap(),
        owner: OwnerId::new(Uuid::from_u128(0x74686569610000000000000000000003)).unwrap(),
    };
    let world = [63; 32];
    let position = GlobalPosition::new([8068.25, -17.125, 8000.5]).unwrap();
    let mut store = Store::connect_loopback(&url, realm).await?;
    binding.owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/restart-fixture",
            "synthetic-restart-subject",
        )
        .await?;
    match mode.as_str() {
        "write" => {
            let mut record = store
                .seed(
                    binding,
                    world,
                    GlobalPosition::new([8060., -17., 8000.]).unwrap(),
                )
                .await?;
            record.sequence = 1;
            record.position = position;
            if store.checkpoint(&[record]).await? != [Saved::Committed] {
                return Err(Error::Invalid("Probe checkpoint not committed"));
            }
            println!("store_probe_committed=true");
        }
        "read" => {
            let record = store
                .load(binding, world)
                .await?
                .ok_or(Error::Invalid("Probe record missing"))?;
            if record.position != position || record.sequence != 1 {
                return Err(Error::Invalid("Probe restart state mismatch"));
            }
            println!("store_probe_reloaded=true");
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    }
    store.close().await?;
    drop(admin);
    driver
        .await
        .map_err(|_| Error::Invalid("Fixture driver stopped"))??;
    Ok(())
}
