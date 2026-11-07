//! Presentation of committed owner progression; never derives combat stats.
use theia_network::{client::State, progression_wire::Profile};

pub(crate) fn owner_level(state: &State, profile: &Profile) -> Option<u8> {
    let welcome = state.welcome.as_ref()?;
    let view = state.progression.as_ref()?;
    if state.error.is_some()
        || welcome.world == [0; 32]
        || state.id != Some(welcome.id)
        || view.actor.entity.value() != welcome.id
        || view.actor.epoch.value() != welcome.epoch
        || view.profile_hash != profile.hash()
    {
        return None;
    }
    profile
        .profile()
        .progress(&view.progress)
        .ok()
        .map(|p| p.level)
}

/// Names are presentation only and must belong to the current admitted view.
pub(crate) fn owner_name(state: &State) -> Option<String> {
    let welcome = state.welcome.as_ref()?;
    let snapshot = state.snapshot.as_ref()?;
    let view = state.player_identity.as_ref()?;
    if state.error.is_some()
        || state.id != Some(welcome.id)
        || snapshot.epoch != welcome.epoch
        || !snapshot.players.iter().any(|p| p.id == welcome.id)
    {
        return None;
    }
    let owner = theia_core::identity::EntityKey {
        epoch: theia_core::identity::WorldEpoch::new(welcome.epoch)?,
        entity: theia_core::identity::EntityId::new(welcome.id)?,
    };
    view.validate_successor(owner, welcome.world, None).ok()?;
    view.actors
        .iter()
        .find(|a| a.key == owner)?
        .name
        .as_ref()
        .map(|name| name.as_str().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_name_requires_current_admission_snapshot_and_actor() {
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        let name =
            theia_core::character_name::CharacterName::parse("Abcdefghijklmnopqrst").unwrap();
        state.player_identity = Some(theia_network::player_identity_wire::State {
            epoch: actor.epoch.value(),
            tick: 1,
            world: state.welcome.as_ref().unwrap().world,
            actors: vec![theia_network::player_identity_wire::Actor {
                key: actor,
                name: Some(name.clone()),
            }],
        });
        assert_eq!(owner_name(&state).as_deref(), Some(name.as_str()));
        let original = state.clone();
        for case in 0..9 {
            state = original.clone();
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.id = Some(999),
                2 => state.snapshot = None,
                3 => state.snapshot.as_mut().unwrap().players.clear(),
                4 => state.snapshot.as_mut().unwrap().epoch += 1,
                5 => state.player_identity.as_mut().unwrap().world = [2; 32],
                6 => state.player_identity.as_mut().unwrap().epoch += 1,
                7 => state.player_identity.as_mut().unwrap().actors.clear(),
                8 => state.player_identity.as_mut().unwrap().actors[0].name = None,
                _ => unreachable!(),
            }
            assert_eq!(owner_name(&state), None, "case {case}");
        }
    }
    #[test]
    fn committed_level_requires_the_current_owner_epoch_and_profile() {
        let profile = Profile::from_json(include_bytes!(
            "../../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let actor = state.quest_journal.as_ref().unwrap().actor;
        state.progression = Some(theia_network::progression_wire::State {
            actor,
            profile_hash: profile.hash(),
            progress: theia_core::progression::State::new(profile.profile(), 4, 125).unwrap(),
        });
        assert_eq!(owner_level(&state, &profile), Some(2));
        let original = state.clone();
        for case in 0..7 {
            state = original.clone();
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.welcome = None,
                2 => state.progression = None,
                3 => state.id = Some(actor.entity.value() + 1),
                4 => state.welcome.as_mut().unwrap().epoch += 1,
                5 => state.welcome.as_mut().unwrap().world = [0; 32],
                6 => state.progression.as_mut().unwrap().profile_hash = [7; 32],
                _ => unreachable!(),
            }
            assert_eq!(owner_level(&state, &profile), None, "case {case}");
        }
    }
}
