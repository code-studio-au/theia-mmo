//! Bounded, life-bound combat membership prerequisite. Only authority may supply
//! hostile attempts/encounter observations. Selection and swing intent are not
//! membership inputs. This module grants no permissions or threat/credit rights.
use crate::{content::ContentId, identity::EntityKey, player_life::LifeKey, time::TickRate};
use serde::Deserialize;
use std::time::Duration;

pub const MAX_PROFILE_BYTES: usize = 4096;
pub const MAX_OPPONENTS: usize = 32;
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    ruleset: String,
    maximum_opponents: usize,
    hostility_grace_ms: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    maximum_opponents: usize,
    hostility_grace_ms: u32,
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Combat membership profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid membership profile")?;
        if raw.schema != 1
            || raw.ruleset != "combat_membership_fixture_v1"
            || !(1..=MAX_OPPONENTS).contains(&raw.maximum_opponents)
            || !(100..=60_000).contains(&raw.hostility_grace_ms)
        {
            return Err("Unsupported membership policy or bounds");
        }
        Ok(Self {
            id: ContentId::new(&raw.id).ok_or("Invalid membership profile ID")?,
            maximum_opponents: raw.maximum_opponents,
            hostility_grace_ms: raw.hostility_grace_ms,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn maximum_opponents(&self) -> usize {
        self.maximum_opponents
    }
    pub fn hostility_grace_ms(&self) -> u32 {
        self.hostility_grace_ms
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hostility {
    Outgoing,
    Incoming,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    StaleLife,
    LifeEnded,
    StaleTick,
    InvalidOpponent,
    Capacity,
    TimeExhausted,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub opponents: usize,
    pub outgoing_hostility: bool,
    pub incoming_hostility: bool,
    pub active_encounter: bool,
}
impl Summary {
    pub fn in_combat(self) -> bool {
        self.opponents > 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Contact {
    opponent: EntityKey,
    outgoing_until: Option<u64>,
    incoming_until: Option<u64>,
    encounter: bool,
}
impl Contact {
    fn fresh(opponent: EntityKey) -> Self {
        Self {
            opponent,
            outgoing_until: None,
            incoming_until: None,
            encounter: false,
        }
    }
    fn live(self, now: u64) -> bool {
        self.encounter
            || self.outgoing_until.is_some_and(|until| now < until)
            || self.incoming_until.is_some_and(|until| now < until)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    owner: LifeKey,
    profile: Profile,
    grace_ticks: u64,
    observed: Option<u64>,
    ended: bool,
    contacts: Vec<Contact>,
}
impl State {
    /// A replacement player life requires a fresh State. Opponent entity keys
    /// must denote authoritative lifetimes: an NPC respawn gets a fresh key.
    /// Player opponents require an adapter that explicitly retires old lives;
    /// this prerequisite does not infer PvP life identity from a session key.
    pub fn new(profile: Profile, rate: TickRate, owner: LifeKey) -> Self {
        let grace_ticks = rate
            .ticks_for(Duration::from_millis(u64::from(profile.hostility_grace_ms)))
            .expect("Bounded positive membership grace");
        let contacts = Vec::with_capacity(profile.maximum_opponents);
        Self {
            owner,
            profile,
            grace_ticks,
            observed: None,
            ended: false,
            contacts,
        }
    }
    pub fn owner(&self) -> LifeKey {
        self.owner
    }
    /// Summary at the last accepted observation. The authority must advance this
    /// state each fixed step, including steps with no combat events.
    pub fn summary(&self) -> Summary {
        Summary {
            opponents: self.contacts.len(),
            outgoing_hostility: self.contacts.iter().any(|c| c.outgoing_until.is_some()),
            incoming_hostility: self.contacts.iter().any(|c| c.incoming_until.is_some()),
            active_encounter: self.contacts.iter().any(|c| c.encounter),
        }
    }
    fn check(&self, now: u64, owner: LifeKey) -> Result<(), Error> {
        if owner != self.owner {
            return Err(Error::StaleLife);
        }
        if self.observed.is_some_and(|last| now < last) {
            return Err(Error::StaleTick);
        }
        if self.ended {
            return Err(Error::LifeEnded);
        }
        Ok(())
    }
    fn check_opponent(&self, opponent: EntityKey) -> Result<(), Error> {
        if opponent.epoch != self.owner.actor.epoch || opponent == self.owner.actor {
            return Err(Error::InvalidOpponent);
        }
        Ok(())
    }
    fn check_capacity(&self, now: u64, opponent: EntityKey) -> Result<(), Error> {
        if !self.contacts.iter().any(|c| c.opponent == opponent)
            && self.contacts.iter().filter(|c| c.live(now)).count()
                >= self.profile.maximum_opponents
        {
            return Err(Error::Capacity);
        }
        Ok(())
    }
    fn prune(&mut self, now: u64) {
        self.contacts.retain_mut(|c| {
            c.outgoing_until = c.outgoing_until.filter(|until| now < *until);
            c.incoming_until = c.incoming_until.filter(|until| now < *until);
            c.live(now)
        });
        self.observed = Some(now);
    }
    fn contact(&mut self, opponent: EntityKey) -> &mut Contact {
        if let Some(index) = self.contacts.iter().position(|c| c.opponent == opponent) {
            return &mut self.contacts[index];
        }
        debug_assert!(self.contacts.len() < self.profile.maximum_opponents);
        self.contacts.push(Contact::fresh(opponent));
        self.contacts.last_mut().expect("Inserted bounded contact")
    }
    /// An eligible resolved hostile attempt can enter combat even if avoided or
    /// zero damage. Rejected requests, target selection and autoattack intent
    /// must never call this. All refusals leave state unchanged.
    pub fn hostility(
        &mut self,
        now: u64,
        owner: LifeKey,
        opponent: EntityKey,
        direction: Hostility,
    ) -> Result<(), Error> {
        self.check(now, owner)?;
        self.check_opponent(opponent)?;
        let until = now
            .checked_add(self.grace_ticks)
            .ok_or(Error::TimeExhausted)?;
        self.check_capacity(now, opponent)?;
        self.prune(now);
        let contact = self.contact(opponent);
        match direction {
            Hostility::Outgoing => contact.outgoing_until = Some(until),
            Hostility::Incoming => contact.incoming_until = Some(until),
        }
        Ok(())
    }
    /// Independently pin combat while the NPC is actively pursuing/attacking
    /// this life. Releasing that reason preserves unexpired hostility reasons.
    pub fn encounter(
        &mut self,
        now: u64,
        owner: LifeKey,
        opponent: EntityKey,
        active: bool,
    ) -> Result<(), Error> {
        self.check(now, owner)?;
        self.check_opponent(opponent)?;
        if active {
            self.check_capacity(now, opponent)?;
        }
        self.prune(now);
        if active {
            self.contact(opponent).encounter = true;
        } else if let Some(contact) = self.contacts.iter_mut().find(|c| c.opponent == opponent) {
            contact.encounter = false;
            self.contacts.retain(|c| c.live(now));
        }
        Ok(())
    }
    /// Authority-confirmed encounter resolution, evade, NPC retirement or
    /// disconnect releases all reasons for this exact opponent lifetime.
    pub fn forget(&mut self, now: u64, owner: LifeKey, opponent: EntityKey) -> Result<(), Error> {
        self.check(now, owner)?;
        self.check_opponent(opponent)?;
        self.prune(now);
        self.contacts.retain(|c| c.opponent != opponent);
        Ok(())
    }
    pub fn advance(&mut self, now: u64, owner: LifeKey) -> Result<Summary, Error> {
        self.check(now, owner)?;
        self.prune(now);
        Ok(self.summary())
    }
    pub fn end_life(&mut self, now: u64, owner: LifeKey) -> Result<(), Error> {
        self.check(now, owner)?;
        self.contacts.clear();
        self.ended = true;
        self.observed = Some(now);
        Ok(())
    }
}
