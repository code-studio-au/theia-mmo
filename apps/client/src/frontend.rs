//! P3-01/P3-03: native title, owned roster and cosmetic creation in one window.
mod creation;
mod login;
mod preview;
mod registration;
mod ui;
use bevy::prelude::*;
use serde::Deserialize;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
use theia_core::appearance::Appearance;
#[derive(Component)]
pub(crate) struct PreviewCamera;
pub(crate) type WorldCamera = (With<Camera3d>, Without<PreviewCamera>);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    Login,
    Register,
    Characters,
    Create,
    Entering,
}
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Character {
    id: String,
    name: String,
    #[serde(default)]
    appearance: Appearance,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: u8,
    serial: u64,
    phase: String,
    busy: bool,
    #[serde(default)]
    registration: bool,
    message: String,
    characters: Vec<Character>,
}
#[derive(Resource)]
pub(crate) struct Menu {
    directory: PathBuf,
    page: Page,
    characters: Vec<Character>,
    selected: Option<usize>,
    appearance: Appearance,
    category: creation::Category,
    yaw: f32,
    zoom: f32,
    message: String,
    serial: u64,
    pending: bool,
    registration: bool,
    last_snapshot: Vec<u8>,
    elapsed: f32,
    action: Option<ui::Choice>,
}
impl Menu {
    fn preview_visible(&self) -> bool {
        self.page == Page::Create
            || self.page == Page::Characters
                && self.selected.and_then(|i| self.characters.get(i)).is_some()
    }
    pub(crate) fn active(&self) -> bool {
        self.page != Page::Entering
    }
    fn clear_account(&mut self) {
        self.page = Page::Login;
        self.characters.clear();
        self.selected = None;
        self.appearance = Appearance::default();
        self.category = creation::Category::Hair;
        self.yaw = 0.;
        self.zoom = 1.;
    }
}
pub(crate) fn active(world: &World) -> bool {
    world.get_resource::<Menu>().is_some_and(Menu::active)
}
pub(crate) fn arguments(args: &mut Vec<String>) -> Result<Option<PathBuf>, String> {
    let Some(i) = args.iter().position(|a| a == "--game-front-end") else {
        return Ok(None);
    };
    args.remove(i);
    if i >= args.len() {
        return Err("Game front end requires a private session directory".into());
    }
    let directory = PathBuf::from(args.remove(i))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !directory.is_dir() {
        return Err("Invalid front-end directory".into());
    }
    Ok(Some(directory))
}
pub(crate) fn install(app: &mut App, directory: PathBuf) {
    app.insert_resource(Menu {
        directory,
        page: Page::Login,
        characters: vec![],
        selected: None,
        appearance: Appearance::default(),
        category: creation::Category::Hair,
        yaw: 0.,
        zoom: 1.,
        message: "Welcome to Theia Online".into(),
        serial: 0,
        pending: false,
        registration: false,
        last_snapshot: Vec::new(),
        elapsed: 0.,
        action: None,
    })
    .init_resource::<login::Login>()
    .init_resource::<registration::Form>()
    .add_systems(Startup, ui::setup)
    .add_observer(ui::choose)
    .add_observer(ui::rotate_drag)
    .add_systems(
        Update,
        (
            registration::keyboard,
            login::keyboard,
            login::actions,
            registration::actions,
            tick,
            preview::tick,
            ui::anchor,
            ui::sync,
            creation::sync,
            ui::name_validation,
            ui::rotate_keyboard,
            ui::focus,
            login::present,
            registration::present,
        )
            .chain()
            .before(crate::loading::observe),
    )
    .add_systems(Last, ui::evidence);
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::other("Menu file exceeds limit"));
    }
    Ok(bytes)
}
fn snapshot(bytes: &[u8]) -> Result<Snapshot, String> {
    let data: Snapshot = serde_json::from_slice(bytes).map_err(|_| "Invalid account response")?;
    if data.schema != 1
        || data.characters.len() > 8
        || data.phase == "login" && !data.characters.is_empty()
        || data.message.len() > 512
        || !matches!(data.phase.as_str(), "login" | "characters" | "entering")
    {
        return Err("Invalid account response".into());
    }
    let mut ids = std::collections::HashSet::new();
    for row in &data.characters {
        let id = uuid::Uuid::parse_str(&row.id).map_err(|_| "Invalid character")?;
        if id.is_nil()
            || id.to_string() != row.id
            || !ids.insert(id)
            || row.name.is_empty()
            || row.name.len() > 80
            || row.name.chars().any(char::is_control)
        {
            return Err("Invalid character".into());
        }
    }
    Ok(data)
}
fn send(menu: &mut Menu, mut command: serde_json::Value) -> Result<(), String> {
    command["serial"] = (menu.serial + 1).into();
    let pending = menu.directory.join("frontend-command.pending");
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec(&command).unwrap())
        .map_err(|e| e.to_string())?;
    drop(file);
    std::fs::rename(pending, menu.directory.join("frontend-command.json"))
        .map_err(|e| e.to_string())?;
    menu.serial += 1;
    menu.pending = true;
    menu.message = "Please wait…".into();
    Ok(())
}
fn tick(
    mut menu: ResMut<Menu>,
    time: Res<Time>,
    mut names: Query<
        &mut bevy::text::EditableText,
        (With<ui::NameInput>, Without<login::Username>),
    >,
    mut usernames: Query<
        &mut bevy::text::EditableText,
        (With<login::Username>, Without<ui::NameInput>),
    >,
    mut login: ResMut<login::Login>,
    mut exit: MessageWriter<AppExit>,
) {
    menu.elapsed += time.delta_secs();
    if menu.active() && menu.elapsed >= 0.15 {
        menu.elapsed = 0.;
        match read(&menu.directory.join("frontend-state.json"), 8192) {
            Ok(bytes) => match snapshot(&bytes) {
                Ok(data) if data.serial >= menu.serial && bytes != menu.last_snapshot => {
                    menu.last_snapshot = bytes;
                    let changed = menu.characters.len() != data.characters.len();
                    menu.characters = data.characters;
                    menu.message = data.message;
                    menu.serial = data.serial;
                    menu.pending = data.busy;
                    menu.registration = data.registration;
                    if data.phase == "login" && !matches!(menu.page, Page::Login | Page::Register) {
                        menu.clear_account();
                        login.clear_account();
                        for mut input in &mut names {
                            input.clear();
                        }
                        for mut input in &mut usernames {
                            input.clear();
                        }
                    } else if data.phase == "entering" {
                        menu.page = Page::Entering;
                    } else if data.phase == "characters"
                        && (matches!(menu.page, Page::Login | Page::Register) || changed)
                    {
                        menu.page = Page::Characters;
                        menu.selected = menu.characters.len().checked_sub(1);
                        if let Some(i) = menu.selected {
                            menu.appearance = menu.characters[i].appearance;
                        }
                    }
                }
                Ok(_) => (),
                Err(e) => {
                    menu.message = e;
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => {
                menu.message = "Could not read account status".into();
            }
        }
    }
    let Some(action) = menu.action.take() else {
        return;
    };
    if action == ui::Choice::Exit {
        exit.write(AppExit::Success);
        return;
    }
    if menu.pending || !menu.active() {
        return;
    }
    use ui::Choice;
    let result = match action {
        Choice::New if menu.characters.len() < 8 => {
            menu.page = Page::Create;
            menu.message.clear();
            menu.appearance = Appearance::default();
            menu.category = creation::Category::Hair;
            menu.yaw = 0.;
            menu.zoom = 1.;
            if let Ok(mut name) = names.single_mut() {
                name.clear();
            }
            Ok(())
        }
        Choice::Zoom(closer) if menu.preview_visible() => {
            menu.zoom = (menu.zoom + if closer { 0.1 } else { -0.1 }).clamp(0.75, 1.3);
            Ok(())
        }
        Choice::ResetView if menu.preview_visible() => {
            menu.zoom = 1.;
            menu.yaw = 0.;
            Ok(())
        }
        Choice::SignOut if menu.page == Page::Characters => {
            send(&mut menu, serde_json::json!({"action":"logout"}))
        }
        Choice::Back => {
            menu.page = Page::Characters;
            if let Some(i) = menu.selected {
                menu.appearance = menu.characters[i].appearance;
            }
            Ok(())
        }
        Choice::Select(i) if i < menu.characters.len() => {
            menu.selected = Some(i);
            menu.appearance = menu.characters[i].appearance;
            Ok(())
        }
        Choice::Category(category) if menu.page == Page::Create => {
            menu.category = category;
            Ok(())
        }
        Choice::Randomize if menu.page == Page::Create => {
            menu.appearance = creation::randomized(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            );
            Ok(())
        }
        Choice::Appearance(field, value) if menu.page == Page::Create => {
            menu.appearance = creation::select(menu.appearance, field, value);
            Ok(())
        }
        Choice::Enter => {
            if let Some(i) = menu.selected {
                let id = menu.characters[i].id.clone();
                send(&mut menu, serde_json::json!({"action":"enter","id":id}))
            } else {
                Err("Choose an adventurer first.".into())
            }
        }
        Choice::Create if menu.page == Page::Create => {
            let name = names
                .single()
                .map(|n| n.value().to_string())
                .unwrap_or_default();
            match theia_core::character_name::CharacterName::parse(name.trim()) {
                Ok(_) => {
                    let appearance = menu.appearance;
                    send(
                        &mut menu,
                        serde_json::json!({"action":"create","name":name,"appearance":appearance}),
                    )
                }
                Err(_) => {
                    // Live inline validation owns name-format feedback.
                    menu.message.clear();
                    Ok(())
                }
            }
        }
        _ => Ok(()),
    };
    if let Err(e) = result {
        menu.message = e;
    }
}
/// Wait outside Bevy's frame loop; cancellation remains live while login is open.
pub(crate) async fn admission(
    directory: &Path,
    ticket: &Path,
    mut stop: tokio::sync::watch::Receiver<bool>,
) -> Result<(std::net::SocketAddr, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Admission {
        address: std::net::SocketAddr,
    }
    loop {
        if *stop.borrow() {
            return Err("Game entry cancelled".into());
        }
        match read(&directory.join("frontend-admission.json"), 256) {
            Ok(bytes) => {
                let data: Admission = serde_json::from_slice(&bytes)?;
                if !data.address.ip().is_loopback() || data.address.port() == 0 {
                    return Err("Invalid game endpoint".into());
                }
                return Ok((data.address, read(ticket, 2048)?));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        tokio::select! { _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => (), _ = stop.changed() => { if *stop.borrow() { return Err("Game entry cancelled".into()); } } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_snapshot_rejects_duplicate_identity_unknown_phase_and_bad_appearance() {
        let row = serde_json::json!({"id":"12345678-1234-4234-8234-123456789abc", "name":"Arden", "appearance":Appearance::default()});
        let mut value = serde_json::json!({"schema":1,"serial":0,"phase":"characters","busy":false,"message":"Ready","characters":[row.clone()]});
        assert!(snapshot(&serde_json::to_vec(&value).unwrap()).is_ok());
        value["phase"] = "login".into();
        assert!(snapshot(&serde_json::to_vec(&value).unwrap()).is_err());
        value["phase"] = "characters".into();
        value["characters"] = serde_json::json!([row.clone(), row]);
        assert!(snapshot(&serde_json::to_vec(&value).unwrap()).is_err());
        value["characters"] = serde_json::json!([]);
        value["phase"] = "trusted".into();
        assert!(snapshot(&serde_json::to_vec(&value).unwrap()).is_err());
        value["phase"] = "characters".into();
        value["characters"] = serde_json::json!([{"id":"12345678-1234-4234-8234-123456789abc","name":"Arden","appearance":{"schema":99}}]);
        assert!(snapshot(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn empty_accounts_have_no_preview_even_after_abandoning_creation() {
        let mut menu = Menu {
            directory: PathBuf::new(),
            page: Page::Characters,
            characters: vec![],
            selected: None,
            appearance: Appearance::default(),
            category: creation::Category::Hair,
            yaw: 0.,
            zoom: 1.,
            message: String::new(),
            serial: 0,
            pending: false,
            registration: false,
            last_snapshot: vec![],
            elapsed: 0.,
            action: None,
        };
        assert!(!menu.preview_visible());
        menu.page = Page::Create;
        assert!(menu.preview_visible());
        menu.page = Page::Characters;
        assert!(!menu.preview_visible());
        menu.characters.push(Character {
            id: "12345678-1234-4234-8234-123456789abc".into(),
            name: "Arden".into(),
            appearance: Appearance::default(),
        });
        assert!(!menu.preview_visible());
        menu.selected = Some(0);
        assert!(menu.preview_visible());
        menu.yaw = 2.;
        menu.zoom = 1.3;
        menu.clear_account();
        assert_eq!(menu.page, Page::Login);
        assert!(menu.characters.is_empty());
        assert!(menu.selected.is_none());
        assert_eq!(menu.appearance, Appearance::default());
        assert_eq!((menu.yaw, menu.zoom), (0., 1.));
        assert!(!menu.preview_visible());
        menu.page = Page::Entering;
        assert!(!menu.preview_visible());
    }
    #[tokio::test]
    async fn deferred_admission_honours_stop_without_a_ticket() {
        let (_, stop) = tokio::sync::watch::channel(true);
        let result = admission(Path::new("/missing"), Path::new("/missing/ticket"), stop).await;
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }
    #[test]
    fn gameplay_camera_queries_exclude_the_character_preview() {
        let mut world = World::new();
        let game = world.spawn(Camera3d::default()).id();
        world.spawn((Camera3d::default(), PreviewCamera));
        assert_eq!(
            world
                .query_filtered::<Entity, WorldCamera>()
                .single(&world)
                .unwrap(),
            game
        );
    }
}
