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
    if graybox {
        if tickets.is_none() || origin.is_some() || replication_span.is_some() {
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
    if dev_relays.is_some() && tickets.is_none() {
        return Err("Loopback relay qualification requires Renet tickets".into());
    }
    // Validate world options before opening an endpoint.
    if !bind.ip().is_loopback() {
        return Err("Local prototype only: bind must be loopback".into());
    }
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
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = shutdown.send(true);
        });
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
