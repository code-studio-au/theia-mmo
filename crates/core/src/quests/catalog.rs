//! Versioned multi-quest definitions. Pure/offline only; not live journal state.
use crate::content::ContentId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_CATALOG_BYTES: usize = 65536;
pub const MAX_QUESTS: usize = 32;
pub const MAX_STAGES: usize = 8;
pub const MAX_OBJECTIVES: usize = 8;
pub const MAX_TOTAL_OBJECTIVES: usize = 512;
pub mod registry;
type Result<T> = std::result::Result<T, &'static str>;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    schema: u32,
    id: String,
    revision: u32,
    quests: Vec<RawQuest>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawQuest {
    id: String,
    version: u32,
    title: String,
    description: String,
    prerequisites: Vec<String>,
    giver: String,
    turn_in: String,
    repeat_policy: String,
    credit_scope: String,
    experience: u32,
    start_stage: String,
    stages: Vec<RawStage>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawStage {
    id: String,
    completion: RawCompletion,
    next: Option<String>,
    objectives: Vec<RawObjective>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawCompletion {
    kind: String,
    #[serde(
        default,
        deserialize_with = "present_count",
        skip_serializing_if = "Option::is_none"
    )]
    count: Option<u32>,
}
fn present_count<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<u32>, D::Error> {
    u32::deserialize(d).map(Some)
}
impl RawCompletion {
    fn checked(self) -> Result<Completion> {
        match (self.kind.as_str(), self.count) {
            ("all", None) => Ok(Completion::All),
            ("any", None) => Ok(Completion::Any),
            ("at_least", Some(count)) => Ok(Completion::AtLeast { count }),
            _ => Err("Unsupported stage completion rule"),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    All,
    Any,
    AtLeast { count: u32 },
}
impl Completion {
    pub fn satisfied(self, completed: usize, total: usize) -> bool {
        match self {
            Self::All => completed == total,
            Self::Any => completed > 0,
            Self::AtLeast { count } => completed >= count as usize,
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawObjective {
    Kill {
        id: String,
        label: String,
        creature: String,
        required: u32,
    },
    NpcInteraction {
        id: String,
        label: String,
        npc: String,
        interaction: String,
    },
    ReachArea {
        id: String,
        label: String,
        area: String,
    },
    PossessItem {
        id: String,
        label: String,
        item: String,
        required: u32,
        consume_on_turn_in: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventKey {
    Kill(ContentId),
    NpcInteraction {
        npc: ContentId,
        interaction: ContentId,
    },
    ReachArea(ContentId),
    PossessItem(ContentId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    id: ContentId,
    label: String,
    event: EventKey,
    required: u32,
    consume_on_turn_in: bool,
}
impl Objective {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn event(&self) -> &EventKey {
        &self.event
    }
    pub fn consume_on_turn_in(&self) -> bool {
        self.consume_on_turn_in
    }
    pub fn required(&self) -> u32 {
        self.required
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    id: ContentId,
    completion: Completion,
    next: Option<ContentId>,
    objectives: Vec<Objective>,
}
impl Stage {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn completion(&self) -> Completion {
        self.completion
    }
    pub fn next(&self) -> Option<&ContentId> {
        self.next.as_ref()
    }
    pub fn objectives(&self) -> &[Objective] {
        &self.objectives
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    id: ContentId,
    version: u32,
    title: String,
    description: String,
    prerequisites: Vec<ContentId>,
    giver: ContentId,
    turn_in: ContentId,
    experience: u32,
    start_stage: ContentId,
    stages: BTreeMap<ContentId, Stage>,
}
impl Definition {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn prerequisites(&self) -> &[ContentId] {
        &self.prerequisites
    }
    pub fn giver(&self) -> &ContentId {
        &self.giver
    }
    pub fn turn_in(&self) -> &ContentId {
        &self.turn_in
    }
    pub fn experience(&self) -> u32 {
        self.experience
    }
    pub fn start_stage(&self) -> &ContentId {
        &self.start_stage
    }
    pub fn stage(&self, id: &ContentId) -> Option<&Stage> {
        self.stages.get(id)
    }
    pub fn has_possession_objectives(&self) -> bool {
        self.stages().any(|s| {
            s.objectives()
                .iter()
                .any(|o| matches!(o.event(), EventKey::PossessItem(_)))
        })
    }
    pub fn possession_requirements(
        &self,
        items: &crate::inventory::Catalog,
    ) -> Result<Option<crate::inventory::requirements::Requirements>> {
        use crate::inventory::requirements::{Requirement, Requirements};
        let mut requirements = Vec::new();
        for objective in self.stages().flat_map(|s| s.objectives()) {
            if let EventKey::PossessItem(item) = objective.event() {
                requirements.push(
                    Requirement::new(
                        item.clone(),
                        objective.required(),
                        objective.consume_on_turn_in(),
                    )
                    .ok_or("Invalid possession quantity")?,
                );
            }
        }
        if requirements.is_empty() {
            return Ok(None);
        }
        Requirements::new(items, requirements)
            .map(Some)
            .map_err(|_| "Invalid possession item requirements")
    }
    pub fn stages(&self) -> impl Iterator<Item = &Stage> {
        self.stages.values()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectiveRef {
    pub quest: ContentId,
    pub stage: ContentId,
    pub objective: ContentId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    schema: u32,
    id: ContentId,
    revision: u32,
    quests: BTreeMap<ContentId, Definition>,
    index: BTreeMap<EventKey, Vec<ObjectiveRef>>,
}
fn id(raw: &str) -> Result<ContentId> {
    ContentId::new(raw).ok_or("Invalid quest content ID")
}
fn text(raw: &str, max: usize) -> bool {
    !raw.is_empty() && raw.len() <= max && raw.trim() == raw && !raw.chars().any(char::is_control)
}
fn unique(raw: &str, seen: &mut BTreeSet<ContentId>) -> Result<ContentId> {
    let id = id(raw)?;
    if !seen.insert(id.clone()) {
        return Err("Quest definition IDs must be unique");
    }
    Ok(id)
}
impl Catalog {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        Self::parse(bytes, true)
    }
    // Only the child registry can defer prerequisite linking. It never exposes
    // its internal Catalog values to a runtime journal or World.
    fn parse(bytes: &[u8], local_dependencies: bool) -> Result<Self> {
        if bytes.len() > MAX_CATALOG_BYTES {
            return Err("Quest catalogue exceeds byte budget");
        }
        let raw: RawCatalog =
            serde_json::from_slice(bytes).map_err(|_| "Invalid quest catalogue")?;
        if !matches!(raw.schema, 1..=3)
            || raw.revision == 0
            || raw.quests.is_empty()
            || raw.quests.len() > MAX_QUESTS
        {
            return Err("Unsupported quest catalogue schema or bounds");
        }
        let mut ids = BTreeSet::new();
        let catalog_id = unique(&raw.id, &mut ids)?;
        let mut quests = BTreeMap::new();
        let mut index: BTreeMap<EventKey, Vec<ObjectiveRef>> = BTreeMap::new();
        let mut total = 0;
        for quest in raw.quests {
            if quest.version == 0
                || !text(&quest.title, 128)
                || !text(&quest.description, 2048)
                || quest.repeat_policy != "once"
                || quest.credit_scope != "character"
                || quest.experience > crate::progression::MAX_GRANT_EXPERIENCE
                || quest.prerequisites.len() > MAX_QUESTS
                || quest.stages.is_empty()
                || quest.stages.len() > MAX_STAGES
            {
                return Err("Unsupported quest definition or bounds");
            }
            let quest_id = unique(&quest.id, &mut ids)?;
            let mut prerequisites = Vec::new();
            for prerequisite in quest.prerequisites {
                let prerequisite = id(&prerequisite)?;
                if prerequisites.contains(&prerequisite) {
                    return Err("Duplicate quest prerequisite");
                }
                prerequisites.push(prerequisite);
            }
            let mut stages = BTreeMap::new();
            for stage in quest.stages {
                let completion = stage.completion.checked()?;
                if stage.objectives.is_empty()
                    || stage.objectives.len() > MAX_OBJECTIVES
                    || matches!(completion,Completion::AtLeast {count} if count==0 || count as usize>stage.objectives.len())
                {
                    return Err("Impossible stage completion rule or bounds");
                }
                let stage_id = unique(&stage.id, &mut ids)?;
                let mut objectives = Vec::new();
                let mut events = BTreeSet::new();
                for objective in stage.objectives {
                    let (raw_id, label, event, required, consume_on_turn_in) = match objective {
                        RawObjective::Kill {
                            id: raw_id,
                            label,
                            creature,
                            required,
                        } => (
                            raw_id,
                            label,
                            EventKey::Kill(id(&creature)?),
                            required,
                            false,
                        ),
                        RawObjective::NpcInteraction {
                            id: raw_id,
                            label,
                            npc,
                            interaction,
                        } => (
                            raw_id,
                            label,
                            EventKey::NpcInteraction {
                                npc: id(&npc)?,
                                interaction: id(&interaction)?,
                            },
                            1,
                            false,
                        ),
                        RawObjective::ReachArea {
                            id: raw_id,
                            label,
                            area,
                        } => {
                            if raw.schema < 2 {
                                return Err(
                                    "Reach-area objectives require quest catalogue schema 2",
                                );
                            }
                            (raw_id, label, EventKey::ReachArea(id(&area)?), 1, false)
                        }
                        RawObjective::PossessItem {
                            id: raw_id,
                            label,
                            item,
                            required,
                            consume_on_turn_in,
                        } => {
                            if raw.schema != 3
                                || stage.next.is_some()
                                || completion != Completion::All
                            {
                                return Err(
                                    "Possession objectives require schema 3 and an all-required final stage",
                                );
                            }
                            (
                                raw_id,
                                label,
                                EventKey::PossessItem(id(&item)?),
                                required,
                                consume_on_turn_in,
                            )
                        }
                    };
                    if !text(&label, 128)
                        || !(1..=100).contains(&required)
                        || !events.insert(event.clone())
                    {
                        return Err("Invalid or duplicate stage objective");
                    }
                    let objective_id = unique(&raw_id, &mut ids)?;
                    total += 1;
                    if total > MAX_TOTAL_OBJECTIVES {
                        return Err("Quest objective budget exceeded");
                    }
                    index.entry(event.clone()).or_default().push(ObjectiveRef {
                        quest: quest_id.clone(),
                        stage: stage_id.clone(),
                        objective: objective_id.clone(),
                    });
                    objectives.push(Objective {
                        id: objective_id,
                        label,
                        event,
                        required,
                        consume_on_turn_in,
                    });
                }
                stages.insert(
                    stage_id.clone(),
                    Stage {
                        id: stage_id,
                        completion,
                        next: stage.next.as_deref().map(id).transpose()?,
                        objectives,
                    },
                );
            }
            let start_stage = id(&quest.start_stage)?;
            let mut visited = BTreeSet::new();
            let mut next = Some(start_stage.clone());
            while let Some(current) = next {
                if !visited.insert(current.clone()) {
                    return Err("Quest stage cycle");
                }
                next = stages
                    .get(&current)
                    .ok_or("Unknown quest stage reference")?
                    .next
                    .clone();
            }
            if visited.len() != stages.len() {
                return Err("Unreachable quest stage");
            }
            let definition = Definition {
                id: quest_id.clone(),
                version: quest.version,
                title: quest.title,
                description: quest.description,
                prerequisites,
                giver: id(&quest.giver)?,
                turn_in: id(&quest.turn_in)?,
                experience: quest.experience,
                start_stage,
                stages,
            };
            quests.insert(quest_id, definition);
        }
        if local_dependencies {
            for quest in quests.values() {
                let mut visiting = BTreeSet::new();
                let mut done = BTreeSet::new();
                dependencies(&quest.id, &quests, &mut visiting, &mut done)?;
            }
        }
        // External gameplay targets cannot masquerade as quest definition IDs.
        for quest in quests.values() {
            if ids.contains(quest.giver()) || ids.contains(quest.turn_in()) {
                return Err("Quest NPC aliases a definition ID");
            }
        }
        for event in index.keys() {
            let alias = match event {
                EventKey::Kill(creature) => ids.contains(creature),
                EventKey::ReachArea(area) | EventKey::PossessItem(area) => ids.contains(area),
                EventKey::NpcInteraction { npc, interaction } => {
                    ids.contains(npc) || ids.contains(interaction)
                }
            };
            if alias {
                return Err("Objective target aliases a definition ID");
            }
        }
        for objectives in index.values_mut() {
            objectives.sort();
        }
        Ok(Self {
            schema: raw.schema,
            id: catalog_id,
            revision: raw.revision,
            quests,
            index,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn validate_possession_items(&self, items: &crate::inventory::Catalog) -> Result<()> {
        let mut reserved = BTreeSet::from([self.id().clone()]);
        let mut references = BTreeSet::new();
        for quest in self.quests() {
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
                        EventKey::PossessItem(item) => {
                            references.insert(item.clone());
                        }
                        EventKey::Kill(id) | EventKey::ReachArea(id) => {
                            reserved.insert(id.clone());
                        }
                        EventKey::NpcInteraction { npc, interaction } => {
                            reserved.extend([npc.clone(), interaction.clone()]);
                        }
                    }
                }
            }
            quest.possession_requirements(items)?;
        }
        if references.iter().any(|id| reserved.contains(id)) {
            return Err("Possession item aliases another quest target or definition");
        }
        Ok(())
    }
    pub fn schema(&self) -> u32 {
        self.schema
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn quest(&self, id: &ContentId) -> Option<&Definition> {
        self.quests.get(id)
    }
    pub fn quests(&self) -> impl Iterator<Item = &Definition> {
        self.quests.values()
    }
    pub fn matching(&self, event: &EventKey) -> &[ObjectiveRef] {
        self.index.get(event).map_or(&[], Vec::as_slice)
    }
}
fn dependencies(
    current: &ContentId,
    quests: &BTreeMap<ContentId, Definition>,
    visiting: &mut BTreeSet<ContentId>,
    done: &mut BTreeSet<ContentId>,
) -> Result<()> {
    if done.contains(current) {
        return Ok(());
    }
    if !visiting.insert(current.clone()) {
        return Err("Quest prerequisite cycle");
    }
    for prerequisite in &quests
        .get(current)
        .ok_or("Unknown prerequisite quest")?
        .prerequisites
    {
        dependencies(prerequisite, quests, visiting, done)?;
    }
    visiting.remove(current);
    done.insert(current.clone());
    Ok(())
}
