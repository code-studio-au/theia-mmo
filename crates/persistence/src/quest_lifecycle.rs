//! Explicit lifecycle binding is distinct from the old auto-start objective.
use crate::{ProgressionApplied, QuestCatalog, Result};
use ring::digest::{SHA256, digest};
use std::sync::Arc;
use theia_core::quests::{Profile, lifecycle, turn_in_xp};

#[derive(Clone, Debug)]
struct CompletionExperience {
    recipe: Arc<turn_in_xp::Recipe>,
    progression_hash: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct QuestLifecycleCatalog {
    objective: QuestCatalog,
    hash: [u8; 32],
    completion: Option<CompletionExperience>,
}
impl QuestLifecycleCatalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let objective = QuestCatalog::from_json(bytes)?;
        let mut bound = lifecycle::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        Ok(Self {
            objective,
            hash: digest(&SHA256, &bound).as_ref().try_into().unwrap(),
            completion: None,
        })
    }
    /// Distinct exact-byte mode. Existing journal-only rows/receipts are never
    /// silently adopted as rewarded completion state.
    pub fn from_json_with_turn_in_xp(
        quest: &[u8],
        recipe: &[u8],
        xp: &crate::ProgressionCatalog,
    ) -> Result<Self> {
        let mut catalog = Self::from_json(quest)?;
        let checked = turn_in_xp::Recipe::from_json(recipe, catalog.profile(), xp.profile())
            .map_err(crate::Error::Invalid)?;
        let mut bytes = turn_in_xp::PROFILE_DOMAIN.to_vec();
        bytes.extend(recipe);
        let recipe_hash = digest(&SHA256, &bytes);
        let mut bound = turn_in_xp::LIFECYCLE_PROFILE_DOMAIN.to_vec();
        bound.extend(catalog.hash);
        bound.extend(recipe_hash.as_ref());
        bound.extend(xp.hash());
        catalog.hash = digest(&SHA256, &bound).as_ref().try_into().unwrap();
        catalog.completion = Some(CompletionExperience {
            recipe: Arc::new(checked),
            progression_hash: xp.hash(),
        });
        Ok(catalog)
    }
    pub fn turn_in_xp(&self) -> Option<&turn_in_xp::Recipe> {
        self.completion.as_ref().map(|c| c.recipe.as_ref())
    }
    pub fn profile(&self) -> &Profile {
        self.objective.profile()
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    pub(crate) fn check_progression(&self, xp: &crate::ProgressionCatalog) -> Result<()> {
        self.objective.check_progression(xp)?;
        if self
            .completion
            .as_ref()
            .is_some_and(|c| c.progression_hash != xp.hash())
        {
            return Err(crate::Error::ProgressionCatalogMismatch);
        }
        Ok(())
    }
    pub(crate) fn command_hash(&self, command: lifecycle::Command) -> [u8; 32] {
        let mut bound = b"theia-quest-lifecycle-command-v1\0".to_vec();
        bound.extend(self.hash);
        bound.extend(command.canonical_bytes());
        digest(&SHA256, &bound).as_ref().try_into().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestLifecycleApplied {
    pub state: lifecycle::State,
    pub changed: bool,
    pub became_ready: bool,
    pub turned_in_now: bool,
    pub replayed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EarnedQuestLifecycle {
    pub experience: ProgressionApplied,
    pub quest: QuestLifecycleApplied,
}
