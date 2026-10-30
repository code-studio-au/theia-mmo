//! Bounded codec contracts, not authenticated item commands or native inventory.
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{Bag, Catalog, Command, MAX_REVISION, Operation, Stack},
};
use theia_network::{
    inventory_wire::{self as wire, Outcome, Reason, Reply, Request, State},
    terrain::sha256,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-items.json");
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    }
}
fn request(operation: Operation) -> Request {
    Request {
        actor: actor(),
        sequence: 1,
        command: Command {
            expected_revision: 0,
            operation,
        },
    }
}
fn state(catalog: &Catalog) -> State {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(
        catalog,
        ContentId::new("theia.dev.training-stone").unwrap(),
        20,
    );
    State {
        actor: actor(),
        catalog_hash: sha256(DATA),
        bag: Bag::from_slots(catalog, 3, slots).unwrap(),
        reply: None,
    }
}
#[test]
fn requests_are_fixed_checked_and_cannot_upload_arbitrary_item_contents() {
    for operation in [
        Operation::Move { from: 0, to: 15 },
        Operation::Split {
            from: 15,
            to: 0,
            amount: 19,
        },
    ] {
        let value = request(operation);
        let bytes = value.encode().unwrap();
        assert_eq!(bytes.len(), wire::REQUEST_BYTES);
        assert_eq!(Request::decode(&bytes).unwrap(), value);
        for n in 0..bytes.len() {
            assert!(Request::decode(&bytes[..n]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Request::decode(&trailing).is_err());
        for offset in [4, 12, 20] {
            let mut bad = bytes.clone();
            bad[offset..offset + 8].fill(0);
            assert!(Request::decode(&bad).is_err());
        }
        for (offset, byte) in [(0, b'X'), (36, 2), (37, 16), (38, 255)] {
            let mut bad = bytes.clone();
            bad[offset] = byte;
            assert!(Request::decode(&bad).is_err());
        }
        let mut bad = bytes.clone();
        bad[28..36].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(Request::decode(&bad).is_err());
        let mut bad = bytes.clone();
        bad[39..43].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Request::decode(&bad).is_err());
    }
    let mut alias = request(Operation::Move { from: 0, to: 1 })
        .encode()
        .unwrap();
    alias[39] = 1;
    assert!(
        Request::decode(&alias).is_err(),
        "Move must use canonical zero amount"
    );
    let mut maximum = request(Operation::Split {
        from: 0,
        to: 1,
        amount: 10_000,
    });
    maximum.command.expected_revision = MAX_REVISION;
    assert_eq!(
        Request::decode(&maximum.encode().unwrap()).unwrap(),
        maximum
    );
    // Amount zero/self moves are structurally bounded requests; the authority
    // rejects their gameplay semantics without treating them as committed work.
    assert!(
        request(Operation::Split {
            from: 0,
            to: 0,
            amount: 0
        })
        .encode()
        .is_ok()
    );
}
#[test]
fn snapshots_roundtrip_all_replies_and_reject_every_truncation_or_malformed_field() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let hash = sha256(DATA);
    let original = state(&catalog);
    let mut outcomes = vec![Outcome::Applied, Outcome::Replayed];
    outcomes.extend(
        [
            Reason::Stale,
            Reason::Conflict,
            Reason::SameSlot,
            Reason::EmptySource,
            Reason::Full,
            Reason::Occupied,
            Reason::Amount,
            Reason::Exhausted,
            Reason::Busy,
            Reason::Slot,
        ]
        .map(Outcome::Rejected),
    );
    for outcome in outcomes {
        let mut value = original.clone();
        value.reply = Some(Reply {
            sequence: 17,
            outcome,
        });
        let bytes = value.encode(&catalog, hash).unwrap();
        assert_eq!(bytes.len(), wire::STATE_BYTES);
        assert_eq!(State::decode(&bytes, &catalog, hash).unwrap(), value);
    }
    let bytes = original.encode(&catalog, hash).unwrap();
    for n in 0..bytes.len() {
        assert!(State::decode(&bytes[..n], &catalog, hash).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(State::decode(&trailing, &catalog, hash).is_err());
    for offset in [4, 12] {
        let mut bad = bytes.clone();
        bad[offset..offset + 8].fill(0);
        assert!(State::decode(&bad, &catalog, hash).is_err());
    }
    for (offset, byte) in [
        (0, b'X'),
        (20, hash[0] ^ 255),
        (68, 255),
        (69, 64),
        (70, 0),
        (72, 0),
        (73, 1),
    ] {
        let mut bad = bytes.clone();
        bad[offset] = byte;
        assert!(
            State::decode(&bad, &catalog, hash).is_err(),
            "offset {offset}"
        );
    }
    let mut bad = bytes.clone();
    bad[70..72].copy_from_slice(&21_u16.to_le_bytes());
    assert!(State::decode(&bad, &catalog, hash).is_err());
    let mut bad = bytes.clone();
    bad[52..60].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(State::decode(&bad, &catalog, hash).is_err());
    let mut bad = bytes.clone();
    bad[60] = 1;
    assert!(
        State::decode(&bad, &catalog, hash).is_err(),
        "No reply flag with nonzero sequence"
    );
    assert!(State::decode(&bytes, &catalog, [0; 32]).is_err());
    assert!(State::decode(&bytes, &catalog, [1; 32]).is_err());
    let mut zero = original.clone();
    zero.bag = Bag::empty(&catalog);
    zero.reply = Some(Reply {
        sequence: 1,
        outcome: Outcome::Applied,
    });
    assert!(
        zero.encode(&catalog, hash).is_err(),
        "No successful revision-zero mutation"
    );
    let mut bad = bytes.clone();
    bad[52..60].fill(0);
    bad[60] = 1;
    bad[68] = 1;
    assert!(State::decode(&bad, &catalog, hash).is_err());
}
#[test]
fn largest_catalog_and_full_backpack_still_fit_one_small_reliable_frame() {
    let items:Vec<_>=(0..64).map(|i|serde_json::json!({"id":format!("theia.test.item-{i:02}"),"name":"Test Item","maximum_stack":10_000})).collect();
    let data =
        serde_json::to_vec(&serde_json::json!({"schema":1,"id":"theia.test.items","items":items}))
            .unwrap();
    let catalog = Catalog::from_json(&data).unwrap();
    let hash = sha256(&data);
    let definitions: Vec<_> = catalog.definitions().collect();
    let slots =
        std::array::from_fn(|i| Stack::new(&catalog, definitions[63 - i].id().clone(), 10_000));
    let value = State {
        actor: actor(),
        catalog_hash: hash,
        bag: Bag::from_slots(&catalog, MAX_REVISION, slots).unwrap(),
        reply: Some(Reply {
            sequence: u64::MAX,
            outcome: Outcome::Replayed,
        }),
    };
    let bytes = value.encode(&catalog, hash).unwrap();
    assert_eq!(bytes.len(), 117);
    assert!(bytes.len() < 1200);
    assert_eq!(State::decode(&bytes, &catalog, hash).unwrap(), value);
}
#[test]
fn legacy_wire_never_drops_equipped_state_or_accepts_cross_profile_frames() {
    let legacy = Catalog::from_json(DATA).unwrap();
    let old = state(&legacy);
    let bytes = old.encode(&legacy, sha256(DATA)).unwrap();
    let data = include_bytes!("../../../content/gameplay/development-equipment-items.json");
    let mut raw: serde_json::Value = serde_json::from_slice(data).unwrap();
    // A matching logical name cannot make a new state compatible with IS01.
    raw["id"] = serde_json::json!(legacy.id().as_str());
    let data = serde_json::to_vec(&raw).unwrap();
    let catalog = Catalog::from_json(&data).unwrap();
    assert!(State::decode(&bytes, &catalog, sha256(DATA)).is_err());
    assert!(old.encode(&catalog, sha256(DATA)).is_err());
    let mut newer = old;
    newer.bag = Bag::empty(&catalog);
    assert!(newer.encode(&legacy, sha256(DATA)).is_err());
    assert!(theia_network::renet_runtime::inventory::Profile::from_json(&data).is_ok());
    assert!(theia_network::renet_runtime::inventory::Profile::from_json(DATA).is_ok());
}
