//! Combat profile 2: catalog-bound ordinals, bounded reliable state and events.
use crate::{
    Result,
    simulation::combat::{self, Cast, Event, Outcome, Queued, Reason, Reply},
};
use theia_core::{
    abilities::Catalog,
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    resources::Vitals,
};
pub const REQUEST_TAG: &[u8; 4] = b"CA01";
pub const STATE_TAG: &[u8; 4] = b"CS01";
pub const RESOURCE_STATE_TAG: &[u8; 4] = b"CS02";
pub const EVENTS_TAG: &[u8; 4] = b"CE02";
pub const REQUEST_BYTES: usize = 29;
pub const MAX_BYTES: usize = 1200;
pub const MAX_BATCH_EVENTS: usize = 16;
const MAX_DEADLINE: u64 = 3600;
fn index(catalog: &Catalog, id: &ContentId) -> Result<u8> {
    catalog
        .abilities()
        .iter()
        .position(|a| a.id() == id)
        .map(|n| n as u8)
        .ok_or_else(|| "Unknown catalog ability".into())
}
fn ability(catalog: &Catalog, index: u8) -> Result<ContentId> {
    catalog
        .abilities()
        .get(usize::from(index))
        .map(|a| a.id().clone())
        .ok_or_else(|| "Invalid catalog ordinal".into())
}
fn key(epoch: u64, entity: u64) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(epoch).ok_or("Invalid combat epoch")?,
        entity: EntityId::new(entity).ok_or("Invalid combat entity")?,
    })
}
fn future(at: u64, tick: u64) -> bool {
    at > tick && at - tick <= MAX_DEADLINE
}
fn reason(value: u8) -> Result<Reason> {
    Ok(match value {
        0 => Reason::UnknownAbility,
        1 => Reason::UnknownTarget,
        2 => Reason::Dead,
        3 => Reason::OutOfRange,
        4 => Reason::NotReady,
        5 => Reason::Obstructed,
        6 => Reason::Moving,
        7 => Reason::Interrupted,
        8 => Reason::Unavailable,
        9 => Reason::Superseded,
        10 => Reason::TimeExhausted,
        11 => Reason::InsufficientMana,
        12 => Reason::StatusCapacity,
        _ => return Err("Unknown combat rejection reason".into()),
    })
}
fn reason_id(value: Reason) -> u8 {
    match value {
        Reason::UnknownAbility => 0,
        Reason::UnknownTarget => 1,
        Reason::Dead => 2,
        Reason::OutOfRange => 3,
        Reason::NotReady => 4,
        Reason::Obstructed => 5,
        Reason::Moving => 6,
        Reason::Interrupted => 7,
        Reason::Unavailable => 8,
        Reason::Superseded => 9,
        Reason::TimeExhausted => 10,
        Reason::InsufficientMana => 11,
        Reason::StatusCapacity => 12,
    }
}
struct Read<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Read<'a> {
    fn new(bytes: &'a [u8], tag: &[u8; 4]) -> Result<Self> {
        if bytes.len() > MAX_BYTES || !bytes.starts_with(tag) {
            return Err("Invalid combat frame tag/budget".into());
        }
        Ok(Self { bytes, at: 4 })
    }
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let next = self.at.checked_add(N).ok_or("Combat length overflow")?;
        let bytes = self
            .bytes
            .get(self.at..next)
            .ok_or("Truncated combat frame")?
            .try_into()?;
        self.at = next;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take::<1>()?[0])
    }
    fn word(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn number(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn end(&self) -> Result<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err("Trailing combat bytes".into())
        }
    }
}
fn word(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend(value.to_le_bytes());
}
fn number(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend(value.to_le_bytes());
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub epoch: u64,
    pub sequence: u64,
    pub ability: u8,
    pub target: EntityId,
}
impl Request {
    pub fn new(
        epoch: u64,
        sequence: u64,
        id: &ContentId,
        target: EntityId,
        catalog: &Catalog,
    ) -> Result<Self> {
        let value = Self {
            epoch,
            sequence,
            ability: index(catalog, id)?,
            target,
        };
        value.command(catalog)?;
        Ok(value)
    }
    pub fn command(self, catalog: &Catalog) -> Result<combat::Request> {
        if self.sequence == 0 {
            return Err("Zero combat request sequence".into());
        }
        Ok(combat::Request {
            epoch: self.epoch,
            sequence: self.sequence,
            ability: ability(catalog, self.ability)?,
            target: key(self.epoch, self.target.value())?,
        })
    }
    pub fn encode(self) -> Result<Vec<u8>> {
        if WorldEpoch::new(self.epoch).is_none() || self.sequence == 0 || self.ability >= 64 {
            return Err("Invalid combat request".into());
        }
        let mut bytes = REQUEST_TAG.to_vec();
        word(&mut bytes, self.epoch);
        word(&mut bytes, self.sequence);
        bytes.push(self.ability);
        word(&mut bytes, self.target.value());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES {
            return Err("Invalid combat request length".into());
        }
        let mut read = Read::new(bytes, REQUEST_TAG)?;
        let value = Self {
            epoch: read.word()?,
            sequence: read.word()?,
            ability: read.byte()?,
            target: EntityId::new(read.word()?).ok_or("Invalid cast target")?,
        };
        read.end()?;
        value.command(catalog)?;
        Ok(value)
    }
}
fn put_cast(bytes: &mut Vec<u8>, cast: &Cast, catalog: &Catalog) -> Result<()> {
    word(bytes, cast.sequence);
    bytes.push(index(catalog, &cast.ability)?);
    word(bytes, cast.target.entity.value());
    word(bytes, cast.started);
    word(bytes, cast.completes);
    Ok(())
}
fn get_cast(read: &mut Read<'_>, epoch: u64, catalog: &Catalog) -> Result<Cast> {
    Ok(Cast {
        sequence: read.word()?,
        ability: ability(catalog, read.byte()?)?,
        target: key(epoch, read.word()?)?,
        started: read.word()?,
        completes: read.word()?,
    })
}
fn valid_cast(cast: &Cast, epoch: u64, tick: u64, catalog: &Catalog) -> bool {
    cast.target.epoch.value() == epoch
        && cast.sequence > 0
        && cast.started <= tick
        && future(cast.completes, tick)
        && cast.completes - cast.started <= MAX_DEADLINE
        && index(catalog, &cast.ability).is_ok()
}
fn put_queue(bytes: &mut Vec<u8>, queued: &Queued, catalog: &Catalog) -> Result<()> {
    word(bytes, queued.request.sequence);
    bytes.push(index(catalog, &queued.request.ability)?);
    word(bytes, queued.request.target.entity.value());
    word(bytes, queued.at);
    Ok(())
}
fn get_queue(read: &mut Read<'_>, epoch: u64, catalog: &Catalog) -> Result<Queued> {
    Ok(Queued {
        request: combat::Request {
            epoch,
            sequence: read.word()?,
            ability: ability(catalog, read.byte()?)?,
            target: key(epoch, read.word()?)?,
        },
        at: read.word()?,
    })
}
fn valid_queue(queued: &Queued, epoch: u64, tick: u64, catalog: &Catalog) -> bool {
    queued.request.epoch == epoch
        && queued.request.target.epoch.value() == epoch
        && queued.request.sequence > 0
        && future(queued.at, tick)
        && index(catalog, &queued.request.ability).is_ok()
}
fn put_reply(bytes: &mut Vec<u8>, reply: Reply) -> Result<()> {
    word(bytes, reply.sequence);
    let (tag, payload) = match reply.outcome {
        Outcome::Ignored => return Err("Ignored requests have no authoritative reply".into()),
        Outcome::Rejected(reason) => (0, u64::from(reason_id(reason))),
        Outcome::Started { completes } => (1, completes),
        Outcome::Queued { at } => (2, at),
    };
    bytes.push(tag);
    word(bytes, payload);
    Ok(())
}
fn get_reply(read: &mut Read<'_>) -> Result<Reply> {
    let sequence = read.word()?;
    let tag = read.byte()?;
    let value = read.word()?;
    Ok(Reply {
        sequence,
        outcome: match tag {
            0 => Outcome::Rejected(reason(u8::try_from(value)?)?),
            1 => Outcome::Started { completes: value },
            2 => Outcome::Queued { at: value },
            _ => return Err("Invalid combat reply".into()),
        },
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Health {
    pub entity: EntityId,
    pub current: u32,
    pub maximum: u32,
    pub respawn_at: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub vitals: Option<Vitals>,
    pub epoch: u64,
    pub tick: u64,
    pub reply: Option<Reply>,
    pub gcd_until: u64,
    pub cast: Option<Cast>,
    pub queued: Option<Queued>,
    pub cooldowns: Vec<(ContentId, u64)>,
    pub targets: Vec<Health>,
}
impl State {
    pub fn new(epoch: u64, snapshot: combat::Snapshot, catalog: &Catalog) -> Result<Self> {
        if snapshot
            .targets
            .iter()
            .any(|t| t.key.epoch.value() != epoch)
        {
            return Err("Foreign combat health incarnation".into());
        }
        let mut cooldowns = snapshot.cooldowns;
        cooldowns.sort_by_key(|(id, _)| index(catalog, id).unwrap_or(u8::MAX));
        let mut targets: Vec<_> = snapshot
            .targets
            .into_iter()
            .map(|t| Health {
                entity: t.key.entity,
                current: t.current,
                maximum: t.maximum,
                respawn_at: t.respawn_at,
            })
            .collect();
        targets.sort_by_key(|t| t.entity);
        let value = Self {
            vitals: snapshot.vitals,
            epoch,
            tick: snapshot.tick,
            reply: snapshot.reply,
            gcd_until: snapshot.gcd_until,
            cast: snapshot.cast,
            queued: snapshot.queued,
            cooldowns,
            targets,
        };
        value.validate(catalog)?;
        Ok(value)
    }
    pub fn same_view(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.vitals == other.vitals
            && self.reply == other.reply
            && self.gcd_until == other.gcd_until
            && self.cast == other.cast
            && self.queued == other.queued
            && self.cooldowns == other.cooldowns
            && self.targets == other.targets
    }
    fn validate(&self, catalog: &Catalog) -> Result<()> {
        if self.vitals.is_none()
            && self
                .reply
                .is_some_and(|r| r.outcome == Outcome::Rejected(Reason::InsufficientMana))
        {
            return Err("Mana rejection requires the resource state profile".into());
        }
        if WorldEpoch::new(self.epoch).is_none()
            || self.targets.len() > 16
            || self.cooldowns.len() > 64
            || self.gcd_until.saturating_sub(self.tick) > MAX_DEADLINE
        {
            return Err("Invalid combat state budget/epoch/deadline".into());
        }
        if let Some(reply) = self.reply {
            if reply.sequence == 0 || matches!(reply.outcome, Outcome::Ignored) {
                return Err("Invalid combat state reply".into());
            }
            let deadline = match reply.outcome {
                Outcome::Started { completes } => completes,
                Outcome::Queued { at } => at,
                _ => 0,
            };
            if deadline == 0
                && matches!(
                    reply.outcome,
                    Outcome::Started { .. } | Outcome::Queued { .. }
                )
            {
                return Err("Zero cast reply deadline".into());
            }
            if deadline.saturating_sub(self.tick) > MAX_DEADLINE {
                return Err("Unbounded reply deadline".into());
            }
        }
        let sequence = self.reply.map_or(0, |r| r.sequence);
        if self.cast.as_ref().is_some_and(|c| {
            c.sequence > sequence || !valid_cast(c, self.epoch, self.tick, catalog)
        }) || self.queued.as_ref().is_some_and(|q| {
            q.request.sequence > sequence || !valid_queue(q, self.epoch, self.tick, catalog)
        }) {
            return Err("Invalid combat cast/queue".into());
        }
        let mut previous = None;
        for (id, at) in &self.cooldowns {
            let idx = index(catalog, id)?;
            if previous.is_some_and(|old| old >= idx) || !future(*at, self.tick) {
                return Err("Invalid cooldown order/deadline".into());
            }
            previous = Some(idx);
        }
        if self.targets.windows(2).any(|p| p[0].entity >= p[1].entity)
            || self.targets.iter().any(|t| {
                t.maximum == 0
                    || t.maximum > 1_000_000
                    || t.current > t.maximum
                    || (t.current == 0) != t.respawn_at.is_some()
                    || t.respawn_at.is_some_and(|at| !future(at, self.tick))
            })
        {
            return Err("Invalid combat health/lifecycle".into());
        }
        Ok(())
    }
    pub fn encode(&self, catalog: &Catalog) -> Result<Vec<u8>> {
        self.validate(catalog)?;
        let mut bytes = if self.vitals.is_some() {
            RESOURCE_STATE_TAG.to_vec()
        } else {
            STATE_TAG.to_vec()
        };
        word(&mut bytes, self.epoch);
        word(&mut bytes, self.tick);
        bytes.push(
            u8::from(self.reply.is_some())
                | (u8::from(self.cast.is_some()) << 1)
                | (u8::from(self.queued.is_some()) << 2),
        );
        word(&mut bytes, self.gcd_until);
        if let Some(reply) = self.reply {
            put_reply(&mut bytes, reply)?;
        }
        if let Some(cast) = &self.cast {
            put_cast(&mut bytes, cast, catalog)?;
        }
        if let Some(queued) = &self.queued {
            put_queue(&mut bytes, queued, catalog)?;
        }
        bytes.push(self.cooldowns.len() as u8);
        for (id, at) in &self.cooldowns {
            bytes.push(index(catalog, id)?);
            word(&mut bytes, *at);
        }
        bytes.push(self.targets.len() as u8);
        for target in &self.targets {
            word(&mut bytes, target.entity.value());
            number(&mut bytes, target.current);
            number(&mut bytes, target.maximum);
            word(&mut bytes, target.respawn_at.unwrap_or(0));
        }
        if let Some(vitals) = self.vitals {
            for value in [
                vitals.health(),
                vitals.maximum_health(),
                vitals.mana(),
                vitals.maximum_mana(),
            ] {
                number(&mut bytes, value);
            }
        }
        if bytes.len() > MAX_BYTES {
            return Err("Combat state exceeds message budget".into());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog) -> Result<Self> {
        let resources = bytes.starts_with(RESOURCE_STATE_TAG);
        let mut read = Read::new(
            bytes,
            if resources {
                RESOURCE_STATE_TAG
            } else {
                STATE_TAG
            },
        )?;
        let epoch = read.word()?;
        let tick = read.word()?;
        let flags = read.byte()?;
        if flags & !7 != 0 {
            return Err("Unknown combat state flags".into());
        }
        let gcd_until = read.word()?;
        let reply = if flags & 1 != 0 {
            Some(get_reply(&mut read)?)
        } else {
            None
        };
        let cast = if flags & 2 != 0 {
            Some(get_cast(&mut read, epoch, catalog)?)
        } else {
            None
        };
        let queued = if flags & 4 != 0 {
            Some(get_queue(&mut read, epoch, catalog)?)
        } else {
            None
        };
        let count = usize::from(read.byte()?);
        if count > 64 {
            return Err("Cooldown budget exceeded".into());
        }
        let mut cooldowns = Vec::with_capacity(count);
        for _ in 0..count {
            cooldowns.push((ability(catalog, read.byte()?)?, read.word()?));
        }
        let count = usize::from(read.byte()?);
        if count > 16 {
            return Err("Health budget exceeded".into());
        }
        let mut targets = Vec::with_capacity(count);
        for _ in 0..count {
            let entity = EntityId::new(read.word()?).ok_or("Invalid health entity")?;
            let current = read.number()?;
            let maximum = read.number()?;
            let at = read.word()?;
            targets.push(Health {
                entity,
                current,
                maximum,
                respawn_at: (at != 0).then_some(at),
            });
        }
        let vitals = if resources {
            Some(
                Vitals::new(
                    read.number()?,
                    read.number()?,
                    read.number()?,
                    read.number()?,
                )
                .ok_or("Invalid player resource bounds")?,
            )
        } else {
            None
        };
        read.end()?;
        let value = Self {
            vitals,
            epoch,
            tick,
            reply,
            gcd_until,
            cast,
            queued,
            cooldowns,
            targets,
        };
        value.validate(catalog)?;
        Ok(value)
    }
}
fn put_point(bytes: &mut Vec<u8>, position: GlobalPosition) {
    for value in position.metres() {
        word(bytes, value.to_bits());
    }
}
fn get_point(read: &mut Read<'_>) -> Result<GlobalPosition> {
    let point = [
        f64::from_bits(read.word()?),
        f64::from_bits(read.word()?),
        f64::from_bits(read.word()?),
    ];
    GlobalPosition::new(point).ok_or_else(|| "Invalid global combat cue position".into())
}
#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    pub epoch: u64,
    pub sequence: u64,
    pub tick: u64,
    pub events: Vec<Event>,
}
impl Batch {
    fn validate(&self, catalog: &Catalog) -> Result<()> {
        if WorldEpoch::new(self.epoch).is_none()
            || self.sequence == 0
            || self.events.is_empty()
            || self.events.len() > MAX_BATCH_EVENTS
        {
            return Err("Invalid combat event batch".into());
        }
        let valid_key = |key: EntityKey| key.epoch.value() == self.epoch;
        for event in &self.events {
            let valid = match event {
                Event::Started { actor, cast, .. } => {
                    valid_key(*actor) && valid_cast(cast, self.epoch, self.tick, catalog)
                }
                Event::Queued { actor, queued } => {
                    valid_key(*actor) && valid_queue(queued, self.epoch, self.tick, catalog)
                }
                Event::Cancelled {
                    actor, sequence, ..
                } => valid_key(*actor) && *sequence > 0,
                Event::Damage {
                    actor,
                    sequence,
                    target,
                    amount,
                    remaining,
                    ability,
                    ..
                } => {
                    valid_key(*actor)
                        && valid_key(*target)
                        && *sequence > 0
                        && (1..=1_000_000).contains(amount)
                        && *remaining <= 1_000_000
                        && index(catalog, ability).is_ok()
                }
                Event::Died {
                    target, respawn_at, ..
                } => valid_key(*target) && future(*respawn_at, self.tick),
                Event::Respawned {
                    previous, target, ..
                } => valid_key(*previous) && valid_key(*target) && target.entity > previous.entity,
            };
            if !valid {
                return Err("Invalid combat event identity/payload".into());
            }
        }
        Ok(())
    }
    pub fn encode(&self, catalog: &Catalog) -> Result<Vec<u8>> {
        self.validate(catalog)?;
        let mut bytes = EVENTS_TAG.to_vec();
        word(&mut bytes, self.epoch);
        word(&mut bytes, self.sequence);
        word(&mut bytes, self.tick);
        bytes.push(self.events.len() as u8);
        for event in &self.events {
            match event {
                Event::Started {
                    actor,
                    cast,
                    position,
                } => {
                    bytes.push(0);
                    word(&mut bytes, actor.entity.value());
                    put_cast(&mut bytes, cast, catalog)?;
                    put_point(&mut bytes, *position);
                }
                Event::Queued { actor, queued } => {
                    bytes.push(1);
                    word(&mut bytes, actor.entity.value());
                    put_queue(&mut bytes, queued, catalog)?;
                }
                Event::Cancelled {
                    actor,
                    sequence,
                    reason,
                } => {
                    bytes.push(2);
                    word(&mut bytes, actor.entity.value());
                    word(&mut bytes, *sequence);
                    bytes.push(reason_id(*reason));
                }
                Event::Damage {
                    actor,
                    sequence,
                    target,
                    amount,
                    remaining,
                    ability,
                    position,
                } => {
                    bytes.push(3);
                    word(&mut bytes, actor.entity.value());
                    word(&mut bytes, *sequence);
                    word(&mut bytes, target.entity.value());
                    bytes.push(index(catalog, ability)?);
                    number(&mut bytes, *amount);
                    number(&mut bytes, *remaining);
                    put_point(&mut bytes, *position);
                }
                Event::Died {
                    target,
                    respawn_at,
                    position,
                } => {
                    bytes.push(4);
                    word(&mut bytes, target.entity.value());
                    word(&mut bytes, *respawn_at);
                    put_point(&mut bytes, *position);
                }
                Event::Respawned {
                    previous,
                    target,
                    position,
                } => {
                    bytes.push(5);
                    word(&mut bytes, previous.entity.value());
                    word(&mut bytes, target.entity.value());
                    put_point(&mut bytes, *position);
                }
            }
        }
        if bytes.len() > MAX_BYTES {
            return Err("Combat events exceed message budget".into());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog) -> Result<Self> {
        let mut read = Read::new(bytes, EVENTS_TAG)?;
        let epoch = read.word()?;
        let sequence = read.word()?;
        let tick = read.word()?;
        let count = usize::from(read.byte()?);
        if count == 0 || count > MAX_BATCH_EVENTS {
            return Err("Event count exceeds budget".into());
        }
        let mut events = Vec::with_capacity(count);
        for _ in 0..count {
            let tag = read.byte()?;
            let event = match tag {
                0 => Event::Started {
                    actor: key(epoch, read.word()?)?,
                    cast: get_cast(&mut read, epoch, catalog)?,
                    position: get_point(&mut read)?,
                },
                1 => Event::Queued {
                    actor: key(epoch, read.word()?)?,
                    queued: get_queue(&mut read, epoch, catalog)?,
                },
                2 => Event::Cancelled {
                    actor: key(epoch, read.word()?)?,
                    sequence: read.word()?,
                    reason: reason(read.byte()?)?,
                },
                3 => Event::Damage {
                    actor: key(epoch, read.word()?)?,
                    sequence: read.word()?,
                    target: key(epoch, read.word()?)?,
                    ability: ability(catalog, read.byte()?)?,
                    amount: read.number()?,
                    remaining: read.number()?,
                    position: get_point(&mut read)?,
                },
                4 => Event::Died {
                    target: key(epoch, read.word()?)?,
                    respawn_at: read.word()?,
                    position: get_point(&mut read)?,
                },
                5 => Event::Respawned {
                    previous: key(epoch, read.word()?)?,
                    target: key(epoch, read.word()?)?,
                    position: get_point(&mut read)?,
                },
                _ => return Err("Unknown combat event tag".into()),
            };
            events.push(event);
        }
        read.end()?;
        let value = Self {
            epoch,
            sequence,
            tick,
            events,
        };
        value.validate(catalog)?;
        Ok(value)
    }
}
