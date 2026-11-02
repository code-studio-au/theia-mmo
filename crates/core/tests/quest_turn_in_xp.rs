//! Pure completion/XP proposals only: no SQL, runtime entitlement or receipts.
use serde_json::{Value, json};
use theia_core::{
    progression as xp,
    quests::{
        self, lifecycle as life,
        turn_in_xp::{self as reward, Error, Recipe},
    },
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-quest-turn-in-xp.json");
fn profiles() -> (quests::Profile, xp::Profile) {
    (
        quests::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-quest.json"
        ))
        .unwrap(),
        xp::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-progression.json"
        ))
        .unwrap(),
    )
}
fn ready(q: &quests::Profile) -> life::State {
    let mut s = life::propose_action(
        q,
        &life::State::available(q),
        life::Command {
            expected_revision: 0,
            action: life::Action::Accept,
        },
    )
    .unwrap()
    .state;
    for _ in 0..q.required() {
        s = life::propose_kill(q, &s, q.creature()).unwrap().state;
    }
    s
}
fn command(state: &life::State) -> life::Command {
    life::Command {
        expected_revision: state.revision(),
        action: life::Action::TurnIn,
    }
}
#[test]
fn bounded_recipes_bind_the_checked_quest_and_progression_profiles() {
    let (q, x) = profiles();
    let original: Value = serde_json::from_slice(DATA).unwrap();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    assert_eq!(r.id().as_str(), "theia.dev.training-turn-in-xp");
    assert_eq!(r.quest(), q.id());
    assert_eq!(r.progression(), x.id());
    assert_eq!(r.experience(), 25);
    for (key, value) in [
        ("schema", json!(2)),
        ("id", json!("../recipe")),
        ("quest", json!("other.quest")),
        ("progression", json!("other.progression")),
        ("experience", json!(0)),
        ("experience", json!(xp::MAX_GRANT_EXPERIENCE + 1)),
        ("experience", json!(-1)),
        ("experience", json!(1.5)),
        ("experience", json!(u64::MAX)),
        ("recipient", json!("client")),
        ("items", json!([])),
    ] {
        let mut bad = original.clone();
        bad[key] = value;
        assert!(
            Recipe::from_json(&serde_json::to_vec(&bad).unwrap(), &q, &x).is_err(),
            "{key}"
        );
    }
    assert!(Recipe::from_json(&vec![b' '; reward::MAX_PROFILE_BYTES + 1], &q, &x).is_err());
    let duplicate = String::from_utf8(DATA.to_vec()).unwrap().replacen(
        "\"schema\": 1",
        "\"schema\": 1, \"schema\": 1",
        1,
    );
    assert!(Recipe::from_json(duplicate.as_bytes(), &q, &x).is_err());
    let mut missing = original;
    missing.as_object_mut().unwrap().remove("experience");
    assert!(Recipe::from_json(&serde_json::to_vec(&missing).unwrap(), &q, &x).is_err());
}
#[test]
fn three_default_kills_and_completion_propose_a_level_without_mutating_inputs() {
    let (q, x) = profiles();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    let s = ready(&q);
    let mut experience = xp::State::new(&x, 0, 0).unwrap();
    for _ in 0..3 {
        experience = x.propose_kill(&experience, q.creature()).unwrap().state;
    }
    assert_eq!((experience.revision(), experience.total()), (3, 75));
    let saved = experience.clone();
    let saved_quest = s.clone();
    let p = r.propose(&q, &s, &x, &experience, command(&s)).unwrap();
    assert_eq!(s, saved_quest);
    assert_eq!(experience, saved);
    assert_eq!(p.quest.state.status(), life::Status::TurnedIn);
    assert_eq!(p.quest.state.revision(), 5);
    assert!(p.quest.turned_in_now && p.quest.changed);
    assert_eq!(
        (
            p.experience.state.revision(),
            p.experience.state.total(),
            p.experience.granted,
            p.experience.previous_level,
            p.experience.level
        ),
        (4, 100, 25, 1, 2)
    );
    assert_eq!(
        r.propose(&q, &p.quest.state, &x, &p.experience.state, command(&s)),
        Err(Error::Quest(life::Error::StaleRevision))
    );
    assert_eq!(
        r.propose(
            &q,
            &p.quest.state,
            &x,
            &p.experience.state,
            command(&p.quest.state)
        ),
        Err(Error::Quest(life::Error::AlreadyTurnedIn))
    );
}
#[test]
fn incomplete_or_acceptance_actions_never_propose_experience() {
    let (q, x) = profiles();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    let available = life::State::available(&q);
    let capped = xp::State::new(&x, 9, x.maximum_experience()).unwrap();
    assert_eq!(
        r.propose(&q, &available, &x, &capped, command(&available)),
        Err(Error::Quest(life::Error::NotAccepted))
    );
    let accept = life::Command {
        expected_revision: 0,
        action: life::Action::Accept,
    };
    assert_eq!(
        r.propose(&q, &available, &x, &capped, accept),
        Err(Error::NotTurnIn)
    );
    let active = life::propose_action(&q, &available, accept).unwrap().state;
    assert_eq!(
        r.propose(&q, &active, &x, &capped, command(&active)),
        Err(Error::Quest(life::Error::NotReady))
    );
    assert_eq!(active.count(), 0);
    assert_eq!(capped.total(), 250);
}
#[test]
fn caps_clip_the_award_and_preserve_a_zero_award_revision_even_at_exhaustion() {
    let (q, x) = profiles();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    let s = ready(&q);
    for (total, revision, granted, next_revision) in [
        (240, 7, 10, 8),
        (249, 8, 1, 9),
        (250, 9, 0, 9),
        (250, xp::MAX_REVISION, 0, xp::MAX_REVISION),
    ] {
        let old = xp::State::new(&x, revision, total).unwrap();
        let p = r.propose(&q, &s, &x, &old, command(&s)).unwrap();
        assert_eq!(p.experience.granted, granted);
        assert_eq!(p.experience.state.total(), 250);
        assert_eq!(p.experience.state.revision(), next_revision);
        assert!(p.quest.turned_in_now);
        assert_eq!(s.status(), life::Status::Accepted);
    }
}
#[test]
fn positive_award_exhaustion_returns_no_partial_completion() {
    let (q, x) = profiles();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    let s = ready(&q);
    let old = xp::State::new(&x, xp::MAX_REVISION, 75).unwrap();
    let saved = (s.clone(), old.clone());
    assert_eq!(
        r.propose(&q, &s, &x, &old, command(&s)),
        Err(Error::Progression(xp::Error::RevisionExhausted))
    );
    assert_eq!((s, old), saved);
}
#[test]
fn foreign_profiles_and_states_refuse_before_a_combined_proposal() {
    let (q, x) = profiles();
    let r = Recipe::from_json(DATA, &q, &x).unwrap();
    let s = ready(&q);
    let old = xp::State::new(&x, 0, 75).unwrap();
    let mut qraw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))
    .unwrap();
    qraw["id"] = json!("other.quest");
    let otherq = quests::Profile::from_json(&serde_json::to_vec(&qraw).unwrap()).unwrap();
    assert_eq!(
        r.propose(&otherq, &s, &x, &old, command(&s)),
        Err(Error::RecipeMismatch)
    );
    let foreign = ready(&otherq);
    assert_eq!(
        r.propose(&q, &foreign, &x, &old, command(&foreign)),
        Err(Error::Quest(life::Error::ProfileMismatch))
    );
    let mut xraw: Value = serde_json::from_slice(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    xraw["id"] = json!("other.progression");
    let otherx = xp::Profile::from_json(&serde_json::to_vec(&xraw).unwrap()).unwrap();
    assert_eq!(
        r.propose(&q, &s, &otherx, &old, command(&s)),
        Err(Error::RecipeMismatch)
    );
    let foreign = xp::State::new(&otherx, 0, 75).unwrap();
    assert_eq!(
        r.propose(&q, &s, &x, &foreign, command(&s)),
        Err(Error::Progression(xp::Error::ProfileMismatch))
    );
    xraw["id"] = json!(x.id().as_str());
    xraw["kill_rewards"][0]["creature"] = json!("other.creature");
    let wrong = xp::Profile::from_json(&serde_json::to_vec(&xraw).unwrap()).unwrap();
    assert!(Recipe::from_json(DATA, &q, &wrong).is_err());
}
#[test]
fn trusted_grants_are_bounded_and_preserve_existing_kill_arithmetic() {
    let (q, x) = profiles();
    for total in [0, 99, 249, 250] {
        for revision in [0, xp::MAX_REVISION - 1, xp::MAX_REVISION] {
            let old = xp::State::new(&x, revision, total).unwrap();
            assert_eq!(
                x.propose_grant(&old, 25),
                x.propose_kill(&old, q.creature())
            );
            for invalid in [0, xp::MAX_GRANT_EXPERIENCE + 1, u32::MAX] {
                assert_eq!(x.propose_grant(&old, invalid), Err(xp::Error::InvalidGrant));
            }
        }
    }
    let old = xp::State::new(&x, 0, 0).unwrap();
    let max = x.propose_grant(&old, xp::MAX_GRANT_EXPERIENCE).unwrap();
    assert_eq!(max.granted, 250);
    assert_eq!(max.state.revision(), 1);
    assert_ne!(reward::PROFILE_DOMAIN, life::PROFILE_DOMAIN);
}
