use crate::{
    Result, tls,
    wire::{Input, Intent, Snapshot, TICK_HZ, VERSION, WELCOME_BYTES, Welcome, encode},
};
use std::{net::SocketAddr, time::Duration};
use tokio::{sync::watch, time::MissedTickBehavior};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub correction: Option<crate::prediction_wire::Corrected>,
    pub correction_ready: Option<crate::wire::ChunkReady>,
    pub correction_received: Option<std::time::Instant>,
    pub id: Option<u64>,
    pub snapshot: Option<Snapshot>,
    pub error: Option<String>,
    pub transport: Option<TransportStats>,
    pub welcome: Option<Welcome>,
    pub wire_rebases: u64,
    pub chunk_report: Option<crate::wire::ChunkReady>,
    pub chunk_ack: Option<crate::wire::ChunkAck>,
    pub chunk_offer: Option<crate::wire::ChunkOffer>,
}

impl State {
    /// Latest authoritative owner centre, even if its independent correction
    /// datagram arrives before the matching multi-player snapshot.
    pub fn authority_focus(&self) -> Option<theia_core::coordinates::GlobalPosition> {
        if let Some(c) = self.correction.filter(|c| {
            self.snapshot
                .as_ref()
                .is_none_or(|s| c.state.input.tick >= s.tick)
        }) {
            let mut p = c.state.pose.feet.metres();
            p[1] += f64::from(crate::world::PAWN_Y);
            return theia_core::coordinates::GlobalPosition::new(p);
        }
        let snapshot = self.snapshot.as_ref()?;
        snapshot.global_position(snapshot.players.iter().find(|p| Some(p.id) == self.id)?)
    }

    /// Authority-side retention barrier only. Actual unload must additionally
    /// protect all currently presented/interpolated poses and geometry needs.
    pub fn chunk_permission_confirmed(&self, chunks: &[theia_core::coordinates::ChunkId]) -> bool {
        let (Some(welcome), Some(report), Some(ack), Some(snapshot)) = (
            &self.welcome,
            &self.chunk_report,
            &self.chunk_ack,
            &self.snapshot,
        ) else {
            return false;
        };
        report.valid()
            && ack.tick > 0
            && ack.matches(report)
            && ack.epoch == welcome.epoch
            && snapshot.epoch == welcome.epoch
            && snapshot.tick >= ack.tick
            && report
                .chunks
                .iter()
                .copied()
                .eq(chunks.iter().map(|id| [id.x, id.z]))
    }
    /// Content/render coordinates reconstructed through checked global positions.
    pub fn local_view(&self) -> Result<Self> {
        let mut view = self.clone();
        if let Some(snapshot) = &self.snapshot {
            let welcome = self.welcome.as_ref().ok_or("Snapshot without admission")?;
            if !welcome.accepts(snapshot) {
                return Err("Snapshot outside admitted world/frame".into());
            }
            if welcome.terrain_stream {
                return Ok(view);
            }
            view.snapshot = Some(
                snapshot
                    .projected(welcome.frame())
                    .ok_or("Snapshot outside content frame")?,
            );
        }
        Ok(view)
    }
}

/// Observed Renet diagnostics; byte rates exclude UDP/netcode overhead.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct TransportStats {
    pub chunk_ready_reports_sent: u64,
    pub chunk_ready_acks_received: u64,
    pub chunk_offers_received: u64,
    pub rtt_ms: f64,
    pub packet_loss: f64,
    pub bytes_sent_per_second: f64,
    pub bytes_received_per_second: f64,
    pub max_movement_queue_bytes: usize,
    pub max_event_queue_bytes: usize,
    pub snapshots_received: u64,
    pub stale_snapshots_discarded: u64,
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
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(stream.read_to_end(WELCOME_BYTES).await?)
        }) => result??,
        _ = shutdown.changed() => return Ok(()),
    };
    let welcome = Welcome::decode(&welcome, expected_world)?;
    let id = welcome.id;
    state.send_replace(State {
        id: Some(id),
        welcome: Some(welcome.clone()),
        ..State::default()
    });
    let mut tick = tokio::time::interval(Duration::from_secs_f64(1. / f64::from(TICK_HZ)));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut sequence = 0u64;
    let mut latest_tick = 0;
    let mut last_origin = welcome.origin;
    let mut rebases = 0;
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => break,
            _ = tick.tick() => {
                sequence=sequence.checked_add(1).ok_or("Input sequence exhausted")?;
                let intent = *input.borrow();
                if !intent.valid() { return Err("Invalid local intent".into()); }
                connection.send_datagram(encode(&Input {version:VERSION, epoch:welcome.epoch, sequence, intent})?.into())?;
            }
            bytes = connection.read_datagram() => {
                let snapshot = Snapshot::decode(&bytes?)?;
                if !welcome.accepts(&snapshot) {return Err("Snapshot epoch, actor or frame mismatch".into());}
                if snapshot.tick>latest_tick {
                    if snapshot.origin!=last_origin {rebases+=1; last_origin=snapshot.origin;}
                    latest_tick=snapshot.tick;
                    state.send_replace(State {id:Some(id), snapshot:Some(snapshot), error:None, transport:None, welcome:Some(welcome.clone()), wire_rebases:rebases, ..State::default()});
                }
            }
        }
    }
    connection.close(0u32.into(), b"Client leaving");
    endpoint.close(0u32.into(), b"Client stopping");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::{ChunkAck, ChunkReady, Player};
    use theia_core::coordinates::ChunkId;

    #[test]
    fn chunk_confirmation_waits_for_exact_report_and_post_application_snapshot() {
        let ids = [ChunkId { x: 0, z: 0 }];
        let report = ChunkReady {
            version: VERSION,
            epoch: 7,
            sequence: 1,
            chunks: vec![[0, 0]],
        };
        let mut state = State {
            welcome: Some(Welcome {
                terrain_stream: false,
                id: 1,
                epoch: 7,
                world: [0; 32],
                origin: [0.; 3],
            }),
            snapshot: Some(Snapshot {
                version: VERSION,
                epoch: 7,
                tick: 9,
                tick_hz: 30,
                origin: [0.; 3],
                players: vec![Player {
                    id: 1,
                    xz: [0.; 2],
                    y: 0.9,
                    applied_sequence: 1,
                }],
            }),
            chunk_report: Some(report.clone()),
            ..State::default()
        };
        assert!(!state.chunk_permission_confirmed(&ids));
        state.chunk_ack = Some(ChunkAck::new(&report, 10));
        assert!(
            !state.chunk_permission_confirmed(&ids),
            "Reliable ack can overtake its snapshot"
        );
        state.snapshot.as_mut().unwrap().tick = 10;
        assert!(state.chunk_permission_confirmed(&ids));
        // A -> empty -> A must not reuse the earlier acknowledgement of A.
        state.chunk_report.as_mut().unwrap().sequence = 3;
        assert!(!state.chunk_permission_confirmed(&ids));
        state.chunk_ack = Some(ChunkAck::new(state.chunk_report.as_ref().unwrap(), 11));
        assert!(!state.chunk_permission_confirmed(&ids));
        state.snapshot.as_mut().unwrap().tick = 11;
        assert!(state.chunk_permission_confirmed(&ids));
        assert!(!state.chunk_permission_confirmed(&[]));
        state.snapshot.as_mut().unwrap().epoch = 8;
        assert!(!state.chunk_permission_confirmed(&ids));
        state.snapshot.as_mut().unwrap().epoch = 7;
        state.chunk_report.as_mut().unwrap().chunks.clear();
        assert!(!state.chunk_permission_confirmed(&[]));
        state.chunk_ack = Some(ChunkAck::new(state.chunk_report.as_ref().unwrap(), 12));
        state.snapshot.as_mut().unwrap().tick = 12;
        assert!(state.chunk_permission_confirmed(&[]));
        let local = state.local_view().unwrap();
        assert!(
            local.chunk_permission_confirmed(&[]),
            "Projection must preserve the barrier"
        );
    }
}
