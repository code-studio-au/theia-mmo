//! Versioned, bounded datagrams. No client coordinates, elapsed time or player ID.
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;
pub const MAX_PLAYERS: usize = 8;
pub const MAX_BYTES: usize = 1200;
pub const TICK_HZ: u32 = 30;
pub const WORLD_LIMIT: f32 = 19.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
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
    pub sequence: u64,
    pub intent: Intent,
}
impl Input {
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        if value.version != VERSION || value.sequence == 0 || !value.intent.valid() {
            return Err("Invalid input version, sequence or axes".into());
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub id: u64,
    pub xz: [f32; 2],
    pub applied_sequence: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub tick: u64,
    pub players: Vec<Player>,
}
impl Snapshot {
    pub fn decode(bytes: &[u8]) -> crate::Result<Self> {
        let value: Self = decode(bytes)?;
        if value.version != VERSION
            || value.players.len() > MAX_PLAYERS
            || value
                .players
                .iter()
                .any(|p| p.id == 0 || p.xz.iter().any(|v| !v.is_finite() || v.abs() > WORLD_LIMIT))
            || value.players.windows(2).any(|p| p[0].id >= p[1].id)
        {
            return Err("Invalid snapshot".into());
        }
        Ok(value)
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
    fn input_rejects_truncation_unknown_fields_versions_and_invalid_axes() {
        let input = Input {
            version: VERSION,
            sequence: 1,
            intent: Intent::default(),
        };
        let bytes = encode(&input).unwrap();
        assert_eq!(Input::decode(&bytes).unwrap().sequence, 1);
        for len in 0..bytes.len() {
            assert!(Input::decode(&bytes[..len]).is_err());
        }
        for bytes in [
            br#"{"version":1,"sequence":1,"intent":{"axes":[1001,0],"sprint":false}}"#.as_slice(),
            br#"{"version":2,"sequence":1,"intent":{"axes":[0,0],"sprint":false}}"#,
            br#"{"version":1,"sequence":0,"intent":{"axes":[0,0],"sprint":false}}"#,
            br#"{"version":1,"sequence":1,"intent":{"axes":[0,0],"sprint":false},"position":[19,19]}"#,
        ] { assert!(Input::decode(bytes).is_err()); }
        assert!(Input::decode(&vec![0; MAX_BYTES + 1]).is_err());
    }

    #[test]
    fn snapshots_are_ordered_bounded_and_fit_the_datagram_budget() {
        let mut snapshot = Snapshot {
            version: VERSION,
            tick: u64::MAX,
            players: (1..=MAX_PLAYERS as u64)
                .map(|id| Player {
                    id: u64::MAX - MAX_PLAYERS as u64 + id,
                    xz: [18.123456, -18.123456],
                    applied_sequence: u64::MAX,
                })
                .collect(),
        };
        let bytes = encode(&snapshot).unwrap();
        assert_eq!(Snapshot::decode(&bytes).unwrap(), snapshot);
        snapshot.players.swap(0, 1);
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
        snapshot.players[0].xz[0] = 20.;
        assert!(Snapshot::decode(&encode(&snapshot).unwrap()).is_err());
    }
}
