//! Explicit local preference file. Reads before App creation; writes on Bevy's I/O pool.
use super::{PreferredScale, graphics::ViewDistance};
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
fn default_vsync() -> bool {
    true
}
fn default_fps_cap() -> u16 {
    super::graphics::FrameCap::default().0
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema: u8,
    ui_scale: f32,
    #[serde(default = "default_fps_cap")]
    fps_cap: u16,
    #[serde(default = "default_vsync")]
    vsync: bool,
    #[serde(default)]
    view_distance: ViewDistance,
    #[serde(default)]
    debug_stats: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    world_effects: Option<bool>,
}
impl File {
    fn valid(&self) -> bool {
        self.schema == 1
            && super::graphics::FrameCap(self.fps_cap).valid()
            && self.ui_scale.is_finite()
            && (0.75..=1.5).contains(&self.ui_scale)
    }
}
pub(crate) struct Config {
    path: PathBuf,
    pub scale: f32,
    fps_cap: u16,
    vsync: bool,
    view_distance: ViewDistance,
    debug_stats: bool,
    pub world_effects: Option<bool>,
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
        let saved = read_settings(&path)?;
        Ok(Some(Self {
            path,
            scale: if explicit {
                scale
            } else {
                saved.as_ref().map_or(scale, |file| file.ui_scale)
            },
            debug_stats: saved.as_ref().is_some_and(|file| file.debug_stats),
            fps_cap: saved
                .as_ref()
                .map_or_else(default_fps_cap, |file| file.fps_cap),
            vsync: saved.as_ref().is_none_or(|file| file.vsync),
            view_distance: saved
                .as_ref()
                .map_or_else(ViewDistance::default, |file| file.view_distance),
            world_effects: saved.and_then(|file| file.world_effects),
        }))
    }
    pub(crate) fn install(self, app: &mut App) {
        let observed_effects = app
            .world()
            .get_resource::<crate::particle_world::VisibilityChoice>()
            .map(|choice| choice.0);
        app.insert_resource(super::debug::Enabled(self.debug_stats));
        app.insert_resource(super::graphics::FrameCap(self.fps_cap));
        app.insert_resource(super::graphics::Vsync(self.vsync));
        app.insert_resource(self.view_distance);
        app.insert_resource(Store {
            requested_debug: self.debug_stats,
            requested_fps: self.fps_cap,
            requested_vsync: self.vsync,
            requested_view: self.view_distance,
            path: self.path,
            requested: self.scale,
            world_effects: self.world_effects,
            requested_effects: self.world_effects,
            observed_effects,
            pending: None,
            retry_requested: false,
            status: Status::Unchanged,
        })
        .add_systems(Update, save);
    }
}
fn read_settings(path: &Path) -> Result<Option<File>, Box<dyn std::error::Error>> {
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
        return Err(
            "Interface preferences require schema 1, UI scale 0.75..1.5 and a supported FPS cap"
                .into(),
        );
    }
    Ok(Some(file))
}
#[cfg(test)]
fn read(path: &Path) -> Result<Option<f32>, Box<dyn std::error::Error>> {
    Ok(read_settings(path)?.map(|file| file.ui_scale))
}
#[cfg(test)]
fn write(path: &Path, scale: f32) -> Result<(), String> {
    write_preferences(
        path,
        scale,
        None,
        false,
        default_fps_cap(),
        true,
        ViewDistance::default(),
    )
}
fn write_preferences(
    path: &Path,
    scale: f32,
    world_effects: Option<bool>,
    debug_stats: bool,
    fps_cap: u16,
    vsync: bool,
    view_distance: ViewDistance,
) -> Result<(), String> {
    let data = File {
        schema: 1,
        ui_scale: scale,
        fps_cap,
        vsync,
        view_distance,
        debug_stats,
        world_effects,
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
    requested_debug: bool,
    requested_fps: u16,
    requested_vsync: bool,
    requested_view: ViewDistance,
    path: PathBuf,
    requested: f32,
    world_effects: Option<bool>,
    requested_effects: Option<bool>,
    observed_effects: Option<bool>,
    pending: Option<Task<Result<(), String>>>,
    retry_requested: bool,
    status: Status,
}
impl Store {
    pub(super) fn retry_failed(&mut self) {
        if matches!(self.status, Status::Failed) && self.pending.is_none() {
            self.retry_requested = true;
        }
    }
    pub(super) fn hint(&self) -> &'static str {
        match self.status {
            Status::Unchanged => "Local preferences enabled. The world continues running.",
            Status::Saving => "Saving preferences on this device. The world continues running.",
            Status::Saved => "Preferences saved on this device. The world continues running.",
            Status::Failed => {
                "Could not save preferences. Change a setting or select a UI size to retry."
            }
        }
    }
}
fn save(
    preferred: Res<PreferredScale>,
    effects: Option<Res<crate::particle_world::VisibilityChoice>>,
    mut store: ResMut<Store>,
    debug: Option<Res<super::debug::Enabled>>,
    cap: Option<Res<super::graphics::FrameCap>>,
    vsync: Option<Res<super::graphics::Vsync>>,
    view: Option<Res<ViewDistance>>,
) {
    let view_distance = view.as_deref().copied().unwrap_or_default();
    let vsync = vsync.as_ref().is_none_or(|choice| choice.0);
    let debug_stats = debug.as_ref().is_some_and(|v| v.0);
    let fps_cap = cap.as_ref().map_or_else(default_fps_cap, |cap| cap.0);
    let current_effects = effects.as_ref().map(|choice| choice.0);
    let effects_changed = current_effects.is_some() && current_effects != store.observed_effects;
    if effects_changed {
        store.observed_effects = current_effects;
        store.world_effects = current_effects;
    }
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
    if store.pending.is_none()
        && (store.requested != preferred.0
            || store.retry_requested
            || store.requested_effects != store.world_effects
            || store.requested_debug != debug_stats
            || store.requested_fps != fps_cap
            || store.requested_vsync != vsync
            || store.requested_view != view_distance)
    {
        store.retry_requested = false;
        store.requested = preferred.0;
        store.requested_debug = debug_stats;
        store.requested_fps = fps_cap;
        store.requested_vsync = vsync;
        store.requested_view = view_distance;
        store.requested_effects = store.world_effects;
        let scale = preferred.0;
        let path = store.path.clone();
        let world_effects = store.world_effects;
        store.pending = Some(IoTaskPool::get().spawn(async move {
            write_preferences(
                &path,
                scale,
                world_effects,
                debug_stats,
                fps_cap,
                vsync,
                view_distance,
            )
        }));
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
    fn view_distance_buttons_save_reload_and_guard_hidden_or_unfocused_settings() {
        let scratch = Scratch::new();
        let path = scratch.file();
        fs::write(
            &path,
            r#"{"schema":1,"ui_scale":1,"fps_cap":144,"vsync":false}"#,
        )
        .unwrap();
        assert_eq!(
            read_settings(&path).unwrap().unwrap().view_distance,
            ViewDistance::Nearby
        );
        let args = || {
            vec![
                "--ui-preferences".into(),
                path.to_string_lossy().into_owned(),
            ]
        };
        let config = Config::parse(&mut args(), true, 1., false)
            .unwrap()
            .unwrap();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        super::super::install_scale(&mut app, 1.);
        config.install(&mut app);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        app.update();
        let button = |world: &mut World, title: &str| {
            world
                .query::<(Entity, &AccessibleLabel)>()
                .iter(world)
                .find(|(_, label)| label.0 == title)
                .unwrap()
                .0
        };
        let maximum = button(app.world_mut(), "View distance: Maximum");
        for (open, graphics, focused) in [
            (false, false, true),
            (true, false, true),
            (true, true, false),
        ] {
            let mut ui = app.world_mut().resource_mut::<super::super::settings::Ui>();
            ui.open = open;
            ui.graphics_open = graphics;
            app.world_mut().get_mut::<Window>(window).unwrap().focused = focused;
            app.world_mut()
                .trigger(bevy::ui_widgets::Activate { entity: maximum });
            assert_eq!(
                *app.world().resource::<ViewDistance>(),
                ViewDistance::Nearby
            );
        }
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        for choice in [
            ViewDistance::Maximum,
            ViewDistance::Far,
            ViewDistance::Nearby,
        ] {
            let entity = button(
                app.world_mut(),
                &format!("View distance: {}", choice.label()),
            );
            app.world_mut()
                .trigger(bevy::ui_widgets::Activate { entity });
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                app.update();
                if matches!(app.world().resource::<Store>().status, Status::Saved)
                    && app.world().resource::<Store>().requested_view == choice
                {
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            let saved = read_settings(&path).unwrap().unwrap();
            assert_eq!(saved.view_distance, choice);
            assert_eq!(saved.fps_cap, 144);
            assert!(!saved.vsync);
            let mut restarted = App::new();
            Config::parse(&mut args(), true, 1., false)
                .unwrap()
                .unwrap()
                .install(&mut restarted);
            assert_eq!(*restarted.world().resource::<ViewDistance>(), choice);
        }
        let invalid = r#"{"schema":1,"ui_scale":1,"view_distance":"unbounded"}"#;
        fs::write(&path, invalid).unwrap();
        assert!(read_settings(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), invalid);
    }
    #[test]
    fn vsync_defaults_on_for_legacy_settings_and_round_trips_off() {
        let scratch = Scratch::new();
        let path = scratch.file();
        for bytes in [
            r#"{"schema":1,"ui_scale":1}"#,
            r#"{"schema":1,"ui_scale":1,"fps_cap":144}"#,
        ] {
            fs::write(&path, bytes).unwrap();
            assert!(read_settings(&path).unwrap().unwrap().vsync);
        }
        write_preferences(&path, 1., None, false, 144, false, ViewDistance::default()).unwrap();
        let saved = read_settings(&path).unwrap().unwrap();
        assert!(!saved.vsync);
        assert_eq!(saved.fps_cap, 144);
        for value in ["null", "0", "\"on\""] {
            let bytes = format!(r#"{{"schema":1,"ui_scale":1,"vsync":{value}}}"#);
            fs::write(&path, &bytes).unwrap();
            assert!(read_settings(&path).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
        }
    }
    #[test]
    fn fps_preferences_accept_legacy_files_and_reject_invalid_caps_without_rewriting() {
        let scratch = Scratch::new();
        let path = scratch.file();
        fs::write(&path, r#"{"schema":1,"ui_scale":1}"#).unwrap();
        assert_eq!(read_settings(&path).unwrap().unwrap().fps_cap, 60);
        for cap in [0, 30, 60, 90, 120, 144, 165, 240] {
            write_preferences(&path, 1., None, false, cap, true, ViewDistance::default()).unwrap();
            assert_eq!(read_settings(&path).unwrap().unwrap().fps_cap, cap);
        }
        for cap in [1, 29, 1000, 65535] {
            let bytes = format!(r#"{{"schema":1,"ui_scale":1,"fps_cap":{cap}}}"#);
            fs::write(&path, &bytes).unwrap();
            assert!(read_settings(&path).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
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
                .any(|text| text.0
                    == "Preferences saved on this device. The world continues running.")
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
    fn focused_same_preset_retries_failed_save_without_automatic_or_background_retries() {
        let scratch = Scratch::new();
        let path = scratch.file();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        super::super::install_scale(&mut app, 1.);
        Config {
            debug_stats: false,
            fps_cap: default_fps_cap(),
            vsync: true,
            view_distance: ViewDistance::default(),
            path: path.clone(),
            scale: 1.,
            world_effects: None,
        }
        .install(&mut app);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                resolution: bevy::window::WindowResolution::new(1920, 1200),
                ..default()
            })
            .id();
        app.update();
        let button = |world: &mut World, title: &str| {
            world
                .query::<(Entity, &AccessibleLabel)>()
                .iter(world)
                .find(|(_, label)| label.0 == title)
                .unwrap()
                .0
        };
        let toggle = button(app.world_mut(), "Interface settings [F10]");
        let preset = button(app.world_mut(), "UI size 125%");
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: toggle });
        fs::create_dir(&path).unwrap();
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: preset });
        let wait_status = |app: &mut App, expected: fn(Status) -> bool| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                app.update();
                let store = app.world().resource::<Store>();
                if store.pending.is_none() && expected(store.status) {
                    break;
                }
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            app.update();
        };
        wait_status(&mut app, |status| matches!(status, Status::Failed));
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        let world = app.world_mut();
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0.contains("select a UI size to retry"))
        );
        fs::remove_dir(&path).unwrap();
        for _ in 0..20 {
            app.update();
        }
        assert!(
            !path.exists(),
            "Fixing the path alone must not retry a failed write"
        );
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: preset });
        app.update();
        assert!(!app.world().resource::<Store>().retry_requested);
        assert!(!path.exists());
        app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
        for _ in 0..2 {
            app.world_mut()
                .trigger(bevy::ui_widgets::Activate { entity: preset });
        }
        wait_status(&mut app, |status| matches!(status, Status::Saved));
        assert_eq!(read(&path).unwrap(), Some(1.25));
        assert_eq!(app.world().resource::<PreferredScale>().0, 1.25);
        assert!(!app.world().resource::<Store>().retry_requested);
        app.world_mut()
            .trigger(bevy::ui_widgets::Activate { entity: preset });
        app.update();
        assert!(
            app.world().resource::<Store>().pending.is_none(),
            "A saved preset needs no rewrite"
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
            debug_stats: false,
            fps_cap: default_fps_cap(),
            vsync: true,
            view_distance: ViewDistance::default(),
            path: path.clone(),
            scale: 1.,
            world_effects: None,
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
            debug_stats: false,
            fps_cap: default_fps_cap(),
            vsync: true,
            view_distance: ViewDistance::default(),
            path: path.clone(),
            scale: 1.,
            world_effects: None,
        }
        .install(&mut app);
        app.update();
        assert!(!path.exists(), "No launch-time write");
        app.world_mut().resource_mut::<PreferredScale>().0 = 1.5;
        app.world_mut()
            .resource_mut::<super::super::graphics::FrameCap>()
            .0 = 120;
        app.update();
        app.world_mut().resource_mut::<PreferredScale>().0 = 0.75;
        app.world_mut()
            .resource_mut::<super::super::graphics::FrameCap>()
            .0 = 144;
        app.world_mut()
            .resource_mut::<super::super::graphics::Vsync>()
            .0 = false;
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
        assert_eq!(fresh.fps_cap, 144);
        assert!(!fresh.vsync);
        assert_eq!(super::super::fit_scale(1.5, Vec2::new(1280., 720.)), 1.);
        assert_eq!(read(&path).unwrap(), Some(0.75));
    }
    #[test]
    fn world_effects_choice_persists_without_saving_a_startup_override() {
        let scratch = Scratch::new();
        let path = scratch.file();
        write_preferences(
            &path,
            1.25,
            Some(true),
            false,
            default_fps_cap(),
            true,
            ViewDistance::default(),
        )
        .unwrap();
        let mut args = vec![
            "--ui-preferences".into(),
            path.to_string_lossy().into_owned(),
        ];
        let config = Config::parse(&mut args, true, 1., false).unwrap().unwrap();
        assert_eq!(config.world_effects, Some(true));
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        super::super::install_scale(&mut app, config.scale);
        // An explicit one-session hide must not overwrite the saved preference.
        app.insert_resource(crate::particle_world::VisibilityChoice(false));
        config.install(&mut app);
        app.update();
        assert_eq!(
            read_settings(&path).unwrap().unwrap().world_effects,
            Some(true)
        );
        app.world_mut()
            .resource_mut::<crate::particle_world::VisibilityChoice>()
            .0 = true;
        app.update();
        app.world_mut()
            .resource_mut::<crate::particle_world::VisibilityChoice>()
            .0 = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            app.update();
            let store = app.world().resource::<Store>();
            if store.pending.is_none()
                && matches!(store.status, Status::Saved)
                && read_settings(&path).unwrap().unwrap().world_effects == Some(false)
            {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        let saved = read_settings(&path).unwrap().unwrap();
        assert_eq!(saved.ui_scale, 1.25);
        assert_eq!(saved.world_effects, Some(false));
        let mut args = vec![
            "--ui-preferences".into(),
            path.to_string_lossy().into_owned(),
        ];
        assert_eq!(
            Config::parse(&mut args, true, 1., false)
                .unwrap()
                .unwrap()
                .world_effects,
            Some(false)
        );
    }
}
