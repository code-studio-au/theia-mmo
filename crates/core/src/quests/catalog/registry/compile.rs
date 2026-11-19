//! Deterministic reviewed subsets, not runtime pack loading or save migration.
use super::super::{
    Catalog, Completion, Definition, EventKey, MAX_CATALOG_BYTES, MAX_QUESTS, MAX_TOTAL_OBJECTIVES,
    RawCatalog, RawCompletion, RawObjective, RawQuest, RawStage,
};
use super::Registry;
use crate::content::ContentId;
use std::{collections::BTreeSet, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    Revision,
    SelectionCount,
    DuplicateSelection { quest: ContentId },
    UnknownSelection { quest: ContentId },
    QuestBudget,
    ObjectiveBudget,
    ByteBudget,
    Serialization,
    ClosedCatalogue { reason: &'static str },
}
impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Revision => write!(f, "Compiled catalogue requires a positive revision"),
            Self::SelectionCount => write!(f, "Select 1 to {MAX_QUESTS} distinct quest roots"),
            Self::DuplicateSelection { quest } => {
                write!(f, "Duplicate selected quest {}", quest.as_str())
            }
            Self::UnknownSelection { quest } => {
                write!(f, "Unknown selected quest {}", quest.as_str())
            }
            Self::QuestBudget => write!(
                f,
                "Selected quests and prerequisites exceed the {MAX_QUESTS}-quest runtime catalogue budget"
            ),
            Self::ObjectiveBudget => write!(
                f,
                "Selected quests exceed the {MAX_TOTAL_OBJECTIVES}-objective runtime catalogue budget"
            ),
            Self::ByteBudget => write!(
                f,
                "Compiled catalogue exceeds the {MAX_CATALOG_BYTES}-byte runtime budget"
            ),
            Self::Serialization => write!(f, "Compiled catalogue serialization failed"),
            Self::ClosedCatalogue { reason } => write!(
                f,
                "Compiled catalogue refused by the closed runtime validator: {reason}"
            ),
        }
    }
}
impl std::error::Error for CompileError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledCatalogue {
    bytes: Vec<u8>,
    catalogue: Catalog,
}
impl CompiledCatalogue {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn catalogue(&self) -> &Catalog {
        &self.catalogue
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
impl Registry {
    /// Include every transitive prerequisite. Source IDs, versions and behavior
    /// remain unchanged; metadata is explicit and byte compatibility is not a
    /// migration promise. The result passes the existing closed-catalogue API.
    pub fn compile_catalogue(
        &self,
        id: &ContentId,
        revision: u32,
        roots: &[ContentId],
    ) -> Result<CompiledCatalogue, CompileError> {
        if revision == 0 {
            return Err(CompileError::Revision);
        }
        if roots.is_empty() || roots.len() > MAX_QUESTS {
            return Err(CompileError::SelectionCount);
        }
        let mut unique = BTreeSet::new();
        for quest in roots {
            if !unique.insert(quest) {
                return Err(CompileError::DuplicateSelection {
                    quest: quest.clone(),
                });
            }
            if self.quest(quest).is_none() {
                return Err(CompileError::UnknownSelection {
                    quest: quest.clone(),
                });
            }
        }
        let mut closure = BTreeSet::new();
        let mut stack: Vec<_> = roots.iter().collect();
        let mut objectives = 0usize;
        while let Some(quest) = stack.pop() {
            if !closure.insert(quest) {
                continue;
            }
            if closure.len() > MAX_QUESTS {
                return Err(CompileError::QuestBudget);
            }
            let definition = self.quest(quest).expect("Validated registry selection");
            objectives += definition
                .stages()
                .map(|s| s.objectives().len())
                .sum::<usize>();
            if objectives > MAX_TOTAL_OBJECTIVES {
                return Err(CompileError::ObjectiveBudget);
            }
            stack.extend(definition.prerequisites());
        }
        let schema = if closure.iter().any(|q| {
            self.quest(q)
                .expect("Validated closure")
                .has_possession_objectives()
        }) {
            3
        } else if closure.iter().any(|q| {
            self.quest(q).expect("Validated closure").stages().any(|s| {
                s.objectives()
                    .iter()
                    .any(|o| matches!(o.event(), EventKey::ReachArea(_)))
            })
        }) {
            2
        } else {
            1
        };
        let raw = RawCatalog {
            schema,
            id: id.as_str().into(),
            revision,
            quests: closure
                .into_iter()
                .map(|q| definition(self.quest(q).expect("Validated closure")))
                .collect(),
        };
        let bytes = serde_json::to_vec(&raw).map_err(|_| CompileError::Serialization)?;
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err(CompileError::ByteBudget);
        }
        let catalogue = Catalog::from_json(&bytes)
            .map_err(|reason| CompileError::ClosedCatalogue { reason })?;
        Ok(CompiledCatalogue { bytes, catalogue })
    }
}
fn definition(q: &Definition) -> RawQuest {
    RawQuest {
        id: q.id().as_str().into(),
        version: q.version(),
        title: q.title().into(),
        description: q.description().into(),
        prerequisites: q
            .prerequisites()
            .iter()
            .map(|p| p.as_str().into())
            .collect(),
        giver: q.giver().as_str().into(),
        turn_in: q.turn_in().as_str().into(),
        repeat_policy: "once".into(),
        credit_scope: "character".into(),
        experience: q.experience(),
        start_stage: q.start_stage().as_str().into(),
        stages: q
            .stages()
            .map(|s| {
                let (kind, count) = match s.completion() {
                    Completion::All => ("all", None),
                    Completion::Any => ("any", None),
                    Completion::AtLeast { count } => ("at_least", Some(count)),
                };
                RawStage {
                    id: s.id().as_str().into(),
                    completion: RawCompletion {
                        kind: kind.into(),
                        count,
                    },
                    next: s.next().map(|id| id.as_str().into()),
                    objectives: s
                        .objectives()
                        .iter()
                        .map(|o| match o.event() {
                            EventKey::PossessItem(item) => RawObjective::PossessItem {
                                id: o.id().as_str().into(),
                                label: o.label().into(),
                                item: item.as_str().into(),
                                required: o.required(),
                                consume_on_turn_in: o.consume_on_turn_in(),
                            },
                            EventKey::ReachArea(area) => RawObjective::ReachArea {
                                id: o.id().as_str().into(),
                                label: o.label().into(),
                                area: area.as_str().into(),
                            },
                            EventKey::Kill(creature) => RawObjective::Kill {
                                id: o.id().as_str().into(),
                                label: o.label().into(),
                                creature: creature.as_str().into(),
                                required: o.required(),
                            },
                            EventKey::NpcInteraction { npc, interaction } => {
                                RawObjective::NpcInteraction {
                                    id: o.id().as_str().into(),
                                    label: o.label().into(),
                                    npc: npc.as_str().into(),
                                    interaction: interaction.as_str().into(),
                                }
                            }
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}
