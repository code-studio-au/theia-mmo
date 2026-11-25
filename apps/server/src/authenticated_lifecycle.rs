//! Local bridge to the host controller. Only consulted by the asynchronous owner.
use crate::admissions::{GameAdmissions, HostedJoin};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use theia_network::Result;

#[derive(Deserialize)]
struct Drain {
    run: String,
    at: u64,
}

pub(super) struct Lifecycle {
    dir: PathBuf,
    run: String,
    drain: Option<Drain>,
    declined: Option<u64>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_secs()
}

fn publish(dir: &Path, name: &str, value: serde_json::Value) -> Result<()> {
    let file = dir.join(format!("{name}.new"));
    fs::write(&file, serde_json::to_vec(&value)?)?;
    fs::rename(file, dir.join(name))?;
    Ok(())
}

#[derive(Deserialize)]
pub(crate) struct LeaseSnapshot {
    pub(crate) at: u64,
    leases: std::collections::BTreeMap<String, Lease>,
}

#[derive(Deserialize)]
struct Lease {
    subject: String,
    expires: u64,
}

impl LeaseSnapshot {
    pub(crate) fn current() -> Option<Self> {
        let dir = std::env::var_os("THEIA_REALM_LIFECYCLE_DIR")?;
        let file = fs::read(PathBuf::from(dir).join("leases.json")).ok()?;
        if file.len() > 4096 {
            return None;
        }
        let snapshot: Self = serde_json::from_slice(&file).ok()?;
        let at = now();
        (snapshot.at <= at && at - snapshot.at < 12).then_some(snapshot)
    }

    pub(crate) fn allows(&self, join: &HostedJoin) -> bool {
        self.leases
            .get(&join.id)
            .is_some_and(|lease| lease.subject == join.subject && lease.expires > now())
    }
}

impl Lifecycle {
    pub(super) fn from_environment(hosted: bool) -> Result<Option<Self>> {
        let Some(dir) = std::env::var_os("THEIA_REALM_LIFECYCLE_DIR").filter(|_| hosted) else {
            return Ok(None);
        };
        let dir = PathBuf::from(dir);
        if !dir.is_absolute() || !dir.join("game").is_dir() {
            return Err("Lifecycle directory and its game subdirectory must exist".into());
        }
        let run = format!("{}-{}", std::process::id(), now());
        Ok(Some(Self {
            dir,
            run,
            drain: None,
            declined: None,
        }))
    }

    pub(super) fn tick(&mut self, gate: &GameAdmissions) -> Result<bool> {
        let at = now();
        let request = match fs::read(self.dir.join("drain.json")) {
            Ok(data) if data.len() <= 512 => Some(serde_json::from_slice::<Drain>(&data)?),
            Ok(_) => return Err("Lifecycle drain request is too large".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        }
        .filter(|r| {
            r.run == self.run && r.at <= at && at - r.at < 20 && self.declined != Some(r.at)
        });
        let status = gate.lifecycle(request.is_some())?;
        if status.draining {
            if self.drain.is_none() {
                self.drain = request;
            }
            return Ok(true);
        }
        if let Some(request) = request {
            self.declined = Some(request.at);
        }
        publish(
            &self.dir.join("game"),
            "game.json",
            serde_json::json!({
                "run": self.run, "at": at, "occupied": status.occupied, "players": status.players, "admitted": status.admitted, "declined": self.declined,
            }),
        )?;
        Ok(false)
    }

    pub(super) fn flushed(self) -> Result<()> {
        if let Some(drain) = self.drain {
            publish(
                &self.dir.join("game"),
                "flushed.json",
                serde_json::json!({"run": drain.run, "at": drain.at}),
            )?;
        }
        Ok(())
    }
}
