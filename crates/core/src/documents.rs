//! Bounded authored item documents. Resolution against a checked bag is pure:
//! it grants no session authorization, durable read receipt or quest credit.
use crate::{content::ContentId, inventory};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const MAX_CATALOG_BYTES: usize = 32 * 1024;
pub const MAX_DOCUMENTS: usize = 16;
pub const MAX_PAGES: usize = 4;
pub const MAX_PAGE_BYTES: usize = 2048;
pub const MAX_TITLE_BYTES: usize = 96;
pub const PROFILE_DOMAIN: &[u8] = b"theia-item-documents-v1\0";
pub const JOURNAL_PROFILE_DOMAIN: &[u8] = b"theia-quest-journal-documents-v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    revision: u32,
    item_catalog: String,
    documents: Vec<RawDocument>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDocument {
    id: String,
    item: String,
    title: String,
    pages: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    id: ContentId,
    item: ContentId,
    title: String,
    pages: Vec<String>,
}
impl Document {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn item(&self) -> &ContentId {
        &self.item
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    /// Plain UTF-8 text, in authored order; no markup or external resources.
    pub fn pages(&self) -> &[String] {
        &self.pages
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    id: ContentId,
    revision: u32,
    items: inventory::Catalog,
    documents: BTreeMap<ContentId, Document>,
    by_item: BTreeMap<ContentId, ContentId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError {
    InvalidBag,
    StaleRevision,
    InvalidSlot,
    MissingItem,
    UnknownDocument,
    WrongItem,
}
impl Catalog {
    pub fn from_json(bytes: &[u8], items: &inventory::Catalog) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Document catalogue exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid document catalogue")?;
        if raw.schema != 1
            || raw.revision == 0
            || raw.documents.is_empty()
            || raw.documents.len() > MAX_DOCUMENTS
        {
            return Err("Unsupported document schema or bounds");
        }
        if ContentId::new(&raw.item_catalog).as_ref() != Some(items.id()) {
            return Err("Document item catalogue mismatch");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid document catalogue ID")?;
        let aliases_item = |id: &ContentId| id == items.id() || items.definition(id).is_some();
        if aliases_item(&id) {
            return Err("Document catalogue aliases item definitions");
        }
        let mut documents = BTreeMap::new();
        let mut by_item = BTreeMap::new();
        for entry in raw.documents {
            let document = ContentId::new(&entry.id).ok_or("Invalid document ID")?;
            let item = ContentId::new(&entry.item).ok_or("Invalid document item ID")?;
            if document == id || aliases_item(&document) || documents.contains_key(&document) {
                return Err("Document definition IDs alias");
            }
            if items.definition(&item).is_none() || by_item.contains_key(&item) {
                return Err("Unknown or ambiguous document item");
            }
            if entry.title.is_empty()
                || entry.title.len() > MAX_TITLE_BYTES
                || entry.title.trim() != entry.title
                || entry.title.chars().any(char::is_control)
            {
                return Err("Invalid document title");
            }
            if entry.pages.is_empty()
                || entry.pages.len() > MAX_PAGES
                || entry.pages.iter().any(|page| {
                    page.trim().is_empty()
                        || page.len() > MAX_PAGE_BYTES
                        || page
                            .chars()
                            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
                })
            {
                return Err("Invalid document pages");
            }
            by_item.insert(item.clone(), document.clone());
            documents.insert(
                document.clone(),
                Document {
                    id: document,
                    item,
                    title: entry.title,
                    pages: entry.pages,
                },
            );
        }
        Ok(Self {
            id,
            revision: raw.revision,
            items: items.clone(),
            documents,
            by_item,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn item_catalog(&self) -> &ContentId {
        self.items.id()
    }
    pub fn inventory(&self) -> &inventory::Catalog {
        &self.items
    }
    pub fn documents(&self) -> impl ExactSizeIterator<Item = &Document> {
        self.documents.values()
    }
    pub fn document(&self, id: &ContentId) -> Option<&Document> {
        self.documents.get(id)
    }
    pub fn for_item(&self, item: &ContentId) -> Option<&Document> {
        self.by_item.get(item).and_then(|id| self.document(id))
    }
    pub fn validate_quests(
        &self,
        quests: &crate::quests::catalog::Catalog,
    ) -> Result<(), &'static str> {
        use crate::quests::catalog::EventKey;
        let mut reserved = std::collections::BTreeSet::from([quests.id().clone()]);
        for quest in quests.quests() {
            reserved.extend([
                quest.id().clone(),
                quest.giver().clone(),
                quest.turn_in().clone(),
            ]);
            for stage in quest.stages() {
                reserved.insert(stage.id().clone());
                for objective in stage.objectives() {
                    reserved.insert(objective.id().clone());
                    match objective.event() {
                        EventKey::ReadDocument(document) => {
                            if self.document(document).is_none() {
                                return Err("Unknown quest document reference");
                            }
                        }
                        EventKey::Kill(id)
                        | EventKey::ReachArea(id)
                        | EventKey::PossessItem(id) => {
                            reserved.insert(id.clone());
                        }
                        EventKey::NpcInteraction { npc, interaction } => {
                            reserved.extend([npc.clone(), interaction.clone()]);
                        }
                    }
                }
            }
        }
        if reserved.contains(self.id()) || self.documents().any(|d| reserved.contains(d.id())) {
            return Err("Document definitions alias quest content");
        }
        quests.validate_possession_items(&self.items)
    }
    pub fn validate_areas(
        &self,
        areas: &crate::quests::areas::Catalog,
    ) -> Result<(), &'static str> {
        let ids: std::collections::BTreeSet<_> = std::iter::once(areas.id())
            .chain(areas.areas().map(|a| a.id()))
            .collect();
        if ids.contains(self.id()) || self.documents().any(|d| ids.contains(d.id())) {
            return Err("Document and area definition IDs alias");
        }
        Ok(())
    }
    /// Check the current stack and original selected bag revision. An authority
    /// must separately validate session/profile identity and lock the saved bag.
    pub fn resolve_owned(
        &self,
        bag: &inventory::Bag,
        expected_revision: u64,
        slot: u8,
        document: &ContentId,
    ) -> Result<&Document, OpenError> {
        bag.validate(&self.items)
            .map_err(|_| OpenError::InvalidBag)?;
        if bag.revision() != expected_revision {
            return Err(OpenError::StaleRevision);
        }
        let stack = bag
            .slots()
            .get(usize::from(slot))
            .ok_or(OpenError::InvalidSlot)?
            .as_ref()
            .ok_or(OpenError::MissingItem)?;
        let document = self.document(document).ok_or(OpenError::UnknownDocument)?;
        if document.item() != stack.item() {
            return Err(OpenError::WrongItem);
        }
        Ok(document)
    }
}

#[cfg(test)]
#[path = "documents/tests.rs"]
mod tests;
