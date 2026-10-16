use crate::{
    Result, tls,
    wire::{Input, Intent, Snapshot, TICK_HZ, VERSION, encode},
};
use std::{net::SocketAddr, time::Duration};
use tokio::{sync::watch, time::MissedTickBehavior};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub id: Option<u64>,
    pub snapshot: Option<Snapshot>,
    pub error: Option<String>,
}

/// Run outside the Bevy thread. Watch mailboxes remain bounded when rendering stalls.
pub async fn run(
    address: SocketAddr,
    certificate: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    shutdown: watch::Receiver<bool>,
) -> Result<()> {
    run_in_world(
        address,
        certificate,
        input,
        state,
        shutdown,
        crate::terrain::GRAYBOX_ID,
    )
    .await
}

pub async fn run_in_world(
    address: SocketAddr,
    certificate: &[u8],
    input: watch::Receiver<Intent>,
    state: watch::Sender<State>,
    mut shutdown: watch::Receiver<bool>,
    expected_world: [u8; 32],
) -> Result<()> {
    let endpoint = tls::client(address, certificate)?;
    let connecting = endpoint.connect(address, "localhost")?;
    let connection = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(5), connecting) => result??,
        _ = shutdown.changed() => return Ok(()),
    };
    let welcome = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(5), async {
            let mut stream = connection.accept_uni().await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(stream.read_to_end(44).await?)
        }) => result??,
        _ = shutdown.changed() => return Ok(()),
    };
    if welcome.len() != 44 || u32::from_le_bytes(welcome[..4].try_into()?) != VERSION {
        return Err("Invalid welcome".into());
    }
    let id = u64::from_le_bytes(welcome[4..12].try_into()?);
    if welcome[12..] != expected_world {
        return Err("Server world does not match local terrain/graybox".into());
    }
    if id == 0 {
        return Err("Invalid player identity".into());
    }
    state.send_replace(State {
        id: Some(id),
        ..State::default()
    });
    let mut tick = tokio::time::interval(Duration::from_secs_f64(1. / f64::from(TICK_HZ)));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut sequence = 0u64;
    let mut latest_tick = 0;
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            _ = tick.tick() => {
                sequence=sequence.checked_add(1).ok_or("Input sequence exhausted")?;
                let intent = *input.borrow();
                if !intent.valid() { return Err("Invalid local intent".into()); }
                connection.send_datagram(encode(&Input {version:VERSION, sequence, intent})?.into())?;
            }
            bytes = connection.read_datagram() => {
                let snapshot = Snapshot::decode(&bytes?)?;
                if snapshot.tick>latest_tick {
                    latest_tick=snapshot.tick;
                    state.send_replace(State {id:Some(id), snapshot:Some(snapshot), error:None});
                }
            }
        }
    }
    connection.close(0u32.into(), b"Client leaving");
    endpoint.close(0u32.into(), b"Client stopping");
    Ok(())
}
