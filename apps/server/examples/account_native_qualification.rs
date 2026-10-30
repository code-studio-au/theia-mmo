//! Owned real-provider/database/native-client qualification. Uses the same
//! account/admission/authority runner as the opt-in server executable.
#[path = "../../../crates/persistence/tests/support/mod.rs"]
mod support;
use std::{net::UdpSocket, path::PathBuf, sync::Arc};
use theia_network::{procedural::Source, saved_terrain, simulation::World};
use theia_persistence::{RealmId, Store};
use theia_server::{accounts::Config, authenticated};
use tokio::sync::watch;
#[tokio::main]
async fn main() -> theia_network::Result<()> {
    let (url, admin, driver) = support::owned().await;
    let source = Source::Saved(Arc::new(saved_terrain::Store::open(&PathBuf::from(
        std::env::var("THEIA_TEST_SAVED_TERRAIN")?,
    ))?));
    let spawn = std::env::var("THEIA_TEST_SPAWN").unwrap_or_else(|_| "15400,12860".into());
    let spawn: Vec<f64> = spawn
        .split(',')
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    let mut world = World::with_terrain_source(
        spawn.try_into().map_err(|_| "Invalid test spawn")?,
        1.,
        source.clone(),
    )?;
    world.enable_prediction()?;
    if std::env::var("THEIA_TEST_DIALOGUE").as_deref() == Ok("1") {
        let values: Vec<f64> = std::env::var("THEIA_TEST_GUIDE_POSITION")?
            .split(',')
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;
        let profile = Arc::new(theia_network::dialogue::Profile::from_json(
            include_bytes!("../../../content/gameplay/development-dialogue.json"),
        )?);
        theia_server::development::prepare_guide(
            &mut world,
            profile,
            values.try_into().map_err(|_| "Invalid guide x,z")?,
        )
        .await?;
    }
    let store = Store::connect_loopback(
        &url,
        RealmId::parse("74686569-6100-0000-0000-000000000007").unwrap(),
    )
    .await?;
    let config = Config {
        listen: std::env::var("THEIA_TEST_ACCOUNT_BIND")?.parse()?,
        issuer: std::env::var("THEIA_TEST_OIDC_ISSUER")?,
        client_id: std::env::var("THEIA_TEST_OIDC_CLIENT")?,
    };
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let address = socket.local_addr()?;
    let (stop, shutdown) = watch::channel(false);
    let inventory = if std::env::var("THEIA_TEST_INVENTORY").is_ok_and(|v| v == "1") {
        let bytes: &[u8] = if std::env::var("THEIA_TEST_EQUIPMENT").is_ok_and(|v| v == "1") {
            include_bytes!("../../../content/gameplay/development-equipment-items.json")
        } else {
            include_bytes!("../../../content/gameplay/development-items.json")
        };
        Some(theia_persistence::InventoryCatalog::from_json(bytes)?)
    } else {
        None
    };
    let rewards = if std::env::var("THEIA_TEST_REWARDS").is_ok_and(|v| v == "1") {
        let profile = theia_network::simulation::creatures::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-creature-patrol.json"
        ))?;
        let navigation = theia_network::navigation::Store::open(
            &PathBuf::from(std::env::var("THEIA_TEST_NAVIGATION")?),
            source.identity(),
        )?;
        world.enable_patrol(theia_network::simulation::creatures::Prepared::load(
            profile,
            &navigation,
            &source,
        )?)?;
        let abilities = if std::env::var("THEIA_TEST_QUESTS").is_ok_and(|v| v == "1") {
            std::fs::read(std::env::var("THEIA_TEST_COMBAT_PROFILE")?)?
        } else {
            include_bytes!("../../../content/gameplay/development-abilities.json").to_vec()
        };
        world.enable_combat(
            &abilities,
            theia_network::simulation::combat::DEVELOPMENT_HEALTH,
            theia_network::simulation::combat::DEVELOPMENT_RESPAWN_MS,
        )?;
        let bytes = include_bytes!("../../../content/gameplay/development-rewards.json");
        let profile = theia_persistence::RewardCatalog::from_json(
            bytes,
            inventory.as_ref().ok_or("Reward fixture requires items")?,
        )?;
        let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-equipment-items.json"
        ))?;
        world.enable_rewards(Arc::new(theia_network::reward_wire::Profile::from_json(
            bytes, &items,
        )?))?;
        Some(profile)
    } else {
        None
    };
    let progression = if std::env::var("THEIA_TEST_PROGRESSION").is_ok_and(|v| v == "1") {
        let bytes = std::fs::read(std::env::var("THEIA_TEST_PROGRESSION_PROFILE")?)?;
        let profile = theia_persistence::ProgressionCatalog::from_json(&bytes)?;
        world.enable_progression(Arc::new(
            theia_network::progression_wire::Profile::from_json(&bytes)?,
        ))?;
        Some(profile)
    } else {
        None
    };
    let quest = if std::env::var("THEIA_TEST_QUESTS").is_ok_and(|v| v == "1") {
        let bytes = include_bytes!("../../../content/gameplay/development-quest.json");
        let profile = theia_persistence::QuestCatalog::from_json(bytes)?;
        world.enable_quest(Arc::new(theia_network::quest_wire::Profile::from_json(
            bytes,
        )?))?;
        Some(profile)
    } else {
        None
    };
    let vendor = if std::env::var("THEIA_TEST_VENDORS").is_ok_and(|v| v == "1") {
        let bytes = include_bytes!("../../../content/gameplay/development-vendor.json");
        let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-equipment-items.json"
        ))?;
        let profile = theia_persistence::VendorCatalog::from_json(
            bytes,
            inventory.as_ref().ok_or("Vendor fixture requires items")?,
        )?;
        world.enable_vendor(Arc::new(theia_network::vendor_wire::Profile::from_json(
            bytes, &items,
        )?))?;
        Some(profile)
    } else {
        None
    };
    let run = authenticated::run_with_profiles(
        world,
        socket,
        vec![address],
        store,
        config,
        shutdown,
        authenticated::GameplayProfiles {
            vendor,
            inventory,
            rewards,
            progression,
            quest,
        },
    );
    tokio::pin!(run);
    let result = tokio::select! {
        result=&mut run=>result,
        _=tokio::signal::ctrl_c()=>{stop.send_replace(true);run.await},
    };
    drop(admin);
    driver.await??;
    result
}
