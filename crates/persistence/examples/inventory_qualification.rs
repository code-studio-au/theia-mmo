//! Owned PostgreSQL clean-restart inventory probe; no native/OIDC claim.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, Operation, Stack},
};
use theia_persistence::{Binding, CharacterName, Error, InventoryCatalog, RealmId, Store};
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000006").unwrap();
    let world = [94; 32];
    let profile = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/backpack-restart-fixture",
            "synthetic-backpack-subject",
        )
        .await?;
    let last = Command {
        expected_revision: 1,
        operation: Operation::Split {
            from: 1,
            to: 3,
            amount: 7,
        },
    };
    let (binding, bag) = match mode.as_str() {
        "write" => {
            let character = store
                .create_character(
                    owner,
                    world,
                    CharacterName::parse("Backpack")?,
                    GlobalPosition::new([8060., -17., 8000.]).unwrap(),
                )
                .await?;
            let binding = character.saved.binding;
            let mut slots = std::array::from_fn(|_| None);
            slots[0] = Stack::new(
                profile.catalog(),
                ContentId::new("theia.dev.training-stone").unwrap(),
                13,
            );
            slots[1] = Stack::new(
                profile.catalog(),
                ContentId::new("theia.dev.training-stone").unwrap(),
                15,
            );
            let seed =
                Bag::from_slots(profile.catalog(), 0, slots).map_err(|_| Error::InventoryData)?;
            store
                .ensure_inventory(binding, world, &profile, &seed)
                .await?;
            store
                .mutate_inventory(
                    binding,
                    world,
                    &profile,
                    Command {
                        expected_revision: 0,
                        operation: Operation::Move { from: 0, to: 1 },
                    },
                )
                .await?;
            (
                binding,
                store
                    .mutate_inventory(binding, world, &profile, last)
                    .await?
                    .bag,
            )
        }
        "read" => {
            let entries = store.list_characters(owner, world).await?;
            if entries.len() != 1 {
                return Err(Error::Invalid("Backpack restart character count mismatch"));
            }
            let binding = Binding {
                owner,
                character: entries[0].id,
            };
            let bag = store.load_inventory(binding, world, &profile).await?;
            let replay = store
                .mutate_inventory(binding, world, &profile, last)
                .await?;
            if !replay.replayed || replay.bag != bag {
                return Err(Error::Invalid("Backpack restart receipt mismatch"));
            }
            let ensured = store
                .ensure_inventory(binding, world, &profile, &Bag::empty(profile.catalog()))
                .await?;
            if ensured != bag {
                return Err(Error::Invalid("Backpack was reseeded after restart"));
            }
            (binding, bag)
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    if bag.revision() != 2
        || bag.slots()[0].as_ref().map(Stack::count) != Some(8)
        || bag.slots()[1].as_ref().map(Stack::count) != Some(13)
        || bag.slots()[3].as_ref().map(Stack::count) != Some(7)
    {
        return Err(Error::Invalid("Backpack restart contents mismatch"));
    }
    let checked: serde_json::Value =
        serde_json::from_slice(&bag.to_json()).map_err(|_| Error::InventoryData)?;
    println!(
        "inventory_probe {}",
        serde_json::json!({"owner":binding.owner.value().to_string(),"character":binding.character.value().to_string(),"catalog":profile.hash(),"bag":checked})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
