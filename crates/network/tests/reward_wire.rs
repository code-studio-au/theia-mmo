//! Structural bounds and protocol separation, not live/durable award evidence.
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{Catalog as Items, ItemCommand, MAX_REVISION},
    rewards::{Catalog, ClaimCommand, ClaimId},
};
use theia_network::{
    inventory_wire,
    reward_wire::{self as wire, Available, Request, State},
};
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    }
}
fn catalog() -> Catalog {
    let items = Items::from_json(include_bytes!(
        "../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    Catalog::from_json(
        include_bytes!("../../../content/gameplay/development-rewards.json"),
        &items,
    )
    .unwrap()
}
fn request() -> Request {
    Request {
        actor: actor(),
        sequence: 4,
        command: ClaimCommand {
            claim: ClaimId::new([17; 16]).unwrap(),
            expected_revision: 3,
        },
    }
}
#[test]
fn claims_have_an_explicit_canonical_frame_and_cannot_enter_legacy_item_profiles() {
    let request = request();
    let bytes = request.encode().unwrap();
    assert_eq!(bytes.len(), 52);
    assert_eq!(Request::decode(&bytes).unwrap(), request);
    assert!(inventory_wire::Request::decode(&bytes).is_err());
    assert!(inventory_wire::ItemRequest::decode(&bytes).is_err());
    assert!(
        inventory_wire::ItemRequest {
            actor: actor(),
            sequence: 4,
            command: ItemCommand::RewardClaim(request.command)
        }
        .encode()
        .is_err()
    );
    for len in 0..bytes.len() {
        assert!(Request::decode(&bytes[..len]).is_err());
    }
    let mut tail = bytes.clone();
    tail.push(0);
    assert!(Request::decode(&tail).is_err());
    for range in [4..12, 12..20, 20..28, 36..52] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(Request::decode(&bad).is_err());
    }
    let mut bad = bytes.clone();
    bad[28..36].copy_from_slice(&(MAX_REVISION + 1).to_le_bytes());
    assert!(Request::decode(&bad).is_err());
    let mut bad = request;
    bad.sequence = 0;
    assert!(bad.encode().is_err());
    bad = request;
    bad.command.expected_revision = MAX_REVISION + 1;
    assert!(bad.encode().is_err());
}
#[test]
fn private_availability_has_checked_identity_bounds_and_known_unique_recipes() {
    let c = catalog();
    let reward = c.definitions().next().unwrap().id().clone();
    let state = State {
        actor: actor(),
        generation: 1,
        profile_hash: [19; 32],
        available: vec![Available {
            reward,
            claim: request().command.claim,
        }],
    };
    let bytes = state.encode(&c, [19; 32]).unwrap();
    assert_eq!(bytes.len(), 78);
    assert_eq!(State::decode(&bytes, &c, [19; 32]).unwrap(), state);
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], &c, [19; 32]).is_err());
    }
    assert!(State::decode(&bytes, &c, [20; 32]).is_err());
    for range in [4..12, 12..20, 20..28, 62..78] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, &c, [19; 32]).is_err());
    }
    let mut bad = bytes.clone();
    bad[61] = 255;
    assert!(State::decode(&bad, &c, [19; 32]).is_err());
    let mut bad = bytes.clone();
    bad[60] = 17;
    assert!(State::decode(&bad, &c, [19; 32]).is_err());
    let mut bad = bytes.clone();
    bad.extend_from_slice(&bytes[61..]);
    bad[60] = 2;
    assert!(State::decode(&bad, &c, [19; 32]).is_err());
    let mut duplicate = state.clone();
    duplicate.available.push(duplicate.available[0].clone());
    assert!(duplicate.encode(&c, [19; 32]).is_err());
    let mut empty = state;
    empty.available.clear();
    let bytes = empty.encode(&c, [19; 32]).unwrap();
    assert_eq!(bytes.len(), 61);
    assert_eq!(State::decode(&bytes, &c, [19; 32]).unwrap(), empty);
    empty.generation = 0;
    assert!(empty.encode(&c, [19; 32]).is_err());
    assert!(State::decode(&vec![0; wire::MAX_BYTES + 1], &c, [19; 32]).is_err());
}
