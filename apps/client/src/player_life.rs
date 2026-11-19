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

/// Identity/configuration only; the server owns threat arithmetic and victims.
pub(crate) fn parse_threat(
    args: &mut Vec<String>,
    context: bool,
) -> Result<Option<Vec<u8>>, Error> {
    let Some(file) = path(args, "--creature-threat")? else {
        return Ok(None);
    };
    if !context {
        return Err("Creature threat requires explicit return reset".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(file)?
        .take(theia_core::threat::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    theia_core::threat::Profile::from_json(&bytes)?;
    Ok(Some(bytes))
}

/// Public timing is an explicit presentation contract, with unchanged authority.
pub(crate) fn parse_public_attacks(
    args: &mut Vec<String>,
    classic: bool,
    life: Option<&Arc<Profile>>,
) -> Result<Option<Arc<theia_network::creature_attack_wire::Profile>>, Error> {
    let Some(index) = args.iter().position(|a| a == "--public-creature-attacks") else {
        return Ok(None);
    };
    args.remove(index);
    if args.iter().any(|a| a == "--public-creature-attacks") {
        return Err("Choose public creature attacks once".into());
    }
    let life = life
        .filter(|_| classic)
        .ok_or("Public creature attacks require Classic pursuit/player life")?;
    Ok(Some(Arc::new(
        theia_network::creature_attack_wire::Profile::new(life.clone()),
    )))
}

pub(crate) fn parse_public_avatar(
    args: &mut Vec<String>,
    classic: bool,
    life: Option<&Arc<Profile>>,
) -> Result<Option<Arc<theia_network::avatar_life_wire::Profile>>, Error> {
    let Some(index) = args.iter().position(|a| a == "--public-avatar-life") else {
        return Ok(None);
    };
    args.remove(index);
    if args.iter().any(|a| a == "--public-avatar-life") {
        return Err("Choose public avatar life once".into());
    }
    let life = life
        .filter(|_| classic)
        .ok_or("Public avatar life requires Classic player life")?;
    Ok(Some(Arc::new(
        theia_network::avatar_life_wire::Profile::new(life.clone()),
    )))
}

#[cfg(test)]
mod public_tests {
    use super::*;
    #[test]
    fn public_flag_is_exclusive_and_requires_matching_pursuit_life_context() {
        let flag = "--public-creature-attacks".to_owned();
        assert!(
            parse_public_attacks(&mut vec![], false, None)
                .unwrap()
                .is_none()
        );
        assert!(parse_public_attacks(&mut vec![flag.clone()], false, None).is_err());
        assert!(parse_public_attacks(&mut vec![flag.clone(), flag.clone()], true, None).is_err());
        let life = Arc::new(
            Profile::from_json(
                include_bytes!("../../../content/gameplay/development-character-stats.json"),
                include_bytes!(
                    "../../../content/gameplay/development-creature-deflect-pursuit.json"
                ),
                include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
                include_bytes!("../../../content/gameplay/development-player-respawn.json"),
            )
            .unwrap(),
        );
        assert!(parse_public_attacks(&mut vec![flag.clone()], false, Some(&life)).is_err());
        let mut args = vec![flag, "--other".into()];
        let public = parse_public_attacks(&mut args, true, Some(&life))
            .unwrap()
            .unwrap();
        assert_eq!(args, vec!["--other"]);
        assert_eq!(public.life().hash(), life.hash());
        assert!(
            parse_public_avatar(&mut vec![], false, None)
                .unwrap()
                .is_none()
        );
        assert!(
            parse_public_avatar(&mut vec!["--public-avatar-life".into()], false, Some(&life))
                .is_err()
        );
        assert!(
            parse_public_avatar(
                &mut vec!["--public-avatar-life".into(), "--public-avatar-life".into()],
                true,
                Some(&life)
            )
            .is_err()
        );
        assert_eq!(
            parse_public_avatar(&mut vec!["--public-avatar-life".into()], true, Some(&life))
                .unwrap()
                .unwrap()
                .life()
                .hash(),
            life.hash()
        );
    }
}
