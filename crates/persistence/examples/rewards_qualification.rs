//! Owned PostgreSQL reward/claim clean-restart probe, not a creature-kill proof.
#[path = "../tests/support/mod.rs"]
mod support;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::{Bag, Stack},
};
use theia_persistence::{
    Binding, CharacterName, Error, InventoryCatalog, RealmId, RewardCatalog, Store,
};
#[tokio::main]
async fn main() -> theia_persistence::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let mode = std::env::args()
        .nth(1)
        .ok_or(Error::Invalid("Expected write or read"))?;
    let realm = RealmId::parse("74686569-6100-0000-0000-000000000008").unwrap();
    let world = [118; 32];
    let items = InventoryCatalog::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))?;
    let profile = RewardCatalog::from_json(
        include_bytes!("../../../content/gameplay/development-rewards.json"),
        &items,
    )?;
    let reward = ContentId::new("theia.dev.first-training-kill").unwrap();
    let mut store = Store::connect_loopback(&url, realm).await?;
    let owner = store
        .resolve_owner(
            "http://127.0.0.1:40123/reward-restart-fixture",
            "synthetic-reward-subject",
        )
        .await?;
    let mut probes = Vec::new();
    for name in ["Rewardready", "Rewardtaken"] {
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
                let people = store.list_characters(owner, world).await?;
                let person = people
                    .iter()
                    .find(|p| p.name.as_str() == name)
                    .ok_or(Error::Invalid("Missing reward restart character"))?;
                Binding {
                    owner,
                    character: person.id,
                }
            }
            _ => return Err(Error::Invalid("Expected write or read")),
        };
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(
            items.catalog(),
            ContentId::new("theia.dev.training-token").unwrap(),
            99,
        );
        let seed = Bag::from_parts(
            items.catalog(),
            0,
            slots,
            Stack::new(
                items.catalog(),
                ContentId::new("theia.dev.training-sword").unwrap(),
                1,
            ),
        )
        .map_err(|_| Error::InventoryData)?;
        if mode == "write" {
            store
                .ensure_inventory(binding, world, &items, &seed)
                .await?;
            store
                .offer_reward(binding, world, &profile, &reward)
                .await?;
        }
        let claims = store.list_reward_claims(binding, world, &profile).await?;
        if claims.len() != 1 {
            return Err(Error::Invalid("Reward restart claim count mismatch"));
        }
        let claim = claims[0].clone();
        if name == "Rewardtaken" {
            let result = store
                .claim_reward(binding, world, &profile, claim.id, 0)
                .await?;
            if result.replayed != (mode == "read") {
                return Err(Error::Invalid("Reward restart receipt mismatch"));
            }
        }
        let bag = store.load_inventory(binding, world, &items).await?;
        let saved = store
            .offer_reward(binding, world, &profile, &reward)
            .await?;
        if saved.id != claim.id
            || saved.consumed_revision != if name == "Rewardtaken" { Some(1) } else { None }
        {
            return Err(Error::Invalid(
                "Reward availability/consumption changed across restart",
            ));
        }
        if bag.revision() != if name == "Rewardtaken" { 1 } else { 0 }
            || bag.main_hand().map(|s| s.item().as_str()) != Some("theia.dev.training-sword")
            || bag
                .slots()
                .iter()
                .flatten()
                .filter(|s| s.item().as_str() == "theia.dev.training-token")
                .map(Stack::count)
                .sum::<u32>()
                != if name == "Rewardtaken" { 100 } else { 99 }
        {
            return Err(Error::Invalid("Reward restart item contents mismatch"));
        }
        if store
            .ensure_inventory(binding, world, &items, &Bag::empty(items.catalog()))
            .await?
            != bag
        {
            return Err(Error::Invalid("Reward items were reseeded"));
        }
        let bag: serde_json::Value =
            serde_json::from_slice(&bag.to_json()).map_err(|_| Error::InventoryData)?;
        probes.push(serde_json::json!({"name":name,"character":binding.character.value().to_string(),"claim":uuid::Uuid::from_bytes(saved.id.bytes()).to_string(),"reward":saved.reward.as_str(),"consumed_revision":saved.consumed_revision,"bag":bag}));
    }
    println!(
        "reward_probe {}",
        serde_json::json!({"owner":owner.value().to_string(),"profile":profile.hash(),"characters":probes})
    );
    store.close().await?;
    drop(admin);
    driver.await.unwrap().unwrap();
    Ok(())
}
