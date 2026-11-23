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
    ui_widgets::{ActivateOnPress, TextInput},
};
#[derive(Component)]
pub(super) struct Username;
#[derive(Component)]
pub(super) struct UsernamePlaceholder;
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
pub(super) struct InterfaceBody;
#[derive(Component)]
pub(super) struct SupportBody;
#[derive(Component)]
pub(super) struct GraphicsSummary;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Password,
    Reveal,
    Settings,
    Interface,
    Support,
    Close,
    Cap(u16),
    Distance(crate::fantasy_ui::graphics::ViewDistance),
}
#[derive(Clone, Copy, PartialEq)]
enum Dialog {
    Settings,
    Interface,
    Support,
}
#[derive(Resource, Default)]
pub(super) struct Login {
    secret: Secret,
    attempted: bool,
    dialog: Option<Dialog>,
    task: Option<Task<Result<(), String>>>,
}
impl Login {
    pub(super) fn clear_account(&mut self) {
        self.secret.clear();
        self.attempted = false;
        self.dialog = None;
        self.task = None;
    }
}
#[derive(Default)]
pub(super) struct Secret {
    pub(super) password: Vec<char>,
    pub(super) cursor: usize,
    pub(super) select_all: bool,
    pub(super) revealed: bool,
}
impl Secret {
    pub(super) fn clear(&mut self) {
        self.password.fill('\0');
        self.password.clear();
        self.cursor = 0;
        self.select_all = false;
        self.revealed = false;
    }
    pub(super) fn edit(&mut self, key: &Key, control: bool) {
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
impl Drop for Secret {
    fn drop(&mut self) {
        self.clear();
    }
}
pub(super) fn panel(root: &mut ChildSpawnerCommands) {
    root.spawn((
        ui::Screen(Page::Login),
        crate::ui_theme::Panel,
        Node {
            width: px(480),
            row_gap: px(18),
            max_width: percent(90),
            padding: UiRect::all(px(28)),
            border: UiRect::all(px(2)),
            ..ui::column()
        },
        BackgroundColor(crate::ui_theme::PANEL),
        BorderColor::all(ui::GOLD),
        Outline::new(px(1), px(5), ui::GOLD),
    ))
    .with_children(|panel| {
        panel
            .spawn((
                Username,
                TextInput,
                TabIndex(0),
                EditableText {
                    max_characters: Some(256),
                    allow_newlines: false,
                    ..default()
                },
                crate::ui_theme::input_box(true, false),
                BorderColor::all(ui::GOLD),
                BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
                TextFont::from_font_size(20.),
                TextColor(ui::CREAM),
                TextLayout::no_wrap(),
                bevy::text::TextCursorStyle {
                    color: ui::CREAM,
                    ..default()
                },
            ))
            .with_children(|field| {
                field.spawn((
                    crate::ui_theme::Icon(0),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(10),
                        top: px(12),
                        width: px(24),
                        height: px(24),
                        ..default()
                    },
                ));
                field.spawn((
                    UsernamePlaceholder,
                    ui::text("Account Name", 20.),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(crate::ui_theme::INPUT_ICON_TEXT_INSET),
                        top: px(10),
                        ..default()
                    },
                ));
            });
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
                        crate::ui_theme::input_box(true, true),
                    ))
                    .with_child((PasswordLabel, ui::text("Password", 20.)))
                    .with_child((
                        crate::ui_theme::Icon(1),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(10),
                            top: px(12),
                            width: px(24),
                            height: px(24),
                            ..default()
                        },
                    ));
                let eye = ui::button(row, ui::Choice::Account(Action::Reveal), "");
                row.commands()
                    .entity(eye)
                    .insert((
                        crate::ui_theme::Style::Icon,
                        AccessibleLabel::new("Show password"),
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(8),
                            top: px(8),
                            width: px(32),
                            height: px(32),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_child((
                        crate::ui_theme::Icon(2),
                        Node {
                            width: px(24),
                            height: px(24),
                            ..default()
                        },
                    ));
            });
        panel
            .spawn(Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|r| {
                crate::fantasy_ui::kit::unsupported_check(r, "Remember Me");
                crate::fantasy_ui::kit::disabled_link(r, "Forgot Password?");
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
        let create = ui::button(
            panel,
            ui::Choice::Registration(registration::Action::Open),
            "Create Account",
        );
        panel.commands().entity(create).insert(Node {
            min_height: px(44),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
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
    root.spawn((DialogRoot,crate::fantasy_ui::kit::SettingsSurface,GlobalZIndex(1200),TabGroup::modal(),Visibility::Hidden,
        Node{position_type:PositionType::Absolute,width:percent(100),height:percent(100),display:Display::None,
            justify_content:JustifyContent::Center,align_items:AlignItems::Center,..default()},
        BackgroundColor(Color::srgba(0.,0.,0.,0.7)))).with_children(|overlay| {
        overlay.spawn((crate::ui_theme::Panel, crate::fantasy_ui::kit::SettingsFrame, UiTransform::default(), Node{width:px(1440),height:px(940),flex_shrink:0.,padding:UiRect::all(px(24)),..ui::column()},
            BackgroundColor(crate::ui_theme::PANEL))).with_children(|pane| {
            crate::fantasy_ui::kit::settings_panel_gradient(pane);
            crate::fantasy_ui::kit::settings_header(pane, ui::Choice::Account(Action::Close));
            pane.spawn(crate::fantasy_ui::kit::settings_body()).with_children(|body| {
                body.spawn(crate::fantasy_ui::kit::settings_sidebar()).with_children(|nav| {
                    for (index,title,action) in [(9,"Graphics",Some(Action::Settings)),(10,"Audio",None),(2,"Gameplay",None),(11,"Controls",None),(12,"Key Bindings",None),(13,"Interface",Some(Action::Interface)),(14,"HUD",None),(15,"Social",None),(16,"Accessibility",None),(17,"Language",None),(24,"Support",Some(Action::Support))] {
                        let tab = crate::fantasy_ui::kit::settings_tab(nav, title, index, action.map(ui::Choice::Account));
                        if action.is_some() {
                            nav.commands().entity(tab).insert((ActivateOnPress, TabIndex(0)));
                        }
                    }
                    crate::fantasy_ui::kit::settings_watermark(nav);
                });
                crate::fantasy_ui::kit::settings_content(body).with_children(|content| {
                    content.spawn((SettingsBody,ui::column())).with_children(|settings| {
                        crate::fantasy_ui::kit::graphics_heading(settings);
                        crate::fantasy_ui::kit::graphics(settings,|row,value| match value {
                            crate::fantasy_ui::kit::Value::FrameCap => {for value in crate::fantasy_ui::graphics::CAPS {
                                crate::fantasy_ui::kit::dropdown_option(row, ui::Choice::Account(Action::Cap(value)), &crate::fantasy_ui::graphics::FrameCap(value).label())
                                    .insert((ActivateOnPress, TabIndex(0)));
                            }},
                            crate::fantasy_ui::kit::Value::Distance => {for value in crate::fantasy_ui::graphics::ViewDistance::ALL {
                                crate::fantasy_ui::kit::dropdown_option(row, ui::Choice::Account(Action::Distance(value)), value.label())
                                    .insert((ActivateOnPress, TabIndex(0)));
                            }},
                            _ => {}
                        });
                    });
                    content.spawn((InterfaceBody,ui::column())).with_children(|settings| {
                        crate::ui_theme::heading(settings,"Interface");
                        crate::ui_theme::scale_control(settings);
                    });
                    content.spawn((SupportBody,ui::column())).with_children(|support| {
                        crate::ui_theme::heading(support,"Support");
                        support.spawn(ui::text("Check your username and password, then try Log In again. Passwords are case-sensitive.",17.));
                        support.spawn(ui::text("For account access or password recovery, contact your game administrator. This preview uses local development accounts.",16.));
                    });
                });
            });
            pane.spawn(crate::fantasy_ui::kit::settings_footer()).with_children(|footer| {
                crate::fantasy_ui::kit::defaults(footer);
                footer.spawn(Node{column_gap:px(12),..default()}).with_children(|r| {
                    let apply=crate::fantasy_ui::kit::footer_button(r,crate::fantasy_ui::kit::Footer::Apply,"Apply");r.commands().entity(apply).insert(crate::ui_theme::Style::Gold);
                    let ok=crate::fantasy_ui::kit::footer_button(r,crate::fantasy_ui::kit::Footer::Accept,"OK");r.commands().entity(ok).insert((ui::Choice::Account(Action::Close),crate::ui_theme::Style::Primary));
                    let cancel=crate::fantasy_ui::kit::footer_button(r,crate::fantasy_ui::kit::Footer::Cancel,"Cancel");r.commands().entity(cancel).insert(ui::Choice::Account(Action::Close));
                });
            });
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
        login.secret.revealed = false;
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
            login.secret.revealed = false;
            focus.clear();
            continue;
        }
        if event.logical_key == Key::F10 {
            login.dialog = Some(Dialog::Settings);
            login.secret.revealed = false;
            continue;
        }
        if menu.page != Page::Login || menu.pending || login.dialog.is_some() {
            continue;
        }
        if password {
            if event.logical_key == Key::Enter {
                menu.action = Some(ui::Choice::Login);
            } else {
                login.secret.edit(&event.logical_key, control);
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
            login.secret.clear();
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
                ui::Choice::Account(
                    Action::Settings | Action::Interface | Action::Support | Action::Close
                )
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
                if username.trim().is_empty() || login.secret.password.is_empty() {
                    world.resource_mut::<Menu>().message =
                        "Enter your username and password.".into();
                    return;
                }
                let password: String = login.secret.password.iter().collect();
                login.secret.clear();
                let mut menu = world.resource_mut::<Menu>();
                let path = menu.directory.join("frontend-credentials.json");
                menu.serial += 1;
                let serial = menu.serial;
                menu.pending = true;
                menu.message = "Signing in…".into();
                login.task = Some(
                    IoTaskPool::get()
                        .spawn(async move { submit(path, serial, username, password, None) }),
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
                login.secret.revealed = !login.secret.revealed;
            }
            Action::Interface => {
                login.dialog = Some(Dialog::Interface);
                login.secret.revealed = false;
            }
            Action::Settings => {
                login.secret.revealed = false;
                login.dialog = Some(Dialog::Settings);
            }
            Action::Support => {
                login.secret.revealed = false;
                login.dialog = Some(Dialog::Support);
            }
            Action::Close => {
                login.dialog = None;
                world.resource_mut::<InputFocus>().clear();
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
pub(super) fn submit(
    path: PathBuf,
    serial: u64,
    username: String,
    mut password: String,
    email: Option<String>,
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
        let mut payload = serde_json::json!({"serial":serial,"action":"credentials","username":username,"password":password,"token":endpoint.token});
        if let Some(email) = email {
            payload["action"] = "register".into();
            payload["email"] = email.into();
        }
        let mut bytes = serde_json::to_vec(&payload)?;
        drop(payload);
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
    mut commands: Commands,
    accessible: Query<(Entity, &ui::Choice)>,
    usernames: Query<&EditableText, With<Username>>,
    mut labels: Query<(
        &mut Text,
        Has<PasswordLabel>,
        Has<UsernamePlaceholder>,
        Has<LoginMessage>,
        Has<GraphicsSummary>,
        Option<&ChildOf>,
    )>,
    mut nodes: Query<(
        &mut Node,
        &mut Visibility,
        Has<DialogRoot>,
        Has<SettingsBody>,
        Has<InterfaceBody>,
        Has<SupportBody>,
        Has<LoginMessage>,
    )>,
    cap: Option<Res<crate::fantasy_ui::graphics::FrameCap>>,
    vsync: Option<Res<crate::fantasy_ui::graphics::Vsync>>,
    view: Option<Res<crate::fantasy_ui::graphics::ViewDistance>>,
) {
    let focused = focus.get().and_then(|e| choices.get(e).ok())
        == Some(&ui::Choice::Account(Action::Password));
    for (mut text, password, username_placeholder, message, summary, parent) in &mut labels {
        if username_placeholder {
            **text = if usernames
                .single()
                .is_ok_and(|u| u.value().to_string().is_empty())
            {
                "Account Name".into()
            } else {
                String::new()
            };
        }
        if password {
            let start = login.secret.cursor.saturating_sub(28);
            **text = login
                .secret
                .password
                .iter()
                .skip(start)
                .take(36)
                .map(|c| if login.secret.revealed { *c } else { '•' })
                .collect();
            if login.secret.password.is_empty() && !focused {
                text.push_str("Password");
            }
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
            text.clear();
        }
    }
    for (entity, choice) in &accessible {
        if *choice == ui::Choice::Account(Action::Reveal) {
            commands
                .entity(entity)
                .insert(AccessibleLabel::new(if login.secret.revealed {
                    "Hide password"
                } else {
                    "Show password"
                }));
        }
        if let ui::Choice::Account(action) = choice {
            let selected = matches!(action, Action::Settings)
                && login.dialog == Some(Dialog::Settings)
                || matches!(action, Action::Interface) && login.dialog == Some(Dialog::Interface)
                || matches!(action, Action::Support) && login.dialog == Some(Dialog::Support)
                || matches!(action,Action::Cap(value) if cap.as_ref().is_some_and(|c|c.0==*value))
                || matches!(action,Action::Distance(value) if view.as_ref().is_some_and(|v|**v==*value));
            if selected {
                commands.entity(entity).insert(crate::ui_theme::Selected);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::ui_theme::Selected>();
            }
        }
    }
    for (mut node, mut vis, dialog, settings, interface, support, message) in &mut nodes {
        let show = if dialog {
            login.dialog.is_some()
        } else if settings {
            login.dialog == Some(Dialog::Settings)
        } else if interface {
            login.dialog == Some(Dialog::Interface)
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

pub(super) fn dialog_open(login: &Login) -> bool {
    login.dialog.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn password_editing_bounds_cursor_and_clears_selection_without_plaintext_ui() {
        let mut login = Login::default();
        login.secret.edit(&Key::Character("sëcret".into()), false);
        login.secret.edit(&Key::ArrowLeft, false);
        login.secret.edit(&Key::Backspace, false);
        assert_eq!(login.secret.password.iter().collect::<String>(), "sëcrt");
        login.secret.edit(&Key::Character("a".into()), true);
        login.secret.edit(&Key::Character("new".into()), false);
        assert_eq!(login.secret.password.iter().collect::<String>(), "new");
        login
            .secret
            .edit(&Key::Character("x".repeat(1100).into()), false);
        assert_eq!(login.secret.password.len(), 1024);
        login.secret.clear();
        assert!(login.secret.password.is_empty());
        assert!(!login.secret.revealed);
    }
}
