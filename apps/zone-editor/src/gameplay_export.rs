//! Publish authored areas with an exact validated gameplay-template snapshot.
use std::{fs, path::Path};
use theia_network::gameplay_bundle::{self, Snapshot};
use theia_zone::Zone;

pub(super) struct Draft {
    template: Snapshot,
    manifest: Vec<u8>,
    areas: Vec<u8>,
}

impl Draft {
    pub(super) fn prepare(
        zone: &Zone,
        template: &Path,
        id: &str,
        revision: u32,
    ) -> Result<Self, String> {
        let template = gameplay_bundle::snapshot(template).map_err(|e| e.to_string())?;
        if template.report().manifest_schema < 16 {
            return Err("Gameplay template must support quest areas (schema 16 or newer)".into());
        }
        if template
            .report()
            .profiles
            .iter()
            .any(|r| r.path == "gameplay.manifest.json")
        {
            return Err("Gameplay profile conflicts with the exported manifest filename".into());
        }
        let areas = crate::quest_area_export::prepare(
            zone,
            template
                .profile("quest")
                .ok_or("Gameplay template needs quests")?,
            id,
            revision,
        )?;
        let mut manifest: serde_json::Value =
            serde_json::from_slice(template.manifest()).map_err(|e| e.to_string())?;
        manifest["id"] = serde_json::json!(id);
        manifest["revision"] = serde_json::json!(revision);
        let manifest = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
        Ok(Self {
            template,
            manifest,
            areas,
        })
    }

    /// The caller owns this new staging directory and cleans it on error.
    pub(super) fn write(&self, root: &Path) -> Result<gameplay_bundle::Report, String> {
        fs::create_dir(root).map_err(|e| e.to_string())?;
        for record in &self.template.report().profiles {
            let path = root.join(&record.path);
            fs::create_dir_all(path.parent().ok_or("Missing profile parent")?)
                .map_err(|e| e.to_string())?;
            let bytes = if record.kind == "quest_areas" {
                &self.areas
            } else {
                self.template
                    .profile(&record.kind)
                    .ok_or("Missing validated profile")?
            };
            write_new(&path, bytes)?;
        }
        let manifest = root.join("gameplay.manifest.json");
        write_new(&manifest, &self.manifest)?;
        // Recheck all cross-profile rules after replacing the authored area role.
        gameplay_bundle::validate(&manifest).map_err(|e| e.to_string())
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}
