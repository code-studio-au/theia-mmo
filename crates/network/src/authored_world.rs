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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
}
/// One explicit dialogue contact, matching the current runtime contact budget.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub npc: String,
    pub feet: [f64; 3],
}
impl Contact {
    pub fn validate(&self, origin: GlobalPosition, terrain: &StaticScene) -> Result<()> {
        if theia_core::content::ContentId::new(&self.npc).is_none() {
            return Err("Authored contact needs a valid NPC catalog ID".into());
        }
        let position = GlobalPosition::new(self.feet).ok_or("Invalid authored contact feet")?;
        if procedural::owner(position).is_none() {
            return Err("Authored contact leaves the supported world".into());
        }
        let local = LocalFrame::new(origin)
            .to_local(position)
            .ok_or("Contact outside local frame")?;
        crate::kinematic::spawn(terrain, local)
            .map_err(|_| "Authored contact has no safe support")?;
        Ok(())
    }
}
pub struct Store {
    terrain: Terrain,
    origin: GlobalPosition,
    identity: [u8; 32],
    visual: Arc<Vec<u8>>,
    contact: Option<Contact>,
    particles: Option<(crate::particle_content::Catalog, std::path::PathBuf)>,
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
        if !matches!(anchor.schema, 1 | 2)
            || (anchor.schema == 2) != anchor.contact.is_some()
            || anchor.origin[1] != 0.
        {
            return Err("Authored prediction requires schema 1, or schema 2 with a contact, and zero origin Y".into());
        }
        let terrain = Terrain::load(&path(&root, "terrain.json")?)?;
        let package = terrain
            .package
            .as_ref()
            .ok_or("Authored prediction requires a versioned world package")?;
        if package.identity() != anchor.world_identity
            || package.requires_material_profiles()
            || !package.shared_images().is_empty()
        {
            return Err("Authored anchor mismatch or unsupported presentation dependencies".into());
        }
        if !terrain.has_spawns() {
            return Err("Authored world requires an explicit safe player spawn".into());
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
        let particles = if package.has_particle_catalog() {
            let bytes = content::read_bounded(
                &path(&root, "particles/catalog.json")?,
                crate::particle_content::MAX_CATALOG_BYTES,
            )?;
            let catalog = package.verify_particle_catalog(&bytes)?;
            package.verify_visual_for_particle_catalog(&path(&root, "terrain.glb")?, &bytes)?;
            // Validate exact dependency bytes at startup, without renderer or simulation I/O.
            catalog.read_effects(&path(&root, "particles/effects")?)?;
            catalog.read_textures(&path(&root, "particles/textures")?)?;
            Some((catalog, root.join("particles")))
        } else {
            None
        };
        let mut identity_bytes = b"theia-authored-predicted-world-v1\0".to_vec();
        identity_bytes.extend(&bytes);
        identity_bytes.extend(package.identity());
        let terrain = terrain.into_world();
        if let Some(contact) = &anchor.contact {
            contact.validate(origin, &terrain.physics)?;
        }
        Ok(Self {
            terrain,
            origin,
            identity: sha256(&identity_bytes),
            visual: Arc::new(visual),
            contact: anchor.contact,
            particles,
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn origin(&self) -> GlobalPosition {
        self.origin
    }
    pub fn particles(&self) -> Option<(&crate::particle_content::Catalog, &Path)> {
        self.particles
            .as_ref()
            .map(|(catalog, root)| (catalog, root.as_path()))
    }
    pub fn contact_placement(
        &self,
        profile: &crate::dialogue::Profile,
    ) -> Result<crate::dialogue::Placement> {
        let contact = self
            .contact
            .as_ref()
            .ok_or("Authored world has no bound NPC contact")?;
        if profile.profile().npc().as_str() != contact.npc {
            return Err("Dialogue catalog does not match the authored NPC contact".into());
        }
        Ok(crate::dialogue::Placement::Feet(
            GlobalPosition::new(contact.feet).ok_or("Invalid authored contact")?,
        ))
    }
    /// Hash-bound capsule feet, including the authored vertical support layer.
    /// Multiple players may share an authored spawn; actors never block actors.
    pub fn spawn(&self, slot: usize) -> Option<GlobalPosition> {
        LocalFrame::new(self.origin).to_global(self.terrain.spawn(slot)?)
    }
    pub fn visual(&self) -> Arc<Vec<u8>> {
        self.visual.clone()
    }
    pub fn map_contains(&self, position: GlobalPosition) -> bool {
        let [x, _, z] = position.metres();
        let origin = self.origin.metres();
        let bounds = self.terrain.bounds();
        [x - origin[0], z - origin[2]]
            .into_iter()
            .enumerate()
            .all(|(i, p)| p >= f64::from(bounds[0][i]) && p <= f64::from(bounds[1][i]))
    }
    /// Read-only topographic presentation sample from already admitted ground.
    /// This is the highest ground surface, not actor support/navigation or a
    /// terrain request. Global Y is irrelevant to the map's downward projection.
    pub fn map_height(&self, position: GlobalPosition) -> Option<f32> {
        if !self.map_contains(position) {
            return None;
        }
        let [x, _, z] = position.metres();
        let local = LocalFrame::new(self.origin)
            .to_local(GlobalPosition::new([x, 0., z])?)?
            .metres();
        self.terrain.height([local[0], local[2]])
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
        spawn_fixture([8000., 0., 8000.], false, false)
    }
    fn spawn_fixture(origin: [f64; 3], roof: bool, empty: bool) -> (Folder, Anchor) {
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
        let mut objects =
            vec![json!({"kind":"box","center":[4,1,0],"size":[1,2,4],"rotation":[0,0,0,1]})];
        let mut spawns = vec![[0., 0.1, 0.]];
        if roof {
            objects
                .push(json!({"kind":"box","center":[0,4,0],"size":[12,1,12],"rotation":[0,0,0,1]}));
            spawns.push([0., 4.6, 0.]);
        }
        if empty {
            spawns.clear();
        }
        let profile=serde_json::to_vec(&json!({"schema":4,"source_origin":[0,0,0],"bounds":[[-32,-32],[32,32]],"spawns":spawns,"visual_sha256":sha256(visual),"triangles":[[[-32,0,-32],[32,0,32],[32,0,-32]],[[-32,0,-32],[-32,0,32],[32,0,32]]],"static_shapes":{"meshes":[],"objects":objects}})).unwrap();
        let manifest=serde_json::to_vec(&json!({"schema":1,"content_id":"theia.test.authored","revision":1,"collision_schema":4,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(visual)})).unwrap();
        let identity = content::Manifest::decode(&manifest).unwrap().identity();
        let anchor = Anchor {
            schema: 1,
            origin,
            world_identity: identity,
            contact: None,
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
    fn map_samples_use_checked_global_origin_and_do_not_extend_the_world() {
        let (folder, _) = spawn_fixture([8192., 0., 8064.], true, false);
        let store = Store::open(&folder.0).unwrap();
        let identity = store.identity();
        let spawn = store.spawn(0).unwrap();
        // A topographic map shows the upper roof even when an actor stands
        // below it. Requested Y must not select a different support layer or
        // change the immutable world's identity.
        for y in [0., 4.6, -100.] {
            assert_eq!(
                store.map_height(GlobalPosition::new([8192., y, 8064.]).unwrap()),
                Some(4.5)
            );
        }
        assert_eq!(
            store.map_height(GlobalPosition::new([8200., 0., 8064.]).unwrap()),
            Some(0.)
        );
        for xz in [
            [8159.9, 8064.],
            [8224.1, 8064.],
            [8192., 8031.9],
            [8192., 8096.1],
            [0., 0.],
        ] {
            assert_eq!(
                store.map_height(GlobalPosition::new([xz[0], 0., xz[1]]).unwrap()),
                None
            );
        }
        assert_eq!(store.identity(), identity);
        assert_eq!(store.spawn(0).unwrap(), spawn);
    }
    #[test]
    fn contact_metadata_is_versioned_supported_profile_checked_and_identity_bound() {
        let (folder, mut anchor) = spawn_fixture([8192., 0., 8192.], true, false);
        let legacy = Store::open(&folder.0).unwrap().identity();
        let profile = crate::dialogue::Profile::from_json(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../content/gameplay/development-dialogue.json"
        )))
        .unwrap();
        anchor.schema = 2;
        anchor.contact = Some(Contact {
            npc: profile.profile().npc().as_str().into(),
            feet: [8192., 0.03, 8192.],
        });
        let publish = |anchor: &Anchor| {
            std::fs::write(
                folder.0.join(ANCHOR_FILE),
                serde_json::to_vec(anchor).unwrap(),
            )
            .unwrap()
        };
        publish(&anchor);
        let floor = Arc::new(Store::open(&folder.0).unwrap());
        assert_ne!(legacy, floor.identity());
        let source = procedural::Source::Authored(floor.clone());
        let placement = crate::dialogue::Placement::resolve("authored", &source, &profile).unwrap();
        assert_eq!(placement.focus().unwrap().metres(), [8192., 0.03, 8192.]);
        assert!(
            crate::dialogue::Placement::resolve(
                "authored",
                &procedural::Source::Procedural,
                &profile
            )
            .is_err()
        );
        anchor.contact.as_mut().unwrap().feet[1] = 4.53;
        publish(&anchor);
        assert_ne!(floor.identity(), Store::open(&folder.0).unwrap().identity());
        anchor.contact.as_mut().unwrap().npc = "theia.other.contact".into();
        publish(&anchor);
        assert!(
            Store::open(&folder.0)
                .unwrap()
                .contact_placement(&profile)
                .is_err()
        );
        for y in [4., 100.] {
            anchor.contact.as_mut().unwrap().feet[1] = y;
            publish(&anchor);
            assert!(Store::open(&folder.0).is_err());
        }
        anchor.contact.as_mut().unwrap().feet[1] = 0.03;
        anchor.schema = 1;
        publish(&anchor);
        assert!(Store::open(&folder.0).is_err());
        anchor.schema = 2;
        anchor.contact = None;
        publish(&anchor);
        assert!(Store::open(&folder.0).is_err());
    }
    #[test]
    fn explicit_guide_height_selects_its_floor_and_rejects_unsupported_positions_atomically() {
        use crate::dialogue::{Placement, Profile};
        let (folder, _) = spawn_fixture([8192., 0., 8192.], true, false);
        let store = Arc::new(Store::open(&folder.0).unwrap());
        let profile = Arc::new(
            Profile::from_json(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../content/gameplay/development-dialogue.json"
            )))
            .unwrap(),
        );
        let mut identities = Vec::new();
        for height in [0.1, 4.6] {
            let mut world = crate::simulation::World::with_terrain_source(
                [8192.; 2],
                1.,
                procedural::Source::Authored(store.clone()),
            )
            .unwrap();
            world.enable_prediction().unwrap();
            let base = world.identity();
            for position in [[8192., 20., 8192.], [8192., 4., 8192.], [0.; 3]] {
                assert!(
                    world
                        .spawn_placed_guide(
                            profile.clone(),
                            Placement::Feet(GlobalPosition::new(position).unwrap())
                        )
                        .is_err()
                );
                assert!(world.dialogue_profile().is_none());
                assert_eq!(world.identity(), base);
            }
            let placement = Placement::Feet(GlobalPosition::new([8192., height, 8192.]).unwrap());
            let focus = placement.focus().unwrap();
            let count = world
                .procedural()
                .unwrap()
                .patch(focus)
                .unwrap()
                .scene
                .colliders
                .len();
            let guide = world
                .spawn_placed_guide(profile.clone(), placement)
                .unwrap();
            let expected = if height < 1. { 0. } else { 4.5 } + f64::from(crate::world::PAWN_Y);
            assert!((guide.position.metres()[1] - expected).abs() < 0.01);
            world.enable_classic().unwrap();
            assert_eq!(
                world.identity(),
                crate::classic::profile_identity(placement.identity(base, profile.hash()))
            );
            identities.push(world.identity());
            assert_eq!(
                world
                    .procedural()
                    .unwrap()
                    .patch(focus)
                    .unwrap()
                    .scene
                    .colliders
                    .len(),
                count
            );
            let body = |slot| {
                let mut p = store.spawn(slot).unwrap().metres();
                p[1] += f64::from(crate::world::PAWN_Y);
                GlobalPosition::new(p).unwrap()
            };
            let ground = world.join_restored(body(0)).unwrap();
            let rooftop = world.join_restored(body(1)).unwrap();
            assert_eq!(world.nearby_guide(ground).is_some(), height < 1.);
            assert_eq!(world.nearby_guide(rooftop).is_some(), height > 1.);
            assert!(
                world
                    .spawn_placed_guide(profile.clone(), placement)
                    .is_err()
            );
        }
        assert_ne!(identities[0], identities[1]);
    }
    #[test]
    fn test_players_bind_checked_xyz_to_client_ids_and_refuse_stale_or_unsafe_plans() {
        use crate::renet_runtime::test_players::{MAX_BYTES, Plan};
        let (folder, _) = spawn_fixture([8192., 0., 8192.], true, false);
        let store = Arc::new(Store::open(&folder.0).unwrap());
        let mut world = crate::simulation::World::with_terrain_source(
            [8192.; 2],
            1.,
            procedural::Source::Authored(store.clone()),
        )
        .unwrap();
        let document = json!({"schema":1,"world_identity":store.identity(),"players":[
            {"client_id":2,"feet":[8192.,4.6,8192.]},
            {"client_id":1,"feet":[8190.,0.1,8192.]}
        ]});
        let bytes = serde_json::to_vec(&document).unwrap();
        assert!(Plan::from_json(&bytes, &world).is_err());
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        let original_identity = world.identity();
        for client_id in [0, 3, u64::MAX] {
            let mut invalid = document.clone();
            // Every syntactic invalidity is refused before there can be admission.
            invalid["players"][0]["client_id"] = json!(client_id);
            assert!(Plan::from_json(&serde_json::to_vec(&invalid).unwrap(), &world).is_err());
        }
        for (field, value) in [
            ("schema", json!(2)),
            ("world_identity", json!(vec![0; 32])),
            ("players", json!([])),
            ("unknown", json!(true)),
        ] {
            let mut invalid = document.clone();
            invalid[field] = value;
            assert!(Plan::from_json(&serde_json::to_vec(&invalid).unwrap(), &world).is_err());
        }
        let mut duplicate = document.clone();
        duplicate["players"][0]["client_id"] = json!(1);
        assert!(Plan::from_json(&serde_json::to_vec(&duplicate).unwrap(), &world).is_err());
        assert!(Plan::from_json(&vec![b' '; MAX_BYTES + 1], &world).is_err());
        for feet in [
            [8192., 100., 8192.],
            [8192., 4., 8192.],
            [8196., 0.1, 8192.],
        ] {
            let mut invalid = document.clone();
            invalid["players"][0]["feet"] = json!(feet);
            let plan = Plan::from_json(&serde_json::to_vec(&invalid).unwrap(), &world).unwrap();
            assert!(plan.prepare(&world).is_err());
            assert!(!world.has_players());
            assert_eq!(world.identity(), original_identity);
        }
        let plan = Plan::from_json(&bytes, &world).unwrap();
        let positions = plan.positions();
        assert_eq!(positions[0].metres()[0], 8190.);
        assert_eq!(positions[1].metres()[0], 8192.);
        assert!((positions[1].metres()[1] - positions[0].metres()[1] - 4.5).abs() < 0.0001);
        plan.prepare(&world).unwrap();
        // Reverse arrival and an unrelated target do not define the client's spawn.
        world.spawn_development_target([8188., 8192.]).unwrap();
        for position in positions.into_iter().rev() {
            let actor = world.join_restored(position).unwrap();
            assert_eq!(world.global_position(actor), Some(position));
        }
        assert!(
            Plan::from_json(&bytes, &world)
                .unwrap()
                .prepare(&world)
                .is_err()
        );
    }

    #[test]
    fn authored_spawn_keeps_ground_and_roof_layers_for_all_player_slots() {
        let (folder, _) = spawn_fixture([8192., 0., 8192.], true, false);
        let store = Arc::new(Store::open(&folder.0).unwrap());
        let ground = store.spawn(0).unwrap();
        let roof = store.spawn(1).unwrap();
        assert!(ground.metres()[1] < 0.02);
        assert!((roof.metres()[1] - 4.5).abs() < 0.02);
        let source = procedural::Source::Authored(store);
        assert!(
            crate::simulation::World::with_terrain_source([8000., 8000.], 1., source.clone())
                .is_err()
        );
        let mut world =
            crate::simulation::World::with_terrain_source([8192., 8192.], 1., source).unwrap();
        world.enable_prediction().unwrap();
        world.enable_classic().unwrap();
        for slot in 1..=4 {
            let expected = if slot % 2 == 1 { ground } else { roof };
            let development = world.development_player_spawn(slot).unwrap();
            let id = world.join().unwrap();
            let actual = world.global_position(id).unwrap();
            assert_eq!(actual, development);
            let mut feet = actual.metres();
            feet[1] -= f64::from(crate::world::PAWN_Y);
            assert_eq!([feet[0], feet[2]], [8192., 8192.]);
            assert!((feet[1] - expected.metres()[1]).abs() < 0.001);
            world.validate_restored_player(actual).unwrap();
        }
        let mut restored = ground.metres();
        restored[1] += f64::from(crate::world::PAWN_Y);
        let restored = GlobalPosition::new(restored).unwrap();
        let id = world.join_restored(restored).unwrap();
        assert_eq!(world.global_position(id), Some(restored));
    }
    #[test]
    fn authored_world_without_explicit_spawn_refuses() {
        let (folder, _) = spawn_fixture([8192., 0., 8192.], false, true);
        assert!(
            Store::open(&folder.0)
                .err()
                .unwrap()
                .to_string()
                .contains("explicit safe player spawn")
        );
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
