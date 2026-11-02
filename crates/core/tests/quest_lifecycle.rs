//! Pure lifecycle acceptance; no database, authority, NPC or reward claim.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::{
        self, Profile,
        lifecycle::{self as life, Action, Command, Error, Phase, State, Status},
    },
};

fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))
    .unwrap()
}

fn action(profile: &Profile, state: &State, action: Action) -> Result<life::Proposal, Error> {
    life::propose_action(
        profile,
        state,
        Command {
            expected_revision: state.revision(),
            action,
        },
    )
}

fn ready(profile: &Profile) -> State {
    let mut state = action(profile, &State::available(profile), Action::Accept)
        .unwrap()
        .state;
    for _ in 0..profile.required() {
        state = life::propose_kill(profile, &state, profile.creature())
            .unwrap()
            .state;
    }
    state
}

#[test]
fn explicit_acceptance_counts_only_later_kills_and_turns_in_once() {
    let p = profile();
    let available = State::available(&p);
    assert_eq!(available.progress(&p).unwrap().phase, Phase::Available);
    let before_accept = life::propose_kill(&p, &available, p.creature()).unwrap();
    assert!(!before_accept.changed);
    assert_eq!(before_accept.state, available);
    let accepted = action(&p, &available, Action::Accept).unwrap();
    assert!(accepted.changed);
    assert!(!accepted.turned_in_now);
    assert_eq!(accepted.state.count(), 0);
    assert_eq!(accepted.state.revision(), 1);
    let mut state = accepted.state;
    for count in 1..=p.required() {
        let saved = state.clone();
        let proposed = life::propose_kill(&p, &state, p.creature()).unwrap();
        assert_eq!(state, saved, "Proposals leave persisted input untouched");
        assert!(proposed.changed);
        assert_eq!(proposed.became_ready, count == p.required());
        assert!(!proposed.turned_in_now);
        state = proposed.state;
        assert_eq!(state.revision(), u64::from(count) + 1);
        assert_eq!(state.count(), count);
    }
    assert_eq!(state.progress(&p).unwrap().phase, Phase::Ready);
    assert_eq!(state.status(), Status::Accepted);
    let capped = life::propose_kill(&p, &state, p.creature()).unwrap();
    assert!(!capped.changed);
    assert_eq!(capped.state, state);
    let turned_in = action(&p, &state, Action::TurnIn).unwrap();
    assert!(turned_in.changed && turned_in.turned_in_now);
    assert_eq!(turned_in.state.progress(&p).unwrap().phase, Phase::TurnedIn);
    assert_eq!(turned_in.state.revision(), u64::from(p.required()) + 2);
    assert_eq!(available.revision(), 0);
    for attempt in [Action::Accept, Action::TurnIn] {
        assert_eq!(
            action(&p, &turned_in.state, attempt),
            Err(Error::AlreadyTurnedIn)
        );
    }
    let later = life::propose_kill(&p, &turned_in.state, p.creature()).unwrap();
    assert!(!later.changed);
    assert_eq!(later.state, turned_in.state);
}

#[test]
fn early_turn_in_and_reaccept_refuse_without_mutating_progress() {
    let p = profile();
    let available = State::available(&p);
    assert_eq!(
        action(&p, &available, Action::TurnIn),
        Err(Error::NotAccepted)
    );
    let mut state = action(&p, &available, Action::Accept).unwrap().state;
    for _ in 0..p.required() {
        let saved = state.clone();
        assert_eq!(action(&p, &state, Action::TurnIn), Err(Error::NotReady));
        assert_eq!(
            action(&p, &state, Action::Accept),
            Err(Error::AlreadyAccepted)
        );
        assert_eq!(state, saved);
        state = life::propose_kill(&p, &state, p.creature()).unwrap().state;
    }
    assert_eq!(
        action(&p, &state, Action::Accept),
        Err(Error::AlreadyAccepted)
    );
}

#[test]
fn displayed_revision_fences_stale_acceptance_and_turn_in() {
    let p = profile();
    let initial = State::available(&p);
    let accepted = action(&p, &initial, Action::Accept).unwrap().state;
    let done = ready(&p);
    for (state, action, revision) in [
        (&initial, Action::Accept, 1),
        (&accepted, Action::Accept, 0),
        (&done, Action::TurnIn, 1),
        (&done, Action::TurnIn, u64::MAX),
    ] {
        assert_eq!(
            life::propose_action(
                &p,
                state,
                Command {
                    expected_revision: revision,
                    action
                }
            ),
            Err(Error::StaleRevision)
        );
    }
    let completed = action(&p, &done, Action::TurnIn).unwrap().state;
    assert_eq!(
        life::propose_action(
            &p,
            &completed,
            Command {
                expected_revision: done.revision(),
                action: Action::TurnIn
            }
        ),
        Err(Error::StaleRevision),
        "Exact retry recovery must occur in the durable receipt owner"
    );
}

#[test]
fn restores_refuse_impossible_counts_status_and_revision_combinations() {
    let p = profile();
    for (revision, status, count) in [
        (1, Status::Available, 0),
        (0, Status::Available, 1),
        (0, Status::Accepted, 0),
        (2, Status::Accepted, 0),
        (1, Status::Accepted, 1),
        (4, Status::Accepted, 4),
        (1, Status::TurnedIn, 0),
        (4, Status::TurnedIn, 3),
        (u64::MAX, Status::Accepted, 0),
        (quests::MAX_REVISION + 1, Status::TurnedIn, 3),
    ] {
        assert_eq!(
            State::restore(&p, revision, status, count),
            Err(Error::InvalidState)
        );
    }
    let done = ready(&p);
    let restored = State::restore(&p, done.revision(), done.status(), done.count()).unwrap();
    assert_eq!(restored, done);
    assert_eq!(restored.profile(), p.id());
}

#[test]
fn foreign_profiles_are_rejected_and_unrelated_creatures_never_advance() {
    let p = profile();
    let mut data: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))
    .unwrap();
    data["id"] = json!("theia.dev.other-quest");
    let other = Profile::from_json(&serde_json::to_vec(&data).unwrap()).unwrap();
    let foreign = State::available(&other);
    assert_eq!(foreign.progress(&p), Err(Error::ProfileMismatch));
    assert_eq!(
        action(&p, &foreign, Action::Accept),
        Err(Error::ProfileMismatch)
    );
    assert_eq!(
        life::propose_kill(&p, &foreign, p.creature()),
        Err(Error::ProfileMismatch)
    );
    let unrelated = ContentId::new("theia.dev.other-creature").unwrap();
    let active = action(&p, &State::available(&p), Action::Accept)
        .unwrap()
        .state;
    let proposal = life::propose_kill(&p, &active, &unrelated).unwrap();
    assert!(!proposal.changed && !proposal.became_ready && !proposal.turned_in_now);
    assert_eq!(proposal.state, active);
}

#[test]
fn minimum_and_maximum_goals_have_bounded_exact_lifecycles() {
    let data: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))
    .unwrap();
    for goal in [1, quests::MAX_REQUIRED_KILLS] {
        let mut raw = data.clone();
        raw["objective"]["required"] = json!(goal);
        let p = Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
        let state = ready(&p);
        assert_eq!(state.count(), goal);
        assert_eq!(state.revision(), u64::from(goal) + 1);
        let done = action(&p, &state, Action::TurnIn).unwrap().state;
        assert_eq!(done.revision(), u64::from(goal) + 2);
        assert!(done.revision() <= 102);
    }
}

#[test]
fn command_identity_distinguishes_action_revision_and_lifecycle_mode() {
    let accept = Command {
        expected_revision: 0,
        action: Action::Accept,
    };
    let turn_in = Command {
        action: Action::TurnIn,
        ..accept
    };
    let newer = Command {
        expected_revision: 1,
        ..accept
    };
    assert_ne!(accept.canonical_bytes(), turn_in.canonical_bytes());
    assert_ne!(accept.canonical_bytes(), newer.canonical_bytes());
    assert_ne!(life::PROFILE_DOMAIN, quests::PROFILE_DOMAIN);
    assert_eq!(accept.canonical_bytes(), [0, 0, 0, 0, 0, 0, 0, 0, 1]);
}
