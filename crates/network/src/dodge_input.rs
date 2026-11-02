//! Bounded client-side action edge. Authority still decides every acceptance.
use crate::{
    dodging::{self, Profile, Request},
    prediction::Correction,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use theia_core::identity::EntityKey;
pub const EDGE_LIFETIME: Duration = Duration::from_millis(200);
#[derive(Clone, Copy, PartialEq, Eq)]
struct Context {
    actor: EntityKey,
    stream: u64,
}
pub struct Input {
    world: [u8; 32],
    profile: Arc<Profile>,
    context: Option<Context>,
    observed_tick: u64,
    highest_nonce: u64,
    pending: Option<(Request, Instant)>,
}
impl Input {
    pub fn new(source_world: [u8; 32], profile: Arc<Profile>) -> Self {
        Self {
            world: dodging::profile_identity(source_world, &profile),
            profile,
            context: None,
            observed_tick: 0,
            highest_nonce: 0,
            pending: None,
        }
    }
    /// Call once per input frame. `allowed` includes focus, panels, current life,
    /// connection and prepared movement permission. A context change discards
    /// even a simultaneous key edge; resuming never replays an earlier tap.
    pub fn update(
        &mut self,
        correction: Option<Correction>,
        allowed: bool,
        pressed: Option<[i16; 2]>,
        now: Instant,
    ) -> Result<Option<Request>, &'static str> {
        let Some(c) = correction else {
            self.context = None;
            self.pending = None;
            return Ok(None);
        };
        let Some(d) = c.pose.dodge.filter(|d| {
            c.valid() && c.world == self.world && d.valid_at(&self.profile, c.rate, c.input.tick)
        }) else {
            self.pending = None;
            self.context = None;
            return Err("Unbound dodge input correction");
        };
        let context = Context {
            actor: c.actor,
            stream: c.input.stream_epoch,
        };
        let changed = self.context != Some(context);
        if !changed && c.input.tick < self.observed_tick {
            self.pending = None;
            return Err("Regressed dodge input correction");
        }
        self.highest_nonce = self.highest_nonce.max(d.last_nonce);
        self.observed_tick = c.input.tick;
        self.context = Some(context);
        if changed || !allowed {
            self.pending = None;
            return Ok(None);
        }
        if self.pending.is_some_and(|(r, at)| {
            d.last_nonce >= r.nonce || now.saturating_duration_since(at) >= EDGE_LIFETIME
        }) {
            self.pending = None;
        }
        if let Some(axes) = pressed
            && self.pending.is_none()
            && d.policy.active.is_none()
            && c.pose.motion.grounded
            && c.input.tick >= d.policy.ready_at
        {
            let nonce = self
                .highest_nonce
                .checked_add(1)
                .ok_or("Dodge input nonce exhausted")?;
            let request = Request { nonce, axes };
            if !request.valid() {
                return Err("Invalid dodge input direction");
            }
            self.highest_nonce = nonce;
            self.pending = Some((request, now));
        }
        Ok(self.pending.map(|(r, _)| r))
    }
}
