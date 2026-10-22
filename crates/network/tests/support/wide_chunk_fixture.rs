//! Original 6x6 grid for full-capacity collision/coverage/secure UDP checks.
use serde_json::json;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::coordinates::ChunkId;
use theia_network::{content::Manifest, terrain::sha256};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct WideFixture {
    pub root: PathBuf,
    pub ids: Vec<ChunkId>,
}
impl WideFixture {
    pub fn new(base: i64) -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-wide-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let mut ids = Vec::new();
        let mut entries = Vec::new();
        for x in 0..6 {
            for z in 0..6 {
                let id = ChunkId {
                    x: base + x,
                    z: base + z,
                };
                let directory = root.join(format!("chunk_{}_{}", id.x, id.z));
                std::fs::create_dir(&directory).unwrap();
                let visual = b"Original headless full-capacity fixture; not a rendered asset";
                let collision = serde_json::to_vec(&json!({"schema":3,"bounds":[[0.,0.],[128.,128.]],"source_origin":[0.,0.,0.],"spawns":[],"visual_sha256":sha256(visual),"triangles":[[[0.,0.,0.],[0.,0.,128.],[128.,0.,0.]],[[128.,0.,0.],[0.,0.,128.],[128.,0.,128.]]]})).unwrap();
                let manifest = serde_json::to_vec(&json!({"schema":1,"content_id":format!("theia.test.wide.x{}-z{}",id.x,id.z),"revision":1,"collision_schema":3,"collision_bytes":collision.len(),"collision_sha256":sha256(&collision),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)})).unwrap();
                std::fs::write(directory.join("terrain.json"), collision).unwrap();
                std::fs::write(directory.join("terrain.glb"), visual).unwrap();
                std::fs::write(directory.join("world.manifest.json"), &manifest).unwrap();
                entries.push(json!({"id":[id.x,id.z],"package":Manifest::decode(&manifest).unwrap().identity(),"bounds":[[0.,0.],[128.,128.]]}));
                ids.push(id);
            }
        }
        std::fs::write(root.join("world.chunks.json"), serde_json::to_vec(&json!({"schema":2,"chunk_metres":128,"query_origin":[base as f64*128.+8.,25.,base as f64*128.-8.],"chunks":entries})).unwrap()).unwrap();
        Self { root, ids }
    }
}
impl Drop for WideFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
