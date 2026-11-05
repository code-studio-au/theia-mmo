//! Explicit reward messages. Structural validity never authorizes a recipient
//! or acknowledges durable items. Runtime admission binds the actor and profile.
use crate::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    inventory::MAX_REVISION,
    progression::KillId,
    rewards::{
        Catalog, ClaimCommand, ClaimId, CorpseSource, MAX_DEFINITIONS, MAX_VISIBLE_LIFE_CLAIMS,
        Scope,
    },
};
pub const REQUEST_TAG: &[u8; 4] = b"RC01";
pub const REQUEST_BYTES: usize = 52;
pub const PICKUP_TAG: &[u8; 4] = b"RC02";
pub const PICKUP_BYTES: usize = REQUEST_BYTES + 16;
pub const STATE_TAG: &[u8; 4] = b"RW01";
pub const LOOT_STATE_TAG: &[u8; 4] = b"RW02";
pub const HEADER_BYTES: usize = 61;
pub const ROW_BYTES: usize = 17;
pub const MAX_BYTES: usize = HEADER_BYTES + MAX_DEFINITIONS * ROW_BYTES;
pub const MAX_AVAILABLE_CLAIMS: usize = MAX_DEFINITIONS + MAX_VISIBLE_LIFE_CLAIMS;
pub const LOOT_ROW_BYTES: usize = ROW_BYTES + 1 + 16 + 24;
pub const MAX_LOOT_BYTES: usize =
    HEADER_BYTES + MAX_DEFINITIONS * (ROW_BYTES + 1) + MAX_VISIBLE_LIFE_CLAIMS * LOOT_ROW_BYTES;
fn has_life_rewards(catalog: &Catalog) -> bool {
    catalog
        .definitions()
        .any(|d| d.scope() == Scope::PerCreatureLife)
}
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
        Self::parse(bytes, items, false)
    }
    /// Explicit opt-in for headless per-life pickup. Default content admission
    /// remains unchanged until normal client interaction is qualified.
    pub fn from_json_with_creature_loot(
        bytes: &[u8],
        items: &crate::renet_runtime::inventory::Profile,
    ) -> Result<Self> {
        Self::parse(bytes, items, true)
    }
    fn parse(
        bytes: &[u8],
        items: &crate::renet_runtime::inventory::Profile,
        allow_life: bool,
    ) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes, items.catalog())?);
        if catalog.has_quest_required() {
            return Err("Quest-only loot runtime is not installed".into());
        }
        if has_life_rewards(&catalog) && !allow_life {
            return Err("Per-creature-life reward runtime is not installed".into());
        }
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
/// Explicit pickup of one private claim from an exact original creature life.
/// No client-supplied position or recipient. Structural validity grants nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickupRequest {
    pub request: Request,
    pub life: KillId,
}
impl PickupRequest {
    pub fn encode(self) -> Result<Vec<u8>> {
        let mut bytes = self.request.encode()?;
        bytes[..4].copy_from_slice(PICKUP_TAG);
        bytes.extend(self.life.bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != PICKUP_BYTES || !bytes.starts_with(PICKUP_TAG) {
            return Err("Invalid corpse pickup frame".into());
        }
        let mut request = bytes[..REQUEST_BYTES].to_vec();
        request[..4].copy_from_slice(REQUEST_TAG);
        Ok(Self {
            request: Request::decode(&request)?,
            life: KillId::from_bytes(bytes[REQUEST_BYTES..].try_into().unwrap())
                .ok_or("Invalid corpse pickup life")?,
        })
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Available {
    pub reward: ContentId,
    pub claim: ClaimId,
    pub corpse: Option<CorpseSource>,
}
/// Private owned availability, independent of the backpack revision and reply.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub actor: EntityKey,
    pub generation: u64,
    pub profile_hash: [u8; 32],
    pub available: Vec<Available>,
}
impl State {
    pub fn encode(&self, catalog: &Catalog, expected: [u8; 32]) -> Result<Vec<u8>> {
        let life = has_life_rewards(catalog);
        let limit = if life {
            MAX_AVAILABLE_CLAIMS
        } else {
            MAX_DEFINITIONS
        };
        if self.generation == 0
            || expected == [0; 32]
            || self.profile_hash != expected
            || self.available.len() > limit
        {
            return Err("Invalid reward observation header".into());
        }
        let definitions: Vec<_> = catalog.definitions().collect();
        let mut bytes = if life {
            LOOT_STATE_TAG.to_vec()
        } else {
            STATE_TAG.to_vec()
        };
        write_key(&mut bytes, self.actor);
        bytes.extend(self.generation.to_le_bytes());
        bytes.extend(self.profile_hash);
        bytes.push(self.available.len() as u8);
        let mut previous: Option<(usize, [u8; 16])> = None;
        let mut claims = BTreeSet::new();
        let mut once = BTreeSet::new();
        let mut occurrences = BTreeSet::new();
        let mut positions = BTreeMap::new();
        let mut life_count = 0;
        for row in &self.available {
            let ordinal = definitions
                .iter()
                .position(|d| d.id() == &row.reward)
                .ok_or("Unknown reward definition")?;
            let order = (ordinal, row.claim.bytes());
            if previous.is_some_and(|p| if life { order <= p } else { ordinal <= p.0 })
                || !claims.insert(row.claim)
            {
                return Err("Duplicate/unordered reward availability".into());
            }
            previous = Some(order);
            bytes.push(ordinal as u8);
            bytes.extend(row.claim.bytes());
            match (definitions[ordinal].scope(), row.corpse) {
                (Scope::OncePerCharacter, None) => {
                    if !once.insert(ordinal) {
                        return Err("Duplicate once-character availability".into());
                    }
                    if life {
                        bytes.push(0);
                    }
                }
                (Scope::PerCreatureLife, Some(source)) => {
                    life_count += 1;
                    if life_count > MAX_VISIBLE_LIFE_CLAIMS
                        || !occurrences.insert((ordinal, source.life.bytes()))
                    {
                        return Err("Duplicate or over-budget life availability".into());
                    }
                    if positions
                        .insert(source.life.bytes(), source.position)
                        .is_some_and(|old| old != source.position)
                    {
                        return Err("Conflicting corpse source positions".into());
                    }
                    bytes.push(1);
                    bytes.extend(source.life.bytes());
                    for value in source.position.metres() {
                        bytes.extend(value.to_le_bytes());
                    }
                }
                _ => return Err("Reward scope/source mismatch".into()),
            }
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], catalog: &Catalog, expected: [u8; 32]) -> Result<Self> {
        let life = has_life_rewards(catalog);
        let (tag, maximum, limit) = if life {
            (LOOT_STATE_TAG, MAX_LOOT_BYTES, MAX_AVAILABLE_CLAIMS)
        } else {
            (STATE_TAG, MAX_BYTES, MAX_DEFINITIONS)
        };
        if bytes.len() < HEADER_BYTES
            || bytes.len() > maximum
            || !bytes.starts_with(tag)
            || expected == [0; 32]
        {
            return Err("Invalid reward observation frame/profile".into());
        }
        let actor = key(bytes)?;
        let generation = word(bytes, 20);
        let profile_hash = bytes[28..60].try_into().unwrap();
        let count = usize::from(bytes[60]);
        if count > limit || generation == 0 || profile_hash != expected {
            return Err("Invalid reward observation identity/count/generation".into());
        }
        let definitions: Vec<_> = catalog.definitions().collect();
        let mut available = Vec::with_capacity(count);
        let mut at = HEADER_BYTES;
        for _ in 0..count {
            let row = take::<ROW_BYTES>(bytes, &mut at)?;
            let reward = definitions
                .get(usize::from(row[0]))
                .ok_or("Unknown reward ordinal")?
                .id()
                .clone();
            let claim =
                ClaimId::new(row[1..].try_into().unwrap()).ok_or("Invalid reward claim ID")?;
            let corpse = if life {
                match take::<1>(bytes, &mut at)?[0] {
                    0 => None,
                    1 => {
                        let life = KillId::from_bytes(take::<16>(bytes, &mut at)?)
                            .ok_or("Invalid corpse victim life")?;
                        let mut xyz = [0.; 3];
                        for value in &mut xyz {
                            *value = f64::from_le_bytes(take::<8>(bytes, &mut at)?);
                        }
                        Some(CorpseSource {
                            life,
                            position: GlobalPosition::new(xyz)
                                .ok_or("Invalid corpse global position")?,
                        })
                    }
                    _ => return Err("Unknown reward source kind".into()),
                }
            } else {
                None
            };
            available.push(Available {
                reward,
                claim,
                corpse,
            });
        }
        if at != bytes.len() {
            return Err("Trailing reward observation bytes".into());
        }
        let state = Self {
            actor,
            generation,
            profile_hash,
            available,
        };
        // Share scope, ordering, unique receipt/life and location validation.
        if state.encode(catalog, expected)?.as_slice() != bytes {
            return Err("Noncanonical reward observation".into());
        }
        Ok(state)
    }
}
fn take<const N: usize>(bytes: &[u8], at: &mut usize) -> Result<[u8; N]> {
    let end = at.checked_add(N).ok_or("Reward cursor overflow")?;
    let value = bytes.get(*at..end).ok_or("Truncated reward observation")?;
    *at = end;
    Ok(value.try_into().unwrap())
}
