//! Publish one checked passive population with its freshly exported static world.
//! Navigation stays in the external offline baker, never the editor or server graph.
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use theia_network::{
    authored_world, procedural,
    simulation::{creatures, population},
};
use theia_zone::{Zone, markers::MarkerKind};

pub(super) struct Settings<'a> {
    pub creature: &'a Path,
    pub baker: &'a Path,
    pub spacing: f64,
}

pub(super) fn default_baker() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(executable
        .parent()
        .ok_or("Editor executable has no directory")?
        .join(format!("theia-nav-bake{}", std::env::consts::EXE_SUFFIX)))
}

fn validate(zone: &Zone, profile: &creatures::Profile, spacing: f64) -> Result<u64, String> {
    zone.validate()?;
    let groups: Vec<_> = zone.spawn_groups.iter().filter(|g| g.enabled).collect();
    if groups.len() != 1 {
        return Err("Population export requires exactly one enabled spawn group".into());
    }
    if !zone.quest_areas.is_empty()
        || zone
            .markers
            .iter()
            .any(|m| m.kind == MarkerKind::Npc && m.definition.is_some())
    {
        return Err(
            "Passive population export does not publish quest areas or NPC dialogue contacts"
                .into(),
        );
    }
    if !zone.particles.is_empty() || !zone.test_players.is_empty() {
        return Err(
            "Passive population export does not yet support particles or test-player presets"
                .into(),
        );
    }
    let id = groups[0].id;
    population::authoring::binding(zone, id, profile.definition().clone())
        .map_err(|e| e.to_string())?;
    population::authoring::request(zone, id, spacing, zone.spawn).map_err(|e| e.to_string())?;
    Ok(id)
}

pub(super) fn publish(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
) -> Result<serde_json::Value, String> {
    if output.exists() {
        return Err("Population export requires a new output directory".into());
    }
    let mut creature = Vec::new();
    fs::File::open(settings.creature)
        .map_err(|e| e.to_string())?
        .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut creature)
        .map_err(|e| e.to_string())?;
    let profile = creatures::Profile::from_json(&creature).map_err(|e| e.to_string())?;
    let group = validate(zone, &profile, settings.spacing)?;
    let baker = settings
        .baker
        .canonicalize()
        .map_err(|e| format!("Navigation baker: {e}"))?;
    if !baker.is_file() {
        return Err("Navigation baker must be an executable file".into());
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = parent.join(format!(".population-export-{}-{nonce}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let mut destination_created = false;
    let result = (|| {
        // Only population metadata is excluded from static collision/visual export.
        // Preserve the complete validated authoring document beside that world.
        let mut geometry = zone.clone();
        geometry.spawn_groups.clear();
        let world_report = crate::export::publish_with_assets(
            &geometry,
            &stage.join("world"),
            id,
            revision,
            None,
            scenery,
        )?;
        zone.save(&stage.join("zone.json"))?;
        fs::write(stage.join("creature.json"), &creature).map_err(|e| e.to_string())?;
        let store =
            Arc::new(authored_world::Store::open(&stage.join("world")).map_err(|e| e.to_string())?);
        let access = store.spawn(0).ok_or("Exported world has no player spawn")?;
        let centre = procedural::owner(access).ok_or("Player spawn outside navigation domain")?;
        let baked = Command::new(&baker)
            .arg("bake-authored-area")
            .arg(stage.join("world"))
            .arg(stage.join("navigation"))
            .args([centre.x.to_string(), centre.z.to_string()])
            .output()
            .map_err(|e| format!("Navigation bake: {e}"))?;
        if !baked.status.success() {
            return Err(format!(
                "Population navigation bake failed: {}",
                String::from_utf8_lossy(&baked.stderr)
            ));
        }
        let config = serde_json::json!({"schema":1,"zone":"zone.json","group":group,
            "creature":"creature.json","navigation":"navigation",
            "minimum_spacing_metres":settings.spacing,"access_from":access.metres()});
        fs::write(
            stage.join("population.json"),
            serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let loaded = population::configuration::Loaded::open(
            &stage.join("population.json"),
            &procedural::Source::Authored(store.clone()),
        )
        .map_err(|e| format!("Population placement: {e}"))?;
        let report = serde_json::json!({"population_export":{"passed":true,"group":group,
            "count":loaded.count(),"respawn_ms":loaded.respawn_ms(),"world":world_report,
            "population_identity":loaded.profile_identity(theia_network::targeting::profile_identity(store.identity())),
            "world_directory":output.join("world"),"configuration":output.join("population.json"),
            "scope":"One checked passive population and matching static world/navigation; no server launched, model catalog, pursuit or quest publication"}});
        fs::write(
            stage.join("population-export.json"),
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // Reserve exclusively, and expose the runtime configuration last.
        fs::create_dir(output).map_err(|e| e.to_string())?;
        destination_created = true;
        for name in [
            "world",
            "navigation",
            "zone.json",
            "creature.json",
            "population-export.json",
            "population.json",
        ] {
            fs::rename(stage.join(name), output.join(name)).map_err(|e| e.to_string())?;
        }
        Ok(report)
    })();
    let _ = fs::remove_dir_all(&stage);
    if result.is_err() && destination_created {
        let _ = fs::remove_dir_all(output);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    const CREATURE: &[u8] =
        include_bytes!("../../../content/gameplay/development-creature-patrol.json");
    fn fixture() -> Zone {
        let mut terrain = theia_zone::terrain::HeightPatch::flat([62, 62]);
        terrain.span_chunks = 4;
        terrain.heights = vec![0.; terrain.axis() * terrain.axis()];
        Zone {
            schema: 14,
            origin: [8192., 0., 8192.],
            spawn: [8192., 0.1, 8192.],
            terrain: Some(terrain),
            spawn_groups: vec![theia_zone::spawn_groups::SpawnGroup {
                id: 1,
                name: "Grove".into(),
                enabled: true,
                creature: "theia.dev.training-creature".into(),
                model: None,
                target_alive: 5,
                respawn_seconds: 3,
                seed: 42,
                volume: theia_core::spawn_volume::PolygonVolume::new(
                    vec![
                        [8184., 8184.],
                        [8200., 8184.],
                        [8200., 8200.],
                        [8184., 8200.],
                    ],
                    -2.,
                    8.,
                )
                .unwrap(),
            }],
            ..Default::default()
        }
    }
    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "theia-population-export-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn passive_export_refuses_ambiguous_or_unsupported_group_policies() {
        let profile = creatures::Profile::from_json(CREATURE).unwrap();
        let zone = fixture();
        assert_eq!(validate(&zone, &profile, 3.).unwrap(), 1);
        for spacing in [0., -1., f64::NAN, f64::INFINITY] {
            assert!(validate(&zone, &profile, spacing).is_err());
        }
        let mut changed = zone.clone();
        changed.spawn_groups[0].enabled = false;
        assert!(validate(&changed, &profile, 3.).is_err());
        let mut changed = zone.clone();
        let mut group = changed.spawn_groups[0].clone();
        group.id = 2;
        changed.spawn_groups.push(group);
        assert!(validate(&changed, &profile, 3.).is_err());
        changed.spawn_groups[1].enabled = false;
        assert!(validate(&changed, &profile, 3.).is_ok());
        for count in [17, 64] {
            changed.spawn_groups[0].target_alive = count;
            assert!(validate(&changed, &profile, 3.).is_err());
        }
        let mut changed = zone.clone();
        changed.spawn_groups[0].model = Some("theia.model.wolf".into());
        assert!(validate(&changed, &profile, 3.).is_err());
        let aggressive = creatures::Profile::from_json(include_bytes!(
            "../../../content/gameplay/development-creature-pursuit.json"
        ))
        .unwrap();
        assert!(validate(&zone, &aggressive, 3.).is_err());
    }
    #[test]
    #[cfg(unix)]
    fn failed_bake_and_existing_destination_preserve_sources_and_leave_no_partial_package() {
        let root = Folder::new();
        let creature = root.0.join("profile.json");
        fs::write(&creature, CREATURE).unwrap();
        let zone = fixture();
        let original = zone.clone();
        let output = root.0.join("package");
        let settings = || Settings {
            creature: &creature,
            baker: Path::new("/bin/false"),
            spacing: 3.,
        };
        assert!(
            publish(
                &zone,
                &output,
                "theia.test.population",
                1,
                settings(),
                &Default::default()
            )
            .unwrap_err()
            .contains("navigation bake failed")
        );
        assert!(!output.exists());
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
        fs::create_dir(&output).unwrap();
        fs::write(output.join("keep"), b"keep").unwrap();
        assert!(
            publish(
                &zone,
                &output,
                "theia.test.population",
                1,
                settings(),
                &Default::default()
            )
            .is_err()
        );
        assert_eq!(fs::read(output.join("keep")).unwrap(), b"keep");
        assert_eq!(fs::read(&creature).unwrap(), CREATURE);
        assert_eq!(zone, original);
    }
    #[test]
    #[ignore = "requires THEIA_NAV_BAKER actual offline binary"]
    fn actual_population_package_reloads_matching_world_navigation_and_saved_group() {
        let root = Folder::new();
        let creature = root.0.join("profile.json");
        fs::write(&creature, CREATURE).unwrap();
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let zone = fixture();
        let original = zone.clone();
        let output = root.0.join("package");
        let report = publish(
            &zone,
            &output,
            "theia.test.population",
            1,
            Settings {
                creature: &creature,
                baker: &baker,
                spacing: 3.,
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(report["population_export"]["count"], 5);
        assert_eq!(report["population_export"]["respawn_ms"], 3000);
        assert_eq!(Zone::load(&output.join("zone.json")).unwrap(), zone);
        let mut geometry = zone.clone();
        geometry.spawn_groups.clear();
        assert_eq!(
            Zone::load(&output.join("world/authoring.zone.json")).unwrap(),
            geometry
        );
        let source = procedural::Source::Authored(Arc::new(
            authored_world::Store::open(&output.join("world")).unwrap(),
        ));
        let loaded =
            population::configuration::Loaded::open(&output.join("population.json"), &source)
                .unwrap();
        assert_eq!(loaded.count(), 5);
        assert_eq!(loaded.respawn_ms(), 3000);
        // Valid authoring inputs can still fail actual placement. Do not publish
        // a package when five actors cannot fit with the selected 32 m spacing.
        let rejected = root.0.join("insufficient-placement");
        let error = publish(
            &zone,
            &rejected,
            "theia.test.population",
            1,
            Settings {
                creature: &creature,
                baker: &baker,
                spacing: 32.,
            },
            &Default::default(),
        )
        .unwrap_err();
        assert!(error.contains("Population placement"), "{error}");
        assert!(!rejected.exists());
        assert_eq!(zone, original);
        assert_eq!(fs::read(&creature).unwrap(), CREATURE);
        assert!(!fs::read_dir(&root.0).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".population-export-")
        }));
    }
}
