//! Immutable height tiles for the existing continental terrain loader.
//! Opening reads only the bounded manifest; tile I/O belongs on terrain workers.
use crate::{
    Result,
    procedural::{self, Geometry},
};
use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::{content::ContentId, coordinates::ChunkId};

pub const MANIFEST: &str = "terrain.manifest.json";
const AXIS: usize = 125;
const SAMPLES: usize = 33;
const TILE_BYTES: usize = 24 + SAMPLES * SAMPLES * 4;
const MANIFEST_LIMIT: u64 = 8 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"THTILE01";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    content_id: String,
    revision: u32,
    chunk_metres: u32,
    chunks_per_axis: usize,
    samples_per_axis: usize,
    /// Z-major, then X. Tile coordinates also occur inside each hashed file.
    tiles: Vec<[u8; 32]>,
}

pub struct Store {
    root: PathBuf,
    manifest: Manifest,
    identity: [u8; 32],
    loads: AtomicU64,
    bytes: AtomicU64,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Stats {
    pub tile_reads: u64,
    pub tile_bytes: u64,
}
fn hash(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes).as_ref().try_into().unwrap()
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Terrain file exceeds byte budget".into());
    }
    Ok(bytes)
}
fn name(id: ChunkId) -> String {
    format!("tile_{}_{}.height", id.x, id.z)
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let bytes = read(&root.join(MANIFEST), MANIFEST_LIMIT)?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        if manifest.schema != 1
            || ContentId::new(&manifest.content_id).is_none()
            || manifest.revision == 0
            || manifest.chunk_metres != 128
            || manifest.chunks_per_axis != AXIS
            || manifest.samples_per_axis != SAMPLES
            || manifest.tiles.len() != AXIS * AXIS
        {
            return Err("Unsupported saved terrain manifest".into());
        }
        // Domain separate saved content from other world/package identities.
        let mut identity_bytes = b"theia-saved-height-terrain-v1\0".to_vec();
        identity_bytes.extend(&bytes);
        Ok(Self {
            root,
            manifest,
            identity: hash(&identity_bytes),
            loads: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn stats(&self) -> Stats {
        Stats {
            tile_reads: self.loads.load(Ordering::Relaxed),
            tile_bytes: self.bytes.load(Ordering::Relaxed),
        }
    }
    pub fn heights(&self, id: ChunkId) -> Result<Vec<f32>> {
        if !procedural::contains(id) {
            return Err("Terrain tile outside world".into());
        }
        let path = self.root.join(name(id)).canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err("Terrain tile escaped package".into());
        }
        let bytes = read(&path, TILE_BYTES as u64)?;
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        if bytes.len() != TILE_BYTES
            || hash(&bytes) != self.manifest.tiles[id.z as usize * AXIS + id.x as usize]
            || &bytes[..8] != MAGIC
            || i64::from_le_bytes(bytes[8..16].try_into()?) != id.x
            || i64::from_le_bytes(bytes[16..24].try_into()?) != id.z
        {
            return Err(format!(
                "Terrain tile identity/length/hash mismatch: {},{}",
                id.x, id.z
            )
            .into());
        }
        let heights: Vec<_> = bytes[24..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| f32::from_le_bytes(*v))
            .collect();
        if heights.iter().any(|h| !h.is_finite() || h.abs() > 128.) {
            return Err("Terrain height outside supported -128..128m range".into());
        }
        Ok(heights)
    }
    pub fn geometry(&self, id: ChunkId, cells: usize) -> Result<Geometry> {
        if !matches!(cells, 8 | 32) {
            return Err("Unsupported terrain detail".into());
        }
        let heights = self.heights(id)?;
        // Topology and triangulation are shared with the retained baseline.
        let mut g = procedural::grid(cells);
        for (i, p) in g.positions.iter_mut().enumerate() {
            p[1] = heights
                [(i / (cells + 1)) * (32 / cells) * SAMPLES + (i % (cells + 1)) * (32 / cells)];
        }
        Ok(g)
    }
    /// Offline complete-package check; runtime only validates worker-loaded tiles.
    pub fn verify(&self) -> Result<()> {
        let mut previous_row: Vec<Vec<f32>> = Vec::new();
        for z in 0..AXIS {
            let mut row: Vec<Vec<f32>> = Vec::new();
            for x in 0..AXIS {
                let h = self.heights(ChunkId {
                    x: x as i64,
                    z: z as i64,
                })?;
                if x > 0
                    && (0..SAMPLES)
                        .any(|i| row[x - 1][i * SAMPLES + 32].to_bits() != h[i * SAMPLES].to_bits())
                {
                    return Err("Saved terrain X seam mismatch".into());
                }
                if z > 0
                    && (0..SAMPLES)
                        .any(|i| previous_row[x][32 * SAMPLES + i].to_bits() != h[i].to_bits())
                {
                    return Err("Saved terrain Z seam mismatch".into());
                }
                row.push(h);
            }
            previous_row = row;
        }
        Ok(())
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
    let manifest = Manifest {
        schema: 1,
        content_id: content_id.into(),
        revision,
        chunk_metres: 128,
        chunks_per_axis: AXIS,
        samples_per_axis: SAMPLES,
        tiles,
    };
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(MANIFEST))?
        .write_all(&serde_json::to_vec(&manifest)?)?;
    Ok(())
}
