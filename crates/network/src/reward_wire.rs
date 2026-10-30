//! Explicit reward messages. Structural validity never authorizes a recipient
//! or acknowledges durable items. Runtime admission binds the actor and profile.
use crate::Result;
use std::collections::BTreeSet;
use std::sync::Arc;
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::MAX_REVISION,
    rewards::{Catalog, ClaimCommand, ClaimId, MAX_DEFINITIONS},
};
pub const REQUEST_TAG: &[u8; 4] = b"RC01";
pub const REQUEST_BYTES: usize = 52;
pub const STATE_TAG: &[u8; 4] = b"RW01";
pub const HEADER_BYTES: usize = 61;
pub const ROW_BYTES: usize = 17;
pub const MAX_BYTES: usize = HEADER_BYTES + MAX_DEFINITIONS * ROW_BYTES;
/// The same byte-bound reward/item identity used by the durable store.
#[derive(Clone, Debug)]
pub struct Profile {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
    item_hash: [u8; 32],
}
impl Profile {
    pub fn from_json(
        bytes: &[u8],
        items: &crate::renet_runtime::inventory::Profile,
    ) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes, items.catalog())?);
        let mut bound = theia_core::rewards::PROFILE_DOMAIN.to_vec();
        bound.extend(items.hash());
        bound.extend(bytes);
        Ok(Self {
            catalog,
            hash: crate::terrain::sha256(&bound),
            item_hash: items.hash(),
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn item_hash(&self) -> [u8; 32] {
        self.item_hash
    }
}
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn key(bytes: &[u8]) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(word(bytes, 4)).ok_or("Invalid reward epoch")?,
        entity: EntityId::new(word(bytes, 12)).ok_or("Invalid reward actor")?,
    })
}
fn write_key(bytes: &mut Vec<u8>, actor: EntityKey) {
    bytes.extend(actor.epoch.value().to_le_bytes());
    bytes.extend(actor.entity.value().to_le_bytes());
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: ClaimCommand,
}
impl Request {
    pub fn encode(self) -> Result<Vec<u8>> {
        if self.sequence == 0 || self.command.expected_revision > MAX_REVISION {
            return Err("Invalid reward request sequence/revision".into());
        }
        let mut bytes = REQUEST_TAG.to_vec();
        write_key(&mut bytes, self.actor);
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.command.expected_revision.to_le_bytes());
        bytes.extend(self.command.claim.bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid reward request frame".into());
        }
        let request = Self {
            actor: key(bytes)?,
            sequence: word(bytes, 20),
            command: ClaimCommand {
                expected_revision: word(bytes, 28),
                claim: ClaimId::new(bytes[36..52].try_into().unwrap())
                    .ok_or("Invalid reward claim ID")?,
            },
        };
        if request.sequence == 0 || request.command.expected_revision > MAX_REVISION {
            return Err("Invalid reward request sequence/revision".into());
        }
        Ok(request)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Available {
    pub reward: ContentId,
    pub claim: ClaimId,
}
/// Private owned availability, independent of the backpack revision and reply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub actor: EntityKey,
    pub generation: u64,
    pub profile_hash: [u8; 32],
    pub available: Vec<Available>,
}
impl State {
    pub fn encode(&self, catalog: &Catalog, expected: [u8; 32]) -> Result<Vec<u8>> {
        if self.generation == 0
            || expected == [0; 32]
            || self.profile_hash != expected
            || self.available.len() > MAX_DEFINITIONS
        {
            return Err("Invalid reward observation header".into());
        }
        let definitions: Vec<_> = catalog.definitions().map(|d| d.id()).collect();
        let mut bytes = STATE_TAG.to_vec();
        write_key(&mut bytes, self.actor);
        bytes.extend(self.generation.to_le_bytes());
        bytes.extend(self.profile_hash);
        bytes.push(self.available.len() as u8);
        let mut previous = None;
        let mut claims = BTreeSet::new();
        for row in &self.available {
            let ordinal = definitions
                .iter()
                .position(|id| **id == row.reward)
                .ok_or("Unknown reward definition")?;
            if previous.is_some_and(|p| ordinal <= p) || !claims.insert(row.claim) {
                return Err("Duplicate/unordered reward availability".into());
            }
            previous = Some(ordinal);
            bytes.push(ordinal as u8);
            bytes.extend(row.claim.bytes());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog, expected: [u8; 32]) -> Result<Self> {
        if bytes.len() < HEADER_BYTES
            || bytes.len() > MAX_BYTES
            || !bytes.starts_with(STATE_TAG)
            || expected == [0; 32]
        {
            return Err("Invalid reward observation frame/profile".into());
        }
        let actor = key(bytes)?;
        let generation = word(bytes, 20);
        let profile_hash = bytes[28..60].try_into().unwrap();
        let count = usize::from(bytes[60]);
        if count > MAX_DEFINITIONS
            || bytes.len() != HEADER_BYTES + count * ROW_BYTES
            || generation == 0
            || profile_hash != expected
        {
            return Err("Invalid reward observation identity/count/generation".into());
        }
        let definitions: Vec<_> = catalog.definitions().map(|d| d.id()).collect();
        let mut available = Vec::with_capacity(count);
        let mut previous = None;
        let mut claims = BTreeSet::new();
        for row in bytes[HEADER_BYTES..].as_chunks::<ROW_BYTES>().0 {
            let ordinal = usize::from(row[0]);
            let reward = (*definitions.get(ordinal).ok_or("Unknown reward ordinal")?).clone();
            let claim =
                ClaimId::new(row[1..].try_into().unwrap()).ok_or("Invalid reward claim ID")?;
            if previous.is_some_and(|p| ordinal <= p) || !claims.insert(claim) {
                return Err("Duplicate/unordered reward availability".into());
            }
            previous = Some(ordinal);
            available.push(Available { reward, claim });
        }
        Ok(Self {
            actor,
            generation,
            profile_hash,
            available,
        })
    }
}
