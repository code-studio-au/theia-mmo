//! Own the development launcher through a private pipe; never block the UI waiting.
use super::*;
use std::{
    fs,
    path::Path,
    process::{Child, Command as Process, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Resource, Default)]
pub(super) struct Manager {
    child: Option<Child>,
    pub player_assets: Option<PathBuf>,
    pub creature_asset: Option<PathBuf>,
    pub template: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub account: bool,
    log: Option<PathBuf>,
    stopping: bool,
}
impl Manager {
    pub fn new(player_assets: Option<PathBuf>, creature_asset: Option<PathBuf>) -> Self {
        Self {
            child: None,
            player_assets,
            creature_asset,
            template: None,
            output: None,
            account: false,
            log: None,
            stopping: false,
        }
    }
    pub fn running(&self) -> bool {
        self.child.is_some()
    }
    pub fn start(&mut self, snapshot: &Path, players: &Path) -> Result<(), String> {
        self.start_mode(snapshot, players, false, false, None)
    }
    pub fn start_population(&mut self, snapshot: &Path, players: &Path) -> Result<(), String> {
        self.start_mode(snapshot, players, true, false, None)
    }
    pub fn start_account_population(
        &mut self,
        snapshot: &Path,
        players: &Path,
    ) -> Result<(), String> {
        self.start_mode(snapshot, players, true, true, None)
    }
    pub fn start_account_catalogues(
        &mut self,
        snapshot: &Path,
        players: &Path,
        items: &Path,
        rewards: &Path,
    ) -> Result<(), String> {
        self.start_mode(snapshot, players, true, true, Some((items, rewards)))
    }
    fn start_mode(
        &mut self,
        snapshot: &Path,
        players: &Path,
        population: bool,
        account: bool,
        catalogues: Option<(&Path, &Path)>,
    ) -> Result<(), String> {
        if self.running() {
            return Err("Stop the current network playtest first".into());
        }
        let snapshot = fs::canonicalize(snapshot).map_err(|e| e.to_string())?;
        let players = fs::canonicalize(players).map_err(|e| e.to_string())?;
        if !players.join("character.glb").is_file() {
            return Err("Choose a player asset directory containing character.glb".into());
        }
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let binaries = executable
            .parent()
            .ok_or("Editor executable has no directory")?;
        for name in ["theia-server", "theia-client"] {
            let name = if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            };
            if !binaries.join(name).is_file() {
                return Err(
                    "Build the client and server beside the editor before network playtesting"
                        .into(),
                );
            }
        }
        let script_name = if account {
            "account_population_playtest.py"
        } else {
            "network_playtest.py"
        };
        let script = PathBuf::from("tools").join(script_name);
        let script = if script.is_file() {
            script
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tools")
                .join(script_name)
        };
        let script = fs::canonicalize(script)
            .map_err(|_| "Cannot locate the playtest launcher in this development checkout")?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let runs = snapshot.join("runs");
        fs::create_dir_all(&runs).map_err(|e| e.to_string())?;
        let output = runs.join(format!("session-{}-{nonce}", std::process::id()));
        let log = output.with_extension("launcher.log");
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&log)
            .map_err(|e| e.to_string())?;
        let error = file.try_clone().map_err(|e| e.to_string())?;
        let python = std::env::var_os("THEIA_NETWORK_TEST_PYTHON").unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
        let mut command = Process::new(python);
        command.arg(script);
        if population && !account {
            command
                .arg("--population-config")
                .arg(snapshot.join("population.json"))
                .arg("--combat-catalog")
                .arg(snapshot.join("combat.json"))
                .arg("--player-life");
            if snapshot.join("creature.glb").is_file() {
                command
                    .arg("--creature-asset")
                    .arg(snapshot.join("creature.glb"));
            }
        }
        if account {
            command.arg("--package").arg(&snapshot);
            if let Some((items, rewards)) = catalogues {
                command
                    .arg("--items")
                    .arg(items)
                    .arg("--reward-catalog")
                    .arg(rewards);
            }
        } else {
            command
                .arg("--scenario")
                .arg(snapshot.join("scenario.json"))
                .arg("--world")
                .arg(snapshot.join("world"))
                .arg("--stop-on-stdin-eof");
        }
        let child = command
            .arg("--players")
            .arg(&players)
            .arg("--binaries")
            .arg(binaries)
            .arg("--output")
            .arg(&output)
            .stdin(Stdio::piped())
            .stdout(file)
            .stderr(error)
            .spawn()
            .map_err(|e| format!("Cannot start Python network launcher: {e}"))?;
        self.child = Some(child);
        self.account = account;
        self.output = Some(output);
        self.log = Some(log);
        self.player_assets = Some(players);
        self.stopping = false;
        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(child) = &mut self.child {
            // EOF also works if the editor exits/crashes while the launcher is
            // validating input and has not created its session directory yet.
            drop(child.stdin.take());
            self.stopping = true;
        }
    }
}
impl Drop for Manager {
    fn drop(&mut self) {
        self.stop();
    }
}

pub(super) fn poll(
    mut manager: ResMut<Manager>,
    mut session: ResMut<Session>,
    mut requests: ResMut<workspace::Requests>,
) {
    if std::mem::take(&mut requests.stop_network_playtest) && manager.running() && !manager.stopping
    {
        manager.stop();
        session.status = "Stopping network server and clients…".into();
    }
    let result = manager
        .child
        .as_mut()
        .and_then(|child| match child.try_wait() {
            Ok(Some(status)) => Some(Ok(status.success())),
            Ok(None) => None,
            Err(error) => Some(Err(error.to_string())),
        });
    if let Some(result) = result {
        match result {
            Ok(success) => {
                manager.child.take();
                session.status = if success {
                    "Network playtest stopped".into()
                } else {
                    format!(
                        "Network playtest failed; see {}",
                        manager
                            .log
                            .as_ref()
                            .map_or_else(String::new, |p| p.display().to_string())
                    )
                };
            }
            Err(error) => {
                manager.stop();
                session.status = format!("Stopping network playtest after process error: {error}");
            }
        }
    }
    session.network_active = manager.running();
}
