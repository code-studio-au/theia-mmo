//! Offline-only Recast bake/query adapter. No backend dependency enters the server.
//! Pipeline uses the published Rerecast 0.5 API, reviewed against its C++ comparison
//! test. World-facing geometry remains Y-up; Landmass's internal frame is Z-up.
use landmass::{
    Archipelago, ArchipelagoOptions, FromAgentRadius, HeightNavigationMesh, HeightPolygon, Island,
    NavigationMesh, PathStep, PermittedAnimationLinks, PointSampleDistance3d, Transform, Vec3, XYZ,
};
use nav_glam::{UVec3, Vec3A};
use rerecast::{AreaType, ConfigBuilder, DetailNavmesh, HeightfieldBuilder, TriMesh};
use std::{collections::HashMap, sync::Arc};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame, LocalPosition};
use theia_network::{
    Result,
    navigation::{Agent, MAX_ROUTE_POINTS, MAX_TILES, Tile},
    procedural::Geometry,
};
pub fn bake(id: ChunkId, source: [u8; 32], agent: Agent, geometry: Geometry) -> Result<Tile> {
    if !agent.valid()
        || !theia_network::procedural::contains(id)
        || geometry.positions.is_empty()
        || geometry.positions.len() > 16_384
        || geometry.triangles.is_empty()
        || geometry.triangles.len() > 32_768
    {
        return Err("Navigation bake input exceeds supported profile/geometry bounds".into());
    }
    if geometry.positions.iter().any(|p| {
        !p.iter().all(|v| v.is_finite())
            || p[0].abs() > 256.
            || p[2].abs() > 256.
            || p[1].abs() > 128.
    }) || geometry
        .triangles
        .iter()
        .flatten()
        .any(|i| *i as usize >= geometry.positions.len())
    {
        return Err("Invalid navigation bake geometry".into());
    }
    let mut mesh = TriMesh {
        vertices: geometry.positions.into_iter().map(Vec3A::from).collect(),
        indices: geometry.triangles.iter().map(|p| UVec3::from(*p)).collect(),
        area_types: vec![AreaType::NOT_WALKABLE; geometry.triangles.len()],
    };
    let mut aabb = mesh.compute_aabb().ok_or("Empty navigation source")?;
    aabb.min.y -= 1.;
    aabb.max.y += agent.height;
    let mut config = ConfigBuilder {
        aabb,
        agent_radius: agent.radius,
        agent_height: agent.height,
        walkable_climb: agent.step,
        walkable_slope_angle: agent.slope_radians,
        cell_size_fraction: agent.radius / agent.cell_size,
        cell_height_fraction: agent.radius / agent.cell_height,
        min_region_size: 1,
        merge_region_size: 8,
        ..Default::default()
    }
    .build();
    let padding = f32::from(config.border_size) * config.cell_size;
    aabb.min.x = -padding;
    aabb.min.z = -padding;
    aabb.max.x = 128. + padding;
    aabb.max.z = 128. + padding;
    config.aabb = aabb;
    mesh.mark_walkable_triangles(config.walkable_slope_angle);
    let mut field = HeightfieldBuilder {
        aabb,
        cell_size: config.cell_size,
        cell_height: config.cell_height,
    }
    .build()?;
    field.rasterize_triangles(&mesh, config.walkable_climb)?;
    field.filter_low_hanging_walkable_obstacles(config.walkable_climb);
    field.filter_ledge_spans(config.walkable_height, config.walkable_climb);
    field.filter_walkable_low_height_spans(config.walkable_height);
    let mut compact = field.into_compact(config.walkable_height, config.walkable_climb)?;
    compact.erode_walkable_area(config.walkable_radius);
    compact.build_distance_field();
    compact.build_regions(
        config.border_size,
        config.min_region_area,
        config.merge_region_area,
    )?;
    let mesh = compact
        .build_contours(
            config.max_simplification_error,
            config.max_edge_len,
            config.contour_flags,
        )
        .into_polygon_mesh(config.max_vertices_per_polygon)?;
    let detail = DetailNavmesh::new(
        &mesh,
        &compact,
        config.detail_sample_dist,
        config.detail_sample_max_error,
    )?;
    if detail
        .vertices
        .iter()
        .any(|v| !(-0.001..=128.001).contains(&v.x) || !(-0.001..=128.001).contains(&v.z))
    {
        return Err("Detail height mesh escaped clipped tile boundary".into());
    }
    let mut vertices = Vec::with_capacity(mesh.vertices.len());
    for v in &mesh.vertices {
        let mut p = [
            mesh.aabb.min.x + f32::from(v.x) * mesh.cell_size,
            mesh.aabb.min.y + f32::from(v.y) * mesh.cell_height,
            mesh.aabb.min.z + f32::from(v.z) * mesh.cell_size,
        ];
        for axis in [0, 2] {
            if !(-0.001..=128.001).contains(&p[axis]) {
                return Err("Baked polygon escaped clipped tile boundary".into());
            }
            p[axis] = p[axis].clamp(0., 128.);
        }
        vertices.push(p);
    }
    let tile = Tile {
        schema: 1,
        id: [id.x, id.z],
        source,
        agent,
        vertices,
        polygons: mesh.polygons().map(Iterator::collect).collect(),
        detail: theia_network::navigation::Detail {
            vertices: detail
                .vertices
                .iter()
                .map(|v| [v.x.clamp(0., 128.), v.y, v.z.clamp(0., 128.)])
                .collect(),
            triangles: detail.triangles,
            polygons: detail
                .meshes
                .iter()
                .map(|p| {
                    [
                        p.base_vertex_index,
                        p.vertex_count,
                        p.base_triangle_index,
                        p.triangle_count,
                    ]
                })
                .collect(),
        },
    };
    tile.validate()?;
    Ok(tile)
}
/// Offline path queries only. The prototype has no runtime expansion budget.
/// Bound package geometry first; reject routes that leave the prepared package.
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
