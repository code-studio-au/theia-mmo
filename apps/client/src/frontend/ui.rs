use super::*;
use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    picking::hover::Hovered,
    text::EditableText,
    ui::widget::ViewportNode,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
pub(super) const GOLD: Color = crate::ui_theme::GOLD;
pub(super) const CREAM: Color = crate::ui_theme::CREAM;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Choice {
    Login,
    Account(login::Action),
    Registration(registration::Action),
    New,
    Back,
    SignOut,
    Select(usize),
    Category(creation::Category),
    Appearance(usize, u8),
    Randomize,
    Rotate,
    Zoom(bool),
    ResetView,
    Enter,
    Create,
    Exit,
}
#[derive(Component)]
pub(super) struct NameInput;
#[derive(Component)]
pub(super) struct NameError;
fn invalid_name(value: &str) -> bool {
    !value.trim().is_empty()
        && theia_core::character_name::CharacterName::parse(value.trim()).is_err()
}
pub(super) fn name_validation(
    names: Query<&EditableText, With<NameInput>>,
    mut labels: Query<&mut Text, With<NameError>>,
) {
    let invalid = names
        .single()
        .is_ok_and(|name| invalid_name(&name.value().to_string()));
    for mut label in &mut labels {
        let next = if invalid { "- invalid name!" } else { "" };
        if label.0 != next {
            label.0 = next.into();
        }
    }
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) struct Title;
#[derive(Component)]
pub(super) struct RosterBody;
#[derive(Component)]
pub(super) struct Screen(pub(super) Page);
#[derive(Component)]
pub(super) struct Label(usize);
#[derive(Component)]
pub(super) struct Message;
#[derive(Component)]
pub(super) struct Row(usize);
#[derive(Component)]
pub(crate) struct FrontCamera;
#[derive(Component, Clone, Copy)]
pub(super) enum Placement {
    Backdrop(bool),
    Entry,
    ReturnToLogin,
    Rotation,
    DragArea,
    Feet,
    Status,
    Links,
    EmptyRoster,
    Creation,
}
#[derive(Component)]
pub(super) struct SelectedName;
pub(super) fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(CREAM),
        Pickable::IGNORE,
    )
}
pub(super) fn button(parent: &mut ChildSpawnerCommands, choice: Choice, caption: &str) -> Entity {
    parent
        .spawn((
            choice,
            match choice {
                Choice::Login | Choice::Create | Choice::Enter => crate::ui_theme::Style::Primary,
                Choice::Account(login::Action::Password) => crate::ui_theme::Style::Input,
                _ => crate::ui_theme::Style::Secondary,
            },
            Button,
            ActivateOnPress,
            TabIndex(0),
            Hovered::default(),
            Node {
                min_height: px(32),
                padding: UiRect::axes(px(16), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(GOLD),
            BackgroundColor(Color::srgba(0.025, 0.11, 0.15, 0.96)),
        ))
        .with_child(text(caption, 17.))
        .id()
}
pub(super) fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(10),
        ..default()
    }
}
pub(super) fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    server: Option<Res<AssetServer>>,
    references: Option<Res<crate::reference_scene::Config>>,
) {
    let atlas = super::thumbnail_atlas(server.as_deref(), references.as_deref(), false);
    let registration_art = registration::art(&mut images);
    let preview = preview::setup(&mut commands, &mut images);
    let camera = commands
        .spawn((
            FrontCamera,
            Camera2d,
            Camera {
                order: 1100,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            bevy::camera::visibility::RenderLayers::none(),
        ))
        .id();
    let image = Image::from_buffer(
        include_bytes!("../../../../content/ui/frontend/sunset-cloaked-v2.png"),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("Embedded title artwork");
    let image = images.add(image);
    let courtyard = Image::from_buffer(
        include_bytes!("../../../../content/ui/frontend/character-courtyard-v1.png"),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("Embedded character courtyard");
    let courtyard = images.add(courtyard);
    let title = Image::from_buffer(
        include_bytes!("../../../../content/ui/frontend/title.png"),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("Embedded illustrated Theia title");
    let title = images.add(title);
    let foot_arrow = images.add(
        Image::from_buffer(
            include_bytes!("../../../../content/ui/theme/standalone-v3/foot-arrow.png"),
            bevy::image::ImageType::Extension("png"),
            bevy::image::CompressedImageFormats::NONE,
            true,
            bevy::image::ImageSampler::default(),
            bevy::asset::RenderAssetUsages::default(),
        )
        .expect("Embedded character rotation arrow"),
    );
    commands
        .spawn((
            Root,
            TabGroup::modal(),
            UiTargetCamera(camera),
            GlobalZIndex(1100),
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Placement::Backdrop(true),
                ImageNode {
                    image,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            root.spawn((
                Placement::Backdrop(false),
                ImageNode {
                    image: courtyard,
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.025, 0.04, 0.16)),
                Pickable::IGNORE,
            ));
            root.spawn((
                Title,
                ImageNode::new(title),
                Pickable::IGNORE,
                Node {
                    width: px(520),
                    height: px(266),
                    flex_shrink: 0.,
                    ..default()
                },
            ));
            login::panel(root);
            registration::setup(root, registration_art);
            root.spawn((
                Screen(Page::Characters),
                crate::ui_theme::Panel,
                Node {
                    width: px(960),
                    max_width: percent(96),
                    padding: UiRect::all(px(20)),
                    border: UiRect::all(px(1)),
                    column_gap: px(24),
                    ..default()
                },
                BackgroundColor(crate::ui_theme::PANEL),
                BorderColor::all(GOLD),
            ))
            .with_children(|panel| {
                panel
                    .spawn((
                        RosterBody,
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..column()
                        },
                    ))
                    .with_children(|body| {
                        crate::ui_theme::heading(body, "Select Character");
                        body.spawn((
                            Placement::EmptyRoster,
                            text(
                                "No characters yet.
Create your first adventurer to begin your story.",
                                18.,
                            ),
                            Node {
                                padding: UiRect::vertical(px(24)),
                                ..default()
                            },
                        ));
                        for i in 0..8 {
                            body.spawn((
                                Row(i),
                                Choice::Select(i),
                                crate::ui_theme::Style::Roster,
                                Button,
                                ActivateOnPress,
                                TabIndex(0),
                                Hovered::default(),
                                Node {
                                    min_height: px(44),
                                    align_items: AlignItems::Center,
                                    flex_shrink: 0.,
                                    padding: UiRect::axes(px(24), px(10)),
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BorderColor::all(GOLD),
                                BackgroundColor(Color::srgba(0.04, 0.08, 0.1, 0.95)),
                            ))
                            .with_child((Label(10 + i), text("", 18.)));
                        }
                        body.spawn(Node {
                            flex_grow: 1.,
                            ..default()
                        });
                        button(body, Choice::New, "Create Character");
                        crate::fantasy_ui::kit::disabled(body, "Delete Character");
                    });
            });
            let back = button(root, Choice::SignOut, "Back to Login");
            root.commands()
                .entity(back)
                .insert((Screen(Page::Characters), Placement::ReturnToLogin));
            creation::setup(root, atlas);
            // Shared live model, never shown for an empty account or on Login.
            root.spawn((
                Screen(Page::Entering),
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(34),
                    height: percent(82),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    ViewportNode::new(preview),
                    Visibility::Hidden,
                    Pickable::IGNORE,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                ));
            });
            root.spawn((
                Screen(Page::Characters),
                Placement::Entry,
                Node {
                    align_items: AlignItems::Center,
                    ..column()
                },
            ))
            .with_children(|entry| {
                entry.spawn((SelectedName, text("", 24.)));
                button(entry, Choice::Enter, "Enter World");
            });
            root.spawn((
                Screen(Page::Entering),
                Placement::Rotation,
                Node {
                    column_gap: px(10),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|row| {
                for (choice, icon, label) in [
                    (Choice::Zoom(false), 32, "Zoom out"),
                    (Choice::ResetView, 31, "Reset character view"),
                    (Choice::Zoom(true), 33, "Zoom in"),
                ] {
                    let control = button(row, choice, "");
                    row.commands()
                        .entity(control)
                        .insert((
                            AccessibleLabel::new(label),
                            Node {
                                width: px(52),
                                height: px(44),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_child((
                            crate::ui_theme::Icon(icon),
                            Node {
                                width: px(28),
                                height: px(28),
                                ..default()
                            },
                        ));
                }
            });
            // A transparent, bounded hit area over the model, not the whole page.
            root.spawn((
                Screen(Page::Entering),
                Placement::DragArea,
                Choice::Rotate,
                Button,
                ActivateOnPress,
                TabIndex(0),
                Hovered::default(),
                crate::ui_theme::Style::Icon,
                BackgroundColor(Color::NONE),
                AccessibleLabel::new(
                    "Rotate character: drag left or right; arrow keys when focused",
                ),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ));
            root.spawn((
                Screen(Page::Entering),
                Placement::Feet,
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
            ))
            .with_children(|row| {
                for flip_x in [false, true] {
                    row.spawn((
                        ImageNode {
                            image: foot_arrow.clone(),
                            flip_x,
                            ..default()
                        },
                        Pickable::IGNORE,
                        Node {
                            width: px(96),
                            height: px(56),
                            ..default()
                        },
                    ));
                }
            });
            root.spawn((
                Message,
                Placement::Status,
                text("", 16.),
                Node {
                    max_width: percent(85),
                    ..default()
                },
            ));
            login::links(root);
        });
}
pub(super) fn choose(event: On<Activate>, choices: Query<&Choice>, mut menu: ResMut<Menu>) {
    if let Ok(choice) = choices.get(event.entity) {
        menu.action = Some(*choice);
    }
}
pub(super) fn rotate_drag(
    mut event: On<bevy::picking::events::PointerDrag>,
    choices: Query<&Choice>,
    windows: Query<&Window>,
    mut menu: ResMut<Menu>,
) {
    if choices.get(event.entity) != Ok(&Choice::Rotate)
        || event.button != bevy::picking::pointer::PointerButton::Primary
        || !menu.preview_visible()
        || menu.pending
        || !windows.single().is_ok_and(|w| w.focused)
        || !event.delta.x.is_finite()
    {
        return;
    }
    menu.yaw = (menu.yaw - event.delta.x * 0.012).rem_euclid(std::f32::consts::TAU);
    event.propagate(false);
}
pub(super) fn rotate_keyboard(
    focus: Res<bevy::input_focus::InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    choices: Query<&Choice>,
    mut menu: ResMut<Menu>,
) {
    if !menu.preview_visible()
        || menu.pending
        || !focus
            .get()
            .is_some_and(|e| choices.get(e) == Ok(&Choice::Rotate))
    {
        return;
    }
    let direction = i32::from(keys.just_pressed(KeyCode::ArrowLeft))
        - i32::from(keys.just_pressed(KeyCode::ArrowRight));
    if direction != 0 {
        menu.yaw = (menu.yaw + direction as f32 * 0.15).rem_euclid(std::f32::consts::TAU);
    }
}
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) fn sync(
    menu: Res<Menu>,
    preview: Res<preview::Preview>,
    mut commands: Commands,
    mut viewports: Query<&mut Visibility, With<ViewportNode>>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut nodes: Query<(
        Entity,
        &mut Node,
        Option<&Screen>,
        Option<&Row>,
        Has<Root>,
        Has<Title>,
        Has<RosterBody>,
        Option<&Placement>,
    )>,
    mut labels: Query<(&mut Text, Option<&Label>, Has<Message>, Has<SelectedName>)>,
    mut buttons: Query<(Entity, &Choice, &mut BackgroundColor, Has<login::MenuLink>)>,
    mut cameras: Query<&mut Camera, With<FrontCamera>>,
) {
    for mut visibility in &mut viewports {
        *visibility = if preview.visible && menu.preview_visible() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut camera in &mut cameras {
        camera.is_active = menu.active();
    }
    let size = windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()) / scale.0)
        .unwrap_or(Vec2::new(1600., 900.));
    let art = size;
    let stage = creation::stage(size, menu.page);
    for (entity, mut node, screen, row, root, title, roster, placement) in &mut nodes {
        if roster {
            node.row_gap = px(if size.y < 800. { 4. } else { 10. });
        }
        if title {
            let login = menu.page == Page::Login;
            let height = if login {
                art.y * 0.34
            } else if menu.page == Page::Register {
                100.
            } else if menu.page == Page::Create {
                (art.y * 0.12).clamp(64., 120.)
            } else {
                (art.y * 0.17).clamp(80., 160.)
            };
            node.position_type = PositionType::Absolute;
            node.left = px(if login || menu.page == Page::Register {
                (size.x - height * (1752. / 897.)) * 0.5
            } else {
                24.
            });
            node.top = px(if login { art.y * 0.01 } else { 12. });
            node.height = px(height);
            node.width = px(height * (1752. / 897.));
        }
        if root {
            node.row_gap = px(if size.y < 800. { 6. } else { 12. });
            node.display = if menu.active() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if let Some(screen) = screen {
            let show = if screen.0 == Page::Entering {
                menu.preview_visible()
            } else {
                menu.page == screen.0
            };
            node.display = if show { Display::Flex } else { Display::None };
            if screen.0 == Page::Login {
                node.position_type = PositionType::Absolute;
                node.width = px((art.x * 0.30).clamp(420., 560.));
                node.left = px((size.x - (art.x * 0.30).clamp(420., 560.)) * 0.5);
                node.top = px(art.y * 0.41);
            }
            if matches!(screen.0, Page::Characters | Page::Create) && placement.is_none() {
                let roster = screen.0 == Page::Characters;
                node.position_type = PositionType::Absolute;
                node.right = px(art.x * 0.025);
                node.top = px(art.y * if roster { 0.08 } else { 0.18 });
                node.width = px(if roster {
                    (art.x * 0.25).clamp(340., 460.)
                } else {
                    (art.x * 0.28).clamp(360., 480.)
                });
                node.height = if roster { px(art.y * 0.82) } else { Val::Auto };
                node.max_width = Val::Auto;
                node.row_gap = px(if size.y < 800. { 4. } else { 10. });
            }
        }
        if let Some(placement) = placement {
            match placement {
                Placement::Backdrop(login) => {
                    let cover = background_size(size);
                    node.width = px(cover.x);
                    node.height = px(cover.y);
                    node.left = px((size.x - cover.x) * 0.5);
                    node.top = px((size.y - cover.y) * 0.5);
                    node.display = if *login == (menu.page == Page::Login) {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
                Placement::Creation => {
                    node.position_type = PositionType::Absolute;
                    node.left = px(0);
                    node.top = px(0);
                    node.width = percent(100);
                    node.height = percent(100);
                }
                Placement::EmptyRoster => {
                    node.display = if menu.characters.is_empty() {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
                Placement::DragArea => {
                    node.width = px(stage.z);
                    node.height = px(stage.y - art.y * 0.11);
                    node.left = px(stage.x - stage.z * 0.5);
                    node.top = px(art.y * 0.11);
                }
                Placement::ReturnToLogin => {
                    node.position_type = PositionType::Absolute;
                    node.left = px(24.);
                    node.bottom = px(24.);
                    node.width = px(220.);
                    node.height = px(44.);
                }
                Placement::Feet => {
                    node.width = px(300.);
                    node.height = px(56.);
                    node.left = px(stage.x - 150.);
                    node.top = px(stage.y - 48.);
                }
                Placement::Entry | Placement::Rotation => {
                    node.position_type = PositionType::Absolute;
                    node.width = px(300.);
                    node.left = px(stage.x - 150.);
                    node.top = px(art.y
                        * if matches!(placement, Placement::Entry) {
                            0.83
                        } else {
                            0.94
                        });
                    if matches!(placement, Placement::Rotation) {
                        node.top = px(if menu.page == Page::Create {
                            stage.y + 12.
                        } else {
                            (art.y * 0.94).min(art.y - 60.)
                        });
                    }
                    node.row_gap = px(4.);
                }
                Placement::Status | Placement::Links => {
                    let login = menu.page == Page::Login;
                    node.position_type = PositionType::Absolute;
                    node.bottom = px(
                        if matches!(placement, Placement::Links) && menu.page == Page::Characters {
                            84.
                        } else {
                            24.
                        },
                    );
                    node.left = px(if matches!(placement, Placement::Links) {
                        52.
                    } else {
                        art.x * 0.71
                    });
                    if matches!(placement, Placement::Links) {
                        node.display = if matches!(menu.page, Page::Create | Page::Register) {
                            Display::None
                        } else {
                            Display::Flex
                        };
                    }
                    if matches!(placement, Placement::Status) {
                        node.display = if login || menu.page == Page::Register {
                            Display::None
                        } else {
                            Display::Flex
                        };
                        if menu.page == Page::Create {
                            node.left = px(art.x * 0.30);
                            node.bottom = px(110.);
                        }
                        node.max_width = if login {
                            percent(85)
                        } else {
                            px(art.x * 0.265)
                        };
                    }
                }
            }
        }
        if let Some(row) = row {
            node.padding = UiRect::axes(px(24), px(if size.y < 800. { 6. } else { 10. }));
            node.display = if row.0 < menu.characters.len() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if root || screen.is_some() || row.is_some() || placement.is_some() {
            commands
                .entity(entity)
                .insert(if node.display == Display::None {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                });
        }
    }
    for (mut text, label, message, selected_name) in &mut labels {
        if selected_name {
            **text = menu
                .selected
                .and_then(|i| menu.characters.get(i))
                .map(|c| c.name.clone())
                .unwrap_or_default();
        }
        if message {
            **text = menu.message.clone();
        }
        if let Some(Label(i)) = label {
            **text = if *i < 7 {
                let v = menu.appearance.values()[*i];
                format!(
                    "{}  ·  {}",
                    theia_core::appearance::LABELS[*i],
                    if *i == 0 {
                        if v == 0 {
                            "Masculine".into()
                        } else {
                            "Feminine".into()
                        }
                    } else {
                        (v + 1).to_string()
                    }
                )
            } else {
                menu.characters
                    .get(*i - 10)
                    .map_or_else(String::new, |c| c.name.clone())
            };
        }
    }
    for (entity, choice, mut color, link) in &mut buttons {
        let always_available = matches!(
            choice,
            Choice::Exit
                | Choice::Account(
                    login::Action::Settings
                        | login::Action::Interface
                        | login::Action::Support
                        | login::Action::Close
                )
        );
        let disabled = !always_available
            && (menu.pending
                || matches!(choice, Choice::Registration(registration::Action::Open))
                    && !menu.registration
                || matches!(choice, Choice::Enter) && (menu.selected.is_none() || !preview.ready)
                || matches!(choice, Choice::Create) && !preview.ready
                || matches!(choice, Choice::Zoom(true)) && menu.zoom >= 1.3
                || matches!(choice, Choice::Zoom(false)) && menu.zoom <= 0.75
                || matches!(choice, Choice::New) && menu.characters.len() >= 8
                || matches!(choice, Choice::Appearance(3, _)) && menu.appearance.values()[0] == 1);
        if disabled {
            commands
                .entity(entity)
                .insert(bevy::ui::InteractionDisabled);
        } else {
            commands
                .entity(entity)
                .remove::<bevy::ui::InteractionDisabled>();
        }
        let selected = matches!(choice,Choice::Select(i) if Some(*i)==menu.selected)
            || matches!(choice, Choice::Category(c) if *c == menu.category)
            || matches!(choice, Choice::Appearance(field, value) if menu.appearance.values().get(*field) == Some(value));
        if selected {
            commands.entity(entity).insert(crate::ui_theme::Selected);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui_theme::Selected>();
        }
        color.0 = if link || matches!(choice, Choice::Rotate) {
            Color::NONE
        } else if disabled {
            Color::srgb(0.08, 0.08, 0.08)
        } else if selected {
            Color::srgb(0.30, 0.19, 0.07)
        } else {
            Color::srgba(0.025, 0.11, 0.15, 0.96)
        };
    }
}
#[derive(Resource, Default)]
struct PreviewContinuity {
    seen: bool,
    frames: u64,
    gaps: u64,
    pending: u64,
    premature_actions: u64,
}
fn preview_continuity(world: &mut World, wanted: bool) -> serde_json::Value {
    let preview = world.resource::<preview::Preview>();
    let ready = preview.ready;
    let model_visible = preview.model_visible(world);
    let viewport_visible = world
        .query_filtered::<&Visibility, With<ViewportNode>>()
        .iter(world)
        .any(|v| *v != Visibility::Hidden);
    let page = world.resource::<Menu>().page;
    let premature = !ready
        && world
            .query::<(&Choice, Has<bevy::ui::InteractionDisabled>)>()
            .iter(world)
            .any(|(choice, disabled)| {
                !disabled
                    && match page {
                        Page::Create => matches!(choice, Choice::Create),
                        Page::Characters => matches!(choice, Choice::Enter),
                        _ => false,
                    }
            });
    world.init_resource::<PreviewContinuity>();
    let mut evidence = world.resource_mut::<PreviewContinuity>();
    if !wanted {
        evidence.seen = false;
    } else {
        evidence.seen |= ready;
        if evidence.seen {
            evidence.frames += 1;
            evidence.gaps += u64::from(!model_visible || !viewport_visible);
            evidence.pending += u64::from(!ready);
            evidence.premature_actions += u64::from(premature);
        }
    }
    serde_json::json!({"frames":evidence.frames,"gaps":evidence.gaps,"pending":evidence.pending,
        "premature_actions":evidence.premature_actions,"model_visible":model_visible,
        "viewport_visible":viewport_visible})
}
pub(super) fn evidence(world: &mut World) {
    if std::env::var_os("THEIA_WORLD_INPUT_OS_QUALIFY").is_none() {
        return;
    }
    let menu = world.resource::<Menu>();
    if !menu.active() {
        return;
    }
    let path = menu.directory.join("frontend-evidence.json");
    let page = format!("{:?}", menu.page);
    let appearance = menu.appearance;
    let preview_yaw = menu.yaw;
    let preview_zoom = menu.zoom;
    let message = menu.message.clone();
    let preview_visible = menu.preview_visible();
    let character_count = menu.characters.len();
    let invalid_name = world
        .query_filtered::<&EditableText, With<NameInput>>()
        .single(world)
        .is_ok_and(|name| invalid_name(&name.value().to_string()));
    let scale = world.resource::<UiScale>().0;
    let dialog_open = world
        .query_filtered::<&Node, With<login::DialogRoot>>()
        .iter(world)
        .any(|n| n.display != Display::None);
    let geometry: serde_json::Map<String, serde_json::Value> = world
        .query::<(
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&Choice>,
            Has<NameInput>,
            Has<login::Username>,
            Has<login::MenuLink>,
            Option<&registration::Field>,
        )>()
        .iter(world)
        .filter_map(
            |(node, transform, choice, name, username, login_link, registration_field)| {
                if node.size().min_element() <= 0. || (dialog_open && login_link) {
                    return None;
                }
                let label = if let Some(registration::Field(i)) = registration_field {
                    format!("RegistrationField({i})")
                } else if username {
                    "Username".to_string()
                } else if name {
                    "Name".to_string()
                } else {
                    format!("{:?}", choice?)
                };
                let factor = crate::fantasy_ui::window_factor(node, scale);
                let centre = transform.affine().translation * factor;
                let size = node.size() * factor * transform.affine().matrix2.x_axis.length();
                Some((
                    label,
                    serde_json::json!([
                        centre.x - size.x / 2.,
                        centre.y - size.y / 2.,
                        size.x,
                        size.y
                    ]),
                ))
            },
        )
        .collect();
    let disabled: Vec<_> = world
        .query_filtered::<&Choice, With<bevy::ui::InteractionDisabled>>()
        .iter(world)
        .map(|c| format!("{c:?}"))
        .collect();
    let ready = world.resource::<preview::Preview>().ready;
    let registration = registration::evidence(world);
    let continuity = preview_continuity(world, preview_visible);
    let value = serde_json::json!({"preview_continuity":continuity,"registration":registration,"page":page,"appearance":appearance,"message":message,"invalid_name":invalid_name,"preview_yaw":preview_yaw,"preview_zoom":preview_zoom,"preview_ready":ready,"preview_visible":preview_visible,"character_count":character_count,"disabled":disabled,"widgets":geometry,"theme":crate::ui_theme::snapshot(world)});
    let temporary = path.with_extension("tmp");
    if std::fs::write(&temporary, serde_json::to_vec(&value).unwrap()).is_ok() {
        let _ = std::fs::rename(temporary, path);
    }
}

pub(super) fn focus(
    menu: Res<Menu>,
    mut previous: Local<Option<Page>>,
    mut focus: ResMut<bevy::input_focus::InputFocus>,
    names: Query<Entity, With<NameInput>>,
    registration_fields: Query<(Entity, &registration::Field)>,
    choices: Query<(Entity, &Choice)>,
) {
    if *previous == Some(menu.page) {
        return;
    }
    *previous = Some(menu.page);
    focus.clear();
    let target = match menu.page {
        Page::Create => names.single().ok(),
        Page::Login => choices
            .iter()
            .find(|(_, c)| **c == Choice::Login)
            .map(|(e, _)| e),
        Page::Characters => choices
            .iter()
            .find(|(_, c)| **c == Choice::New)
            .map(|(e, _)| e),
        Page::Register => registration_fields
            .iter()
            .find(|(_, f)| f.0 == 0)
            .map(|(e, _)| e),
        Page::Entering => None,
    };
    if let Some(entity) = target {
        focus.set(entity, bevy::input_focus::FocusCause::Navigated);
    }
}

/// Anchor the rig's ground origin to the character courtyard, independent of
/// window size, UI scale, preview aspect and character rotation.
pub(super) fn anchor(world: &mut World) {
    if !world.resource::<Menu>().active() {
        return;
    }
    let camera_entity = world.resource::<preview::Preview>().camera;
    let Some(camera) = world.get::<Camera>(camera_entity) else {
        return;
    };
    let Some(transform) = world.get::<GlobalTransform>(camera_entity) else {
        return;
    };
    let Some(feet) = camera.world_to_ndc(transform, Vec3::ZERO) else {
        return;
    };
    let size = world
        .query_filtered::<&ComputedNode, With<ViewportNode>>()
        .single(world)
        .map(|n| n.size() * n.inverse_scale_factor())
        .unwrap_or(Vec2::ZERO);
    if size.min_element() <= 1. {
        return;
    }
    let scale = world.resource::<UiScale>().0;
    let window = world
        .query::<&Window>()
        .single(world)
        .map(|w| Vec2::new(w.width(), w.height()) / scale);
    let Ok(window) = window else {
        return;
    };
    let stage = creation::stage(window, world.resource::<Menu>().page);
    let position = stage.truncate() - size * Vec2::new((feet.x + 1.) * 0.5, (1. - feet.y) * 0.5);
    let zoom = world.resource::<Menu>().zoom;
    let art = window;
    for (screen, mut node) in world
        .query_filtered::<(&Screen, &mut Node), Without<Placement>>()
        .iter_mut(world)
    {
        if screen.0 == Page::Entering {
            node.left = px(position.x);
            node.top = px(position.y);
            node.height = px(art.y * 0.82 * zoom);
            node.width = px(art.x * 0.34 * zoom);
        }
    }
}

fn background_size(window: Vec2) -> Vec2 {
    // Cover each client window without stretching the artwork. Foreground UI
    // anchors to the window, independently of the cropped background.
    let source = Vec2::new(1672., 941.);
    source * (window / source).max_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backgrounds_cover_single_half_and_quarter_windows() {
        for window in [
            Vec2::new(3440., 1400.),
            Vec2::new(1720., 1400.),
            Vec2::new(1720., 700.),
            Vec2::new(1280., 720.),
        ] {
            let cover = background_size(window);
            assert!(cover.x + 0.01 >= window.x && cover.y + 0.01 >= window.y);
            assert!((cover.x / cover.y - 1672. / 941.).abs() < 0.001);
            assert!((cover.x - window.x).abs().min((cover.y - window.y).abs()) < 0.01);
        }
    }
    #[test]
    fn native_widget_tree_builds_without_duplicate_components() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Image>>()
            .init_resource::<bevy::picking::pointer::PointerMap>()
            .add_systems(Startup, setup);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&NameInput>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&ViewportNode>()
                .iter(app.world())
                .count(),
            1
        );
    }
}
