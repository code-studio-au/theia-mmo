//! In-process account API, Renet authority and asynchronous checkpoint lifecycle.
//! The simulation never awaits identity-provider, HTTP or SQL operations.
use crate::{
    accounts::{AccountService, Config},
    admissions::GameAdmissions,
};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};
use theia_network::{Result, procedural::Source, renet_runtime, simulation::World};
use theia_persistence::{Store, Worker};
use tokio::sync::watch;

pub async fn run(
    world: World,
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
    store: Store,
    config: Config,
    external: watch::Receiver<bool>,
) -> Result<()> {
    run_with_inventory(world, socket, advertised, store, config, external, None).await
}
pub async fn run_with_inventory(
    world: World,
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
    store: Store,
    config: Config,
    external: watch::Receiver<bool>,
    inventory: Option<theia_persistence::InventoryCatalog>,
) -> Result<()> {
    run_with_rewards(
        world,
        socket,
        advertised,
        store,
        config,
        external,
        (inventory, None),
    )
    .await
}
pub async fn run_with_rewards(
    world: World,
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
    store: Store,
    config: Config,
    external: watch::Receiver<bool>,
    items: (
        Option<theia_persistence::InventoryCatalog>,
        Option<theia_persistence::RewardCatalog>,
    ),
) -> Result<()> {
    run_with_progression(
        world, socket, advertised, store, config, external, items, None,
    )
    .await
}
#[allow(clippy::too_many_arguments)]
pub async fn run_with_progression(
    world: World,
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
    store: Store,
    config: Config,
    external: watch::Receiver<bool>,
    items: (
        Option<theia_persistence::InventoryCatalog>,
        Option<theia_persistence::RewardCatalog>,
    ),
    progression: Option<theia_persistence::ProgressionCatalog>,
) -> Result<()> {
    run_with_profiles(
        world,
        socket,
        advertised,
        store,
        config,
        external,
        GameplayProfiles {
            inventory: items.0,
            rewards: items.1,
            progression,
            quest: None,
            vendor: None,
        },
    )
    .await
}
#[derive(Default)]
pub struct GameplayProfiles {
    pub vendor: Option<theia_persistence::VendorCatalog>,
    pub inventory: Option<theia_persistence::InventoryCatalog>,
    pub rewards: Option<theia_persistence::RewardCatalog>,
    pub progression: Option<theia_persistence::ProgressionCatalog>,
    pub quest: Option<theia_persistence::QuestCatalog>,
}
pub async fn run_with_profiles(
    world: World,
    socket: UdpSocket,
    advertised: Vec<SocketAddr>,
    store: Store,
    config: Config,
    external: watch::Receiver<bool>,
    profiles: GameplayProfiles,
) -> Result<()> {
    let GameplayProfiles {
        inventory,
        rewards,
        progression,
        quest,
        vendor,
    } = profiles;
    let vendor = match (world.vendor_profile(), vendor) {
        (None, None) => None,
        (Some(wire), Some(profile)) if wire.hash() == profile.hash() => {
            Some((profile, wire.clone()))
        }
        _ => return Err("Vendor authority/admission profile mismatch".into()),
    };
    let quest = match (world.quest_profile(), quest) {
        (None, None) => None,
        (Some(wire), Some(profile)) if wire.hash() == profile.hash() => {
            Some((profile, wire.clone()))
        }
        _ => return Err("Objective authority/admission profile mismatch".into()),
    };
    let progression = match (world.progression_profile(), progression) {
        (None, None) => None,
        (Some(wire), Some(profile)) if wire.hash() == profile.hash() => {
            Some((profile, wire.clone()))
        }
        _ => return Err("XP authority/admission profile mismatch".into()),
    };
    let rewards = match (world.reward_profile(), rewards) {
        (None, None) => None,
        (Some(wire), Some(profile))
            if wire.hash() == profile.hash()
                && inventory
                    .as_ref()
                    .is_some_and(|i| i.hash() == profile.items().hash()) =>
        {
            Some((profile, wire.clone()))
        }
        _ => return Err("Reward authority/admission profile mismatch".into()),
    };
    let terrain = world
        .procedural()
        .ok_or("Account entry requires saved terrain")?;
    if !matches!(terrain.source, Source::Saved(_))
        || terrain.speed != 1.
        || world.dev_tour
        || !world.prediction_enabled()
    {
        return Err("Account entry requires normal predicted saved terrain".into());
    }
    let identity = terrain.source.identity();
    let spawn = world
        .development_player_spawn(1)
        .ok_or("Account spawn lacks collision support")?;
    world.validate_restored_player(spawn)?;
    let (transport, issuer) = renet_runtime::secure_issuer(socket, advertised)?;
    let (publisher, worker) = Worker::start(store);
    let status = worker.subscribe_status();
    let mut service = AccountService::prepare(config, publisher.clone(), identity, spawn).await?;
    let gate =
        service.enable_game_with_vendors(issuer, inventory, rewards, progression, quest, vendor)?;
    println!(
        "account_game_ready {}",
        serde_json::json!({"account_address":service.address(),"world":world.identity()})
    );
    let (stop, shutdown) = watch::channel(false);
    let (finished, done) = watch::channel(false);
    let game = async {
        let result = renet_runtime::run_server_with_admissions(
            transport,
            shutdown.clone(),
            world,
            gate.clone(),
        )
        .await;
        finished.send_replace(true);
        stop.send_replace(true);
        if let Ok(stats) = &result {
            println!("transport_summary {}", serde_json::json!(stats));
        }
        result
    };
    let api = async {
        let result = service.run(shutdown.clone(), status).await;
        stop.send_replace(true);
        if let Ok(stats) = &result {
            println!("account_summary {}", serde_json::json!(stats));
        }
        result
    };
    let saving = manage(
        gate.clone(),
        publisher,
        worker,
        external,
        done,
        stop.clone(),
    );
    let (game, api, saved) = tokio::join!(game, api, saving);
    gate.fail_closed();
    game?;
    api?;
    saved?;
    Ok(())
}
async fn manage(
    gate: Arc<GameAdmissions>,
    publisher: theia_persistence::Publisher,
    worker: Worker,
    mut external: watch::Receiver<bool>,
    mut done: watch::Receiver<bool>,
    stop: watch::Sender<bool>,
) -> Result<()> {
    let mut status = worker.subscribe_status();
    let mut cadence = tokio::time::interval(Duration::from_secs(1));
    cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut failure = false;
    let mut external_open = true;
    loop {
        if *external.borrow() {
            stop.send_replace(true);
        }
        if *done.borrow() {
            break;
        }
        tokio::select! {biased;
            _=done.changed()=>{},
            changed=status.changed()=>{
                if changed.is_err() || status.borrow().failed {
                    failure=true;gate.fail_closed();stop.send_replace(true);
                    break;
                }
            },
            _=external.changed(),if external_open=>{external_open=false;stop.send_replace(true);},
            _=cadence.tick()=>{
                if gate.flush(&publisher,&worker.status()).is_err() {failure=true;gate.fail_closed();stop.send_replace(true);break;}
            },
        }
    }
    if !*done.borrow() {
        let wait = async {
            while !*done.borrow() {
                done.changed()
                    .await
                    .map_err(|_| "Authority completion channel closed")?;
            }
            Ok::<_, &str>(())
        };
        if !matches!(
            tokio::time::timeout(Duration::from_secs(3), wait).await,
            Ok(Ok(()))
        ) {
            failure = true;
        }
    }
    if !failure {
        let flushed = tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                if gate.flush(&publisher, &worker.status())? == 0 {
                    return Ok::<_, crate::admissions::Error>(());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await;
        failure = !matches!(flushed, Ok(Ok(())));
    }
    let stored = worker.shutdown().await;
    println!(
        "account_persistence_summary {}",
        serde_json::json!({"worker":stored.as_ref().ok(),"failed":failure||stored.is_err()})
    );
    if failure {
        return Err("Account admission/checkpoint lifecycle failed".into());
    }
    stored?;
    Ok(())
}
