//! Explicit, context-bound normal-physical result events. Legacy event layouts
//! remain unchanged. Facts are observations, never health-changing commands.
use super::{Read, get_point, key, number, put_point, word};
use crate::{
    Result,
    simulation::combat::physical::{Fact, Profile},
};
use std::num::NonZeroU64;
use theia_core::{
    content::ContentId,
    damage::{catalog::Attack, physical::Resolved},
};
pub const EVENTS_TAG: &[u8; 4] = b"CE06";
pub const FACT_TAG: u8 = 10;
pub const FACT_BYTES: usize = 94;
pub const HEADER_BYTES: usize = 61;
fn expected(
    profile: &Profile,
    attack: &ContentId,
    roll: u32,
    health: u32,
) -> Result<(bool, Resolved)> {
    match profile.catalog().attack(attack) {
        Some(Attack::Weapon { item, .. }) => Ok((true, profile.weapon_result(item, roll, health)?)),
        Some(Attack::Creature { .. }) => Ok((false, profile.creature_result(roll, health)?)),
        _ => Err("Unknown or unsupported physical attack".into()),
    }
}
pub(super) fn validate(fact: &Fact, epoch: u64, profile: &Profile) -> Result<()> {
    if fact.actor.epoch.value() != epoch
        || fact.target.epoch.value() != epoch
        || fact.actor == fact.target
        || fact.sequence == 0
    {
        return Err("Invalid physical result identity/sequence".into());
    }
    let health = fact
        .result
        .health_lost
        .checked_add(fact.remaining)
        .filter(|h| *h > 0)
        .ok_or("Invalid physical victim health")?;
    let (weapon, result) = expected(profile, &fact.attack, fact.roll, health)?;
    if result != fact.result
        || fact.actor_generation.is_some() != weapon
        || fact.target_generation.is_some() == weapon
    {
        return Err("Physical result contradicts its profile or life namespace".into());
    }
    Ok(())
}
fn index(profile: &Profile, id: &ContentId) -> Result<u8> {
    profile
        .catalog()
        .attacks()
        .position(|(key, _)| key == id)
        .map(|n| n as u8)
        .ok_or_else(|| "Unknown physical attack ordinal".into())
}
fn attack(profile: &Profile, index: u8) -> Result<ContentId> {
    profile
        .catalog()
        .attacks()
        .nth(usize::from(index))
        .map(|(id, _)| id.clone())
        .ok_or_else(|| "Unknown physical attack ordinal".into())
}
pub(super) fn put(bytes: &mut Vec<u8>, fact: &Fact, profile: &Profile) -> Result<()> {
    bytes.push(FACT_TAG);
    word(bytes, fact.actor.entity.value());
    word(bytes, fact.sequence);
    word(bytes, fact.target.entity.value());
    bytes.push(index(profile, &fact.attack)?);
    word(bytes, fact.actor_generation.map_or(0, NonZeroU64::get));
    word(bytes, fact.target_generation.map_or(0, NonZeroU64::get));
    for value in [
        fact.roll,
        fact.result.starting_damage,
        fact.result.armour_prevented,
        fact.result.resolved_damage,
        fact.result.health_lost,
        fact.result.overkill,
        fact.remaining,
    ] {
        number(bytes, value);
    }
    put_point(bytes, fact.position);
    Ok(())
}
pub(super) fn get(read: &mut Read<'_>, epoch: u64, profile: &Profile) -> Result<Fact> {
    let actor = key(epoch, read.word()?)?;
    let sequence = read.word()?;
    let target = key(epoch, read.word()?)?;
    let attack = attack(profile, read.byte()?)?;
    let actor_generation = NonZeroU64::new(read.word()?);
    let target_generation = NonZeroU64::new(read.word()?);
    let roll = read.number()?;
    let starting = read.number()?;
    let armour = read.number()?;
    let resolved = read.number()?;
    let lost = read.number()?;
    let overkill = read.number()?;
    let remaining = read.number()?;
    let position = get_point(read)?;
    let health = lost
        .checked_add(remaining)
        .filter(|h| *h > 0)
        .ok_or("Invalid physical victim health")?;
    let (_, result) = expected(profile, &attack, roll, health)?;
    if [starting, armour, resolved, lost, overkill]
        != [
            result.starting_damage,
            result.armour_prevented,
            result.resolved_damage,
            result.health_lost,
            result.overkill,
        ]
    {
        return Err("Physical wire facts contradict authoritative definitions".into());
    }
    Ok(Fact {
        attack,
        actor,
        actor_generation,
        sequence,
        target,
        target_generation,
        position,
        roll,
        result,
        remaining,
    })
}
#[cfg(test)]
mod tests;
