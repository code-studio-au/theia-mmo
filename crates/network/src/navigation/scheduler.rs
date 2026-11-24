//! Bounded per-creature mailboxes sharing one existing path worker.
//! Owner-pump dispatch never searches or waits. Fixed ticks only submit/take;
//! stale actor lives, revisions and deadlines cannot publish replacement routes.
use super::worker::{Reply, Request, SubmitError, Worker};
use theia_core::identity::EntityKey;

pub const MAX_REQUESTERS: usize = crate::targeting::MAX_TARGETS;
struct Slot {
    creature: EntityKey,
    revision: u64,
    queued: Option<Request>,
    ready: Option<Reply>,
}
pub struct Scheduler {
    worker: Worker,
    slots: Vec<Slot>,
    cursor: usize,
    last_tick: Option<u64>,
    active_retired: bool,
}
impl Scheduler {
    pub fn new(worker: Worker) -> Self {
        Self {
            worker,
            slots: Vec::with_capacity(MAX_REQUESTERS),
            cursor: 0,
            last_tick: None,
            active_retired: false,
        }
    }
    pub fn navigation(&self) -> [u8; 32] {
        self.worker.navigation()
    }
    pub fn registered(&self) -> usize {
        self.slots.len()
    }
    pub fn queued(&self) -> usize {
        self.slots.iter().filter(|s| s.queued.is_some()).count()
    }
    /// Keep at most the latest query and one reply per exact creature life.
    /// Replacing a goal does not jump ahead of other creatures in dispatch order.
    pub fn submit(&mut self, request: Request) -> Result<(), SubmitError> {
        if !request.valid() || self.last_tick.is_some_and(|now| request.expires < now) {
            return Err(SubmitError::Invalid);
        }
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|s| s.creature == request.tag.creature)
        {
            if request.tag.revision <= slot.revision {
                return Err(SubmitError::Invalid);
            }
            slot.revision = request.tag.revision;
            slot.queued = Some(request);
            slot.ready = None;
        } else {
            if self.slots.len() == MAX_REQUESTERS {
                return Err(SubmitError::Busy);
            }
            self.slots.push(Slot {
                creature: request.tag.creature,
                revision: request.tag.revision,
                queued: Some(request),
                ready: None,
            });
        }
        Ok(())
    }
    /// A computing request is not interrupted. Its eventual result is discarded.
    pub fn retire(&mut self, creature: EntityKey) {
        if self
            .worker
            .pending()
            .is_some_and(|r| r.tag.creature == creature)
        {
            self.active_retired = true;
        }
        if let Some(index) = self.slots.iter().position(|slot| slot.creature == creature) {
            self.slots.remove(index);
            if index < self.cursor {
                self.cursor -= 1;
            }
        }
        self.cursor %= self.slots.len().max(1);
    }
    pub fn pending(&self, creature: EntityKey) -> Option<Request> {
        let slot = self.slots.iter().find(|s| s.creature == creature)?;
        slot.queued
            .or_else(|| slot.ready.as_ref().map(|r| r.request))
            .or_else(|| {
                self.worker
                    .pending()
                    .filter(|r| r.tag.creature == creature && r.tag.revision == slot.revision)
            })
    }
    /// Call from the owner pump, not a simulation-step search or blocking wait.
    pub fn pump(&mut self, now: u64) -> Result<(), SubmitError> {
        if self.last_tick.is_some_and(|previous| now < previous) {
            return Err(SubmitError::Invalid);
        }
        self.last_tick = Some(now);
        if let Some(reply) = self.worker.poll()?
            && !self.active_retired
            && let Some(slot) = self
                .slots
                .iter_mut()
                .find(|s| s.creature == reply.request.tag.creature)
            && slot.revision == reply.request.tag.revision
            && (reply.request.submitted..=reply.request.expires).contains(&now)
        {
            slot.ready = Some(reply);
        }
        for slot in &mut self.slots {
            if slot.queued.is_some_and(|r| now > r.expires) {
                slot.queued = None;
            }
            if slot.ready.as_ref().is_some_and(|r| now > r.request.expires) {
                slot.ready = None;
            }
        }
        if self.worker.pending().is_some() {
            return Ok(());
        }
        for offset in 0..self.slots.len() {
            let index = (self.cursor + offset) % self.slots.len();
            if let Some(request) = self.slots[index].queued.filter(|r| r.submitted <= now) {
                self.worker.submit(request)?;
                self.active_retired = false;
                self.slots[index].queued = None;
                self.cursor = (index + 1) % self.slots.len();
                break;
            }
        }
        Ok(())
    }
    pub fn take_reply(&mut self, creature: EntityKey, now: u64) -> Option<Reply> {
        if self.last_tick.is_some_and(|previous| now < previous) {
            return None;
        }
        let slot = self.slots.iter_mut().find(|s| s.creature == creature)?;
        slot.ready
            .take()
            .filter(|reply| (reply.request.submitted..=reply.request.expires).contains(&now))
    }
}
