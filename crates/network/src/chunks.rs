//! Bounded residency, normal opt-in startup and movement coverage. Package
//! loading stays outside the simulation step; the default tile is monolithic.
use crate::{
    Result,
    content::{MANIFEST_FILE, Manifest, read_bounded},
    physics::{PlacedCollider, StaticScene, SweepHit},
    terrain::Terrain,
};
use avian3d::prelude::SimpleCollider;
use bevy::prelude::Vec3;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use theia_core::coordinates::{ChunkId, GlobalPosition, INITIAL_CHUNK_METRES, LocalFrame};
const MAX_INDEX_ENTRIES: usize = 64;
// Measured finite town demand reaches all 26 packages with recovery and actors.
// Individual sessions retain their configured smaller budgets.
pub const MAX_RESIDENT: usize = 32;
const MAX_COLLIDERS: usize = 8192;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: [i64; 2],
    package: [u8; 32],
    #[serde(default)]
    bounds: Option<[[f64; 2]; 2]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    chunk_metres: u32,
    query_origin: [f64; 3],
    chunks: Vec<Entry>,
}
#[derive(Clone)]
pub struct Index {
    directory: PathBuf,
    frame: LocalFrame,
    entries: BTreeMap<ChunkId, [u8; 32]>,
    bounds: BTreeMap<ChunkId, [[f64; 2]; 2]>,
    contributors: BTreeMap<ChunkId, Vec<ChunkId>>,
}
impl Index {
    pub fn load(directory: &Path) -> Result<Self> {
        let raw: Document = serde_json::from_slice(&read_bounded(
            &directory.join("world.chunks.json"),
            32 * 1024,
        )?)?;
        if !matches!(raw.schema, 1 | 2)
            || raw.chunk_metres != INITIAL_CHUNK_METRES
            || raw.chunks.is_empty()
            || raw.chunks.len() > MAX_INDEX_ENTRIES
        {
            return Err("Unsupported or out-of-budget chunk index".into());
        }
        let frame = LocalFrame::new(
            GlobalPosition::new(raw.query_origin).ok_or("Invalid chunk query origin")?,
        );
        let mut entries = BTreeMap::new();
        let mut bounds = BTreeMap::new();
        for entry in raw.chunks {
            let id = ChunkId {
                x: entry.id[0],
                z: entry.id[1],
            };
            let origin = id
                .origin(INITIAL_CHUNK_METRES)
                .ok_or("Invalid global chunk corner")?;
            let offset = frame
                .to_local(origin)
                .ok_or("Chunk outside bounded query frame")?
                .metres();
            // Existing fixture geometry is bounded to +/-1023m. Leave room for
            // that payload; this is a small residency proof, not large streaming.
            if offset.iter().any(|v| v.abs() > 3000.) || entries.insert(id, entry.package).is_some()
            {
                return Err("Duplicate/out-of-frame chunk".into());
            }
            match (raw.schema, entry.bounds) {
                (1, None) => {}
                (2, Some(mut envelope)) => {
                    if envelope
                        .iter()
                        .flatten()
                        .any(|v| !v.is_finite() || v.abs() > 1023.)
                        || (0..2).any(|axis| envelope[0][axis] > envelope[1][axis])
                    {
                        return Err("Invalid chunk contributor bounds".into());
                    }
                    // Canonicalize negative zero before identity hashing.
                    for v in envelope.iter_mut().flatten() {
                        if *v == 0. {
                            *v = 0.;
                        }
                    }
                    bounds.insert(id, envelope);
                }
                _ => return Err("Chunk schema disagrees with contributor bounds".into()),
            }
        }
        let mut contributors = BTreeMap::new();
        for &cell in entries.keys() {
            let mut required = BTreeSet::from([cell]);
            for (&owner, envelope) in &bounds {
                // Subtract checked integer owners before converting to f64;
                // global coordinates never pass through f32 here.
                let relative = [owner.x - cell.x, owner.z - cell.z]
                    .map(|v| v as f64 * f64::from(INITIAL_CHUNK_METRES));
                if (0..2).all(|axis| {
                    let lo = relative[axis] + envelope[0][axis];
                    let hi = relative[axis] + envelope[1][axis];
                    if lo == hi {
                        lo >= 0. && lo <= f64::from(INITIAL_CHUNK_METRES)
                    } else {
                        lo < f64::from(INITIAL_CHUNK_METRES) && hi > 0.
                    }
                }) {
                    required.insert(owner);
                }
            }
            if required.len() > MAX_RESIDENT {
                return Err("Cell contributors exceed residency budget".into());
            }
            contributors.insert(cell, required.into_iter().collect());
        }
        Ok(Self {
            directory: directory.to_owned(),
            frame,
            entries,
            bounds,
            contributors,
        })
    }
    /// Canonical dataset identity; the query frame is an implementation choice.
    pub fn identity(&self) -> [u8; 32] {
        let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
        hash.update(if self.bounds.is_empty() {
            b"theia-chunk-index-v1\0"
        } else {
            b"theia-chunk-index-v2\0"
        });
        hash.update(&INITIAL_CHUNK_METRES.to_le_bytes());
        hash.update(&(self.entries.len() as u16).to_le_bytes());
        for (id, package) in &self.entries {
            hash.update(&id.x.to_le_bytes());
            hash.update(&id.z.to_le_bytes());
            hash.update(package);
            if let Some(bounds) = self.bounds.get(id) {
                for v in bounds.iter().flatten() {
                    hash.update(&v.to_le_bytes());
                }
            }
        }
        hash.finish().as_ref().try_into().unwrap()
    }
    pub fn contains(&self, id: ChunkId) -> bool {
        self.entries.contains_key(&id)
    }
    pub fn ids(&self) -> impl Iterator<Item = ChunkId> + '_ {
        self.entries.keys().copied()
    }
    /// Packages needed for complete geometry in this indexed movement cell.
    pub fn contributors(&self, id: ChunkId) -> Option<&[ChunkId]> {
        self.contributors.get(&id).map(Vec::as_slice)
    }
    pub fn bounds(&self, id: ChunkId) -> Option<[[f64; 2]; 2]> {
        self.bounds.get(&id).copied()
    }
    pub fn coverage(&self, ids: impl IntoIterator<Item = ChunkId>) -> Result<Coverage> {
        let mut coverage = Coverage {
            frame: self.frame,
            ids: [None; MAX_RESIDENT],
            corners: [[0.; 2]; MAX_RESIDENT],
            len: 0,
        };
        let mut loaded = BTreeSet::new();
        for id in ids {
            if loaded.len() == MAX_RESIDENT || !self.contains(id) || !loaded.insert(id) {
                return Err("Invalid or over-budget movement coverage".into());
            }
        }
        for &id in &loaded {
            if !self.contributors[&id]
                .iter()
                .all(|needed| loaded.contains(needed))
            {
                continue;
            }
            let corner = self
                .frame
                .to_local(
                    id.origin(INITIAL_CHUNK_METRES)
                        .ok_or("Invalid chunk corner")?,
                )
                .ok_or("Unbounded coverage")?
                .metres();
            coverage.ids[coverage.len] = Some(id);
            coverage.corners[coverage.len] = [corner[0], corner[2]];
            coverage.len += 1;
        }
        Ok(coverage)
    }
    pub fn owner(&self, position: GlobalPosition) -> Option<ChunkId> {
        let id = position.chunk(INITIAL_CHUNK_METRES)?;
        self.entries.contains_key(&id).then_some(id)
    }
    pub fn directory(&self, id: ChunkId) -> PathBuf {
        self.directory.join(format!("chunk_{}_{}", id.x, id.z))
    }
    pub fn frame(&self) -> LocalFrame {
        self.frame
    }
}
/// Bounded immutable footprint coverage and infinite-height outer edges. Local
/// movement never shortcuts a missing corner or disconnected chunk gap.
#[derive(Clone, Copy, Debug)]
pub struct Coverage {
    frame: LocalFrame,
    ids: [Option<ChunkId>; MAX_RESIDENT],
    corners: [[f32; 2]; MAX_RESIDENT],
    len: usize,
}
impl crate::kinematic::Boundary for Coverage {
    fn sweep(&self, feet: Vec3, delta: Vec3) -> Option<SweepHit> {
        Coverage::sweep(self, feet, delta)
    }
}
impl Coverage {
    /// The procedural recipe guarantees one complete terrain surface per cell.
    /// Imported datasets still use Index::coverage and its contributor checks.
    pub(crate) fn procedural_cells(
        frame: LocalFrame,
        ids: impl IntoIterator<Item = ChunkId>,
    ) -> Result<Self> {
        let mut coverage = Self {
            frame,
            ids: [None; MAX_RESIDENT],
            corners: [[0.; 2]; MAX_RESIDENT],
            len: 0,
        };
        let mut seen = BTreeSet::new();
        for id in ids {
            if coverage.len == MAX_RESIDENT || !crate::procedural::contains(id) || !seen.insert(id)
            {
                return Err("Invalid procedural coverage cells".into());
            }
            let p = frame
                .to_local(
                    id.origin(INITIAL_CHUNK_METRES)
                        .ok_or("Invalid cell corner")?,
                )
                .ok_or("Cell outside local query frame")?
                .metres();
            coverage.ids[coverage.len] = Some(id);
            coverage.corners[coverage.len] = [p[0], p[2]];
            coverage.len += 1;
        }
        Ok(coverage)
    }
    pub fn contains(&self, feet: GlobalPosition) -> bool {
        let [x, y, z] = feet.metres();
        let radius = f64::from(crate::kinematic::FOOTPRINT_RADIUS);
        for dx in [-radius, 0., radius] {
            for dz in [-radius, 0., radius] {
                let Some(id) = GlobalPosition::new([x + dx, y, z + dz])
                    .and_then(|p| p.chunk(INITIAL_CHUNK_METRES))
                else {
                    return false;
                };
                if !self.ids[..self.len].contains(&Some(id)) {
                    return false;
                }
            }
        }
        true
    }
    pub fn sweep(&self, feet: Vec3, delta: Vec3) -> Option<SweepHit> {
        let horizontal = Vec3::new(delta.x, 0., delta.z);
        if horizontal.length_squared() < 1e-12 {
            return None;
        }
        let valid = theia_core::coordinates::LocalPosition::new(feet.to_array())
            .and_then(|p| self.frame.to_global(p))
            .is_some_and(|p| self.contains(p));
        if !valid {
            // Initial loading or withdrawal can leave the current footprint
            // outside permission. Hold XZ without suppressing vertical sweeps.
            return Some(SweepHit {
                distance: 0.,
                normal: -horizontal.normalize(),
            });
        }
        let radius = crate::kinematic::FOOTPRINT_RADIUS;
        let size = INITIAL_CHUNK_METRES as f32;
        let mut nearest: Option<SweepHit> = None;
        for n in 0..self.len {
            let id = self.ids[n].unwrap();
            for axis in 0..2 {
                let component = if axis == 0 { 0 } else { 2 };
                let other = if axis == 0 { 2 } else { 0 };
                for side in [-1i64, 1] {
                    let neighbor = if axis == 0 {
                        id.x.checked_add(side).map(|x| ChunkId { x, z: id.z })
                    } else {
                        id.z.checked_add(side).map(|z| ChunkId { x: id.x, z })
                    };
                    if neighbor.is_some_and(|id| self.ids[..self.len].contains(&Some(id))) {
                        continue;
                    }
                    let sign = side as f32;
                    if delta[component] * sign <= 0. {
                        continue;
                    }
                    let plane =
                        self.corners[n][axis] + if side > 0 { size - radius } else { radius };
                    let fraction = (plane - feet[component]) / delta[component];
                    if !(0.0..=1.0).contains(&fraction) {
                        continue;
                    }
                    let along = feet[other] + delta[other] * fraction;
                    let lower = self.corners[n][1 - axis];
                    if along < lower - radius || along > lower + size + radius {
                        continue;
                    }
                    let mut normal = Vec3::ZERO;
                    normal[component] = -sign;
                    let hit = SweepHit {
                        distance: fraction * delta.length(),
                        normal,
                    };
                    if nearest.is_none_or(|old| hit.distance < old.distance) {
                        nearest = Some(hit);
                    }
                }
            }
        }
        nearest
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Metrics {
    pub loads: usize,
    pub unloads: usize,
    pub max_active: usize,
}
/// One standard GLB root per active chunk, derived from immutable global truth.
/// A client consumer must still load/spawn/despawn these roots; this proof does
/// not count descriptors as rendered or asynchronously streamed content.
pub struct VisualRoot {
    pub chunk: ChunkId,
    pub asset: PathBuf,
    pub origin: GlobalPosition,
}
/// Active collision and its package ownership change together only after all
/// candidate artifacts validate. Peak staging can retain old + new sets.
#[derive(Clone)]
pub struct Residency {
    index: Index,
    budget: usize,
    active: BTreeMap<ChunkId, Arc<Terrain>>,
    scene: Option<Arc<StaticScene>>,
    metrics: Metrics,
    generation: u64,
}
impl Residency {
    pub fn new(index: Index, budget: usize) -> Result<Self> {
        if !(1..=MAX_RESIDENT).contains(&budget) {
            return Err("Resident budget must be 1..32 chunks".into());
        }
        Ok(Self {
            index,
            budget,
            active: BTreeMap::new(),
            scene: None,
            metrics: Metrics::default(),
            generation: 0,
        })
    }
    pub fn replace(&mut self, required: &[ChunkId], protected: &[GlobalPosition]) -> Result<()> {
        let ids: BTreeSet<_> = required.iter().copied().collect();
        if ids.len() != required.len()
            || ids.len() > self.budget
            || ids.iter().any(|id| !self.index.entries.contains_key(id))
        {
            return Err("Invalid or over-budget requested chunks".into());
        }
        for &position in protected {
            if self.index.owner(position).is_none_or(|id| {
                !self.index.contributors[&id]
                    .iter()
                    .all(|needed| ids.contains(needed))
            }) {
                return Err("Cannot unload a protected actor's geometry contributors".into());
            }
        }
        if self.active.keys().copied().eq(ids.iter().copied()) {
            return Ok(());
        }
        let mut next = BTreeMap::new();
        let mut loads = 0;
        for id in ids {
            let terrain = if let Some(terrain) = self.active.get(&id) {
                terrain.clone()
            } else {
                let directory = self.index.directory(id);
                // Chunk indices require packages, never the legacy fixture path.
                let manifest = Manifest::load_optional(&directory.join(MANIFEST_FILE))?
                    .ok_or("Chunk package is missing")?;
                if manifest.identity() != self.index.entries[&id] {
                    return Err("Chunk package disagrees with index".into());
                }
                let terrain = Terrain::load(&directory.join("terrain.json"))?;
                if terrain.identity != manifest.identity() {
                    return Err("Chunk changed during loading".into());
                }
                if let Some(bounds) = self.index.bounds(id) {
                    for collider in &terrain.physics.colliders {
                        let aabb =
                            collider
                                .shape
                                .aabb(collider.center, bevy::prelude::Quat::IDENTITY, 0.);
                        for (axis, component) in [0, 2].into_iter().enumerate() {
                            if f64::from(aabb.min[component]) < bounds[0][axis]
                                || f64::from(aabb.max[component]) > bounds[1][axis]
                            {
                                return Err("Collision exceeds indexed contributor bounds".into());
                            }
                        }
                    }
                }
                loads += 1;
                Arc::new(terrain)
            };
            next.insert(id, terrain);
        }
        let mut colliders = Vec::new();
        for (&id, terrain) in &next {
            if colliders.len() + terrain.physics.colliders.len() > MAX_COLLIDERS {
                return Err("Resident collision budget exceeded".into());
            }
            let offset = Vec3::from_array(
                self.index
                    .frame
                    .to_local(id.origin(INITIAL_CHUNK_METRES).unwrap())
                    .unwrap()
                    .metres(),
            );
            colliders.extend(terrain.physics.colliders.iter().map(|c| PlacedCollider {
                shape: c.shape.clone(),
                center: c.center + offset,
                layer: c.layer,
            }));
        }
        let scene = (!colliders.is_empty()).then(|| Arc::new(StaticScene::new(colliders)));
        let generation = self
            .generation
            .checked_add(1)
            .ok_or("Residency generation exhausted")?;
        self.metrics.loads += loads;
        self.metrics.unloads += self
            .active
            .keys()
            .filter(|id| !next.contains_key(id))
            .count();
        self.metrics.max_active = self.metrics.max_active.max(next.len());
        self.generation = generation;
        self.active = next;
        self.scene = scene;
        Ok(())
    }
    /// Independent candidate for a bounded worker; the published set stays usable.
    pub fn staged(&self, required: &[ChunkId], protected: &[GlobalPosition]) -> Result<Self> {
        let mut candidate = self.clone();
        candidate.replace(required, protected)?;
        Ok(candidate)
    }
    pub fn verify_visuals(&self) -> Result<()> {
        for (&id, terrain) in &self.active {
            terrain.verify_visual_in(
                &self.index.directory(id).join("terrain.glb"),
                &self.index.directory,
            )?;
        }
        Ok(())
    }
    /// Publication protects current/presented actor roots and trusted recovery
    /// feet. This never changes either residency or a pose while waiting.
    pub fn validate_presentation(
        &self,
        previous: &Self,
        roots: &[GlobalPosition],
        spawns: &[GlobalPosition],
    ) -> Result<()> {
        if roots.len() > crate::wire::MAX_PLAYERS * 3
            || spawns.len() > crate::wire::MAX_PLAYERS
            || self.index.identity() != previous.index.identity()
            || self.index.frame().origin() != previous.index.frame().origin()
        {
            return Err("Invalid presentation protection set".into());
        }
        let coverage = self.index.coverage(self.loaded())?;
        let scene = self.scene().ok_or("No candidate collision")?;
        let before = previous.scene().ok_or("No active collision")?;
        for &p in spawns {
            if !coverage.contains(p) {
                return Err("Candidate removes recovery coverage".into());
            }
            crate::kinematic::validate_pose(
                scene,
                self.index
                    .frame
                    .to_local(p)
                    .ok_or("Unbounded recovery pose")?,
                true,
            )?;
        }
        for &root in roots {
            let mut p = root.metres();
            p[1] -= f64::from(crate::world::PAWN_Y);
            let p = GlobalPosition::new(p).ok_or("Invalid protected root")?;
            if !coverage.contains(p) {
                return Err("Candidate removes presented footprint".into());
            }
            let local = self
                .index
                .frame
                .to_local(p)
                .ok_or("Unbounded protected pose")?;
            crate::kinematic::validate_pose(scene, local, false)?;
            if let Some(old) = crate::kinematic::support_below(before, local)
                && crate::kinematic::support_below(scene, local)
                    .is_none_or(|new| (new - old).abs() > 0.02)
            {
                return Err("Candidate changes presented landing support".into());
            }
        }
        Ok(())
    }
    pub fn colliders(&self, id: ChunkId) -> Option<&[PlacedCollider]> {
        self.active.get(&id).map(|t| t.physics.colliders.as_slice())
    }
    pub fn loaded(&self) -> Vec<ChunkId> {
        self.active.keys().copied().collect()
    }
    pub fn visuals(&self) -> Vec<VisualRoot> {
        self.active
            .keys()
            .map(|&id| VisualRoot {
                chunk: id,
                asset: self.index.directory(id).join("terrain.glb"),
                origin: id.origin(INITIAL_CHUNK_METRES).unwrap(),
            })
            .collect()
    }
    pub fn scene(&self) -> Option<&StaticScene> {
        self.scene.as_deref()
    }
    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn budget(&self) -> usize {
        self.budget
    }
    pub fn metrics(&self) -> Metrics {
        self.metrics
    }
}
