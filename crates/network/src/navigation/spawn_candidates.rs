//! Bounded seeded proposals on navigation surfaces. Loader/worker use only.
//! Proposals are not admitted spawns: callers must check world collision, support,
//! access, final volume containment and spacing before publishing a candidate pool.
use super::{MAX_TILES, Tile};
use crate::Result;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use theia_core::{
    content::ContentId,
    coordinates::{ChunkId, GlobalPosition, LocalFrame, LocalPosition},
    spawn_volume::PolygonVolume,
};

pub const MAX_ATTEMPTS: usize = 4096;
pub const MAX_COUNT: usize = 64;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: u32,
    pub id: String,
    pub volume: PolygonVolume,
    pub count: usize,
    pub seed: u64,
    pub minimum_spacing_metres: f64,
    /// A known accessible feet position. Does not impose a roaming/chase boundary.
    pub access_from: [f64; 3],
}
impl Request {
    pub fn validate(&self) -> Result<()> {
        if self.schema != 1
            || ContentId::new(&self.id).is_none()
            || !(1..=MAX_COUNT).contains(&self.count)
            || !self.minimum_spacing_metres.is_finite()
            || !(0.25..=32.).contains(&self.minimum_spacing_metres)
            || GlobalPosition::new(self.access_from).is_none()
        {
            return Err(
                "Invalid spawn candidate request identity/count/spacing/access point".into(),
            );
        }
        Ok(())
    }
}
struct Triangle {
    frame: LocalFrame,
    vertices: [[f32; 3]; 3],
    cumulative_area: f64,
}
/// Fixed draw budget and canonical tile order; no unbounded retry loop. Triangles
/// are weighted by horizontal area, including detail heights for layered surfaces.
pub fn proposals(
    tiles: &[Tile],
    volume: &PolygonVolume,
    seed: u64,
    attempts: usize,
) -> Result<Vec<GlobalPosition>> {
    if tiles.is_empty() || tiles.len() > MAX_TILES || !(1..=MAX_ATTEMPTS).contains(&attempts) {
        return Err("Spawn proposals exceed tile or attempt budget".into());
    }
    let mut ordered = tiles.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|t| t.id);
    let min = [0, 1].map(|axis| {
        volume
            .polygon()
            .iter()
            .map(|p| p[axis])
            .fold(f64::INFINITY, f64::min)
    });
    let max = [0, 1].map(|axis| {
        volume
            .polygon()
            .iter()
            .map(|p| p[axis])
            .fold(f64::NEG_INFINITY, f64::max)
    });
    let mut triangles = Vec::new();
    let mut total_area = 0.;
    let mut previous = None;
    for tile in ordered {
        tile.validate()?;
        if tile.source != tiles[0].source
            || tile.agent != tiles[0].agent
            || previous == Some(tile.id)
        {
            return Err("Spawn proposals require matching distinct navigation tiles".into());
        }
        previous = Some(tile.id);
        let frame = LocalFrame::new(
            ChunkId {
                x: tile.id[0],
                z: tile.id[1],
            }
            .origin(128)
            .ok_or("Invalid spawn tile frame")?,
        );
        for &[base, _, first, count] in &tile.detail.polygons {
            for indices in &tile.detail.triangles[first as usize..(first + count) as usize] {
                let vertices =
                    indices.map(|i| tile.detail.vertices[base as usize + usize::from(i)]);
                let world = vertices.map(|v| {
                    frame
                        .to_global(LocalPosition::new(v).expect("Validated detail vertex"))
                        .expect("Bounded tile frame")
                        .metres()
                });
                if [
                    (0, min[0], max[0]),
                    (2, min[1], max[1]),
                    (1, volume.min_y(), volume.max_y()),
                ]
                .iter()
                .any(|&(axis, lo, hi)| {
                    world.iter().all(|v| v[axis] < lo) || world.iter().all(|v| v[axis] > hi)
                }) {
                    continue;
                }
                let [a, b, c] = vertices.map(|v| v.map(f64::from));
                let area =
                    ((b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])).abs() * 0.5;
                total_area += area;
                triangles.push(Triangle {
                    frame,
                    vertices,
                    cumulative_area: total_area,
                });
            }
        }
    }
    if triangles.is_empty() || !total_area.is_finite() || total_area <= 0. {
        return Err("No navigation surface overlaps the spawn bounds".into());
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut points = Vec::new();
    for _ in 0..attempts {
        let area = rng.random::<f64>() * total_area;
        let index = triangles
            .partition_point(|t| t.cumulative_area <= area)
            .min(triangles.len() - 1);
        let triangle = &triangles[index];
        let root = rng.random::<f64>().sqrt();
        let second = rng.random::<f64>();
        let weights = [1. - root, root * (1. - second), root * second];
        let local = std::array::from_fn(|axis| {
            (0..3)
                .map(|i| f64::from(triangle.vertices[i][axis]) * weights[i])
                .sum::<f64>() as f32
        });
        let point = triangle
            .frame
            .to_global(LocalPosition::new(local).ok_or("Invalid sampled spawn point")?)
            .ok_or("Invalid global spawn point")?;
        if volume.contains(point) {
            points.push(point);
        }
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::{Agent, Detail};
    fn tile(id: [i64; 2], height: f32) -> Tile {
        let vertices = vec![
            [0., height, 0.],
            [0., height, 128.],
            [128., height, 128.],
            [128., height, 0.],
        ];
        Tile {
            schema: 1,
            id,
            source: [7; 32],
            agent: Agent::development(),
            vertices: vertices.clone(),
            polygons: vec![vec![0, 1, 2, 3]],
            detail: Detail {
                vertices,
                triangles: vec![[0, 1, 2], [0, 2, 3]],
                polygons: vec![[0, 4, 0, 2]],
            },
        }
    }
    #[test]
    fn proposals_are_seeded_order_independent_inside_concave_volume_and_on_the_requested_layer() {
        let volume = PolygonVolume::new(
            vec![
                [7952., 7952.],
                [8180., 7952.],
                [8180., 7980.],
                [8000., 7980.],
                [8000., 8050.],
                [7952., 8050.],
            ],
            -1.,
            1.,
        )
        .unwrap();
        let tiles = [tile([62, 62], 0.), tile([63, 62], 0.)];
        let points = proposals(&tiles, &volume, 42, 512).unwrap();
        assert!(points.len() > 30);
        assert!(
            points
                .iter()
                .all(|p| volume.contains(*p) && p.metres()[1] == 0.)
        );
        assert_eq!(
            points,
            proposals(&[tiles[1].clone(), tiles[0].clone()], &volume, 42, 512).unwrap()
        );
        assert_ne!(points, proposals(&tiles, &volume, 43, 512).unwrap());
        let roof = tile([62, 62], 5.);
        assert!(proposals(&[roof], &volume, 42, 512).is_err());
        assert!(proposals(&tiles, &volume, 42, MAX_ATTEMPTS + 1).is_err());
        assert!(proposals(&[tiles[0].clone(), tiles[0].clone()], &volume, 42, 10).is_err());
        let mut foreign = tiles[1].clone();
        foreign.source = [8; 32];
        assert!(proposals(&[tiles[0].clone(), foreign], &volume, 42, 10).is_err());
    }
    #[test]
    fn request_limits_refuse_nonfinite_or_unbounded_search_and_unknown_fields() {
        let valid = Request {
            schema: 1,
            id: "theia.test.spawns".into(),
            volume: PolygonVolume::new(vec![[0., 0.], [8., 0.], [0., 8.]], -1., 1.).unwrap(),
            count: 5,
            seed: 42,
            minimum_spacing_metres: 2.,
            access_from: [1., 0., 1.],
        };
        valid.validate().unwrap();
        for count in [0, MAX_COUNT + 1] {
            let mut r = valid.clone();
            r.count = count;
            assert!(r.validate().is_err());
        }
        for spacing in [f64::NAN, f64::INFINITY, 0., 33.] {
            let mut r = valid.clone();
            r.minimum_spacing_metres = spacing;
            assert!(r.validate().is_err());
        }
        let mut doc = serde_json::to_value(&valid).unwrap();
        doc["trusted"] = true.into();
        assert!(serde_json::from_value::<Request>(doc).is_err());
    }
}
