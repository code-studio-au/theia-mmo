use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::coordinates::ChunkId;
use theia_network::{
    saved_terrain::{self, Store},
    terrain::sha256,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Fixture {
    pub root: PathBuf,
}
impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "theia-saved-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        saved_terrain::bake(&root, "theia-test-terrain", 1).unwrap();
        Self { root }
    }
    pub fn open(&self) -> Store {
        Store::open(&self.root).unwrap()
    }
    pub fn tile(&self, id: ChunkId) -> PathBuf {
        self.root.join(format!("tile_{}_{}.height", id.x, id.z))
    }
    pub fn rehash(&self, id: ChunkId) {
        let path = self.root.join(saved_terrain::MANIFEST);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["tiles"][id.z as usize * 125 + id.x as usize] =
            serde_json::json!(sha256(&fs::read(self.tile(id)).unwrap()));
        fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
