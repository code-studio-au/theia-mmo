//! Bounded original document-open intents. Decoding grants no read permission;
//! session entitlement and current item ownership belong to the authority.
use crate::{Result, documents::Profile};
use theia_core::{
    documents::OpenCommand,
    identity::{EntityId, EntityKey, WorldEpoch},
};
pub const REQUEST_TAG: &[u8; 4] = b"DO01";
pub const REQUEST_BYTES: usize = 86;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: OpenCommand,
}
/// Matching committed acknowledgement for a locally retained original intent.
/// Consumers display only content from this exact profile, never wire text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Approved {
    pub command: OpenCommand,
    pub profile_hash: [u8; 32],
}
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
impl Request {
    pub fn encode(self, profile: &Profile) -> Result<Vec<u8>> {
        if self.sequence == 0 {
            return Err("Invalid document sequence".into());
        }
        profile.resolve(self.command)?;
        let mut bytes = REQUEST_TAG.to_vec();
        bytes.extend(self.actor.epoch.value().to_le_bytes());
        bytes.extend(self.actor.entity.value().to_le_bytes());
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(profile.hash());
        bytes.extend(self.command.request_id.get().to_le_bytes());
        bytes.extend(self.command.expected_revision.to_le_bytes());
        bytes.extend([self.command.slot, self.command.document]);
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], profile: &Profile) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid document request frame".into());
        }
        if bytes[28..60] != profile.hash() {
            return Err("Document profile mismatch".into());
        }
        let value = Self {
            actor: EntityKey {
                epoch: WorldEpoch::new(word(bytes, 4)).ok_or("Invalid document epoch")?,
                entity: EntityId::new(word(bytes, 12)).ok_or("Invalid document actor")?,
            },
            sequence: word(bytes, 20),
            command: OpenCommand {
                request_id: std::num::NonZeroU128::new(u128::from_le_bytes(
                    bytes[60..76].try_into().unwrap(),
                ))
                .ok_or("Invalid original document request ID")?,
                expected_revision: word(bytes, 76),
                slot: bytes[84],
                document: bytes[85],
            },
        };
        if value.sequence == 0 {
            return Err("Invalid document sequence".into());
        }
        profile.resolve(value.command)?;
        Ok(value)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        let items = crate::renet_runtime::inventory::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-loot-items.json"
        ))
        .unwrap();
        Profile::from_json(br#"{"schema":1,"id":"notes","revision":1,"item_catalog":"theia.dev.equipment-items","documents":[{"id":"note","item":"theia.dev.training-stone","title":"Orders","pages":["Read these orders."]}]}"#, &items).unwrap()
    }
    fn request() -> Request {
        Request {
            actor: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(3).unwrap(),
            },
            sequence: 2,
            command: OpenCommand {
                request_id: std::num::NonZeroU128::new(91).unwrap(),
                expected_revision: 5,
                slot: 1,
                document: 0,
            },
        }
    }
    #[test]
    fn exact_profile_original_id_and_fields_round_trip_without_old_item_admission() {
        let profile = profile();
        let original = request();
        let bytes = original.encode(&profile).unwrap();
        assert_eq!(bytes.len(), REQUEST_BYTES);
        assert_eq!(Request::decode(&bytes, &profile).unwrap(), original);
        assert!(crate::inventory_wire::Request::decode(&bytes).is_err());
        assert!(crate::inventory_wire::ItemRequest::decode(&bytes).is_err());
        assert!(
            crate::inventory_wire::ItemRequest {
                actor: original.actor,
                sequence: original.sequence,
                command: original.command.into()
            }
            .encode()
            .is_err()
        );
        let mut foreign = bytes.clone();
        foreign[28] ^= 1;
        assert!(Request::decode(&foreign, &profile).is_err());
    }
    #[test]
    fn truncated_extended_foreign_zero_and_out_of_range_intents_refuse() {
        let profile = profile();
        let bytes = request().encode(&profile).unwrap();
        for length in 0..REQUEST_BYTES {
            assert!(Request::decode(&bytes[..length], &profile).is_err());
        }
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(Request::decode(&extended, &profile).is_err());
        for range in [4..12, 12..20, 20..28, 60..76] {
            let mut forged = bytes.clone();
            forged[range].fill(0);
            assert!(Request::decode(&forged, &profile).is_err());
        }
        for (at, value) in [(0, 0), (28, 0), (84, 16), (85, 1)] {
            let mut forged = bytes.clone();
            forged[at] = value;
            assert!(Request::decode(&forged, &profile).is_err());
        }
        let mut exhausted = request();
        exhausted.command.expected_revision = u64::MAX;
        assert!(exhausted.encode(&profile).is_err());
    }
    #[test]
    fn zero_revision_read_responses_require_the_explicit_exact_document_profile() {
        use crate::inventory_wire::{Outcome, Reply, State};
        let profile = profile();
        let items = profile.catalog().inventory();
        let state = State {
            actor: request().actor,
            catalog_hash: profile.inventory_hash(),
            bag: theia_core::inventory::Bag::empty(items),
            reply: Some(Reply {
                sequence: 1,
                outcome: Outcome::Applied,
            }),
        };
        assert!(state.encode(items, profile.inventory_hash()).is_err());
        let bytes = state.encode_with_documents(&profile).unwrap();
        assert_eq!(
            State::decode_with_documents(&bytes, &profile).unwrap(),
            state
        );
        assert!(State::decode(&bytes, items, profile.inventory_hash()).is_err());
        for length in 0..bytes.len() {
            assert!(State::decode_with_documents(&bytes[..length], &profile).is_err());
        }
        let mut foreign = bytes.clone();
        *foreign.last_mut().unwrap() ^= 1;
        assert!(State::decode_with_documents(&foreign, &profile).is_err());
        let mut extended = bytes.clone();
        extended.push(0);
        assert!(State::decode_with_documents(&extended, &profile).is_err());
        let mut legacy = bytes[..bytes.len() - 32].to_vec();
        legacy[..4].copy_from_slice(crate::inventory_wire::ITEM_STATE_TAG);
        assert!(State::decode(&legacy, items, profile.inventory_hash()).is_err());
        assert!(State::decode_with_documents(&legacy, &profile).is_err());
    }
    #[test]
    fn a_document_intent_cannot_mutate_a_bag_through_generic_proposals() {
        let profile = profile();
        let bag = theia_core::inventory::Bag::empty(profile.catalog().inventory());
        assert_eq!(
            theia_core::inventory::ItemCommand::from(request().command)
                .propose(&bag, profile.catalog().inventory()),
            Err(theia_core::inventory::Error::DocumentAuthorityRequired)
        );
    }
}
