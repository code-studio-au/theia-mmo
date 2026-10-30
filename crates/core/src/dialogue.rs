//! Bounded informational dialogue. Pure navigation establishes no authorized
//! interaction session, proximity, persistence or gameplay effects.
use crate::content::ContentId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_PROFILE_BYTES: usize = 16 * 1024;
pub const MAX_NODES: usize = 8;
pub const MAX_CHOICES: usize = 4;
pub const MAX_NAME_BYTES: usize = 64;
pub const MAX_TEXT_BYTES: usize = 512;
pub const MAX_CHOICE_BYTES: usize = 96;
pub const PROFILE_DOMAIN: &[u8] = b"theia-informational-dialogue-v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    id: String,
    npc: String,
    name: String,
    start: String,
    nodes: Vec<RawNode>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNode {
    id: String,
    text: String,
    choices: Vec<RawChoice>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawChoice {
    id: String,
    label: String,
    next: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    id: ContentId,
    npc: ContentId,
    name: String,
    start: ContentId,
    nodes: BTreeMap<ContentId, Node>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    id: ContentId,
    text: String,
    choices: Vec<Choice>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    id: ContentId,
    label: String,
    next: ContentId,
}

fn text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
fn define(value: &str, definitions: &mut BTreeSet<ContentId>) -> Result<ContentId, &'static str> {
    let id = ContentId::new(value).ok_or("Invalid dialogue definition ID")?;
    if !definitions.insert(id.clone()) {
        return Err("Dialogue definition IDs alias");
    }
    Ok(id)
}
fn visit(
    id: &ContentId,
    nodes: &BTreeMap<ContentId, Node>,
    visiting: &mut BTreeSet<ContentId>,
    visited: &mut BTreeSet<ContentId>,
) -> Result<(), &'static str> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id.clone()) {
        return Err("Initial dialogue must be acyclic");
    }
    let node = nodes.get(id).ok_or("Unknown dialogue node destination")?;
    for choice in &node.choices {
        visit(&choice.next, nodes, visiting, visited)?;
    }
    visiting.remove(id);
    visited.insert(id.clone());
    Ok(())
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("Dialogue profile exceeds byte budget");
        }
        let raw: Raw = serde_json::from_slice(bytes).map_err(|_| "Invalid dialogue profile")?;
        if raw.schema != 1
            || !text(&raw.name, MAX_NAME_BYTES)
            || raw.nodes.is_empty()
            || raw.nodes.len() > MAX_NODES
        {
            return Err("Invalid dialogue profile bounds");
        }
        let mut definitions = BTreeSet::new();
        let id = define(&raw.id, &mut definitions)?;
        let npc = define(&raw.npc, &mut definitions)?;
        let start = ContentId::new(&raw.start).ok_or("Invalid dialogue start ID")?;
        let mut nodes = BTreeMap::new();
        for raw in raw.nodes {
            if !text(&raw.text, MAX_TEXT_BYTES) || raw.choices.len() > MAX_CHOICES {
                return Err("Invalid dialogue node bounds");
            }
            let id = define(&raw.id, &mut definitions)?;
            let mut choices = Vec::new();
            for raw in raw.choices {
                if !text(&raw.label, MAX_CHOICE_BYTES) {
                    return Err("Invalid dialogue choice label");
                }
                choices.push(Choice {
                    id: define(&raw.id, &mut definitions)?,
                    label: raw.label,
                    next: ContentId::new(&raw.next).ok_or("Invalid dialogue destination ID")?,
                });
            }
            nodes.insert(
                id.clone(),
                Node {
                    id,
                    text: raw.text,
                    choices,
                },
            );
        }
        let mut visited = BTreeSet::new();
        visit(&start, &nodes, &mut BTreeSet::new(), &mut visited)?;
        if visited.len() != nodes.len() {
            return Err("Dialogue has unreachable nodes");
        }
        Ok(Self {
            id,
            npc,
            name: raw.name,
            start,
            nodes,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn npc(&self) -> &ContentId {
        &self.npc
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn start(&self) -> &Node {
        self.nodes.get(&self.start).unwrap()
    }
    pub fn node(&self, id: &ContentId) -> Option<&Node> {
        self.nodes.get(id)
    }
    /// A pure branch lookup, never permission to interact or advance a session.
    pub fn choose(&self, node: &ContentId, choice: &ContentId) -> Result<&Node, Error> {
        let node = self.nodes.get(node).ok_or(Error::UnknownNode)?;
        let choice = node
            .choices
            .iter()
            .find(|offered| &offered.id == choice)
            .ok_or(Error::ChoiceNotOffered)?;
        Ok(self.nodes.get(&choice.next).unwrap())
    }
}
impl Node {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn choices(&self) -> &[Choice] {
        &self.choices
    }
}
impl Choice {
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn next(&self) -> &ContentId {
        &self.next
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownNode,
    ChoiceNotOffered,
}
