//! Immutable local playtest snapshot for the managed multi-client launcher.
use super::*;
use std::{fs, path::Path};
use theia_core::coordinates::GlobalPosition;
use theia_network::{authored_world::Store, procedural};

const WALK_TURN: &[u8] = include_bytes!("../../../content/testing/walk-turn-controller.json");
const WALK_TURN_PATH: &str = "controllers/walk-turn.json";

fn controller(id: Option<&str>) -> Result<Option<&'static str>, String> {
    match id {
        None => Ok(None),
        Some("test.walk-turn") => Ok(Some(WALK_TURN_PATH)),
        Some(id) => Err(format!("Unknown test controller: {id}")),
    }
}

pub(super) fn publish(
    zone: &Zone,
    output: &Path,
    content_id: &str,
    revision: u32,
    template: Option<&Path>,
    scenery: &scenery_library::Library,
) -> Result<serde_json::Value, String> {
    zone.validate()?;
    let players: Vec<_> = zone.test_players.iter().filter(|p| p.enabled).collect();
    if players.is_empty() {
        return Err("Enable at least one test player before exporting a network test".into());
    }
    for player in &players {
        controller(player.controller.as_deref())?;
    }
    // Reserve only a new directory; on error remove only the directory we own.
    // Never modify the source zone, controller files or an earlier test export.
    fs::create_dir(output)
        .map_err(|e| format!("Network test requires a fresh output directory: {e}"))?;
    let result = (|| {
        let world_path = output.join("world");
        let world_report = export::publish_with_assets(
            zone,
            &world_path,
            content_id,
            revision,
            template,
            scenery,
        )?;
        let world = Store::open(&world_path).map_err(|e| e.to_string())?;
        write_scenario(zone, &world, output)?;
        let report = serde_json::json!({
            "clients":players.len(),"world":"world","scenario":"scenario.json","world_export":world_report,
            "scope":"Local movement test; final spawn admission remains server-authoritative; no account or combat session",
        });
        fs::write(
            output.join("network-test.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        Ok(report)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(output);
    }
    result
}

/// Publish checked test-player starts and immutable controller scripts into an owned stage.
pub(super) fn write_scenario(zone: &Zone, world: &Store, output: &Path) -> Result<usize, String> {
    let players: Vec<_> = zone.test_players.iter().filter(|p| p.enabled).collect();
    if players.is_empty() {
        return Err("Enable at least one test player before exporting a network test".into());
    }
    let scripts = players
        .iter()
        .map(|p| controller(p.controller.as_deref()))
        .collect::<Result<Vec<_>, _>>()?;
    for player in &players {
        let feet = GlobalPosition::new(player.position).ok_or("Invalid test player feet")?;
        let owner = procedural::owner(feet).ok_or("Test player leaves supported network world")?;
        let patch = world.patch(owner).map_err(|e| e.to_string())?;
        if !patch.covers(feet, None) || patch.try_landing_spawn(feet).is_none() {
            return Err(format!(
                "Test player {} needs clear space and bounded landing support",
                player.client_id
            ));
        }
    }
    if scripts.iter().any(Option::is_some) {
        fs::create_dir(output.join("controllers")).map_err(|e| e.to_string())?;
        fs::write(output.join(WALK_TURN_PATH), WALK_TURN).map_err(|e| e.to_string())?;
    }
    let entries: Vec<_> = players
        .iter()
        .zip(scripts)
        .map(|(p, script)| {
            serde_json::json!({
                "client_id":p.client_id,"feet":p.position,"controller":script,"network":p.network,
            })
        })
        .collect();
    let scenario =
        serde_json::json!({"schema":1,"world_identity":world.identity(),"players":entries});
    let bytes = serde_json::to_vec_pretty(&scenario).map_err(|e| e.to_string())?;
    if bytes.len() > theia_network::renet_runtime::test_players::MAX_BYTES {
        return Err("Network test scenario exceeds launcher byte budget".into());
    }
    fs::write(output.join("scenario.json"), bytes).map_err(|e| e.to_string())?;
    Ok(players.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_zone::test_players::{NetworkConditions, TestPlayer};
    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "theia-network-test-export-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
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
    fn fixture() -> Zone {
        Zone {
            schema: 12,
            origin: [8192., 0., 8192.],
            spawn: [8192., 0.1, 8192.],
            test_players: (1..=2)
                .map(|id| TestPlayer {
                    id,
                    name: format!("Test player {id}"),
                    client_id: id as u8,
                    enabled: true,
                    position: [8192. + (id - 1) as f64 * 2., 0.1, 8192.],
                    controller: (id == 2).then(|| "test.walk-turn".into()),
                    network: NetworkConditions {
                        delay_ms: if id == 2 { 80 } else { 0 },
                        ..default()
                    },
                })
                .collect(),
            ..default()
        }
    }
    #[test]
    fn enabled_pawns_export_exact_checked_world_and_immutable_controller_snapshot() {
        let folder = Folder::new();
        let zone = fixture();
        let original = serde_json::to_vec(&zone).unwrap();
        let path = folder.0.join("test");
        let report = publish(
            &zone,
            &path,
            "theia.test.players",
            1,
            None,
            &scenery_library::Library::default(),
        )
        .unwrap();
        assert_eq!(report["clients"], 2);
        let scenario: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("scenario.json")).unwrap()).unwrap();
        let world = Store::open(&path.join("world")).unwrap();
        assert_eq!(
            scenario["world_identity"],
            serde_json::json!(world.identity())
        );
        assert_eq!(
            scenario["players"][0]["feet"],
            serde_json::json!(zone.test_players[0].position)
        );
        assert_eq!(scenario["players"][1]["network"]["delay_ms"], 80);
        assert!(scenario["players"][0]["controller"].is_null());
        assert_eq!(scenario["players"][1]["controller"], WALK_TURN_PATH);
        assert_eq!(fs::read(path.join(WALK_TURN_PATH)).unwrap(), WALK_TURN);
        assert_eq!(
            Zone::load(&path.join("world/authoring.zone.json")).unwrap(),
            zone
        );
        assert_eq!(serde_json::to_vec(&zone).unwrap(), original);
        // Refusing an existing output must not remove or overwrite its contents.
        assert!(
            publish(
                &zone,
                &path,
                "theia.test.players",
                1,
                None,
                &scenery_library::Library::default()
            )
            .is_err()
        );
        assert_eq!(fs::read(path.join(WALK_TURN_PATH)).unwrap(), WALK_TURN);
        let mut zone = zone;
        zone.test_players[0].enabled = false;
        zone.test_players[0].controller = Some("test.not-installed".into());
        let second = folder.0.join("only-second");
        publish(
            &zone,
            &second,
            "theia.test.players",
            1,
            None,
            &scenery_library::Library::default(),
        )
        .unwrap();
        let scenario: serde_json::Value =
            serde_json::from_slice(&fs::read(second.join("scenario.json")).unwrap()).unwrap();
        assert_eq!(scenario["players"].as_array().unwrap().len(), 1);
        assert_eq!(scenario["players"][0]["client_id"], 2);
    }
    #[test]
    fn invalid_setup_and_unsupported_or_blocked_spawns_leave_no_partial_export() {
        let folder = Folder::new();
        for variant in 0..5 {
            let mut zone = fixture();
            match variant {
                0 => zone.test_players.iter_mut().for_each(|p| p.enabled = false),
                1 => zone.test_players[1].controller = Some("test.unknown".into()),
                2 => zone.test_players[1].position[1] = -2.,
                3 => zone.test_players[1].position[1] = 100.,
                _ => zone.objects.push(theia_zone::Object {
                    id: 3,
                    name: "Blocked spawn".into(),
                    primitive: Primitive::Box,
                    pose: Pose {
                        position: [8194., 1., 8192.],
                        yaw_degrees: 0.,
                        size: [1., 2., 1.],
                    },
                }),
            }
            let path = folder.0.join(format!("test-{variant}"));
            assert!(
                publish(
                    &zone,
                    &path,
                    "theia.test.players",
                    1,
                    None,
                    &scenery_library::Library::default()
                )
                .is_err(),
                "variant {variant}"
            );
            assert!(!path.exists());
        }
    }
}
