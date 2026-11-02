//! Explicit ordinary-outcome facts. The immutable bound profile reconstructs
//! amounts from checked draws and positive pre-hit health; these are observations,
//! never health-changing client commands. Existing CE02–CE06 layouts stay intact.
use super::{Read, get_point, key, number, put_point, word};
use crate::{
    Result,
    simulation::combat::white_melee::{DefenderState, Draws, Fact, Profile},
};
use std::num::NonZeroU64;
use theia_core::{
    content::ContentId,
    damage::{catalog::Attack, white_melee::impact::Resolved},
    hit_outcomes::Roll,
};
pub const EVENTS_TAG: &[u8; 4] = b"CE07";
pub const FACT_TAG: u8 = 11;
pub const FACT_BYTES: usize = 82;
pub const HEADER_BYTES: usize = 61;
pub(super) fn state_bits(state: DefenderState) -> u8 {
    [
        state.standing,
        state.can_react,
        state.casting_non_melee,
        state.in_front,
        state.disarmed,
        state.usable_melee_weapon,
        state.usable_shield,
    ]
    .into_iter()
    .enumerate()
    .fold(0, |bits, (i, value)| bits | (u8::from(value) << i))
}
pub(super) fn state(bits: u8) -> Result<DefenderState> {
    if bits & 128 != 0 {
        return Err("Reserved ordinary defender-state bit".into());
    }
    Ok(DefenderState {
        standing: bits & 1 != 0,
        can_react: bits & 2 != 0,
        casting_non_melee: bits & 4 != 0,
        in_front: bits & 8 != 0,
        disarmed: bits & 16 != 0,
        usable_melee_weapon: bits & 32 != 0,
        usable_shield: bits & 64 != 0,
    })
}
fn expected(
    profile: &Profile,
    attack: &ContentId,
    defender: DefenderState,
    draws: Draws,
    health: u32,
) -> Result<(bool, Resolved)> {
    match profile.physical().catalog().attack(attack) {
        Some(Attack::Weapon { item, .. }) => Ok((
            true,
            profile.evaluate_outgoing(item, defender, draws, health)?,
        )),
        Some(Attack::Creature { .. }) => {
            Ok((false, profile.evaluate_incoming(defender, draws, health)?))
        }
        _ => Err("Unknown ordinary attack".into()),
    }
}
pub(super) fn validate(fact: &Fact, epoch: u64, profile: &Profile) -> Result<()> {
    if fact.actor.epoch.value() != epoch
        || fact.target.epoch.value() != epoch
        || fact.actor == fact.target
        || fact.sequence == 0
    {
        return Err("Invalid ordinary fact identity/sequence".into());
    }
    let health = fact
        .remaining
        .checked_add(fact.result.health_lost)
        .filter(|h| *h > 0)
        .ok_or("Invalid ordinary victim health")?;
    let (weapon, result) = expected(profile, &fact.attack, fact.defender, fact.draws, health)?;
    if result != fact.result
        || fact.actor_generation.is_some() != weapon
        || fact.target_generation.is_some() == weapon
    {
        return Err("Ordinary fact contradicts profile, outcome or life namespace".into());
    }
    Ok(())
}
fn index(profile: &Profile, attack: &ContentId) -> Result<u8> {
    profile
        .physical()
        .catalog()
        .attacks()
        .position(|(id, _)| id == attack)
        .map(|n| n as u8)
        .ok_or_else(|| "Unknown ordinary attack ordinal".into())
}
fn attack(profile: &Profile, index: u8) -> Result<ContentId> {
    profile
        .physical()
        .catalog()
        .attacks()
        .nth(usize::from(index))
        .map(|(id, _)| id.clone())
        .ok_or_else(|| "Unknown ordinary attack ordinal".into())
}
pub(super) fn put(bytes: &mut Vec<u8>, fact: &Fact, profile: &Profile) -> Result<()> {
    bytes.push(FACT_TAG);
    word(bytes, fact.actor.entity.value());
    word(bytes, fact.sequence);
    word(bytes, fact.target.entity.value());
    bytes.push(index(profile, &fact.attack)?);
    word(bytes, fact.actor_generation.map_or(0, NonZeroU64::get));
    word(bytes, fact.target_generation.map_or(0, NonZeroU64::get));
    number(bytes, fact.draws.base);
    bytes.extend(fact.draws.outcome.value().to_le_bytes());
    bytes.push(fact.draws.glancing.unwrap_or(0));
    bytes.push(state_bits(fact.defender));
    number(
        bytes,
        fact.remaining
            .checked_add(fact.result.health_lost)
            .ok_or("Ordinary health sum exhausted")?,
    );
    number(bytes, fact.remaining);
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
    let base = read.number()?;
    let outcome = Roll::new(u16::from_le_bytes(read.take::<2>()?))
        .ok_or("Ordinary outcome roll exceeds domain")?;
    let glancing = match read.byte()? {
        0 => None,
        value => Some(value),
    };
    let defender = state(read.byte()?)?;
    let health = read.number()?;
    let remaining = read.number()?;
    let position = get_point(read)?;
    let draws = Draws {
        outcome,
        base,
        glancing,
    };
    let (_, result) = expected(profile, &attack, defender, draws, health)?;
    if health.checked_sub(result.health_lost) != Some(remaining) {
        return Err("Ordinary remaining health contradicts checked outcome".into());
    }
    Ok(Fact {
        attack,
        actor,
        actor_generation,
        sequence,
        target,
        target_generation,
        position,
        defender,
        draws,
        result,
        remaining,
    })
}
#[cfg(test)]
mod tests;
