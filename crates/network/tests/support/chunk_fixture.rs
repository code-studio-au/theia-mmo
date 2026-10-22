//! Original headless package fixtures; visual bytes bind identity, not rendering.
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalPosition};
use theia_network::{
    chunks::{Index, Residency},
    content::Manifest,
    simulation::World,
    terrain::sha256,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Fixture {
    pub root: PathBuf,
    pub base: i64,
}
impl Fixture {
    pub fn new(base: i64, raised_left: bool, wall_right: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-activation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let fixture = Self { root, base };
        let mut entries = Vec::new();
        for offset in [-1, 0, 1] {
            let id = fixture.id(offset);
            let directory = fixture.root.join(format!("chunk_{}_{}", id.x, id.z));
            std::fs::create_dir(&directory).unwrap();
            let visual = b"Original headless collision fixture; not rendered";
            let mut triangles = vec![
                [[-1., 0., -1.], [-1., 0., 129.], [129., 0., -1.]],
                [[129., 0., -1.], [-1., 0., 129.], [129., 0., 129.]],
            ];
            if offset == -1 && raised_left {
                triangles.extend([
                    [[168., 2., 27.], [168., 2., 37.], [178., 2., 27.]],
                    [[178., 2., 27.], [168., 2., 37.], [178., 2., 37.]],
                ]);
            }
            let solids = if offset == 1 && wall_right {
                vec![json!({"center":[-83.,1.5,32.],"size":[2.,3.,2.]})]
            } else {
                vec![]
            };
            let profile = serde_json::to_vec(&json!({"schema":3,"bounds":[[-1.,-1.],[129.,129.]],"source_origin":[0.,0.,0.],"visual_sha256":sha256(visual),"triangles":triangles,"solids":solids,"spawns":[]})).unwrap();
            let manifest = serde_json::to_vec(&json!({"schema":1,"content_id":format!("theia.test.activation.x{}-z{}",id.x,id.z),"revision":1,"collision_schema":3,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)})).unwrap();
            std::fs::write(directory.join("terrain.json"), &profile).unwrap();
            std::fs::write(directory.join("terrain.glb"), visual).unwrap();
            std::fs::write(directory.join("world.manifest.json"), &manifest).unwrap();
            entries.push(
                json!({"id":[id.x,id.z],"package":Manifest::decode(&manifest).unwrap().identity()}),
            );
        }
        std::fs::write(fixture.root.join("world.chunks.json"),serde_json::to_vec(&json!({"schema":1,"chunk_metres":128,"query_origin":[base as f64*128.,0.,base as f64*128.],"chunks":entries})).unwrap()).unwrap();
        fixture
    }
    pub fn index(&self) -> Index {
        Index::load(&self.root).unwrap()
    }
    pub fn id(&self, offset: i64) -> ChunkId {
        ChunkId {
            x: self.base + offset,
            z: self.base,
        }
    }
    pub fn point(&self, x: f32, y: f32) -> GlobalPosition {
        self.index()
            .frame()
            .to_global(LocalPosition::new([x, y, 32.]).unwrap())
            .unwrap()
    }
    pub fn residency(&self, offsets: &[i64]) -> Residency {
        let mut r = Residency::new(self.index(), 2).unwrap();
        r.replace(
            &offsets.iter().map(|&o| self.id(o)).collect::<Vec<_>>(),
            &[],
        )
        .unwrap();
        r
    }
    pub fn world(&self, offsets: &[i64], x: f32) -> World {
        World::with_residency(self.residency(offsets), &[self.point(x, 0.)], -18.).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
