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

#[cfg(test)]
mod tests {
    use super::*;
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
