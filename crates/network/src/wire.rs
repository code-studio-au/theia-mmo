//! Versioned, bounded datagrams. No client coordinates, elapsed time or player ID.
use serde::{Deserialize, Serialize};
use theia_core::{
    coordinates::{GlobalPosition, LocalFrame, LocalPosition, MAX_LOCAL_METRES},
    identity::{EntityId, WorldEpoch},
};

pub const VERSION: u32 = 11;
pub const MAX_PLAYERS: usize = 8;
pub const MAX_BYTES: usize = 1200;
pub const TICK_HZ: u32 = 30;
pub const WORLD_LIMIT: f32 = 19.;
pub const COORDINATE_LIMIT: f32 = 1023.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    #[serde(default)]
    pub jump: bool,
    pub axes: [i16; 2],
    pub sprint: bool,
}
impl Intent {
    pub fn valid(self) -> bool {
        self.axes.iter().all(|v| (-1000..=1000).contains(v))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub version: u32,
    pub epoch: u64,
    pub sequence: u64,
    pub intent: Intent,
}
impl Input {
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        if value.version != VERSION
            || WorldEpoch::new(value.epoch).is_none()
            || value.sequence == 0
            || !value.intent.valid()
        {
            return Err("Invalid input version, sequence or axes".into());
        }
        Ok(value)
    }
}

/// Reliable client-owned presentation readiness; never supplies world geometry
/// or another actor ID. Empty sets withdraw permission for intentional movement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkReady {
    pub version: u32,
    pub epoch: u64,
    pub sequence: u64,
    pub chunks: Vec<[i64; 2]>,
}

/// Authority has simulated at least this tick with the exact accepted report.
/// The client must retain old geometry until this and presentation checks pass.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkAck {
    pub version: u32,
    pub epoch: u64,
    pub sequence: u64,
    pub chunks: Vec<[i64; 2]>,
    pub tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkOffer {
    pub version: u32,
    pub epoch: u64,
    pub generation: u64,
    pub chunks: Vec<[i64; 2]>,
}
impl ChunkOffer {
    pub fn valid(&self) -> bool {
        !self.chunks.is_empty()
            && ChunkReady {
                version: self.version,
                epoch: self.epoch,
                sequence: self.generation,
                chunks: self.chunks.clone(),
            }
            .valid()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", deny_unknown_fields)]
pub enum ChunkEvent {
    Ack(ChunkAck),
    Offer(ChunkOffer),
}
impl ChunkEvent {
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let event: Self = decode(bytes)?;
        let valid = match &event {
            Self::Ack(ack) => {
                ack.tick > 0
                    && ChunkReady {
                        version: ack.version,
                        epoch: ack.epoch,
                        sequence: ack.sequence,
                        chunks: ack.chunks.clone(),
                    }
                    .valid()
            }
            Self::Offer(offer) => offer.valid(),
        };
        if !valid {
            return Err("Invalid chunk event".into());
        }
        Ok(event)
    }
}
impl ChunkAck {
    pub fn new(report: &ChunkReady, tick: u64) -> Self {
        Self {
            version: report.version,
            epoch: report.epoch,
            sequence: report.sequence,
            chunks: report.chunks.clone(),
            tick,
        }
    }
    pub fn matches(&self, report: &ChunkReady) -> bool {
        self.version == report.version
            && self.epoch == report.epoch
            && self.sequence == report.sequence
            && self.chunks == report.chunks
    }
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        let report = ChunkReady {
            version: value.version,
            epoch: value.epoch,
            sequence: value.sequence,
            chunks: value.chunks.clone(),
        };
        if !report.valid() || value.tick == 0 {
            return Err("Invalid chunk acknowledgement".into());
        }
        Ok(value)
    }
}
impl ChunkReady {
    pub fn valid(&self) -> bool {
        self.version == VERSION
            && WorldEpoch::new(self.epoch).is_some()
            && self.sequence != 0
            && self.chunks.len() <= crate::chunks::MAX_RESIDENT
            && self.chunks.iter().all(|&[x, z]| {
                theia_core::coordinates::ChunkId { x, z }
                    .origin(theia_core::coordinates::INITIAL_CHUNK_METRES)
                    .is_some()
            })
            && self.chunks.windows(2).all(|ids| ids[0] < ids[1])
    }
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        if !value.valid() {
            return Err("Invalid chunk readiness version, epoch, sequence or set".into());
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub id: u64,
    pub xz: [f32; 2],
    pub y: f32,
    pub applied_sequence: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub epoch: u64,
    pub origin: [f64; 3],
    pub tick: u64,
    pub tick_hz: u32,
    pub players: Vec<Player>,
}
impl Snapshot {
    pub fn frame(&self) -> Option<LocalFrame> {
        Some(LocalFrame::new(GlobalPosition::new(self.origin)?))
    }
    pub fn global_position(&self, player: &Player) -> Option<GlobalPosition> {
        self.frame()?
            .to_global(LocalPosition::new([player.xz[0], player.y, player.xz[1]])?)
    }
    /// Reconstruct global truth before projecting into a content/render frame.
    pub fn projected(&self, frame: LocalFrame) -> Option<Self> {
        let mut projected = self.clone();
        projected.origin = frame.origin().metres();
        for (player, source) in projected.players.iter_mut().zip(&self.players) {
            let p = frame.to_local(self.global_position(source)?)?.metres();
            player.xz = [p[0], p[2]];
            player.y = p[1];
        }
        Some(projected)
    }
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        if value.version != VERSION
            || WorldEpoch::new(value.epoch).is_none()
            || GlobalPosition::new(value.origin).is_none()
            || theia_core::time::TickRate::from_hz(value.tick_hz).is_none()
            || value.players.len() > MAX_PLAYERS
            || value.players.iter().any(|p| {
                EntityId::new(p.id).is_none()
                    || !p.y.is_finite()
                    || f64::from(p.y.abs()) > MAX_LOCAL_METRES
                    || p.xz
                        .iter()
                        .any(|v| !v.is_finite() || f64::from(v.abs()) > MAX_LOCAL_METRES)
            })
            || value
                .players
                .iter()
                .any(|p| value.global_position(p).is_none())
            || value.players.windows(2).any(|p| p[0].id >= p[1].id)
        {
            return Err("Invalid snapshot".into());
        }
        Ok(value)
    }
}

/// Bounded reliable welcome shared by both transports. Content coordinates are
/// local to this immutable anchor; snapshot origins may change independently.
pub const WELCOME_BYTES: usize = 76;
#[derive(Clone, Debug)]
pub struct Welcome {
    pub id: u64,
    pub epoch: u64,
    pub world: [u8; 32],
    pub origin: [f64; 3],
}
impl Welcome {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = VERSION.to_le_bytes().to_vec();
        bytes.extend(self.id.to_le_bytes());
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.world);
        for axis in self.origin {
            bytes.extend(axis.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8], expected_world: [u8; 32]) -> crate::Result<Self> {
        if bytes.len() != WELCOME_BYTES || u32::from_le_bytes(bytes[..4].try_into()?) != VERSION {
            return Err("Invalid welcome".into());
        }
        let id = u64::from_le_bytes(bytes[4..12].try_into()?);
        let epoch = u64::from_le_bytes(bytes[12..20].try_into()?);
        let world: [u8; 32] = bytes[20..52].try_into()?;
        let origin = std::array::from_fn(|i| {
            f64::from_le_bytes(bytes[52 + i * 8..60 + i * 8].try_into().unwrap())
        });
        if EntityId::new(id).is_none()
            || WorldEpoch::new(epoch).is_none()
            || GlobalPosition::new(origin).is_none()
            || world != expected_world
        {
            return Err("Invalid welcome identity, origin or server world mismatch".into());
        }
        Ok(Self {
            id,
            epoch,
            world,
            origin,
        })
    }
    pub fn frame(&self) -> LocalFrame {
        LocalFrame::new(GlobalPosition::new(self.origin).expect("Validated welcome origin"))
    }
    pub fn accepts(&self, snapshot: &Snapshot) -> bool {
        snapshot.epoch == self.epoch
            && (if self.world == crate::procedural::IDENTITY {
                snapshot.players.iter().all(|p| {
                    snapshot
                        .global_position(p)
                        .is_some_and(|g| crate::procedural::owner(g).is_some())
                })
            } else {
                snapshot.projected(self.frame()).is_some()
            })
            && snapshot.players.iter().any(|p| p.id == self.id)
    }
}

pub fn encode(value: &impl Serialize) -> crate::Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_BYTES {
        return Err("Datagram exceeds protocol budget".into());
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> crate::Result<T> {
    if bytes.len() > MAX_BYTES {
        return Err("Oversize datagram".into());
    }
    Ok(serde_json::from_slice(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_events_bind_type_and_bound_offer_epoch_generation_and_set() {
        let offer = ChunkOffer {
            version: VERSION,
            epoch: 7,
            generation: 2,
            chunks: vec![[0, 0], [1, 0]],
        };
        let bytes = encode(&ChunkEvent::Offer(offer.clone())).unwrap();
        assert!(matches!(ChunkEvent::decode(&bytes).unwrap(),ChunkEvent::Offer(v) if v==offer));
        for (field, value) in [
            ("generation", serde_json::json!(0)),
            ("epoch", serde_json::json!(0)),
            ("version", serde_json::json!(VERSION - 1)),
            ("chunks", serde_json::json!([])),
            ("chunks", serde_json::json!([[1, 0], [0, 0]])),
            ("actor", serde_json::json!(4)),
        ] {
            let mut bad = serde_json::to_value(ChunkEvent::Offer(offer.clone())).unwrap();
            bad["payload"][field] = value;
            assert!(ChunkEvent::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
        assert!(ChunkEvent::decode(&encode(&offer).unwrap()).is_err());
        assert!(ChunkEvent::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }

    #[test]
    fn chunk_ack_rejects_malformed_and_binds_report_and_application_tick() {
        let report = ChunkReady {
            version: VERSION,
            epoch: 7,
            sequence: 2,
            chunks: vec![[-1, 0], [0, 0]],
        };
        let ack = ChunkAck::new(&report, 10);
        let bytes = encode(&ack).unwrap();
        assert_eq!(ChunkAck::decode(&bytes).unwrap(), ack);
        assert!(ack.matches(&report));
        for len in 0..bytes.len() {
            assert!(ChunkAck::decode(&bytes[..len]).is_err());
        }
        for (field, value) in [
            ("version", serde_json::json!(VERSION - 1)),
            ("epoch", serde_json::json!(0)),
            ("sequence", serde_json::json!(0)),
            ("tick", serde_json::json!(0)),
            ("chunks", serde_json::json!([[0, 0], [0, 0]])),
            ("chunks", serde_json::json!([[1, 0], [0, 0]])),
            (
                "chunks",
                serde_json::json!(
                    (0..=crate::chunks::MAX_RESIDENT)
                        .map(|x| [x as i64, 0])
                        .collect::<Vec<_>>()
                ),
            ),
            ("actor", serde_json::json!(9)),
        ] {
            let mut bad = serde_json::to_value(&ack).unwrap();
            bad[field] = value;
            assert!(
                ChunkAck::decode(&serde_json::to_vec(&bad).unwrap()).is_err(),
                "{bad}"
            );
        }
        let mut changed = report.clone();
        changed.sequence += 1;
        assert!(!ack.matches(&changed));
        changed = report.clone();
        changed.epoch += 1;
        assert!(!ack.matches(&changed));
        changed = report;
        changed.chunks.clear();
        assert!(!ack.matches(&changed));
        assert!(ChunkAck::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }

    #[test]
    fn chunk_readiness_is_bounded_canonical_and_cannot_address_another_actor() {
        let valid = ChunkReady {
            version: VERSION,
            epoch: 1,
            sequence: 1,
            chunks: vec![[-1, 0], [0, 0]],
        };
        let bytes = encode(&valid).unwrap();
        assert_eq!(ChunkReady::decode(&bytes).unwrap().chunks, valid.chunks);
        for len in 0..bytes.len() {
            assert!(ChunkReady::decode(&bytes[..len]).is_err());
        }
        for (field, value) in [
            ("version", serde_json::json!(VERSION - 1)),
            ("epoch", serde_json::json!(0)),
            ("sequence", serde_json::json!(0)),
            ("chunks", serde_json::json!([[0, 0], [0, 0]])),
            ("chunks", serde_json::json!([[1, 0], [0, 0]])),
            (
                "chunks",
                serde_json::json!(
                    (0..=crate::chunks::MAX_RESIDENT)
                        .map(|x| [x as i64, 0])
                        .collect::<Vec<_>>()
                ),
            ),
            ("actor", serde_json::json!(2)),
        ] {
            let mut bad = serde_json::to_value(&valid).unwrap();
            bad[field] = value;
            assert!(
                ChunkReady::decode(&serde_json::to_vec(&bad).unwrap()).is_err(),
                "{bad}"
            );
        }
        let empty = ChunkReady {
            chunks: vec![],
            ..valid
        };
        assert!(ChunkReady::decode(&encode(&empty).unwrap()).is_ok());
        assert!(ChunkReady::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }

    #[test]
    fn input_rejects_truncation_unknown_fields_versions_and_invalid_axes() {
        let input = Input {
            version: VERSION,
            epoch: 1,
            sequence: 1,
            intent: Intent::default(),
        };
        let bytes = encode(&input).unwrap();
        assert_eq!(Input::decode(&bytes).unwrap().sequence, 1);
        for len in 0..bytes.len() {
            assert!(Input::decode(&bytes[..len]).is_err());
        }
        for (field, value) in [
            ("version", serde_json::json!(VERSION - 1)),
            ("sequence", serde_json::json!(0)),
            ("position", serde_json::json!([19, 19])),
            (
                "intent",
                serde_json::json!({"axes":[1001,0],"sprint":false}),
            ),
        ] {
            let mut bad = serde_json::to_value(input).unwrap();
            bad[field] = value;
            assert!(Input::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
        assert!(Input::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }

    #[test]
    fn snapshots_are_ordered_bounded_and_fit_the_datagram_budget() {
        let mut snapshot = Snapshot {
            version: VERSION,
            epoch: u64::MAX,
            origin: [1.0e9 + 0.012345, -1.0e9 - 0.012345, 1.0e9 + 0.012345],
            tick: u64::MAX,
            tick_hz: TICK_HZ,
            players: (1..=MAX_PLAYERS as u64)
                .map(|id| Player {
                    id: u64::MAX - MAX_PLAYERS as u64 + id,
                    xz: [4095.1235, -4095.1235],
                    y: 4095.1235,
                    applied_sequence: u64::MAX,
                })
                .collect(),
        };
        let bytes = encode(&snapshot).unwrap();
        assert_eq!(Snapshot::decode(&bytes).unwrap(), snapshot);
        for tick_hz in [0, 1, 1000] {
            let mut invalid = snapshot.clone();
            invalid.tick_hz = tick_hz;
            assert!(Snapshot::decode(&encode(&invalid).unwrap()).is_err());
        }
        let mut unordered = snapshot.clone();
        unordered.players.swap(0, 1);
        assert!(Snapshot::decode(&encode(&unordered).unwrap()).is_err());
        snapshot.players[0].xz[0] = 4097.;
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
        snapshot.players[0].xz[0] = 18.;
        snapshot.players[0].y = 4097.;
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
    }
    #[test]
    fn welcome_and_snapshot_reject_invalid_identity_epoch_origin_and_trailing_bytes() {
        let welcome = Welcome {
            id: 1,
            epoch: 9,
            world: [0; 32],
            origin: [1e9 + 0.01, 0., -1e9 - 0.02],
        };
        let encoded = welcome.encode();
        assert_eq!(encoded.len(), WELCOME_BYTES);
        assert!(Welcome::decode(&encoded, [0; 32]).is_ok());
        for length in 0..encoded.len() {
            assert!(Welcome::decode(&encoded[..length], [0; 32]).is_err());
        }
        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(Welcome::decode(&trailing, [0; 32]).is_err());
        assert!(Welcome::decode(&encoded, [1; 32]).is_err());
        for position in [4, 12] {
            let mut bytes = encoded.clone();
            bytes[position..position + 8].fill(0);
            assert!(Welcome::decode(&bytes, [0; 32]).is_err());
        }
        let mut bytes = encoded;
        bytes[52..60].copy_from_slice(&f64::NAN.to_le_bytes());
        assert!(Welcome::decode(&bytes, [0; 32]).is_err());
        let mut snapshot = Snapshot {
            version: VERSION,
            epoch: 9,
            origin: welcome.origin,
            tick: 1,
            tick_hz: 30,
            players: vec![Player {
                id: 1,
                xz: [0., 0.],
                y: 0.9,
                applied_sequence: 0,
            }],
        };
        assert!(welcome.accepts(&snapshot));
        snapshot.epoch = 10;
        assert!(!welcome.accepts(&snapshot));
        snapshot.epoch = 0;
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
        snapshot.epoch = 9;
        snapshot.origin = [1e12 + 1.; 3];
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
        snapshot.origin = [1e12; 3];
        snapshot.players[0].xz = [1., 1.];
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
        snapshot.origin = [1e9 + 9000., 0., 0.];
        assert!(!welcome.accepts(&snapshot));
    }
}

#[cfg(test)]
mod capacity_tests {
    use super::*;
    #[test]
    fn thirty_two_checked_addresses_fit_actual_wrapped_control_encoding() {
        let ready = ChunkReady {
            version: VERSION,
            epoch: u64::MAX,
            sequence: u64::MAX,
            chunks: (0..crate::chunks::MAX_RESIDENT)
                .map(|n| [-7_812_500_000 + n as i64, -7_812_500_000])
                .collect(),
        };
        assert!(ready.valid());
        let raw = encode(&ready).unwrap();
        assert!(raw.len() <= 926);
        assert_eq!(ChunkReady::decode(&raw).unwrap(), ready);
        let ack = ChunkAck::new(&ready, u64::MAX);
        let raw = encode(&ChunkEvent::Ack(ack.clone())).unwrap();
        assert!(raw.len() <= 979);
        assert!(matches!(ChunkEvent::decode(&raw).unwrap(),ChunkEvent::Ack(value) if value==ack));
        let offer = ChunkOffer {
            version: VERSION,
            epoch: u64::MAX,
            generation: u64::MAX,
            chunks: ready.chunks.clone(),
        };
        let raw = encode(&ChunkEvent::Offer(offer.clone())).unwrap();
        assert!(raw.len() <= 955);
        assert!(
            matches!(ChunkEvent::decode(&raw).unwrap(),ChunkEvent::Offer(value) if value==offer)
        );
        let mut excess = ready.clone();
        excess.chunks.push([
            -7_812_500_000 + crate::chunks::MAX_RESIDENT as i64,
            -7_812_500_000,
        ]);
        assert!(!excess.valid());
        assert!(ChunkReady::decode(&encode(&excess).unwrap()).is_err());
        for x in [i64::MIN, -7_812_500_001, 7_812_500_001, i64::MAX] {
            let bad = ChunkReady {
                chunks: vec![[x, 0]],
                ..ready.clone()
            };
            assert!(ChunkReady::decode(&encode(&bad).unwrap()).is_err());
            assert!(
                ChunkEvent::decode(&encode(&ChunkEvent::Ack(ChunkAck::new(&bad, 1))).unwrap())
                    .is_err()
            );
            let bad_offer = ChunkOffer {
                chunks: bad.chunks,
                ..offer.clone()
            };
            assert!(ChunkEvent::decode(&encode(&ChunkEvent::Offer(bad_offer)).unwrap()).is_err());
        }
    }
}
