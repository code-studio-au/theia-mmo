//! Owned PostgreSQL process/database restart probe; no network/appearance claim.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Command, EquipmentCommand, EquipmentOperation as Op, Operation, Stack},
};
use theia_persistence::{Binding, CharacterName, Error, InventoryCatalog, RealmId, Store};
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000007").unwrap();
    let world = [110; 32];
    let profile = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/equipment-restart-fixture",
            "synthetic-equipment-subject",
        )
        .await?;
    let last = EquipmentCommand {
        expected_revision: 2,
        operation: Op::Equip { from: 5 },
    };
    let (binding, bag) = match mode.as_str() {
        "write" => {
            let binding = store
                .create_character(
                    owner,
                    world,
                    CharacterName::parse("Equipment")?,
                    GlobalPosition::new([8060., -17., 8000.]).unwrap(),
                )
                .await?
                .saved
                .binding;
            let mut slots = std::array::from_fn(|_| None);
            for (slot, name, count) in [(0, "sword", 1), (1, "wand", 1), (2, "stone", 13)] {
                slots[slot] = Stack::new(
                    profile.catalog(),
                    ContentId::new(&format!("theia.dev.training-{name}")).unwrap(),
                    count,
                );
            }
            let seed =
                Bag::from_slots(profile.catalog(), 0, slots).map_err(|_| Error::InventoryData)?;
            store
                .ensure_inventory(binding, world, &profile, &seed)
                .await?;
            store
                .mutate_equipment(
                    binding,
                    world,
                    &profile,
                    EquipmentCommand {
                        expected_revision: 0,
                        operation: Op::Equip { from: 0 },
                    },
                )
                .await?;
            store
                .mutate_inventory(
                    binding,
                    world,
                    &profile,
                    Command {
                        expected_revision: 1,
                        operation: Operation::Move { from: 1, to: 5 },
                    },
                )
                .await?;
            (
                binding,
                store
                    .mutate_equipment(binding, world, &profile, last)
                    .await?
                    .bag,
            )
        }
        "read" => {
            let characters = store.list_characters(owner, world).await?;
            if characters.len() != 1 {
                return Err(Error::Invalid("Equipment restart character count mismatch"));
            }
            let binding = Binding {
                owner,
                character: characters[0].id,
            };
            let bag = store.load_inventory(binding, world, &profile).await?;
            let replay = store
                .mutate_equipment(binding, world, &profile, last)
                .await?;
            if !replay.replayed || replay.bag != bag {
                return Err(Error::Invalid("Equipment restart receipt mismatch"));
            }
            if store
                .ensure_inventory(binding, world, &profile, &Bag::empty(profile.catalog()))
                .await?
                != bag
            {
                return Err(Error::Invalid("Equipment was reseeded after restart"));
            }
            (binding, bag)
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    if bag.revision() != 3
        || bag.main_hand().map(|s| s.item().as_str()) != Some("theia.dev.training-wand")
        || bag.slots()[5].as_ref().map(|s| s.item().as_str()) != Some("theia.dev.training-sword")
        || bag.slots()[2].as_ref().map(Stack::count) != Some(13)
        || bag.slots().iter().flatten().count() != 2
    {
        return Err(Error::Invalid("Equipment restart contents mismatch"));
    }
    let checked: serde_json::Value =
        serde_json::from_slice(&bag.to_json()).map_err(|_| Error::InventoryData)?;
    println!(
        "equipment_probe {}",
        serde_json::json!({"owner":binding.owner.value().to_string(),"character":binding.character.value().to_string(),"catalog":profile.hash(),"bag":checked})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
