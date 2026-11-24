//! Resolve scenery into the existing GLB and schema-4 static-shape package.
use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub(super) fn geometry_with_ground(
    zone: &Zone,
    gameplay_areas: bool,
    library: &scenery_library::Library,
    ground: Option<&theia_network::procedural::Geometry>,
) -> Result<(Vec<u8>, Value, Value), String> {
    zone.validate()?;
    // Build the established terrain/primitive payload first; every removed
    // scenery reference is resolved below before any artifact can be published.
    let mut base = zone.clone();
    base.scenery.clear();
    let (visual, mut profile, mut report) =
        export::geometry_with_ground(&base, gameplay_areas, ground)?;
    let mut glb = scenery_glb::Glb::read(&visual)?;
    let mut meshes = BTreeMap::new();
    let mut collision_meshes = BTreeMap::new();
    let mut triangles = 0;
    let mut bodies = 0;
    for placement in &zone.scenery {
        let asset = library
            .assets
            .get(&placement.asset)
            .ok_or_else(|| format!("Missing scenery asset: {}", placement.asset))?;
        let transform = scenery_scene::transform(zone, placement, &asset.entry)?;
        let index = if let Some(index) = meshes.get(&asset.entry.sha256) {
            *index
        } else {
            let index = glb.append(&asset.bytes)?;
            meshes.insert(asset.entry.sha256.clone(), index);
            index
        };
        glb.instance(index, &placement.name, transform)?;
        triangles += asset.entry.triangles;
        match &asset.entry.collision {
            Some(scenery_collision::Binding::Decorative) => {}
            Some(scenery_collision::Binding::TriangleMesh { sha256, .. }) => {
                let mesh = asset
                    .collision
                    .as_ref()
                    .ok_or("Missing scenery collision snapshot")?;
                let index = if let Some(index) = collision_meshes.get(sha256) {
                    *index
                } else {
                    let pool = profile["static_shapes"]["meshes"]
                        .as_array_mut()
                        .ok_or("Missing static mesh pool")?;
                    let index = pool.len();
                    pool.push(json!({"vertices":mesh.vertices,"indices":mesh.indices}));
                    collision_meshes.insert(sha256.clone(), index);
                    index
                };
                profile["static_shapes"]["objects"].as_array_mut().ok_or("Missing static bodies")?.push(json!({"kind":"mesh","mesh":index,"affine":transform.to_matrix().transpose().to_cols_array(),"convex":false}));
                bodies += 1;
            }
            None => {
                return Err(format!(
                    "Scenery {} has no explicit collision policy",
                    placement.asset
                ));
            }
        }
    }
    let total = report["visual_triangles"]
        .as_u64()
        .ok_or("Missing triangle count")?
        + triangles as u64;
    if total > 262_144 {
        return Err("Scenery export exceeds visual triangle budget".into());
    }
    let visual = glb.finish()?;
    profile["visual_sha256"] = json!(theia_network::terrain::sha256(&visual));
    report["visual_triangles"] = json!(total);
    report["scenery"] = json!({"placements":zone.scenery.len(),"unique_visuals":meshes.len(),"solid_instances":bodies,"unique_collision_meshes":collision_meshes.len(),"decorative_instances":zone.scenery.len()-bodies,"visual_triangles":triangles});
    Ok((visual, profile, report))
}
