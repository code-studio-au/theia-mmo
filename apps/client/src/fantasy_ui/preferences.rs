//! Explicit local preference file. Reads before App creation; writes on Bevy's I/O pool.
use super::PreferredScale;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

const LIMIT: u64 = 4096;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema: u8,
    ui_scale: f32,
}
impl File {
    fn valid(&self) -> bool {
        self.schema == 1 && self.ui_scale.is_finite() && (0.75..=1.5).contains(&self.ui_scale)
    }
}
pub(crate) struct Config {
    path: PathBuf,
    pub scale: f32,
}
impl Config {
    pub(crate) fn parse(
        args: &mut Vec<String>,
        enabled: bool,
        scale: f32,
        explicit: bool,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Some(index) = args.iter().position(|a| a == "--ui-preferences") else {
            return Ok(None);
        };
        args.remove(index);
        if !enabled || index >= args.len() || args[index].starts_with("--") {
            return Err("--ui-preferences requires --fantasy-ui and a local file path".into());
        }
        let path = PathBuf::from(args.remove(index));
        if args.iter().any(|a| a == "--ui-preferences") || path.file_name().is_none() {
            return Err("Choose --ui-preferences once with a file path".into());
        }
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        let saved = read(&path)?;
        Ok(Some(Self {
            path,
            scale: if explicit {
                scale
            } else {
                saved.unwrap_or(scale)
            },
        }))
    }
    pub(crate) fn install(self, app: &mut App) {
        app.insert_resource(Store {
            path: self.path,
            requested: self.scale,
            pending: None,
            status: Status::Unchanged,
        })
        .add_systems(Update, save);
    }
}
fn read(path: &Path) -> Result<Option<f32>, Box<dyn std::error::Error>> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !file.metadata()?.is_file() {
        return Err("Interface preferences must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err("Interface preferences exceed 4 KiB".into());
    }
    let file: File = serde_json::from_slice(&bytes)?;
    if !file.valid() {
        return Err("Interface preferences require schema 1 and UI scale 0.75..1.5".into());
    }
    Ok(Some(file.ui_scale))
}
fn write(path: &Path, scale: f32) -> Result<(), String> {
    let data = File {
        schema: 1,
        ui_scale: scale,
    };
    if !data.valid() {
        return Err("Invalid UI scale".into());
    }
    let parent = path.parent().ok_or("Missing preference directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // A unique same-directory candidate avoids truncating an existing preference.
    // Rename replaces only after the complete candidate has been flushed.
    let name = path
        .file_name()
        .ok_or("Missing preference filename")?
        .to_string_lossy();
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let temp = parent.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let bytes = serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        fs::rename(&temp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
#[derive(Clone, Copy)]
enum Status {
    Unchanged,
    Saving,
    Saved,
    Failed,
}
#[derive(Resource)]
pub(super) struct Store {
    path: PathBuf,
    requested: f32,
    pending: Option<Task<Result<(), String>>>,
    status: Status,
}
impl Store {
    pub(super) fn hint(&self) -> &'static str {
        match self.status {
            Status::Unchanged => "Local preferences enabled. The world continues running.",
            Status::Saving => "Saving UI size on this device. The world continues running.",
            Status::Saved => "UI size saved on this device. The world continues running.",
            Status::Failed => "Could not save UI size; current session still uses it.",
        }
    }
}
fn save(preferred: Res<PreferredScale>, mut store: ResMut<Store>) {
    if let Some(task) = &mut store.pending
        && let Some(result) = check_ready(task)
    {
        store.pending = None;
        store.status = if let Err(error) = result {
            warn!("Could not save interface preferences: {error}");
            Status::Failed
        } else {
            Status::Saved
        };
    }
    if store.pending.is_none() && store.requested != preferred.0 {
        store.requested = preferred.0;
        let scale = preferred.0;
        let path = store.path.clone();
        store.pending = Some(IoTaskPool::get().spawn(async move { write(&path, scale) }));
        store.status = Status::Saving;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "theia-ui-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn file(&self) -> PathBuf {
            self.0.join("interface.json")
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn validates_bounded_versioned_preferences_without_rewriting_bad_files() {
        let scratch = Scratch::new();
        let path = scratch.file();
        assert_eq!(read(&path).unwrap(), None);
        for bytes in [
            b"{}".as_slice(),
            b"{\"schema\":2,\"ui_scale\":1}",
            b"{\"schema\":1,\"ui_scale\":2}",
            b"{\"schema\":1,\"ui_scale\":1,\"extra\":true}",
            b"{\"schema\":1,\"ui_scale\":NaN}",
        ] {
            fs::write(&path, bytes).unwrap();
            assert!(read(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
        fs::write(&path, vec![b' '; 4097]).unwrap();
        assert!(read(&path).is_err());
    }
    #[test]
    fn explicit_cli_overrides_saved_scale_and_file_is_opt_in() {
        let scratch = Scratch::new();
        let path = scratch.file();
        write(&path, 1.25).unwrap();
        let args = || {
            vec![
                "--ui-preferences".into(),
                path.to_string_lossy().into_owned(),
            ]
        };
        assert_eq!(
            Config::parse(&mut args(), true, 1., false)
                .unwrap()
                .unwrap()
                .scale,
            1.25
        );
        assert_eq!(
            Config::parse(&mut args(), true, 0.75, true)
                .unwrap()
                .unwrap()
                .scale,
            0.75
        );
        assert_eq!(read(&path).unwrap(), Some(1.25));
        assert!(Config::parse(&mut args(), false, 1., false).is_err());
        assert!(Config::parse(&mut vec!["--ui-preferences".into()], true, 1., false).is_err());
        assert!(Config::parse(&mut [args(), args()].concat(), true, 1., false).is_err());
        assert!(
            Config::parse(&mut Vec::new(), true, 1., false)
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn replacement_is_complete_and_failed_write_preserves_previous_file() {
        let scratch = Scratch::new();
        let path = scratch.file();
        write(&path, 1.25).unwrap();
        write(&path, 0.75).unwrap();
        assert_eq!(read(&path).unwrap(), Some(0.75));
        assert!(write(&path, f32::NAN).is_err());
        assert_eq!(read(&path).unwrap(), Some(0.75));
        let directory = scratch.0.join("directory");
        fs::create_dir(&directory).unwrap();
        assert!(write(&directory, 1.).is_err());
        assert_eq!(
            fs::read_dir(&scratch.0).unwrap().count(),
            2,
            "Failed candidate removed"
        );
    }
    #[test]
    fn actual_interface_buttons_save_preference_while_resize_only_changes_fitted_size() {
        let scratch = Scratch::new();
        let path = scratch.file();
        let mut args = vec![
            "--ui-preferences".into(),
            path.to_string_lossy().into_owned(),
        ];
        let config = Config::parse(&mut args, true, 1., false).unwrap().unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        super::super::install_scale(&mut app, config.scale);
        config.install(&mut app);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                resolution: bevy::window::WindowResolution::new(1920, 1200),
                ..default()
            })
            .id();
        app.update();
        assert!(!path.exists());
        let button = |world: &mut World, title: &str| {
            world
                .query::<(Entity, &AccessibleLabel)>()
                .iter(world)
                .find(|(_, label)| label.0 == title)
                .unwrap()
                .0
        };
        let toggle = button(app.world_mut(), "Interface settings [F10]");
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: toggle });
        let preset = button(app.world_mut(), "UI size 125%");
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: preset });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            app.update();
            if matches!(app.world().resource::<Store>().status, Status::Saved) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        // The painter sees the completed status on its following PostUpdate.
        app.update();
        assert_eq!(read(&path).unwrap(), Some(1.25));
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.25);
        let world = app.world_mut();
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0 == "UI size saved on this device. The world continues running.")
        );
        let saved = fs::read(&path).unwrap();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1280., 720.);
        app.update();
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.);
        assert!(app.world().resource::<Store>().pending.is_none());
        assert_eq!(
            fs::read(&path).unwrap(),
            saved,
            "Viewport fitting must never overwrite the user's size"
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(1920., 1200.);
        app.update();
        assert_eq!(app.world().resource::<bevy::ui::UiScale>().0, 1.25);
        let world = app.world_mut();
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0 == "UI size: 125% preferred · 125% displayed")
        );
    }
    #[test]
    fn failed_background_save_keeps_session_value_and_recovers_on_a_later_change() {
        let scratch = Scratch::new();
        let path = scratch.file();
        fs::create_dir(&path).unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PreferredScale(1.));
        Config {
            path: path.clone(),
            scale: 1.,
        }
        .install(&mut app);
        app.world_mut().resource_mut::<PreferredScale>().0 = 1.25;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            app.update();
            let store = app.world().resource::<Store>();
            if store.pending.is_none() && matches!(store.status, Status::Failed) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        assert!(
            app.world()
                .resource::<Store>()
                .hint()
                .contains("Could not save")
        );
        for _ in 0..5 {
            app.update();
        }
        assert!(
            app.world().resource::<Store>().pending.is_none(),
            "No per-frame failed-write retry"
        );
        fs::remove_dir(&path).unwrap();
        app.world_mut().resource_mut::<PreferredScale>().0 = 1.5;
        loop {
            app.update();
            let store = app.world().resource::<Store>();
            if store.pending.is_none() && matches!(store.status, Status::Saved) {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(read(&path).unwrap(), Some(1.5));
    }
    #[test]
    fn actual_background_save_retains_preferred_size_and_survives_new_client_resources() {
        let scratch = Scratch::new();
        let path = scratch.file();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PreferredScale(1.));
        Config {
            path: path.clone(),
            scale: 1.,
        }
        .install(&mut app);
        app.update();
        assert!(!path.exists(), "No launch-time write");
        app.world_mut().resource_mut::<PreferredScale>().0 = 1.5;
        app.update();
        app.world_mut().resource_mut::<PreferredScale>().0 = 0.75;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            app.update();
            let store = app.world().resource::<Store>();
            if store.pending.is_none() && store.requested == 0.75 {
                assert!(matches!(store.status, Status::Saved));
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        let mut args = vec![
            "--ui-preferences".into(),
            path.to_string_lossy().into_owned(),
        ];
        let fresh = Config::parse(&mut args, true, 1., false).unwrap().unwrap();
        assert_eq!(fresh.scale, 0.75);
        assert_eq!(super::super::fit_scale(1.5, Vec2::new(1280., 720.)), 1.);
        assert_eq!(read(&path).unwrap(), Some(0.75));
    }
}
