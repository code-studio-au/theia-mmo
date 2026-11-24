use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    env, fs,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    time::Duration,
};
use theia_realm_gateway::{
    Host, Instance, Release, Result,
    aws::{Ec2, Store},
    now,
};

#[derive(Deserialize)]
struct Game {
    run: String,
    at: u64,
    occupied: bool,
    players: usize,
    admitted: Vec<Join>,
    declined: Option<u64>,
}

#[derive(Deserialize)]
struct Join {
    id: String,
    subject: String,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Drain {
    run: String,
    at: u64,
}

fn read<T: DeserializeOwned>(file: &Path) -> Result<Option<T>> {
    let file = match fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(file)
    {
        Ok(file) if file.metadata()?.is_file() => file,
        Ok(_) => return Err("Lifecycle observation is not a regular file".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut data = Vec::new();
    file.take(4097).read_to_end(&mut data)?;
    if data.len() > 4096 {
        return Err("Lifecycle observation is too large".into());
    }
    Ok(Some(serde_json::from_slice(&data)?))
}

fn publish(dir: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    let temp = dir.join(format!("{name}.new"));
    fs::write(&temp, serde_json::to_vec(value)?)?;
    fs::rename(temp, dir.join(name))?;
    Ok(())
}

struct Controller {
    store: Store,
    boot: String,
    dir: PathBuf,
    release: Release,
}

impl Controller {
    async fn clear_drain(&self, failed: bool) -> Result<()> {
        match fs::remove_file(self.dir.join("drain.json")) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        self.store
            .change(|realm| {
                if realm.stopping.as_ref() != Some(&self.boot) {
                    return None;
                }
                realm.stopping = None;
                realm.idle_since = None;
                realm.failed = failed;
                Some(())
            })
            .await?;
        Ok(())
    }

    async fn tick(&self) -> Result<bool> {
        let at = now();
        let game: Option<Game> = read(&self.dir.join("game/game.json"))?;
        let game = game.filter(|g| g.at <= at && at - g.at < 5);
        let (revision, state) = self.store.read().await?;
        let leases = state
            .reservations
            .iter()
            .filter(|(_, lease)| {
                lease.expires > at && !state.maintenance && state.stopping.is_none()
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        publish(
            &self.dir,
            "leases.json",
            &serde_json::json!({"at": at, "leases": leases}),
        )?;
        let request = read::<Drain>(&self.dir.join("drain.json"))?;
        if state.stopping.as_ref() == Some(&self.boot) {
            let Some(request) = request else {
                // The controller was interrupted after winning the claim but
                // before signalling the game, so it has no flush to verify.
                self.clear_drain(false).await?;
                return Ok(false);
            };
            let flushed = read::<Drain>(&self.dir.join("game/flushed.json"))?;
            if flushed.as_ref() == Some(&request) {
                let service = tokio::process::Command::new("/usr/bin/systemctl")
                    .args([
                        "show",
                        "--property=ActiveState",
                        "--value",
                        "theia-development.service",
                    ])
                    .output()
                    .await?;
                if !service.status.success() {
                    return Err("Could not verify the game service stopped".into());
                }
                match service.stdout.as_slice() {
                    b"inactive\n" => (),
                    b"active\n" | b"deactivating\n"
                        if at.saturating_sub(request.at) < 45
                            && !game.as_ref().is_some_and(|game| game.run != request.run) =>
                    {
                        return Ok(false);
                    }
                    _ => {
                        self.clear_drain(true).await?;
                        return Err("The game service did not stay stopped after the flush".into());
                    }
                }
                let status = tokio::process::Command::new("/usr/bin/systemctl")
                    .arg("poweroff")
                    .status()
                    .await?;
                if !status.success() {
                    self.clear_drain(true).await?;
                    return Err("Power-off failed".into());
                }
                return Ok(true);
            }
            let elapsed = at.saturating_sub(request.at);
            if elapsed >= 45 {
                self.clear_drain(true).await?;
                eprintln!(
                    "Idle shutdown cancelled because the server did not confirm a complete persistence flush"
                );
            } else if game
                .as_ref()
                .is_some_and(|g| g.run == request.run && g.declined == Some(request.at))
            {
                // The authoritative gate received a direct or late admission.
                self.clear_drain(false).await?;
            }
            return Ok(false);
        }

        let claimed = self
            .store
            .change_from(revision, state, |state| {
                let host = Host {
                    boot: self.boot.clone(),
                    heartbeat: at,
                    ready: game.is_some() && state.release.as_ref() == Some(&self.release),
                    occupied: game.as_ref().is_some_and(|g| g.occupied),
                    players: game.as_ref().map_or(0, |g| g.players),
                };
                let mut admitted = false;
                if let Some(game) = &game {
                    for join in &game.admitted {
                        if state
                            .reservations
                            .get(&join.id)
                            .is_some_and(|lease| lease.subject == join.subject)
                        {
                            state.release(&join.id, &join.subject);
                            admitted = true;
                        }
                    }
                }
                if !admitted && !state.should_report(&host, at) {
                    return None;
                }
                Some(state.report(host, at))
            })
            .await?;
        if claimed == Some(true)
            && let Some(game) = &game
        {
            publish(
                &self.dir,
                "drain.json",
                &Drain {
                    run: game.run.clone(),
                    at: now(),
                },
            )?;
        }
        Ok(false)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let store = Store::new(&config, env::var("THEIA_REALM_TABLE")?);
    let args = env::args().skip(1).collect::<Vec<_>>();
    // The stopped host cannot clear the drain marker itself.
    if matches!(
        args.first().map(String::as_str),
        Some("maintenance" | "keep-alive")
    ) && let Some(expected) = store.read().await?.1.stopping
        && let Ok(id) = env::var("THEIA_INSTANCE_ID")
        && Ec2::new(config.clone(), id)?.state().await? == Instance::Stopped
    {
        store
            .change(|state| {
                if state.stopping.as_ref() != Some(&expected) {
                    return None;
                }
                state.stopping = None;
                state.host = None;
                Some(())
            })
            .await?;
    }
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["run"] => {
            let boot = fs::read_to_string("/proc/sys/kernel/random/boot_id")?
                .trim()
                .to_owned();
            let dir = PathBuf::from(env::var("THEIA_REALM_LIFECYCLE_DIR")?);
            let release = read::<Release>(&PathBuf::from(env::var("THEIA_REALM_RELEASE_FILE")?))?
                .ok_or("Missing running release metadata")?;
            if !release.valid() || !dir.is_absolute() || !dir.is_dir() {
                return Err("Invalid realm lifecycle configuration".into());
            }
            let controller = Controller {
                store,
                boot,
                dir,
                release,
            };
            loop {
                match controller.tick().await {
                    Ok(true) => return Ok(()),
                    Ok(false) => {}
                    Err(e) => eprintln!("Realm lifecycle observation unavailable: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(4)).await;
            }
        }
        ["status"] => {
            let state = store.read().await?.1;
            let at = now();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "release": state.release, "host": state.host, "draining": state.stopping.is_some(),
                    "maintenance": state.maintenance, "keep_alive_until": state.keep_alive_until,
                    "idle_since": state.idle_since, "failed": state.failed,
                    "pending_joins": state.reservations.values().filter(|join| join.expires > at).count(),
                }))?
            );
        }
        ["maintenance", mode @ ("on" | "off")] => {
            store
                .change(|s| {
                    if s.stopping.is_some() {
                        return None;
                    }
                    s.maintenance = *mode == "on";
                    s.idle_since = None;
                    Some(())
                })
                .await?
                .ok_or("Shutdown is already committed")?;
        }
        ["keep-alive", minutes @ ("0" | "30" | "60" | "120" | "240")] => {
            let minutes: u64 = minutes.parse()?;
            store
                .change(|s| {
                    if s.stopping.is_some() {
                        return None;
                    }
                    s.keep_alive_until = if minutes == 0 {
                        0
                    } else {
                        now() + minutes * 60
                    };
                    s.idle_since = None;
                    Some(())
                })
                .await?
                .ok_or("Shutdown is already committed")?;
        }
        ["clear-error"] => {
            store
                .change(|s| {
                    if !s.maintenance || s.stopping.is_some() {
                        return None;
                    }
                    s.failed = false;
                    s.start_deadline = 0;
                    s.idle_since = None;
                    Some(())
                })
                .await?
                .ok_or("Recovery requires maintenance and no committed shutdown")?;
        }
        ["activate", file] => {
            let release = read::<Release>(Path::new(file))?.ok_or("Missing release metadata")?;
            if !release.valid() {
                return Err("Invalid release metadata".into());
            }
            store
                .change(|s| {
                    if !s.maintenance || s.stopping.is_some() {
                        return None;
                    }
                    s.release = Some(release.clone());
                    s.host = None;
                    s.failed = false;
                    s.idle_since = None;
                    Some(())
                })
                .await?
                .ok_or("Release activation requires maintenance and no committed shutdown")?;
        }
        _ => {
            return Err(
                "Use run, status, maintenance on|off, keep-alive MINUTES, clear-error, or activate FILE".into(),
            );
        }
    }
    Ok(())
}
