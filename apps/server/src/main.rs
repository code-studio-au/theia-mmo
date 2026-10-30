mod persistence;
use std::{net::SocketAddr, path::PathBuf};
use theia_network::{Result, server, tls};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let bind: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:5000".into())
        .parse()?;
    let cert = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "build/server-cert.der".into()),
    );
    let mut tickets = None;
    let mut dev_relays: Option<Vec<SocketAddr>> = None;
    let mut chunk_stream = false;
    let mut world_selected = false;
    let mut graybox = false;
    let mut terrain_source = theia_network::procedural::Source::Procedural;
    let mut predicted = false;
    let mut targeted = false;
    let mut combat_catalog = None;
    let mut character_stats = None;
    let mut status_catalog = None;
    let mut patrol_profile = None;
    let mut navigation = None;
    let mut item_catalog = None;
    let mut reward_catalog = None;
    let mut progression_catalog = None;
    let mut quest_catalog = None;
    let mut dialogue_catalog = None;
    let mut vendor_catalog = None;
    let mut guide_position: Option<[f64; 2]> = None;
    let mut dev_persistence = false;
    let mut account_login = false;
    let mut dev_tour = false;
    let mut dev_speed = 1.;
    let mut dev_spawn = [8000., 8000.];
    let mut origin = None;
    let mut replication_span = None;
    let mut world = theia_network::simulation::World::default();
    while let Some(option) = args.next() {
        match option.as_str() {
            "--dev-relays" => dev_relays=Some(args.next().ok_or("Missing relay endpoints")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?),
            "--prediction" => predicted=true,
            "--dev-persistence" => dev_persistence=true,
            "--account-login" => account_login=true,
            "--vendor-catalog" => {
                if vendor_catalog.is_some() { return Err("Choose one vendor catalog".into()); }
                vendor_catalog = Some(PathBuf::from(args.next().ok_or("Missing vendor catalog")?));
            }
            "--dialogue-catalog" => {
                if dialogue_catalog.is_some() {return Err("Choose one dialogue catalog".into());}
                dialogue_catalog=Some(PathBuf::from(args.next().ok_or("Missing dialogue catalog")?));
            }
            "--guide-position" => {
                if guide_position.is_some() {return Err("Choose one guide position".into());}
                let values:Vec<f64>=args.next().ok_or("Missing guide position")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;
                guide_position=Some(values.try_into().map_err(|_|"Expected guide x,z")?);
            }
            "--quest-catalog" => {
                if quest_catalog.is_some() {return Err("Choose one quest catalog".into());}
                quest_catalog=Some(PathBuf::from(args.next().ok_or("Missing quest catalog")?));
            }
            "--progression-catalog" => {
                if progression_catalog.is_some() {return Err("Choose one progression catalog".into());}
                progression_catalog=Some(PathBuf::from(args.next().ok_or("Missing progression catalog")?));
            }
            "--reward-catalog" => {
                if reward_catalog.is_some() {return Err("Choose one reward catalog".into());}
                reward_catalog=Some(PathBuf::from(args.next().ok_or("Missing reward catalog")?));
            }
            "--item-catalog" => {
                if item_catalog.is_some() {return Err("Choose one item catalog".into());}
                item_catalog=Some(PathBuf::from(args.next().ok_or("Missing item catalog")?));
            }
            "--creature-profile" => {
                if patrol_profile.is_some() {return Err("Choose one creature profile".into());}
                patrol_profile=Some(PathBuf::from(args.next().ok_or("Missing creature profile")?));targeted=true;
            }
            "--navigation" => {
                if navigation.is_some() {return Err("Choose one navigation package".into());}
                navigation=Some(PathBuf::from(args.next().ok_or("Missing navigation package")?));
            }
            "--targeting" => targeted=true,
            "--status-catalog" => {
                if status_catalog.is_some() {return Err("Choose one status catalog".into());}
                status_catalog=Some(PathBuf::from(args.next().ok_or("Missing status catalog")?));
            }
            "--character-stats" => {
                if character_stats.is_some() {return Err("Choose one character stats profile".into());}
                character_stats=Some(PathBuf::from(args.next().ok_or("Missing character stats profile")?));
            }
            "--combat-catalog" => {
                if combat_catalog.is_some() {return Err("Choose one combat catalog".into());}
                combat_catalog=Some(PathBuf::from(args.next().ok_or("Missing combat catalog")?));
                targeted=true;
            }
            "--dev-tour" => {dev_tour=true;}
            "--saved-terrain" => { if world_selected {return Err("Choose one world".into());} world_selected=true; graybox=true;
                terrain_source=theia_network::procedural::Source::Saved(std::sync::Arc::new(theia_network::saved_terrain::Store::open(&PathBuf::from(args.next().ok_or("Missing saved terrain directory")?))?)); }
            "--graybox-world" => { if world_selected {return Err("Choose one world".into());} world_selected=true; graybox=true; }
            "--dev-speed" => {dev_speed=args.next().ok_or("Missing developer speed")?.parse()?;}
            "--dev-spawn" => {let values:Vec<f64>=args.next().ok_or("Missing developer spawn")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;dev_spawn=values.try_into().map_err(|_|"Expected x,z")?;}
            "--chunk-stream"=>chunk_stream=true,
            "--world-origin" => {
                let xyz: Vec<f64> = args.next().ok_or("Missing world origin")?.split(',').map(str::parse).collect::<std::result::Result<_,_>>()?;
                let xyz: [f64;3] = xyz.try_into().map_err(|_|"World origin must be x,y,z")?;
                origin = Some(theia_core::coordinates::GlobalPosition::new(xyz).ok_or("Invalid global origin")?);
            }
            "--replication-span" => {replication_span = Some(args.next().ok_or("Missing replication span")?.parse()?);}

            "--renet-tickets" => {
                tickets = Some(PathBuf::from(
                    args.next().ok_or("Missing ticket directory")?,
                ));
            }
            "--tick-hz" => {
                let hz: u32 = args.next().ok_or("Missing tick rate")?.parse()?;
                let rate = theia_core::time::TickRate::from_hz(hz)
                    .ok_or("Tick rate must be 20, 30 or 60")?;
                if !world.set_tick_rate(rate) {
                    return Err("Cannot change active tick rate".into());
                }
            }
            "--chunk-world" => {
                if world_selected { return Err("Choose one world descriptor".into()); }
                world_selected = true;
                let rate = world.tick_rate();
                let directory = PathBuf::from(args.next().ok_or("Missing chunk directory")?);
                let config = theia_network::chunk_world::Config::load(&directory)?;
                world = theia_network::simulation::World::with_chunk_config(&config)?;
                assert!(world.set_tick_rate(rate));
                println!("chunk_world_startup resident={} spawns={}", config.initial().len(), config.spawns().len());
            }
            "--terrain-world" => {
                if world_selected { return Err("Choose one world descriptor".into()); }
                world_selected = true;
                let rate = world.tick_rate();
                let directory = PathBuf::from(args.next().ok_or("Missing terrain directory")?);
                let terrain =
                    theia_network::terrain::Terrain::load(&directory.join("terrain.json"))?;
                if !terrain.has_spawns() { return Err("World requires an explicit validated spawn; chunk-only packages cannot host sessions".into()); }
                world = theia_network::simulation::World::with_terrain(terrain);
                assert!(world.set_tick_rate(rate));
            }
            _ => return Err("Expected --terrain-world or --chunk-world <directory>, --tick-hz <20|30|60> or --renet-tickets <fresh-directory>".into()),
        }
    }
    if status_catalog.is_some() && (character_stats.is_none() || combat_catalog.is_none()) {
        return Err("Statuses require combat and character stats".into());
    }
    if patrol_profile.is_some() != navigation.is_some() {
        return Err("Creature profile and navigation package must be selected together".into());
    }
    if patrol_profile.is_some() && (combat_catalog.is_none() || !predicted || !graybox) {
        return Err("Creature patrol requires predicted graybox combat".into());
    }
    if character_stats.is_some() && combat_catalog.is_none() {
        return Err("Character stats require combat".into());
    }
    if graybox {
        if (tickets.is_none() && !account_login) || origin.is_some() || replication_span.is_some() {
            return Err("Graybox requires Renet and manages its own frames".into());
        }
        let rate = world.tick_rate();
        world = theia_network::simulation::World::with_terrain_source(
            dev_spawn,
            dev_speed,
            terrain_source,
        )?;
        world.set_tick_rate(rate);
        world.dev_tour = dev_tour;
    } else if dev_tour || dev_speed != 1. || dev_spawn != [8000., 8000.] {
        return Err("Developer travel requires --graybox-world".into());
    }
    if predicted {
        world.enable_prediction()?;
    }
    if targeted {
        if !graybox || (tickets.is_none() && !account_login) {
            return Err("Targeting requires a graybox world and secure Renet".into());
        }
        let target = if let Some(path) = patrol_profile {
            use std::io::Read;
            use theia_network::simulation::creatures;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = creatures::Profile::from_json(&bytes)?;
            let source = world
                .procedural()
                .ok_or("Creature patrol requires terrain")?
                .source
                .clone();
            let navigation =
                theia_network::navigation::Store::open(&navigation.unwrap(), source.identity())?;
            println!("development_creature profile={}", profile.id().as_str());
            world.enable_patrol(creatures::Prepared::load(profile, &navigation, &source)?)?
        } else {
            let x = if dev_spawn[0] + 8. <= 15999. {
                dev_spawn[0] + 8.
            } else {
                dev_spawn[0] - 8.
            };
            world.spawn_development_target([x, dev_spawn[1]])?
        };
        println!("development_target id={}", target.entity.value());
    }
    if let Some(path) = combat_catalog {
        use std::io::Read;
        use theia_network::simulation::combat::{DEVELOPMENT_HEALTH, DEVELOPMENT_RESPAWN_MS};
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(theia_core::abilities::MAX_CATALOG_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        if let Some(path) = character_stats {
            let mut stats = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::resources::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut stats)?;
            if let Some(path) = status_catalog {
                let mut statuses = Vec::new();
                std::fs::File::open(path)?
                    .take(theia_core::statuses::MAX_CATALOG_BYTES as u64 + 1)
                    .read_to_end(&mut statuses)?;
                world.enable_status_combat(
                    &bytes,
                    DEVELOPMENT_HEALTH,
                    DEVELOPMENT_RESPAWN_MS,
                    &stats,
                    &statuses,
                )?;
            } else {
                world.enable_resource_combat(
                    &bytes,
                    DEVELOPMENT_HEALTH,
                    DEVELOPMENT_RESPAWN_MS,
                    &stats,
                )?;
            }
        } else {
            world.enable_combat(&bytes, DEVELOPMENT_HEALTH, DEVELOPMENT_RESPAWN_MS)?;
        }
        println!(
            "development_combat abilities={}",
            world.combat_catalog().unwrap().abilities().len()
        );
    }
    if let Some(origin) = origin
        && !world.set_origin(origin)
    {
        return Err("Unsafe world origin".into());
    }
    if let Some(span) = replication_span
        && !world.set_replication_span(span)
    {
        return Err("Replication span must be finite, 1..1024 m".into());
    }
    if world.chunks().is_some() && tickets.is_none() {
        return Err("Chunk worlds require the Renet readiness transport".into());
    }
    if chunk_stream && (world.chunks().is_none() || tickets.is_none()) {
        return Err("--chunk-stream requires --chunk-world and Renet".into());
    }
    if dev_relays.is_some() && tickets.is_none() && !account_login {
        return Err("Loopback relay qualification requires Renet tickets".into());
    }
    if progression_catalog.is_some()
        && (!account_login || world.patrol_snapshot().is_none() || !world.combat_enabled())
    {
        return Err("Progression requires account login and authoritative creature combat".into());
    }
    if quest_catalog.is_some() && progression_catalog.is_none() {
        return Err("Quest objectives require progression".into());
    }
    // Validate world options before opening an endpoint.
    match (dialogue_catalog, guide_position) {
        (Some(path), Some(xz)) if graybox && (account_login || tickets.is_some()) => {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::dialogue::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = std::sync::Arc::new(theia_network::dialogue::Profile::from_json(&bytes)?);
            let guide = theia_server::development::prepare_guide(&mut world, profile, xz).await?;
            println!(
                "guide_ready epoch={} entity={} position={:?}",
                guide.key.epoch.value(),
                guide.key.entity.value(),
                guide.position.metres()
            );
        }
        (None, None) => {}
        _ => {
            return Err(
                "Dialogue requires secure graybox, catalog and explicit guide position".into(),
            );
        }
    }
    if !bind.ip().is_loopback() {
        return Err("Local prototype only: bind must be loopback".into());
    }
    if reward_catalog.is_some()
        && (!account_login
            || item_catalog.is_none()
            || world.patrol_snapshot().is_none()
            || !world.combat_enabled())
    {
        return Err(
            "Rewards require account login, items and authoritative creature combat".into(),
        );
    }
    if item_catalog.is_some() && !account_login {
        return Err("Persistent inventory requires account login".into());
    }
    if account_login {
        let mut wire_items = None;
        let inventory = if let Some(path) = item_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::inventory::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            wire_items = Some(theia_network::renet_runtime::inventory::Profile::from_json(
                &bytes,
            )?);
            Some(theia_persistence::InventoryCatalog::from_json(&bytes)?)
        } else {
            None
        };
        let vendor = if let Some(path) = vendor_catalog.take() {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::vendors::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = theia_persistence::VendorCatalog::from_json(
                &bytes,
                inventory.as_ref().ok_or("Vendor requires items")?,
            )?;
            world.enable_vendor(std::sync::Arc::new(
                theia_network::vendor_wire::Profile::from_json(
                    &bytes,
                    wire_items.as_ref().ok_or("Vendor requires item profile")?,
                )?,
            ))?;
            Some(profile)
        } else {
            None
        };
        let rewards = if let Some(path) = reward_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::rewards::MAX_CATALOG_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let inventory = inventory.as_ref().ok_or("Rewards require items")?;
            let profile = theia_persistence::RewardCatalog::from_json(&bytes, inventory)?;
            let wire = wire_items.as_ref().ok_or("Reward wire items missing")?;
            world.enable_rewards(std::sync::Arc::new(
                theia_network::reward_wire::Profile::from_json(&bytes, wire)?,
            ))?;
            Some(profile)
        } else {
            None
        };
        let progression = if let Some(path) = progression_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::progression::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = theia_persistence::ProgressionCatalog::from_json(&bytes)?;
            world.enable_progression(std::sync::Arc::new(
                theia_network::progression_wire::Profile::from_json(&bytes)?,
            ))?;
            Some(profile)
        } else {
            None
        };
        let quest = if let Some(path) = quest_catalog {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(theia_core::quests::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)?;
            let profile = theia_persistence::QuestCatalog::from_json(&bytes)?;
            world.enable_quest(std::sync::Arc::new(
                theia_network::quest_wire::Profile::from_json(&bytes)?,
            ))?;
            Some(profile)
        } else {
            None
        };
        if tickets.is_some() || dev_persistence || chunk_stream || !graybox || !predicted {
            return Err("Account login requires saved predicted terrain and excludes development ticket/persistence overrides".into());
        }
        let realm = theia_persistence::RealmId::parse(
            &std::env::var("THEIA_DEV_REALM_ID")
                .unwrap_or_else(|_| "74686569-6100-0000-0000-000000000007".into()),
        )
        .ok_or("Invalid account realm UUID")?;
        let url = std::env::var("THEIA_DATABASE_URL")
            .map_err(|_| "Account login requires THEIA_DATABASE_URL")?;
        let store = theia_persistence::Store::connect_loopback(&url, realm).await?;
        let config = theia_server::accounts::Config {
            listen: std::env::var("THEIA_ACCOUNT_BIND")
                .unwrap_or_else(|_| "127.0.0.1:5050".into())
                .parse()?,
            issuer: std::env::var("THEIA_OIDC_ISSUER")
                .map_err(|_| "Account login requires THEIA_OIDC_ISSUER")?,
            client_id: std::env::var("THEIA_OIDC_CLIENT_ID")
                .map_err(|_| "Account login requires THEIA_OIDC_CLIENT_ID")?,
        };
        let socket = std::net::UdpSocket::bind(bind)?;
        let addresses = dev_relays.unwrap_or_else(|| vec![socket.local_addr().unwrap()]);
        let (stop, shutdown) = tokio::sync::watch::channel(false);
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            stop.send_replace(true);
        });
        return theia_server::authenticated::run_with_profiles(
            world,
            socket,
            addresses,
            store,
            config,
            shutdown,
            theia_server::authenticated::GameplayProfiles {
                inventory,
                rewards,
                progression,
                quest,
                vendor,
            },
        )
        .await;
    }
    if vendor_catalog.is_some() {
        return Err("Vendor requires account login".into());
    }
    let persistent = if dev_persistence {
        if tickets.is_none() || !graybox || !predicted || chunk_stream {
            return Err("Development persistence requires saved terrain, predicted Renet and no chunk-stream override".into());
        }
        Some(persistence::prepare(&mut world).await?)
    } else {
        None
    };
    if let Some(directory) = tickets {
        let socket = std::net::UdpSocket::bind(bind)?;
        let address = socket.local_addr()?;
        let transport = if let Some(addresses) = dev_relays {
            theia_network::renet_runtime::issue_local_advertised(socket, &directory, addresses)?
        } else {
            theia_network::renet_runtime::issue_local(socket, &directory)?
        };
        println!("listening={address} local_tickets={}", directory.display());
        let (shutdown, stop) = tokio::sync::watch::channel(false);
        let signal = shutdown.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = signal.send(true);
        });
        if let Some(persistent) = persistent {
            let (bridge, manager) = persistent.into_parts(shutdown);
            let (game, saved) = tokio::join!(
                theia_network::renet_runtime::run_server_with_checkpoints(
                    transport, stop, world, bridge
                ),
                manager
            );
            game?;
            saved?;
            return Ok(());
        }
        return if chunk_stream {
            theia_network::renet_runtime::run_server_streaming(transport, stop, world)
                .await
                .map(|_| ())
        } else {
            theia_network::renet_runtime::run_server(transport, stop, world).await
        };
    }
    let (endpoint, certificate) = tls::server(bind)?;
    if let Some(parent) = cert.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cert, certificate)?;
    println!(
        "listening={} public_certificate={}",
        endpoint.local_addr()?,
        cert.display()
    );
    let (shutdown, stop) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown.send(true);
    });
    server::run_world(endpoint, stop, world).await
}
