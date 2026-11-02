//! CE08 ability results, reconstructed from the exact bound profile. These are
//! authenticated server observations, never health-authorizing client commands.
use super::{Read, get_point, key, number, put_point, white_melee, word};
use crate::{
    Result,
    simulation::combat::ability_damage::{Draws, Fact, Profile},
};
use std::num::NonZeroU64;
use theia_core::{content::ContentId, hit_outcomes::Roll};
pub const EVENTS_TAG: &[u8; 4] = b"CE08";
pub const FACT_TAG: u8 = 12;
pub const FACT_BYTES: usize = 75;
pub const HEADER_BYTES: usize = 61;
fn index(profile: &Profile, ability: &ContentId) -> Result<u8> {
    profile
        .recipes()
        .recipes()
        .position(|(key, _)| key == ability)
        .map(|i| i as u8)
        .ok_or_else(|| "Unknown ability-damage ordinal".into())
}
fn ability(profile: &Profile, ordinal: u8) -> Result<ContentId> {
    profile
        .recipes()
        .recipes()
        .nth(usize::from(ordinal))
        .map(|(key, _)| key.clone())
        .ok_or_else(|| "Invalid ability-damage ordinal".into())
}
pub(super) fn validate(fact: &Fact, epoch: u64, profile: &Profile) -> Result<()> {
    if fact.actor.epoch.value() != epoch
        || fact.target.epoch.value() != epoch
        || fact.actor == fact.target
        || fact.sequence == 0
    {
        return Err("Invalid ability-damage identity/sequence".into());
    }
    let health = fact
        .remaining
        .checked_add(fact.result.damage.health_lost)
        .filter(|h| *h > 0)
        .ok_or("Invalid ability-damage pre-hit health")?;
    if profile.evaluate(&fact.ability, fact.defender, fact.draws, health)? != fact.result {
        return Err("Ability-damage fact contradicts its bound profile".into());
    }
    Ok(())
}
pub(super) fn put(bytes: &mut Vec<u8>, fact: &Fact, profile: &Profile) -> Result<()> {
    bytes.push(FACT_TAG);
    word(bytes, fact.actor.entity.value());
    word(bytes, fact.sequence);
    word(bytes, fact.target.entity.value());
    bytes.push(index(profile, &fact.ability)?);
    word(bytes, fact.actor_generation.get());
    number(bytes, fact.draws.base);
    bytes.extend(fact.draws.landing.value().to_le_bytes());
    bytes.extend(
        fact.draws
            .critical
            .map_or(u16::MAX, Roll::value)
            .to_le_bytes(),
    );
    bytes.push(white_melee::state_bits(fact.defender));
    number(
        bytes,
        fact.remaining
            .checked_add(fact.result.damage.health_lost)
            .ok_or("Ability-damage health exhausted")?,
    );
    number(bytes, fact.remaining);
    put_point(bytes, fact.position);
    Ok(())
}
pub(super) fn get(read: &mut Read<'_>, epoch: u64, profile: &Profile) -> Result<Fact> {
    let actor = key(epoch, read.word()?)?;
    let sequence = read.word()?;
    let target = key(epoch, read.word()?)?;
    let ability = ability(profile, read.byte()?)?;
    let actor_generation =
        NonZeroU64::new(read.word()?).ok_or("Missing ability-damage player life")?;
    let base = read.number()?;
    let landing = Roll::new(u16::from_le_bytes(read.take::<2>()?))
        .ok_or("Ability landing roll exceeds domain")?;
    let critical = match u16::from_le_bytes(read.take::<2>()?) {
        u16::MAX => None,
        n => Some(Roll::new(n).ok_or("Ability critical roll exceeds domain")?),
    };
    let defender = white_melee::state(read.byte()?)?;
    let health = read.number()?;
    let remaining = read.number()?;
    let position = get_point(read)?;
    let draws = Draws {
        base,
        landing,
        critical,
    };
    let result = profile.evaluate(&ability, defender, draws, health)?;
    if health.checked_sub(result.damage.health_lost) != Some(remaining) {
        return Err("Ability remaining health contradicts checked outcome".into());
    }
    Ok(Fact {
        ability,
        actor,
        actor_generation,
        sequence,
        target,
        position,
        defender,
        draws,
        result,
        remaining,
    })
}
#[cfg(test)]
mod tests;
