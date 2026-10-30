//! Bounded private objective observations, not live entitlement or SQL proof.
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    quests::State as Progress,
};
use theia_network::quest_wire::{Profile, STATE_BYTES, State};
fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-quest.json"
    ))
    .unwrap()
}
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    }
}
fn state(profile: &Profile, revision: u64, count: u32) -> State {
    State {
        actor: actor(),
        profile_hash: profile.hash(),
        progress: Progress::new(profile.profile(), revision, count).unwrap(),
    }
}
#[test]
fn objective_observations_bind_exact_actor_profile_and_finite_counter() {
    let p = profile();
    let s = state(&p, 2, 2);
    let bytes = s.encode(&p).unwrap();
    assert_eq!(bytes.len(), STATE_BYTES);
    assert_eq!(State::decode(&bytes, &p).unwrap(), s);
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], &p).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(State::decode(&bad, &p).is_err());
    for range in [4..12, 12..20, 20..52] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, &p).is_err());
    }
    let mut bad = bytes.clone();
    bad[52..60].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
    let mut bad = bytes.clone();
    bad[60..64].copy_from_slice(&4u32.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
    let padded = Profile::from_json(
        &[
            include_bytes!("../../../content/gameplay/development-quest.json").as_slice(),
            b"\n",
        ]
        .concat(),
    )
    .unwrap();
    assert!(State::decode(&bytes, &padded).is_err());
    let xp = theia_network::progression_wire::Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))
    .unwrap();
    assert!(theia_network::progression_wire::State::decode(&bytes, &xp).is_err());
    assert!(theia_network::inventory_wire::Request::decode(&bytes).is_err());
    assert!(theia_network::reward_wire::Request::decode(&bytes).is_err());
}
#[test]
fn private_coalesced_updates_accept_skips_but_reject_aliases_regression_and_other_actors() {
    let p = profile();
    let old = state(&p, 1, 1);
    old.validate_successor(actor(), Some(&old)).unwrap();
    state(&p, 3, 3)
        .validate_successor(actor(), Some(&old))
        .unwrap();
    for bad in [
        state(&p, 0, 0),
        state(&p, 1, 2),
        state(&p, 2, 1),
        state(&p, 2, 0),
    ] {
        assert!(bad.validate_successor(actor(), Some(&old)).is_err());
    }
    assert!(
        old.validate_successor(
            EntityKey {
                entity: EntityId::new(10).unwrap(),
                ..actor()
            },
            None
        )
        .is_err()
    );
}
