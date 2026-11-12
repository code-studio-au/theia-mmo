//! Immutable bounded editor worlds for the existing prediction/terrain-patch path.
//! This adapter preserves authored f64 world origins; it never resamples collision.
use crate::{
    Result, content,
    physics::StaticScene,
    procedural,
    terrain::{Terrain, sha256},
};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use theia_core::coordinates::{ChunkId, GlobalPosition, LocalFrame};

pub const ANCHOR_FILE: &str = "authored.world.json";
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub schema: u32,
    pub origin: [f64; 3],
    pub world_identity: [u8; 32],
}
pub struct Store {
    terrain: Terrain,
    origin: GlobalPosition,
    identity: [u8; 32],
    visual: Arc<Vec<u8>>,
}
fn path(root: &Path, name: &str) -> Result<std::path::PathBuf> {
    let file = root.join(name).canonicalize()?;
    if !file.starts_with(root) {
        return Err("Authored world file escapes package".into());
    }
    Ok(file)
}
impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;
        let bytes = content::read_bounded(&path(&root, ANCHOR_FILE)?, 4096)?;
        let anchor: Anchor = serde_json::from_slice(&bytes)?;
        let origin = GlobalPosition::new(anchor.origin).ok_or("Invalid authored origin")?;
        if anchor.schema != 1 || anchor.origin[1] != 0. {
            return Err("Initial authored prediction requires schema 1 and zero origin Y".into());
        }
        let terrain = Terrain::load(&path(&root, "terrain.json")?)?;
        let package = terrain
            .package
            .as_ref()
            .ok_or("Authored prediction requires a versioned world package")?;
        if package.identity() != anchor.world_identity
            || package.has_particle_catalog()
            || package.requires_material_profiles()
            || !package.shared_images().is_empty()
        {
            return Err("Authored anchor mismatch or unsupported presentation dependencies".into());
        }
        let bounds = terrain.bounds();
        for (i, (&min, &max)) in bounds[0].iter().zip(&bounds[1]).enumerate() {
            let origin = anchor.origin[i * 2];
            if origin + f64::from(min) < 1. || origin + f64::from(max) > procedural::SIDE - 1. {
                return Err(
                    "Authored bounds leave the existing continental prediction domain".into(),
                );
            }
        }
        let visual = content::read_bounded(&path(&root, "terrain.glb")?, 64 * 1024 * 1024)?;
        if sha256(&visual) != terrain.visual_sha256 {
            return Err("Authored visual changed after collision admission".into());
        }
        let mut identity_bytes = b"theia-authored-predicted-world-v1\0".to_vec();
        identity_bytes.extend(&bytes);
        identity_bytes.extend(package.identity());
        Ok(Self {
            terrain: terrain.into_world(),
            origin,
            identity: sha256(&identity_bytes),
            visual: Arc::new(visual),
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn origin(&self) -> GlobalPosition {
        self.origin
    }
    pub fn visual(&self) -> Arc<Vec<u8>> {
        self.visual.clone()
    }
    pub fn patch(&self, owner: ChunkId) -> Result<procedural::Patch> {
        if !procedural::contains(owner) {
            return Err("Authored patch leaves supported domain".into());
        }
        let frame = LocalFrame::new(owner.origin(128).ok_or("Invalid patch origin")?);
        let offset = frame
            .to_local(self.origin)
            .ok_or("Authored world outside bounded patch frame")?
            .metres();
        let bounds = self.terrain.bounds();
        // Only owners intersecting the authored package or one neighbouring
        // preparation tile may hold a copy; never populate the continent with it.
        for i in 0..2 {
            if f64::from(offset[i * 2] + bounds[1][i]) < -128.
                || f64::from(offset[i * 2] + bounds[0][i]) > 256.
            {
                return Err("Requested patch is outside authored world".into());
            }
        }
        let mut colliders = self.terrain.physics.colliders.clone();
        for collider in &mut colliders {
            collider.center += bevy::prelude::Vec3::from_array(offset);
        }
        Ok(procedural::Patch {
            owner,
            frame,
            scene: StaticScene::new(colliders),
            ids: procedural::neighbours(owner, procedural::PATCH_RADIUS),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    struct Folder(std::path::PathBuf);
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn fixture() -> (Folder, Anchor) {
        let root = std::env::temp_dir().join(format!(
            "theia-authored-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let visual = b"Synthetic visual snapshot; no rendered acceptance";
        let profile=serde_json::to_vec(&json!({"schema":4,"source_origin":[0,0,0],"bounds":[[-32,-32],[32,32]],"spawns":[[0,0.1,0]],"visual_sha256":sha256(visual),"triangles":[[[-32,0,-32],[32,0,32],[32,0,-32]],[[-32,0,-32],[-32,0,32],[32,0,32]]],"static_shapes":{"meshes":[],"objects":[{"kind":"box","center":[4,1,0],"size":[1,2,4],"rotation":[0,0,0,1]}]}})).unwrap();
        let manifest=serde_json::to_vec(&json!({"schema":1,"content_id":"theia.test.authored","revision":1,"collision_schema":4,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)})).unwrap();
        let identity = content::Manifest::decode(&manifest).unwrap().identity();
        let anchor = Anchor {
            schema: 1,
            origin: [8000., 0., 8000.],
            world_identity: identity,
        };
        for (name, bytes) in [
            ("terrain.json", profile),
            ("terrain.glb", visual.to_vec()),
            (content::MANIFEST_FILE, manifest),
            (ANCHOR_FILE, serde_json::to_vec(&anchor).unwrap()),
        ] {
            std::fs::write(root.join(name), bytes).unwrap();
        }
        (Folder(root), anchor)
    }
    #[test]
    fn authored_world_preserves_frame_collision_and_can_prepare_classic_guide() {
        let (folder, _) = fixture();
        let store = Arc::new(Store::open(&folder.0).unwrap());
        let mut heights = Vec::new();
        for owner in [ChunkId { x: 62, z: 62 }, ChunkId { x: 63, z: 62 }] {
            let patch = store.patch(owner).unwrap();
            let position = patch.try_spawn(8000., 8000.).unwrap();
            heights.push(position.metres()[1]);
            let a = patch
                .frame
                .to_local(GlobalPosition::new([8000., 1., 8000.]).unwrap())
                .unwrap();
            let b = patch
                .frame
                .to_local(GlobalPosition::new([8008., 1., 8000.]).unwrap())
                .unwrap();
            assert!(!patch.scene.line_clear(a.metres().into(), b.metres().into()));
        }
        assert!((heights[0] - heights[1]).abs() < 0.0001);
        assert!(store.patch(ChunkId { x: 1, z: 1 }).is_err());
        let mut world = crate::simulation::World::with_terrain_source(
            [8000., 8000.],
            1.,
            procedural::Source::Authored(store),
        )
        .unwrap();
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let before = world
            .procedural()
            .unwrap()
            .patches
            .values()
            .next()
            .unwrap()
            .scene
            .colliders
            .len();
        let profile = Arc::new(
            crate::dialogue::Profile::from_json(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../content/gameplay/development-dialogue.json"
            )))
            .unwrap(),
        );
        let guide = world
            .spawn_development_guide(profile, [8008., 8000.])
            .unwrap();
        assert_eq!(guide.position.metres()[0], 8008.);
        assert_eq!(
            world
                .procedural()
                .unwrap()
                .patches
                .values()
                .next()
                .unwrap()
                .scene
                .colliders
                .len(),
            before
        );
        assert!(world.prediction_enabled() && world.classic_enabled());
    }
    #[test]
    fn authored_world_binds_origin_content_and_immutable_visual_bytes() {
        let (folder, mut anchor) = fixture();
        let store = Store::open(&folder.0).unwrap();
        let visual = store.visual();
        anchor.origin[0] += 128.;
        std::fs::write(
            folder.0.join(ANCHOR_FILE),
            serde_json::to_vec(&anchor).unwrap(),
        )
        .unwrap();
        assert_ne!(store.identity(), Store::open(&folder.0).unwrap().identity());
        anchor.world_identity[0] ^= 1;
        std::fs::write(
            folder.0.join(ANCHOR_FILE),
            serde_json::to_vec(&anchor).unwrap(),
        )
        .unwrap();
        assert!(Store::open(&folder.0).is_err());
        anchor.world_identity[0] ^= 1;
        anchor.origin = [0.; 3];
        std::fs::write(
            folder.0.join(ANCHOR_FILE),
            serde_json::to_vec(&anchor).unwrap(),
        )
        .unwrap();
        assert!(Store::open(&folder.0).is_err());
        anchor.origin = [8000., 0., 8000.];
        std::fs::write(
            folder.0.join(ANCHOR_FILE),
            serde_json::to_vec(&anchor).unwrap(),
        )
        .unwrap();
        std::fs::write(folder.0.join("terrain.glb"), b"changed").unwrap();
        assert!(Store::open(&folder.0).is_err());
        assert_eq!(
            visual.as_slice(),
            b"Synthetic visual snapshot; no rendered acceptance"
        );
    }
}
