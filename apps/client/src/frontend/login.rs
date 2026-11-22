//! Native credential fields, private in-memory handoff, and front-end dialogs.
use super::*;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{
        FocusCause, InputFocus,
        tab_navigation::{TabGroup, TabIndex},
    },
    tasks::{IoTaskPool, Task, futures::check_ready},
    text::EditableText,
    ui_widgets::TextInput,
};
#[derive(Component)]
pub(super) struct Username;
#[derive(Component)]
pub(super) struct MenuLink;
#[derive(Component)]
pub(super) struct PasswordLabel;
#[derive(Component)]
pub(super) struct LoginMessage;
#[derive(Component)]
pub(super) struct DialogRoot;
#[derive(Component)]
pub(super) struct SettingsBody;
#[derive(Component)]
pub(super) struct SupportBody;
#[derive(Component)]
pub(super) struct GraphicsSummary;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Password,
    Reveal,
    Settings,
    Support,
    Close,
    Vsync(bool),
    Cap(u16),
    Distance(crate::fantasy_ui::graphics::ViewDistance),
}
#[derive(Clone, Copy, PartialEq)]
enum Dialog {
    Settings,
    Support,
}
#[derive(Resource, Default)]
pub(super) struct Login {
    password: Vec<char>,
    attempted: bool,
    cursor: usize,
    select_all: bool,
    revealed: bool,
    dialog: Option<Dialog>,
    task: Option<Task<Result<(), String>>>,
}
impl Login {
    fn clear(&mut self) {
        self.password.fill('\0');
        self.password.clear();
        self.cursor = 0;
        self.select_all = false;
        self.revealed = false;
    }
    fn edit(&mut self, key: &Key, control: bool) {
        if control {
            if matches!(key, Key::Character(c) if c.eq_ignore_ascii_case("a")) {
                self.select_all = true;
            }
            return;
        }
        match key {
            Key::ArrowLeft => {
                self.cursor = self.cursor.saturating_sub(1);
                self.select_all = false;
            }
            Key::ArrowRight => {
                self.cursor = (self.cursor + 1).min(self.password.len());
                self.select_all = false;
            }
            Key::Home => {
                self.cursor = 0;
                self.select_all = false;
            }
            Key::End => {
                self.cursor = self.password.len();
                self.select_all = false;
            }
            Key::Backspace | Key::Delete if self.select_all => self.clear(),
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.password.remove(self.cursor);
            }
            Key::Delete if self.cursor < self.password.len() => {
                self.password.remove(self.cursor);
            }
            Key::Character(value) => {
                if self.select_all {
                    self.clear();
                }
                for c in value.chars().filter(|c| !c.is_control()) {
                    if self.password.len() >= 1024 {
                        break;
                    }
                    self.password.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
            Key::Space => self.edit(&Key::Character(" ".into()), false),
            _ => (),
        }
    }
}
impl Drop for Login {
    fn drop(&mut self) {
        self.clear();
    }
}
fn button(parent: &mut ChildSpawnerCommands, action: Action, text: &str) {
    ui::button(parent, ui::Choice::Account(action), text);
}
fn field_label(parent: &mut ChildSpawnerCommands, caption: &str, icon: usize) {
    parent
        .spawn(Node {
            column_gap: px(8),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                crate::ui_theme::Icon(icon),
                Node {
                    width: px(24),
                    height: px(24),
                    ..default()
                },
            ));
            row.spawn(ui::text(caption, 18.));
        });
}
pub(super) fn panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        ui::Screen(Page::Login),
        crate::ui_theme::Panel,
        Node {
            width: px(480),
            max_width: percent(90),
            padding: UiRect::all(px(28)),
            border: UiRect::all(px(2)),
            ..ui::column()
        },
        BackgroundColor(Color::srgba(0.018, 0.027, 0.033, 0.95)),
        BorderColor::all(ui::GOLD),
        Outline::new(px(1), px(5), ui::GOLD),
    ))
    .with_children(|panel| {
        field_label(panel, "Username", 0);
        panel.spawn((
            Username,
            TextInput,
            TabIndex(0),
            EditableText {
                max_characters: Some(256),
                allow_newlines: false,
                ..default()
            },
            Node {
                min_height: px(44),
                padding: UiRect::all(px(8)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(ui::GOLD),
            BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
            TextFont::from_font_size(20.),
            TextColor(ui::CREAM),
            TextLayout::no_wrap(),
            bevy::text::TextCursorStyle {
                color: ui::CREAM,
                ..default()
            },
        ));
        field_label(panel, "Password", 1);
        panel
            .spawn(Node {
                column_gap: px(8),
                ..default()
            })
            .with_children(|row| {
                let field = ui::button(row, ui::Choice::Account(Action::Password), "");
                row.commands()
                    .entity(field)
                    .insert((
                        AccessibleLabel::new("Password"),
                        Node {
                            min_width: px(0),
                            flex_grow: 1.,
                            min_height: px(44),
                            padding: UiRect::all(px(8)),
                            border: UiRect::all(px(1)),
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                    ))
                    .with_child((PasswordLabel, ui::text("", 20.)));
                button(row, Action::Reveal, "Show");
            });
        let enter = ui::button(panel, ui::Choice::Login, "Log In");
        panel.commands().entity(enter).insert(Node {
            min_height: px(48),
            margin: UiRect::top(px(10)),
            border: UiRect::all(px(1)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        });
        panel.spawn((
            LoginMessage,
            ui::text("", 15.),
            Node {
                display: Display::None,
                ..default()
            },
        ));
    });
}
pub(super) fn links(root: &mut ChildSpawnerCommands) {
    root.spawn((ui::Placement::Links, ui::column()))
        .with_children(|links| {
            for (icon, (choice, caption)) in [
                (ui::Choice::Account(Action::Settings), "Settings"),
                (ui::Choice::Account(Action::Support), "Support"),
                (ui::Choice::Exit, "Exit"),
            ]
            .into_iter()
            .enumerate()
            {
                let entity = ui::button(links, choice, caption);
                links
                    .commands()
                    .entity(entity)
                    .insert((
                        MenuLink,
                        crate::ui_theme::Style::Link,
                        Node {
                            min_height: px(32),
                            padding: UiRect::axes(px(8), px(4)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_child((
                        crate::ui_theme::Icon(icon + 3),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(-24),
                            width: px(26),
                            height: px(26),
                            ..default()
                        },
                    ));
            }
        });
    root.spawn((DialogRoot,GlobalZIndex(1200),TabGroup::modal(),Visibility::Hidden,
        Node{position_type:PositionType::Absolute,width:percent(100),height:percent(100),display:Display::None,
            justify_content:JustifyContent::Center,align_items:AlignItems::Center,..default()},
        BackgroundColor(Color::srgba(0.,0.,0.,0.7)))).with_children(|overlay| {
        overlay.spawn((crate::ui_theme::Panel, Node{width:px(500),max_width:percent(90),padding:UiRect::all(px(24)),border:UiRect::all(px(1)),..ui::column()},
            BackgroundColor(Color::srgb(0.025,0.035,0.04)),BorderColor::all(ui::GOLD))).with_children(|pane| {
            pane.spawn((SettingsBody,ui::column())).with_children(|settings| {
                settings.spawn(ui::text("Graphics settings",26.));
                crate::ui_theme::controls(settings);
                settings.spawn((GraphicsSummary,ui::text("",17.)));
                settings.spawn(Node{column_gap:px(8),..default()}).with_children(|r| {
                    button(r,Action::Vsync(true),"VSync On");button(r,Action::Vsync(false),"VSync Off");
                });
                settings.spawn(ui::text("Frame limit",17.));
                settings.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()}).with_children(|r| {
                    for value in crate::fantasy_ui::graphics::CAPS {
                        button(r,Action::Cap(value),&crate::fantasy_ui::graphics::FrameCap(value).label());
                    }
                });
                settings.spawn(ui::text("Terrain view distance",17.));
                settings.spawn(Node{flex_wrap:FlexWrap::Wrap,column_gap:px(8),row_gap:px(8),..default()}).with_children(|r| {
                    for value in crate::fantasy_ui::graphics::ViewDistance::ALL { button(r,Action::Distance(value),value.label()); }
                });
            });
            pane.spawn((SupportBody,ui::column())).with_children(|support| {
                support.spawn(ui::text("Support",26.));
                support.spawn(ui::text("Check your username and password, then try Log In again. Passwords are case-sensitive.",18.));
                support.spawn(ui::text("For account access or password recovery, contact your game administrator. This preview uses local development accounts.",16.));
            });
            button(pane,Action::Close,"Close");
        });
    });
}
pub(super) fn keyboard(
    mut input: MessageReader<KeyboardInput>,
    mut focus: ResMut<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    choices: Query<&ui::Choice>,
    windows: Query<&Window>,
    mut login: ResMut<Login>,
    mut menu: ResMut<Menu>,
) {
    let password = focus.get().and_then(|e| choices.get(e).ok())
        == Some(&ui::Choice::Account(Action::Password));
    if !menu.active() || !windows.single().is_ok_and(|w| w.focused) {
        login.revealed = false;
        input.clear();
        return;
    }
    let control = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight);
    for event in input.read().filter(|e| e.state == ButtonState::Pressed) {
        if event.logical_key == Key::Escape {
            login.dialog = None;
            login.revealed = false;
            focus.clear();
            continue;
        }
        if event.logical_key == Key::F10 {
            login.dialog = Some(Dialog::Settings);
            login.revealed = false;
            continue;
        }
        if menu.page != Page::Login || menu.pending || login.dialog.is_some() {
            continue;
        }
        if password {
            if event.logical_key == Key::Enter {
                menu.action = Some(ui::Choice::Login);
            } else {
                login.edit(&event.logical_key, control);
            }
        }
    }
}
pub(super) fn actions(world: &mut World) {
    world.resource_scope(|world, mut login: Mut<Login>| {
        if let Some(task) = login.task.as_mut()
            && let Some(result) = check_ready(task)
        {
            login.task = None;
            if let Err(error) = result {
                let mut menu = world.resource_mut::<Menu>();
                menu.pending = false;
                menu.serial = menu.serial.saturating_sub(1);
                menu.message = error;
            }
        }
        if world.resource::<Menu>().page != Page::Login {
            login.clear();
        }
        let Some(choice) = world.resource::<Menu>().action else {
            return;
        };
        if !matches!(choice, ui::Choice::Account(_) | ui::Choice::Login) {
            return;
        }
        world.resource_mut::<Menu>().action = None;
        let pending = world.resource::<Menu>().pending;
        if pending
            && !matches!(
                choice,
                ui::Choice::Account(Action::Settings | Action::Support | Action::Close)
            )
        {
            return;
        }
        let action = match choice {
            ui::Choice::Account(action) => action,
            ui::Choice::Login => {
                if world.resource::<Menu>().page != Page::Login {
                    return;
                }
                login.attempted = true;
                let username = world
                    .query_filtered::<&EditableText, With<Username>>()
                    .single(world)
                    .map(|u| u.value().to_string())
                    .unwrap_or_default();
                if username.trim().is_empty() || login.password.is_empty() {
                    world.resource_mut::<Menu>().message =
                        "Enter your username and password.".into();
                    return;
                }
                let password: String = login.password.iter().collect();
                login.clear();
                let mut menu = world.resource_mut::<Menu>();
                let path = menu.directory.join("frontend-credentials.json");
                menu.serial += 1;
                let serial = menu.serial;
                menu.pending = true;
                menu.message = "Signing in…".into();
                login.task = Some(
                    IoTaskPool::get()
                        .spawn(async move { submit(path, serial, username, password) }),
                );
                return;
            }
            _ => return,
        };
        match action {
            Action::Password => {
                if let Some(entity) = world
                    .query::<(Entity, &ui::Choice)>()
                    .iter(world)
                    .find(|(_, c)| **c == choice)
                    .map(|(e, _)| e)
                {
                    world
                        .resource_mut::<InputFocus>()
                        .set(entity, FocusCause::Navigated);
                }
            }
            Action::Reveal => {
                login.revealed = !login.revealed;
            }
            Action::Settings => {
                login.revealed = false;
                login.dialog = Some(Dialog::Settings);
            }
            Action::Support => {
                login.revealed = false;
                login.dialog = Some(Dialog::Support);
            }
            Action::Close => {
                login.dialog = None;
                world.resource_mut::<InputFocus>().clear();
            }
            Action::Vsync(v) => {
                if let Some(mut r) = world.get_resource_mut::<crate::fantasy_ui::graphics::Vsync>()
                {
                    r.0 = v;
                }
            }
            Action::Cap(v) => {
                if let Some(mut r) =
                    world.get_resource_mut::<crate::fantasy_ui::graphics::FrameCap>()
                {
                    r.0 = v;
                }
            }
            Action::Distance(v) => {
                if let Some(mut r) =
                    world.get_resource_mut::<crate::fantasy_ui::graphics::ViewDistance>()
                {
                    *r = v;
                }
            }
        }
        if login.dialog.is_some() && matches!(action, Action::Settings | Action::Support) {
            let close = world
                .query::<(Entity, &ui::Choice)>()
                .iter(world)
                .find(|(_, c)| **c == ui::Choice::Account(Action::Close))
                .map(|(e, _)| e);
            if let Some(e) = close {
                world
                    .resource_mut::<InputFocus>()
                    .set(e, FocusCause::Navigated);
            }
        }
    });
}
fn submit(
    path: PathBuf,
    serial: u64,
    username: String,
    mut password: String,
) -> Result<(), String> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Endpoint {
        address: std::net::SocketAddr,
        token: String,
    }
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let endpoint: Endpoint = serde_json::from_slice(&read(&path, 512)?)?;
        if endpoint.address.ip() != std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            || endpoint.address.port() == 0
            || endpoint.token.len() != 64
        {
            return Err("invalid endpoint".into());
        }
        let mut stream = std::net::TcpStream::connect_timeout(
            &endpoint.address,
            std::time::Duration::from_secs(2),
        )?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(2)))?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
        let mut bytes = serde_json::to_vec(
            &serde_json::json!({"serial":serial,"action":"credentials","username":username,"password":password,"token":endpoint.token}),
        )?;
        let written = (|| -> std::io::Result<()> {
            stream.write_all(&(bytes.len() as u32).to_be_bytes())?;
            stream.write_all(&bytes)
        })();
        bytes.fill(0);
        written?;
        let mut ack = [0];
        stream.read_exact(&mut ack)?;
        if ack != [1] {
            return Err("rejected".into());
        }
        Ok(())
    })();
    password.clear();
    result.map_err(|_| "Could not contact sign-in. Please try again.".into())
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn present(
    login: Res<Login>,
    menu: Res<Menu>,
    focus: Res<InputFocus>,
    choices: Query<&ui::Choice>,
    mut labels: Query<(
        &mut Text,
        Has<PasswordLabel>,
        Has<LoginMessage>,
        Has<GraphicsSummary>,
        Option<&ChildOf>,
    )>,
    mut nodes: Query<(
        &mut Node,
        &mut Visibility,
        Has<DialogRoot>,
        Has<SettingsBody>,
        Has<SupportBody>,
        Has<LoginMessage>,
    )>,
    cap: Option<Res<crate::fantasy_ui::graphics::FrameCap>>,
    vsync: Option<Res<crate::fantasy_ui::graphics::Vsync>>,
    view: Option<Res<crate::fantasy_ui::graphics::ViewDistance>>,
) {
    let focused = focus.get().and_then(|e| choices.get(e).ok())
        == Some(&ui::Choice::Account(Action::Password));
    for (mut text, password, message, summary, parent) in &mut labels {
        if password {
            let start = login.cursor.saturating_sub(28);
            **text = login
                .password
                .iter()
                .skip(start)
                .take(36)
                .map(|c| if login.revealed { *c } else { '•' })
                .collect();
            if focused {
                text.push('│');
            }
        }
        if message {
            **text = if login.attempted || menu.pending {
                menu.message.clone()
            } else {
                String::new()
            };
        }
        if summary {
            **text = format!(
                "{} · VSync {} · {}",
                cap.as_deref().copied().unwrap_or_default().label(),
                if vsync.as_ref().is_none_or(|v| v.0) {
                    "On"
                } else {
                    "Off"
                },
                view.as_deref().copied().unwrap_or_default().label()
            );
        }
        if let Some(parent) = parent
            && choices.get(parent.parent()) == Ok(&ui::Choice::Account(Action::Reveal))
        {
            **text = if login.revealed { "Hide" } else { "Show" }.into();
        }
    }
    for (mut node, mut vis, dialog, settings, support, message) in &mut nodes {
        let show = if dialog {
            login.dialog.is_some()
        } else if settings {
            login.dialog == Some(Dialog::Settings)
        } else if support {
            login.dialog == Some(Dialog::Support)
        } else if message {
            login.attempted || menu.pending
        } else {
            continue;
        };
        node.display = if show { Display::Flex } else { Display::None };
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn password_editing_bounds_cursor_and_clears_selection_without_plaintext_ui() {
        let mut login = Login::default();
        login.edit(&Key::Character("sëcret".into()), false);
        login.edit(&Key::ArrowLeft, false);
        login.edit(&Key::Backspace, false);
        assert_eq!(login.password.iter().collect::<String>(), "sëcrt");
        login.edit(&Key::Character("a".into()), true);
        login.edit(&Key::Character("new".into()), false);
        assert_eq!(login.password.iter().collect::<String>(), "new");
        login.edit(&Key::Character("x".repeat(1100).into()), false);
        assert_eq!(login.password.len(), 1024);
        login.clear();
        assert!(login.password.is_empty());
        assert!(!login.revealed);
    }
}
