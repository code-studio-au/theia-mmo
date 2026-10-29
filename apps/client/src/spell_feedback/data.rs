//! Immutable bounded client presentation data; no gameplay effect values.
use serde::Deserialize;
use std::{collections::BTreeSet, io::Read, path::Path};
use theia_core::{abilities::Catalog, content::ContentId};
pub const MAX_BYTES: usize = 64 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    CastStart,
    Damage,
    Death,
    Respawn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    Cosmetic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sound {
    Cast,
    Impact,
}
impl Sound {
    pub fn seconds(self) -> f32 {
        match self {
            Self::Cast => 0.12,
            Self::Impact => 0.18,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u32,
    visual_limit: usize,
    reserved_visuals: usize,
    voice_limit: usize,
    reserved_voices: usize,
    max_distance_metres: f64,
    max_age_ms: u32,
    cues: Vec<RawCue>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCue {
    id: String,
    trigger: Trigger,
    ability: Option<String>,
    priority: Priority,
    label: String,
    colour: [f32; 4],
    radius_metres: f32,
    duration_ms: u32,
    sound: Sound,
    gain: f32,
    pitch: f32,
}
#[derive(Clone, Debug)]
pub struct Cue {
    pub id: ContentId,
    pub trigger: Trigger,
    pub ability: Option<ContentId>,
    pub priority: Priority,
    pub label: String,
    pub colour: [f32; 4],
    pub radius_metres: f32,
    pub duration_seconds: f32,
    pub sound: Sound,
    pub gain: f32,
    pub pitch: f32,
}
#[derive(Clone, Debug)]
pub struct Definition {
    pub visual_limit: usize,
    pub reserved_visuals: usize,
    pub voice_limit: usize,
    pub reserved_voices: usize,
    pub max_distance_metres: f64,
    pub max_age_seconds: f64,
    pub cues: Vec<Cue>,
}
impl Definition {
    pub fn load(path: &Path, catalog: &Catalog) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Ok(Self::from_json(&bytes, catalog)?)
    }
    pub fn from_json(bytes: &[u8], catalog: &Catalog) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("Spell feedback exceeds byte budget".into());
        }
        let raw: Raw =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid spell feedback: {e}"))?;
        if raw.schema != 1
            || !(1..=64).contains(&raw.visual_limit)
            || !(1..=32).contains(&raw.voice_limit)
            || raw.reserved_visuals == 0
            || raw.reserved_visuals > raw.visual_limit
            || raw.reserved_voices == 0
            || raw.reserved_voices > raw.voice_limit
            || !raw.max_distance_metres.is_finite()
            || !(1. ..=128.).contains(&raw.max_distance_metres)
            || !(100..=2000).contains(&raw.max_age_ms)
            || raw.cues.len() != catalog.abilities().len() * 2 + 2
        {
            return Err("Spell feedback schema/budget/coverage mismatch".into());
        }
        let mut ids = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut cues = Vec::new();
        for cue in raw.cues {
            let id = ContentId::new(&cue.id).ok_or("Invalid cue ID")?;
            let ability = cue
                .ability
                .as_deref()
                .map(|s| ContentId::new(s).ok_or("Invalid cue ability ID"))
                .transpose()?;
            if !ids.insert(id.clone()) || !keys.insert((cue.trigger, ability.clone())) {
                return Err("Duplicate spell feedback cue/mapping".into());
            }
            let has_ability = matches!(cue.trigger, Trigger::CastStart | Trigger::Damage);
            if has_ability != ability.is_some()
                || ability
                    .as_ref()
                    .is_some_and(|a| catalog.ability(a.as_str()).is_none())
                || cue.label.is_empty()
                || cue.label.len() > 64
                || cue.label.chars().any(char::is_control)
                || cue
                    .colour
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                || cue.colour[3] < 0.1
                || !cue.radius_metres.is_finite()
                || !(0.05..=4.).contains(&cue.radius_metres)
                || !(50..=2000).contains(&cue.duration_ms)
                || !cue.gain.is_finite()
                || !(0. ..=0.3).contains(&cue.gain)
                || !cue.pitch.is_finite()
                || !(0.5..=2.).contains(&cue.pitch)
            {
                return Err("Invalid spell feedback mapping/parameters".into());
            }
            cues.push(Cue {
                id,
                trigger: cue.trigger,
                ability,
                priority: cue.priority,
                label: cue.label,
                colour: cue.colour,
                radius_metres: cue.radius_metres,
                duration_seconds: cue.duration_ms as f32 / 1000.,
                sound: cue.sound,
                gain: cue.gain,
                pitch: cue.pitch,
            });
        }
        for ability in catalog.abilities() {
            for trigger in [Trigger::CastStart, Trigger::Damage] {
                if !keys.contains(&(trigger, Some(ability.id().clone()))) {
                    return Err("Missing ability feedback mapping".into());
                }
            }
        }
        for trigger in [Trigger::Death, Trigger::Respawn] {
            if !keys.contains(&(trigger, None)) {
                return Err("Missing lifecycle feedback mapping".into());
            }
        }
        Ok(Self {
            visual_limit: raw.visual_limit,
            reserved_visuals: raw.reserved_visuals,
            voice_limit: raw.voice_limit,
            reserved_voices: raw.reserved_voices,
            max_distance_metres: raw.max_distance_metres,
            max_age_seconds: f64::from(raw.max_age_ms) / 1000.,
            cues,
        })
    }
}
