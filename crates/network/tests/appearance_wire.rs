//! Bounded public cosmetic state, separate from private backpack observations.
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::Catalog,
};
use theia_network::{
    appearance_wire::{self as wire, Actor, State},
    terrain::sha256,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
fn actor(id: u64, hand: Option<&str>) -> Actor {
    Actor {
        key: EntityKey {
            epoch: WorldEpoch::new(7).unwrap(),
            entity: EntityId::new(id).unwrap(),
        },
        main_hand: hand.and_then(ContentId::new),
    }
}
fn state() -> State {
    State {
        epoch: 7,
        tick: 20,
        catalog_hash: sha256(DATA),
        actors: vec![actor(1, Some("theia.dev.simple-sword")), actor(2, None)],
    }
}
#[test]
fn public_state_is_bounded_sorted_cosmetic_data_and_excludes_private_inventory() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let view = state();
    let bytes = view.encode(&catalog, sha256(DATA)).unwrap();
    assert_eq!(bytes.len(), 71);
    assert_eq!(State::decode(&bytes, &catalog, sha256(DATA)).unwrap(), view);
    let mut newer = view.clone();
    newer.tick += 1;
    assert!(view.same_view(&newer));
    newer.actors[0].main_hand = None;
    assert!(!view.same_view(&newer));
    let full = State {
        actors: (1..=wire::MAX_ACTORS)
            .map(|i| actor(i as u64, Some("theia.dev.simple-wand")))
            .collect(),
        ..view
    };
    let bytes = full.encode(&catalog, sha256(DATA)).unwrap();
    assert_eq!(bytes.len(), wire::MAX_BYTES);
    assert!(bytes.len() < 1200);
    assert_eq!(State::decode(&bytes, &catalog, sha256(DATA)).unwrap(), full);
}
#[test]
fn malformed_foreign_or_unordered_public_state_rejects_without_partial_results() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let view = state();
    let bytes = view.encode(&catalog, sha256(DATA)).unwrap();
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], &catalog, sha256(DATA)).is_err());
    }
    let mut tail = bytes.clone();
    tail.push(0);
    assert!(State::decode(&tail, &catalog, sha256(DATA)).is_err());
    for (offset, value) in [(0, b'X'), (52, 9), (61, 64)] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(State::decode(&bad, &catalog, sha256(DATA)).is_err());
    }
    for range in [4..12, 12..20, 53..61] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, &catalog, sha256(DATA)).is_err());
    }
    let mut duplicate = bytes.clone();
    duplicate[62..70].copy_from_slice(&1_u64.to_le_bytes());
    assert!(State::decode(&duplicate, &catalog, sha256(DATA)).is_err());
    assert!(State::decode(&bytes, &catalog, [1; 32]).is_err());
    assert!(State::decode(&bytes, &catalog, [0; 32]).is_err());
    for actors in [
        vec![actor(2, None), actor(1, None)],
        vec![actor(1, None), actor(1, None)],
        vec![actor(1, Some("theia.dev.unpublished"))],
        (1..=9).map(|i| actor(i, None)).collect(),
    ] {
        assert!(
            State {
                actors,
                ..view.clone()
            }
            .encode(&catalog, sha256(DATA))
            .is_err()
        );
    }
    let mut foreign = view.clone();
    foreign.actors[0].key.epoch = WorldEpoch::new(8).unwrap();
    assert!(foreign.encode(&catalog, sha256(DATA)).is_err());
    let old = Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))
    .unwrap();
    assert!(view.encode(&old, sha256(DATA)).is_err());
    assert!(State::decode(&bytes, &old, sha256(DATA)).is_err());
}
#[test]
fn repeated_appearance_names_share_one_canonical_ordinal() {
    let data=serde_json::to_vec(&serde_json::json!({"schema":2,"id":"theia.test.items","items":[{"id":"theia.test.a","name":"Sword","maximum_stack":1,"main_hand_appearance":"theia.test.sword"},{"id":"theia.test.b","name":"Other Sword","maximum_stack":1,"main_hand_appearance":"theia.test.sword"}]})).unwrap();
    let catalog = Catalog::from_json(&data).unwrap();
    let view = State {
        catalog_hash: sha256(&data),
        actors: vec![actor(1, Some("theia.test.sword"))],
        ..state()
    };
    let bytes = view.encode(&catalog, sha256(&data)).unwrap();
    assert_eq!(bytes[61], 0);
    assert_eq!(
        State::decode(&bytes, &catalog, sha256(&data)).unwrap(),
        view
    );
    let mut bad = bytes;
    bad[61] = 1;
    assert!(State::decode(&bad, &catalog, sha256(&data)).is_err());
}
