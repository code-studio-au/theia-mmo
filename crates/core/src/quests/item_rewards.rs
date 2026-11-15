//! Fixed quest reward metadata only. Parsing grants no items or completion,
//! authorizes no character and does not admit a live reward profile.
use super::catalog::{Catalog as Quests, EventKey, MAX_QUESTS};
use crate::{
    content::ContentId,
    inventory,
    rewards::{Grant, MAX_GRANTS},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const PROFILE_DOMAIN: &[u8] = b"theia-quest-item-rewards-v1\0";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    revision: u32,
    quest_catalog: String,
    item_catalog: String,
    rewards: Vec<RawReward>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReward {
    quest: String,
    grants: Vec<RawGrant>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGrant {
    item: String,
    count: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    revision: u32,
    quests: Quests,
    items: inventory::Catalog,
    rewards: BTreeMap<ContentId, Vec<Grant>>,
}
impl Catalog {
    pub fn from_json(
        bytes: &[u8],
        quests: &Quests,
        items: &inventory::Catalog,
    ) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Quest item reward catalogue exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid quest item reward catalogue")?;
        if raw.schema != 1
            || raw.revision == 0
            || raw.rewards.is_empty()
            || raw.rewards.len() > MAX_QUESTS
        {
            return Err("Unsupported quest item reward schema or bounds");
        }
        if ContentId::new(&raw.quest_catalog).as_ref() != Some(quests.id())
            || ContentId::new(&raw.item_catalog).as_ref() != Some(items.id())
        {
            return Err("Quest item reward context mismatch");
        }
        quests.validate_possession_items(items)?;
        let mut reserved = BTreeSet::from([quests.id().clone(), items.id().clone()]);
        for q in quests.quests() {
            reserved.extend([q.id().clone(), q.giver().clone(), q.turn_in().clone()]);
            for s in q.stages() {
                reserved.insert(s.id().clone());
                for o in s.objectives() {
                    reserved.insert(o.id().clone());
                    match o.event() {
                        EventKey::PossessItem(_) => {}
                        EventKey::FollowNpc(id)
                        | EventKey::Kill(id)
                        | EventKey::ReachArea(id)
                        | EventKey::ReadDocument(id) => {
                            reserved.insert(id.clone());
                        }
                        EventKey::NpcInteraction { npc, interaction } => {
                            reserved.extend([npc.clone(), interaction.clone()]);
                        }
                    }
                }
            }
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid quest item reward catalogue ID")?;
        if reserved.contains(&id) || items.definition(&id).is_some() {
            return Err("Quest item reward catalogue aliases existing content");
        }
        let mut rewards = BTreeMap::new();
        for reward in raw.rewards {
            let quest = ContentId::new(&reward.quest)
                .filter(|q| quests.quest(q).is_some())
                .ok_or("Unknown item reward quest")?;
            if reward.grants.is_empty()
                || reward.grants.len() > MAX_GRANTS
                || rewards.contains_key(&quest)
            {
                return Err("Duplicate quest reward or invalid grant count");
            }
            let mut seen = BTreeSet::new();
            let mut grants = Vec::with_capacity(reward.grants.len());
            let mut empty_bag_slots = 0u32;
            for raw in reward.grants {
                let item = ContentId::new(&raw.item).ok_or("Invalid quest reward item")?;
                if reserved.contains(&item) || !seen.insert(item.clone()) {
                    return Err("Quest reward item aliases content or duplicates a grant");
                }
                let grant = Grant::checked(items, item.clone(), raw.count)
                    .ok_or("Invalid quest item reward grant")?;
                empty_bag_slots += grant.count().div_ceil(
                    items
                        .definition(&item)
                        .ok_or("Unknown quest reward item")?
                        .maximum_stack(),
                );
                if empty_bag_slots > inventory::SLOT_COUNT as u32 {
                    return Err("Quest reward bundle cannot fit an empty backpack");
                }
                grants.push(grant);
            }
            rewards.insert(quest, grants);
        }
        Ok(Self {
            id,
            revision: raw.revision,
            quests: quests.clone(),
            items: items.clone(),
            rewards,
        })
    }
    /// Check additional installed metadata; this still establishes no runtime admission.
    pub fn validate_context(
        &self,
        areas: Option<&super::areas::Catalog>,
        documents: Option<&crate::documents::Catalog>,
    ) -> Result<(), &'static str> {
        let mut reserved = BTreeSet::new();
        if let Some(areas) = areas {
            areas.validate_quests(&self.quests)?;
            reserved.insert(areas.id());
            reserved.extend(areas.areas().map(|a| a.id()));
        }
        if let Some(documents) = documents {
            if documents.inventory() != &self.items {
                return Err("Quest reward document inventory mismatch");
            }
            documents.validate_quests(&self.quests)?;
            if let Some(areas) = areas {
                documents.validate_areas(areas)?;
            }
            reserved.insert(documents.id());
            reserved.extend(documents.documents().map(|d| d.id()));
        }
        if reserved.contains(&self.id)
            || self
                .rewards
                .values()
                .flatten()
                .any(|g| reserved.contains(g.item()))
        {
            return Err("Quest item rewards alias area or document content");
        }
        Ok(())
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn quest_catalog(&self) -> &Quests {
        &self.quests
    }
    pub fn item_catalog(&self) -> &inventory::Catalog {
        &self.items
    }
    pub fn rewards(&self) -> impl ExactSizeIterator<Item = (&ContentId, &[Grant])> {
        self.rewards.iter().map(|(q, g)| (q, g.as_slice()))
    }
    pub fn grants(&self, quest: &ContentId) -> Option<&[Grant]> {
        self.rewards.get(quest).map(Vec::as_slice)
    }
}
#[cfg(test)]
mod tests;

mod proposal;
pub use proposal::{TurnIn, TurnInError};
