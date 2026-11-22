//! Geometry adapter and offline baker for the shared immutable height-tile reader.
use crate::{
    Result,
    procedural::{self, Geometry},
};
use ring::digest::{SHA256, digest};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    ops::Deref,
    path::Path,
};
use theia_core::{content::ContentId, coordinates::ChunkId};
pub use theia_zone::height_tiles::{MANIFEST, Stats};
const AXIS: usize = 125;
const SAMPLES: usize = 33;
const MAGIC: &[u8; 8] = b"THTILE01";
fn hash(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
fn name(id: ChunkId) -> String {
    format!("tile_{}_{}.height", id.x, id.z)
}
pub struct Store(theia_zone::height_tiles::Store);
impl Deref for Store {
    type Target = theia_zone::height_tiles::Store;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        theia_zone::height_tiles::Store::open(root).map(Self)
    }
    pub fn geometry(&self, id: ChunkId, cells: usize) -> Result<Geometry> {
        if !matches!(cells, 8 | 32 | 64 | 128) || cells > self.cells() {
            return Err("Unsupported terrain detail".into());
        }
        let heights = self.heights(id)?;
        // Topology and triangulation are shared with the retained baseline.
        let mut g = procedural::grid(cells);
        for (i, p) in g.positions.iter_mut().enumerate() {
            p[1] = heights[(i / (cells + 1)) * (self.cells() / cells) * (self.cells() + 1)
                + (i % (cells + 1)) * (self.cells() / cells)];
        }
        Ok(g)
    }
}

/// Offline tool. Refuses existing destinations; manifest publication is last.
pub fn bake(root: &Path, content_id: &str, revision: u32) -> Result<()> {
    if ContentId::new(content_id).is_none() || revision == 0 {
        return Err("Invalid terrain ID/revision".into());
    }
    fs::create_dir(root)?;
    let mut tiles = Vec::with_capacity(AXIS * AXIS);
    for z in 0..AXIS {
        for x in 0..AXIS {
            let id = ChunkId {
                x: x as i64,
                z: z as i64,
            };
            let mut bytes = MAGIC.to_vec();
            bytes.extend(id.x.to_le_bytes());
            bytes.extend(id.z.to_le_bytes());
            for p in procedural::geometry(id, 32).positions {
                bytes.extend(p[1].to_le_bytes());
            }
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(name(id)))?
                .write_all(&bytes)?;
            tiles.push(hash(&bytes));
        }
    }
    let manifest = serde_json::json!({
        "schema": 1, "content_id": content_id, "revision": revision,
        "chunk_metres": 128, "chunks_per_axis": AXIS,
        "samples_per_axis": SAMPLES, "tiles": tiles,
    });
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(MANIFEST))?
        .write_all(&serde_json::to_vec(&manifest)?)?;
    Ok(())
}
