//! Structural/private observation checks, distinct from actual SQL qualification.
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    progression::State as Progress,
};
use theia_network::progression_wire::{Profile, STATE_BYTES, State};
fn profile() -> Profile {
    Profile::from_json(include_bytes!(
        "../../../content/gameplay/development-progression.json"
    ))
    .unwrap()
}
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    }
}
fn state(profile: &Profile, revision: u64, total: u32) -> State {
    State {
        actor: actor(),
        profile_hash: profile.hash(),
        progress: Progress::new(profile.profile(), revision, total).unwrap(),
    }
}
#[test]
fn observations_have_exact_profile_actor_and_bounded_integer_state() {
    let p = profile();
    let s = state(&p, 4, 100);
    let bytes = s.encode(&p).unwrap();
    assert_eq!(bytes.len(), STATE_BYTES);
    assert_eq!(State::decode(&bytes, &p).unwrap(), s);
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], &p).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(State::decode(&extra, &p).is_err());
    for range in [4..12, 12..20, 20..52] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, &p).is_err());
    }
    let mut bad = bytes.clone();
    bad[52..60].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
    let mut bad = bytes.clone();
    bad[60..64].copy_from_slice(&251u32.to_le_bytes());
    assert!(State::decode(&bad, &p).is_err());
    let padded = Profile::from_json(
        &[
            include_bytes!("../../../content/gameplay/development-progression.json").as_slice(),
            b"\n",
        ]
        .concat(),
    )
    .unwrap();
    assert!(State::decode(&bytes, &padded).is_err());
    assert!(theia_network::inventory_wire::Request::decode(&bytes).is_err());
    assert!(theia_network::reward_wire::Request::decode(&bytes).is_err());
}
#[test]
fn coalesced_commits_may_skip_revisions_but_cannot_regress_or_alias() {
    let p = profile();
    let old = state(&p, 1, 25);
    state(&p, 4, 100)
        .validate_successor(actor(), Some(&old))
        .unwrap();
    old.validate_successor(actor(), Some(&old)).unwrap();
    for bad in [
        state(&p, 0, 0),
        state(&p, 1, 50),
        state(&p, 2, 25),
        state(&p, 2, 0),
    ] {
        assert!(bad.validate_successor(actor(), Some(&old)).is_err());
    }
    let other = EntityKey {
        entity: EntityId::new(10).unwrap(),
        ..actor()
    };
    assert!(old.validate_successor(other, None).is_err());
}
