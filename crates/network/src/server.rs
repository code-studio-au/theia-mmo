use crate::{
    Result,
    simulation::World,
    wire::{Input, MAX_PLAYERS, TICK_HZ, VERSION, encode},
};
use quinn::{Connection, Endpoint};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Semaphore, mpsc, oneshot, watch},
    task::JoinSet,
    time::{Instant, MissedTickBehavior},
};

struct Join {
    connection: Connection,
    inputs: watch::Receiver<Option<Input>>,
    welcome: oneshot::Sender<u64>,
}
struct Peer {
    connection: Connection,
    inputs: watch::Receiver<Option<Input>>,
}

/// One authority owns the world. Each connection has one superseding input slot.
pub async fn run(endpoint: Endpoint, mut shutdown: watch::Receiver<bool>) -> Result<()> {
    let mut world = World::default();
    let mut peers = BTreeMap::<u64, Peer>::new();
    let (joins_tx, mut joins) = mpsc::channel::<Join>(MAX_PLAYERS);
    let permits = Arc::new(Semaphore::new(MAX_PLAYERS));
    let mut sessions = JoinSet::new();
    let mut tick = tokio::time::interval(Duration::from_secs_f64(1. / f64::from(TICK_HZ)));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            Some(join) = joins.recv() => {
                if let Some(id) = world.join() {
                    println!("player_join id={id}");
                    peers.insert(id, Peer {connection:join.connection, inputs:join.inputs});
                    let _ = join.welcome.send(id);
                } else { join.connection.close(1u32.into(), b"World full"); }
            }
            _ = tick.tick() => {
                peers.retain(|id, peer| {
                    if peer.connection.close_reason().is_some() {
                        world.leave(*id); println!("player_leave id={id}"); false
                    } else {
                        if let Some(input) = *peer.inputs.borrow_and_update() { world.input(*id, input); }
                        true
                    }
                });
                let snapshot = world.step();
                let bytes = encode(&snapshot)?;
                for peer in peers.values() {
                    // A newer full snapshot supersedes any lost datagram.
                    let _ = peer.connection.send_datagram(bytes.clone().into());
                }
            }
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { break; };
                let Ok(permit) = permits.clone().try_acquire_owned() else { incoming.refuse(); continue; };
                let joins = joins_tx.clone();
                sessions.spawn(async move {
                    let _permit = permit;
                    if let Err(error) = session(incoming, joins).await { eprintln!("session_end: {error}"); }
                });
            }
            Some(_) = sessions.join_next(), if !sessions.is_empty() => {}
        }
    }
    endpoint.close(0u32.into(), b"Server stopping");
    sessions.abort_all();
    while sessions.join_next().await.is_some() {}
    Ok(())
}

async fn session(incoming: quinn::Incoming, joins: mpsc::Sender<Join>) -> Result<()> {
    let connection = tokio::time::timeout(Duration::from_secs(5), incoming).await??;
    let (inputs_tx, inputs) = watch::channel(None);
    let (welcome, id) = oneshot::channel();
    joins
        .send(Join {
            connection: connection.clone(),
            inputs,
            welcome,
        })
        .await?;
    let id = id.await?;
    let mut stream = connection.open_uni().await?;
    let mut welcome = VERSION.to_le_bytes().to_vec();
    welcome.extend(id.to_le_bytes());
    stream.write_all(&welcome).await?;
    stream.finish()?;
    let mut latest = 0;
    let mut period = Instant::now();
    let mut count = 0u32;
    loop {
        let bytes = connection.read_datagram().await?;
        if period.elapsed() >= Duration::from_secs(1) {
            period = Instant::now();
            count = 0;
        }
        count += 1;
        if count > 180 {
            connection.close(2u32.into(), b"Input rate exceeded");
            return Err("Input rate exceeded".into());
        }
        let input = match Input::decode(&bytes) {
            Ok(input) => input,
            Err(error) => {
                connection.close(3u32.into(), b"Invalid input");
                return Err(error);
            }
        };
        if input.sequence > latest {
            latest = input.sequence;
            inputs_tx.send_replace(Some(input));
        }
    }
}
