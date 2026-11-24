//! Whole-area terrain: bounded overview geometry and nearby source-detail meshes.
//! File verification happens at document admission; frame updates use shared samples.
use super::*;
pub(crate) mod area_qualification;
mod qualification;
pub(crate) mod sculpt_qualification;
pub(super) fn install(app: &mut App) {
    qualification::install(app);
    sculpt_qualification::install(app);
    area_qualification::install(app);
}
use bevy::{
    asset::RenderAssetUsages,
    camera::primitives::MeshAabb,
    mesh::{Indices, PrimitiveTopology},
};
use std::sync::Arc;
use theia_core::coordinates::ChunkId;
use theia_zone::imported::Imported;

#[derive(Component)]
pub(super) struct Tile;
pub(super) type GroundFilter = Or<(With<terrain::GroundPart>, With<Tile>)>;
pub(super) fn overview_radius(source: &Imported) -> f32 {
    Vec2::new(
        (source.bounds[1][0] - source.bounds[0][0]) as f32,
        (source.bounds[1][1] - source.bounds[0][1]) as f32,
    )
    .length()
        * 0.5
}
struct Resident {
    id: ChunkId,
    entity: Entity,
    mesh: Handle<Mesh>,
    cells: usize,
    samples: Arc<[f32]>,
    weights: Option<Arc<[[u8; 4]]>>,
    neighbours: [Option<Arc<[f32]>>; 4],
}
#[derive(Resource, Default)]
pub(super) struct Scene {
    document: Option<(u64, [u8; 32])>,
    tiles: Vec<Resident>,
}
#[derive(Resource)]
pub(super) struct Material(Handle<StandardMaterial>);
pub(super) fn max_view_distance(zone: &Zone) -> f32 {
    if zone.imported_terrain.is_some() {
        12288.
    } else {
        selection::MAX_VIEW_DISTANCE
    }
}
pub(super) fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.init_resource::<Scene>();
    commands.insert_resource(Material(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.,
        ..default()
    })));
}
fn detail(source: &Imported, id: ChunkId, zone: &Zone, view: &View, brush: Option<Vec3>) -> usize {
    // The same bounded 5x5 detail region follows the active brush, even when
    // zoomed out. Otherwise a fine overlay intersects the coarse overview.
    let focus = brush
        .or_else(|| (view.distance < 512.).then_some(view.target))
        .and_then(|p| zone.global(p.to_array()).ok())
        .map(|p| [p[0], p[2]]);
    detail_at(source.cells, source.tile_count(), id, focus)
}
fn detail_at(cells: usize, count: usize, id: ChunkId, focus: Option<[f64; 2]>) -> usize {
    if focus.is_some_and(|p| {
        (id.x - (p[0] / 128.).floor() as i64).abs() <= 2
            && (id.z - (p[1] / 128.).floor() as i64).abs() <= 2
    }) {
        cells
    } else if count > 1024 {
        8
    } else {
        32
    }
}
fn neighbours(source: &Imported, id: ChunkId) -> [Option<Arc<[f32]>>; 4] {
    [(-1, 0), (1, 0), (0, -1), (0, 1)].map(|(x, z)| {
        source.tile_samples(ChunkId {
            x: id.x + x,
            z: id.z + z,
        })
    })
}
fn same_samples(a: &Option<Arc<[f32]>>, b: &Option<Arc<[f32]>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
pub(super) fn mesh(zone: &Zone, source: &Imported, id: ChunkId, h: &[f32], cells: usize) -> Mesh {
    let mut g = theia_network::procedural::grid(cells);
    let stride = source.cells / cells;
    let offset = zone
        .local([id.x as f64 * 128., 0., id.z as f64 * 128.])
        .expect("Admitted imported tile");
    for (i, p) in g.positions.iter_mut().enumerate() {
        p[0] += offset[0];
        p[2] += offset[2];
        p[1] = h[(i / (cells + 1)) * stride * (source.cells + 1) + (i % (cells + 1)) * stride]
            + offset[1];
    }
    let weights = source.tile_weights(id);
    let palette = [
        Color::srgb(0.38, 0.46, 0.28),
        Color::srgb(0.38, 0.24, 0.11),
        Color::srgb(0.37, 0.39, 0.43),
        Color::srgb(0.65, 0.55, 0.31),
    ]
    .map(|c| c.to_linear().to_f32_array());
    let mut colours: Vec<[f32; 4]> = (0..g.positions.len())
        .map(|i| {
            let w = weights.as_ref().map_or(theia_zone::painting::GRASS, |w| {
                w[(i / (cells + 1)) * stride * (source.cells + 1) + (i % (cells + 1)) * stride]
            });
            let mut c = [0., 0., 0., 1.];
            for channel in 0..3 {
                c[channel] = (0..4)
                    .map(|j| palette[j][channel] * f32::from(w[j]) / 255.)
                    .sum();
            }
            c
        })
        .collect();
    // Independent downward skirt vertices hide LOD T-junctions without changing
    // top-surface normals or the exact 1 m grounding samples.
    let minimum = h.iter().copied().fold(f32::INFINITY, f32::min);
    let maximum = h.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let depth = (maximum - minimum + 1.).max(4.);
    for edge in 0..4 {
        for i in 0..cells {
            let indices = match edge {
                0 => [i, i + 1],
                1 => [cells * (cells + 1) + i, cells * (cells + 1) + i + 1],
                2 => [i * (cells + 1), (i + 1) * (cells + 1)],
                _ => [i * (cells + 1) + cells, (i + 1) * (cells + 1) + cells],
            };
            let [a, b] = indices.map(|j| g.positions[j]);
            colours.extend([
                colours[indices[0]],
                colours[indices[1]],
                colours[indices[0]],
                colours[indices[1]],
            ]);
            let n = g.positions.len() as u32;
            g.positions
                .extend([a, b, [a[0], a[1] - depth, a[2]], [b[0], b[1] - depth, b[2]]]);
            g.triangles
                .extend([[n, n + 2, n + 1], [n + 1, n + 2, n + 3]]);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, g.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.insert_indices(Indices::U32(g.triangles.into_iter().flatten().collect()));
    mesh.compute_normals();
    // Central differences from full source/edit samples keep normals identical
    // on shared edges, independently of either tile's rendered LOD.
    if source.tile_count() > 0 {
        let spacing = 128. / source.cells as f64;
        let height = |x: isize, z: isize| -> f32 {
            if (0..=source.cells as isize).contains(&x) && (0..=source.cells as isize).contains(&z)
            {
                h[z as usize * (source.cells + 1) + x as usize]
            } else {
                source
                    .height([
                        (id.x as f64 * 128. + x as f64 * spacing)
                            .clamp(source.bounds[0][0], source.bounds[1][0]),
                        (id.z as f64 * 128. + z as f64 * spacing)
                            .clamp(source.bounds[0][1], source.bounds[1][1]),
                    ])
                    .unwrap()
            }
        };
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            for z in 0..=cells {
                for x in 0..=cells {
                    let sx = (x * stride) as isize;
                    let sz = (z * stride) as isize;
                    normals[z * (cells + 1) + x] = Vec3::new(
                        height(sx - 1, sz) - height(sx + 1, sz),
                        2. * spacing as f32,
                        height(sx, sz - 1) - height(sx, sz + 1),
                    )
                    .normalize()
                    .to_array();
                }
            }
        }
    }
    mesh
}
pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    view: Res<View>,
    mut scene: ResMut<Scene>,
    material: Res<Material>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let zone = session
        .stroke
        .as_ref()
        .map_or(session.editor.zone(), |s| s.draft.zone());
    let key = zone
        .imported_terrain
        .as_ref()
        .map(|t| (session.document_id, t.identity));
    if scene.document != key {
        for tile in scene.tiles.drain(..) {
            commands.entity(tile.entity).despawn();
            meshes.remove(tile.mesh.id());
        }
        scene.document = key;
    }
    let Some(source) = &zone.imported_terrain else {
        return;
    };
    // Bound mesh creation work; the complete overview appears progressively.
    for (id, h) in source.tiles().skip(scene.tiles.len()).take(8) {
        let cells = detail(
            source,
            id,
            zone,
            &view,
            session.brush_point.filter(|_| session.terrain_mode),
        );
        let mesh = meshes.add(mesh(zone, source, id, h, cells));
        let entity = commands
            .spawn((
                Tile,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.0.clone()),
                Transform::default(),
            ))
            .id();
        scene.tiles.push(Resident {
            id,
            entity,
            mesh,
            cells,
            samples: source.tile_samples(id).unwrap(),
            weights: source.tile_weights(id),
            neighbours: neighbours(source, id),
        });
    }
    let mut updated = 0;
    for tile in &mut scene.tiles {
        let cells = detail(
            source,
            tile.id,
            zone,
            &view,
            session.brush_point.filter(|_| session.terrain_mode),
        );
        let samples = source.tile_samples(tile.id).unwrap();
        let adjacent = neighbours(source, tile.id);
        let same_neighbours = tile
            .neighbours
            .iter()
            .zip(&adjacent)
            .all(|(a, b)| same_samples(a, b));
        let weights = source.tile_weights(tile.id);
        let same_weights = match (&tile.weights, &weights) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if tile.cells == cells
            && Arc::ptr_eq(&tile.samples, &samples)
            && same_weights
            && same_neighbours
        {
            continue;
        }
        let replacement = mesh(zone, source, tile.id, &samples, cells);
        // LOD can reveal peaks absent from the coarse samples. Keep visibility
        // and ray culling bounds in step with the replacement geometry.
        commands
            .entity(tile.entity)
            .insert(replacement.get_aabb().unwrap());
        *meshes.get_mut(&tile.mesh).expect("Resident imported mesh") = replacement;
        tile.cells = cells;
        tile.samples = samples;
        tile.weights = weights;
        tile.neighbours = adjacent;
        updated += 1;
        if updated == 2 {
            break;
        }
    }
}
impl Scene {
    pub(super) fn summary(&self) -> serde_json::Value {
        serde_json::json!({"tiles":self.tiles.len(),"fine_tiles":self.tiles.iter().filter(|t|t.cells>32).count(),
            "vertices":self.tiles.iter().map(|t|(t.cells+1).pow(2)+t.cells*16).sum::<usize>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Imported {
        serde_json::from_value(serde_json::json!({"package":"/tmp/mesh-only-fixture", "identity":vec![0;32], "bounds":[[0.,0.],[4096.,3072.]], "cells":128})).unwrap()
    }
    fn zone() -> Zone {
        Zone {
            schema: 16,
            origin: [2048., 0., 1536.],
            imported_terrain: Some(source()),
            ..default()
        }
    }
    #[test]
    fn imported_mesh_preserves_source_samples_and_coarse_tile_edges() {
        let z = zone();
        let s = z.imported_terrain.as_ref().unwrap();
        let heights: Vec<f32> = (0..129 * 129)
            .map(|i| 100. + (i % 129) as f32 * 0.25 + (i / 129) as f32 * 0.125)
            .collect();
        for cells in [8, 32, 128] {
            let m = mesh(&z, s, ChunkId { x: 31, z: 23 }, &heights, cells);
            let bevy::mesh::VertexAttributeValues::Float32x3(p) =
                m.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
            else {
                panic!()
            };
            assert_eq!(p.len(), (cells + 1).pow(2) + 16 * cells);
            assert_eq!(p[0], [1920., 100., 1408.]);
            assert_eq!(p[(cells + 1).pow(2) - 1], [2048., 148., 1536.]);
            let stride = 128 / cells;
            assert_eq!(p[cells + 2][1], heights[stride * 129 + stride]);
            let bevy::mesh::VertexAttributeValues::Float32x3(n) =
                m.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap()
            else {
                panic!()
            };
            assert!(n[..(cells + 1).pow(2)].iter().all(|normal| normal[1] > 0.9));
        }
    }
    #[test]
    fn detail_is_bounded_to_twenty_five_tiles_and_full_zone_pan_reaches_outer_edge() {
        let z = zone();
        let s = z.imported_terrain.as_ref().unwrap();
        let mut view = View {
            target: Vec3::ZERO,
            yaw: std::f32::consts::PI,
            pitch: 0.8,
            distance: 90.,
            preset: camera_views::Preset::Perspective,
            show_grid: false,
        };
        for target in [
            Vec3::ZERO,
            Vec3::new(64., 0., 64.),
            Vec3::new(1900., 0., -1400.),
        ] {
            view.target = target;
            let fine = (0..24)
                .flat_map(|z| (0..32).map(move |x| ChunkId { x, z }))
                .filter(|id| detail(s, *id, &z, &view, None) == 128)
                .count();
            assert!((1..=25).contains(&fine));
        }
        view.distance = 4000.;
        assert_eq!(detail(s, ChunkId { x: 16, z: 12 }, &z, &view, None), 32);
        view.preset = camera_views::Preset::Top;
        camera_views::pan(&mut view, Vec2::splat(10000.), 1000., z.local_bounds());
        assert!(view.target.x.abs() > 512. && view.target.z.abs() > 512.);
        assert!(view.target.x.abs() <= 2048. && view.target.z.abs() <= 1536.);
    }
}

#[cfg(test)]
mod area_detail_tests {
    use super::*;
    #[test]
    fn brush_focus_keeps_exactly_twenty_five_fine_tiles_and_covers_largest_support() {
        for focus in [[1024., 1024.], [1151.9, 1151.9]] {
            let fine = (0..20)
                .flat_map(|z| (0..20).map(move |x| ChunkId { x, z }))
                .filter(|id| detail_at(64, 3648, *id, Some(focus)) == 64)
                .count();
            assert_eq!(fine, 25);
            for dx in [-240., 0., 240.] {
                for dz in [-240., 0., 240.] {
                    let id = ChunkId {
                        x: ((focus[0] + dx) / 128.).floor() as i64,
                        z: ((focus[1] + dz) / 128.).floor() as i64,
                    };
                    assert_eq!(detail_at(64, 3648, id, Some(focus)), 64);
                }
            }
        }
        assert_eq!(detail_at(64, 3648, ChunkId { x: 8, z: 8 }, None), 8);
    }
}
