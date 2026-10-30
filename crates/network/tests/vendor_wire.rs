//! Structure and profile binding, not live SQL or encrypted purchasing proof.
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::{Bag, ItemCommand},
    vendors::Command,
};
use theia_network::{
    inventory_wire,
    renet_runtime::inventory::Profile as Items,
    vendor_wire::{self, Profile, Request},
};
const ITEMS: &[u8] = include_bytes!("../../../content/gameplay/development-equipment-items.json");
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-vendor.json");
fn request() -> Request {
    let actor = EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(9).unwrap(),
    };
    Request {
        actor,
        sequence: 4,
        command: Command {
            expected_revision: 3,
            dialogue_revision: 2,
            npc: EntityKey {
                entity: EntityId::new(10).unwrap(),
                ..actor
            },
            offer: 0,
        },
    }
}
#[test]
fn vendor_intent_is_explicit_and_cannot_be_a_generic_item_grant() {
    let request = request();
    let bytes = request.encode().unwrap();
    assert_eq!(bytes.len(), vendor_wire::REQUEST_BYTES);
    assert_eq!(Request::decode(&bytes).unwrap(), request);
    assert!(inventory_wire::Request::decode(&bytes).is_err());
    assert!(inventory_wire::ItemRequest::decode(&bytes).is_err());
    assert!(
        inventory_wire::ItemRequest {
            actor: request.actor,
            sequence: 4,
            command: request.command.into()
        }
        .encode()
        .is_err()
    );
    let items = Items::from_json(ITEMS).unwrap();
    let intent: ItemCommand = request.command.into();
    assert_eq!(intent.expected_revision(), 3);
    assert_eq!(
        intent.propose(&Bag::empty(items.catalog()), items.catalog()),
        Err(theia_core::inventory::Error::VendorAuthorityRequired)
    );
}
#[test]
fn vendor_frames_reject_truncation_trailing_aliases_and_invalid_bounds() {
    let original = request().encode().unwrap();
    for length in 0..original.len() {
        assert!(Request::decode(&original[..length]).is_err());
    }
    let mut extended = original.clone();
    extended.push(0);
    assert!(Request::decode(&extended).is_err());
    for (at, value) in [(0, b'X'), (60, 8), (60, 255)] {
        let mut bytes = original.clone();
        bytes[at] = value;
        assert!(Request::decode(&bytes).is_err());
    }
    for at in [4, 12, 20, 36, 44, 52] {
        let mut bytes = original.clone();
        bytes[at..at + 8].fill(0);
        assert!(Request::decode(&bytes).is_err());
    }
    let mut bytes = original.clone();
    bytes[28..36].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(Request::decode(&bytes).is_err());
    let mut bytes = original.clone();
    bytes[52..60].copy_from_slice(&9_u64.to_le_bytes());
    assert!(Request::decode(&bytes).is_err());
    let mut bytes = original;
    bytes[44..52].copy_from_slice(&8_u64.to_le_bytes());
    assert!(Request::decode(&bytes).is_err());
}
#[test]
fn exact_item_and_vendor_bytes_bind_canonical_offer_indices() {
    let items = Items::from_json(ITEMS).unwrap();
    let profile = Profile::from_json(DATA, &items).unwrap();
    let purchase = profile.purchase(request().command).unwrap();
    assert_eq!(purchase.offer.as_str(), "theia.dev.training-vendor.stones");
    let mut unknown = request().command;
    unknown.offer = 1;
    assert!(profile.purchase(unknown).is_err());
    let mut padded = DATA.to_vec();
    padded.push(b' ');
    assert_ne!(
        profile.hash(),
        Profile::from_json(&padded, &items).unwrap().hash()
    );
    let mut padded_items = ITEMS.to_vec();
    padded_items.push(b' ');
    assert_ne!(
        profile.hash(),
        Profile::from_json(DATA, &Items::from_json(&padded_items).unwrap())
            .unwrap()
            .hash()
    );
    let adopted = items.clone().with_vendor(DATA).unwrap();
    assert_eq!(adopted.hash(), items.hash());
    assert_eq!(adopted.vendor().unwrap().hash(), profile.hash());
    assert_ne!(
        vendor_wire::profile_identity([1; 32], profile.hash()),
        [1; 32]
    );
}
