//! Status codec only; no transport or live effect claim.
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    statuses::Catalog,
    time::TickRate,
};
use theia_network::{
    simulation::combat::Status,
    status_wire::{self as wire, State},
};
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn catalog() -> Catalog {
    Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-statuses.json"
    ))
    .unwrap()
}
fn state(rate: TickRate) -> State {
    let hz = u64::from(rate.hz());
    State {
        epoch: 7,
        tick: 100,
        rate,
        statuses: vec![Status {
            source: key(2),
            target: key(1),
            status: ContentId::new("theia.dev.scorch").unwrap(),
            expires_at: 100 + 6 * hz,
            next_tick: Some(100 + hz),
        }],
    }
}
#[test]
fn exact_status_frames_roundtrip_every_rate_and_reject_truncations_fields_and_trailing_bytes() {
    let catalog = catalog();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let value = state(rate);
        let bytes = value.encode(&catalog).unwrap();
        assert_eq!(bytes.len(), 55);
        assert_eq!(State::decode(&bytes, &catalog).unwrap(), value);
        for end in 0..bytes.len() {
            assert!(State::decode(&bytes[..end], &catalog).is_err());
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(State::decode(&bad, &catalog).is_err());
        for (offset, byte) in [(0, b'X'), (20, 0), (21, 17), (38, 255)] {
            let mut bad = bytes.clone();
            bad[offset] = byte;
            assert!(State::decode(&bad, &catalog).is_err());
        }
        for offset in [4, 22, 30] {
            let mut bad = bytes.clone();
            bad[offset..offset + 8].fill(0);
            assert!(State::decode(&bad, &catalog).is_err());
        }
        for (offset, word) in [
            (39, 100),
            (39, 100 + 6 * u64::from(rate.hz()) + 1),
            (47, 100),
            (47, 100 + u64::from(rate.hz()) + 1),
        ] {
            let mut bad = bytes.clone();
            bad[offset..offset + 8].copy_from_slice(&word.to_le_bytes());
            assert!(State::decode(&bad, &catalog).is_err());
        }
        let mut none = value.clone();
        none.statuses[0].next_tick = None;
        assert!(
            none.encode(&catalog).is_err(),
            "An active full interval cannot omit its next tick"
        );
        none.statuses[0].expires_at = none.tick + u64::from(rate.hz()) - 1;
        assert_eq!(
            State::decode(&none.encode(&catalog).unwrap(), &catalog).unwrap(),
            none
        );
        let empty = State {
            statuses: vec![],
            ..value
        };
        assert_eq!(empty.encode(&catalog).unwrap().len(), wire::HEADER_BYTES);
    }
}
#[test]
fn duplicates_unsorted_foreign_sources_and_target_capacity_fail_before_publication() {
    let catalog = catalog();
    let original = state(TickRate::Hz30);
    let mut bad = original.clone();
    bad.statuses.push(bad.statuses[0].clone());
    assert!(bad.encode(&catalog).is_err());
    let mut bad = original.clone();
    bad.statuses[0].source = bad.statuses[0].target;
    assert!(bad.encode(&catalog).is_err());
    let mut bad = original.clone();
    bad.statuses[0].source.epoch = WorldEpoch::new(8).unwrap();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = original.clone();
    bad.statuses[0].status = ContentId::new("theia.unknown.status").unwrap();
    assert!(bad.encode(&catalog).is_err());
    let mut bad = original.clone();
    bad.statuses = (2..=10)
        .map(|id| Status {
            source: key(id),
            ..original.statuses[0].clone()
        })
        .collect();
    assert!(
        bad.encode(&catalog).is_err(),
        "Per-target limit is independent of the global byte budget"
    );
    let mut bad = original.clone();
    bad.statuses = vec![
        Status {
            source: key(3),
            ..original.statuses[0].clone()
        },
        original.statuses[0].clone(),
    ];
    assert!(bad.encode(&catalog).is_err());
    // Mutating an otherwise exact-length frame must also reject duplicate keys.
    let mut two = original.clone();
    two.statuses.push(Status {
        source: key(3),
        ..two.statuses[0].clone()
    });
    let mut bytes = two.encode(&catalog).unwrap();
    bytes[wire::HEADER_BYTES + wire::STATUS_BYTES..wire::HEADER_BYTES + wire::STATUS_BYTES + 8]
        .copy_from_slice(&2_u64.to_le_bytes());
    assert!(State::decode(&bytes, &catalog).is_err());
}
#[test]
fn maximum_world_status_set_fits_a_550_byte_frame_and_unchanged_views_ignore_only_tick() {
    let catalog = catalog();
    let mut value = state(TickRate::Hz60);
    let prototype = value.statuses[0].clone();
    value.statuses = (0..2)
        .flat_map(|target| (0..8).map(move |source| (target, source)))
        .map(|(target, source)| Status {
            target: key(target + 1),
            source: key(source + 10),
            ..prototype.clone()
        })
        .collect();
    let bytes = value.encode(&catalog).unwrap();
    assert_eq!(bytes.len(), wire::MAX_BYTES);
    assert_eq!(bytes.len(), 550);
    assert_eq!(State::decode(&bytes, &catalog).unwrap(), value);
    let mut newer = value.clone();
    newer.tick += 1;
    assert!(value.same_view(&newer));
    newer.statuses.pop();
    assert!(!value.same_view(&newer));
}
