//! P2-04 command timing contract. Not yet selected by wire v11 applications.
//! One advance per authoritative simulation tick; packets only populate a
//! bounded future window and can never request elapsed time or extra steps.
//! The stream epoch is per actor and changes on teleports/resets. It is distinct
//! from the world incarnation; callers validate world and actor identity first.
use crate::wire::Intent;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use theia_core::time::TickRate;
pub const FUTURE_TICKS: u64 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub sequence: u64,
    pub tick: u64,
    pub intent: Intent,
}
/// Completed-step acknowledgement and held input needed to replay gaps/expiry.
/// Queued future commands are not falsely acknowledged as applied.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub stream_epoch: u64,
    pub tick: u64,
    pub applied_sequence: u64,
    pub fresh_tick: u64,
    pub intent: Intent,
}
impl State {
    /// A new stream has not observed a client release yet. Keep the body's
    /// held-jump latch through neutral bootstrap, so teleport/recovery cannot
    /// manufacture a release/repress edge before the first actual command.
    pub fn effective_intent(self, motion: crate::kinematic::MotionState) -> Intent {
        let mut intent = self.intent;
        if self.applied_sequence == 0 {
            intent.jump = motion.jump_held;
        }
        intent
    }
}
#[derive(Clone, Copy)]
struct Queued {
    command: Command,
    received_tick: u64,
}
pub struct Timeline {
    rate: TickRate,
    state: State,
    pending: BTreeMap<u64, Queued>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Receipt {
    Queued,
    Duplicate,
}
impl Timeline {
    pub fn new(stream_epoch: u64, rate: TickRate, tick: u64) -> Result<Self, &'static str> {
        Self::restore(
            rate,
            State {
                stream_epoch,
                tick,
                applied_sequence: 0,
                fresh_tick: tick,
                intent: Intent::default(),
            },
        )
    }
    pub fn restore(rate: TickRate, state: State) -> Result<Self, &'static str> {
        let timeout = rate
            .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
            .ok_or("Invalid expiry")?;
        if state.stream_epoch == 0
            || state.fresh_tick > state.tick
            || !state.intent.valid()
            || (state.applied_sequence == 0 && state.intent != Intent::default())
            || (state.tick.saturating_sub(state.fresh_tick) > timeout
                && state.intent != Intent::default())
        {
            return Err("Invalid completed input state");
        }
        Ok(Self {
            rate,
            state,
            pending: BTreeMap::new(),
        })
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn pending(&self) -> usize {
        self.pending.len()
    }
    /// Out-of-order future ticks may fill gaps. Conflicting mappings, old epochs,
    /// late/replayed sequences and over-window commands leave all state unchanged.
    pub fn offer(&mut self, stream_epoch: u64, command: Command) -> Result<Receipt, &'static str> {
        let timeout = self
            .rate
            .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
            .ok_or("Invalid expiry")?;
        if stream_epoch != self.state.stream_epoch
            || command.sequence == 0
            || !command.intent.valid()
            || command.tick <= self.state.tick
            || command.tick.saturating_sub(self.state.tick) > FUTURE_TICKS.min(timeout)
            || command.sequence <= self.state.applied_sequence
        {
            return Err("Stale, foreign or out-of-window command");
        }
        if let Some(existing) = self.pending.get(&command.tick) {
            return if existing.command == command {
                Ok(Receipt::Duplicate)
            } else {
                Err("Conflicting command tick")
            };
        }
        // Tick and sequence order must agree even when packets arrive out of
        // order. A rejected huge sequence cannot poison later valid progress.
        if self.pending.values().any(|queued| {
            let other = &queued.command;
            (other.tick < command.tick && other.sequence >= command.sequence)
                || (other.tick > command.tick && other.sequence <= command.sequence)
        }) {
            return Err("Command sequence disagrees with tick order");
        }
        if self.pending.len() >= FUTURE_TICKS as usize {
            return Err("Command window is full");
        }
        self.pending.insert(
            command.tick,
            Queued {
                command,
                received_tick: self.state.tick,
            },
        );
        Ok(Receipt::Queued)
    }
    /// Called exactly once by the fixed-step owner. No file/network I/O occurs.
    /// A missing command holds the previous intent until simulation-time expiry.
    pub fn advance(&mut self) -> Result<State, &'static str> {
        let tick = self
            .state
            .tick
            .checked_add(1)
            .ok_or("Simulation tick exhausted")?;
        let timeout = self
            .rate
            .ticks_for(std::time::Duration::from_secs_f64(1. / 3.))
            .ok_or("Invalid expiry")?;
        self.state.tick = tick;
        if let Some(queued) = self.pending.remove(&tick) {
            self.state.applied_sequence = queued.command.sequence;
            self.state.fresh_tick = queued.received_tick;
            self.state.intent = queued.command.intent;
        }
        if tick - self.state.fresh_tick > timeout {
            self.state.intent = Intent::default();
        }
        Ok(self.state)
    }
}
