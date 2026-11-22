//! Immutable height tiles for the existing continental terrain loader.
//! Opening reads only the bounded manifest; tile I/O belongs on terrain workers.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use ring::digest::{SHA256, digest};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use theia_core::{content::ContentId, coordinates::ChunkId};

pub const MANIFEST: &str = "terrain.manifest.json";
const AXIS: usize = 125;
const SAMPLES: usize = 33;
const MANIFEST_LIMIT: u64 = 8 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"THTILE01";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    content_id: String,
    revision: u32,
    chunk_metres: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chunks_per_axis: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chunks_xz: Option<[usize; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chunk_origin: Option<[i64; 2]>,
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
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn chunk_layout(&self) -> ([i64; 2], [usize; 2]) {
        (
            self.manifest.chunk_origin.unwrap_or([0, 0]),
            self.manifest.chunks_xz.unwrap_or([AXIS, AXIS]),
        )
    }
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let bytes = read(&root.join(MANIFEST), MANIFEST_LIMIT)?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        let layout_valid = match manifest.schema {
            1 => {
                manifest.chunks_per_axis == Some(AXIS)
                    && manifest.chunks_xz.is_none()
                    && manifest.chunk_origin.is_none()
                    && manifest.samples_per_axis == SAMPLES
                    && manifest.tiles.len() == AXIS * AXIS
            }
            2 => match (manifest.chunks_xz, manifest.chunk_origin) {
                (Some([x, z]), Some([ox, oz])) => {
                    manifest.chunks_per_axis.is_none()
                        && (1..=AXIS).contains(&x)
                        && (1..=AXIS).contains(&z)
                        && (0..AXIS as i64).contains(&ox)
                        && (0..AXIS as i64).contains(&oz)
                        && ox + x as i64 <= AXIS as i64
                        && oz + z as i64 <= AXIS as i64
                        && matches!(manifest.samples_per_axis, 33 | 65 | 129)
                        && manifest.tiles.len() == x * z
                }
                _ => false,
            },
            _ => false,
        };
        if !layout_valid
            || ContentId::new(&manifest.content_id).is_none()
            || manifest.revision == 0
            || manifest.chunk_metres != 128
        {
            return Err("Unsupported saved terrain manifest".into());
        }
        // Domain separate saved content from other world/package identities.
        let mut identity_bytes =
            format!("theia-saved-height-terrain-v{}\0", manifest.schema).into_bytes();
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
    pub fn cells(&self) -> usize {
        self.manifest.samples_per_axis - 1
    }
    pub fn bounds(&self) -> [[f64; 2]; 2] {
        let [x, z] = self.manifest.chunk_origin.unwrap_or([0, 0]);
        let [nx, nz] = self.manifest.chunks_xz.unwrap_or([AXIS, AXIS]);
        [
            [x as f64 * 128., z as f64 * 128.],
            [(x as f64 + nx as f64) * 128., (z as f64 + nz as f64) * 128.],
        ]
    }
    pub fn contains(&self, id: ChunkId) -> bool {
        let [x, z] = self.manifest.chunk_origin.unwrap_or([0, 0]);
        let [nx, nz] = self.manifest.chunks_xz.unwrap_or([AXIS, AXIS]);
        (x..x + nx as i64).contains(&id.x) && (z..z + nz as i64).contains(&id.z)
    }
    pub fn heights(&self, id: ChunkId) -> Result<Vec<f32>> {
        if !self.contains(id) {
            return Err("Terrain tile outside world".into());
        }
        let path = self.root.join(name(id)).canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err("Terrain tile escaped package".into());
        }
        let tile_bytes = 24 + self.manifest.samples_per_axis.pow(2) * 4;
        let bytes = read(&path, tile_bytes as u64)?;
        self.loads.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        let [ox, oz] = self.manifest.chunk_origin.unwrap_or([0, 0]);
        let width = self.manifest.chunks_xz.map_or(AXIS, |v| v[0]);
        let index = (id.z - oz) as usize * width + (id.x - ox) as usize;
        if bytes.len() != tile_bytes
            || hash(&bytes) != self.manifest.tiles[index]
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
        let limit = if self.manifest.schema == 1 {
            128.
        } else {
            1024.
        };
        if heights.iter().any(|h| !h.is_finite() || h.abs() > limit) {
            return Err("Terrain height outside supported schema range".into());
        }
        Ok(heights)
    }
    /// Offline complete-package check; runtime only validates worker-loaded tiles.
    pub fn verify(&self) -> Result<()> {
        let mut previous_row: Vec<Vec<f32>> = Vec::new();
        let [nx, nz] = self.manifest.chunks_xz.unwrap_or([AXIS, AXIS]);
        let [ox, oz] = self.manifest.chunk_origin.unwrap_or([0, 0]);
        let samples = self.manifest.samples_per_axis;
        let last = samples - 1;
        for z in 0..nz {
            let mut row: Vec<Vec<f32>> = Vec::new();
            for x in 0..nx {
                let h = self.heights(ChunkId {
                    x: ox + x as i64,
                    z: oz + z as i64,
                })?;
                if x > 0
                    && (0..samples).any(|i| {
                        row[x - 1][i * samples + last].to_bits() != h[i * samples].to_bits()
                    })
                {
                    return Err("Saved terrain X seam mismatch".into());
                }
                if z > 0
                    && (0..samples)
                        .any(|i| previous_row[x][last * samples + i].to_bits() != h[i].to_bits())
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
