//! Shared one-shot dodge consumption for future authority and prediction users.
//! This is not installed by current World/transport/client modes. The caller
//! supplies current owner/life, completed ticks and authoritative gate facts.
use crate::Result;
use std::sync::Arc;
use theia_core::{
    dodge::{self, Gate, Rejection},
    time::TickRate,
};
#[derive(Clone, Debug)]
pub struct Profile {
    policy: Arc<dodge::Profile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let policy = Arc::new(dodge::Profile::from_json(bytes)?);
        let mut bound = b"theia-dodge-policy-v1\0".to_vec();
        bound.extend(bytes);
        Ok(Self {
            policy,
            hash: crate::terrain::sha256(&bound),
        })
    }
    pub fn policy(&self) -> &dodge::Profile {
        &self.policy
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
pub fn profile_identity(world: [u8; 32], profile: &Profile) -> [u8; 32] {
    let mut bytes = b"theia-dodge-world-v1\0".to_vec();
    bytes.extend(world);
    bytes.extend(profile.hash);
    crate::terrain::sha256(&bytes)
}
/// A bounded action edge, repeated until acknowledged/expired by the sender.
/// No client duration, cooldown, position, speed or collision result is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub nonce: u64,
    pub axes: [i16; 2],
}
impl Request {
    pub fn valid(self) -> bool {
        self.nonce != 0 && dodge::Direction::from_axes(self.axes).is_some()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub last_nonce: u64,
    pub policy: dodge::Snapshot,
}
impl Snapshot {
    pub fn valid_at(self, profile: &Profile, rate: TickRate, tick: u64) -> bool {
        self.policy.tick == tick
            && (self.last_nonce != 0 || (self.policy.ready_at == 0 && self.policy.active.is_none()))
            && dodge::State::restore(profile.policy(), rate, self.policy).is_ok()
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Movement {
    /// Duplicate simulation call: do not run even ordinary movement/gravity.
    None,
    Ordinary,
    /// None holds the accepted start tick; Some is one swept proposal.
    Dodge(Option<dodge::Step>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    Duplicate,
    Started,
    Refused(Rejection),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Advanced {
    pub movement: Movement,
    pub outcome: Outcome,
    pub retired: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Controller {
    binding: [u8; 32],
    rate: TickRate,
    last_nonce: u64,
    policy: dodge::State,
}
impl Controller {
    pub fn new(profile: &Profile, rate: TickRate, tick: u64) -> Self {
        let mut policy = dodge::State::default();
        policy
            .advance(tick)
            .expect("Empty policy only anchors time");
        Self {
            binding: profile.hash,
            rate,
            last_nonce: 0,
            policy,
        }
    }
    pub fn restore(
        profile: &Profile,
        rate: TickRate,
        tick: u64,
        snapshot: Snapshot,
    ) -> Result<Self> {
        if !snapshot.valid_at(profile, rate, tick) {
            return Err("Invalid dodge correction state".into());
        }
        Ok(Self {
            binding: profile.hash,
            rate,
            last_nonce: snapshot.last_nonce,
            policy: dodge::State::restore(profile.policy(), rate, snapshot.policy)
                .map_err(|_| "Invalid dodge policy correction")?,
        })
    }
    pub fn snapshot(self) -> Snapshot {
        Snapshot {
            last_nonce: self.last_nonce,
            policy: self
                .policy
                .snapshot()
                .expect("Controller always has an anchored policy"),
        }
    }
    pub fn cancel(&mut self) -> bool {
        self.policy.cancel()
    }
    pub fn advance(
        &mut self,
        profile: &Profile,
        rate: TickRate,
        tick: u64,
        request: Option<Request>,
        gate: Gate,
    ) -> Result<Advanced> {
        if self.binding != profile.hash || self.rate != rate {
            return Err("Changed live dodge profile or rate".into());
        }
        if request.is_some_and(|r| !r.valid()) {
            return Err("Invalid dodge request".into());
        }
        let previous = self.snapshot().policy.tick;
        if tick < previous {
            return Err("Regressed dodge simulation tick".into());
        }
        if tick == previous {
            return Ok(Advanced {
                movement: Movement::None,
                outcome: Outcome::Duplicate,
                retired: false,
            });
        }
        let mut retired = if !gate.alive || !gate.terrain_ready {
            self.policy.cancel()
        } else {
            false
        };
        let outcome = if let Some(request) = request {
            if request.nonce <= self.last_nonce {
                Outcome::Duplicate
            } else {
                // A refused edge is still consumed. Holding/retransmitting it
                // must not start a surprise dodge after landing or cooldown.
                self.last_nonce = request.nonce;
                match self.policy.start(
                    profile.policy(),
                    rate,
                    tick,
                    dodge::Direction::from_axes(request.axes).expect("Validated axes"),
                    gate,
                ) {
                    Ok(()) => Outcome::Started,
                    Err(reason) => Outcome::Refused(reason),
                }
            }
        } else {
            Outcome::None
        };
        let active = self.policy.active();
        let step = match self.policy.advance(tick) {
            Ok(step) => step,
            Err(Rejection::SkippedTick) => {
                retired = true;
                None
            }
            Err(_) => return Err("Invalid dodge policy clock".into()),
        };
        let movement = if active && !retired {
            Movement::Dodge(step)
        } else {
            Movement::Ordinary
        };
        Ok(Advanced {
            movement,
            outcome,
            retired,
        })
    }
}
