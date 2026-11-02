//! Pure membership prerequisites, not live World/transport or permission checks.
use std::num::NonZeroU64;
use theia_core::{
    combat_membership::{Error, Hostility, MAX_OPPONENTS, MAX_PROFILE_BYTES, Profile, State},
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    time::TickRate,
};
const PROFILE: &[u8] =
    include_bytes!("../../../content/gameplay/development-combat-membership.json");
fn actor(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn life() -> LifeKey {
    LifeKey {
        actor: actor(1),
        generation: NonZeroU64::MIN,
    }
}
fn profile(capacity: usize) -> Profile {
    let mut value: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    value["maximum_opponents"] = capacity.into();
    Profile::from_json(&serde_json::to_vec(&value).unwrap()).unwrap()
}
#[test]
fn hostility_reasons_expire_independently_on_exact_boundaries_at_all_tick_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = State::new(profile(2), rate, life());
        assert!(!state.summary().in_combat());
        assert_eq!(state.owner(), life());
        let second = u64::from(rate.hz());
        // An avoided/zero-damage authorized attack is still hostility; amounts
        // are deliberately not part of this membership contract.
        state
            .hostility(0, life(), actor(2), Hostility::Outgoing)
            .unwrap();
        state
            .hostility(second, life(), actor(2), Hostility::Incoming)
            .unwrap();
        let both = state.advance(second * 5 - 1, life()).unwrap();
        assert_eq!(both.opponents, 1);
        assert!(both.in_combat() && both.outgoing_hostility && both.incoming_hostility);
        let incoming = state.advance(second * 5, life()).unwrap();
        assert!(!incoming.outgoing_hostility && incoming.incoming_hostility);
        assert!(state.advance(second * 6 - 1, life()).unwrap().in_combat());
        assert!(!state.advance(second * 6, life()).unwrap().in_combat());
    }
}
#[test]
fn encounter_keeps_membership_without_refreshing_hostility_or_requiring_damage() {
    let mut state = State::new(profile(2), TickRate::Hz30, life());
    state
        .hostility(0, life(), actor(2), Hostility::Outgoing)
        .unwrap();
    state.encounter(0, life(), actor(2), true).unwrap();
    let held = state.advance(30 * 120, life()).unwrap();
    assert!(held.in_combat() && held.active_encounter);
    assert!(!held.outgoing_hostility && !held.incoming_hostility);
    state.encounter(30 * 120, life(), actor(2), false).unwrap();
    assert!(!state.summary().in_combat());

    state
        .hostility(30 * 121, life(), actor(3), Hostility::Incoming)
        .unwrap();
    state.encounter(30 * 121, life(), actor(3), true).unwrap();
    state.encounter(30 * 122, life(), actor(3), false).unwrap();
    let remaining = state.summary();
    assert!(remaining.in_combat() && remaining.incoming_hostility && !remaining.active_encounter);
}
#[test]
fn explicit_resolution_releases_only_the_exact_opponent_and_ended_life_cannot_reenter() {
    let mut state = State::new(profile(2), TickRate::Hz30, life());
    state
        .hostility(0, life(), actor(2), Hostility::Outgoing)
        .unwrap();
    state.encounter(0, life(), actor(2), true).unwrap();
    state.encounter(0, life(), actor(3), true).unwrap();
    state.forget(1, life(), actor(2)).unwrap();
    assert_eq!(state.summary().opponents, 1);
    assert!(!state.summary().outgoing_hostility);
    state.end_life(2, life()).unwrap();
    assert!(!state.summary().in_combat());
    let ended = state.clone();
    assert_eq!(
        state.hostility(3, life(), actor(2), Hostility::Incoming),
        Err(Error::LifeEnded)
    );
    assert_eq!(
        state.encounter(3, life(), actor(2), true),
        Err(Error::LifeEnded)
    );
    assert_eq!(state.advance(3, life()), Err(Error::LifeEnded));
    assert_eq!(state, ended);
    let next_life = LifeKey {
        generation: NonZeroU64::new(2).unwrap(),
        ..life()
    };
    let mut replacement = State::new(profile(2), TickRate::Hz30, next_life);
    assert!(!replacement.summary().in_combat());
    assert_eq!(
        replacement.hostility(3, life(), actor(2), Hostility::Incoming),
        Err(Error::StaleLife)
    );
    replacement.encounter(3, next_life, actor(3), true).unwrap();
    assert_eq!(replacement.summary().opponents, 1);
}
#[test]
fn bounded_capacity_refusal_is_atomic_but_existing_and_expired_slots_can_be_reused() {
    let mut state = State::new(profile(1), TickRate::Hz30, life());
    state
        .hostility(0, life(), actor(2), Hostility::Outgoing)
        .unwrap();
    let full = state.clone();
    assert_eq!(
        state.hostility(1, life(), actor(3), Hostility::Incoming),
        Err(Error::Capacity)
    );
    assert_eq!(state, full);
    assert_eq!(
        state.encounter(1, life(), actor(3), true),
        Err(Error::Capacity)
    );
    assert_eq!(state, full);
    state
        .hostility(1, life(), actor(2), Hostility::Incoming)
        .unwrap();
    state.encounter(1, life(), actor(4), false).unwrap();
    assert_eq!(state.summary().opponents, 1);
    // No intervening advance required to reclaim expired contacts.
    state
        .hostility(151, life(), actor(3), Hostility::Outgoing)
        .unwrap();
    assert_eq!(state.summary().opponents, 1);
    assert!(!state.summary().incoming_hostility);
    state.forget(152, life(), actor(3)).unwrap();
    state.encounter(152, life(), actor(4), true).unwrap();
    assert_eq!(state.summary().opponents, 1);
}
#[test]
fn life_tick_epoch_self_and_overflow_refusals_preserve_the_complete_state() {
    let mut state = State::new(profile(2), TickRate::Hz30, life());
    state.encounter(10, life(), actor(2), true).unwrap();
    let before = state.clone();
    let stale = LifeKey {
        generation: NonZeroU64::new(2).unwrap(),
        ..life()
    };
    assert_eq!(
        state.hostility(10, stale, actor(2), Hostility::Outgoing),
        Err(Error::StaleLife)
    );
    assert_eq!(state.advance(9, life()), Err(Error::StaleTick));
    assert_eq!(
        state.encounter(9, life(), actor(2), false),
        Err(Error::StaleTick)
    );
    assert_eq!(state.forget(9, life(), actor(2)), Err(Error::StaleTick));
    assert_eq!(state.end_life(9, life()), Err(Error::StaleTick));
    let foreign = EntityKey {
        epoch: WorldEpoch::new(8).unwrap(),
        ..actor(2)
    };
    assert_eq!(
        state.hostility(10, life(), foreign, Hostility::Incoming),
        Err(Error::InvalidOpponent)
    );
    assert_eq!(
        state.encounter(10, life(), life().actor, true),
        Err(Error::InvalidOpponent)
    );
    assert_eq!(
        state.forget(10, life(), foreign),
        Err(Error::InvalidOpponent)
    );
    assert_eq!(
        state.hostility(u64::MAX, life(), actor(2), Hostility::Outgoing),
        Err(Error::TimeExhausted)
    );
    assert_eq!(state, before);
    assert_eq!(state.advance(10, life()).unwrap(), before.summary());
    assert_eq!(state, before);
}
#[test]
fn adversarial_distinct_attempts_never_grow_beyond_the_declared_bound() {
    let mut state = State::new(profile(MAX_OPPONENTS), TickRate::Hz20, life());
    for id in 2..=1000 {
        let result = state.hostility(0, life(), actor(id), Hostility::Outgoing);
        assert_eq!(result.is_ok(), id < 2 + MAX_OPPONENTS as u64);
        assert!(state.summary().opponents <= MAX_OPPONENTS);
    }
    assert_eq!(state.advance(100, life()).unwrap().opponents, 0);
    state.encounter(100, life(), actor(1001), true).unwrap();
    assert_eq!(state.summary().opponents, 1);
}
#[test]
fn profile_is_strict_bounded_and_explicitly_fixture_versioned() {
    let parsed = Profile::from_json(PROFILE).unwrap();
    assert_eq!(parsed.maximum_opponents(), MAX_OPPONENTS);
    assert_eq!(parsed.hostility_grace_ms(), 5000);
    assert_eq!(parsed.id().as_str(), "theia.dev.combat-membership-v1");
    let original: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    for (field, value) in [
        ("schema", serde_json::json!(2)),
        ("ruleset", serde_json::json!("classic_verified")),
        ("maximum_opponents", serde_json::json!(0)),
        ("maximum_opponents", serde_json::json!(33)),
        ("hostility_grace_ms", serde_json::json!(99)),
        ("hostility_grace_ms", serde_json::json!(60001)),
        ("unknown", serde_json::json!(true)),
        ("id", serde_json::json!("")),
        ("maximum_opponents", serde_json::Value::Null),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        assert!(Profile::from_json(&serde_json::to_vec(&changed).unwrap()).is_err());
    }
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
}
