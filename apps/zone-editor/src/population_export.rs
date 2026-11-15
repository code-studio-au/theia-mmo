//! Publish one checked population with its freshly exported static world.
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
    pub gameplay: Option<&'a Path>,
    pub access_path: Option<u64>,
    pub test_path: Option<u64>,
}

pub(super) fn default_baker() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(executable
        .parent()
        .ok_or("Editor executable has no directory")?
        .join(format!("theia-nav-bake{}", std::env::consts::EXE_SUFFIX)))
}

fn validate(zone: &Zone, profile: &creatures::Definition, spacing: f64) -> Result<u64, String> {
    validate_mode(zone, profile, spacing, false, false, false)
}

fn validate_mode(
    zone: &Zone,
    profile: &creatures::Definition,
    spacing: f64,
    network: bool,
    account: bool,
    gameplay: bool,
) -> Result<u64, String> {
    zone.validate()?;
    if network && !zone.test_players.iter().any(|p| p.enabled) {
        return Err(
            "Enable at least one test player before exporting a population playtest".into(),
        );
    }
    if network {
        if gameplay {
            return Err(
                "Population quest snapshots require an account launch, not Test Player mode".into(),
            );
        }
        crate::network_test_export::validate_controllers(zone, true)?;
    }
    let groups: Vec<_> = zone.spawn_groups.iter().filter(|g| g.enabled).collect();
    if groups.len() != 1 {
        return Err("Population export requires exactly one enabled spawn group".into());
    }
    if !gameplay
        && (!zone.quest_areas.is_empty()
            || zone
                .markers
                .iter()
                .any(|m| m.kind == MarkerKind::Npc && m.definition.is_some()))
    {
        return Err(
            "Population quest areas and NPC dialogue contacts require an explicit gameplay template"
                .into(),
        );
    }
    if !network && !account && !zone.test_players.is_empty() {
        return Err("Population export requires network mode for test-player presets".into());
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
    publish_mode(zone, output, id, revision, settings, scenery, false, false)
}

pub(super) fn publish_network(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
) -> Result<serde_json::Value, String> {
    publish_mode(zone, output, id, revision, settings, scenery, true, false)
}

pub(super) fn publish_account(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
) -> Result<serde_json::Value, String> {
    publish_mode(zone, output, id, revision, settings, scenery, false, true)
}

#[allow(clippy::too_many_arguments)]
fn publish_mode(
    zone: &Zone,
    output: &Path,
    id: &str,
    revision: u32,
    settings: Settings<'_>,
    scenery: &crate::scenery_library::Library,
    network: bool,
    account: bool,
) -> Result<serde_json::Value, String> {
    if output.exists() {
        return Err("Population export requires a new output directory".into());
    }
    if let Some(id) = settings.test_path {
        if !account {
            return Err("Paused AI test roads require an account playtest export".into());
        }
        crate::account_test_route::road(zone, id)?;
    }
    let mut creature = Vec::new();
    fs::File::open(settings.creature)
        .map_err(|e| e.to_string())?
        .take(creatures::MAX_PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut creature)
        .map_err(|e| e.to_string())?;
    let profile = creatures::Definition::from_json(&creature).map_err(|e| e.to_string())?;
    let live = network || account;
    let group = if live || settings.gameplay.is_some() {
        validate_mode(
            zone,
            &profile,
            settings.spacing,
            network,
            account,
            settings.gameplay.is_some(),
        )?
    } else {
        validate(zone, &profile, settings.spacing)?
    };
    if let Some(path_id) = settings.access_path {
        population::authoring::access_waypoints(zone, group, path_id, zone.spawn)
            .map_err(|e| e.to_string())?;
    }
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
        if live {
            geometry.test_players.clear();
        }
        let world_report = crate::export::publish_with_assets(
            &geometry,
            &stage.join("world"),
            id,
            revision,
            settings.gameplay,
            scenery,
        )?;
        let gameplay_manifest = if settings.gameplay.is_some() {
            // Inspect the exact snapshot actually written, not a second read of
            // mutable template inputs. Reject mismatched kill/reward definitions
            // before baking navigation or exposing the outer package.
            let manifest = stage.join("world/gameplay/gameplay.manifest.json");
            let snapshot =
                theia_network::gameplay_bundle::snapshot(&manifest).map_err(|e| e.to_string())?;
            let declared = creatures::Profile::from_json(
                snapshot
                    .profile("creature")
                    .ok_or("Population gameplay snapshot needs a creature profile")?,
            )
            .map_err(|e| e.to_string())?;
            if declared.id() != profile.id() {
                return Err("Population creature ID differs from the gameplay template".into());
            }
            Some("world/gameplay/gameplay.manifest.json")
        } else {
            None
        };
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
        let mut config = serde_json::json!({"schema":if settings.access_path.is_some() {2} else {1},"zone":"zone.json","group":group,
            "creature":"creature.json","navigation":"navigation",
            "minimum_spacing_metres":settings.spacing,"access_from":access.metres()});
        if let Some(path_id) = settings.access_path {
            config["access_path"] = path_id.into();
        }
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
        let (quest_travel, ground_items) = if let Some(manifest) = gameplay_manifest {
            let snapshot = theia_network::gameplay_bundle::snapshot(&stage.join(manifest))
                .map_err(|e| e.to_string())?;
            let travel = crate::quest_travel_export::prepare(
                zone,
                snapshot
                    .profile("quest")
                    .ok_or("Gameplay snapshot needs quests")?,
                &store,
                loaded.navigation(),
            )?;
            (
                travel,
                crate::ground_item_export::prepare(&snapshot, &store)?,
            )
        } else {
            (Vec::new(), Vec::new())
        };
        let test_travel = settings
            .test_path
            .map(|id| {
                let (bytes, report) =
                    crate::account_test_route::prepare(zone, id, &store, loaded.navigation())?;
                fs::write(stage.join("test-route.json"), bytes).map_err(|e| e.to_string())?;
                Ok::<_, String>(report)
            })
            .transpose()?;
        let clients = if network {
            crate::network_test_export::write_scenario(zone, &store, &stage)?
        } else {
            usize::from(account)
        };
        if live {
            fs::write(
                stage.join("combat.json"),
                include_bytes!("../../../content/gameplay/development-resource-abilities.json"),
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
        }
        let report = serde_json::json!({"population_export":{"passed":true,"group":group,
            "appearance":appearance.as_ref().map(|bytes|serde_json::json!({"file":"creature.glb","sha256":theia_network::terrain::sha256(bytes)})),
            "player_life":live,"account_playtest":account,
            "count":loaded.count(),"clients":clients,"respawn_ms":loaded.respawn_ms(),"world":world_report,
            "population_identity":loaded.profile_identity(theia_network::targeting::profile_identity(store.identity())),
            "world_directory":output.join("world"),"configuration":output.join("population.json"),
            "gameplay_manifest":gameplay_manifest,"access_path":settings.access_path,"quest_travel":quest_travel,"ground_items":ground_items,"test_travel":test_travel,
            "scope":"One checked population with configured creature behavior and matching static world/navigation; optional explicit quest/contact snapshot, no server launched; population definition controls NPC behavior, template patrol is not instantiated"}});
        fs::write(
            stage.join("population-export.json"),
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // Reserve exclusively, and expose the runtime configuration last.
        fs::create_dir(output).map_err(|e| e.to_string())?;
        destination_created = true;
        if test_travel.is_some() {
            fs::rename(
                stage.join("test-route.json"),
                output.join("test-route.json"),
            )
            .map_err(|e| e.to_string())?;
        }
        if live {
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
                roam_radius_metres: 0.,
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
    fn account_export_does_not_require_enabled_test_players() {
        let zone = fixture();
        let profile = creatures::Definition::from_json(CREATURE).unwrap();
        assert!(validate_mode(&zone, &profile, 3., true, false, false).is_err());
        assert_eq!(
            validate_mode(&zone, &profile, 3., false, true, false).unwrap(),
            1
        );
    }
    fn quest_fixture() -> Zone {
        let mut zone = fixture();
        zone.markers.push(theia_zone::markers::Marker {
            id: 20,
            name: "Guide".into(),
            kind: MarkerKind::Npc,
            position: [8193., 0.008, 8191.],
            yaw_degrees: 0.,
            radius_metres: 1.,
            definition: Some("theia.dev.guide".into()),
        });
        zone.quest_areas.push(theia_zone::quest_areas::QuestArea {
            id: 21,
            definition: "theia.dev.area.western-overlook".into(),
            name: "Overlook".into(),
            min: [8180., -2., 8180.],
            max: [8184., 4., 8184.],
            map_objectives: vec!["theia.dev.quest-overlook.objective.reach".into()],
        });
        zone
    }
    fn quest_template() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/development-gameplay-classic-quest-loot-fixture.json")
    }
    #[test]
    #[ignore = "requires THEIA_QUEST_TRAVEL_PACKAGE, THEIA_SCENERY_LIBRARY and THEIA_NAV_BAKER"]
    fn authored_ground_item_failure_never_publishes_partial_population() {
        let root = Folder::new();
        let package = PathBuf::from(std::env::var_os("THEIA_QUEST_TRAVEL_PACKAGE").unwrap());
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let library = crate::scenery_library::Library::load(&PathBuf::from(
            std::env::var_os("THEIA_SCENERY_LIBRARY").unwrap(),
        ))
        .unwrap();
        let zone = Zone::load(&package.join("zone.json")).unwrap();
        let original = zone.clone();
        let snapshot = theia_network::gameplay_bundle::snapshot(
            &package.join("world/gameplay/gameplay.manifest.json"),
        )
        .unwrap();
        let copied = root.0.join("gameplay");
        fs::create_dir(&copied).unwrap();
        let manifest = copied.join("gameplay.manifest.json");
        fs::write(&manifest, snapshot.manifest()).unwrap();
        for record in &snapshot.report().profiles {
            let path = copied.join(&record.path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, snapshot.profile(&record.kind).unwrap()).unwrap();
        }
        let rewards = copied.join(
            &snapshot
                .report()
                .profiles
                .iter()
                .find(|p| p.kind == "rewards")
                .unwrap()
                .path,
        );
        let original_rewards = fs::read(&rewards).unwrap();
        let mut bad: serde_json::Value = serde_json::from_slice(&original_rewards).unwrap();
        let ground = bad["rewards"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r.get("ground").is_some())
            .unwrap();
        ground["ground"]["position"][1] =
            (ground["ground"]["position"][1].as_f64().unwrap() + 8.).into();
        fs::write(&rewards, serde_json::to_vec(&bad).unwrap()).unwrap();
        let creature = package.join("creature.json");
        let output = root.0.join("package");
        let publish = || {
            publish_account(
                &zone,
                &output,
                "theia.test.ground-preflight",
                1,
                Settings {
                    creature: &creature,
                    baker: &baker,
                    spacing: 3.,
                    appearance: None,
                    gameplay: Some(&manifest),
                    access_path: Some(1201),
                    test_path: None,
                },
                &library,
            )
        };
        let error = publish().unwrap_err();
        assert!(
            error.contains("theia.dev.compass-ground") && error.contains("Northern Meadow"),
            "{error}"
        );
        assert!(!output.exists());
        assert_eq!(
            fs::read_dir(&root.0).unwrap().count(),
            1,
            "Staging directory leaked"
        );
        fs::write(&rewards, &original_rewards).unwrap();
        let report = publish().unwrap();
        assert_eq!(
            report["population_export"]["ground_items"][0]["reward"],
            "theia.dev.compass-ground"
        );
        assert!(output.join("population.json").is_file());
        assert_eq!(zone, original);
        assert_eq!(original_rewards, fs::read(&rewards).unwrap());
        println!("{}", report["population_export"]["ground_items"]);
    }
    #[test]
    fn quest_metadata_requires_explicit_template_and_never_enters_test_player_mode() {
        let zone = quest_fixture();
        let profile = creatures::Definition::from_json(CREATURE).unwrap();
        assert!(validate_mode(&zone, &profile, 3., false, true, false).is_err());
        assert!(validate_mode(&zone, &profile, 3., false, true, true).is_ok());
        assert!(validate_mode(&zone, &profile, 3., true, false, true).is_err());
    }
    #[test]
    #[cfg(unix)]
    fn mismatched_quest_contact_or_creature_leaves_no_published_population() {
        for contact in [true, false] {
            let root = Folder::new();
            let creature = root.0.join("creature.json");
            let mut definition: serde_json::Value = serde_json::from_slice(CREATURE).unwrap();
            let mut zone = quest_fixture();
            if contact {
                zone.markers[0].definition = Some("theia.dev.other-guide".into());
            } else {
                definition["id"] = "theia.dev.other-creature".into();
                zone.spawn_groups[0].creature = "theia.dev.other-creature".into();
            }
            fs::write(&creature, serde_json::to_vec(&definition).unwrap()).unwrap();
            let output = root.0.join("package");
            let original = zone.clone();
            let error = publish_account(
                &zone,
                &output,
                "theia.test.quest-population",
                1,
                Settings {
                    creature: &creature,
                    baker: Path::new("/bin/false"),
                    spacing: 3.,
                    appearance: None,
                    gameplay: Some(&quest_template()),
                    access_path: None,
                    test_path: None,
                },
                &Default::default(),
            )
            .unwrap_err();
            assert!(
                error.contains(if contact {
                    "Unknown or duplicate NPC contact"
                } else {
                    "creature ID differs"
                }),
                "{error}"
            );
            assert!(!output.exists());
            assert_eq!(zone, original);
            assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
        }
    }
    #[test]
    #[ignore = "requires THEIA_NAV_BAKER actual offline binary"]
    fn population_quest_snapshot_binds_contact_areas_and_exact_profiles_before_navigation() {
        let root = Folder::new();
        let creature = root.0.join("creature.json");
        fs::write(&creature, CREATURE).unwrap();
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let zone = quest_fixture();
        let original = zone.clone();
        let template = quest_template();
        let source = theia_network::gameplay_bundle::snapshot(&template).unwrap();
        let output = root.0.join("package");
        let report = publish_account(
            &zone,
            &output,
            "theia.test.quest-population",
            2,
            Settings {
                creature: &creature,
                baker: &baker,
                spacing: 3.,
                appearance: None,
                gameplay: Some(&template),
                access_path: None,
                test_path: None,
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            report["population_export"]["gameplay_manifest"],
            "world/gameplay/gameplay.manifest.json"
        );
        let saved = theia_network::gameplay_bundle::snapshot(
            &output.join("world/gameplay/gameplay.manifest.json"),
        )
        .unwrap();
        for role in ["quest", "dialogue", "items", "rewards", "progression"] {
            assert_eq!(saved.profile(role), source.profile(role));
        }
        let areas: serde_json::Value =
            serde_json::from_slice(saved.profile("quest_areas").unwrap()).unwrap();
        assert_eq!(
            areas["areas"][0]["min"],
            serde_json::json!(zone.quest_areas[0].min)
        );
        assert_eq!(
            areas["areas"][0]["max"],
            serde_json::json!(zone.quest_areas[0].max)
        );
        let store = Arc::new(authored_world::Store::open(&output.join("world")).unwrap());
        let dialogue =
            theia_network::dialogue::Profile::from_json(saved.profile("dialogue").unwrap())
                .unwrap();
        let theia_network::dialogue::Placement::Feet(feet) =
            store.contact_placement(&dialogue).unwrap()
        else {
            panic!("Expected authored feet")
        };
        assert_eq!(feet.metres(), zone.markers[0].position);
        let loaded = population::configuration::Loaded::open(
            &output.join("population.json"),
            &procedural::Source::Authored(store),
        )
        .unwrap();
        assert_eq!(loaded.count(), 5);
        assert_eq!(Zone::load(&output.join("zone.json")).unwrap(), original);
        assert_eq!(zone, original);
    }
    #[test]
    #[ignore = "requires THEIA_NAV_BAKER actual offline binary"]
    fn account_population_snapshot_keeps_author_spawn_and_omits_test_controllers() {
        let root = Folder::new();
        let creature = root.0.join("profile.json");
        fs::write(&creature, CREATURE).unwrap();
        let baker = PathBuf::from(std::env::var_os("THEIA_NAV_BAKER").unwrap());
        let mut zone = fixture();
        zone.test_players
            .push(theia_zone::test_players::TestPlayer {
                id: 10,
                name: "Disabled preset".into(),
                client_id: 1,
                enabled: false,
                position: [8195., 0.1, 8195.],
                controller: Some("test.walk-turn".into()),
                network: Default::default(),
            });
        let original = zone.clone();
        let output = root.0.join("account");
        let report = publish_account(
            &zone,
            &output,
            "theia.test.account-population",
            1,
            Settings {
                creature: &creature,
                baker: &baker,
                spacing: 3.,
                appearance: None,
                gameplay: None,
                access_path: None,
                test_path: None,
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(report["population_export"]["account_playtest"], true);
        assert_eq!(report["population_export"]["clients"], 1);
        assert_eq!(report["population_export"]["player_life"], true);
        assert!(!output.join("scenario.json").exists());
        assert!(!output.join("controllers").exists());
        assert_eq!(Zone::load(&output.join("zone.json")).unwrap(), original);
        let world = authored_world::Store::open(&output.join("world")).unwrap();
        let spawn = world.spawn(0).unwrap().metres();
        assert_eq!([spawn[0], spawn[2]], [zone.spawn[0], zone.spawn[2]]);
        assert!((spawn[1] - 0.008).abs() < 1e-6);
        assert_eq!(zone, original);
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
        assert!(validate(&zone, &aggressive, 3.).is_ok());
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
            gameplay: None,
            access_path: None,
            test_path: None,
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
            gameplay: None,
            access_path: None,
            test_path: None,
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
            include_bytes!("../../../content/gameplay/development-resource-abilities.json")
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
                gameplay: None,
                access_path: None,
                test_path: None,
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
                gameplay: None,
                access_path: None,
                test_path: None,
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
        // A selected saved road is checked again at runtime, not trusted because
        // an editor exported it. The access request also participates in identity.
        let mut with_road = zone.clone();
        with_road.paths.push(theia_zone::paths::GuidePath {
            id: 2,
            name: "Population access".into(),
            kind: theia_zone::paths::PathKind::Road,
            width_metres: 1.,
            points: vec![[8192., 0., 8192.], [8194., 0., 8192.], [8196., 0., 8192.]],
        });
        with_road.save(&output.join("zone.json")).unwrap();
        let config_path = output.join("population.json");
        let mut config: serde_json::Value =
            serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
        config["schema"] = 2.into();
        config["access_path"] = 2.into();
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        let checked = population::configuration::Loaded::open(&config_path, &source).unwrap();
        assert_eq!(checked.count(), 5);
        assert_ne!(
            checked.profile_identity([0; 32]),
            loaded.profile_identity([0; 32])
        );
        let checked_identity = checked.profile_identity([0; 32]);
        with_road.paths[0].points[1][2] += 1.;
        with_road.save(&output.join("zone.json")).unwrap();
        let alternate = population::configuration::Loaded::open(&config_path, &source).unwrap();
        assert_ne!(alternate.profile_identity([0; 32]), checked_identity);
        let (prepared, binding) = checked.into_parts();
        assert_eq!(
            checked_identity,
            binding.profile_identity(prepared.profile_identity([0; 32]))
        );
        // Same endpoint and population placements; only the requested access
        // route differs, so peers cannot silently disagree about the road.
        assert_eq!(alternate.into_parts().0.points(), prepared.points());
        with_road.paths[0].points[1][2] -= 1.;
        with_road.paths[0].points[1][1] = 5.;
        with_road.save(&output.join("zone.json")).unwrap();
        assert!(population::configuration::Loaded::open(&config_path, &source).is_err());
        with_road.paths[0].points[1][1] = 0.;
        with_road.save(&output.join("zone.json")).unwrap();
        config["schema"] = 1.into();
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        assert!(population::configuration::Loaded::open(&config_path, &source).is_err());
        config["schema"] = 2.into();
        config.as_object_mut().unwrap().remove("access_path");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        assert!(population::configuration::Loaded::open(&config_path, &source).is_err());
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
                gameplay: None,
                access_path: None,
                test_path: None,
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
