//! Profile-2 structural contracts; no durable ownership or native appearance claim.
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{
        Bag, Catalog, Command, EquipmentCommand, EquipmentOperation as Op, ItemCommand,
        MAX_REVISION, Operation, Stack,
    },
};
use theia_network::{
    inventory_wire::{self as wire, ItemRequest, Outcome, Reason, Reply, State},
    terrain::sha256,
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
fn actor() -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    }
}
fn equip(op: Op) -> ItemCommand {
    EquipmentCommand {
        expected_revision: 3,
        operation: op,
    }
    .into()
}
fn request(command: ItemCommand) -> ItemRequest {
    ItemRequest {
        actor: actor(),
        sequence: 4,
        command,
    }
}
fn state(c: &Catalog) -> State {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(c, ContentId::new("theia.dev.training-stone").unwrap(), 13);
    State {
        actor: actor(),
        catalog_hash: sha256(DATA),
        bag: Bag::from_parts(
            c,
            3,
            slots,
            Stack::new(c, ContentId::new("theia.dev.training-sword").unwrap(), 1),
        )
        .unwrap(),
        reply: None,
    }
}
#[test]
fn version_two_requests_share_one_fixed_stream_without_reserved_field_aliases() {
    for command in [
        Command {
            expected_revision: 3,
            operation: Operation::Move { from: 0, to: 15 },
        }
        .into(),
        Command {
            expected_revision: 3,
            operation: Operation::Split {
                from: 1,
                to: 14,
                amount: 3,
            },
        }
        .into(),
        equip(Op::Equip { from: 0 }),
        equip(Op::Unequip { to: 15 }),
    ] {
        let value = request(command);
        let bytes = value.encode().unwrap();
        assert_eq!(bytes.len(), 43);
        assert!(bytes.starts_with(b"IR02"));
        assert_eq!(ItemRequest::decode(&bytes).unwrap(), value);
        assert!(wire::Request::decode(&bytes).is_err());
        for len in 0..bytes.len() {
            assert!(ItemRequest::decode(&bytes[..len]).is_err());
        }
        let mut tail = bytes.clone();
        tail.push(0);
        assert!(ItemRequest::decode(&tail).is_err());
    }
    for (op, reserved) in [(Op::Equip { from: 0 }, 38), (Op::Unequip { to: 15 }, 37)] {
        let good = request(equip(op)).encode().unwrap();
        for offset in [reserved, 39, 40, 41, 42] {
            let mut bad = good.clone();
            bad[offset] = 1;
            assert!(ItemRequest::decode(&bad).is_err());
        }
        let mut bad = good.clone();
        bad[36] = 4;
        assert!(ItemRequest::decode(&bad).is_err());
        for field in [4..12, 12..20, 20..28] {
            let mut bad = good.clone();
            bad[field].fill(0);
            assert!(ItemRequest::decode(&bad).is_err());
        }
    }
    for op in [Op::Equip { from: 16 }, Op::Unequip { to: 255 }] {
        assert!(request(equip(op)).encode().is_err());
    }
    let mut max = request(equip(Op::Equip { from: 0 }));
    max.command = EquipmentCommand {
        expected_revision: MAX_REVISION + 1,
        operation: Op::Equip { from: 0 },
    }
    .into();
    assert!(max.encode().is_err());
    let old = wire::Request {
        actor: actor(),
        sequence: 4,
        command: Command {
            expected_revision: 3,
            operation: Operation::Move { from: 0, to: 15 },
        },
    };
    let legacy = old.encode().unwrap();
    let newer = request(old.command.into()).encode().unwrap();
    assert_eq!(legacy[4..], newer[4..]);
    assert!(ItemRequest::decode(&legacy).is_err());
}
#[test]
fn equipment_states_roundtrip_all_reasons_and_conserve_the_seventeenth_slot() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let mut value = state(&catalog);
    let reasons = [
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
        Reason::EquipmentDisabled,
        Reason::NotEquippable,
        Reason::EmptyHand,
    ];
    let outcomes = [Outcome::Applied, Outcome::Replayed]
        .into_iter()
        .chain(reasons.into_iter().map(Outcome::Rejected));
    for outcome in outcomes {
        value.reply = Some(Reply {
            sequence: 4,
            outcome,
        });
        let bytes = value.encode(&catalog, sha256(DATA)).unwrap();
        assert_eq!(bytes.len(), 120);
        assert!(bytes.starts_with(b"IS02"));
        assert_eq!(
            State::decode(&bytes, &catalog, sha256(DATA)).unwrap(),
            value
        );
    }
    value.reply = None;
    let bytes = value.encode(&catalog, sha256(DATA)).unwrap();
    let decoded = State::decode(&bytes, &catalog, sha256(DATA)).unwrap();
    assert_eq!(
        decoded.bag.main_hand().unwrap().item().as_str(),
        "theia.dev.training-sword"
    );
    assert_eq!(decoded.bag.slots()[0].as_ref().unwrap().count(), 13);
}
#[test]
fn malformed_or_cross_profile_state_never_loses_or_fabricates_equipment() {
    let catalog = Catalog::from_json(DATA).unwrap();
    let value = state(&catalog);
    let bytes = value.encode(&catalog, sha256(DATA)).unwrap();
    for len in 0..bytes.len() {
        assert!(State::decode(&bytes[..len], &catalog, sha256(DATA)).is_err());
    }
    let mut tail = bytes.clone();
    tail.push(0);
    assert!(State::decode(&tail, &catalog, sha256(DATA)).is_err());
    for (offset, value) in [(117, 255), (118, 2), (119, 1), (68, 16)] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(State::decode(&bad, &catalog, sha256(DATA)).is_err());
    }
    let definitions: Vec<_> = catalog.definitions().collect();
    let stone = definitions
        .iter()
        .position(|d| d.id().as_str() == "theia.dev.training-stone")
        .unwrap() as u8;
    let mut ordinary = bytes.clone();
    ordinary[117] = stone;
    assert!(State::decode(&ordinary, &catalog, sha256(DATA)).is_err());
    assert!(State::decode(&bytes, &catalog, [1; 32]).is_err());
    let legacy = Catalog::from_json(include_bytes!(
        "../../../content/gameplay/development-items.json"
    ))
    .unwrap();
    assert!(State::decode(&bytes, &legacy, sha256(DATA)).is_err());
    assert!(value.encode(&legacy, sha256(DATA)).is_err());
    let mut old = State {
        actor: actor(),
        catalog_hash: [1; 32],
        bag: Bag::empty(&legacy),
        reply: Some(Reply {
            sequence: 1,
            outcome: Outcome::Rejected(Reason::EmptyHand),
        }),
    };
    assert!(old.encode(&legacy, [1; 32]).is_err());
    old.reply = None;
    let old_bytes = old.encode(&legacy, [1; 32]).unwrap();
    assert!(State::decode(&old_bytes, &catalog, [1; 32]).is_err());
    let mut bad = bytes;
    bad[52..60].fill(0);
    bad[60..68].copy_from_slice(&1_u64.to_le_bytes());
    bad[68] = 1;
    assert!(State::decode(&bad, &catalog, sha256(DATA)).is_err());
}
#[test]
fn full_largest_catalog_and_equipment_fit_one_bounded_frame() {
    let items:Vec<_>=(0..64).map(|i|serde_json::json!({"id":format!("theia.test.item-{i:02}"),"name":"Item","maximum_stack":1,"main_hand_appearance":"theia.test.sword"})).collect();
    let data =
        serde_json::to_vec(&serde_json::json!({"schema":2,"id":"theia.test.items","items":items}))
            .unwrap();
    let catalog = Catalog::from_json(&data).unwrap();
    let definitions: Vec<_> = catalog.definitions().collect();
    let slots = std::array::from_fn(|i| Stack::new(&catalog, definitions[i].id().clone(), 1));
    let hand = Stack::new(&catalog, definitions[63].id().clone(), 1);
    let value = State {
        actor: actor(),
        catalog_hash: sha256(&data),
        bag: Bag::from_parts(&catalog, MAX_REVISION, slots, hand).unwrap(),
        reply: Some(Reply {
            sequence: u64::MAX,
            outcome: Outcome::Replayed,
        }),
    };
    let bytes = value.encode(&catalog, sha256(&data)).unwrap();
    assert_eq!(bytes.len(), wire::ITEM_STATE_BYTES);
    assert_eq!(
        State::decode(&bytes, &catalog, sha256(&data)).unwrap(),
        value
    );
}
