//! Pure objective arithmetic, not an authorized quest or persistent kill receipt.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    quests::{self, Error, Profile, State},
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-quest.json");
fn profile() -> Profile {
    Profile::from_json(DATA).unwrap()
}
#[test]
fn checked_profiles_reject_unbounded_ambiguous_or_unsupported_objectives() {
    let data: Value = serde_json::from_slice(DATA).unwrap();
    for (key, value) in [
        ("schema", json!(2)),
        ("id", json!("../quest")),
        ("name", json!("")),
        ("name", json!(" trailing ")),
        ("name", json!("line\nbreak")),
        ("name", json!("a".repeat(65))),
        ("extra", json!(true)),
    ] {
        let mut bad = data.clone();
        bad[key] = value;
        assert!(
            Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{key}"
        );
    }
    for (key, value) in [
        ("required", json!(0)),
        ("required", json!(-1)),
        ("required", json!(1.5)),
        ("required", json!(quests::MAX_REQUIRED_KILLS + 1)),
        ("kind", json!("collect")),
        ("creature", json!("../npc")),
        ("id", json!("invalid id")),
        ("recipient", json!("client")),
        ("grants", json!([])),
    ] {
        let mut bad = data.clone();
        bad["objective"][key] = value;
        assert!(
            Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{key}"
        );
    }
    for (key, value) in [
        ("id", data["id"].clone()),
        ("id", data["objective"]["creature"].clone()),
        ("creature", data["id"].clone()),
    ] {
        let mut bad = data.clone();
        bad["objective"][key] = value;
        assert!(Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let root_duplicate = String::from_utf8(DATA.to_vec()).unwrap().replacen(
        "\"schema\": 1",
        "\"schema\": 1, \"schema\": 1",
        1,
    );
    assert!(Profile::from_json(root_duplicate.as_bytes()).is_err());
    let nested_duplicate = String::from_utf8(DATA.to_vec()).unwrap().replacen(
        "\"required\": 3",
        "\"required\": 3, \"required\": 3",
        1,
    );
    assert!(Profile::from_json(nested_duplicate.as_bytes()).is_err());
    let mut maximum = DATA.to_vec();
    maximum.resize(quests::MAX_PROFILE_BYTES, b' ');
    assert!(Profile::from_json(&maximum).is_ok());
    maximum.push(b' ');
    assert!(Profile::from_json(&maximum).is_err());
}
#[test]
fn eligible_kills_advance_once_per_proposal_and_complete_exactly_at_the_goal() {
    let p = profile();
    let initial = State::new(&p, 0, 0).unwrap();
    let mut state = initial.clone();
    assert_eq!(p.name(), "Development training");
    assert_eq!(p.required(), 3);
    assert_eq!(p.objective().as_str(), "theia.dev.training-exercise.kills");
    for count in 1..=3 {
        let old = state.clone();
        let proposal = p.propose_kill(&state, p.creature()).unwrap();
        assert!(proposal.advanced);
        assert_eq!(proposal.completed_now, count == 3);
        assert_eq!(
            state, old,
            "Pure proposals leave the saved source unchanged"
        );
        state = proposal.state;
        assert_eq!(state.revision(), u64::from(count));
        assert_eq!(state.count(), count);
        assert_eq!(p.progress(&state).unwrap().complete, count == 3);
    }
    assert_eq!(initial.count(), 0);
    assert_eq!(initial.revision(), 0);
    let done = p.propose_kill(&state, p.creature()).unwrap();
    assert_eq!(done.state, state);
    assert!(!done.advanced);
    assert!(!done.completed_now);
    // Pure proposals intentionally have no event history. Repeating a source
    // creature can progress twice; the durable victim-life receipt must dedup.
    let first = p.propose_kill(&initial, p.creature()).unwrap();
    assert_eq!(
        p.propose_kill(&first.state, p.creature())
            .unwrap()
            .state
            .count(),
        2
    );
}
#[test]
fn unrelated_kills_and_complete_objectives_are_noops_even_at_revision_exhaustion() {
    let p = profile();
    let other = ContentId::new("theia.dev.other-creature").unwrap();
    for (revision, count) in [(0, 0), (quests::MAX_REVISION, 1), (quests::MAX_REVISION, 3)] {
        let saved = State::new(&p, revision, count).unwrap();
        let proposal = p.propose_kill(&saved, &other).unwrap();
        assert!(!proposal.advanced);
        assert_eq!(proposal.state, saved);
        if count == 3 {
            assert_eq!(p.propose_kill(&saved, p.creature()).unwrap().state, saved);
        } else if revision == quests::MAX_REVISION {
            assert_eq!(
                p.propose_kill(&saved, p.creature()),
                Err(Error::RevisionExhausted)
            );
        }
    }
}
#[test]
fn invalid_totals_revisions_and_foreign_logical_profiles_cannot_be_proposed() {
    let p = profile();
    assert_eq!(State::new(&p, 0, 4), Err(Error::InvalidState));
    assert_eq!(
        State::new(&p, quests::MAX_REVISION + 1, 0),
        Err(Error::InvalidState)
    );
    let mut data: Value = serde_json::from_slice(DATA).unwrap();
    data["id"] = json!("theia.dev.other-quest");
    let other = Profile::from_json(&serde_json::to_vec(&data).unwrap()).unwrap();
    let saved = State::new(&other, 0, 0).unwrap();
    assert_eq!(p.progress(&saved), Err(Error::ProfileMismatch));
    assert_eq!(
        p.propose_kill(&saved, p.creature()),
        Err(Error::ProfileMismatch)
    );
}
#[test]
fn minimum_and_maximum_goals_stay_finite_and_never_count_beyond_completion() {
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for goal in [1, quests::MAX_REQUIRED_KILLS] {
        let mut data = original.clone();
        data["objective"]["required"] = json!(goal);
        let p = Profile::from_json(&serde_json::to_vec(&data).unwrap()).unwrap();
        let mut state = State::new(&p, 0, 0).unwrap();
        for _ in 0..goal {
            state = p.propose_kill(&state, p.creature()).unwrap().state;
        }
        for _ in 0..100 {
            assert!(!p.propose_kill(&state, p.creature()).unwrap().advanced);
        }
        assert_eq!(state.count(), goal);
        assert_eq!(state.revision(), u64::from(goal));
        assert!(p.progress(&state).unwrap().complete);
    }
}
