//! Public cosmetic observations. Never includes backpack contents, private item
//! revisions, currency, stats, asset paths or client-authoritative equipment.
use crate::Result;
use std::collections::BTreeSet;
use theia_core::{
    content::ContentId,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::Catalog,
};
pub const STATE_TAG: &[u8; 4] = b"EA01";
pub const HEADER_BYTES: usize = 53;
pub const ROW_BYTES: usize = 9;
pub const MAX_ACTORS: usize = crate::wire::MAX_PLAYERS;
pub const MAX_BYTES: usize = HEADER_BYTES + MAX_ACTORS * ROW_BYTES;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub key: EntityKey,
    pub main_hand: Option<ContentId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub epoch: u64,
    pub tick: u64,
    pub catalog_hash: [u8; 32],
    pub actors: Vec<Actor>,
}
fn appearances(catalog: &Catalog) -> Vec<ContentId> {
    catalog
        .definitions()
        .filter_map(|d| d.main_hand_appearance().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
impl State {
    pub fn same_view(&self, other: &Self) -> bool {
        self.epoch == other.epoch
            && self.catalog_hash == other.catalog_hash
            && self.actors == other.actors
    }
    pub fn encode(&self, catalog: &Catalog, expected: [u8; 32]) -> Result<Vec<u8>> {
        if !catalog.equipment_enabled()
            || self.epoch == 0
            || self.tick == 0
            || expected == [0; 32]
            || self.catalog_hash != expected
            || self.actors.len() > MAX_ACTORS
        {
            return Err("Invalid appearance profile/header".into());
        }
        let definitions = appearances(catalog);
        let mut bytes = STATE_TAG.to_vec();
        bytes.extend(self.epoch.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.catalog_hash);
        bytes.push(self.actors.len() as u8);
        let mut previous = 0;
        for actor in &self.actors {
            if actor.key.epoch.value() != self.epoch || actor.key.entity.value() <= previous {
                return Err("Appearance actors must be ordered in the admitted epoch".into());
            }
            previous = actor.key.entity.value();
            bytes.extend(previous.to_le_bytes());
            bytes.push(match &actor.main_hand {
                None => 255,
                Some(id) => definitions
                    .iter()
                    .position(|d| d == id)
                    .ok_or("Unknown appearance ID")? as u8,
            });
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog, expected: [u8; 32]) -> Result<Self> {
        if bytes.len() < HEADER_BYTES
            || bytes.len() > MAX_BYTES
            || !bytes.starts_with(STATE_TAG)
            || !catalog.equipment_enabled()
            || expected == [0; 32]
        {
            return Err("Invalid appearance frame/profile".into());
        }
        let epoch = WorldEpoch::new(u64::from_le_bytes(bytes[4..12].try_into().unwrap()))
            .ok_or("Invalid appearance epoch")?;
        let tick = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
        let catalog_hash: [u8; 32] = bytes[20..52].try_into().unwrap();
        let count = usize::from(bytes[52]);
        if count > MAX_ACTORS
            || bytes.len() != HEADER_BYTES + count * ROW_BYTES
            || tick == 0
            || catalog_hash != expected
        {
            return Err("Invalid appearance byte identity/count/tick".into());
        }
        let definitions = appearances(catalog);
        let mut actors = Vec::with_capacity(count);
        let mut previous = 0;
        for row in bytes[HEADER_BYTES..].as_chunks::<ROW_BYTES>().0 {
            let entity = EntityId::new(u64::from_le_bytes(row[..8].try_into().unwrap()))
                .ok_or("Invalid appearance actor")?;
            if entity.value() <= previous {
                return Err("Duplicate/unordered appearance actor".into());
            }
            previous = entity.value();
            let main_hand = match row[8] {
                255 => None,
                ordinal => Some(
                    definitions
                        .get(usize::from(ordinal))
                        .ok_or("Invalid appearance ordinal")?
                        .clone(),
                ),
            };
            actors.push(Actor {
                key: EntityKey { epoch, entity },
                main_hand,
            });
        }
        Ok(Self {
            epoch: epoch.value(),
            tick,
            catalog_hash,
            actors,
        })
    }
}
