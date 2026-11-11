use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, MeshVertexAttributeId, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};

use crate::asset::{ParticleMesh, QuadOrientation, RibbonTrailShape};

/// Cache for instanced particle meshes, keyed by mesh configuration and particle count.
#[derive(Resource, Default)]
pub struct ParticleMeshCache {
    cache: HashMap<(ParticleMesh, u32), Handle<Mesh>>,
}

impl ParticleMeshCache {
    pub fn get_or_create(
        &mut self,
        config: &ParticleMesh,
        particle_count: u32,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.cache
            .entry((config.clone(), particle_count))
            .or_insert_with(|| build_particle_mesh(config, particle_count, meshes))
            .clone()
    }
}

// Mesh configurations/counts change while authoring. Keep sharing live meshes, but
// release historical configurations rather than pinning every edit for app lifetime.
pub(crate) fn prune_particle_mesh_cache(
    mut cache: ResMut<ParticleMeshCache>,
    meshes: Query<&Mesh3d>,
) {
    let used: HashSet<_> = meshes.iter().map(|mesh| mesh.0.id()).collect();
    cache.cache.retain(|_, mesh| used.contains(&mesh.id()));
}

fn create_cylinder_mesh(
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    radial_segments: u32,
    rings: u32,
    cap_top: bool,
    cap_bottom: bool,
) -> Mesh {
    let radial_segments = radial_segments.max(3);
    let rings = rings.max(1);
    let half_height = height / 2.0;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Compute the slope in f64 so a very small nonzero height cannot overflow
    // before normalization. A flat cylinder has no unique side normal.
    let slope = if height == 0.0 {
        0.0
    } else {
        (f64::from(bottom_radius) - f64::from(top_radius)) / f64::from(height)
    };
    let scale = 1.0 / (1.0 + slope * slope).sqrt();
    let side_normal_scale = scale as f32;
    let side_normal_y = (slope * scale) as f32;

    for ring in 0..=rings {
        let v = ring as f32 / rings as f32;
        let y = half_height - height * v;
        let radius = top_radius + (bottom_radius - top_radius) * v;

        for segment in 0..=radial_segments {
            let u = segment as f32 / radial_segments as f32;
            let theta = u * std::f32::consts::TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            let x = cos_theta * radius;
            let z = sin_theta * radius;

            positions.push([x, y, z]);

            let nx = cos_theta * side_normal_scale;
            let ny = side_normal_y;
            let nz = sin_theta * side_normal_scale;
            normals.push([nx, ny, nz]);

            uvs.push([u, v]);
        }
    }

    let verts_per_ring = radial_segments + 1;
    for ring in 0..rings {
        for segment in 0..radial_segments {
            let top_left = ring * verts_per_ring + segment;
            let top_right = ring * verts_per_ring + segment + 1;
            let bottom_left = (ring + 1) * verts_per_ring + segment;
            let bottom_right = (ring + 1) * verts_per_ring + segment + 1;

            indices.push(top_left);
            indices.push(top_right);
            indices.push(bottom_left);

            indices.push(top_right);
            indices.push(bottom_right);
            indices.push(bottom_left);
        }
    }

    generate_cap(
        cap_top,
        top_radius,
        half_height,
        1.0,
        radial_segments,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
    );

    generate_cap(
        cap_bottom,
        bottom_radius,
        -half_height,
        -1.0,
        radial_segments,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn generate_cap(
    enabled: bool,
    radius: f32,
    y: f32,
    normal_y: f32,
    radial_segments: u32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    if !enabled || radius <= 0.0 {
        return;
    }

    let center_index = positions.len() as u32;

    positions.push([0.0, y, 0.0]);
    normals.push([0.0, normal_y, 0.0]);
    uvs.push([0.5, 0.5]);

    for segment in 0..=radial_segments {
        let u = segment as f32 / radial_segments as f32;
        let theta = u * std::f32::consts::TAU;
        let (sin_theta, cos_theta) = theta.sin_cos();

        positions.push([cos_theta * radius, y, sin_theta * radius]);
        normals.push([0.0, normal_y, 0.0]);
        uvs.push([cos_theta * 0.5 + 0.5, sin_theta * 0.5 + 0.5]);
    }

    for segment in 0..radial_segments {
        let first = center_index + 1 + segment;
        let second = center_index + 1 + segment + 1;
        if normal_y > 0.0 {
            indices.push(center_index);
            indices.push(second);
            indices.push(first);
        } else {
            indices.push(center_index);
            indices.push(first);
            indices.push(second);
        }
    }
}

fn create_prism_mesh(left_to_right: f32, size: Vec3, subdivide: Vec3) -> Mesh {
    let start_pos = size * -0.5;
    let subdivide_w = subdivide.x as usize;
    let subdivide_h = subdivide.y as usize;
    let subdivide_d = subdivide.z as usize;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    {
        let mut y = start_pos.y;
        let mut thisrow: u32 = 0;
        let mut prevrow: u32 = 0;

        for j in 0..=(subdivide_h + 1) {
            let scale = j as f32 / (subdivide_h + 1) as f32;
            let scaled_size_x = size.x * scale;
            let start_x = start_pos.x + (1.0 - scale) * size.x * left_to_right;

            let v = j as f32 / (2.0 * (subdivide_h + 1) as f32);

            let mut x = 0.0_f32;
            for i in 0..=(subdivide_w + 1) {
                let u = (i as f32 / (subdivide_w + 1) as f32) * scale;

                positions.push([start_x + x, -y, -start_pos.z]);
                normals.push([0.0, 0.0, 1.0]);
                uvs.push([u, v]);

                positions.push([start_x + scaled_size_x - x, -y, start_pos.z]);
                normals.push([0.0, 0.0, -1.0]);
                uvs.push([u, v]);

                if i > 0 && j == 1 {
                    let i2 = (i * 2) as u32;

                    indices.extend_from_slice(&[thisrow + i2 - 2, thisrow + i2, prevrow + i2]);
                    indices.extend_from_slice(&[
                        thisrow + i2 - 1,
                        thisrow + i2 + 1,
                        prevrow + i2 + 1,
                    ]);
                } else if i > 0 && j > 0 {
                    let i2 = (i * 2) as u32;

                    indices.extend_from_slice(&[
                        thisrow + i2 - 2,
                        prevrow + i2,
                        prevrow + i2 - 2,
                        thisrow + i2 - 2,
                        thisrow + i2,
                        prevrow + i2,
                    ]);
                    indices.extend_from_slice(&[
                        thisrow + i2 - 1,
                        prevrow + i2 + 1,
                        prevrow + i2 - 1,
                        thisrow + i2 - 1,
                        thisrow + i2 + 1,
                        prevrow + i2 + 1,
                    ]);
                }

                x += scale * size.x / (subdivide_w + 1) as f32;
            }

            y += size.y / (subdivide_h + 1) as f32;
            prevrow = thisrow;
            thisrow = positions.len() as u32;
        }
    }

    {
        let normal_left = Vec3::new(-size.y, size.x * left_to_right, 0.0).normalize_or_zero();
        let normal_right =
            Vec3::new(size.y, size.x * (1.0 - left_to_right), 0.0).normalize_or_zero();

        let mut y = start_pos.y;
        let mut thisrow = positions.len() as u32;
        let mut prevrow: u32 = 0;

        for j in 0..=(subdivide_h + 1) {
            let scale = j as f32 / (subdivide_h + 1) as f32;
            let left = start_pos.x + (size.x * (1.0 - scale) * left_to_right);
            let right = left + (size.x * scale);

            let v = j as f32 / (2.0 * (subdivide_h + 1) as f32);

            let mut z = start_pos.z;
            for i in 0..=(subdivide_d + 1) {
                let u = i as f32 / (subdivide_d + 1) as f32;

                positions.push([right, -y, -z]);
                normals.push(normal_right.to_array());
                uvs.push([u, v]);

                positions.push([left, -y, z]);
                normals.push(normal_left.to_array());
                uvs.push([u, v]);

                if i > 0 && j > 0 {
                    let i2 = (i * 2) as u32;

                    indices.extend_from_slice(&[
                        thisrow + i2 - 2,
                        prevrow + i2,
                        prevrow + i2 - 2,
                        thisrow + i2 - 2,
                        thisrow + i2,
                        prevrow + i2,
                    ]);
                    indices.extend_from_slice(&[
                        thisrow + i2 - 1,
                        prevrow + i2 + 1,
                        prevrow + i2 - 1,
                        thisrow + i2 - 1,
                        thisrow + i2 + 1,
                        prevrow + i2 + 1,
                    ]);
                }

                z += size.z / (subdivide_d + 1) as f32;
            }

            y += size.y / (subdivide_h + 1) as f32;
            prevrow = thisrow;
            thisrow = positions.len() as u32;
        }
    }

    {
        let mut z = start_pos.z;
        let mut thisrow = positions.len() as u32;
        let mut prevrow: u32 = 0;

        for j in 0..=(subdivide_d + 1) {
            let v = j as f32 / (2.0 * (subdivide_d + 1) as f32);

            let mut x = start_pos.x;
            for i in 0..=(subdivide_w + 1) {
                let u = i as f32 / (subdivide_w + 1) as f32;

                positions.push([x, start_pos.y, -z]);
                normals.push([0.0, -1.0, 0.0]);
                uvs.push([u, v]);

                if i > 0 && j > 0 {
                    let curr = thisrow + i as u32;
                    let prev_curr = prevrow + i as u32;

                    indices.extend_from_slice(&[
                        curr - 1,
                        prev_curr,
                        prev_curr - 1,
                        curr - 1,
                        curr,
                        prev_curr,
                    ]);
                }

                x += size.x / (subdivide_w + 1) as f32;
            }

            z += size.z / (subdivide_d + 1) as f32;
            prevrow = thisrow;
            thisrow = positions.len() as u32;
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Validate the arithmetic used by quad and sphere construction and replication.
/// This is not a limit on memory or validation of other mesh types.
pub(crate) fn checked_mesh_arithmetic(config: &ParticleMesh, amount: u32) -> bool {
    let checked = || -> Option<()> {
        let (vertices, indices) = match config {
            ParticleMesh::Quad { subdivide, .. } => {
                let cols = (subdivide.x as u32).checked_add(1)?;
                let rows = (subdivide.y as u32).checked_add(1)?;
                (
                    cols.checked_add(1)?.checked_mul(rows.checked_add(1)?)?,
                    usize::try_from(cols.checked_mul(rows)?.checked_mul(6)?).ok()?,
                )
            }
            ParticleMesh::Sphere {
                segments, rings, ..
            } => {
                let sectors = (*segments).max(3);
                let stacks = (*rings).max(2);
                // Bevy's UV sphere builder computes this capacity in u32 first.
                let capacity = usize::try_from(sectors.checked_mul(stacks)?).ok()?;
                capacity.checked_mul(6)?;
                let vertices = sectors
                    .checked_add(1)?
                    .checked_mul(stacks.checked_add(1)?)?;
                let indices = usize::try_from(sectors)
                    .ok()?
                    .checked_mul(usize::try_from(stacks - 1).ok()?)?
                    .checked_mul(6)?;
                (vertices, indices)
            }
            _ => return Some(()),
        };
        vertices.checked_mul(amount)?;
        indices.checked_mul(usize::try_from(amount).ok()?)?;
        Some(())
    };
    checked().is_some()
}

fn create_subdivided_quad(size: Vec2, subdivisions_x: u32, subdivisions_y: u32) -> Mesh {
    let cols = subdivisions_x + 1;
    let rows = subdivisions_y + 1;
    let vertex_count = ((cols + 1) * (rows + 1)) as usize;
    let index_count = (cols * rows * 6) as usize;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(index_count);

    let half_w = size.x * 0.5;
    let half_d = size.y * 0.5;

    for iy in 0..=rows {
        let ty = iy as f32 / rows as f32;
        let y = -half_d + ty * size.y;
        for ix in 0..=cols {
            let tx = ix as f32 / cols as f32;
            let x = -half_w + tx * size.x;
            positions.push([x, y, 0.0]);
            normals.push([0.0, 0.0, 1.0]);
            uvs.push([tx, 1.0 - ty]);
        }
    }

    let stride = cols + 1;
    for iy in 0..rows {
        for ix in 0..cols {
            let i00 = iy * stride + ix;
            let i10 = i00 + 1;
            let i01 = i00 + stride;
            let i11 = i01 + 1;
            indices.push(i00);
            indices.push(i10);
            indices.push(i11);
            indices.push(i00);
            indices.push(i11);
            indices.push(i01);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn create_tube_trail_mesh(
    radius: f32,
    radial_steps: u32,
    sections: u32,
    section_rings: u32,
) -> Mesh {
    let radial_steps = radial_steps.max(3);
    let sections = sections.max(2);
    let total_rings = sections * section_rings.max(1);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut uv_bs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for ring in 0..=total_rings {
        let section_frac = ring as f32 / total_rings as f32;
        for segment in 0..=radial_steps {
            let u = segment as f32 / radial_steps as f32;
            let theta = u * std::f32::consts::TAU;
            let (sin_theta, cos_theta) = theta.sin_cos();

            positions.push([cos_theta * radius, -section_frac, sin_theta * radius]);
            normals.push([cos_theta, 0.0, sin_theta]);
            uvs.push([u, section_frac]);
            uv_bs.push([0.0, section_frac]);
        }
    }

    let verts_per_ring = radial_steps + 1;
    for ring in 0..total_rings {
        for segment in 0..radial_steps {
            let top_left = ring * verts_per_ring + segment;
            let top_right = ring * verts_per_ring + segment + 1;
            let bottom_left = (ring + 1) * verts_per_ring + segment;
            let bottom_right = (ring + 1) * verts_per_ring + segment + 1;

            indices.push(top_left);
            indices.push(top_right);
            indices.push(bottom_left);

            indices.push(top_right);
            indices.push(bottom_right);
            indices.push(bottom_left);
        }
    }

    let cap_center = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    uv_bs.push([0.0, 0.0]);

    for segment in 0..=radial_steps {
        let u = segment as f32 / radial_steps as f32;
        let theta = u * std::f32::consts::TAU;
        let (sin_theta, cos_theta) = theta.sin_cos();

        positions.push([cos_theta * radius, 0.0, sin_theta * radius]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([cos_theta * 0.5 + 0.5, sin_theta * 0.5 + 0.5]);
        uv_bs.push([0.0, 0.0]);
    }

    for segment in 0..radial_steps {
        let first = cap_center + 1 + segment;
        let second = cap_center + 2 + segment;
        indices.push(cap_center);
        indices.push(first);
        indices.push(second);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_bs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn create_ribbon_trail_mesh(
    size: f32,
    sections: u32,
    section_rings: u32,
    shape: RibbonTrailShape,
) -> Mesh {
    let sections = sections.max(2);
    let total_subdivs = sections * section_rings.max(1);

    let strip_offsets: Vec<[f32; 2]> = match shape {
        RibbonTrailShape::Flat => vec![[1.0, 0.0]],
        RibbonTrailShape::Cross => vec![[1.0, 0.0], [0.0, 1.0]],
    };

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut uv_bs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for offsets in &strip_offsets {
        let base_vertex = positions.len() as u32;

        let normal = if offsets[0] != 0.0 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };

        for row in 0..=total_subdivs {
            let section_frac = row as f32 / total_subdivs as f32;

            positions.push([-size * offsets[0], -section_frac, -size * offsets[1]]);
            normals.push(normal);
            uvs.push([0.0, section_frac]);
            uv_bs.push([0.0, section_frac]);

            positions.push([size * offsets[0], -section_frac, size * offsets[1]]);
            normals.push(normal);
            uvs.push([1.0, section_frac]);
            uv_bs.push([0.0, section_frac]);
        }

        for row in 0..total_subdivs {
            let left = base_vertex + row * 2;
            let right = left + 1;
            let next_left = left + 2;
            let next_right = left + 3;

            indices.push(left);
            indices.push(right);
            indices.push(next_left);

            indices.push(right);
            indices.push(next_right);
            indices.push(next_left);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_bs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn create_base_mesh(config: &ParticleMesh) -> Mesh {
    match config {
        ParticleMesh::Quad {
            orientation,
            size,
            subdivide,
        } => {
            let subdivisions_x = subdivide.x as u32;
            let subdivisions_y = subdivide.y as u32;
            let mut mesh = create_subdivided_quad(*size, subdivisions_x, subdivisions_y);

            let rotation = match orientation {
                QuadOrientation::FaceZ => None,
                QuadOrientation::FaceX => Some(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                QuadOrientation::FaceY => Some(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            };

            if let Some(rot) = rotation {
                if let Some(VertexAttributeValues::Float32x3(positions)) =
                    mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                {
                    for pos in positions.iter_mut() {
                        let v = rot * Vec3::from_array(*pos);
                        *pos = v.to_array();
                    }
                }
                if let Some(VertexAttributeValues::Float32x3(normals)) =
                    mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
                {
                    for normal in normals.iter_mut() {
                        let v = rot * Vec3::from_array(*normal);
                        *normal = v.to_array();
                    }
                }
            }

            mesh
        }
        ParticleMesh::Sphere {
            radius,
            segments,
            rings,
        } => {
            // Building with a zero radius divides by zero in Bevy's UV-sphere
            // normal generation. Scale positions instead; normals stay defined.
            let mut mesh = Sphere::new(1.0)
                .mesh()
                .uv((*segments).max(3), (*rings).max(2));
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                for position in positions {
                    *position = (Vec3::from_array(*position) * *radius).to_array();
                }
            }
            mesh
        }
        ParticleMesh::Cuboid { half_size } => Mesh::from(Cuboid::new(
            half_size.x * 2.0,
            half_size.y * 2.0,
            half_size.z * 2.0,
        )),
        ParticleMesh::Cylinder {
            top_radius,
            bottom_radius,
            height,
            radial_segments,
            rings,
            cap_top,
            cap_bottom,
        } => create_cylinder_mesh(
            *top_radius,
            *bottom_radius,
            *height,
            *radial_segments,
            *rings,
            *cap_top,
            *cap_bottom,
        ),
        ParticleMesh::Prism {
            left_to_right,
            size,
            subdivide,
        } => create_prism_mesh(*left_to_right, *size, *subdivide),
        ParticleMesh::TubeTrail {
            radius,
            radial_steps,
            sections,
            section_rings,
        } => create_tube_trail_mesh(*radius, *radial_steps, *sections, *section_rings),
        ParticleMesh::RibbonTrail {
            size,
            sections,
            section_rings,
            shape,
        } => create_ribbon_trail_mesh(*size, *sections, *section_rings, *shape),
    }
}

fn build_particle_mesh(
    config: &ParticleMesh,
    particle_count: u32,
    meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
    let mut base_mesh = create_base_mesh(config);
    let _ = base_mesh.generate_tangents();
    let trail = config.is_trail();

    let base_positions: Vec<[f32; 3]> =
        extract_float32x3(&base_mesh, Mesh::ATTRIBUTE_POSITION).unwrap_or_default();

    let base_normals: Vec<[f32; 3]> = extract_float32x3(&base_mesh, Mesh::ATTRIBUTE_NORMAL)
        .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; base_positions.len()]);

    let base_tangents: Option<Vec<[f32; 4]>> = base_mesh
        .attribute(Mesh::ATTRIBUTE_TANGENT)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x4(v) => Some(v.clone()),
            _ => None,
        });

    let base_uvs: Vec<[f32; 2]> = base_mesh
        .attribute(Mesh::ATTRIBUTE_UV_0)
        .and_then(|attr| match attr {
            VertexAttributeValues::Float32x2(v) => Some(v.clone()),
            _ => None,
        })
        .unwrap_or_else(|| vec![[0.0, 0.0]; base_positions.len()]);

    let base_uv_bs: Option<Vec<[f32; 2]>> = if trail {
        base_mesh
            .attribute(Mesh::ATTRIBUTE_UV_1)
            .and_then(|attr| match attr {
                VertexAttributeValues::Float32x2(v) => Some(v.clone()),
                _ => None,
            })
    } else {
        None
    };

    let base_indices: Vec<u32> = base_mesh
        .indices()
        .map(|indices| indices.iter().map(|i| i as u32).collect())
        .unwrap_or_else(|| (0..base_positions.len() as u32).collect());

    let vertices_per_mesh = base_positions.len();
    let indices_per_mesh = base_indices.len();

    let total_vertices = particle_count as usize * vertices_per_mesh;
    let total_indices = particle_count as usize * indices_per_mesh;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut tangents: Vec<[f32; 4]> = if base_tangents.is_some() {
        Vec::with_capacity(total_vertices)
    } else {
        Vec::new()
    };
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total_vertices);
    let mut uv_bs: Vec<[f32; 2]> = Vec::with_capacity(total_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity(total_indices);

    for particle_idx in 0..particle_count {
        let base_vertex = (particle_idx as usize * vertices_per_mesh) as u32;
        let particle_index_f32 = particle_idx as f32;

        for i in 0..vertices_per_mesh {
            positions.push(base_positions[i]);
            normals.push(base_normals[i]);
            if let Some(ref bt) = base_tangents {
                tangents.push(bt[i]);
            }
            uvs.push(base_uvs[i]);
            let trail_v = base_uv_bs.as_ref().map_or(0.0, |b| b[i][1]);
            uv_bs.push([particle_index_f32, trail_v]);
        }

        for &idx in &base_indices {
            indices.push(base_vertex + idx);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    if !tangents.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_bs);
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh)
}

fn extract_float32x3(
    mesh: &Mesh,
    attribute: impl Into<MeshVertexAttributeId>,
) -> Option<Vec<[f32; 3]>> {
    mesh.attribute(attribute).and_then(|attr| match attr {
        VertexAttributeValues::Float32x3(v) => Some(v.clone()),
        _ => None,
    })
}

#[cfg(test)]
mod theia_cache_tests {
    use super::*;

    #[test]
    fn zero_radius_and_resolution_spheres_have_finite_mesh_attributes() {
        for (radius, segments, rings) in [(0.0, 0, 0), (f32::MIN_POSITIVE, 3, 2), (2.0, 8, 4)] {
            let mesh = create_base_mesh(&ParticleMesh::Sphere {
                radius,
                segments,
                rings,
            });
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("missing positions")
            };
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("missing normals")
            };
            assert!(
                positions
                    .iter()
                    .flatten()
                    .chain(normals.iter().flatten())
                    .all(|value| value.is_finite())
            );
            assert!(
                normals
                    .iter()
                    .all(|normal| (Vec3::from_array(*normal).length() - 1.0).abs() < 1e-5)
            );
            assert!(
                positions
                    .iter()
                    .all(|position| (Vec3::from_array(*position).length() - radius).abs() < 1e-5)
            );
            let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
            assert_eq!(indices.len() % 3, 0);
            assert!(indices.iter().all(|index| *index < positions.len()));
        }
    }

    #[test]
    fn flat_and_nearly_flat_cylinders_keep_mesh_attributes_finite() {
        for (height, top, bottom) in [
            (0.0, 1.0, 1.0),
            (0.0, 0.5, 1.0),
            (f32::MIN_POSITIVE, 0.5, 1.0),
            (2.0, 0.5, 1.0),
        ] {
            let mesh = create_cylinder_mesh(top, bottom, height, 8, 1, true, true);
            for attribute in [Mesh::ATTRIBUTE_POSITION, Mesh::ATTRIBUTE_NORMAL] {
                let Some(VertexAttributeValues::Float32x3(values)) = mesh.attribute(attribute)
                else {
                    panic!("missing mesh attribute")
                };
                assert!(
                    values.iter().flatten().all(|v| v.is_finite()),
                    "nonfinite attribute for height {height}"
                );
            }
            if height == 2.0 {
                let Some(VertexAttributeValues::Float32x3(normals)) =
                    mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
                else {
                    unreachable!()
                };
                let expected = Vec3::new(1.0, 0.25, 0.0).normalize();
                assert!(Vec3::from_array(normals[0]).distance(expected) < 1e-6);
            }
        }
    }

    #[test]
    fn degenerate_prism_normals_and_subdivided_topology_remain_valid() {
        for size in [Vec3::new(0.0, 0.0, 1.0), Vec3::ONE] {
            let mesh = create_prism_mesh(0.5, size, Vec3::new(2.0, 2.0, 1.0));
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("missing positions")
            };
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("missing normals")
            };
            assert!(
                positions
                    .iter()
                    .flatten()
                    .chain(normals.iter().flatten())
                    .all(|v| v.is_finite())
            );
            let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
            assert_eq!(indices.len() % 3, 0);
            assert!(indices.iter().all(|index| *index < positions.len()));
            // Front/back vertices are interleaved, and each triangle stays on one face.
            let first_face_indices = 6 * 3 + 12 * 3 * 2;
            for triangle in indices[..first_face_indices].as_chunks::<3>().0 {
                assert!(triangle.iter().all(|index| index % 2 == triangle[0] % 2));
            }
        }
    }

    #[test]
    fn mesh_edits_share_live_meshes_and_release_previous_counts() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_resource::<ParticleMeshCache>()
            .add_systems(PostUpdate, prune_particle_mesh_cache);
        let owner = app.world_mut().spawn_empty().id();
        let peer = app.world_mut().spawn_empty().id();
        for count in 1..128 {
            let handle =
                app.world_mut()
                    .resource_scope(|world, mut cache: Mut<ParticleMeshCache>| {
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let a = cache.get_or_create(&ParticleMesh::default(), count, &mut meshes);
                        let b = cache.get_or_create(&ParticleMesh::default(), count, &mut meshes);
                        assert_eq!(a.id(), b.id());
                        a
                    });
            app.world_mut()
                .entity_mut(owner)
                .insert(Mesh3d(handle.clone()));
            app.world_mut().entity_mut(peer).insert(Mesh3d(handle));
            for _ in 0..4 {
                app.update();
            }
            assert_eq!(app.world().resource::<ParticleMeshCache>().cache.len(), 1);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
        }
        app.world_mut().despawn(owner);
        app.update();
        assert_eq!(app.world().resource::<ParticleMeshCache>().cache.len(), 1);
        app.world_mut().despawn(peer);
        for _ in 0..4 {
            app.update();
        }
        assert!(app.world().resource::<ParticleMeshCache>().cache.is_empty());
        assert!(app.world().resource::<Assets<Mesh>>().is_empty());
    }
}
