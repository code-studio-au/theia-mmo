//! Exact-byte multi-quest/XP policy binding. All inputs remain trusted domain
//! proposals until the authorized Store transaction and receipt commit.
use crate::{Error, ProgressionCatalog, Result};
use ring::digest::{SHA256, digest};
use std::{num::NonZeroU128, sync::Arc};
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
}
impl QuestJournalCatalog {
    pub fn from_json(bytes: &[u8], xp: &ProgressionCatalog) -> Result<Self> {
        let catalog = Arc::new(Catalog::from_json(bytes).map_err(Error::Invalid)?);
        let mut bound = b"theia-quest-journal-catalog-v1\0".to_vec();
        bound.extend(bytes);
        bound.extend(xp.hash());
        Ok(Self {
            catalog,
            hash: hash(&bound),
            progression: xp.hash(),
        })
    }
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub(crate) fn check_progression(&self, xp: &ProgressionCatalog) -> Result<()> {
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
    pub(crate) fn event_hash(&self, event: &EventKey) -> [u8; 32] {
        let mut bytes = b"theia-quest-journal-event-v1\0".to_vec();
        bytes.extend(self.hash);
        match event {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestJournalAction {
    Accept,
    TurnIn,
}
impl QuestJournalAction {
    pub(crate) fn code(self) -> i16 {
        match self {
            Self::Accept => 1,
            Self::TurnIn => 2,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestJournalCommand {
    pub request_id: NonZeroU128,
    pub quest: ContentId,
    pub npc: ContentId,
    pub expected_revision: u64,
    pub action: QuestJournalAction,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestJournalApplied {
    pub journal: Journal,
    pub revision: u64,
    pub changed: Vec<ContentId>,
    pub replayed: bool,
    pub experience: Option<theia_core::progression::State>,
    pub experience_granted: u32,
}
