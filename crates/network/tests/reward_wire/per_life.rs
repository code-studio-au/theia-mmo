//! Bounded protocol facts only; no live corpse, pickup or SQL authority proof.
use super::*;
use theia_core::{
    coordinates::{GlobalPosition, MAX_GLOBAL_METRES},
    progression::KillId,
    rewards::{CorpseSource, MAX_VISIBLE_LIFE_CLAIMS},
};
fn profile() -> wire::Profile {
    let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    value["schema"] = serde_json::json!(2);
    let mut life = value["rewards"][0].clone();
    life["id"] = serde_json::json!("theia.dev.repeat-training-kill");
    life["scope"] = serde_json::json!("per_creature_life");
    value["rewards"].as_array_mut().unwrap().push(life);
    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(wire::Profile::from_json(&bytes, &items).is_err());
    wire::Profile::from_json_with_creature_loot(&bytes, &items).unwrap()
}
fn source(n: u64, xyz: [f64; 3]) -> CorpseSource {
    CorpseSource {
        life: KillId::from_victim(EntityKey {
            epoch: WorldEpoch::new(51).unwrap(),
            entity: EntityId::new(n).unwrap(),
        }),
        position: GlobalPosition::new(xyz).unwrap(),
    }
}
fn state(profile: &wire::Profile) -> State {
    let ids: Vec<_> = profile
        .catalog()
        .definitions()
        .map(|d| d.id().clone())
        .collect();
    State {
        actor: actor(),
        generation: 1,
        profile_hash: profile.hash(),
        available: vec![
            Available {
                reward: ids[0].clone(),
                claim: ClaimId::new([1; 16]).unwrap(),
                corpse: None,
            },
            Available {
                reward: ids[1].clone(),
                claim: ClaimId::new([2; 16]).unwrap(),
                corpse: Some(source(1, [MAX_GLOBAL_METRES, -MAX_GLOBAL_METRES, 0.])),
            },
            Available {
                reward: ids[1].clone(),
                claim: ClaimId::new([3; 16]).unwrap(),
                corpse: Some(source(2, [-128., 12., 256.])),
            },
        ],
    }
}
#[test]
fn versioned_rows_roundtrip_distinct_lives_and_checked_global_coordinates() {
    let p = profile();
    let state = state(&p);
    let bytes = state.encode(p.catalog(), p.hash()).unwrap();
    assert!(bytes.starts_with(wire::LOOT_STATE_TAG));
    assert_eq!(
        bytes.len(),
        wire::HEADER_BYTES + (wire::ROW_BYTES + 1) + 2 * wire::LOOT_ROW_BYTES
    );
    assert_eq!(State::decode(&bytes, p.catalog(), p.hash()).unwrap(), state);
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], p.catalog(), p.hash()).is_err());
    }
    let mut old = bytes.clone();
    old[..4].copy_from_slice(wire::STATE_TAG);
    assert!(State::decode(&old, p.catalog(), p.hash()).is_err());
    let legacy = super::catalog();
    assert!(State::decode(&bytes, &legacy, p.hash()).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(State::decode(&trailing, p.catalog(), p.hash()).is_err());
    let mut wrong = bytes.clone();
    wrong[60] = 255;
    assert!(State::decode(&wrong, p.catalog(), p.hash()).is_err());
    for range in [4..12, 12..20, 20..28, 28..60] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, p.catalog(), p.hash()).is_err());
    }
}
#[test]
fn malformed_life_keys_locations_kinds_and_recipe_bindings_refuse() {
    let p = profile();
    let state = state(&p);
    let bytes = state.encode(p.catalog(), p.hash()).unwrap();
    let row = wire::HEADER_BYTES + wire::ROW_BYTES + 1;
    let kind = row + wire::ROW_BYTES;
    let life = kind + 1;
    let position = life + 16;
    for range in [life..life + 8, life + 8..life + 16, row + 1..row + 17] {
        let mut bad = bytes.clone();
        bad[range].fill(0);
        assert!(State::decode(&bad, p.catalog(), p.hash()).is_err());
    }
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        MAX_GLOBAL_METRES + 1.,
    ] {
        for axis in 0..3 {
            let mut bad = bytes.clone();
            let at = position + axis * 8;
            bad[at..at + 8].copy_from_slice(&value.to_le_bytes());
            assert!(State::decode(&bad, p.catalog(), p.hash()).is_err());
        }
    }
    for value in [0, 2, 255] {
        let mut bad = bytes.clone();
        bad[kind] = value;
        assert!(State::decode(&bad, p.catalog(), p.hash()).is_err());
    }
    let mut bad = bytes.clone();
    bad[row] = 0;
    assert!(State::decode(&bad, p.catalog(), p.hash()).is_err());
    let mut missing = state.clone();
    missing.available[1].corpse = None;
    assert!(missing.encode(p.catalog(), p.hash()).is_err());
    let mut extra = state;
    extra.available[0].corpse = Some(source(1, [0.; 3]));
    assert!(extra.encode(p.catalog(), p.hash()).is_err());
}
#[test]
fn receipt_life_and_once_recipe_duplicates_and_noncanonical_order_refuse() {
    let p = profile();
    let original = state(&p);
    let mut wrong = original.clone();
    wrong.available[2].claim = wrong.available[1].claim;
    assert!(wrong.encode(p.catalog(), p.hash()).is_err());
    wrong = original.clone();
    wrong.available[2].corpse = wrong.available[1].corpse;
    assert!(wrong.encode(p.catalog(), p.hash()).is_err());
    wrong = original.clone();
    wrong.available.swap(1, 2);
    assert!(wrong.encode(p.catalog(), p.hash()).is_err());
    wrong = original.clone();
    wrong.available[1].reward = wrong.available[0].reward.clone();
    wrong.available[1].corpse = None;
    assert!(wrong.encode(p.catalog(), p.hash()).is_err());
    let mut empty = original;
    empty.available.clear();
    let bytes = empty.encode(p.catalog(), p.hash()).unwrap();
    assert_eq!(bytes.len(), wire::HEADER_BYTES);
    assert_eq!(State::decode(&bytes, p.catalog(), p.hash()).unwrap(), empty);
}
#[test]
fn multiple_items_from_one_corpse_require_one_consistent_source_position() {
    let base = profile();
    let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    value["schema"] = 2.into();
    value["rewards"][0]["scope"] = "per_creature_life".into();
    let mut second = value["rewards"][0].clone();
    second["id"] = "theia.dev.repeat-training-kill".into();
    value["rewards"].as_array_mut().unwrap().push(second);
    let p =
        wire::Profile::from_json_with_creature_loot(&serde_json::to_vec(&value).unwrap(), &items)
            .unwrap();
    let mut state = state(&base);
    state.profile_hash = p.hash();
    state.available.truncate(2);
    state.available[0].corpse = Some(source(1, [0.; 3]));
    state.available[1].corpse = state.available[0].corpse;
    let bytes = state.encode(p.catalog(), p.hash()).unwrap();
    assert_eq!(State::decode(&bytes, p.catalog(), p.hash()).unwrap(), state);
    state.available[1].corpse = Some(source(1, [1., 0., 0.]));
    assert!(state.encode(p.catalog(), p.hash()).is_err());
}
#[test]
fn maximum_mixed_page_is_bounded_and_seventeenth_life_claim_refuses() {
    let items = theia_network::renet_runtime::inventory::Profile::from_json(include_bytes!(
        "../../../../content/gameplay/development-equipment-items.json"
    ))
    .unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../content/gameplay/development-rewards.json"
    ))
    .unwrap();
    value["schema"] = 2.into();
    let original = value["rewards"][0].clone();
    let mut definitions = Vec::new();
    for n in 0..15 {
        let mut d = original.clone();
        d["id"] = format!("theia.dev.bonus-{n:02}").into();
        definitions.push(d);
    }
    let mut repeatable = original;
    repeatable["id"] = "theia.dev.z-repeat".into();
    repeatable["scope"] = "per_creature_life".into();
    definitions.push(repeatable);
    value["rewards"] = definitions.into();
    let p =
        wire::Profile::from_json_with_creature_loot(&serde_json::to_vec(&value).unwrap(), &items)
            .unwrap();
    let ids: Vec<_> = p.catalog().definitions().map(|d| d.id().clone()).collect();
    let mut available: Vec<_> = ids[..15]
        .iter()
        .enumerate()
        .map(|(n, id)| Available {
            reward: id.clone(),
            claim: ClaimId::new([n as u8 + 1; 16]).unwrap(),
            corpse: None,
        })
        .collect();
    for n in 0..MAX_VISIBLE_LIFE_CLAIMS {
        available.push(Available {
            reward: ids[15].clone(),
            claim: ClaimId::new([n as u8 + 16; 16]).unwrap(),
            corpse: Some(source(n as u64 + 1, [n as f64, 0., 0.])),
        });
    }
    let mut state = State {
        actor: actor(),
        generation: 1,
        profile_hash: p.hash(),
        available,
    };
    let bytes = state.encode(p.catalog(), p.hash()).unwrap();
    assert_eq!(state.available.len(), 31);
    assert_eq!(bytes.len(), 1259);
    assert!(bytes.len() <= wire::MAX_LOOT_BYTES);
    assert_eq!(State::decode(&bytes, p.catalog(), p.hash()).unwrap(), state);
    state.available.push(Available {
        reward: ids[15].clone(),
        claim: ClaimId::new([255; 16]).unwrap(),
        corpse: Some(source(100, [0.; 3])),
    });
    assert!(state.encode(p.catalog(), p.hash()).is_err());
    assert!(State::decode(&vec![0; wire::MAX_LOOT_BYTES + 1], p.catalog(), p.hash()).is_err());
}
