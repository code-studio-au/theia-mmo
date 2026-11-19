//! Offline cross-catalogue definition validation. This type deliberately never
//! exposes a Catalog: live journals still require a closed standalone catalogue.
use super::{Catalog, Definition, EventKey, MAX_CATALOG_BYTES, MAX_QUESTS};
use crate::content::ContentId;
use std::collections::{BTreeMap, BTreeSet};

mod compile;
pub use compile::{CompileError, CompiledCatalogue};
mod error;
pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

pub const MAX_CATALOGUES: usize = 32;
pub const MAX_REGISTRY_BYTES: usize = MAX_CATALOGUES * MAX_CATALOG_BYTES;
pub const MAX_REGISTRY_QUESTS: usize = MAX_CATALOGUES * MAX_QUESTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatalogueInfo<'a> {
    pub id: &'a ContentId,
    pub revision: u32,
    pub quests: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registry {
    catalogues: Vec<Catalog>,
    owners: BTreeMap<ContentId, usize>,
    definitions: usize,
}
impl Registry {
    /// All source byte/count budgets are checked before parsing any catalogue.
    /// External prerequisite references may resolve in another supplied pack.
    /// World targets, saved histories and loading/migration remain separate.
    pub fn from_catalogues(sources: &[&[u8]]) -> Result<Self> {
        if sources.is_empty() || sources.len() > MAX_CATALOGUES {
            return Err(Error::CatalogueCount);
        }
        let mut total = 0usize;
        for (index, source) in sources.iter().enumerate() {
            if source.len() > MAX_CATALOG_BYTES {
                return Err(Error::CatalogueBytes { index });
            }
            total = total
                .checked_add(source.len())
                .ok_or(Error::RegistryBytes)?;
            if total > MAX_REGISTRY_BYTES {
                return Err(Error::RegistryBytes);
            }
        }
        let catalogues = sources
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                Catalog::parse(bytes, false).map_err(|reason| Error::Catalogue { index, reason })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut ids = BTreeMap::new();
        let mut owners = BTreeMap::new();
        for (index, catalogue) in catalogues.iter().enumerate() {
            let mut insert = |id: &ContentId| -> Result<()> {
                if let Some(first) = ids.insert(id.clone(), index) {
                    return Err(Error::DuplicateDefinition {
                        id: id.clone(),
                        first,
                        second: index,
                    });
                }
                Ok(())
            };
            insert(catalogue.id())?;
            for quest in catalogue.quests() {
                insert(quest.id())?;
                owners.insert(quest.id().clone(), index);
                for stage in quest.stages() {
                    insert(stage.id())?;
                    for objective in stage.objectives() {
                        insert(objective.id())?;
                    }
                }
            }
        }
        for (catalogue_index, catalogue) in catalogues.iter().enumerate() {
            for quest in catalogue.quests() {
                let check = |reference: &ContentId| -> Result<()> {
                    if let Some(&definition_catalogue) = ids.get(reference) {
                        return Err(Error::TargetAlias {
                            reference: reference.clone(),
                            quest: quest.id().clone(),
                            catalogue: catalogue_index,
                            definition_catalogue,
                        });
                    }
                    Ok(())
                };
                check(quest.giver())?;
                check(quest.turn_in())?;
                for objective in quest.stages().flat_map(|stage| stage.objectives()) {
                    match objective.event() {
                        EventKey::Kill(creature) => check(creature)?,
                        EventKey::ReachArea(area) | EventKey::PossessItem(area) => check(area)?,
                        EventKey::NpcInteraction { npc, interaction } => {
                            check(npc)?;
                            check(interaction)?;
                        }
                    }
                }
            }
        }
        let result = Self {
            definitions: ids.len(),
            catalogues,
            owners,
        };
        result.validate_dependencies()?;
        Ok(result)
    }
    // Explicit iterative traversal keeps a valid full-registry chain off the
    // call stack. Visitation state is global; all edges are bounded by the
    // catalogue count/quest/prerequisite budgets already checked above.
    fn validate_dependencies(&self) -> Result<()> {
        for (quest_id, &catalogue) in &self.owners {
            let quest = self.quest(quest_id).expect("Registry owns its quest");
            for prerequisite in quest.prerequisites() {
                if !self.owners.contains_key(prerequisite) {
                    return Err(Error::UnknownPrerequisite {
                        quest: quest_id.clone(),
                        prerequisite: prerequisite.clone(),
                        catalogue,
                    });
                }
            }
        }
        let mut visiting = BTreeSet::new();
        let mut done = BTreeSet::new();
        for root in self.owners.keys() {
            let mut stack = vec![(root, false)];
            while let Some((id, leaving)) = stack.pop() {
                if leaving {
                    visiting.remove(id);
                    done.insert(id);
                    continue;
                }
                if done.contains(id) {
                    continue;
                }
                let quest = self.quest(id).expect("Validated registry prerequisite");
                if !visiting.insert(id) {
                    return Err(Error::Cycle {
                        quest: id.clone(),
                        catalogue: self.owners[id],
                    });
                }
                stack.push((id, true));
                for prerequisite in quest.prerequisites().iter().rev() {
                    stack.push((prerequisite, false));
                }
            }
        }
        Ok(())
    }
    pub fn quest(&self, id: &ContentId) -> Option<&Definition> {
        self.catalogues[*self.owners.get(id)?].quest(id)
    }
    pub fn catalogue_for(&self, quest: &ContentId) -> Option<&ContentId> {
        Some(self.catalogues[*self.owners.get(quest)?].id())
    }
    pub fn catalogues(&self) -> impl Iterator<Item = CatalogueInfo<'_>> {
        self.catalogues.iter().map(|c| CatalogueInfo {
            id: c.id(),
            revision: c.revision(),
            quests: c.quests().count(),
        })
    }
    pub fn quest_count(&self) -> usize {
        self.owners.len()
    }
    pub fn definition_count(&self) -> usize {
        self.definitions
    }
}
