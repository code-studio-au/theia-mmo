//! Opt-in local-owner speed requests; applied values come from server corrections.
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
};
use std::path::PathBuf;
#[derive(Component)]
pub(super) struct Summary;
#[derive(Component)]
pub(super) struct ResetSummary;
#[derive(Resource)]
pub(super) struct Speed {
    path: PathBuf,
    pub requested: u8,
    pub applied: Option<u8>,
    queued: u8,
    pub reset_nonce: u64,
    queued_reset: u64,
    pub stream: Option<u64>,
    home: Option<[f64; 3]>,
    reset_from: Option<u64>,
    reset_started: Option<std::time::Instant>,
    pub reset_status: String,
    pending: Option<Task<Result<(), String>>>,
    error: Option<String>,
}
impl Speed {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            requested: 1,
            applied: None,
            queued: 1,
            reset_nonce: 0,
            queued_reset: 0,
            stream: None,
            home: None,
            reset_from: None,
            reset_started: None,
            reset_status: "Returns to your starting point and clears falling movement.".into(),
            pending: None,
            error: None,
        }
    }
    pub fn request(&mut self, value: u8) {
        if theia_network::development::valid(value) {
            self.requested = value;
            if self.error.take().is_some() {
                self.queued = 0;
            }
        }
    }
    pub fn reset_position(&mut self) {
        if self.reset_from.is_some() {
            return;
        }
        let Some(stream) = self.stream else {
            self.reset_status = "No active connection. Relaunch the playtest to reconnect.".into();
            return;
        };
        self.reset_nonce = self.reset_nonce.saturating_add(1);
        self.reset_from = Some(stream);
        self.reset_started = Some(std::time::Instant::now());
        self.reset_status = "Returning to your starting position…".into();
        self.error = None;
    }
    fn label(&self) -> String {
        if self.error.is_some() {
            return "Could not request speed. Select a multiplier to retry.".into();
        }
        match self.applied {
            Some(value) if value == self.requested => format!("Running speed: {value}×"),
            Some(value) => format!("Running speed: {value}× · requesting {}×", self.requested),
            None => "Waiting for the local developer server…".into(),
        }
    }
}
pub(super) fn install(app: &mut App) {
    if let Some(path) = std::env::var_os(theia_network::development::ENV)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
    {
        app.insert_resource(Speed::new(path));
    }
    app.add_systems(Update, update);
}
fn write(path: PathBuf, multiplier: u8, reset_nonce: u64) -> Result<(), String> {
    let bytes = serde_json::to_vec(&theia_network::development::Request {
        schema: 1,
        multiplier,
        reset_nonce,
    })
    .map_err(|e| e.to_string())?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        std::fs::write(&temp, bytes)?;
        std::fs::rename(&temp, &path)
    })()
    .map_err(|e: std::io::Error| e.to_string());
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
fn update(
    mut speed: Option<ResMut<Speed>>,
    network: Option<Res<crate::network::Network>>,
    mut labels: Query<&mut Text, (With<Summary>, Without<ResetSummary>)>,
    mut reset_labels: Query<&mut Text, (With<ResetSummary>, Without<Summary>)>,
) {
    let label = if let Some(speed) = speed.as_mut() {
        let correction = network.as_ref().and_then(|n| {
            let state = n.state();
            if state.error.is_some() {
                None
            } else {
                state.correction
            }
        });
        if speed.home.is_none() {
            speed.home = correction.map(|c| c.state.pose.feet.metres());
        }
        speed.applied = correction.map(|c| c.speed);
        speed.stream = correction.map(|c| c.state.input.stream_epoch);
        if let Some(previous) = speed.reset_from {
            if speed.stream.is_some_and(|stream| stream > previous)
                && correction.zip(speed.home).is_some_and(|(c, home)| {
                    let p = c.state.pose.feet.metres();
                    (p[0] - home[0]).hypot(p[2] - home[2]) < 0.25 && (p[1] - home[1]).abs() < 1.
                })
            {
                speed.reset_from = None;
                speed.reset_status = "Returned to your starting position.".into();
            } else if speed
                .reset_started
                .is_some_and(|start| start.elapsed().as_secs() >= 10)
            {
                speed.reset_from = None;
                speed.reset_status =
                    "Reset was not confirmed. Check the connection and try again.".into();
            }
        }
        if let Some(task) = &mut speed.pending
            && let Some(result) = check_ready(task)
        {
            speed.pending = None;
            speed.error = result.err();
            if speed.error.is_some() && speed.reset_from.take().is_some() {
                speed.reset_status = "Could not request reset. Try again.".into();
            }
        }
        if speed.pending.is_none()
            && (speed.queued != speed.requested || speed.queued_reset != speed.reset_nonce)
        {
            let path = speed.path.clone();
            let value = speed.requested;
            let reset = speed.reset_nonce;
            speed.queued = value;
            speed.queued_reset = reset;
            speed.pending = Some(IoTaskPool::get().spawn(async move { write(path, value, reset) }));
        }
        speed.label()
    } else {
        "Speed controls require a local developer walkthrough.".into()
    };
    let reset_label = speed
        .as_ref()
        .map_or("Reset requires a local developer walkthrough.", |s| {
            s.reset_status.as_str()
        });
    for mut text in &mut reset_labels {
        if text.0 != reset_label {
            text.0 = reset_label.into();
        }
    }
    for mut text in &mut labels {
        if text.0 != label {
            text.0 = label.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reset_request_is_atomic_preserves_speed_and_legacy_files_are_admitted() {
        let path = std::env::temp_dir().join(format!("theia-reset-{}.json", std::process::id()));
        write(path.clone(), 8, 42).unwrap();
        let value = theia_network::development::read(&path).unwrap();
        assert_eq!((value.multiplier, value.reset_nonce), (8, 42));
        std::fs::write(&path, br#"{"schema":1,"multiplier":4}"#).unwrap();
        assert_eq!(
            theia_network::development::read(&path).unwrap().reset_nonce,
            0
        );
        std::fs::write(&path, vec![b' '; 129]).unwrap();
        assert!(theia_network::development::read(&path).is_err());
        std::fs::remove_file(path).unwrap();
        let mut control = Speed::new("/tmp/unused-reset".into());
        control.reset_position();
        assert_eq!(control.reset_nonce, 0);
        assert!(control.reset_status.contains("Relaunch"));
    }
}
