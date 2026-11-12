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
    let env_path = |name| PathBuf::from(std::env::var_os(name).expect(name));
    let world_path = env_path("THEIA_AUTHORED_WORLD");
    let config = env_path("THEIA_POPULATION_CONFIG");
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
    first.finish().await.unwrap();
    second.finish().await.unwrap();
    drop(server);
}
