//! Reviewed static Landmass adapter shared by the offline baker and path worker.
//! Query construction and searches must stay outside the fixed simulation step.
use super::{MAX_ROUTE_POINTS, MAX_TILES, Tile};
use crate::Result;
use landmass::{
    Archipelago, ArchipelagoOptions, FromAgentRadius, HeightNavigationMesh, HeightPolygon, Island,
    NavigationMesh, PathStep, PermittedAnimationLinks, PointSampleDistance3d, Transform, Vec3, XYZ,
};
use std::{collections::HashMap, sync::Arc};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame, LocalPosition};
/// Static query adapter for loaders and the dedicated path worker only.
/// Landmass has no expansion limit: never call this from a simulation step.
pub struct Query {
    nav: Archipelago<XYZ>,
    frame: LocalFrame,
}
impl Query {
    pub fn new(tiles: &[Tile]) -> Result<Self> {
        if tiles.is_empty() || tiles.len() > MAX_TILES {
            return Err("Navigation query requires 1..9 tiles".into());
        }
        let agent = tiles[0].agent;
        let source = tiles[0].source;
        let mut ids = std::collections::BTreeSet::new();
        let first = ChunkId {
            x: tiles[0].id[0],
            z: tiles[0].id[1],
        };
        let frame = LocalFrame::new(first.origin(128).ok_or("Invalid query frame")?);
        let mut nav = Archipelago::<XYZ>::new(ArchipelagoOptions::from_agent_radius(agent.radius));
        for tile in tiles {
            tile.validate()?;
            if tile.agent != agent || tile.source != source || !ids.insert(tile.id) {
                return Err("Navigation query has mismatched or duplicate tiles".into());
            }
            let origin = ChunkId {
                x: tile.id[0],
                z: tile.id[1],
            }
            .origin(128)
            .ok_or("Invalid tile origin")?;
            let local = frame
                .to_local(origin)
                .ok_or("Tile outside bounded query frame")?
                .metres();
            let vertices = tile
                .vertices
                .iter()
                .map(|p| Vec3::new(p[0], -p[2], p[1]))
                .collect();
            let mesh = NavigationMesh::<XYZ> {
                vertices,
                polygons: tile
                    .polygons
                    .iter()
                    .map(|p| p.iter().copied().map(usize::from).collect())
                    .collect(),
                polygon_type_indices: vec![0; tile.polygons.len()],
                height_mesh: Some(HeightNavigationMesh {
                    vertices: tile
                        .detail
                        .vertices
                        .iter()
                        .map(|p| Vec3::new(p[0], -p[2], p[1]))
                        .collect(),
                    triangles: tile.detail.triangles.clone(),
                    polygons: tile
                        .detail
                        .polygons
                        .iter()
                        .map(|&[base, count, first, triangles]| HeightPolygon {
                            base_vertex_index: base,
                            vertex_count: count,
                            base_triangle_index: first,
                            triangle_count: triangles,
                        })
                        .collect(),
                }),
            };
            nav.add_island(Island::new(
                Transform {
                    translation: Vec3::new(local[0], -local[2], local[1]),
                    rotation: 0.,
                },
                Arc::new(mesh.validate()?),
            ));
        }
        // Builds static island connections; there are no agents/crowd obstacles.
        nav.update(0.);
        Ok(Self { nav, frame })
    }
    pub fn sample(&self, point: GlobalPosition) -> Result<GlobalPosition> {
        let p = self
            .frame
            .to_local(point)
            .ok_or("Point outside navigation frame")?
            .metres();
        let distance = PointSampleDistance3d {
            horizontal_distance: 0.25,
            distance_above: 0.2,
            distance_below: 0.2,
            ..PointSampleDistance3d::from_agent_radius(0.355)
        };
        let p = self
            .nav
            .sample_point(Vec3::new(p[0], -p[2], p[1]), &distance)?
            .point();
        self.frame
            .to_global(LocalPosition::new([p.x, p.z, -p.y]).ok_or("Invalid sampled point")?)
            .ok_or_else(|| "Invalid global sampled point".into())
    }
    pub fn route(&self, start: GlobalPosition, end: GlobalPosition) -> Result<Vec<GlobalPosition>> {
        let local = |p| -> Result<Vec3> {
            let p = self
                .frame
                .to_local(p)
                .ok_or("Route outside query frame")?
                .metres();
            Ok(Vec3::new(p[0], -p[2], p[1]))
        };
        // Refined height triangles permit strict layer sampling instead of
        // snapping an unreachable floor endpoint onto a roof.
        let distance = PointSampleDistance3d {
            horizontal_distance: 0.25,
            distance_above: 0.2,
            distance_below: 0.2,
            ..PointSampleDistance3d::from_agent_radius(0.355)
        };
        let start = self.nav.sample_point(local(start)?, &distance)?;
        let end = self.nav.sample_point(local(end)?, &distance)?;
        let path = self.nav.find_path(
            &start,
            &end,
            &HashMap::new(),
            PermittedAnimationLinks::default(),
        )?;
        if !(2..=MAX_ROUTE_POINTS).contains(&path.len()) {
            return Err("Navigation path point count exceeds budget".into());
        }
        path.into_iter()
            .map(|step| {
                let PathStep::Waypoint(p) = step else {
                    return Err("Animation links are outside this navigation slice".into());
                };
                let p = LocalPosition::new([p.x, p.z, -p.y]).ok_or("Nonfinite local path point")?;
                self.frame
                    .to_global(p)
                    .ok_or_else(|| "Invalid global path point".into())
            })
            .collect()
    }
}
