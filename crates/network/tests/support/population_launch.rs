//! Optional real executable qualification; no window, mouse or display.
use super::*;
use std::process::{Child, Command, Stdio};
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_POPULATION_CONFIG, THEIA_CLIENT_BIN and THEIA_SERVER_BIN"]
async fn actual_server_and_headless_client_configuration_admit_two_classic_population_peers() {
    qualify("THEIA_POPULATION_CONFIG", false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires THEIA_AUTHORED_WORLD, THEIA_POPULATION_COMBAT_CONFIG, THEIA_CLIENT_BIN and THEIA_SERVER_BIN"]
async fn actual_population_server_replicates_independent_combat_and_fresh_respawns() {
    qualify("THEIA_POPULATION_COMBAT_CONFIG", true).await;
}

async fn qualify(config_key: &str, exercise_combat: bool) {
    let env_path = |name| PathBuf::from(std::env::var_os(name).expect(name));
    let world_path = env_path("THEIA_AUTHORED_WORLD");
    let config = env_path(config_key);
    let store = Arc::new(theia_network::authored_world::Store::open(&world_path).unwrap());
    let source = theia_network::procedural::Source::Authored(store.clone());
    let loaded =
        theia_network::simulation::population::configuration::Loaded::open(&config, &source)
            .unwrap();
    let count = loaded.count();
    assert_eq!(count, 5);
    let respawn_ms = loaded.respawn_ms();
    let base =
        loaded.profile_identity(theia_network::targeting::profile_identity(store.identity()));
    let classic_source =
        theia_network::simulation::combat::profile_identity(base, DATA, 100, respawn_ms).unwrap();
    let identity = theia_network::classic::profile_identity(classic_source);
    let abilities = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/gameplay/development-abilities.json");
    let common = |command: &mut Command| {
        command
            .arg("--authored-world")
            .arg(&world_path)
            .args(["--prediction", "--classic", "--combat-catalog"])
            .arg(&abilities)
            .arg("--population-config")
            .arg(&config);
    };
    let mut client = Command::new(env_path("THEIA_CLIENT_BIN"));
    common(&mut client);
    let output = client.arg("--configuration-report").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: serde_json::Value =
        serde_json::from_str(&stdout[stdout.find('{').unwrap()..]).unwrap();
    assert_eq!(report["world"], serde_json::json!(identity));
    assert_eq!(report["classic_source"], serde_json::json!(classic_source));
    assert_eq!(
        report["population"],
        serde_json::json!({"count":count,"respawn_ms":respawn_ms})
    );
    assert_eq!(report["renderer_started"], false);
    assert_eq!(report["network_started"], false);

    let scratch = tickets();
    fs::create_dir(&scratch.0).unwrap();
    let log_path = scratch.0.join("server.log");
    let log = fs::File::create(&log_path).unwrap();
    let ticket_dir = scratch.0.join("tickets");
    let mut command = Command::new(env_path("THEIA_SERVER_BIN"));
    command
        .arg("127.0.0.1:0")
        .arg(scratch.0.join("unused-cert.der"));
    common(&mut command);
    let mut server = Process(
        command
            .arg("--renet-tickets")
            .arg(&ticket_dir)
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let address = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let log = fs::read_to_string(&log_path).unwrap();
            if let Some(line) = log.lines().find(|line| line.starts_with("listening=")) {
                break line
                    .strip_prefix("listening=")
                    .unwrap()
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .parse::<SocketAddr>()
                    .unwrap();
            }
            assert!(
                server.0.try_wait().unwrap().is_none(),
                "Server exited: {log}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("Population server startup timed out");
    let spawn = store.spawn(0).unwrap();
    let local =
        World::with_terrain_source([spawn.metres()[0], spawn.metres()[2]], 1., source).unwrap();
    let chunks = local
        .procedural()
        .unwrap()
        .patch(spawn)
        .unwrap()
        .ids
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let catalog = Arc::new(Catalog::from_json(DATA).unwrap());
    let token = |id| fs::read(ticket_dir.join(format!("client-{id}.token"))).unwrap();
    let mut first = Peer::start_mode(
        address,
        token(1),
        identity,
        chunks.clone(),
        catalog.clone(),
        16,
        (None, Some(classic_source)),
    );
    let mut second = Peer::start_mode(
        address,
        token(2),
        identity,
        chunks,
        catalog,
        16,
        (None, Some(classic_source)),
    );
    let admitted = |s: &State, _: &[Event]| {
        s.chunk_ack.is_some() && s.combat.as_ref().is_some_and(|c| c.targets.len() == count)
    };
    let a = first.until(admitted).await;
    let b = second.until(admitted).await;
    assert_ne!(a.id, b.id);
    assert_eq!(a.welcome.as_ref().unwrap().world, identity);
    assert_eq!(
        a.combat.as_ref().unwrap().targets,
        b.combat.as_ref().unwrap().targets
    );
    let targets = &a.combat.as_ref().unwrap().targets;
    assert!(targets.iter().all(|t| t.current == 100));
    first.choice.send_replace(Some(Choice {
        target: Some(targets[0].entity),
    }));
    second.choice.send_replace(Some(Choice {
        target: Some(targets[1].entity),
    }));
    first
        .until(|s, _| {
            s.targeting
                .as_ref()
                .is_some_and(|t| t.selected == Some(targets[0].entity))
        })
        .await;
    second
        .until(|s, _| {
            s.targeting
                .as_ref()
                .is_some_and(|t| t.selected == Some(targets[1].entity))
        })
        .await;
    if exercise_combat {
        connected_combat(&mut first, &mut second, &a, respawn_ms).await;
    }
    first.finish().await.unwrap();
    second.finish().await.unwrap();
    drop(server);
}

async fn connected_combat(first: &mut Peer, second: &mut Peer, initial: &State, respawn_ms: u32) {
    assert_eq!(
        respawn_ms, 3000,
        "Qualification uses the saved 3-second lifecycle"
    );
    let targets = &initial.targeting.as_ref().unwrap().targets;
    let keys = [targets[0].key, targets[1].key];
    let positions = [targets[0].position, targets[1].position];
    let untouched: Vec<_> = initial
        .combat
        .as_ref()
        .unwrap()
        .targets
        .iter()
        .skip(2)
        .cloned()
        .collect();
    for remaining in [80, 60, 40, 20] {
        first.cast(keys[0]).await;
        second.cast(keys[1]).await;
        for peer in [&mut *first, &mut *second] {
            let damaged = peer
                .until(|s, _| {
                    s.combat.as_ref().is_some_and(|c| {
                        keys.iter().all(|key| {
                            c.targets
                                .iter()
                                .any(|t| t.entity == key.entity && t.current == remaining)
                        })
                    })
                })
                .await;
            let deadline = damaged
                .combat
                .as_ref()
                .unwrap()
                .cooldowns
                .iter()
                .map(|(_, at)| *at)
                .max()
                .unwrap_or(damaged.combat.as_ref().unwrap().tick);
            peer.until(|s, _| s.snapshot.as_ref().is_some_and(|s| s.tick >= deadline))
                .await;
        }
    }
    first.cast(keys[0]).await;
    let dead = first
        .until(|s, events| {
            s.combat.as_ref().is_some_and(|c| {
                c.targets
                    .iter()
                    .any(|t| t.entity == keys[0].entity && t.current == 0)
            }) && events
                .iter()
                .any(|e| matches!(e, Event::Died {target,..} if *target == keys[0]))
        })
        .await;
    let first_deadline = dead
        .combat
        .as_ref()
        .unwrap()
        .targets
        .iter()
        .find(|t| t.entity == keys[0].entity)
        .unwrap()
        .respawn_at
        .unwrap();
    // Stagger the second death by a second. Both peers must see independent clocks.
    second
        .until(|s, _| {
            s.snapshot
                .as_ref()
                .is_some_and(|s| s.tick >= first_deadline - 60)
        })
        .await;
    second.cast(keys[1]).await;
    let dead = second
        .until(|s, events| {
            s.combat.as_ref().is_some_and(|c| {
                c.targets
                    .iter()
                    .any(|t| t.entity == keys[1].entity && t.current == 0)
            }) && events
                .iter()
                .any(|e| matches!(e, Event::Died {target,..} if *target == keys[1]))
        })
        .await;
    let second_deadline = dead
        .combat
        .as_ref()
        .unwrap()
        .targets
        .iter()
        .find(|t| t.entity == keys[1].entity)
        .unwrap()
        .respawn_at
        .unwrap();
    assert!(second_deadline >= first_deadline + 30);
    for peer in [&mut *first, &mut *second] {
        let partial = peer
            .until(|s, _| {
                s.combat.as_ref().is_some_and(|c| {
                    c.targets.iter().all(|t| t.entity != keys[0].entity)
                        && c.targets.iter().any(|t| {
                            t.entity == keys[1].entity
                                && t.current == 0
                                && t.respawn_at == Some(second_deadline)
                        })
                })
            })
            .await;
        assert!(partial.combat.as_ref().unwrap().tick >= first_deadline);
    }
    for peer in [&mut *first, &mut *second] {
        let fresh = peer
            .until(|s, events| {
                s.combat.as_ref().is_some_and(|c| {
                    c.targets.len() == 5
                        && c.targets
                            .iter()
                            .all(|t| t.current == 100 && !keys.iter().any(|k| k.entity == t.entity))
                }) && keys.iter().all(|key| {
                    events
                        .iter()
                        .any(|e| matches!(e, Event::Respawned {previous,..} if previous == key))
                })
            })
            .await;
        assert!(fresh.combat.as_ref().unwrap().tick >= second_deadline);
        for health in &untouched {
            assert!(fresh.combat.as_ref().unwrap().targets.contains(health));
        }
        assert!(fresh.targeting.as_ref().unwrap().selected.is_none());
        for (key, position) in keys.iter().zip(positions) {
            let respawns: Vec<_> = peer
                .seen
                .iter()
                .filter_map(|e| match e {
                    Event::Respawned {
                        previous,
                        target,
                        position,
                    } if previous == key => Some((*target, *position)),
                    _ => None,
                })
                .collect();
            assert_eq!(respawns.len(), 1);
            assert_ne!(respawns[0].0, *key);
            assert_eq!(respawns[0].1, position);
            let damage: Vec<_> = peer
                .seen
                .iter()
                .filter(|e| matches!(e, Event::Damage {target,amount:20,..} if target == key))
                .collect();
            assert_eq!(damage.len(), 5);
        }
        let previous_sequence = fresh.targeting.as_ref().unwrap().request_sequence;
        peer.choice.send_replace(Some(Choice {
            target: Some(keys[0].entity),
        }));
        peer.until(|s, _| {
            s.targeting.as_ref().is_some_and(|t| {
                t.request_sequence > previous_sequence && !t.accepted && t.selected.is_none()
            })
        })
        .await;
    }
}
