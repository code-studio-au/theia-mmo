//! Native registration: bounded in-memory secrets, provider-owned account creation.
use super::*;
use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{FocusCause, InputFocus, tab_navigation::TabIndex},
    tasks::{IoTaskPool, Task, futures::check_ready},
    text::EditableText,
    ui_widgets::TextInput,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Open,
    Back,
    Password(usize),
    Reveal(usize),
    Submit,
}
#[derive(Component)]
pub(super) struct Field(pub(super) usize);
#[derive(Component)]
pub(super) struct Placeholder(usize);
#[derive(Component)]
pub(super) struct PasswordText(usize);
#[derive(Component)]
pub(super) struct Rule(usize);
#[derive(Component)]
pub(super) struct RuleTick(usize);
#[derive(Component)]
pub(super) struct Status;
#[derive(Component)]
pub(super) struct Frame;
#[derive(Resource, Default)]
pub(super) struct Form {
    secrets: [login::Secret; 2],
    task: Option<Task<Result<(), String>>>,
}
fn rules(password: &[char]) -> [bool; 3] {
    [
        (8..=128).contains(&password.len()),
        password.iter().any(char::is_ascii_alphabetic) && password.iter().any(char::is_ascii_digit),
        !password.is_empty() && !password.iter().any(|c| c.is_whitespace() || c.is_control()),
    ]
}
fn error(name: &str, email: &str, secrets: &[login::Secret; 2]) -> Option<&'static str> {
    if !(3..=32).contains(&name.len())
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Some(
            "Use 3–32 letters, numbers, dots, underscores or hyphens for your account name.",
        );
    }
    let valid_email = email.split_once('@').is_some_and(|(a, b)| {
        !a.is_empty()
            && b.split('.').count() >= 2
            && b.split('.').all(|v| !v.is_empty())
            && !b.contains('@')
    });
    if email.len() > 254
        || !valid_email
        || email.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Some("Enter a valid email address.");
    }
    if !rules(&secrets[0].password).into_iter().all(|v| v) {
        return Some("Use 8–128 characters, including a letter and a number, with no spaces.");
    }
    if secrets[0].password != secrets[1].password {
        return Some("Your passwords do not match.");
    }
    None
}
pub(super) struct Art {
    portrait: Handle<Image>,
    mist: Handle<Image>,
}
pub(super) fn art(images: &mut Assets<Image>) -> Art {
    let mut load = |bytes: &[u8]| {
        images.add(
            Image::from_buffer(
                bytes,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::NONE,
                true,
                bevy::image::ImageSampler::default(),
                bevy::asset::RenderAssetUsages::default(),
            )
            .expect("Account illustration"),
        )
    };
    Art {
        portrait: load(include_bytes!(
            "../../../../content/ui/frontend/account-adventurer-v1.png"
        )),
        mist: load(include_bytes!(
            "../../../../content/ui/frontend/account-mist-v1.png"
        )),
    }
}
fn button(parent: &mut ChildSpawnerCommands, action: Action, text: &str) -> Entity {
    ui::button(parent, ui::Choice::Registration(action), text)
}
fn icon(parent: &mut ChildSpawnerCommands, index: usize) {
    parent.spawn((
        crate::ui_theme::Icon(index),
        Pickable::IGNORE,
        Node {
            width: px(28),
            height: px(28),
            position_type: PositionType::Absolute,
            left: px(10),
            top: px(10),
            ..default()
        },
    ));
}
pub(super) fn setup(root: &mut ChildSpawnerCommands, art: Art) {
    root.spawn((
        ui::Screen(Page::Register),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            padding: UiRect::top(px(120)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    ))
    .with_children(|page| {
        page.spawn((
            Frame,
            UiTransform::default(),
            crate::ui_theme::Panel,
            Node {
                width: px(1040),
                height: px(740),
                flex_shrink: 0.,
                padding: UiRect::all(px(28)),
                ..ui::column()
            },
            BackgroundColor(crate::ui_theme::PANEL),
        ))
        .with_children(|pane| {
            // Cover the portrait bay without stretching; clip at the inner frame.
            pane.spawn((
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(12),
                    top: px(50),
                    width: px(400),
                    height: px(678),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ))
            .with_children(|portrait| {
                portrait.spawn((
                    ImageNode {
                        color: Color::srgba(1., 1., 1., 0.65),
                        ..ImageNode::new(art.mist)
                    },
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        width: px(400),
                        height: px(678),
                        ..default()
                    },
                ));
                portrait.spawn((
                    ImageNode::new(art.portrait),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(-30),
                        top: px(-4),
                        width: px(460),
                        height: px(690),
                        ..default()
                    },
                ));
            });
            pane.spawn((
                crate::ui_theme::Plate,
                Node {
                    height: px(58),
                    width: px(560),
                    align_self: AlignSelf::Center,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(px(-48)),
                    flex_shrink: 0.,
                    ..default()
                },
            ))
            .with_child(ui::text("Create Account", 36.));
            pane.spawn(Node {
                flex_grow: 1.,
                column_gap: px(14),
                padding: UiRect::top(px(22)),
                min_height: px(0),
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(380),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                body.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(1),
                        height: percent(100),
                        flex_shrink: 0.,
                        ..default()
                    },
                    BackgroundGradient(vec![
                        LinearGradient::new(
                            LinearGradient::TO_BOTTOM,
                            vec![
                                ColorStop::percent(Color::NONE, 0),
                                ColorStop::percent(Color::srgba(0.82, 0.67, 0.38, 0.42), 12),
                                ColorStop::percent(Color::srgba(0.82, 0.67, 0.38, 0.42), 88),
                                ColorStop::percent(Color::NONE, 100),
                            ],
                        )
                        .into(),
                    ]),
                ));
                body.spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    row_gap: px(10),
                    ..ui::column()
                })
                .with_children(|form| {
                    for (index, title, placeholder) in [
                        (0, "Account Name", "Choose a unique account name"),
                        (1, "Email Address", "Enter your email address"),
                        (2, "Password", "Create a password"),
                        (3, "Confirm Password", "Re-enter your password"),
                    ] {
                        form.spawn(Node {
                            row_gap: px(4),
                            ..ui::column()
                        })
                        .with_children(|row| {
                            row.spawn(ui::text(title, 22.));
                            if index < 2 {
                                row.spawn((
                                    Field(index),
                                    AccessibleLabel::new(title),
                                    TextInput,
                                    TabIndex(index as i32),
                                    EditableText {
                                        max_characters: Some(if index == 0 { 32 } else { 254 }),
                                        allow_newlines: false,
                                        ..default()
                                    },
                                    crate::ui_theme::input_box(true, false),
                                    BackgroundColor(crate::ui_theme::PANEL),
                                    TextFont::from_font_size(20.),
                                    TextColor(Color::WHITE),
                                    TextLayout::no_wrap(),
                                    bevy::text::TextCursorStyle {
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ))
                                .with_children(|field| {
                                    icon(field, if index == 0 { 0 } else { 36 });
                                    field.spawn((
                                        Placeholder(index),
                                        ui::text(placeholder, 18.),
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(crate::ui_theme::INPUT_ICON_TEXT_INSET),
                                            top: px(12),
                                            ..default()
                                        },
                                    ));
                                });
                            } else {
                                row.spawn(Node {
                                    width: percent(100),
                                    height: px(48),
                                    ..default()
                                })
                                .with_children(|field| {
                                    let id = button(field, Action::Password(index - 2), "");
                                    field
                                        .commands()
                                        .entity(id)
                                        .insert((
                                            TabIndex(if index == 2 { 2 } else { 4 }),
                                            AccessibleLabel::new(title),
                                            crate::ui_theme::Style::Input,
                                            crate::ui_theme::input_box(true, true),
                                        ))
                                        .with_children(|secret| {
                                            icon(secret, 1);
                                            secret.spawn((
                                                PasswordText(index - 2),
                                                ui::text(placeholder, 18.),
                                            ));
                                        });
                                    let eye = button(field, Action::Reveal(index - 2), "");
                                    field
                                        .commands()
                                        .entity(eye)
                                        .insert((
                                            crate::ui_theme::Style::Icon,
                                            TabIndex(if index == 2 { 3 } else { 5 }),
                                            AccessibleLabel::new(format!(
                                                "Show {}",
                                                title.to_lowercase()
                                            )),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                right: px(8),
                                                top: px(8),
                                                width: px(32),
                                                height: px(32),
                                                align_items: AlignItems::Center,
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                        ))
                                        .with_child((
                                            crate::ui_theme::Icon(2),
                                            Pickable::IGNORE,
                                            Node {
                                                width: px(24),
                                                height: px(24),
                                                ..default()
                                            },
                                        ));
                                });
                            }
                        });
                    }
                    form.spawn(Node {
                        row_gap: px(2),
                        ..ui::column()
                    })
                    .with_children(|r| {
                        for (i, t) in [
                            "8–128 characters",
                            "Includes a letter and a number",
                            "No spaces",
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            r.spawn(Node {
                                column_gap: px(8),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    RuleTick(i),
                                    crate::ui_theme::Sprite("checkbox.tick"),
                                    Visibility::Hidden,
                                    Pickable::IGNORE,
                                    Node {
                                        width: px(18),
                                        height: px(18),
                                        ..default()
                                    },
                                ));
                                row.spawn((Rule(i), ui::text(t, 17.)));
                            });
                        }
                    });
                    form.spawn(Node {
                        row_gap: px(5),
                        ..ui::column()
                    })
                    .with_children(|r| {
                        r.spawn(Node {
                            align_items: AlignItems::Center,
                            column_gap: px(4),
                            ..default()
                        })
                        .with_children(|r| {
                            crate::fantasy_ui::kit::unsupported_check(r, "I agree to the");
                            crate::fantasy_ui::kit::disabled_link(r, "Terms of Service");
                            r.spawn(ui::text("and", 16.));
                            crate::fantasy_ui::kit::disabled_link(r, "Privacy Policy");
                        });
                        crate::fantasy_ui::kit::unsupported_check(
                            r,
                            "Send me news and updates about Theia Online.",
                        );
                    });
                    let create = button(form, Action::Submit, "");
                    form.commands()
                        .entity(create)
                        .insert((crate::ui_theme::Style::Primary, TabIndex(6)))
                        .with_child(ui::text("Create Account", 26.));
                    form.commands().entity(create).insert(Node {
                        height: px(58),
                        min_height: px(58),
                        width: percent(100),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    });
                    form.spawn((
                        Status,
                        ui::text("", 17.),
                        Node {
                            min_height: px(32),
                            ..default()
                        },
                    ));
                });
            });
        });
        let back = button(page, Action::Back, "Back to Login");
        page.commands().entity(back).insert((
            TabIndex(7),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                bottom: px(24),
                width: px(220),
                height: px(44),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));
    });
}
#[allow(clippy::too_many_arguments)]
pub(super) fn keyboard(
    mut input: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    choices: Query<&ui::Choice>,
    windows: Query<&Window>,
    mut form: ResMut<Form>,
    mut menu: ResMut<Menu>,
    login: Res<login::Login>,
) {
    if menu.page != Page::Register || !windows.single().is_ok_and(|w| w.focused) {
        for secret in &mut form.secrets {
            secret.revealed = false;
        }
        input.clear();
        return;
    }
    let field = match focus.get().and_then(|e| choices.get(e).ok()) {
        Some(ui::Choice::Registration(Action::Password(i))) => Some(*i),
        _ => None,
    };
    let control = keys.pressed(KeyCode::ControlLeft)
        || keys.pressed(KeyCode::ControlRight)
        || keys.pressed(KeyCode::SuperLeft)
        || keys.pressed(KeyCode::SuperRight);
    for event in input.read().filter(|e| e.state == ButtonState::Pressed) {
        if menu.pending || login::dialog_open(&login) {
            continue;
        }
        match &event.logical_key {
            Key::Escape => menu.action = Some(ui::Choice::Registration(Action::Back)),
            Key::Enter => menu.action = Some(ui::Choice::Registration(Action::Submit)),
            key => {
                if let Some(i) = field {
                    form.secrets[i].edit(key, control);
                }
            }
        }
    }
}
pub(super) fn actions(world: &mut World) {
    world.resource_scope(|world, mut form: Mut<Form>| {
        if let Some(task) = form.task.as_mut()
            && let Some(result) = check_ready(task)
        {
            form.task = None;
            if let Err(error) = result {
                let mut menu = world.resource_mut::<Menu>();
                menu.pending = false;
                menu.serial = menu.serial.saturating_sub(1);
                menu.message = error;
            }
        }
        if world.resource::<Menu>().page != Page::Register {
            for secret in &mut form.secrets {
                secret.clear();
            }
        }
        let Some(ui::Choice::Registration(action)) = world.resource::<Menu>().action else {
            return;
        };
        world.resource_mut::<Menu>().action = None;
        if world.resource::<Menu>().pending {
            return;
        }
        match action {
            Action::Open
                if world.resource::<Menu>().page == Page::Login
                    && world.resource::<Menu>().registration =>
            {
                for secret in &mut form.secrets {
                    secret.clear();
                }
                for mut value in world
                    .query_filtered::<&mut EditableText, With<Field>>()
                    .iter_mut(world)
                {
                    value.clear();
                }
                let mut menu = world.resource_mut::<Menu>();
                menu.page = Page::Register;
                menu.message.clear();
            }
            Action::Back => {
                for secret in &mut form.secrets {
                    secret.clear();
                }
                let mut menu = world.resource_mut::<Menu>();
                menu.page = Page::Login;
                menu.message.clear();
            }
            Action::Password(i) if i < 2 => {
                let target = world
                    .query::<(Entity, &ui::Choice)>()
                    .iter(world)
                    .find(|(_, c)| **c == ui::Choice::Registration(action))
                    .map(|(e, _)| e);
                if let Some(e) = target {
                    world
                        .resource_mut::<InputFocus>()
                        .set(e, FocusCause::Navigated);
                }
            }
            Action::Reveal(i) if i < 2 => {
                form.secrets[i].revealed = !form.secrets[i].revealed;
            }
            Action::Submit if world.resource::<Menu>().page == Page::Register => {
                let mut values = [String::new(), String::new()];
                for (field, value) in world.query::<(&Field, &EditableText)>().iter(world) {
                    values[field.0] = value.value().to_string();
                }
                if let Some(error) = error(&values[0], &values[1], &form.secrets) {
                    world.resource_mut::<Menu>().message = error.into();
                    return;
                }
                let password = form.secrets[0].password.iter().collect();
                for secret in &mut form.secrets {
                    secret.clear();
                }
                let mut menu = world.resource_mut::<Menu>();
                menu.serial += 1;
                menu.pending = true;
                menu.message = "Creating account…".into();
                let path = menu.directory.join("frontend-credentials.json");
                let serial = menu.serial;
                let [name, email] = values;
                form.task = Some(IoTaskPool::get().spawn(async move {
                    login::submit(path, serial, name, password, Some(email))
                }));
            }
            _ => (),
        }
    });
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn present(
    form: Res<Form>,
    menu: Res<Menu>,
    focus: Res<InputFocus>,
    choices: Query<&ui::Choice>,
    fields: Query<(&Field, &EditableText)>,
    mut labels: Query<(
        &mut Text,
        &mut TextColor,
        Option<&Placeholder>,
        Option<&PasswordText>,
        Option<&Rule>,
        Has<Status>,
    )>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut frames: Query<&mut UiTransform, With<Frame>>,
    mut ticks: Query<(&RuleTick, &mut Visibility)>,
) {
    if let Ok(window) = windows.single() {
        let size = Vec2::new(window.width(), window.height()) / scale.0;
        let fit = ((size.x - 32.) / 1040.)
            .min((size.y - 200.) / 740.)
            .clamp(0.1, 1.);
        for mut frame in &mut frames {
            *frame = UiTransform::from_scale(Vec2::splat(fit));
        }
    }
    let valid = rules(&form.secrets[0].password);
    for (tick, mut visibility) in &mut ticks {
        *visibility = if valid[tick.0] {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (mut text, mut colour, placeholder, password, rule, status) in &mut labels {
        if let Some(Placeholder(i)) = placeholder {
            **text = if fields
                .iter()
                .any(|(f, v)| f.0 == *i && !v.value().to_string().is_empty())
            {
                String::new()
            } else {
                ["Choose a unique account name", "Enter your email address"][*i].into()
            };
        }
        if let Some(PasswordText(i)) = password {
            let secret = &form.secrets[*i];
            let focused = focus.get().and_then(|e| choices.get(e).ok())
                == Some(&ui::Choice::Registration(Action::Password(*i)));
            **text = secret
                .password
                .iter()
                .skip(secret.cursor.saturating_sub(28))
                .take(36)
                .map(|c| if secret.revealed { *c } else { '•' })
                .collect();
            if secret.password.is_empty() && !focused {
                text.push_str(["Create a password", "Re-enter your password"][*i]);
            } else if focused {
                text.push('│');
            }
        }
        if let Some(Rule(i)) = rule {
            **text = [
                "8–128 characters",
                "Includes a letter and a number",
                "No spaces",
            ][*i]
                .into();
            colour.0 = if valid[*i] {
                Color::srgb(0.45, 0.9, 0.4)
            } else {
                ui::CREAM
            };
        }
        if status {
            **text = menu.message.clone();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_validation_bounds_and_confirmation() {
        let mut secrets = [login::Secret::default(), login::Secret::default()];
        for s in &mut secrets {
            s.password = "TestAccount123!".chars().collect();
        }
        assert_eq!(error("adventurer", "a@example.invalid", &secrets), None);
        for (name, email) in [
            ("a", "a@example.invalid"),
            ("a b", "a@example.invalid"),
            ("valid", "bad"),
            ("valid", "a@.invalid"),
        ] {
            assert!(error(name, email, &secrets).is_some());
        }
        secrets[1].password.push('x');
        assert_eq!(
            error("adventurer", "a@example.invalid", &secrets),
            Some("Your passwords do not match.")
        );
        for value in [
            "short1",
            "onlyletters",
            "123456789",
            "Has spaces1",
            "Contains\nNewline1",
        ] {
            assert!(
                !rules(&value.chars().collect::<Vec<_>>())
                    .into_iter()
                    .all(|v| v)
            );
        }
    }
}
