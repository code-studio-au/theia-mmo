//! Bounded authoring documents and transactional commands, independent of Bevy.
mod duplication;
pub mod grading;
pub mod mailbox;
pub mod markers;
mod naming;
pub mod painting;
pub mod paths;
pub mod projection;
pub mod resampling;
pub mod terrain;
pub mod vegetation;

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Read, path::Path};
use theia_core::coordinates::{GlobalPosition, LocalFrame, LocalPosition};

pub const MAX_OBJECTS: usize = 1024;
pub const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;
pub const MAX_HISTORY: usize = 64;
pub const MAX_EDITS: usize = 128;
pub const ZONE_EXTENT: f64 = 512.;
pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Primitive {
    Box,
    Sphere,
    Cylinder,
}

/// Translation is global f64 metres; yaw is degrees around Y; size is metres.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub position: [f64; 3],
    pub yaw_degrees: f64,
    pub size: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub id: u64,
    pub name: String,
    pub primitive: Primitive,
    pub pose: Pose,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub schema: u32,
    pub name: String,
    pub origin: [f64; 3],
    /// Player feet, global metres. Placement clearance is checked at playtest time.
    pub spawn: [f64; 3],
    pub objects: Vec<Object>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<terrain::HeightPatch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<paths::GuidePath>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<markers::Marker>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plants: Vec<vegetation::Plant>,
}

fn require(condition: bool, message: &str) -> Result<()> {
    condition.then_some(()).ok_or_else(|| message.to_owned())
}
fn label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 80 && !value.chars().any(char::is_control)
}
impl Default for Zone {
    fn default() -> Self {
        Self {
            schema: 1,
            name: "Untitled zone".into(),
            origin: [0.; 3],
            spawn: [0., 0.1, 6.],
            objects: vec![],
            terrain: None,
            paths: vec![],
            markers: vec![],
            plants: vec![],
        }
    }
}
impl Zone {
    pub fn frame(&self) -> Result<LocalFrame> {
        GlobalPosition::new(self.origin)
            .map(LocalFrame::new)
            .ok_or("Invalid zone origin".into())
    }
    pub fn local(&self, position: [f64; 3]) -> Result<[f32; 3]> {
        let point = GlobalPosition::new(position).ok_or("Invalid global position")?;
        self.frame()?
            .to_local(point)
            .map(|v| v.metres())
            .ok_or("Outside local frame".into())
    }
    pub fn global(&self, local: [f32; 3]) -> Result<[f64; 3]> {
        let point = LocalPosition::new(local).ok_or("Invalid local position")?;
        self.frame()?
            .to_global(point)
            .map(|p| p.metres())
            .ok_or("Invalid global position".into())
    }
    /// Exact triangular terrain support, falling back to flat zone-origin Y.
    pub fn surface_position(&self, xz: [f64; 2]) -> Result<[f64; 3]> {
        let height = match &self.terrain {
            Some(p) => p.height(xz[0], xz[1])?.unwrap_or(0.),
            None => 0.,
        };
        let position = [xz[0], self.origin[1] + f64::from(height), xz[1]];
        self.check_position(position, 0.)?;
        Ok(position)
    }
    fn check_position(&self, position: [f64; 3], margin: f64) -> Result<()> {
        self.local(position)?;
        require(
            (0..3).all(|i| (position[i] - self.origin[i]).abs() + margin <= ZONE_EXTENT),
            "Outside bounded 1024 m authoring zone",
        )
    }
    pub fn validate(&self) -> Result<()> {
        require(matches!(self.schema, 1..=7), "Unsupported zone schema")?;
        require(
            self.schema != 1 || self.terrain.is_none(),
            "Terrain requires zone schema 2",
        )?;
        if let Some(terrain) = &self.terrain {
            terrain.validate(self)?;
        }
        require(
            self.schema >= 3 || self.paths.is_empty(),
            "Paths require zone schema 3",
        )?;
        require(self.paths.len() <= paths::MAX_PATHS, "Path budget exceeded")?;
        require(
            self.schema >= 4 || self.markers.is_empty(),
            "Markers require zone schema 4",
        )?;
        require(
            self.markers.len() <= markers::MAX_MARKERS,
            "Marker budget exceeded",
        )?;
        require(
            self.schema >= 5 || self.plants.is_empty(),
            "Vegetation requires zone schema 5",
        )?;
        require(
            self.plants.len() <= vegetation::MAX_PLANTS,
            "Vegetation budget exceeded",
        )?;
        require(label(&self.name), "Invalid zone name")?;
        self.frame()?;
        self.check_position(self.spawn, 2.)?;
        require(self.objects.len() <= MAX_OBJECTS, "Object budget exceeded")?;
        let mut ids = BTreeSet::new();
        for object in &self.objects {
            require(
                object.id > 0 && ids.insert(object.id),
                "Duplicate or zero object ID",
            )?;
            require(label(&object.name), "Invalid object name")?;
            let pose = &object.pose;
            require(
                pose.yaw_degrees.is_finite() && pose.yaw_degrees.abs() <= 360000.,
                "Invalid yaw",
            )?;
            require(
                pose.size
                    .iter()
                    .all(|v| v.is_finite() && (0.1..=128.).contains(v)),
                "Size must be 0.1..128 metres",
            )?;
            if object.primitive == Primitive::Sphere {
                require(
                    pose.size[0] == pose.size[1] && pose.size[1] == pose.size[2],
                    "Sphere needs uniform size",
                )?;
            }
            if object.primitive == Primitive::Cylinder {
                require(
                    pose.size[0] == pose.size[2],
                    "Cylinder needs equal X/Z diameter",
                )?;
            }
            // Circumscribed radius conservatively bounds every rotated primitive.
            let radius = pose.size.iter().map(|v| v * v).sum::<f64>().sqrt() / 2.;
            self.check_position(pose.position, radius)?;
        }
        for path in &self.paths {
            require(ids.insert(path.id), "Duplicate authoring ID")?;
            path.validate(self)?;
        }
        for marker in &self.markers {
            require(ids.insert(marker.id), "Duplicate authoring ID")?;
            marker.validate(self)?;
        }
        for plant in &self.plants {
            require(ids.insert(plant.id), "Duplicate authoring ID")?;
            plant.validate(self)?;
        }
        require(
            serde_json::to_vec_pretty(self)
                .map_err(|e| e.to_string())?
                .len() as u64
                <= MAX_DOCUMENT_BYTES,
            "Zone document byte budget exceeded",
        )?;
        Ok(())
    }
    pub fn next_id(&self) -> Option<u64> {
        self.objects
            .iter()
            .map(|o| o.id)
            .chain(self.paths.iter().map(|p| p.id))
            .chain(self.markers.iter().map(|m| m.id))
            .chain(self.plants.iter().map(|p| p.id))
            .max()
            .unwrap_or(0)
            .checked_add(1)
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        require(
            bytes.len() as u64 <= MAX_DOCUMENT_BYTES,
            "Zone file too large",
        )?;
        let zone: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        zone.validate()?;
        Ok(zone)
    }
    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(MAX_DOCUMENT_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        Self::from_json(&bytes)
    }
    /// Same-directory, synced temporary file then rename. A failed replacement
    /// leaves the previous file intact; Windows replacement needs native qualification.
    pub fn save(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        require(
            bytes.len() as u64 <= MAX_DOCUMENT_BYTES,
            "Zone file too large",
        )?;
        let name = path
            .file_name()
            .ok_or("Save path needs a filename")?
            .to_string_lossy();
        let temp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
        let mut file = fs::File::create_new(&temp).map_err(|e| e.to_string())?;
        let result = (|| {
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, path).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Place {
        object: Object,
    },
    Transform {
        id: u64,
        pose: Pose,
    },
    Delete {
        id: u64,
    },
    Duplicate {
        id: u64,
        new_id: u64,
        offset: [f64; 3],
    },
    Rename {
        id: u64,
        name: String,
    },
    RenameZone {
        name: String,
    },
    SetSpawn {
        position: [f64; 3],
    },
    SetTerrain {
        terrain: Option<terrain::HeightPatch>,
    },
    Brush {
        brush: terrain::Brush,
    },
    ProjectToSurface {
        projection: projection::Projection,
    },
    ResampleTerrain {
        resolution: terrain::Resolution,
    },
    PaintTerrain {
        paint: painting::Paint,
    },
    InsertPathPoint {
        id: u64,
        index: usize,
        position: [f64; 3],
    },
    RemovePathPoint {
        id: u64,
        index: usize,
    },
    MovePathPoint {
        id: u64,
        index: usize,
        position: [f64; 3],
    },
    GradePath {
        grade: grading::Grade,
    },
    PutPath {
        path: paths::GuidePath,
    },
    DeletePath {
        id: u64,
    },
    PutPlant {
        plant: vegetation::Plant,
    },
    DeletePlant {
        id: u64,
    },
    ScatterVegetation {
        scatter: vegetation::Scatter,
    },
    PutMarker {
        marker: markers::Marker,
    },
    DeleteMarker {
        id: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Apply {
        expected_revision: u64,
        edits: Vec<Edit>,
    },
    Undo {
        expected_revision: u64,
    },
    Redo {
        expected_revision: u64,
    },
}

/// One history entry per transaction. Revisions never rewind, including undo/load.
/// All refusals preserve document, history, redo branch and revision.
pub struct Editor {
    zone: Zone,
    revision: u64,
    undo: Vec<Zone>,
    redo: Vec<Zone>,
}
impl Editor {
    pub fn new(zone: Zone) -> Result<Self> {
        zone.validate()?;
        Ok(Self {
            zone,
            revision: 0,
            undo: vec![],
            redo: vec![],
        })
    }
    pub fn zone(&self) -> &Zone {
        &self.zone
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn history(&self) -> (usize, usize) {
        (self.undo.len(), self.redo.len())
    }
    fn push(stack: &mut Vec<Zone>, zone: Zone) {
        if stack.len() == MAX_HISTORY {
            stack.remove(0);
        }
        stack.push(zone);
    }
    fn advance(&self, expected: u64) -> Result<u64> {
        require(expected == self.revision, "Stale editor revision")?;
        self.revision
            .checked_add(1)
            .ok_or("Revision exhausted".into())
    }
    pub fn replace(&mut self, zone: Zone, expected: u64) -> Result<()> {
        let next = self.advance(expected)?;
        zone.validate()?;
        Self::push(&mut self.undo, std::mem::replace(&mut self.zone, zone));
        self.redo.clear();
        self.revision = next;
        Ok(())
    }
    pub fn execute(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Apply {
                expected_revision,
                edits,
            } => {
                self.advance(expected_revision)?;
                require(
                    !edits.is_empty() && edits.len() <= MAX_EDITS,
                    "Transaction needs 1..128 edits",
                )?;
                let mut candidate = self.zone.clone();
                for edit in edits {
                    match edit {
                        Edit::Place { object } => {
                            require(
                                candidate.objects.len() < MAX_OBJECTS,
                                "Object budget exceeded",
                            )?;
                            require(
                                !candidate.objects.iter().any(|o| o.id == object.id),
                                "Object ID exists",
                            )?;
                            candidate.objects.push(object);
                        }
                        Edit::Transform { id, pose } => {
                            candidate
                                .objects
                                .iter_mut()
                                .find(|o| o.id == id)
                                .ok_or("Unknown object ID")?
                                .pose = pose;
                        }
                        Edit::Duplicate { id, new_id, offset } => {
                            duplication::apply(&mut candidate, id, new_id, offset)?
                        }
                        Edit::Delete { id } => {
                            let index = candidate
                                .objects
                                .iter()
                                .position(|o| o.id == id)
                                .ok_or("Unknown object ID")?;
                            candidate.objects.remove(index);
                        }
                        Edit::Rename { id, name } => naming::apply(&mut candidate, id, name)?,
                        Edit::RenameZone { name } => candidate.name = name,
                        Edit::SetSpawn { position } => candidate.spawn = position,
                        Edit::SetTerrain { terrain } => {
                            candidate.schema = candidate.schema.max(2);
                            candidate.terrain = terrain;
                        }
                        Edit::ProjectToSurface { projection } => {
                            projection.apply(&mut candidate)?
                        }
                        Edit::ResampleTerrain { resolution } => {
                            candidate.schema = candidate.schema.max(7);
                            candidate.terrain = Some(resampling::resample(&candidate, resolution)?);
                        }
                        Edit::PaintTerrain { paint } => {
                            candidate.schema = candidate.schema.max(6);
                            paint.apply(
                                candidate
                                    .terrain
                                    .as_mut()
                                    .ok_or("Create terrain before painting")?,
                            )?;
                        }
                        Edit::InsertPathPoint {
                            id,
                            index,
                            position,
                        } => {
                            let path = candidate
                                .paths
                                .iter_mut()
                                .find(|p| p.id == id)
                                .ok_or("Unknown path ID")?;
                            require(
                                path.points.len() < paths::MAX_POINTS,
                                "Guide point budget exceeded",
                            )?;
                            require(index <= path.points.len(), "Unknown guide insertion index")?;
                            path.points.insert(index, position);
                        }
                        Edit::RemovePathPoint { id, index } => {
                            let path = candidate
                                .paths
                                .iter_mut()
                                .find(|p| p.id == id)
                                .ok_or("Unknown path ID")?;
                            require(path.points.len() > 2, "Guide needs at least two points")?;
                            require(index < path.points.len(), "Unknown guide point index")?;
                            path.points.remove(index);
                        }
                        Edit::MovePathPoint {
                            id,
                            index,
                            position,
                        } => {
                            let path = candidate
                                .paths
                                .iter_mut()
                                .find(|p| p.id == id)
                                .ok_or("Unknown path ID")?;
                            *path
                                .points
                                .get_mut(index)
                                .ok_or("Unknown guide point index")? = position;
                        }
                        Edit::GradePath { grade } => {
                            candidate.terrain = Some(grade.apply(&candidate)?);
                        }
                        Edit::PutPath { path } => {
                            require(
                                !candidate.objects.iter().any(|o| o.id == path.id),
                                "Authoring ID belongs to an object",
                            )?;
                            candidate.schema = candidate.schema.max(3);
                            if let Some(existing) =
                                candidate.paths.iter_mut().find(|p| p.id == path.id)
                            {
                                *existing = path;
                            } else {
                                require(
                                    candidate.paths.len() < paths::MAX_PATHS,
                                    "Path budget exceeded",
                                )?;
                                candidate.paths.push(path);
                            }
                        }
                        Edit::DeletePath { id } => {
                            let index = candidate
                                .paths
                                .iter()
                                .position(|p| p.id == id)
                                .ok_or("Unknown path ID")?;
                            candidate.paths.remove(index);
                        }
                        Edit::PutMarker { marker } => {
                            candidate.schema = candidate.schema.max(4);
                            if let Some(existing) =
                                candidate.markers.iter_mut().find(|m| m.id == marker.id)
                            {
                                *existing = marker;
                            } else {
                                require(
                                    candidate.markers.len() < markers::MAX_MARKERS,
                                    "Marker budget exceeded",
                                )?;
                                candidate.markers.push(marker);
                            }
                        }
                        Edit::DeleteMarker { id } => {
                            let index = candidate
                                .markers
                                .iter()
                                .position(|m| m.id == id)
                                .ok_or("Unknown marker ID")?;
                            candidate.markers.remove(index);
                        }
                        Edit::PutPlant { plant } => {
                            candidate.schema = candidate.schema.max(5);
                            if let Some(existing) =
                                candidate.plants.iter_mut().find(|p| p.id == plant.id)
                            {
                                *existing = plant;
                            } else {
                                require(
                                    candidate.plants.len() < vegetation::MAX_PLANTS,
                                    "Vegetation budget exceeded",
                                )?;
                                candidate.plants.push(plant);
                            }
                        }
                        Edit::DeletePlant { id } => {
                            let index = candidate
                                .plants
                                .iter()
                                .position(|p| p.id == id)
                                .ok_or("Unknown plant ID")?;
                            candidate.plants.remove(index);
                        }
                        Edit::ScatterVegetation { scatter } => {
                            let plants = scatter.generate(&candidate)?;
                            candidate.schema = candidate.schema.max(5);
                            candidate.plants.extend(plants);
                        }
                        Edit::Brush { brush } => candidate
                            .terrain
                            .as_mut()
                            .ok_or("Create terrain before brushing")?
                            .apply(&brush)?,
                    }
                }
                self.replace(candidate, expected_revision)
            }
            Command::Undo { expected_revision } => {
                let next = self.advance(expected_revision)?;
                let previous = self.undo.pop().ok_or("Nothing to undo")?;
                Self::push(&mut self.redo, std::mem::replace(&mut self.zone, previous));
                self.revision = next;
                Ok(())
            }
            Command::Redo { expected_revision } => {
                let next = self.advance(expected_revision)?;
                let following = self.redo.pop().ok_or("Nothing to redo")?;
                Self::push(&mut self.undo, std::mem::replace(&mut self.zone, following));
                self.revision = next;
                Ok(())
            }
        }
    }
}

/// Snap relative to the zone anchor in f64, including negative/far coordinates.
pub fn snap(value: f64, anchor: f64, step: f64) -> Result<f64> {
    require(
        value.is_finite()
            && anchor.is_finite()
            && step.is_finite()
            && (0.01..=128.).contains(&step),
        "Invalid snap input",
    )?;
    let snapped = anchor + ((value - anchor) / step).round() * step;
    require(snapped.is_finite(), "Snap overflow")?;
    Ok(snapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn object(id: u64) -> Object {
        Object {
            id,
            name: "Test block".into(),
            primitive: Primitive::Box,
            pose: Pose {
                position: [0., 0.5, 0.],
                yaw_degrees: 0.,
                size: [1.; 3],
            },
        }
    }
    fn apply(editor: &mut Editor, edits: Vec<Edit>) -> Result<()> {
        editor.execute(Command::Apply {
            expected_revision: editor.revision(),
            edits,
        })
    }
    #[test]
    fn refused_transaction_preserves_document_revision_and_redo() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(&mut editor, vec![Edit::Place { object: object(1) }]).unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let original = editor.zone().clone();
        let mut bad = object(2);
        bad.pose.size[0] = f64::NAN;
        assert!(
            apply(
                &mut editor,
                vec![
                    Edit::Place { object: object(1) },
                    Edit::Place { object: bad }
                ]
            )
            .is_err()
        );
        assert_eq!(editor.zone(), &original);
        assert_eq!(editor.revision(), 2);
        assert_eq!(editor.history(), (0, 1));
        editor
            .execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone().objects.len(), 1);
        assert!(
            editor
                .execute(Command::Undo {
                    expected_revision: 1
                })
                .is_err()
        );
        assert_eq!(editor.revision(), 3);
    }
    #[test]
    fn commands_round_trip_and_grouped_history_branches() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let command = Command::Apply {
            expected_revision: 0,
            edits: vec![
                Edit::Place { object: object(1) },
                Edit::Place { object: object(2) },
            ],
        };
        editor
            .execute(serde_json::from_slice(&serde_json::to_vec(&command).unwrap()).unwrap())
            .unwrap();
        let placed = editor.zone().clone();
        assert_eq!(editor.history(), (1, 0));
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        assert!(editor.zone().objects.is_empty());
        editor
            .execute(Command::Redo {
                expected_revision: 2,
            })
            .unwrap();
        assert_eq!(editor.zone(), &placed);
        editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        apply(
            &mut editor,
            vec![Edit::SetSpawn {
                position: [2., 0.1, 3.],
            }],
        )
        .unwrap();
        assert_eq!(editor.history(), (1, 0));
        assert!(
            editor
                .execute(Command::Redo {
                    expected_revision: 5
                })
                .is_err()
        );
        assert_eq!(editor.revision(), 5);
    }
    #[test]
    fn load_is_validated_and_undoable_without_rewinding_revision() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        let previous = editor.zone().clone();
        let mut zone = previous.clone();
        zone.name = "New zone".into();
        zone.objects.push(object(7));
        let decoded = Zone::from_json(&serde_json::to_vec(&zone).unwrap()).unwrap();
        editor.replace(decoded, 0).unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(editor.zone(), &previous);
        zone.schema = 99;
        assert!(editor.replace(zone, 2).is_err());
        assert_eq!(editor.history(), (0, 1));
        assert!(Zone::from_json(br#"{"schema":1,"unknown":true}"#).is_err());
        assert!(Zone::from_json(&vec![b' '; MAX_DOCUMENT_BYTES as usize + 1]).is_err());
    }
    #[test]
    fn history_and_transaction_budgets_are_bounded() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        for id in 1..=100 {
            apply(&mut editor, vec![Edit::Place { object: object(id) }]).unwrap();
        }
        assert_eq!(editor.history(), (MAX_HISTORY, 0));
        for _ in 0..MAX_HISTORY {
            editor
                .execute(Command::Undo {
                    expected_revision: editor.revision(),
                })
                .unwrap();
        }
        assert_eq!(editor.zone().objects.len(), 36);
        assert_eq!(editor.history(), (0, MAX_HISTORY));
        assert!(apply(&mut editor, vec![]).is_err());
        assert!(apply(&mut editor, vec![Edit::Delete { id: 1 }; MAX_EDITS + 1]).is_err());
        assert_eq!(editor.history(), (0, MAX_HISTORY));
    }
    #[test]
    fn far_signed_coordinates_and_snapping_keep_centimetres() {
        for sign in [-1., 1.] {
            let origin = [sign * 1e9, 0., -sign * 1e9];
            let mut zone = Zone {
                origin,
                spawn: [origin[0], 0.1, origin[2]],
                ..Zone::default()
            };
            let mut item = object(1);
            item.pose.position = [origin[0] + 0.01, 0.5, origin[2] - 0.02];
            zone.objects.push(item);
            zone.validate().unwrap();
            let local = zone.local(zone.objects[0].pose.position).unwrap();
            assert!((local[0] - 0.01).abs() < 0.000001);
            assert!((local[2] + 0.02).abs() < 0.000001);
            assert_eq!(
                snap(origin[0] - 1.6, origin[0], 1.).unwrap(),
                origin[0] - 2.
            );
        }
        assert!(snap(f64::NAN, 0., 1.).is_err());
        assert!(snap(1., 0., 0.).is_err());
    }
    #[test]
    fn invalid_shapes_ids_extents_and_missing_references_refuse() {
        let mut editor = Editor::new(Zone::default()).unwrap();
        apply(&mut editor, vec![Edit::Place { object: object(1) }]).unwrap();
        assert!(apply(&mut editor, vec![Edit::Place { object: object(1) }]).is_err());
        assert!(apply(&mut editor, vec![Edit::Delete { id: 7 }]).is_err());
        let mut bad = object(2);
        bad.pose.position[0] = 512.;
        assert!(apply(&mut editor, vec![Edit::Place { object: bad }]).is_err());
        let mut bad = object(2);
        bad.primitive = Primitive::Sphere;
        bad.pose.size[1] = 2.;
        assert!(apply(&mut editor, vec![Edit::Place { object: bad }]).is_err());
        let mut bad = object(2);
        bad.primitive = Primitive::Cylinder;
        bad.pose.size[2] = 2.;
        assert!(apply(&mut editor, vec![Edit::Place { object: bad }]).is_err());
        assert_eq!(editor.history(), (1, 0));
        assert_eq!(editor.revision(), 1);
    }
    #[test]
    fn save_load_replace_and_failed_save_preserve_existing_bytes() {
        let dir = std::env::temp_dir().join(format!("theia-zone-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("zone.json");
        let zone = Zone::default();
        zone.save(&path).unwrap();
        assert_eq!(Zone::load(&path).unwrap(), zone);
        let original = fs::read(&path).unwrap();
        let mut invalid = zone.clone();
        invalid.spawn = [f64::INFINITY; 3];
        assert!(invalid.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        let mut changed = zone;
        changed.name = "Replacement".into();
        changed.save(&path).unwrap();
        assert_eq!(Zone::load(&path).unwrap(), changed);
        fs::remove_dir_all(dir).unwrap();
    }
}
