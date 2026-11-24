//! Bounded multi-contact authoring prerequisite. This catalogue grants no live
//! interaction permission; legacy single-contact runtime loaders reject schema 2.
use super::{Profile, define};
use crate::{
    content::ContentId,
    quests::catalog::{Catalog as Quests, EventKey},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// Initial Theia authoring budget, not a historical Classic limit.
pub const MAX_CONTACTS: usize = 16;
pub const MAX_CATALOG_BYTES: usize = MAX_CONTACTS * super::MAX_PROFILE_BYTES;
pub const CATALOG_DOMAIN: &[u8] = b"theia-dialogue-contacts-v2\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    revision: u32,
    contacts: Vec<super::Raw>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    revision: u32,
    contacts: BTreeMap<ContentId, Profile>,
    definitions: BTreeSet<ContentId>,
}

impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Dialogue contact catalogue exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid contact catalogue")?;
        if raw.schema != 2
            || raw.revision == 0
            || raw.contacts.is_empty()
            || raw.contacts.len() > MAX_CONTACTS
        {
            return Err("Invalid contact catalogue bounds");
        }
        let mut definitions = BTreeSet::new();
        let id = define(&raw.id, &mut definitions)?;
        let mut contacts = BTreeMap::new();
        for raw in raw.contacts {
            let bytes = serde_json::to_vec(&raw).map_err(|_| "Invalid contact profile")?;
            let profile = Profile::from_json(&bytes)?;
            // All definitions remain unambiguous across the complete collection,
            // including semantic choices used by quest observations.
            for id in std::iter::once(profile.id())
                .chain(std::iter::once(profile.npc()))
                .chain(profile.nodes().map(|node| node.id()))
                .chain(
                    profile
                        .nodes()
                        .flat_map(|node| node.choices().iter().map(|choice| choice.id())),
                )
            {
                define(id.as_str(), &mut definitions)?;
            }
            contacts.insert(profile.npc().clone(), profile);
        }
        Ok(Self {
            id,
            revision: raw.revision,
            contacts,
            definitions,
        })
    }

    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn contacts(&self) -> impl Iterator<Item = &Profile> {
        self.contacts.values()
    }
    pub fn contact(&self, npc: &ContentId) -> Option<&Profile> {
        self.contacts.get(npc)
    }
    pub fn defines(&self, id: &ContentId) -> bool {
        self.definitions.contains(id)
    }

    /// Resolve only dialogue references. Collision, installed actors, range,
    /// creature/item/area references and durable journal authority are separate.
    pub fn validate_quest_contacts(&self, quests: &Quests) -> Result<(), &'static str> {
        if self.defines(quests.id()) {
            return Err("Quest catalogue aliases dialogue content");
        }
        for quest in quests.quests() {
            if self.contact(quest.giver()).is_none() || self.contact(quest.turn_in()).is_none() {
                return Err("Quest giver or turn-in contact is absent");
            }
            if self.defines(quest.id()) {
                return Err("Quest definition aliases dialogue content");
            }
            for stage in quest.stages() {
                if self.defines(stage.id()) {
                    return Err("Quest stage aliases dialogue content");
                }
                for objective in stage.objectives() {
                    if self.defines(objective.id()) {
                        return Err("Quest objective aliases dialogue content");
                    }
                    if let EventKey::NpcInteraction { npc, interaction } = objective.event()
                        && !self.contact(npc).is_some_and(|profile| {
                            profile.nodes().any(|node| {
                                node.choices()
                                    .iter()
                                    .any(|choice| choice.id() == interaction)
                            })
                        })
                    {
                        return Err("Quest interaction does not belong to its contact");
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
