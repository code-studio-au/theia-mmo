//! Opt-in transient life content and standard Bevy presentation integration.
use std::{io::Read, path::PathBuf, sync::Arc};
use theia_network::player_life_wire::Profile;

type Error = Box<dyn std::error::Error>;
fn path(args: &mut Vec<String>, flag: &str) -> Result<Option<PathBuf>, Error> {
    let Some(index) = args.iter().position(|a| a == flag) else {
        return Ok(None);
    };
    args.remove(index);
    if index >= args.len() {
        return Err(format!("Missing file for {flag}").into());
    }
    let value = args.remove(index);
    if args.iter().any(|a| a == flag) {
        return Err(format!("Choose one {flag} profile").into());
    }
    Ok(Some(value.into()))
}
pub(crate) fn parse(
    args: &mut Vec<String>,
    stats: Option<&[u8]>,
    creature: Option<&[u8]>,
) -> Result<Option<Arc<Profile>>, Error> {
    let policy = path(args, "--creature-retaliation")?;
    let respawn = path(args, "--player-respawn")?;
    match (policy, respawn) {
        (None, None) => Ok(None),
        (Some(policy), Some(respawn)) => {
            let mut policy_bytes = Vec::new();
            std::fs::File::open(policy)?
                .take(theia_core::retaliation::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut policy_bytes)?;
            let mut respawn_bytes = Vec::new();
            std::fs::File::open(respawn)?
                .take(theia_core::player_life::MAX_PROFILE_BYTES as u64 + 1)
                .read_to_end(&mut respawn_bytes)?;
            Ok(Some(Arc::new(
                Profile::from_json(
                    stats.ok_or("Player life requires character stats")?,
                    creature.ok_or("Player life requires predicted creature combat")?,
                    &policy_bytes,
                    &respawn_bytes,
                )
                .map_err(|e| e.to_string())?,
            )))
        }
        _ => Err("Retaliation and player respawn profiles must be selected together".into()),
    }
}

pub(crate) fn parse_membership(
    args: &mut Vec<String>,
    context: bool,
) -> Result<Option<Arc<theia_network::combat_membership_wire::Profile>>, Error> {
    let Some(file) = path(args, "--combat-membership")? else {
        return Ok(None);
    };
    if !context {
        return Err("Combat membership requires explicit ability damage and player life".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(file)?
        .take(theia_core::combat_membership::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(Some(Arc::new(
        theia_network::combat_membership_wire::Profile::from_json(&bytes)
            .map_err(|e| e.to_string())?,
    )))
}

pub(crate) fn parse_credit(
    args: &mut Vec<String>,
    context: bool,
) -> Result<Option<Vec<u8>>, Error> {
    let Some(file) = path(args, "--creature-credit")? else {
        return Ok(None);
    };
    if !context {
        return Err("Creature credit requires explicit combat membership".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(file)?
        .take(theia_core::creature_credit::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let profile = theia_core::creature_credit::Profile::from_json(&bytes)?;
    if profile.recipient() != Some(theia_core::creature_credit::RecipientPolicy::SoloTagOwner) {
        return Err("Normal credit requires explicit solo tag-owner recipient policy".into());
    }
    Ok(Some(bytes))
}

pub(crate) fn parse_reset(
    args: &mut Vec<String>,
    context: bool,
    creature: Option<&theia_network::simulation::creatures::Profile>,
) -> Result<Option<theia_network::simulation::combat::reset::Profile>, Error> {
    let Some(file) = path(args, "--creature-reset")? else {
        return Ok(None);
    };
    if !context {
        return Err("Creature reset requires explicit solo credit".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(file)?
        .take(theia_network::simulation::combat::reset::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let profile = theia_network::simulation::combat::reset::Profile::from_json(&bytes)
        .map_err(|e| e.to_string())?;
    let creature = creature.ok_or("Creature reset requires pursuit creature")?;
    if profile.creature() != creature.id() || creature.aggro().is_none() {
        return Err("Creature reset requires matching pursuit creature".into());
    }
    Ok(Some(profile))
}
