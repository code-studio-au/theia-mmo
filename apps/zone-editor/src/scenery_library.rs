//! Immutable, hash-checked local GLBs served through Bevy's standard asset reader.
use super::*;
use bevy::asset::{
    AssetApp,
    io::{
        AssetSourceBuilder,
        memory::{Dir, MemoryAssetReader, Value},
    },
};
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read, path::Path, sync::Arc};

const MAX_LIBRARY_BYTES: usize = 128 * 1024 * 1024;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Entry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub file: String,
    pub bytes: usize,
    pub sha256: String,
    pub bounds: [[f64; 3]; 2],
    pub triangles: usize,
    pub source: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_checks_bytes_paths_and_keeps_validated_snapshot_immutable() {
        let root = std::env::temp_dir().join(format!(
            "theia-scenery-library-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let bytes = b"CPU integrity fixture; the GLB loader is checked separately";
        let hash: String = theia_network::terrain::sha256(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let asset = root.join(format!("{hash}.glb"));
        std::fs::write(&asset, bytes).unwrap();
        let path = root.join("catalog.json");
        let mut catalog = serde_json::json!({"schema":1,"id":"theia.scenery.test","assets":[{
            "id":"theia.asset.arch","name":"Arch","category":"Structures","file":format!("{hash}.glb"),
            "bytes":bytes.len(),"sha256":hash,"bounds":[[-1.,0.,-1.],[1.,2.,1.]],"triangles":1,"source":{}
        }]});
        std::fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        let library = Library::load(&path).unwrap();
        std::fs::write(&asset, b"changed").unwrap();
        assert!(Library::load(&path).is_err());
        assert_eq!(library.assets["theia.asset.arch"].bytes.as_slice(), bytes);
        catalog["assets"][0]["file"] = serde_json::json!("../outside.glb");
        std::fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
        assert!(Library::load(&path).err().unwrap().contains("metadata"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: u32,
    id: String,
    assets: Vec<Entry>,
}
#[derive(Clone)]
pub(super) struct Asset {
    pub entry: Entry,
    pub bytes: Arc<Vec<u8>>,
}
#[derive(Resource, Clone, Default)]
pub(super) struct Library {
    pub assets: BTreeMap<String, Asset>,
}
fn bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Scenery input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("Scenery input exceeds byte budget".into());
    }
    Ok(bytes)
}
impl Library {
    pub fn load(path: &Path) -> Result<Self, String> {
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let root = path.parent().ok_or("Scenery catalog needs a directory")?;
        let catalog: Catalog =
            serde_json::from_slice(&bounded(&path, 128 * 1024)?).map_err(|e| e.to_string())?;
        if catalog.schema != 1
            || theia_core::content::ContentId::new(&catalog.id).is_none()
            || !(1..=64).contains(&catalog.assets.len())
        {
            return Err("Invalid scenery catalog identity or entry count".into());
        }
        let mut assets = BTreeMap::new();
        let mut total = 0usize;
        for entry in catalog.assets {
            if theia_core::content::ContentId::new(&entry.id).is_none()
                || entry.name.trim().is_empty()
                || entry.name.len() > 80
                || entry.name.chars().any(char::is_control)
                || entry.category.trim().is_empty()
                || entry.category.len() > 32
                || entry.category.chars().any(char::is_control)
                || entry.sha256.len() != 64
                || !entry
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || entry.file != format!("{}.glb", entry.sha256)
                || !(1..=32 * 1024 * 1024).contains(&entry.bytes)
                || !(1..=65_536).contains(&entry.triangles)
                || !entry
                    .bounds
                    .iter()
                    .flatten()
                    .all(|v| v.is_finite() && v.abs() <= 128.)
                || (0..3).any(|i| entry.bounds[0][i] > entry.bounds[1][i])
                || !entry.source.is_object()
            {
                return Err("Invalid scenery entry metadata".into());
            }
            total = total
                .checked_add(entry.bytes)
                .ok_or("Scenery byte budget exceeded")?;
            if total > MAX_LIBRARY_BYTES {
                return Err("Scenery byte budget exceeded".into());
            }
            let asset_path = root
                .join(&entry.file)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !asset_path.starts_with(root) {
                return Err("Scenery asset escapes its library".into());
            }
            let bytes = bounded(&asset_path, entry.bytes)?;
            let hash: String = theia_network::terrain::sha256(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            if bytes.len() != entry.bytes || hash != entry.sha256 {
                return Err("Scenery asset bytes do not match catalog".into());
            }
            if assets
                .insert(
                    entry.id.clone(),
                    Asset {
                        entry,
                        bytes: Arc::new(bytes),
                    },
                )
                .is_some()
            {
                return Err("Duplicate scenery asset ID".into());
            }
        }
        Ok(Self { assets })
    }

    /// Register before DefaultPlugins; rendering reads the validated snapshot,
    /// never a mutable authoring path or imported Unity/Unreal script.
    pub fn register(&self, app: &mut App) {
        let root = Dir::default();
        for asset in self.assets.values() {
            root.insert_asset(
                Path::new(&asset.entry.file),
                Value::Vec(asset.bytes.clone()),
            );
        }
        app.register_asset_source(
            "scenery",
            AssetSourceBuilder::new(move || Box::new(MemoryAssetReader { root: root.clone() })),
        );
    }

    /// Exercise the actual glTF/PNG loader without Winit, a renderer or OS input.
    pub fn check_headless(&self) -> Result<serde_json::Value, String> {
        use bevy::{
            asset::LoadState,
            gltf::{Gltf, GltfPlugin},
        };
        let mut app = App::new();
        self.register(&mut app);
        app.add_plugins((
            bevy::app::TaskPoolPlugin::default(),
            AssetPlugin::default(),
            bevy::world_serialization::WorldSerializationPlugin,
            bevy::mesh::MeshPlugin,
            bevy::image::ImagePlugin::default(),
            GltfPlugin::default(),
        ));
        app.finish();
        app.cleanup();
        let handles: Vec<(String, Handle<Gltf>)> = self
            .assets
            .values()
            .map(|asset| {
                (
                    asset.entry.id.clone(),
                    app.world()
                        .resource::<AssetServer>()
                        .load(format!("scenery://{}", asset.entry.file)),
                )
            })
            .collect();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            app.update();
            let server = app.world().resource::<AssetServer>();
            for (id, handle) in &handles {
                if let Some(LoadState::Failed(error)) = server.get_load_state(handle.id()) {
                    return Err(format!("Scenery {id} failed to load: {error}"));
                }
            }
            if handles
                .iter()
                .all(|(_, handle)| server.is_loaded_with_dependencies(handle.id()))
            {
                break;
            }
            if std::time::Instant::now() >= deadline {
                return Err("Scenery load timed out".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let gltfs = app.world().resource::<Assets<Gltf>>();
        let mut entries = Vec::new();
        let mut worlds = Vec::new();
        for (id, handle) in &handles {
            let gltf = gltfs.get(handle).ok_or("Missing loaded scenery")?;
            if gltf.scenes.len() != 1 || gltf.meshes.len() != 1 || !gltf.skins.is_empty() {
                return Err("Unexpected static scenery graph".into());
            }
            worlds.push(gltf.scenes[0].clone());
            entries.push(serde_json::json!({"id":id,"meshes":gltf.meshes.len(),"materials":gltf.materials.len()}));
        }
        let roots: Vec<_> = worlds
            .into_iter()
            .map(|handle| {
                app.world_mut()
                    .spawn((WorldAssetRoot(handle), Transform::default()))
                    .id()
            })
            .collect();
        for _ in 0..4 {
            app.update();
        }
        let meshes = app.world_mut().query::<&Mesh3d>().iter(app.world()).count();
        if meshes != roots.len() {
            return Err(format!(
                "Expected {} scene meshes, got {meshes}",
                roots.len()
            ));
        }
        for root in roots {
            app.world_mut().entity_mut(root).despawn();
        }
        for _ in 0..4 {
            app.update();
        }
        let remaining = app.world_mut().query::<&Mesh3d>().iter(app.world()).count();
        if remaining != 0 {
            return Err("Scenery descendants survived root deletion".into());
        }
        Ok(
            serde_json::json!({"instantiated_meshes":meshes,"meshes_after_deletion":remaining,"loaded":true,"assets":entries,"images":app.world().resource::<Assets<Image>>().len(),"scope":"CPU glTF and image loading; no rendering or collision qualification"}),
        )
    }
}
