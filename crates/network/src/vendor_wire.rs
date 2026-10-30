//! Fixed barter intents share the backpack sequence and committed response.
//! Parsing never establishes live NPC entitlement or durable success.
use crate::Result;
use std::sync::Arc;
use theia_core::{
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::MAX_REVISION,
    vendors::{Catalog, Command, MAX_OFFERS, Purchase},
};
pub const REQUEST_TAG: &[u8; 4] = b"VP01";
pub const REQUEST_BYTES: usize = 61;
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
        let mut bound = theia_core::vendors::PROFILE_DOMAIN.to_vec();
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
    pub fn purchase(&self, command: Command) -> Result<Purchase> {
        let offer = self
            .catalog
            .offers()
            .nth(usize::from(command.offer))
            .ok_or("Unknown vendor offer")?;
        Ok(Purchase {
            expected_revision: command.expected_revision,
            offer: offer.id().clone(),
        })
    }
}
pub fn profile_identity(base: [u8; 32], profile: [u8; 32]) -> [u8; 32] {
    let mut bytes = b"theia-vendor-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile);
    crate::terrain::sha256(&bytes)
}
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn key(bytes: &[u8], at: usize) -> Result<EntityKey> {
    Ok(EntityKey {
        epoch: WorldEpoch::new(word(bytes, at)).ok_or("Invalid vendor epoch")?,
        entity: EntityId::new(word(bytes, at + 8)).ok_or("Invalid vendor entity")?,
    })
}
fn push_key(bytes: &mut Vec<u8>, key: EntityKey) {
    bytes.extend(key.epoch.value().to_le_bytes());
    bytes.extend(key.entity.value().to_le_bytes());
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub actor: EntityKey,
    pub sequence: u64,
    pub command: Command,
}
impl Request {
    pub fn encode(self) -> Result<Vec<u8>> {
        if self.sequence == 0
            || self.command.expected_revision > MAX_REVISION
            || self.command.dialogue_revision == 0
            || self.command.npc.epoch != self.actor.epoch
            || self.command.npc.entity == self.actor.entity
            || usize::from(self.command.offer) >= MAX_OFFERS
        {
            return Err("Invalid vendor request context".into());
        }
        let mut bytes = REQUEST_TAG.to_vec();
        push_key(&mut bytes, self.actor);
        bytes.extend(self.sequence.to_le_bytes());
        bytes.extend(self.command.expected_revision.to_le_bytes());
        bytes.extend(self.command.dialogue_revision.to_le_bytes());
        push_key(&mut bytes, self.command.npc);
        bytes.push(self.command.offer);
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REQUEST_BYTES || !bytes.starts_with(REQUEST_TAG) {
            return Err("Invalid vendor request frame".into());
        }
        let request = Self {
            actor: key(bytes, 4)?,
            sequence: word(bytes, 20),
            command: Command {
                expected_revision: word(bytes, 28),
                dialogue_revision: word(bytes, 36),
                npc: key(bytes, 44)?,
                offer: bytes[60],
            },
        };
        request.encode()?;
        Ok(request)
    }
}
