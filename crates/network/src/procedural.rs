//! Versioned, deterministic 256 km² development terrain. No forest assets.
//! Geometry is generated on bounded workers; simulation only queries ready patches.
use crate::physics::{CAMERA, GROUND, PlacedCollider, SOLID, StaticScene};
use avian3d::prelude::Collider;
use bevy::prelude::Vec3;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, mpsc},
};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame};

pub const IDENTITY: [u8; 32] = *b"theia-graybox-16km-terrain-v1___";
pub const SIDE: f64 = 16000.;
pub const CELLS: usize = 32;
pub const VISUAL_RADIUS: i64 = 4;
pub const PATCH_RADIUS: i64 = 1;
pub const MAX_VISUALS: usize = 81;

pub fn contains(id: ChunkId) -> bool {
    (0..125).contains(&id.x) && (0..125).contains(&id.z)
}
pub fn owner(p: GlobalPosition) -> Option<ChunkId> {
    p.chunk(128).filter(|id| contains(*id))
}
pub fn neighbours(id: ChunkId, radius: i64) -> BTreeSet<ChunkId> {
    (id.x - radius..=id.x + radius)
        .flat_map(|x| (id.z - radius..=id.z + radius).map(move |z| ChunkId { x, z }))
        .filter(|id| contains(*id))
        .collect()
}
/// Every height sample uses integer world grid coordinates, including shared edges.
/// Gentle broad hills plus a smaller ridge band; no per-chunk random discontinuity.
pub fn height(x: f64, z: f64) -> f32 {
    (24. * (x / 410.).sin() * (z / 530.).cos()
        + 8. * ((x + z) / 145.).sin()
        + 3. * ((x - z) / 71.).cos()) as f32
}
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}
pub fn geometry(id: ChunkId, cells: usize) -> Geometry {
    assert!(contains(id) && matches!(cells, 8 | 32));
    let mut g = grid(cells);
    for p in &mut g.positions {
        p[1] = height(
            id.x as f64 * 128. + f64::from(p[0]),
            id.z as f64 * 128. + f64::from(p[2]),
        );
    }
    g
}
/// Shared regular triangulation. Saved terrain never evaluates the recipe.
pub fn grid(cells: usize) -> Geometry {
    assert!(matches!(cells, 8 | 32 | 64 | 96 | 128));
    let mut positions = Vec::with_capacity((cells + 1).pow(2));
    for z in 0..=cells {
        for x in 0..=cells {
            positions.push([
                x as f32 * 128. / cells as f32,
                0.,
                z as f32 * 128. / cells as f32,
            ]);
        }
    }
    let mut triangles = Vec::with_capacity(cells * cells * 2);
    for z in 0..cells {
        for x in 0..cells {
            let a = (z * (cells + 1) + x) as u32;
            let b = a + 1;
            let c = a + (cells + 1) as u32;
            let d = c + 1;
            triangles.extend([[a, c, b], [b, c, d]]);
        }
    }
    Geometry {
        positions,
        triangles,
    }
}
#[derive(Clone)]
pub enum Source {
    Procedural,
    Saved(Arc<crate::saved_terrain::Store>),
    Authored(Arc<crate::authored_world::Store>),
}
impl Source {
    pub fn saved(&self) -> Option<&Arc<crate::saved_terrain::Store>> {
        match self {
            Self::Saved(store) => Some(store),
            Self::Authored(store) => store.imported(),
            Self::Procedural => None,
        }
    }
    pub fn contains(&self, id: ChunkId) -> bool {
        self.saved()
            .map_or_else(|| contains(id), |s| s.contains(id))
    }
    pub fn cells(&self) -> usize {
        self.saved().map_or(CELLS, |s| s.cells())
    }
    pub fn neighbours(&self, id: ChunkId, radius: i64) -> BTreeSet<ChunkId> {
        neighbours(id, radius)
            .into_iter()
            .filter(|id| self.contains(*id))
            .collect()
    }
    pub fn valid_spawn_xz(&self, xz: [f64; 2]) -> bool {
        let bounds = self
            .saved()
            .map_or([[0., 0.], [SIDE, SIDE]], |s| s.bounds());
        (0..2)
            .all(|i| xz[i].is_finite() && (bounds[0][i] + 1. ..=bounds[1][i] - 1.).contains(&xz[i]))
    }
    pub fn identity(&self) -> [u8; 32] {
        match self {
            Self::Procedural => IDENTITY,
            Self::Saved(s) => s.identity(),
            Self::Authored(s) => s.identity(),
        }
    }
    pub fn geometry(&self, id: ChunkId, cells: usize) -> crate::Result<Geometry> {
        if let Some(store) = self.saved() {
            return store.geometry(id, cells);
        }
        match self {
            Self::Procedural => Ok(geometry(id, cells)),
            Self::Saved(s) => s.geometry(id, cells),
            Self::Authored(_) => Err(
                "Authored visuals require their bound standard GLB, not generated height meshes"
                    .into(),
            ),
        }
    }
}
pub struct Patch {
    pub owner: ChunkId,
    pub frame: LocalFrame,
    pub scene: StaticScene,
    pub ids: BTreeSet<ChunkId>,
}
impl Patch {
    pub fn build(owner: ChunkId) -> Self {
        Self::load(owner, &Source::Procedural).expect("Procedural patch")
    }
    pub fn load(owner: ChunkId, source: &Source) -> crate::Result<Self> {
        if let Source::Authored(store) = source {
            return store.patch(owner);
        }
        if !source.contains(owner) {
            return Err("Patch outside terrain".into());
        }
        let frame = LocalFrame::new(owner.origin(128).unwrap());
        let ids = source.neighbours(owner, PATCH_RADIUS);
        let cells = source.cells();
        let samples = cells + 1;
        let geometry: BTreeMap<_, _> = ids
            .iter()
            .map(|&id| Ok((id, source.geometry(id, cells)?)))
            .collect::<crate::Result<_>>()?;
        // A correctly hashed but incorrectly authored tile must not create a seam.
        for (&id, g) in &geometry {
            if let Some(next) = geometry.get(&ChunkId {
                x: id.x + 1,
                z: id.z,
            }) && (0..samples).any(|i| {
                g.positions[i * samples + cells][1].to_bits()
                    != next.positions[i * samples][1].to_bits()
            }) {
                return Err("Terrain patch X seam mismatch".into());
            }
            if let Some(next) = geometry.get(&ChunkId {
                x: id.x,
                z: id.z + 1,
            }) && (0..samples).any(|i| {
                g.positions[cells * samples + i][1].to_bits() != next.positions[i][1].to_bits()
            }) {
                return Err("Terrain patch Z seam mismatch".into());
            }
        }
        let colliders = geometry
            .into_iter()
            .map(|(id, g)| PlacedCollider {
                shape: if cells > CELLS {
                    // Parry stores heightfields column-major. Preserve the same
                    // [a,c,b]/[b,c,d] diagonal as the visible regular mesh.
                    Collider::heightfield(
                        (0..samples)
                            .map(|x| {
                                (0..samples)
                                    .map(|z| g.positions[z * samples + x][1])
                                    .collect()
                            })
                            .collect(),
                        Vec3::new(128., 1., 128.),
                    )
                } else {
                    Collider::trimesh(
                        g.positions.into_iter().map(Vec3::from_array).collect(),
                        g.triangles,
                    )
                },
                center: Vec3::new(
                    (id.x - owner.x) as f32 * 128. + if cells > CELLS { 64. } else { 0. },
                    0.,
                    (id.z - owner.z) as f32 * 128. + if cells > CELLS { 64. } else { 0. },
                ),
                layer: SOLID | GROUND | CAMERA,
            })
            .collect();
        Ok(Self {
            owner,
            frame,
            scene: StaticScene::new(colliders),
            ids,
        })
    }
    pub fn spawn(&self, x: f64, z: f64) -> GlobalPosition {
        self.try_spawn(x, z)
            .expect("Baseline terrain supports spawn")
    }
    pub fn try_spawn(&self, x: f64, z: f64) -> Option<GlobalPosition> {
        let requested = GlobalPosition::new([x, 2048., z])?;
        let mut local = self.frame.to_local(requested)?.metres();
        local[1] = self.scene.ground([local[0], local[2]], local[1])?.0 + 0.03;
        self.frame.to_global(
            crate::kinematic::spawn(
                &self.scene,
                theia_core::coordinates::LocalPosition::new(local)?,
            )
            .ok()?,
        )
    }
    /// Resolve only near the trusted full position; never search above its layer.
    pub fn try_spawn_near(&self, requested: GlobalPosition) -> Option<GlobalPosition> {
        let local = self.frame.to_local(requested)?;
        self.frame
            .to_global(crate::kinematic::spawn(&self.scene, local).ok()?)
    }
    /// Resolve the bounded landing support beneath trusted feet, including an
    /// airborne restored actor. Unlike the XZ fixture lookup, this cannot choose
    /// an overhead building roof as the actor's respawn home.
    pub fn try_landing_spawn(&self, requested: GlobalPosition) -> Option<GlobalPosition> {
        let local = self.frame.to_local(requested)?;
        crate::kinematic::validate_pose(&self.scene, local, false).ok()?;
        let mut feet = local.metres();
        feet[1] = crate::kinematic::support_below(&self.scene, local)?;
        let supported = theia_core::coordinates::LocalPosition::new(feet)?;
        self.frame
            .to_global(crate::kinematic::spawn(&self.scene, supported).ok()?)
    }
    pub fn covers(&self, p: GlobalPosition, ready: Option<&BTreeSet<ChunkId>>) -> bool {
        let [x, y, z] = p.metres();
        let r = f64::from(crate::kinematic::FOOTPRINT_RADIUS);
        [-r, r].into_iter().all(|dx| {
            [-r, r].into_iter().all(|dz| {
                GlobalPosition::new([x + dx, y, z + dz])
                    .and_then(owner)
                    .is_some_and(|id| {
                        self.ids.contains(&id) && ready.is_none_or(|r| r.contains(&id))
                    })
            })
        })
    }
}
/// One worker, one pending job, one result. Demand includes player/admission
/// footprints and the optional single patrol's home patch. No all-world geometry
/// or unbounded generation cache; patrol preparation shares this same cache.
pub struct Runtime {
    pub patches: BTreeMap<ChunkId, Arc<Patch>>,
    sender: mpsc::SyncSender<ChunkId>,
    receiver: Mutex<mpsc::Receiver<crate::Result<Arc<Patch>>>>,
    pending: Option<ChunkId>,
    pub generated: u64,
    pub retired: u64,
    pub source: Source,
    pub error: Option<String>,
    pub spawn: [f64; 2],
    pub speed: f32,
}
impl Runtime {
    pub fn new(spawn: [f64; 2], speed: f32) -> crate::Result<Self> {
        Self::with_source(spawn, speed, Source::Procedural)
    }
    pub fn with_source(spawn: [f64; 2], speed: f32, source: Source) -> crate::Result<Self> {
        if !matches!(speed, 1. | 8. | 32.) || !source.valid_spawn_xz(spawn) {
            return Err(
                "Terrain spawn must be inside its bounds and developer speed 1, 8 or 32".into(),
            );
        }
        if let Source::Authored(store) = &source {
            let [x, _, z] = store
                .spawn(0)
                .ok_or("Authored world has no spawn")?
                .metres();
            if spawn != [x, z] {
                return Err(
                    "Authored runtime requires the saved player spawn; edit the zone to move it"
                        .into(),
                );
            }
        }
        let id = owner(GlobalPosition::new([spawn[0], 0., spawn[1]]).unwrap()).unwrap();
        let patch = Arc::new(Patch::load(id, &source)?);
        let worker_source = source.clone();
        let (sender, jobs) = mpsc::sync_channel(1);
        let (results, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("terrain-collision".into())
            .spawn(move || {
                while let Ok(id) = jobs.recv() {
                    if results
                        .send(Patch::load(id, &worker_source).map(Arc::new))
                        .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            patches: BTreeMap::from([(id, patch)]),
            sender,
            receiver: Mutex::new(receiver),
            pending: None,
            generated: 1,
            retired: 0,
            source,
            error: None,
            spawn,
            speed,
        })
    }
    /// Call from the owner pump, outside the fixed simulation step.
    /// Safe admission feet. Authored slots retain their original XYZ; the older
    /// generated/saved development worlds retain their four horizontal offsets.
    pub fn spawn_feet(&self, slot: usize) -> Option<GlobalPosition> {
        if let Source::Authored(store) = &self.source {
            let requested = store.spawn(slot)?;
            return self.patch(requested)?.try_spawn_near(requested);
        }
        let offset = (slot % 4) as f64 * 2.;
        let x = if self
            .source
            .valid_spawn_xz([self.spawn[0] + offset, self.spawn[1]])
        {
            self.spawn[0] + offset
        } else {
            self.spawn[0] - offset
        };
        let z = self.spawn[1];
        self.patch(GlobalPosition::new([x, 0., z])?)?
            .try_spawn(x, z)
    }
    pub fn poll(&mut self, positions: &[GlobalPosition]) {
        if self.error.is_some() {
            return;
        }
        if let Ok(result) = self.receiver.lock().unwrap().try_recv() {
            self.pending = None;
            match result {
                Ok(patch) => {
                    self.patches.insert(patch.owner, patch);
                    self.generated += 1;
                }
                Err(error) => {
                    self.error = Some(format!("Terrain collision load failed: {error}"));
                    return;
                }
            }
        }
        let spawn =
            owner(GlobalPosition::new([self.spawn[0], 0., self.spawn[1]]).unwrap()).unwrap();
        let mut required = BTreeSet::from([spawn]);
        for p in positions {
            if let Some(id) = owner(*p).filter(|id| self.source.contains(*id)) {
                required.insert(id);
            }
        }
        // Retain an older patch that still supports an actor while its new owner
        // is preparing. Publication never creates a collision hole.
        let keep: BTreeSet<_> = positions
            .iter()
            .filter_map(|p| self.patch(*p).map(|p| p.owner))
            .chain(required.iter().copied())
            .collect();
        let before = self.patches.len();
        self.patches.retain(|id, _| keep.contains(id));
        self.retired += (before - self.patches.len()) as u64;
        if self.pending.is_none()
            && let Some(id) = required
                .iter()
                .find(|id| !self.patches.contains_key(id))
                .copied()
            && self.sender.try_send(id).is_ok()
        {
            self.pending = Some(id);
        }
    }
    pub fn patch(&self, p: GlobalPosition) -> Option<&Arc<Patch>> {
        owner(p)
            .and_then(|id| self.patches.get(&id))
            .or_else(|| self.patches.values().find(|patch| patch.covers(p, None)))
    }
}
