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

const LIFE_FILES: [(&str, &[u8]); 3] = [
    (
        "character-stats.json",
        include_bytes!("../../../content/gameplay/development-character-stats.json"),
    ),
    (
        "retaliation.json",
        include_bytes!("../../../content/gameplay/development-creature-retaliation.json"),
    ),
    (
        "player-respawn.json",
        include_bytes!("../../../content/gameplay/development-player-respawn.json"),
    ),
];

pub(super) struct Settings<'a> {
    pub creature: &'a Path,
    pub baker: &'a Path,
    pub spacing: f64,
    pub appearance: Option<&'a Path>,
}

pub(super) fn default_baker() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(executable
        .parent()
        .ok_or("Editor executable has no directory")?
        .join(format!("theia-nav-bake{}", std::env::consts::EXE_SUFFIX)))
}

fn validate(zone: &Zone, profile: &creatures::Definition, spacing: f64) -> Result<u64, String> {
    validate_mode(zone, profile, spacing, false)
}

fn validate_mode(
    zone: &Zone,
    profile: &creatures::Definition,
    spacing: f64,
    network: bool,
) -> Result<u64, String> {
    zone.validate()?;
    if network && !zone.test_players.iter().any(|p| p.enabled) {
        return Err(
            "Enable at least one test player before exporting a population playtest".into(),
        );
    }
    if network {
        crate::network_test_export::validate_controllers(zone, true)?;
    }
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
    if !network && !zone.test_players.is_empty() {
        return Err(
            "Passive population export requires network mode for test-player presets".into(),
        );
    }
    let id = groups[0].id;
    population::authoring::binding(zone, id, profile.clone()).map_err(|e| e.to_string())?;
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
    publish_mode(zone, output, id, revision, settings, scenery, false)
}

pub(super) fn publish_network(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
) -> Result<serde_json::Value, String> {
    publish_mode(zone, output, id, revision, settings, scenery, true)
}

fn publish_mode(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
    network: bool,
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
    let profile = creatures::Definition::from_json(&creature).map_err(|e| e.to_string())?;
    let group = if network {
        validate_mode(zone, &profile, settings.spacing, true)?
    } else {
        validate(zone, &profile, settings.spacing)?
    };
    let appearance = settings
        .appearance
        .map(crate::creature_asset::read)
        .transpose()?;
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
        // Exclude runtime population and optional test-player metadata from static export.
        // Preserve the complete validated authoring document beside that world.
        let mut geometry = zone.clone();
        geometry.spawn_groups.clear();
        if network {
            geometry.test_players.clear();
        }
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
        if let Some(bytes) = &appearance {
            fs::write(stage.join("creature.glb"), bytes).map_err(|e| e.to_string())?;
        }
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
        let clients = if network {
            let clients = crate::network_test_export::write_scenario(zone, &store, &stage)?;
            fs::write(
                stage.join("combat.json"),
                include_bytes!("../../../content/gameplay/development-abilities.json"),
            )
            .map_err(|e| e.to_string())?;
            theia_network::player_life_wire::Profile::from_json(
                LIFE_FILES[0].1,
                loaded.creature_bytes(),
                LIFE_FILES[1].1,
                LIFE_FILES[2].1,
            )
            .map_err(|e| format!("Population player life: {e}"))?;
            for (name, bytes) in LIFE_FILES {
                fs::write(stage.join(name), bytes).map_err(|e| e.to_string())?;
            }
            clients
        } else {
            0
        };
        let report = serde_json::json!({"population_export":{"passed":true,"group":group,
            "appearance":appearance.as_ref().map(|bytes|serde_json::json!({"file":"creature.glb","sha256":theia_network::terrain::sha256(bytes)})),
            "player_life":network,
            "count":loaded.count(),"clients":clients,"respawn_ms":loaded.respawn_ms(),"world":world_report,
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
        if network {
            for name in [
                "scenario.json",
                "combat.json",
                "controllers",
                "routes",
                "character-stats.json",
                "retaliation.json",
                "player-respawn.json",
            ] {
                if stage.join(name).exists() {
                    fs::rename(stage.join(name), output.join(name)).map_err(|e| e.to_string())?;
                }
            }
        }
        if appearance.is_some() {
            fs::rename(stage.join("creature.glb"), output.join("creature.glb"))
                .map_err(|e| e.to_string())?;
        }
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
        include_bytes!("../../../content/gameplay/development-creature-passive.json");
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
        let profile = creatures::Definition::from_json(CREATURE).unwrap();
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
        let aggressive = creatures::Definition::from_json(include_bytes!(
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
            appearance: None,
        };
        let bad_appearance = root.0.join("invalid.glb");
        fs::write(&bad_appearance, b"Not a GLB").unwrap();
        let error = publish(
            &zone,
            &output,
            "theia.test.population",
            1,
            Settings {
                appearance: Some(&bad_appearance),
                ..settings()
            },
            &Default::default(),
        )
        .unwrap_err();
        assert!(error.contains("GLB"), "{error}");
        assert!(!output.exists());
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
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), 2); // Only the two input fixtures remain.
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
    fn population_network_snapshot_preserves_players_and_binds_scenario_to_its_world() {
        let root = Folder::new();
        let creature = root.0.join("profile.json");
        fs::write(&creature, CREATURE).unwrap();
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let mut zone = fixture();
        zone.test_players = (1..=2)
            .map(|slot| theia_zone::test_players::TestPlayer {
                id: 10 + u64::from(slot),
                name: format!("Player {slot}"),
                client_id: slot,
                enabled: true,
                position: [8192. + f64::from(slot), 0.1, 8192.],
                controller: (slot == 2).then(|| "test.walk-turn".into()),
                network: Default::default(),
            })
            .collect();
        zone.schema = zone.schema.max(13);
        zone.paths.push(theia_zone::paths::GuidePath {
            id: 30,
            name: "Client approach".into(),
            kind: theia_zone::paths::PathKind::Patrol,
            width_metres: 2.,
            points: vec![
                [8193., 0.008, 8192.],
                [8193., 0.008, 8186.],
                [8200., 0.008, 8186.],
            ],
        });
        zone.test_players[0].controller = Some("theia.zone.patrol.30".into());
        let appearance = std::env::var_os("THEIA_CREATURE_ASSET").map(|source| {
            let copy = root.0.join("appearance.glb");
            fs::copy(source, &copy).unwrap();
            copy
        });
        let original = zone.clone();
        let output = root.0.join("package");
        let settings = || Settings {
            creature: &creature,
            baker: &baker,
            spacing: 3.,
            appearance: appearance.as_deref(),
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
            .is_err()
        );
        assert!(!output.exists());
        let report = publish_network(
            &zone,
            &output,
            "theia.test.population",
            1,
            settings(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(report["population_export"]["clients"], 2);
        assert_eq!(report["population_export"]["player_life"], true);
        for (name, bytes) in LIFE_FILES {
            assert_eq!(fs::read(output.join(name)).unwrap(), bytes);
        }
        assert_eq!(report["population_export"]["count"], 5);
        assert_eq!(Zone::load(&output.join("zone.json")).unwrap(), zone);
        let store = Arc::new(authored_world::Store::open(&output.join("world")).unwrap());
        let scenario: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("scenario.json")).unwrap()).unwrap();
        assert_eq!(
            scenario["world_identity"],
            serde_json::json!(store.identity())
        );
        assert_eq!(
            scenario["players"][0]["feet"],
            serde_json::json!(zone.test_players[0].position)
        );
        assert_eq!(
            scenario["players"][1]["controller"],
            "controllers/walk-turn.json"
        );
        assert!(scenario["players"][0]["controller"].is_null());
        assert_eq!(scenario["players"][0]["route"], "routes/player-1.json");
        let route: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("routes/player-1.json")).unwrap())
                .unwrap();
        assert_eq!(route["navigation"], "../navigation");
        assert_eq!(route["return_trip"], true);
        assert!(route["waypoints"].as_array().unwrap().len() > 3);
        assert_eq!(
            fs::read(output.join("controllers/walk-turn.json")).unwrap(),
            include_bytes!("../../../content/testing/walk-turn-controller.json")
        );
        assert_eq!(
            fs::read(output.join("combat.json")).unwrap(),
            include_bytes!("../../../content/gameplay/development-abilities.json")
        );
        let loaded = population::configuration::Loaded::open(
            &output.join("population.json"),
            &procedural::Source::Authored(store),
        )
        .unwrap();
        assert_eq!(loaded.count(), 5);
        assert_eq!(zone, original);
        if let Some(source) = &appearance {
            let original = fs::read(source).unwrap();
            assert_eq!(fs::read(output.join("creature.glb")).unwrap(), original);
            assert_eq!(
                report["population_export"]["appearance"]["file"],
                "creature.glb"
            );
            fs::write(source, b"Changed appearance source").unwrap();
            assert_eq!(fs::read(output.join("creature.glb")).unwrap(), original);
            fs::write(source, original).unwrap();
        }
        let route_bytes = fs::read(output.join("routes/player-1.json")).unwrap();
        let mut wrong_floor = zone.clone();
        wrong_floor.paths[0].points[1][1] += 20.;
        let refused = root.0.join("wrong-floor");
        let error = publish_network(
            &wrong_floor,
            &refused,
            "theia.test.population",
            2,
            settings(),
            &Default::default(),
        )
        .unwrap_err();
        assert!(error.contains("patrol path"), "{error}");
        assert!(!refused.exists());
        assert_eq!(
            fs::read(output.join("routes/player-1.json")).unwrap(),
            route_bytes
        );
    }
    #[test]
    #[ignore = "requires THEIA_NAV_BAKER actual offline binary"]
    fn world_edge_population_omits_empty_neighbours_and_reloads() {
        let root = Folder::new();
        let creature = root.0.join("profile.json");
        fs::write(&creature, CREATURE).unwrap();
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let mut zone = fixture();
        zone.terrain = Some(theia_zone::terrain::HeightPatch::flat([67, 67]));
        zone.spawn = [8640., 0.1, 8640.];
        zone.spawn_groups[0].volume = theia_core::spawn_volume::PolygonVolume::new(
            vec![
                [8632., 8632.],
                [8648., 8632.],
                [8648., 8648.],
                [8632., 8648.],
            ],
            -2.,
            8.,
        )
        .unwrap();
        let original = zone.clone();
        let output = root.0.join("package");
        let report = publish(
            &zone,
            &output,
            "theia.test.edge-population",
            1,
            Settings {
                creature: &creature,
                baker: &baker,
                spacing: 3.,
                appearance: None,
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(report["population_export"]["count"], 5);
        let store = Arc::new(authored_world::Store::open(&output.join("world")).unwrap());
        let nav =
            theia_network::navigation::Store::open(&output.join("navigation"), store.identity())
                .unwrap();
        // Four ground tiles plus the actual collision-envelope corner top.
        // Preserve that separate high layer; placement must remain on reachable ground.
        assert_eq!(
            nav.manifest()
                .tiles
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![[66, 66], [66, 67], [67, 66], [67, 67], [68, 68]]
        );
        let corner = nav.tile([68, 68]).unwrap();
        assert!(corner.vertices.iter().all(|p| p[1] >= 18.));
        let empty = root.0.join("empty-area");
        let failed = std::process::Command::new(&baker)
            .arg("bake-authored-area")
            .arg(output.join("world"))
            .arg(&empty)
            .args(["70", "70"])
            .output()
            .unwrap();
        assert!(!failed.status.success());
        assert!(String::from_utf8_lossy(&failed.stderr).contains("outside authored world"));
        assert!(!empty.exists());
        let loaded = population::configuration::Loaded::open(
            &output.join("population.json"),
            &procedural::Source::Authored(store),
        )
        .unwrap();
        assert_eq!(loaded.count(), 5);
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
                appearance: None,
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
                appearance: None,
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
