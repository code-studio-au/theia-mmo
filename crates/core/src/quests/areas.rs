//! Bounded static quest volumes in checked global metres. Geometry alone grants
//! no quest credit, actor entitlement, World binding or client authority.
use crate::{content::ContentId, coordinates::GlobalPosition};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const PROFILE_DOMAIN: &[u8] = b"theia-quest-journal-areas-v1\0";
pub const MAX_CATALOG_BYTES: usize = 16 * 1024;
pub const MAX_AREAS: usize = 32;
pub const MIN_EXTENT_METRES: f64 = 1.;
pub const MAX_EXTENT_METRES: f64 = 1024.;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    revision: u32,
    areas: Vec<RawArea>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArea {
    id: String,
    name: String,
    min: [f64; 3],
    max: [f64; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    id: ContentId,
    name: String,
    min: GlobalPosition,
    max: GlobalPosition,
}
impl Area {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn min(&self) -> GlobalPosition {
        self.min
    }
    pub fn max(&self) -> GlobalPosition {
        self.max
    }
    /// Inclusive faces. This observes occupancy at an authoritative sample;
    /// it does not approximate a swept path or inspect rendered geometry.
    pub fn contains(&self, position: GlobalPosition) -> bool {
        let p = position.metres();
        let lo = self.min.metres();
        let hi = self.max.metres();
        (0..3).all(|axis| lo[axis] <= p[axis] && p[axis] <= hi[axis])
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    id: ContentId,
    revision: u32,
    areas: BTreeMap<ContentId, Area>,
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Quest area catalogue exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid quest area catalogue")?;
        if raw.schema != 1
            || raw.revision == 0
            || raw.areas.is_empty()
            || raw.areas.len() > MAX_AREAS
        {
            return Err("Unsupported quest area schema or bounds");
        }
        let id = ContentId::new(&raw.id).ok_or("Invalid quest area catalogue ID")?;
        let mut areas = BTreeMap::new();
        for entry in raw.areas {
            let area_id = ContentId::new(&entry.id).ok_or("Invalid quest area ID")?;
            if area_id == id || areas.contains_key(&area_id) {
                return Err("Quest area definition IDs alias");
            }
            if entry.name.is_empty()
                || entry.name.len() > 96
                || entry.name.trim() != entry.name
                || entry.name.chars().any(char::is_control)
            {
                return Err("Invalid quest area name");
            }
            let min = GlobalPosition::new(entry.min).ok_or("Invalid quest area lower position")?;
            let max = GlobalPosition::new(entry.max).ok_or("Invalid quest area upper position")?;
            if (0..3).any(|axis| {
                !(MIN_EXTENT_METRES..=MAX_EXTENT_METRES)
                    .contains(&(entry.max[axis] - entry.min[axis]))
            }) {
                return Err("Quest area extent exceeds development bounds");
            }
            areas.insert(
                area_id.clone(),
                Area {
                    id: area_id,
                    name: entry.name,
                    min,
                    max,
                },
            );
        }
        Ok(Self {
            id,
            revision: raw.revision,
            areas,
        })
    }
    /// Resolve selectors and keep area definitions distinct from quest/contact/
    /// combat definitions. This does not establish World bounds or authority.
    pub fn validate_quests(&self, quests: &super::catalog::Catalog) -> Result<(), &'static str> {
        let mut namespace = std::collections::BTreeSet::new();
        namespace.insert(quests.id().clone());
        let mut referenced = false;
        for q in quests.quests() {
            namespace.extend([q.id().clone(), q.giver().clone(), q.turn_in().clone()]);
            for stage in q.stages() {
                namespace.insert(stage.id().clone());
                for objective in stage.objectives() {
                    namespace.insert(objective.id().clone());
                    match objective.event() {
                        super::catalog::EventKey::ReachArea(id) => {
                            if self.area(id).is_none() {
                                return Err("Quest area is not installed");
                            }
                            referenced = true;
                        }
                        super::catalog::EventKey::Kill(id)
                        | super::catalog::EventKey::PossessItem(id) => {
                            namespace.insert(id.clone());
                        }
                        super::catalog::EventKey::NpcInteraction { npc, interaction } => {
                            namespace.extend([npc.clone(), interaction.clone()]);
                        }
                    }
                }
            }
        }
        if !referenced
            || namespace.contains(self.id())
            || self.areas().any(|a| namespace.contains(a.id()))
        {
            return Err("Unused or aliased quest area profile");
        }
        Ok(())
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn area(&self, id: &ContentId) -> Option<&Area> {
        self.areas.get(id)
    }
    pub fn areas(&self) -> impl Iterator<Item = &Area> {
        self.areas.values()
    }
    pub fn containing(&self, position: GlobalPosition) -> impl Iterator<Item = &Area> {
        self.areas
            .values()
            .filter(move |area| area.contains(position))
    }
}
