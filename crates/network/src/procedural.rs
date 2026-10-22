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
    let mut positions = Vec::with_capacity((cells + 1).pow(2));
    for z in 0..=cells {
        for x in 0..=cells {
            let lx = x as f32 * 128. / cells as f32;
            let lz = z as f32 * 128. / cells as f32;
            positions.push([
                lx,
                height(
                    id.x as f64 * 128. + f64::from(lx),
                    id.z as f64 * 128. + f64::from(lz),
                ),
                lz,
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
pub struct Patch {
    pub owner: ChunkId,
    pub frame: LocalFrame,
    pub scene: StaticScene,
    pub ids: BTreeSet<ChunkId>,
}
impl Patch {
    pub fn build(owner: ChunkId) -> Self {
        assert!(contains(owner));
        let frame = LocalFrame::new(owner.origin(128).unwrap());
        let ids = neighbours(owner, PATCH_RADIUS);
        let colliders = ids
            .iter()
            .map(|&id| {
                let g = geometry(id, CELLS);
                PlacedCollider {
                    shape: Collider::trimesh(
                        g.positions.into_iter().map(Vec3::from_array).collect(),
                        g.triangles,
                    ),
                    center: Vec3::new(
                        (id.x - owner.x) as f32 * 128.,
                        0.,
                        (id.z - owner.z) as f32 * 128.,
                    ),
                    layer: SOLID | GROUND | CAMERA,
                }
            })
            .collect();
        Self {
            owner,
            frame,
            scene: StaticScene::new(colliders),
            ids,
        }
    }
    pub fn spawn(&self, x: f64, z: f64) -> GlobalPosition {
        let requested = GlobalPosition::new([x, f64::from(height(x, z)) + 2., z]).unwrap();
        let mut local = self.frame.to_local(requested).unwrap().metres();
        local[1] = self
            .scene
            .ground([local[0], local[2]], local[1])
            .expect("Terrain surface")
            .0
            + 0.03;
        self.frame
            .to_global(
                crate::kinematic::spawn(
                    &self.scene,
                    theia_core::coordinates::LocalPosition::new(local).unwrap(),
                )
                .expect("Gentle procedural terrain supports spawn"),
            )
            .unwrap()
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
/// One worker, one pending job, one result. Retains at most one patch per actor
/// plus an admission patch; no all-world geometry or unbounded generation cache.
pub struct Runtime {
    pub patches: BTreeMap<ChunkId, Arc<Patch>>,
    sender: mpsc::SyncSender<ChunkId>,
    receiver: Mutex<mpsc::Receiver<Arc<Patch>>>,
    pending: Option<ChunkId>,
    pub generated: u64,
    pub retired: u64,
    pub spawn: [f64; 2],
    pub speed: f32,
}
impl Runtime {
    pub fn new(spawn: [f64; 2], speed: f32) -> crate::Result<Self> {
        if !matches!(speed, 1. | 8. | 32.)
            || spawn
                .iter()
                .any(|v| !v.is_finite() || !(1. ..=SIDE - 1.).contains(v))
        {
            return Err(
                "Graybox spawn must be inside 1..15999m and developer speed 1, 8 or 32".into(),
            );
        }
        let id = owner(GlobalPosition::new([spawn[0], 0., spawn[1]]).unwrap()).unwrap();
        let patch = Arc::new(Patch::build(id));
        let (sender, jobs) = mpsc::sync_channel(1);
        let (results, receiver) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("terrain-collision".into())
            .spawn(move || {
                while let Ok(id) = jobs.recv() {
                    if results.send(Arc::new(Patch::build(id))).is_err() {
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
            spawn,
            speed,
        })
    }
    /// Call from the owner pump, outside the fixed simulation step.
    pub fn poll(&mut self, positions: &[GlobalPosition]) {
        if let Ok(patch) = self.receiver.lock().unwrap().try_recv() {
            self.patches.insert(patch.owner, patch);
            self.pending = None;
            self.generated += 1;
        }
        let spawn =
            owner(GlobalPosition::new([self.spawn[0], 0., self.spawn[1]]).unwrap()).unwrap();
        let mut required = BTreeSet::from([spawn]);
        for p in positions {
            if let Some(id) = owner(*p) {
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
