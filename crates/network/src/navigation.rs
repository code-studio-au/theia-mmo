//! Bounded offline navigation artifacts. File I/O is for loading/bake workers,
//! never the fixed step. This module does not perform pathfinding or move actors.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use theia_core::{
    content::ContentId,
    coordinates::{ChunkId, GlobalPosition},
};
pub const MANIFEST: &str = "navigation.manifest.json";
pub const MAX_TILES: usize = 9;
pub const MAX_VERTICES: usize = 8192;
pub const MAX_POLYGONS: usize = 4096;
pub const MAX_ROUTE_POINTS: usize = 256;
pub const MAX_MANIFEST_BYTES: usize = 256 * 1024;
pub const MAX_TILE_BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub radius: f32,
    pub height: f32,
    pub step: f32,
    pub slope_radians: f32,
    pub cell_size: f32,
    pub cell_height: f32,
}
impl Agent {
    pub fn development() -> Self {
        let shape = crate::kinematic::clearance();
        Self {
            radius: shape.radius,
            height: shape.height,
            step: shape.step,
            slope_radians: 40_f32.to_radians(),
            cell_size: 0.2,
            cell_height: 0.05,
        }
    }
    pub fn valid(self) -> bool {
        let shape = crate::kinematic::clearance();
        [
            self.radius,
            self.height,
            self.step,
            self.slope_radians,
            self.cell_size,
            self.cell_height,
        ]
        .iter()
        .all(|v| v.is_finite())
            && (shape.radius..=2.).contains(&self.radius)
            && (shape.height..=4.).contains(&self.height)
            && (0. ..=shape.step).contains(&self.step)
            && (0. ..=std::f32::consts::FRAC_PI_2).contains(&self.slope_radians)
            && self.slope_radians.cos() >= shape.minimum_up - 0.000001
            && (0.05..=0.5).contains(&self.cell_size)
            && (0.025..=self.cell_size).contains(&self.cell_height)
            && (128. / self.cell_size - (128. / self.cell_size).round()).abs() < 0.001
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Detail {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u8; 3]>,
    /// Base vertex, vertex count, base triangle, triangle count per polygon.
    pub polygons: Vec<[u32; 4]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub schema: u32,
    pub id: [i64; 2],
    pub source: [u8; 32],
    pub agent: Agent,
    /// Y-up feet, relative to the integer tile's XZ origin, Y origin zero.
    pub vertices: Vec<[f32; 3]>,
    /// Convex clockwise XZ contours, 3..6 distinct indices per polygon.
    pub polygons: Vec<Vec<u16>>,
    pub detail: Detail,
}
fn chunk(id: [i64; 2]) -> ChunkId {
    ChunkId { x: id[0], z: id[1] }
}
impl Tile {
    pub fn validate(&self) -> Result<()> {
        if self.schema != 1
            || !crate::procedural::contains(chunk(self.id))
            || !self.agent.valid()
            || self.vertices.is_empty()
            || self.vertices.len() > MAX_VERTICES
            || self.polygons.is_empty()
            || self.polygons.len() > MAX_POLYGONS
        {
            return Err(
                "Navigation tile schema, coordinates, profile or count exceeds bounds".into(),
            );
        }
        let mut grid = Vec::with_capacity(self.vertices.len());
        for p in &self.vertices {
            if !p.iter().all(|v| v.is_finite())
                || !(0. ..=128.).contains(&p[0])
                || !(0. ..=128.).contains(&p[2])
                || p[1].abs() > 129.
            {
                return Err("Navigation vertex outside bounded tile-local frame".into());
            }
            let x = p[0] / self.agent.cell_size;
            let z = p[2] / self.agent.cell_size;
            if (x - x.round()).abs() > 0.001 || (z - z.round()).abs() > 0.001 {
                return Err("Navigation vertex is not on its declared voxel grid".into());
            }
            grid.push([x.round() as i64, z.round() as i64]);
        }
        for polygon in &self.polygons {
            if !(3..=6).contains(&polygon.len())
                || polygon
                    .iter()
                    .any(|i| usize::from(*i) >= self.vertices.len())
                || polygon
                    .iter()
                    .enumerate()
                    .any(|(i, v)| polygon[..i].contains(v))
            {
                return Err("Invalid navigation polygon indices".into());
            }
            let points: Vec<_> = polygon.iter().map(|i| grid[usize::from(*i)]).collect();
            let mut area = 0_i64;
            for i in 0..points.len() {
                let a = points[i];
                let b = points[(i + 1) % points.len()];
                let dx = b[0] - a[0];
                let dz = b[1] - a[1];
                if (dx == 0 && dz == 0)
                    || points
                        .iter()
                        .any(|p| dx * (p[1] - a[1]) - dz * (p[0] - a[0]) > 0)
                {
                    return Err(
                        "Navigation polygon edge is degenerate or not convex clockwise XZ".into(),
                    );
                }
                area += a[0] * b[1] - a[1] * b[0];
            }
            if area >= 0 {
                return Err("Degenerate or reversed navigation polygon".into());
            }
        }
        if self.detail.polygons.len() != self.polygons.len()
            || self.detail.vertices.len() > 32768
            || self.detail.triangles.len() > 65536
            || self.detail.vertices.iter().any(|p| {
                !p.iter().all(|v| v.is_finite())
                    || !(0. ..=128.).contains(&p[0])
                    || !(0. ..=128.).contains(&p[2])
                    || p[1].abs() > 129.
            })
        {
            return Err("Navigation detail mesh exceeds bounds or has invalid vertices".into());
        }
        let mut vertex_end = 0_u32;
        let mut triangle_end = 0_u32;
        for (i, &[base, count, first, triangles]) in self.detail.polygons.iter().enumerate() {
            if base != vertex_end
                || first != triangle_end
                || !(self.polygons[i].len() as u32..=127).contains(&count)
                || !(1..=255).contains(&triangles)
            {
                return Err("Invalid navigation detail polygon layout".into());
            }
            vertex_end = base
                .checked_add(count)
                .ok_or("Detail vertex count overflow")?;
            triangle_end = first
                .checked_add(triangles)
                .ok_or("Detail triangle count overflow")?;
            if vertex_end as usize > self.detail.vertices.len()
                || triangle_end as usize > self.detail.triangles.len()
            {
                return Err("Navigation detail polygon references absent data".into());
            }
            let vertices = &self.detail.vertices[base as usize..vertex_end as usize];
            for (j, &index) in self.polygons[i].iter().enumerate() {
                let a = self.vertices[usize::from(index)];
                let b = vertices[j];
                if (a[0] - b[0]).abs() > 0.001 || (a[2] - b[2]).abs() > 0.001 {
                    return Err("Detail boundary differs from navigation polygon".into());
                }
            }
            for j in 0..self.polygons[i].len() {
                let a = self.vertices[usize::from(self.polygons[i][j])];
                let b =
                    self.vertices[usize::from(self.polygons[i][(j + 1) % self.polygons[i].len()])];
                let dx = f64::from(b[0]) - f64::from(a[0]);
                let dz = f64::from(b[2]) - f64::from(a[2]);
                if vertices.iter().any(|p| {
                    dx * (f64::from(p[2]) - f64::from(a[2]))
                        - dz * (f64::from(p[0]) - f64::from(a[0]))
                        > 0.001 * (dx.abs() + dz.abs())
                }) {
                    return Err("Detail vertex lies outside its navigation polygon".into());
                }
            }
            for indices in &self.detail.triangles[first as usize..triangle_end as usize] {
                if indices.iter().any(|v| u32::from(*v) >= count) {
                    return Err("Navigation detail triangle index out of bounds".into());
                }
                let [a, b, c] = indices.map(|v| vertices[usize::from(v)]);
                let cross = (f64::from(b[0]) - f64::from(a[0]))
                    * (f64::from(c[2]) - f64::from(a[2]))
                    - (f64::from(b[2]) - f64::from(a[2])) * (f64::from(c[0]) - f64::from(a[0]));
                if cross >= -0.00000001 {
                    return Err("Degenerate or reversed navigation detail triangle".into());
                }
            }
        }
        if vertex_end as usize != self.detail.vertices.len()
            || triangle_end as usize != self.detail.triangles.len()
        {
            return Err("Navigation detail mesh contains unowned data".into());
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_TILE_BYTES {
            return Err("Navigation tile exceeds byte budget".into());
        }
        let tile: Self = serde_json::from_slice(bytes)?;
        tile.validate()?;
        Ok(tile)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > MAX_TILE_BYTES {
            return Err("Navigation tile exceeds byte budget".into());
        }
        Ok(bytes)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileRecord {
    pub id: [i64; 2],
    pub sha256: [u8; 32],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub id: String,
    pub points: Vec<[f64; 3]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub source: [u8; 32],
    pub agent: Agent,
    pub tiles: Vec<TileRecord>,
    pub routes: Vec<Route>,
}
impl Manifest {
    pub fn validate(&self, expected_source: [u8; 32]) -> Result<()> {
        if self.schema != 1
            || self.source != expected_source
            || !self.agent.valid()
            || self.tiles.is_empty()
            || self.tiles.len() > MAX_TILES
            || self.routes.len() > 8
        {
            return Err("Navigation manifest schema, source, profile or count mismatch".into());
        }
        let mut previous = None;
        for tile in &self.tiles {
            if !crate::procedural::contains(chunk(tile.id))
                || previous.is_some_and(|id| id >= tile.id)
            {
                return Err(
                    "Navigation tiles must be distinct, sorted and inside the world".into(),
                );
            }
            previous = Some(tile.id);
        }
        let mut route_ids = std::collections::BTreeSet::new();
        let mut count = 0;
        for route in &self.routes {
            if ContentId::new(&route.id).is_none()
                || !route_ids.insert(&route.id)
                || !(2..=MAX_ROUTE_POINTS).contains(&route.points.len())
            {
                return Err("Invalid navigation route ID or point count".into());
            }
            count += route.points.len();
            let mut length = 0_f64;
            for (i, point) in route.points.iter().enumerate() {
                let p =
                    GlobalPosition::new(*point).ok_or("Invalid global navigation route point")?;
                let tile = p.chunk(128).ok_or("Unaddressable navigation route point")?;
                if point[1].abs() > 129. || !self.tiles.iter().any(|v| v.id == [tile.x, tile.z]) {
                    return Err(
                        "Navigation route references an absent tile or unsupported height".into(),
                    );
                }
                if i > 0 {
                    length += point
                        .iter()
                        .zip(route.points[i - 1])
                        .map(|(a, b)| (a - b) * (a - b))
                        .sum::<f64>()
                        .sqrt();
                }
            }
            if !(0.1..=1024.).contains(&length) {
                return Err("Navigation route exceeds local patrol distance budget".into());
            }
        }
        if count > 1024 {
            return Err("Navigation manifest exceeds total route point budget".into());
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate(self.source)?;
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err("Navigation manifest exceeds byte budget".into());
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], expected_source: [u8; 32]) -> Result<Self> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err("Navigation manifest exceeds byte budget".into());
        }
        let manifest: Self = serde_json::from_slice(bytes)?;
        manifest.validate(expected_source)?;
        Ok(manifest)
    }
}
pub fn tile_name(id: [i64; 2]) -> String {
    format!("tile_{}_{}.nav.json", id[0], id[1])
}
fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("Navigation file exceeds byte budget".into());
    }
    Ok(bytes)
}
pub struct Store {
    root: PathBuf,
    manifest: Manifest,
    identity: [u8; 32],
}
impl Store {
    pub fn open(root: &Path, expected_source: [u8; 32]) -> Result<Self> {
        let root = root.canonicalize()?;
        let bytes = read(&root.join(MANIFEST), MAX_MANIFEST_BYTES)?;
        let manifest = Manifest::decode(&bytes, expected_source)?;
        let mut identity = b"theia-static-navigation-v1\0".to_vec();
        identity.extend(&bytes);
        Ok(Self {
            root,
            manifest,
            identity: crate::terrain::sha256(&identity),
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn tile(&self, id: [i64; 2]) -> Result<Tile> {
        let record = self
            .manifest
            .tiles
            .iter()
            .find(|v| v.id == id)
            .ok_or("Navigation tile is not in this bounded package")?;
        let path = self.root.join(tile_name(id)).canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err("Navigation tile escaped package".into());
        }
        let bytes = read(&path, MAX_TILE_BYTES)?;
        if crate::terrain::sha256(&bytes) != record.sha256 {
            return Err("Navigation tile hash mismatch".into());
        }
        let tile = Tile::decode(&bytes)?;
        if tile.id != id || tile.source != self.manifest.source || tile.agent != self.manifest.agent
        {
            return Err("Navigation tile belongs to another source, profile or coordinate".into());
        }
        Ok(tile)
    }
    pub fn verify(&self) -> Result<()> {
        for tile in &self.manifest.tiles {
            self.tile(tile.id)?;
        }
        Ok(())
    }
}
