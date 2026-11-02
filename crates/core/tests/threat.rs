//! Pure arithmetic/authority-boundary tests; no live victim-switching claim.
use std::num::NonZeroU64;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
    threat::{Eligible, Error, MAX_PROFILE_BYTES, Profile, Table},
};
const PROFILE: &[u8] = include_bytes!("../../../content/gameplay/development-creature-threat.json");
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(1).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn life(id: u64, generation: u64) -> LifeKey {
    LifeKey {
        actor: key(id),
        generation: NonZeroU64::new(generation).unwrap(),
    }
}
fn table(capacity: usize) -> Table {
    let mut raw: serde_json::Value = serde_json::from_slice(PROFILE).unwrap();
    raw["maximum_participants"] = capacity.into();
    Table::new(
        Profile::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),
        key(100),
    )
}
fn eligible(id: u64, in_melee: bool) -> Eligible {
    Eligible {
        life: life(id, 1),
        in_melee,
    }
}
fn pull(table: &mut Table) {
    table.add_damage(life(1, 1), 100).unwrap();
    assert_eq!(table.select(&[eligible(1, true)]), Ok(Some(life(1, 1))));
}
#[test]
fn strict_bounded_empirical_profile() {
    let profile = Profile::from_json(PROFILE).unwrap();
    assert_eq!(profile.maximum_participants(), 32);
    assert_eq!(profile.id().as_str(), "theia.dev.creature-threat");
    for (field, value) in [
        ("schema", serde_json::json!(2)),
        ("ruleset", serde_json::json!("classic_verified")),
        ("maximum_participants", serde_json::json!(0)),
        ("maximum_participants", serde_json::json!(33)),
        ("melee_transfer_basis_points", serde_json::json!(10000)),
        ("ranged_transfer_basis_points", serde_json::json!(11000)),
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
fn only_actual_health_lost_adds_threat_and_zero_does_not_consume_capacity() {
    let mut t = table(1);
    let before = t.clone();
    t.add_damage(life(1, 1), 0).unwrap();
    assert_eq!(t, before);
    t.register(life(1, 1)).unwrap();
    t.register(life(1, 1)).unwrap();
    assert_eq!(t.entries()[0].damage_threat, 0);
    t.add_damage(life(1, 1), 17).unwrap();
    t.add_damage(life(1, 1), 23).unwrap();
    assert_eq!(t.entries()[0].damage_threat, 40);
    let before = t.clone();
    assert_eq!(t.add_damage(life(2, 1), 1), Err(Error::Capacity));
    assert_eq!(t, before);
    t.add_damage(life(2, 1), 0).unwrap();
    assert_eq!(t, before);
}
#[test]
fn melee_boundary_retains_at_110_percent_and_transfers_above_it() {
    let mut t = table(2);
    pull(&mut t);
    t.add_damage(life(2, 1), 110).unwrap();
    let e = [eligible(1, true), eligible(2, true)];
    assert_eq!(t.select(&e), Ok(Some(life(1, 1))));
    t.add_damage(life(2, 1), 1).unwrap();
    assert_eq!(t.select(&e), Ok(Some(life(2, 1))));
}
#[test]
fn ranged_boundary_and_moving_into_melee_use_current_reach() {
    let mut t = table(2);
    pull(&mut t);
    t.add_damage(life(2, 1), 130).unwrap();
    let e = [eligible(1, true), eligible(2, false)];
    assert_eq!(t.select(&e), Ok(Some(life(1, 1))));
    let mut ranged = t.clone();
    ranged.add_damage(life(2, 1), 1).unwrap();
    assert_eq!(ranged.select(&e), Ok(Some(life(2, 1))));
    // No extra damage: the challenger enters melee with its existing threat.
    assert_eq!(
        t.select(&[eligible(1, true), eligible(2, true)]),
        Ok(Some(life(2, 1)))
    );
}
#[test]
fn nonqualifying_ranged_leader_does_not_hide_a_qualifying_melee_challenger() {
    let mut t = table(3);
    pull(&mut t);
    t.add_damage(life(2, 1), 120).unwrap();
    t.add_damage(life(3, 1), 111).unwrap();
    assert_eq!(
        t.select(&[eligible(1, true), eligible(2, false), eligible(3, true)]),
        Ok(Some(life(3, 1)))
    );
}
#[test]
fn initial_and_qualifying_ties_are_stable_and_existing_equal_threat_is_sticky() {
    for reversed in [false, true] {
        let mut t = table(3);
        let ids = if reversed { [3, 2, 1] } else { [1, 2, 3] };
        for id in ids {
            t.register(life(id, 1)).unwrap();
        }
        let e: Vec<_> = ids.into_iter().map(|id| eligible(id, true)).collect();
        assert_eq!(t.select(&e), Ok(Some(life(1, 1))));
        t.add_damage(life(1, 1), 100).unwrap();
        t.add_damage(life(2, 1), 111).unwrap();
        t.add_damage(life(3, 1), 111).unwrap();
        assert_eq!(t.select(&e), Ok(Some(life(2, 1))));
        assert_eq!(t.select(&e), Ok(Some(life(2, 1))));
    }
}
#[test]
fn exact_life_removal_prevents_threat_carryover_or_stale_removal_after_respawn() {
    let mut t = table(2);
    pull(&mut t);
    let before = t.clone();
    assert_eq!(t.add_damage(life(1, 2), 10), Err(Error::DifferentLife));
    assert_eq!(t, before);
    t.remove(life(1, 1));
    assert_eq!(t.current(), None);
    t.add_damage(life(1, 2), 7).unwrap();
    t.remove(life(1, 1));
    assert_eq!(t.entries().len(), 1);
    assert_eq!(t.entries()[0].damage_threat, 7);
    assert_eq!(
        t.select(&[Eligible {
            life: life(1, 2),
            in_melee: true
        }]),
        Ok(Some(life(1, 2)))
    );
}
#[test]
fn eligibility_errors_are_atomic_and_invalid_victim_can_be_replaced() {
    let mut t = table(2);
    pull(&mut t);
    t.add_damage(life(2, 1), 1).unwrap();
    let before = t.clone();
    for e in [
        vec![eligible(1, true), eligible(1, false)],
        vec![eligible(3, true)],
        vec![eligible(1, true), eligible(2, true), eligible(3, true)],
    ] {
        assert_eq!(t.select(&e), Err(Error::InvalidEligibility));
        assert_eq!(t, before);
    }
    assert_eq!(t.select(&[eligible(2, false)]), Ok(Some(life(2, 1))));
    assert_eq!(t.select(&[]), Ok(None));
}
#[test]
fn foreign_epochs_self_sources_and_encounter_clear_are_explicit() {
    let mut t = table(2);
    pull(&mut t);
    let before = t.clone();
    let mut foreign = life(2, 1);
    foreign.actor.epoch = WorldEpoch::new(2).unwrap();
    assert_eq!(t.add_damage(foreign, 10), Err(Error::InvalidSource));
    assert_eq!(t.register(life(100, 1)), Err(Error::InvalidSource));
    assert_eq!(t, before);
    t.clear();
    assert!(t.entries().is_empty());
    assert_eq!(t.current(), None);
    t.register(life(2, 1)).unwrap();
    assert_eq!(t.entries()[0].damage_threat, 0);
}
