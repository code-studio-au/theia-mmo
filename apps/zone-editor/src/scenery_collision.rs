//! Explicit static source surfaces, separate from the visual GLB snapshot.
use super::*;
use serde::Deserialize;
use std::{path::Path, sync::Arc};

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Binding {
    Decorative,
    TriangleMesh {
        file: String,
        bytes: usize,
        sha256: String,
    },
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
}
impl Mesh {
    fn validate(&self, bounds: [[f64; 3]; 2]) -> Result<(), String> {
        if !(3..=262_144).contains(&self.vertices.len())
            || !(1..=65_536).contains(&self.indices.len())
            || self.vertices.iter().any(|p| {
                (0..3).any(|i| {
                    !p[i].is_finite()
                        || f64::from(p[i]) < bounds[0][i] - 0.00001
                        || f64::from(p[i]) > bounds[1][i] + 0.00001
                })
            })
            || self
                .indices
                .iter()
                .flatten()
                .any(|&i| i as usize >= self.vertices.len())
        {
            return Err("Invalid or over-budget scenery collision mesh".into());
        }
        if self.indices.iter().any(|indices| {
            let [a, b, c] = indices.map(|i| Vec3::from_array(self.vertices[i as usize]));
            (b - a).cross(c - a).length_squared() <= 1e-20
        }) {
            return Err("Degenerate scenery collision triangle".into());
        }
        Ok(())
    }
}
pub(super) fn load(
    root: &Path,
    entry: &scenery_library::Entry,
    total: &mut usize,
) -> Result<Option<Arc<Mesh>>, String> {
    let Some(Binding::TriangleMesh {
        file,
        bytes,
        sha256,
    }) = &entry.collision
    else {
        return Ok(None);
    };
    if !(1..=32 * 1024 * 1024).contains(bytes)
        || sha256.len() != 64
        || !sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || file != &format!("{sha256}.collision.json")
    {
        return Err("Invalid scenery collision binding".into());
    }
    *total = total
        .checked_add(*bytes)
        .ok_or("Scenery byte budget exceeded")?;
    if *total > 128 * 1024 * 1024 {
        return Err("Scenery byte budget exceeded".into());
    }
    let path = root.join(file).canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(root) {
        return Err("Scenery collision escapes library".into());
    }
    let data = scenery_library::bounded(&path, *bytes)?;
    let hash: String = theia_network::terrain::sha256(&data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if data.len() != *bytes || &hash != sha256 {
        return Err("Scenery collision bytes do not match catalog".into());
    }
    let mesh: Mesh = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    mesh.validate(entry.bounds)?;
    Ok(Some(Arc::new(mesh)))
}

pub(super) fn colliders(
    zone: &Zone,
    library: &scenery_library::Library,
) -> Result<Vec<theia_network::physics::PlacedCollider>, String> {
    use avian3d::prelude::Collider;
    use theia_network::physics::{CAMERA, GROUND, PlacedCollider, SOLID};
    zone.validate()?;
    let mut unique = std::collections::BTreeSet::new();
    let (mut pool_vertices, mut pool_triangles, mut vertices, mut triangles) = (0, 0, 0, 0);
    let mut result = Vec::new();
    for placement in &zone.scenery {
        let asset = library
            .assets
            .get(&placement.asset)
            .ok_or_else(|| format!("Missing scenery asset: {}", placement.asset))?;
        let transform = scenery_scene::transform(zone, placement, &asset.entry)?;
        match &asset.entry.collision {
            Some(Binding::Decorative) => continue,
            Some(Binding::TriangleMesh { .. }) => {}
            None => {
                return Err(format!(
                    "Scenery {} has no explicit collision policy; restage its catalog",
                    placement.asset
                ));
            }
        }
        let mesh = asset
            .collision
            .as_ref()
            .ok_or("Missing scenery collision snapshot")?;
        if unique.insert(&placement.asset) {
            pool_vertices += mesh.vertices.len();
            pool_triangles += mesh.indices.len();
        }
        vertices += mesh.vertices.len();
        triangles += mesh.indices.len();
        if pool_vertices > 262_144
            || pool_triangles > 131_072
            || vertices > 1_048_576
            || triangles > 262_144
        {
            return Err("Scenery collision geometry exceeds runtime budget".into());
        }
        let matrix = transform.to_matrix();
        let points = mesh
            .vertices
            .iter()
            .map(|p| matrix.transform_vector3(Vec3::from_array(*p)))
            .collect();
        let shape = Collider::try_trimesh(points, mesh.indices.clone())
            .map_err(|e| format!("Scenery collision construction failed: {e:?}"))?;
        result.push(PlacedCollider {
            shape,
            center: transform.translation,
            layer: SOLID | GROUND | CAMERA,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scenery_collision_rejects_invalid_indices_nonfinite_and_degenerate_geometry() {
        let mut mesh = Mesh {
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]],
            indices: vec![[0, 1, 2]],
        };
        let bounds = [[0.; 3], [1.; 3]];
        mesh.validate(bounds).unwrap();
        mesh.indices[0][2] = 3;
        assert!(mesh.validate(bounds).is_err());
        mesh.indices[0][2] = 1;
        assert!(mesh.validate(bounds).is_err());
        mesh.indices[0][2] = 2;
        mesh.vertices[0][0] = f32::NAN;
        assert!(mesh.validate(bounds).is_err());
        mesh.vertices[0][0] = -1.;
        assert!(mesh.validate(bounds).is_err());
    }
}
