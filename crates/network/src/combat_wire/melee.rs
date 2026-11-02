//! Explicit basic-melee wire profile. This codec alone does not enable transport.
use super::{Read, key, word};
use crate::{
    Result,
    simulation::combat::autoattack::{self, Outcome, Pause, Phase, Reply, Snapshot},
};
use theia_core::{content::ContentId, melee::Catalog};

pub const REQUEST_TAG: &[u8; 4] = b"MA01";
pub const STATE_TAG: &[u8; 4] = b"MS01";
pub const EVENTS_TAG: &[u8; 4] = b"CE05";
pub const REQUEST_BYTES: usize = 36;
pub const MAX_STATE_BYTES: usize = 72;

pub(super) fn weapon_index(catalog: &Catalog, id: &ContentId) -> Result<u8> {
    catalog
        .weapons()
        .position(|w| w.item() == id)
        .map(|i| i as u8)
        .ok_or_else(|| "Unknown melee weapon".into())
}
pub(super) fn weapon(catalog: &Catalog, ordinal: u8) -> Result<ContentId> {
    catalog
        .weapons()
        .nth(usize::from(ordinal))
        .map(|w| w.item().clone())
        .ok_or_else(|| "Unknown melee weapon ordinal".into())
}
fn pause_id(pause: Pause) -> u8 {
    match pause {
        Pause::NoWeapon => 0,
        Pause::InvalidTarget => 1,
        Pause::DeadPlayer => 2,
        Pause::NotReady => 3,
        Pause::OutOfRange => 4,
        Pause::NotFacing => 5,
        Pause::Obstructed => 6,
        Pause::Casting => 7,
        Pause::EventBudget => 8,
        Pause::TimeExhausted => 9,
    }
}
fn pause(value: u8) -> Result<Pause> {
    Ok(match value {
        0 => Pause::NoWeapon,
        1 => Pause::InvalidTarget,
        2 => Pause::DeadPlayer,
        3 => Pause::NotReady,
        4 => Pause::OutOfRange,
        5 => Pause::NotFacing,
        6 => Pause::Obstructed,
        7 => Pause::Casting,
        8 => Pause::EventBudget,
        9 => Pause::TimeExhausted,
        _ => return Err("Invalid melee pause reason".into()),
    })
}
/// Actor identity comes from the authenticated peer, never the packet.
pub fn encode_request(value: autoattack::Request) -> Result<Vec<u8>> {
    key(value.epoch, 1)?;
    if value.stream_epoch == 0
        || value.sequence == 0
        || value.target.is_some_and(|t| t.epoch.value() != value.epoch)
    {
        return Err("Invalid melee request context".into());
    }
    let mut bytes = REQUEST_TAG.to_vec();
    for n in [
        value.epoch,
        value.stream_epoch,
        value.sequence,
        value.target.map_or(0, |t| t.entity.value()),
    ] {
        word(&mut bytes, n);
    }
    Ok(bytes)
}
pub fn decode_request(bytes: &[u8]) -> Result<autoattack::Request> {
    if bytes.len() != REQUEST_BYTES {
        return Err("Invalid melee request length".into());
    }
    let mut read = Read::new(bytes, REQUEST_TAG)?;
    let epoch = read.word()?;
    let stream_epoch = read.word()?;
    let sequence = read.word()?;
    let target = read.word()?;
    read.end()?;
    let value = autoattack::Request {
        epoch,
        stream_epoch,
        sequence,
        target: if target == 0 {
            None
        } else {
            Some(key(epoch, target)?)
        },
    };
    encode_request(value)?;
    Ok(value)
}
fn validate(value: &Snapshot, catalog: &Catalog) -> Result<()> {
    if value.stream_epoch == 0
        || value
            .target
            .is_some_and(|t| t.epoch != value.actor.epoch || t == value.actor)
        || matches!(value.phase, Phase::Engaged | Phase::Attacking) && value.target.is_none()
        || matches!(value.phase, Phase::Idle) && value.target.is_some()
        || value.weapon.is_some() != value.ready_at.is_some()
        || value.target.is_some() && value.weapon.is_none()
        || value
            .ready_at
            .is_some_and(|at| at.saturating_sub(value.tick) > autoattack::MAX_READY_AHEAD_TICKS)
        || value
            .reply
            .is_some_and(|r| r.sequence == 0 || r.outcome == Outcome::Ignored)
    {
        return Err("Invalid melee state context/deadline".into());
    }
    if let Some(id) = &value.weapon {
        weapon_index(catalog, id)?;
    }
    Ok(())
}
pub fn encode_state(value: &Snapshot, catalog: &Catalog) -> Result<Vec<u8>> {
    validate(value, catalog)?;
    let mut bytes = STATE_TAG.to_vec();
    for n in [
        value.actor.epoch.value(),
        value.actor.entity.value(),
        value.stream_epoch,
        value.tick,
        value.swing_sequence,
        value.target.map_or(0, |t| t.entity.value()),
    ] {
        word(&mut bytes, n);
    }
    bytes.push(match value.phase {
        Phase::Idle => 0,
        Phase::Engaged => 1,
        Phase::Attacking => 2,
        Phase::Paused(p) => 3 + pause_id(p),
    });
    bytes.push(u8::from(value.weapon.is_some()) | (u8::from(value.reply.is_some()) << 1));
    if let Some(id) = &value.weapon {
        bytes.push(weapon_index(catalog, id)?);
        word(&mut bytes, value.ready_at.unwrap());
    }
    if let Some(reply) = value.reply {
        word(&mut bytes, reply.sequence);
        bytes.push(match reply.outcome {
            Outcome::Engaged => 0,
            Outcome::Stopped => 1,
            Outcome::Rejected(p) => 2 + pause_id(p),
            Outcome::Ignored => return Err("Ignored melee reply".into()),
        });
    }
    Ok(bytes)
}
pub fn decode_state(bytes: &[u8], catalog: &Catalog) -> Result<Snapshot> {
    if bytes.len() > MAX_STATE_BYTES {
        return Err("Melee state exceeds budget".into());
    }
    let mut read = Read::new(bytes, STATE_TAG)?;
    let epoch = read.word()?;
    let actor = key(epoch, read.word()?)?;
    let stream_epoch = read.word()?;
    let tick = read.word()?;
    let swing_sequence = read.word()?;
    let target_id = read.word()?;
    let target = if target_id == 0 {
        None
    } else {
        Some(key(epoch, target_id)?)
    };
    let phase = match read.byte()? {
        0 => Phase::Idle,
        1 => Phase::Engaged,
        2 => Phase::Attacking,
        n => Phase::Paused(pause(n - 3)?),
    };
    let flags = read.byte()?;
    if flags & !3 != 0 {
        return Err("Unknown melee state flags".into());
    }
    let (weapon, ready_at) = if flags & 1 != 0 {
        (Some(weapon(catalog, read.byte()?)?), Some(read.word()?))
    } else {
        (None, None)
    };
    let reply = if flags & 2 != 0 {
        let sequence = read.word()?;
        let outcome = match read.byte()? {
            0 => Outcome::Engaged,
            1 => Outcome::Stopped,
            n => Outcome::Rejected(pause(n - 2)?),
        };
        Some(Reply { sequence, outcome })
    } else {
        None
    };
    read.end()?;
    let value = Snapshot {
        actor,
        stream_epoch,
        tick,
        target,
        phase,
        weapon,
        ready_at,
        swing_sequence,
        reply,
    };
    validate(&value, catalog)?;
    Ok(value)
}

#[cfg(test)]
mod tests;
