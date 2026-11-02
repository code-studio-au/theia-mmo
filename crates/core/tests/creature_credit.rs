//! Pure authority-boundary checks, not live competing-player reward evidence.
use std::num::NonZeroU64;
use theia_core::{
    creature_credit::{Category, DamageFact, Error, Ledger, MAX_PROFILE_BYTES, Profile},
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
};
const PROFILE: &[u8] = include_bytes!("../../../content/gameplay/development-creature-credit.json");
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn ledger(capacity: usize) -> Ledger {
    let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    raw["maximum_contributors"] = capacity.into();
    Ledger::new(
        Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),
        key(100),
    )
}
fn fact(
    ledger: &Ledger,
    actor: u64,
    generation: u64,
    category: Category,
    sequence: u64,
    damage: u32,
) -> DamageFact {
    DamageFact {
        encounter: ledger.encounter(),
        source: LifeKey {
            actor: key(actor),
            generation: NonZeroU64::new(generation).unwrap(),
        },
        category,
        sequence: NonZeroU64::new(sequence).unwrap(),
        health_lost: damage,
    }
}
#[test]
fn strict_bounded_versioned_profile() {
    let profile = Profile::from_json(PROFILE).unwrap();
    assert_eq!(profile.maximum_contributors(), 32);
    assert_eq!(profile.id().as_str(), "theia.dev.creature-credit");
    for (field, value) in [
        ("schema", serde_json::json!(2)),
        ("ruleset", serde_json::json!("classic")),
        ("maximum_contributors", serde_json::json!(0)),
        ("maximum_contributors", serde_json::json!(33)),
        ("maximum_contributors", serde_json::json!(-1)),
        ("id", serde_json::json!("")),
        ("unknown", serde_json::json!(true)),
    ] {
        let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
        raw[field] = value;
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    assert!(Profile::from_json(&vec![b' '; MAX_PROFILE_BYTES + 1]).is_err());
    assert!(Profile::from_json(b"null").is_err());
}
#[test]
fn first_effective_hit_owns_even_when_competitor_finishes() {
    let mut state = ledger(2);
    let zero = fact(&state, 1, 1, Category::WhiteMelee, 1, 0);
    let before = state.clone();
    assert_eq!(state.record(0, zero), Ok(false));
    assert_eq!(state, before);
    let a = fact(&state, 1, 1, Category::WhiteMelee, 2, 3);
    let b = fact(&state, 2, 1, Category::AbilityDamage, 1, 97);
    assert_eq!(state.record(1, a), Ok(true));
    assert_eq!(state.record(2, b), Ok(true));
    let credit = state.finish(2, state.encounter()).unwrap();
    assert_eq!(credit.owner, Some(key(1)));
    assert_eq!(
        credit
            .contributions
            .iter()
            .map(|c| c.health_lost)
            .collect::<Vec<_>>(),
        [3, 97]
    );
    let before = state.clone();
    assert_eq!(state.finish(2, state.encounter()), Err(Error::Resolved));
    assert_eq!(state.record(3, a), Err(Error::Resolved));
    assert_eq!(state.reset(3, state.encounter()), Err(Error::Resolved));
    assert_eq!(state, before);
}
#[test]
fn independent_damage_sequences_and_respawn_preserve_session_ownership() {
    let mut state = ledger(2);
    for category in [Category::WhiteMelee, Category::AbilityDamage] {
        let f = fact(&state, 1, 1, category, 1, 4);
        assert_eq!(state.record(1, f), Ok(true));
        let before = state.clone();
        assert_eq!(state.record(1, f), Err(Error::Replay));
        assert_eq!(state, before);
    }
    let respawned = fact(&state, 1, 2, Category::WhiteMelee, 1, 5);
    state.record(2, respawned).unwrap();
    let old_life = fact(&state, 1, 1, Category::AbilityDamage, 100, 9);
    let before = state.clone();
    assert_eq!(state.record(3, old_life), Err(Error::StaleLife));
    assert_eq!(state, before);
    assert_eq!(state.owner(), Some(key(1)));
    assert_eq!(state.contributions().next().unwrap().health_lost, 13);
}
#[test]
fn capacity_refusal_is_atomic_zero_damage_consumes_no_slots() {
    let mut state = ledger(1);
    let first = fact(&state, 1, 1, Category::WhiteMelee, 1, 4);
    state.record(1, first).unwrap();
    let zero = fact(&state, 2, 1, Category::WhiteMelee, 1, 0);
    let before = state.clone();
    assert_eq!(state.record(2, zero), Ok(false));
    let full = DamageFact {
        health_lost: 3,
        ..zero
    };
    assert_eq!(state.record(2, full), Err(Error::Capacity));
    assert_eq!(state, before);
    let existing = fact(&state, 1, 1, Category::WhiteMelee, 2, 5);
    assert_eq!(state.record(2, existing), Ok(true));
}
#[test]
fn reset_rejects_delayed_facts_and_new_creature_never_inherits_claims() {
    let mut state = ledger(2);
    let old = fact(&state, 1, 1, Category::WhiteMelee, 1, 4);
    state.record(1, old).unwrap();
    let next = state.reset(2, state.encounter()).unwrap();
    assert_eq!(next.generation.get(), 2);
    assert_eq!(state.owner(), None);
    assert_eq!(state.contributions().count(), 0);
    let before = state.clone();
    assert_eq!(state.record(3, old), Err(Error::StaleEncounter));
    assert_eq!(state, before);
    let new = fact(&state, 2, 1, Category::AbilityDamage, 1, 2);
    state.record(3, new).unwrap();
    assert_eq!(state.owner(), Some(key(2)));
    let mut fresh = Ledger::new(Profile::from_json(PROFILE).unwrap(), key(101));
    assert_eq!(fresh.owner(), None);
    assert_eq!(fresh.record(4, new), Err(Error::StaleEncounter));
}
#[test]
fn wrong_epoch_self_and_backward_tick_refuse_without_mutation() {
    let mut state = ledger(2);
    let valid = fact(&state, 1, 1, Category::WhiteMelee, 1, 2);
    state.record(10, valid).unwrap();
    let before = state.clone();
    let next = fact(&state, 1, 1, Category::WhiteMelee, 2, 2);
    assert_eq!(state.record(9, next), Err(Error::StaleTick));
    assert_eq!(state.finish(9, state.encounter()), Err(Error::StaleTick));
    assert_eq!(state.reset(9, state.encounter()), Err(Error::StaleTick));
    let mut wrong = fact(&state, 2, 1, Category::WhiteMelee, 1, 2);
    wrong.source.actor.epoch = WorldEpoch::new(2).unwrap();
    assert_eq!(state.record(11, wrong), Err(Error::InvalidSource));
    let self_hit = fact(&state, 100, 1, Category::WhiteMelee, 1, 2);
    assert_eq!(state.record(11, self_hit), Err(Error::InvalidSource));
    assert_eq!(state, before);
}
#[test]
fn untagged_death_has_no_implicit_killer_or_entitlements() {
    let mut state = ledger(1);
    let credit = state.finish(0, state.encounter()).unwrap();
    assert_eq!(credit.owner, None);
    assert!(credit.contributions.is_empty());
}
#[test]
fn solo_recipient_requires_explicit_schema_ruleset_and_non_null_policy() {
    use theia_core::creature_credit::RecipientPolicy;
    let bytes = include_bytes!("../../../content/gameplay/development-creature-credit-solo.json");
    assert_eq!(
        Profile::from_json(bytes).unwrap().recipient(),
        Some(RecipientPolicy::SoloTagOwner)
    );
    assert_eq!(Profile::from_json(PROFILE).unwrap().recipient(), None);
    for (field, value) in [
        ("schema", serde_json::json!(1)),
        ("ruleset", serde_json::json!("creature_credit_fixture_v1")),
        ("recipient", serde_json::json!(null)),
        ("recipient", serde_json::json!("last_hitter")),
        ("recipient", serde_json::json!(true)),
    ] {
        let mut raw: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        raw[field] = value;
        assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    }
    let mut raw: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    raw.as_object_mut().unwrap().remove("recipient");
    assert!(Profile::from_json(&serde_json::to_vec(&raw).unwrap()).is_err());
    let mut old: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    old["recipient"] = serde_json::json!("solo_tag_owner");
    assert!(Profile::from_json(&serde_json::to_vec(&old).unwrap()).is_err());
}
