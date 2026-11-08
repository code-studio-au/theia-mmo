//! Bounded client receipt prerequisite. No composer, window, network task or UI.
use crate::{
    Result,
    chat_wire::{Delivery, Reply},
};
use std::collections::VecDeque;
use theia_core::identity::EntityKey;
pub const HISTORY: usize = 64;
#[derive(Clone, Debug)]
pub struct Inbox {
    owner: EntityKey,
    world: [u8; 32],
    history: VecDeque<Delivery>,
    last_serial: u64,
    last_tick: u64,
    reply: Option<Reply>,
}
impl Inbox {
    pub fn new(owner: EntityKey, world: [u8; 32]) -> Result<Self> {
        if world == [0; 32] {
            return Err("Chat inbox requires an admitted world".into());
        }
        Ok(Self {
            owner,
            world,
            history: VecDeque::with_capacity(HISTORY),
            last_serial: 0,
            last_tick: 0,
            reply: None,
        })
    }
    /// Validate before mutation. Stale deliveries are ignored; an exact latest
    /// duplicate is harmless, while conflicting data with that serial is invalid.
    /// A server-owned world serial avoids an unbounded per-speaker replay map.
    pub fn receive(&mut self, delivery: Delivery) -> Result<bool> {
        delivery.validate_context(self.owner.epoch, self.world)?;
        if delivery.serial < self.last_serial {
            return Ok(false);
        }
        if delivery.serial == self.last_serial {
            if self.history.back() != Some(&delivery) {
                return Err("Conflicting chat delivery serial".into());
            }
            return Ok(false);
        }
        if delivery.tick < self.last_tick {
            return Err("Regressing chat delivery tick".into());
        }
        self.last_serial = delivery.serial;
        self.last_tick = delivery.tick;
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(delivery);
        Ok(true)
    }
    /// Replies are private to this admitted owner. An older/repeated request
    /// reply cannot overwrite the latest outcome (including replay refusals).
    pub fn receive_reply(&mut self, reply: Reply) -> Result<bool> {
        reply.validate_context(self.owner, self.world)?;
        if self
            .reply
            .as_ref()
            .is_some_and(|old| reply.sequence <= old.sequence)
        {
            return Ok(false);
        }
        if self.reply.as_ref().is_some_and(|old| reply.tick < old.tick) {
            return Err("Regressing chat reply tick".into());
        }
        self.reply = Some(reply);
        Ok(true)
    }
    pub fn history(&self) -> impl ExactSizeIterator<Item = &Delivery> {
        self.history.iter()
    }
    pub fn reply(&self) -> Option<&Reply> {
        self.reply.as_ref()
    }
}
/// Render-independent client view published by the real Renet worker.
#[derive(Clone, Debug)]
pub struct View {
    pub inbox: std::sync::Arc<Inbox>,
    pub submitted: u64,
    pub pending: bool,
    pub timed_out: bool,
    pub exhausted: bool,
}
#[derive(Clone, Debug)]
pub struct Compose {
    pub channel: theia_core::chat::Channel,
    pub text: theia_core::chat::Text,
}
/// Explicit opt-in mode. The source is the complete gameplay identity before
/// the chat suffix (including Classic movement if enabled).
pub struct Client {
    pub(crate) source_world: [u8; 32],
    pub(crate) area: theia_core::chat::Area,
    pub(crate) outgoing: tokio::sync::mpsc::Receiver<Compose>,
}
impl Client {
    pub fn new(
        source_world: [u8; 32],
        area: theia_core::chat::Area,
        outgoing: tokio::sync::mpsc::Receiver<Compose>,
    ) -> Result<Self> {
        if source_world == [0; 32] || outgoing.max_capacity() > 4 {
            return Err("Invalid chat client source/queue capacity".into());
        }
        Ok(Self {
            source_world,
            area,
            outgoing,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_wire::Outcome;
    use theia_core::{
        character_name::CharacterName,
        chat::{Channel, Text},
        identity::{EntityId, WorldEpoch},
    };
    fn key(id: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        }
    }
    fn delivery(serial: u64) -> Delivery {
        Delivery {
            world: [1; 32],
            sender: key(serial),
            serial,
            sequence: 1,
            tick: serial,
            channel: Channel::Area,
            name: CharacterName::parse("Arden").unwrap(),
            text: Text::parse("Hello nearby").unwrap(),
        }
    }
    #[test]
    fn long_lived_inbox_bounds_history_despite_actor_churn_without_replay_map_growth() {
        let mut inbox = Inbox::new(key(1), [1; 32]).unwrap();
        for serial in 1..=10_000 {
            assert!(
                inbox
                    .receive(Delivery::decode(&delivery(serial).encode().unwrap()).unwrap())
                    .unwrap()
            );
        }
        assert_eq!(inbox.history().len(), HISTORY);
        assert_eq!(
            inbox.history().next().unwrap().serial,
            10_001 - HISTORY as u64
        );
        assert_eq!(inbox.history.back().unwrap().serial, 10_000);
        assert!(!inbox.receive(delivery(1)).unwrap());
        assert!(!inbox.receive(delivery(10_000)).unwrap());
        assert_eq!(inbox.history().len(), HISTORY);
    }
    #[test]
    fn delivery_context_and_conflicting_latest_serial_fail_before_cache_mutation() {
        let mut inbox = Inbox::new(key(1), [1; 32]).unwrap();
        inbox.receive(delivery(5)).unwrap();
        for case in 0..6 {
            let mut bad = delivery(6);
            match case {
                0 => bad.world = [2; 32],
                1 => bad.sender.epoch = WorldEpoch::new(8).unwrap(),
                2 => bad.serial = 0,
                3 => bad.tick = 0,
                4 => bad.tick = 4,
                _ => {
                    bad = delivery(5);
                    bad.text = Text::parse("Conflicting text").unwrap();
                }
            }
            assert!(inbox.receive(bad).is_err(), "case {case}");
            assert_eq!(inbox.history.back().unwrap().serial, 5);
        }
        let mut later = delivery(8);
        later.tick = 5;
        assert!(inbox.receive(later).unwrap()); // gaps and multiple accepts/tick are legal
        assert!(Inbox::new(key(1), [0; 32]).is_err());
    }
    #[test]
    fn private_reply_context_staleness_and_clock_are_independent_of_delivery_backlog() {
        let mut inbox = Inbox::new(key(1), [1; 32]).unwrap();
        let reply = Reply {
            world: [1; 32],
            owner: key(1),
            sequence: 4,
            tick: 100,
            outcome: Outcome::RateLimited,
        };
        assert!(
            inbox
                .receive_reply(Reply::decode(&reply.encode().unwrap()).unwrap())
                .unwrap()
        );
        let mut old = reply.clone();
        old.sequence = 1;
        old.tick = 1;
        old.outcome = Outcome::Accepted;
        assert!(!inbox.receive_reply(old).unwrap());
        assert_eq!(inbox.reply().unwrap().outcome, Outcome::RateLimited);
        assert!(inbox.receive(delivery(1)).unwrap()); // older chat can follow its private reply
        for case in 0..3 {
            let mut bad = reply.clone();
            bad.sequence = 5;
            match case {
                0 => bad.owner = key(2),
                1 => bad.world = [2; 32],
                _ => bad.tick = 99,
            }
            assert!(inbox.receive_reply(bad).is_err());
        }
        let mut next = reply;
        next.sequence = 5;
        next.outcome = Outcome::Accepted;
        assert!(inbox.receive_reply(next).unwrap());
    }
}
