//! Bounded schema-4 static collision data. Standard Avian shapes only; actor
//! bodies, gameplay hit detection and navigation never enter this representation.
use crate::{
    Result,
    physics::{self, PlacedCollider},
};
use avian3d::prelude::{Collider, SimpleCollider};
use bevy::prelude::{Mat4, Quat, Vec3};
use serde::Deserialize;

pub(crate) const MAX_INPUT_TRIANGLES: usize = 131_072;
const MAX_POOL_VERTICES: usize = 262_144;
const MAX_INSTANCE_TRIANGLES: usize = 262_144;
const MAX_INSTANCE_VERTICES: usize = 1_048_576;
const MAX_HEIGHT_SAMPLES: usize = 263_169;
const MAX_BODIES: usize = 4096;
const FRAME_LIMIT: f32 = 1023.;
// Keep a millimetre for conservative cross-language contributor bounds.
const GEOMETRY_LIMIT: f32 = FRAME_LIMIT - 0.001;
fn queries() -> u32 {
    physics::GROUND | physics::SOLID | physics::CAMERA
}
fn valid_queries(mask: u32) -> bool {
    mask != 0 && mask & !queries() == 0
}
fn bounded(values: &[f32], limit: f32) -> bool {
    values.iter().all(|v| v.is_finite() && v.abs() <= limit)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mesh {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Object {
    Mesh {
        mesh: usize,
        affine: [f32; 16],
        convex: bool,
        #[serde(default = "queries")]
        queries: u32,
    },
    Box {
        center: [f32; 3],
        size: [f32; 3],
        rotation: [f32; 4],
        #[serde(default = "queries")]
        queries: u32,
    },
    Capsule {
        center: [f32; 3],
        radius: f32,
        half_segment: [f32; 3],
        #[serde(default = "queries")]
        queries: u32,
    },
}
impl Object {
    fn queries(&self) -> u32 {
        match self {
            Self::Mesh { queries, .. }
            | Self::Box { queries, .. }
            | Self::Capsule { queries, .. } => *queries,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Heightfield {
    rows: usize,
    columns: usize,
    heights_column_major: Vec<f32>,
    center: [f32; 3],
    scale: [f32; 3],
    #[serde(default = "queries")]
    queries: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShapeSet {
    #[serde(default)]
    meshes: Vec<Mesh>,
    #[serde(default)]
    objects: Vec<Object>,
    #[serde(default)]
    heightfields: Vec<Heightfield>,
}
pub(crate) struct Compiled {
    pub colliders: Vec<PlacedCollider>,
    pub bounds: Vec<[[f32; 3]; 2]>,
    pub world_min_y: f32,
    pub world_max_y: f32,
}

fn affine(values: &[f32; 16]) -> Result<Mat4> {
    if !bounded(values, 4096.) || values[12..] != [0., 0., 0., 1.] {
        return Err("Invalid bounded static affine transform".into());
    }
    let matrix = Mat4::from_cols_array_2d(&std::array::from_fn(|c| {
        std::array::from_fn(|r| values[r * 4 + c])
    }));
    if matrix.determinant().abs() <= 1e-12 {
        return Err("Singular static mesh transform".into());
    }
    Ok(matrix)
}

impl ShapeSet {
    pub fn has_world_geometry(&self) -> bool {
        self.objects
            .iter()
            .any(|o| o.queries() & (physics::SOLID | physics::GROUND) != 0)
            || self
                .heightfields
                .iter()
                .any(|h| h.queries & (physics::SOLID | physics::GROUND) != 0)
    }
    pub fn compile(&self, legacy_triangles: usize, legacy_bodies: usize) -> Result<Compiled> {
        if legacy_triangles > MAX_INPUT_TRIANGLES || self.meshes.len() > 128 || self.heightfields.len() > 8
            || self.objects.len() + self.heightfields.len() > MAX_BODIES
            // Reserve the four finite-world walls used by standalone adoption.
            || legacy_bodies + self.objects.len() + self.heightfields.len() > 8188
        {
            return Err("Static shape body/pool budget exceeded".into());
        }
        let mut input_triangles = legacy_triangles;
        let mut input_vertices = 0usize;
        for mesh in &self.meshes {
            input_triangles += mesh.indices.len();
            input_vertices += mesh.vertices.len();
            if mesh.vertices.len() < 3
                || mesh.indices.is_empty()
                || input_triangles > MAX_INPUT_TRIANGLES
                || input_vertices > MAX_POOL_VERTICES
                || mesh.vertices.iter().any(|v| !bounded(v, 4096.))
                || mesh
                    .indices
                    .iter()
                    .flatten()
                    .any(|&v| v as usize >= mesh.vertices.len())
            {
                return Err("Invalid or over-budget shared collision mesh".into());
            }
            if !mesh.indices.iter().any(|t| {
                let [a, b, c] = t.map(|i| Vec3::from_array(mesh.vertices[i as usize]));
                (b - a).cross(c - a).length_squared() > 1e-20
            }) {
                return Err("Shared collision mesh has no nondegenerate surface".into());
            }
        }
        let mut used = vec![false; self.meshes.len()];
        let mut triangles = 0usize;
        let mut vertices = 0usize;
        for object in &self.objects {
            if !valid_queries(object.queries()) {
                return Err("Invalid static query layers".into());
            }
            if let Object::Mesh {
                mesh,
                affine: values,
                ..
            } = object
            {
                let geometry = self
                    .meshes
                    .get(*mesh)
                    .ok_or("Unbound shared collision mesh")?;
                affine(values)?;
                used[*mesh] = true;
                triangles += geometry.indices.len();
                vertices += geometry.vertices.len();
                if triangles > MAX_INSTANCE_TRIANGLES || vertices > MAX_INSTANCE_VERTICES {
                    return Err("Instantiated collision geometry exceeds budget".into());
                }
            }
        }
        if used.iter().any(|v| !*v) {
            return Err("Unused shared collision geometry".into());
        }
        let mut samples = 0usize;
        for field in &self.heightfields {
            if !(2..=1025).contains(&field.rows)
                || !(2..=1025).contains(&field.columns)
                || field.heights_column_major.len() != field.rows * field.columns
                || !valid_queries(field.queries)
                || !bounded(&field.center, FRAME_LIMIT)
                || !bounded(&field.scale, FRAME_LIMIT)
                || field.scale.iter().any(|v| *v <= 0.)
                || !bounded(&field.heights_column_major, 4096.)
            {
                return Err("Invalid bounded heightfield dimensions/data".into());
            }
            samples += field.heights_column_major.len();
            if samples > MAX_HEIGHT_SAMPLES {
                return Err("Heightfield sample budget exceeded".into());
            }
            if field
                .heights_column_major
                .iter()
                .any(|h| (h * field.scale[1] + field.center[1]).abs() > FRAME_LIMIT)
                || [0, 2]
                    .iter()
                    .any(|&i| field.center[i].abs() + field.scale[i] / 2. > FRAME_LIMIT)
            {
                return Err("Heightfield exceeds local physics frame".into());
            }
        }
        let mut colliders = Vec::with_capacity(self.objects.len() + self.heightfields.len());
        for object in &self.objects {
            let (shape, center) = match object {
                Object::Mesh {
                    mesh,
                    affine: values,
                    convex,
                    ..
                } => {
                    let matrix = affine(values)?;
                    let geometry = &self.meshes[*mesh];
                    let center = matrix.w_axis.truncate();
                    let points = geometry
                        .vertices
                        .iter()
                        .map(|v| matrix.transform_vector3(Vec3::from_array(*v)))
                        .collect::<Vec<_>>();
                    if !bounded(&center.to_array(), FRAME_LIMIT)
                        || points
                            .iter()
                            .any(|v| !bounded(&(*v + center).to_array(), FRAME_LIMIT))
                    {
                        return Err("Static mesh instance exceeds local physics frame".into());
                    }
                    let shape = if *convex {
                        Collider::convex_hull(&points)
                            .ok_or("Static convex hull construction failed")?
                    } else {
                        Collider::try_trimesh(points, geometry.indices.clone())
                            .map_err(|e| format!("Static mesh construction failed: {e:?}"))?
                    };
                    (shape, center)
                }
                Object::Box {
                    center,
                    size,
                    rotation,
                    ..
                } => {
                    let q = Quat::from_array(*rotation);
                    if !bounded(center, FRAME_LIMIT)
                        || !bounded(size, 1024.)
                        || size.iter().any(|v| *v <= 0.)
                        || !q.is_finite()
                        || (q.length_squared() - 1.).abs() > 0.0001
                    {
                        return Err("Invalid static oriented box".into());
                    }
                    let shape = Collider::compound(vec![(
                        Vec3::ZERO,
                        q.normalize(),
                        Collider::cuboid(size[0], size[1], size[2]),
                    )]);
                    (shape, Vec3::from_array(*center))
                }
                Object::Capsule {
                    center,
                    radius,
                    half_segment,
                    ..
                } => {
                    if !bounded(center, FRAME_LIMIT)
                        || !bounded(half_segment, FRAME_LIMIT)
                        || !radius.is_finite()
                        || *radius <= 0.
                        || *radius > FRAME_LIMIT
                    {
                        return Err("Invalid static capsule".into());
                    }
                    let half = Vec3::from_array(*half_segment);
                    (
                        Collider::capsule_endpoints(*radius, -half, half),
                        Vec3::from_array(*center),
                    )
                }
            };
            colliders.push(PlacedCollider {
                shape,
                center,
                layer: object.queries(),
            });
        }
        for field in &self.heightfields {
            let heights = field
                .heights_column_major
                .chunks_exact(field.columns)
                .map(<[f32]>::to_vec)
                .collect();
            colliders.push(PlacedCollider {
                shape: Collider::heightfield(heights, Vec3::from_array(field.scale)),
                center: Vec3::from_array(field.center),
                layer: field.queries,
            });
        }
        let mut bounds = Vec::with_capacity(colliders.len());
        let mut world_min_y = f32::INFINITY;
        let mut world_max_y = f32::NEG_INFINITY;
        for collider in &colliders {
            let aabb = collider.shape.aabb(collider.center, Quat::IDENTITY, 0.);
            if !bounded(&aabb.min.to_array(), GEOMETRY_LIMIT)
                || !bounded(&aabb.max.to_array(), GEOMETRY_LIMIT)
            {
                return Err("Cooked static shape exceeds local physics frame".into());
            }
            if collider.layer & (physics::SOLID | physics::GROUND) != 0 {
                world_min_y = world_min_y.min(aabb.min.y);
                world_max_y = world_max_y.max(aabb.max.y);
            }
            bounds.push([aabb.min.to_array(), aabb.max.to_array()]);
        }
        Ok(Compiled {
            colliders,
            bounds,
            world_min_y,
            world_max_y,
        })
    }
}
