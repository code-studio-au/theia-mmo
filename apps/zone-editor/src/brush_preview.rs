//! Terrain-only sample influence, on the exact authored/rendered triangles.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
};

pub(super) const LIFT: f32 = 0.04;
pub(super) const OPACITY: f32 = 0.5;

#[cfg(test)]
pub(super) fn mesh(zone: &Zone, centre: [f64; 2], radius: f64) -> Option<Mesh> {
    mesh_with_edge(zone, centre, radius, 0.)
}

pub(super) fn mesh_with_edge(
    zone: &Zone,
    centre: [f64; 2],
    radius: f64,
    edge: f64,
) -> Option<Mesh> {
    if zone.imported_terrain.is_some() {
        return imported_mesh(zone, centre, radius, edge);
    }
    let mut influence = zone
        .terrain
        .as_ref()?
        .brush_influence(centre, radius + edge)
        .ok()?;
    if !influence.iter().any(|&w| w > 0.) {
        return None;
    }
    let mut geometry = terrain::geometry(zone)?;
    if edge > 0. {
        for (weight, position) in influence.iter_mut().zip(&geometry.positions) {
            if *weight > 0. {
                let p = zone.global(*position).ok()?;
                *weight = theia_zone::terrain::area::influence(
                    (p[0] - centre[0]).hypot(p[2] - centre[1]),
                    radius,
                    edge,
                );
            }
        }
    }
    for position in &mut geometry.positions {
        position[1] += LIFT;
    }
    let colour = Color::srgb(1., 0.65, 0.12).to_linear().to_f32_array();
    let colours: Vec<_> = influence
        .into_iter()
        .map(|weight| [colour[0], colour[1], colour[2], OPACITY * weight])
        .collect();
    let normals = vec![[0., 1., 0.]; geometry.positions.len()];
    let mut result = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    result.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry.positions);
    result.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    result.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    result.insert_indices(Indices::U32(
        geometry.triangles.into_iter().flatten().collect(),
    ));
    Some(result)
}

fn imported_mesh(zone: &Zone, centre: [f64; 2], core_radius: f64, edge: f64) -> Option<Mesh> {
    let source = zone.imported_terrain.as_ref()?;
    let radius = core_radius + edge;
    if !radius.is_finite()
        || !(4. ..=240.).contains(&radius)
        || !centre.iter().all(|v| v.is_finite())
    {
        return None;
    }
    if (0..2).any(|i| !(source.bounds[0][i]..=source.bounds[1][i]).contains(&centre[i])) {
        return None;
    }
    let step = 128. / source.cells as f64;
    let lo: [f64; 2] = std::array::from_fn(|i| {
        ((centre[i] - radius) / step)
            .floor()
            .mul_add(step, 0.)
            .max(source.bounds[0][i])
    });
    let hi: [f64; 2] = std::array::from_fn(|i| {
        ((centre[i] + radius) / step)
            .ceil()
            .mul_add(step, 0.)
            .min(source.bounds[1][i])
    });
    let nx = ((hi[0] - lo[0]) / step).round() as usize + 1;
    let nz = ((hi[1] - lo[1]) / step).round() as usize + 1;
    let colour = Color::srgb(1., 0.65, 0.12).to_linear().to_f32_array();
    let mut p = Vec::with_capacity(nx * nz);
    let mut c = Vec::with_capacity(nx * nz);
    let mut indices = vec![];
    for z in 0..nz {
        for x in 0..nx {
            let xz = [lo[0] + x as f64 * step, lo[1] + z as f64 * step];
            let mut local = zone.local(zone.surface_position(xz).ok()?).ok()?;
            local[1] += LIFT;
            p.push(local);
            let distance = (xz[0] - centre[0]).hypot(xz[1] - centre[1]);
            let w = if edge > 0. {
                theia_zone::terrain::area::influence(distance, core_radius, edge)
            } else {
                (1. - distance / radius).max(0.) as f32
            };
            c.push([colour[0], colour[1], colour[2], OPACITY * w]);
            if x + 1 < nx && z + 1 < nz {
                let a = (z * nx + x) as u32;
                let b = a + 1;
                let d = a + nx as u32;
                indices.extend([a, d, b, b, d, d + 1]);
            }
        }
    }
    let mut result = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    result.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; p.len()]);
    result.insert_attribute(Mesh::ATTRIBUTE_POSITION, p);
    result.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    result.insert_indices(Indices::U32(indices));
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use theia_zone::terrain::{HeightPatch, Resolution};
    #[test]
    fn preview_tracks_exact_terrain_triangles_and_shared_weights_in_both_far_frames() {
        for resolution in [Resolution::FourMetres, Resolution::TwoMetres] {
            for sign in [-1_i64, 1] {
                let mut patch = HeightPatch::flat([sign * 1_000_000_000, -sign * 1_000_000_000]);
                patch.resolution = resolution;
                let axis = patch.axis();
                patch.heights = vec![0.; axis * axis];
                patch.heights[(axis / 2) * axis + axis / 2] = 64.;
                patch.heights[(axis / 2 + 1) * axis + axis / 2] = -32.;
                let mut origin = patch.origin().unwrap();
                origin[1] = 20.;
                let zone = Zone {
                    schema: 7,
                    origin,
                    spawn: [origin[0] + 16., 20.1, origin[2] + 16.],
                    terrain: Some(patch),
                    ..default()
                };
                zone.validate().unwrap();
                let before = zone.clone();
                let centre = [origin[0] + 64., origin[2] + 64.];
                let result = mesh(&zone, centre, 16.).unwrap();
                let geometry = terrain::geometry(&zone).unwrap();
                let Some(VertexAttributeValues::Float32x3(positions)) =
                    result.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    panic!("positions");
                };
                let Some(VertexAttributeValues::Float32x4(colours)) =
                    result.attribute(Mesh::ATTRIBUTE_COLOR)
                else {
                    panic!("colours");
                };
                let weights = zone
                    .terrain
                    .as_ref()
                    .unwrap()
                    .brush_influence(centre, 16.)
                    .unwrap();
                assert_eq!(positions.len(), axis * axis);
                for (i, (position, source)) in positions.iter().zip(&geometry.positions).enumerate()
                {
                    assert_eq!(*position, [source[0], source[1] + LIFT, source[2]]);
                    assert_eq!(colours[i][3], weights[i] * OPACITY);
                }
                assert!(positions.iter().any(|p| p[1] > 64.));
                assert!(positions.iter().any(|p| p[1] < -31.));
                assert_eq!(
                    result.indices().unwrap().iter().collect::<Vec<_>>(),
                    geometry
                        .triangles
                        .into_iter()
                        .flatten()
                        .map(|i| i as usize)
                        .collect::<Vec<_>>()
                );
                assert_eq!(zone, before);
            }
        }
    }
    #[test]
    fn empty_missing_and_outside_brushes_do_not_draw_a_false_footprint() {
        assert!(mesh(&Zone::default(), [64., 64.], 8.).is_none());
        let zone = Zone {
            schema: 2,
            terrain: Some(HeightPatch::flat([0, 0])),
            ..default()
        };
        assert!(mesh(&zone, [-1., 64.], 8.).is_none());
        assert!(mesh(&zone, [0., 0.], 4.).is_none());
        assert!(mesh(&zone, [0., 64.], 8.).is_some());
    }
}
