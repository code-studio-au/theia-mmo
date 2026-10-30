//! Pure checked progression only; no kill entitlement, deduplication or COMMIT.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::{self as xp, Error, KillId, Profile, State},
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-progression.json");
fn creature() -> ContentId {
    ContentId::new("theia.dev.training-creature").unwrap()
}
fn profile() -> Profile {
    Profile::from_json(DATA).unwrap()
}
#[test]
fn profiles_reject_ambiguous_unbounded_or_nonmonotonic_curves_and_rewards() {
    let original: Value = serde_json::from_slice(DATA).unwrap();
    for (key, value) in [
        ("schema", json!(2)),
        ("id", json!("../progression")),
        ("extra", json!(1)),
        ("level_thresholds", json!([])),
        ("level_thresholds", json!([0])),
        ("level_thresholds", json!([1, 100])),
        ("level_thresholds", json!([0, 0, 100])),
        ("level_thresholds", json!([0, 100, 99])),
        ("level_thresholds", json!([0, xp::MAX_EXPERIENCE + 1])),
        ("level_thresholds", json!([0, -1])),
        ("level_thresholds", json!([0, 1.5])),
        ("kill_rewards", json!([])),
    ] {
        let mut bad = original.clone();
        bad[key] = value;
        assert!(
            Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{key}"
        );
    }
    let mut bad = original.clone();
    bad["level_thresholds"] = json!((0..=xp::MAX_LEVELS).collect::<Vec<_>>());
    assert!(Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    for row in [
        json!({"creature":"../npc","experience":25}),
        json!({"creature":"theia.dev.training-creature","experience":0}),
        json!({"creature":"theia.dev.training-creature","experience":xp::MAX_KILL_EXPERIENCE+1}),
        json!({"creature":"theia.dev.training-creature","experience":-1}),
        json!({"creature":"theia.dev.training-creature","experience":25,"recipient":"client"}),
    ] {
        let mut bad = original.clone();
        bad["kill_rewards"] = json!([row]);
        assert!(Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let row = original["kill_rewards"][0].clone();
    for count in [2, xp::MAX_KILL_DEFINITIONS + 1] {
        let mut bad = original.clone();
        bad["kill_rewards"] = json!(vec![row.clone(); count]);
        assert!(Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = original;
    bad["kill_rewards"] = json!(
        (0..=xp::MAX_KILL_DEFINITIONS)
            .map(|i| json!({"creature":format!("theia.dev.creature-{i}"),"experience":1}))
            .collect::<Vec<_>>()
    );
    assert!(Profile::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert!(Profile::from_json(br#"{"schema":1,"schema":1,"id":"theia.dev.progression","level_thresholds":[0,1],"kill_rewards":[{"creature":"theia.dev.creature","experience":1}]}"#).is_err());
    assert!(Profile::from_json(&vec![b' '; xp::MAX_PROFILE_BYTES + 1]).is_err());
}
#[test]
fn cumulative_thresholds_derive_levels_and_an_exact_bounded_display() {
    let p = profile();
    assert_eq!(p.thresholds(), &[0, 100, 250]);
    assert_eq!(p.kill_rewards().len(), 1);
    for (total, level, earned, needed) in [
        (0, 1, 0, Some(100)),
        (99, 1, 99, Some(100)),
        (100, 2, 0, Some(150)),
        (249, 2, 149, Some(150)),
        (250, 3, 0, None),
    ] {
        let s = State::new(&p, 7, total).unwrap();
        let view = p.progress(&s).unwrap();
        assert_eq!(
            (
                view.total,
                view.level,
                view.earned_in_level,
                view.required_in_level
            ),
            (total, level, earned, needed)
        );
        assert_eq!(s.revision(), 7);
        assert_eq!(s.profile(), p.id());
    }
    assert!(State::new(&p, 0, 251).is_err());
    assert!(State::new(&p, xp::MAX_REVISION + 1, 0).is_err());
}
#[test]
fn trusted_proposals_cross_levels_clamp_to_cap_and_leave_source_state_unchanged() {
    let p = profile();
    let source = State::new(&p, 8, 75).unwrap();
    let before = source.clone();
    let next = p.propose_kill(&source, &creature()).unwrap();
    assert_eq!(source, before);
    assert_eq!(
        (
            next.state.total(),
            next.state.revision(),
            next.granted,
            next.previous_level,
            next.level
        ),
        (100, 9, 25, 1, 2)
    );
    let near = State::new(&p, 9, 240).unwrap();
    let cap = p.propose_kill(&near, &creature()).unwrap();
    assert_eq!(
        (
            cap.state.total(),
            cap.state.revision(),
            cap.granted,
            cap.level
        ),
        (250, 10, 10, 3)
    );
    let mut data: Value = serde_json::from_slice(DATA).unwrap();
    data["kill_rewards"][0]["experience"] = json!(250);
    let p = Profile::from_json(&serde_json::to_vec(&data).unwrap()).unwrap();
    let next = p
        .propose_kill(&State::new(&p, 0, 0).unwrap(), &creature())
        .unwrap();
    assert_eq!((next.previous_level, next.level, next.granted), (1, 3, 250));
}
#[test]
fn cap_is_a_noop_even_at_last_revision_but_invalid_or_foreign_work_never_awards() {
    let p = profile();
    let capped = State::new(&p, xp::MAX_REVISION, 250).unwrap();
    let noop = p.propose_kill(&capped, &creature()).unwrap();
    assert_eq!(noop.state, capped);
    assert_eq!(noop.granted, 0);
    let exhausted = State::new(&p, xp::MAX_REVISION, 0).unwrap();
    assert_eq!(
        p.propose_kill(&exhausted, &creature()),
        Err(Error::RevisionExhausted)
    );
    let other = ContentId::new("theia.dev.unknown-creature").unwrap();
    assert_eq!(p.propose_kill(&capped, &other), Err(Error::UnknownCreature));
    let mut raw: Value = serde_json::from_slice(DATA).unwrap();
    raw["id"] = json!("theia.dev.foreign-progression");
    let foreign = Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();
    assert_eq!(foreign.progress(&capped), Err(Error::ProfileMismatch));
    assert_eq!(
        foreign.propose_kill(&capped, &creature()),
        Err(Error::ProfileMismatch)
    );
}
#[test]
fn every_positive_award_consumes_finite_budget_and_pure_arithmetic_does_not_deduplicate() {
    let p = profile();
    let mut state = State::new(&p, 0, 0).unwrap();
    let mut awards = 0;
    loop {
        let proposed = p.propose_kill(&state, &creature()).unwrap();
        if proposed.granted == 0 {
            break;
        }
        assert!(proposed.state.total() > state.total());
        state = proposed.state;
        awards += 1;
        assert!(awards <= p.maximum_experience());
    }
    assert_eq!((awards, state.total(), state.revision()), (10, 250, 10));
    let first = p
        .propose_kill(&State::new(&p, 0, 0).unwrap(), &creature())
        .unwrap();
    let second = p.propose_kill(&first.state, &creature()).unwrap();
    assert_eq!(
        second.state.total(),
        50,
        "The later durable receipt boundary, not arithmetic, must reject replayed kills"
    );
}
#[test]
fn victim_life_receipts_bind_both_nonzero_incarnation_and_entity_words() {
    let victim = EntityKey {
        epoch: WorldEpoch::new(u64::MAX).unwrap(),
        entity: EntityId::new(7).unwrap(),
    };
    let id = KillId::from_victim(victim);
    assert_eq!(id.victim(), victim);
    assert_eq!(KillId::from_bytes(id.bytes()), Some(id));
    for range in [0..8, 8..16] {
        let mut bad = id.bytes();
        bad[range].fill(0);
        assert!(KillId::from_bytes(bad).is_none());
    }
    assert_ne!(
        id,
        KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            ..victim
        })
    );
    assert_ne!(
        id,
        KillId::from_victim(EntityKey {
            entity: EntityId::new(8).unwrap(),
            ..victim
        })
    );
}
