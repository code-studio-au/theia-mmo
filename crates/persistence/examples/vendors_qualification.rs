//! Actual clean database/process restart probe; no NPC, OIDC or native claim.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Stack},
    vendors::Purchase,
};
use theia_persistence::{
    Binding, CharacterName, Error, InventoryCatalog, RealmId, Store, VendorCatalog,
};
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000015").unwrap();
    let world = [143; 32];
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))?;
    let vendor = VendorCatalog::from_json(
        include_bytes!("../../../content/gameplay/development-vendor.json"),
        &items,
    )?;
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/vendor-restart-fixture",
            "synthetic-trader-subject",
        )
        .await?;
    let purchase = |expected_revision| Purchase {
        expected_revision,
        offer: ContentId::new("theia.dev.training-vendor.stones").unwrap(),
    };
    let (binding, bag) = match mode.as_str() {
        "write" => {
            let character = store
                .create_character(
                    owner,
                    world,
                    CharacterName::parse("Trader")?,
                    GlobalPosition::new([15400., -17., 12860.]).unwrap(),
                )
                .await?;
            let who = character.saved.binding;
            let mut slots = std::array::from_fn(|_| None);
            slots[0] = Stack::new(
                items.catalog(),
                ContentId::new("theia.dev.training-token").unwrap(),
                6,
            );
            slots[2] = Stack::new(
                items.catalog(),
                ContentId::new("theia.dev.training-stone").unwrap(),
                18,
            );
            let hand = Stack::new(
                items.catalog(),
                ContentId::new("theia.dev.training-sword").unwrap(),
                1,
            );
            let initial = Bag::from_parts(items.catalog(), 0, slots, hand)
                .map_err(|_| Error::InventoryData)?;
            store.ensure_inventory(who, world, &items, &initial).await?;
            store
                .purchase_vendor(who, world, &vendor, &purchase(0))
                .await?;
            (
                who,
                store
                    .purchase_vendor(who, world, &vendor, &purchase(1))
                    .await?
                    .bag,
            )
        }
        "read" => {
            let characters = store.list_characters(owner, world).await?;
            if characters.len() != 1 {
                return Err(Error::Invalid("Vendor restart character count mismatch"));
            }
            let who = store
                .select_character(
                    Binding {
                        owner,
                        character: characters[0].id,
                    },
                    world,
                )
                .await?
                .saved
                .binding;
            let saved = store.load_inventory(who, world, &items).await?;
            let replay = store
                .purchase_vendor(who, world, &vendor, &purchase(1))
                .await?;
            if !replay.replayed || replay.bag != saved {
                return Err(Error::Invalid("Vendor receipt did not survive restart"));
            }
            (who, saved)
        }
        _ => return Err(Error::Invalid("Expected write or read")),
    };
    let count = |name: &str| {
        bag.slots()
            .iter()
            .flatten()
            .filter(|s| s.item().as_str() == name)
            .map(Stack::count)
            .sum::<u32>()
    };
    if bag.revision() != 2
        || count("theia.dev.training-token") != 4
        || count("theia.dev.training-stone") != 24
        || bag
            .main_hand()
            .is_none_or(|s| s.item().as_str() != "theia.dev.training-sword")
    {
        return Err(Error::Invalid("Vendor restart state mismatch"));
    }
    println!(
        "vendor_probe {}",
        serde_json::json!({"character":binding.character.value().to_string(),"profile":vendor.hash(),"bag":serde_json::from_slice::<serde_json::Value>(&bag.to_json()).unwrap()})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
