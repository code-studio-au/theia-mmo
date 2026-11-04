//! Exact-byte multi-quest/XP policy binding. All inputs remain trusted domain
//! proposals until the authorized Store transaction and receipt commit.
use crate::{Error, ProgressionCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::{
    content::ContentId,
    quests::{
        catalog::{Catalog, EventKey},
        journal::Journal,
    },
};
#[derive(Clone, Debug)]
pub struct QuestJournalCatalog {
    catalog: Arc<Catalog>,
    hash: [u8; 32],
    progression: [u8; 32],
    areas: bool,
    inventory: Option<crate::InventoryCatalog>,
}
impl QuestJournalCatalog {
    pub fn from_json(bytes: &[u8], xp: &ProgressionCatalog) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes).map_err(Error::Invalid)?);
        let mut bound = theia_core::quests::journal::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        bound.extend(xp.hash());
        Ok(Self {
            catalog,
            hash: hash(&bound),
            progression: xp.hash(),
            areas: false,
            inventory: None,
        })
    }
    pub fn from_json_with_areas(
        bytes: &[u8],
        area_bytes: &[u8],
        xp: &ProgressionCatalog,
    ) -> Result<Self> {
        let mut catalog = Self::from_json(bytes, xp)?;
        let areas =
            theia_core::quests::areas::Catalog::from_json(area_bytes).map_err(Error::Invalid)?;
        areas
            .validate_quests(&catalog.catalog)
            .map_err(Error::Invalid)?;
        let mut bound = theia_core::quests::areas::PROFILE_DOMAIN.to_vec();
        bound.extend(catalog.hash);
        bound.extend(area_bytes);
        catalog.hash = hash(&bound);
        catalog.areas = true;
        Ok(catalog)
    }
    pub fn with_inventory(mut self, inventory: &crate::InventoryCatalog) -> Result<Self> {
        if self.inventory.is_some() || !self.catalog.quests().any(|q| q.has_possession_objectives())
        {
            return Err(Error::Invalid(
                "Inventory binding requires unbound possession objectives",
            ));
        }
        self.catalog
            .validate_possession_items(inventory.catalog())
            .map_err(Error::Invalid)?;
        let mut bytes = theia_core::quests::journal::ITEM_PROFILE_DOMAIN.to_vec();
        bytes.extend(self.hash);
        bytes.extend(inventory.hash());
        self.hash = hash(&bytes);
        self.inventory = Some(inventory.clone());
        Ok(self)
    }
    pub fn inventory(&self) -> Option<&crate::InventoryCatalog> {
        self.inventory.as_ref()
    }
    pub(crate) fn check_inventory_binding(&self) -> Result<()> {
        if self.inventory.is_none() && self.catalog.quests().any(|q| q.has_possession_objectives())
        {
            return Err(Error::Invalid(
                "Possession journal requires inventory binding",
            ));
        }
        Ok(())
    }
    pub(crate) fn check_reached_areas(
        &self,
        observation: &theia_core::quests::journal::ReachedAreas,
    ) -> Result<()> {
        if !self.areas
            || observation.is_empty()
            || observation.catalog() != self.catalog.id()
            || observation.catalog_revision() != self.catalog.revision()
        {
            return Err(Error::Invalid("Unbound or empty quest area observation"));
        }
        Ok(())
    }
    pub(crate) fn reached_areas_hash(
        &self,
        observation: &theia_core::quests::journal::ReachedAreas,
    ) -> [u8; 32] {
        let mut bytes = b"theia-quest-journal-area-observation-v1\0".to_vec();
        bytes.extend(self.hash);
        push(&mut bytes, observation.catalog());
        bytes.extend(observation.catalog_revision().to_le_bytes());
        bytes.extend((observation.len() as u16).to_le_bytes());
        for goal in observation.goals() {
            push(&mut bytes, goal.quest());
            bytes.extend(goal.definition_version().to_le_bytes());
            push(&mut bytes, goal.stage());
            push(&mut bytes, goal.area());
        }
        hash(&bytes)
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub fn check_progression(&self, xp: &ProgressionCatalog) -> Result<()> {
        if self.progression != xp.hash() {
            return Err(Error::ProgressionCatalogMismatch);
        }
        Ok(())
    }
    pub(crate) fn state_hash(&self, journal: &Journal) -> Result<[u8; 32]> {
        let mut bytes = b"theia-quest-journal-state-v1\0".to_vec();
        bytes.extend(self.hash);
        bytes.extend(
            journal
                .snapshot(&self.catalog)
                .map_err(|_| Error::QuestJournalData)?,
        );
        Ok(hash(&bytes))
    }
    /// Original victim-life identity, matching the explicit World kill adapter.
    pub fn kill_event_id(event: &theia_core::progression::KillEvent) -> std::num::NonZeroU128 {
        let mut bytes = theia_core::quests::journal::KILL_OCCURRENCE_DOMAIN.to_vec();
        bytes.extend(event.id.bytes());
        let value = hash(&bytes);
        std::num::NonZeroU128::new(u128::from_le_bytes(value[..16].try_into().unwrap()).max(1))
            .unwrap()
    }
    pub(crate) fn kill_hash(&self, event: &theia_core::progression::KillEvent) -> [u8; 32] {
        let mut bytes = b"theia-quest-journal-atomic-kill-v1\0".to_vec();
        bytes.extend(self.event_hash(&EventKey::Kill(event.creature.clone())));
        bytes.extend(event.id.bytes());
        hash(&bytes)
    }
    pub(crate) fn event_hash(&self, event: &EventKey) -> [u8; 32] {
        let mut bytes = b"theia-quest-journal-event-v1\0".to_vec();
        bytes.extend(self.hash);
        match event {
            EventKey::PossessItem(item) => {
                bytes.push(4);
                push(&mut bytes, item);
            }
            EventKey::ReachArea(id) => {
                bytes.push(3);
                push(&mut bytes, id);
            }
            EventKey::Kill(id) => {
                bytes.push(1);
                push(&mut bytes, id);
            }
            EventKey::NpcInteraction { npc, interaction } => {
                bytes.push(2);
                push(&mut bytes, npc);
                push(&mut bytes, interaction);
            }
        }
        hash(&bytes)
    }
    pub(crate) fn command_hash(&self, command: &QuestJournalCommand) -> [u8; 32] {
        let mut bytes = b"theia-quest-journal-action-v1\0".to_vec();
        bytes.extend(self.hash);
        bytes.push(command.action.code() as u8);
        push(&mut bytes, &command.quest);
        push(&mut bytes, &command.npc);
        bytes.extend(command.expected_revision.to_le_bytes());
        hash(&bytes)
    }
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
fn push(bytes: &mut Vec<u8>, id: &ContentId) {
    bytes.extend((id.as_str().len() as u16).to_le_bytes());
    bytes.extend(id.as_str().as_bytes());
}
pub use theia_core::quests::journal::{
    Action as QuestJournalAction, Command as QuestJournalCommand,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestJournalApplied {
    /// Current backpack from the same committed transaction, for item-bound profiles.
    pub inventory: Option<theia_core::inventory::Bag>,
    pub journal: Journal,
    pub revision: u64,
    pub changed: Vec<ContentId>,
    pub replayed: bool,
    pub experience: Option<theia_core::progression::State>,
    pub experience_granted: u32,
}
