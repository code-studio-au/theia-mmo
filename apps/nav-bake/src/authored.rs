//! Offline navigation input from the same validated static shapes used by Avian.
//! Clip exact polygon surfaces to one tile plus padding; never substitute boxes
//! for meshes or quietly omit an unsupported collision shape.
use avian3d::parry::{
    math::{Pose, Vector},
    shape::{Shape, TypedShape},
};
use std::collections::BTreeMap;
use theia_network::{
    Result,
    physics::{GROUND, PlacedCollider, SOLID},
    procedural::Geometry,
};

const MIN: f32 = -8.;
const MAX: f32 = 136.;
struct Builder {
    geometry: Geometry,
    vertices: BTreeMap<[u32; 3], u32>,
    input_triangles: usize,
}
impl Builder {
    fn triangle(&mut self, triangle: [Vector; 3]) -> Result<()> {
        self.input_triangles += 1;
        if self.input_triangles > 2_000_000 || triangle.iter().any(|p| !p.is_finite()) {
            return Err("Authored navigation input budget or finite-coordinate failure".into());
        }
        let mut polygon = triangle.to_vec();
        for axis in [0, 2] {
            for (bound, lower) in [(MIN, true), (MAX, false)] {
                let old = std::mem::take(&mut polygon);
                if old.is_empty() {
                    return Ok(());
                }
                for i in 0..old.len() {
                    let a = old[i];
                    let b = old[(i + 1) % old.len()];
                    let inside = |p: Vector| {
                        if lower {
                            p[axis] >= bound
                        } else {
                            p[axis] <= bound
                        }
                    };
                    if inside(a) {
                        polygon.push(a);
                    }
                    if inside(a) != inside(b) {
                        let mut point = a + (b - a) * ((bound - a[axis]) / (b[axis] - a[axis]));
                        point[axis] = bound;
                        polygon.push(point);
                    }
                }
            }
        }
        for i in 1..polygon.len().saturating_sub(1) {
            let points = [polygon[0], polygon[i], polygon[i + 1]];
            if (points[1] - points[0])
                .cross(points[2] - points[0])
                .length_squared()
                <= 1e-12
            {
                continue;
            }
            let mut indices = [0; 3];
            for (i, p) in points.into_iter().enumerate() {
                let point = p.to_array().map(|v| if v == 0. { 0. } else { v });
                let key = point.map(f32::to_bits);
                indices[i] = *self.vertices.entry(key).or_insert_with(|| {
                    let id = self.geometry.positions.len() as u32;
                    self.geometry.positions.push(point);
                    id
                });
            }
            self.geometry.triangles.push(indices);
            if self.geometry.positions.len() > 16_384 || self.geometry.triangles.len() > 32_768 {
                return Err("Authored tile exceeds the existing navigation geometry budget".into());
            }
        }
        Ok(())
    }
    fn shape(&mut self, shape: &dyn Shape, pose: Pose, depth: usize) -> Result<()> {
        if depth > 8 {
            return Err("Authored navigation compound nesting exceeds budget".into());
        }
        let (vertices, indices) = match shape.as_typed_shape() {
            TypedShape::TriMesh(mesh) => {
                for t in mesh.indices() {
                    self.triangle(t.map(|i| pose.transform_point(mesh.vertices()[i as usize])))?;
                }
                return Ok(());
            }
            TypedShape::Cuboid(shape) => shape.to_trimesh(),
            TypedShape::ConvexPolyhedron(shape) => shape.to_trimesh(),
            TypedShape::HeightField(shape) => shape.to_trimesh(),
            TypedShape::Compound(compound) => {
                for (local, child) in compound.shapes() { self.shape(child.as_ref(), pose * *local, depth+1)?; }
                return Ok(());
            }
            _ => return Err("Authored navigation currently requires triangle meshes, boxes, hulls or heightfields".into()),
        };
        for t in indices {
            self.triangle(t.map(|i| pose.transform_point(vertices[i as usize])))?;
        }
        Ok(())
    }
}
pub fn geometry(colliders: &[PlacedCollider]) -> Result<Geometry> {
    let mut builder = Builder {
        geometry: Geometry {
            positions: vec![],
            triangles: vec![],
        },
        vertices: BTreeMap::new(),
        input_triangles: 0,
    };
    for collider in colliders.iter().filter(|c| c.layer & (SOLID | GROUND) != 0) {
        let [x, y, z] = collider.center.to_array();
        builder.shape(
            collider.shape.shape_scaled().as_ref(),
            Pose::translation(x, y, z),
            0,
        )?;
    }
    if builder.geometry.triangles.is_empty() {
        return Err("Authored navigation tile has no supported surfaces".into());
    }
    Ok(builder.geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use avian3d::prelude::Collider;
    #[test]
    fn clipping_preserves_mesh_opening_and_excludes_camera_only_geometry() {
        let shape = Collider::try_trimesh(
            vec![
                [0., 0., 0.].into(),
                [4., 0., 0.].into(),
                [0., 0., 4.].into(),
            ],
            vec![[0, 2, 1]],
        )
        .unwrap();
        let g = geometry(&[
            PlacedCollider {
                shape,
                center: [8., 2., 12.].into(),
                layer: SOLID | GROUND,
            },
            PlacedCollider {
                shape: Collider::cuboid(100., 100., 100.),
                center: [0.; 3].into(),
                layer: theia_network::physics::CAMERA,
            },
        ])
        .unwrap();
        assert_eq!(g.triangles.len(), 1);
        let points = g.triangles[0].map(|i| g.positions[i as usize]);
        assert_eq!(points, [[8., 2., 12.], [8., 2., 16.], [12., 2., 12.]]);
    }
    #[test]
    fn large_static_floor_is_clipped_and_unsupported_solids_refuse_the_bake() {
        let g = geometry(&[PlacedCollider {
            shape: Collider::cuboid(1024., 2., 1024.),
            center: [0., -1., 0.].into(),
            layer: SOLID | GROUND,
        }])
        .unwrap();
        assert!(
            g.positions
                .iter()
                .all(|p| (MIN..=MAX).contains(&p[0]) && (MIN..=MAX).contains(&p[2]))
        );
        assert!(g.positions.iter().any(|p| p[1] == 0.));
        assert!(
            geometry(&[PlacedCollider {
                shape: Collider::capsule(1., 2.),
                center: [0.; 3].into(),
                layer: SOLID
            }])
            .is_err()
        );
    }
}
