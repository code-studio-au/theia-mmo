//! Freeze the unchanged imported ground beside a bounded authored settlement.
//! The small collision-only snapshot admits legacy spawn/contact data; runtime
//! collision and navigation use the full-resolution streamed ground instead.
use super::*;
use std::{fs, path::Path};
use theia_network::{authored_world::ImportedGround, procedural, saved_terrain};

pub(super) struct Prepared {
    pub zone: Zone,
    pub ground: procedural::Geometry,
    source: saved_terrain::Store,
}
impl Prepared {
    pub fn new(zone: &Zone) -> Result<Self, String> {
        let imported = zone
            .imported_terrain
            .as_ref()
            .ok_or("Missing imported terrain")?;
        if imported.stamp_count() != 0 {
            return Err(
                "Bake the edited terrain journal before publishing a populated imported world"
                    .into(),
            );
        }
        let source =
            saved_terrain::Store::open(Path::new(&imported.package)).map_err(|e| e.to_string())?;
        if source.identity() != imported.identity {
            return Err("Imported terrain changed before export".into());
        }
        let owner = procedural::owner(
            theia_core::coordinates::GlobalPosition::new(zone.spawn).ok_or("Invalid spawn")?,
        )
        .ok_or("Spawn outside terrain domain")?;
        let mut local = zone.clone();
        local.imported_terrain = None;
        local.origin = owner.origin(128).ok_or("Invalid village origin")?.metres();
        // This slice publishes one bounded settlement; don't silently omit
        // outlying authored content or turn a regional document into one mesh.
        local.validate()?;
        let mut ground = procedural::Geometry {
            positions: vec![],
            triangles: vec![],
        };
        for id in procedural::neighbours(owner, 1)
            .into_iter()
            .filter(|id| source.contains(*id))
        {
            let mut tile = source
                .geometry(id, source.cells())
                .map_err(|e| e.to_string())?;
            let offset = local.local(id.origin(128).ok_or("Invalid tile origin")?.metres())?;
            for p in &mut tile.positions {
                p[0] += offset[0];
                p[2] += offset[2];
            }
            let base = ground.positions.len() as u32;
            ground.positions.extend(tile.positions);
            ground
                .triangles
                .extend(tile.triangles.into_iter().map(|t| t.map(|i| i + base)));
        }
        Ok(Self {
            zone: local,
            ground,
            source,
        })
    }
    pub fn binding(&self) -> ImportedGround {
        ImportedGround {
            directory: "heightfield".into(),
            identity: self.source.identity(),
        }
    }
    pub fn write(&self, stage: &Path) -> Result<(), String> {
        let destination = stage.join("heightfield");
        fs::create_dir(&destination).map_err(|e| e.to_string())?;
        let (origin, counts) = self.source.chunk_layout();
        for z in 0..counts[1] {
            for x in 0..counts[0] {
                let id = theia_core::coordinates::ChunkId {
                    x: origin[0] + x as i64,
                    z: origin[1] + z as i64,
                };
                let name = format!("tile_{}_{}.height", id.x, id.z);
                fs::copy(self.source.root().join(&name), destination.join(name))
                    .map_err(|e| e.to_string())?;
            }
        }
        fs::copy(
            self.source.root().join(saved_terrain::MANIFEST),
            destination.join(saved_terrain::MANIFEST),
        )
        .map_err(|e| e.to_string())?;
        let frozen = saved_terrain::Store::open(&destination).map_err(|e| e.to_string())?;
        if frozen.identity() != self.source.identity() {
            return Err("Terrain changed during snapshot".into());
        }
        frozen.verify().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use theia_core::coordinates::ChunkId;
    use theia_network::terrain::sha256;

    #[test]
    fn publication_preserves_imported_samples_and_keeps_ground_out_of_the_visual() {
        let root = std::env::temp_dir().join(format!(
            "theia-village-export-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let input = root.join("input");
        fs::create_dir(&input).unwrap();
        let mut hashes = vec![];
        for z in 10_i64..13 {
            for x in 10_i64..13 {
                let mut data = b"THTILE01".to_vec();
                data.extend(x.to_le_bytes());
                data.extend(z.to_le_bytes());
                for _ in 0..65 * 65 {
                    data.extend(7_f32.to_le_bytes());
                }
                fs::write(input.join(format!("tile_{x}_{z}.height")), &data).unwrap();
                hashes.push(sha256(&data));
            }
        }
        fs::write(
            input.join(saved_terrain::MANIFEST),
            serde_json::to_vec(&json!({
                "schema":2,"content_id":"theia.test.village","revision":1,"chunk_metres":128,
                "chunk_origin":[10,10],"chunks_xz":[3,3],"samples_per_axis":65,"tiles":hashes
            }))
            .unwrap(),
        )
        .unwrap();
        let imported = theia_zone::imported::Imported::open(&input).unwrap();
        let mut zone = Zone {
            schema: 16,
            origin: [1472., 0., 1472.],
            spawn: [1472., 7.1, 1472.],
            imported_terrain: Some(imported),
            ..Default::default()
        };
        zone.objects.push(theia_zone::Object {
            id: 1,
            name: "Test building".into(),
            primitive: theia_zone::Primitive::Box,
            pose: theia_zone::Pose {
                position: [1492., 8., 1472.],
                size: [2., 2., 2.],
                yaw_degrees: 0.,
            },
        });
        let original = zone.clone();
        let output = root.join("world");
        let report =
            crate::export::publish(&zone, &output, "theia.test.imported-village", 1).unwrap();
        assert_eq!(zone, original);
        assert_eq!(report["visual_triangles"], 12);
        let store = theia_network::authored_world::Store::open(&output).unwrap();
        assert_eq!(
            store.imported().unwrap().identity(),
            zone.imported_terrain.as_ref().unwrap().identity
        );
        assert_eq!(store.origin().metres(), [1408., 0., 1408.]);
        let patch = store.patch(ChunkId { x: 11, z: 11 }).unwrap();
        assert_eq!(patch.scene.colliders.len(), 10);
        assert!((patch.scene.ground([64., 64.], 20.).unwrap().0 - 7.).abs() < 0.001);
        assert!((patch.scene.ground([84., 64.], 20.).unwrap().0 - 9.).abs() < 0.001);
        assert_eq!(
            Zone::load(&output.join("authoring.zone.json")).unwrap(),
            original
        );
        assert!(crate::export::publish(&zone, &output, "theia.test.imported-village", 2).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
