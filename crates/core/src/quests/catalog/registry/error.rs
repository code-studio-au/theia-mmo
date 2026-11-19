use crate::content::ContentId;
use std::fmt;

/// Bounded identifiers and source indexes, without echoing arbitrary quest prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    CatalogueCount,
    CatalogueBytes {
        index: usize,
    },
    RegistryBytes,
    Catalogue {
        index: usize,
        reason: &'static str,
    },
    DuplicateDefinition {
        id: ContentId,
        first: usize,
        second: usize,
    },
    TargetAlias {
        reference: ContentId,
        quest: ContentId,
        catalogue: usize,
        definition_catalogue: usize,
    },
    UnknownPrerequisite {
        quest: ContentId,
        prerequisite: ContentId,
        catalogue: usize,
    },
    Cycle {
        quest: ContentId,
        catalogue: usize,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CatalogueCount => write!(f, "Quest registry catalogue count exceeds bounds"),
            Self::CatalogueBytes { index } => {
                write!(f, "Catalogue {} exceeds its byte budget", index + 1)
            }
            Self::RegistryBytes => write!(f, "Quest registry byte budget exceeded"),
            Self::Catalogue { index, reason } => write!(f, "Catalogue {}: {reason}", index + 1),
            Self::DuplicateDefinition { id, first, second } => write!(
                f,
                "Definition {} occurs in catalogues {} and {}",
                id.as_str(),
                first + 1,
                second + 1
            ),
            Self::TargetAlias {
                reference,
                quest,
                catalogue,
                definition_catalogue,
            } => write!(
                f,
                "Quest {} in catalogue {} uses definition {} from catalogue {} as a world target",
                quest.as_str(),
                catalogue + 1,
                reference.as_str(),
                definition_catalogue + 1
            ),
            Self::UnknownPrerequisite {
                quest,
                prerequisite,
                catalogue,
            } => write!(
                f,
                "Quest {} in catalogue {} has unknown prerequisite {}",
                quest.as_str(),
                catalogue + 1,
                prerequisite.as_str()
            ),
            Self::Cycle { quest, catalogue } => write!(
                f,
                "Quest prerequisite cycle reaches {} in catalogue {}",
                quest.as_str(),
                catalogue + 1
            ),
        }
    }
}
impl std::error::Error for Error {}
