use super::*;
use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    picking::hover::Hovered,
    text::EditableText,
    ui::widget::ViewportNode,
    ui_widgets::{Activate, ActivateOnPress, Button, TextInput},
};
pub(super) const GOLD: Color = crate::ui_theme::GOLD;
pub(super) const CREAM: Color = crate::ui_theme::CREAM;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Choice {
    Login,
    Account(login::Action),
    New,
    Back,
    Select(usize),
    Cycle(usize, bool),
    Rotate(bool),
    Enter,
    Create,
    Exit,
}
#[derive(Component)]
pub(super) struct NameInput;
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
pub(super) struct FrontCamera;
#[derive(Component, Clone, Copy)]
pub(super) enum Placement {
    Backdrop(bool),
    Entry,
    Rotation,
    Status,
    Links,
    EmptyRoster,
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
pub(super) fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
                padding: UiRect::all(px(16)),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Placement::Backdrop(true),
                ImageNode::new(image),
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
                ImageNode::new(courtyard),
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
                BackgroundColor(Color::srgba(0.018, 0.027, 0.033, 0.94)),
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
                        body.spawn(text("Character selection", 25.));
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
                                Button,
                                ActivateOnPress,
                                TabIndex(0),
                                Hovered::default(),
                                Node {
                                    min_height: px(44),
                                    align_items: AlignItems::Center,
                                    flex_shrink: 0.,
                                    padding: UiRect::all(px(9)),
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
                    });
            });
            root.spawn((
                Screen(Page::Create),
                crate::ui_theme::Panel,
                Node {
                    width: px(960),
                    max_width: percent(96),
                    padding: UiRect::all(px(20)),
                    border: UiRect::all(px(1)),
                    ..column()
                },
                BackgroundColor(Color::srgba(0.018, 0.027, 0.033, 0.94)),
                BorderColor::all(GOLD),
            ))
            .with_children(|body| {
                body.spawn(text("Create your adventurer", 26.));
                for (i, label) in theia_core::appearance::LABELS.iter().enumerate() {
                    body.spawn(Node {
                        column_gap: px(8),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        button(row, Choice::Cycle(i, false), "‹");
                        row.spawn((
                            Label(i),
                            text(label, 17.),
                            Node {
                                flex_grow: 1.,
                                ..default()
                            },
                        ));
                        button(row, Choice::Cycle(i, true), "›");
                    });
                }
                body.spawn(text("Character name", 16.));
                body.spawn((
                    NameInput,
                    TabIndex(0),
                    TextInput,
                    EditableText {
                        max_characters: Some(20),
                        allow_newlines: false,
                        ..default()
                    },
                    Node {
                        min_height: px(38),
                        padding: UiRect::all(px(8)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(GOLD),
                    BackgroundColor(Color::srgba(0., 0., 0., 0.7)),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(19.),
                        ..default()
                    },
                    TextColor(CREAM),
                    TextLayout::no_wrap(),
                    bevy::text::TextCursorStyle {
                        color: CREAM,
                        ..default()
                    },
                ));
                body.spawn(Node {
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|row| {
                    button(row, Choice::Back, "Back");
                    button(row, Choice::Create, "Save Character");
                });
            });
            // Shared live model, never shown for an empty account or on Login.
            root.spawn((
                Screen(Page::Entering),
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
                button(row, Choice::Rotate(false), "Turn left");
                button(row, Choice::Rotate(true), "Turn right");
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
        *visibility = if preview.ready && menu.preview_visible() {
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
    let art = artwork_size(size);
    let gutter = (size - art) * 0.5;
    for (entity, mut node, screen, row, root, title, roster, placement) in &mut nodes {
        if roster {
            node.row_gap = px(if size.y < 800. { 4. } else { 10. });
        }
        if title {
            let login = menu.page == Page::Login;
            let height = if login {
                art.y * 0.34
            } else {
                (art.y * 0.17).clamp(80., 160.)
            };
            node.position_type = PositionType::Absolute;
            node.left = px(if login {
                (size.x - height * (1752. / 897.)) * 0.5
            } else {
                gutter.x + 24.
            });
            node.top = px(gutter.y + if login { art.y * 0.01 } else { 12. });
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
                node.width = px((art.x * 0.30).max(420.));
                node.left = px((size.x - (art.x * 0.30).max(420.)) * 0.5);
                node.top = px(gutter.y + art.y * 0.41);
            }
            if matches!(screen.0, Page::Characters | Page::Create) && placement.is_none() {
                let roster = screen.0 == Page::Characters;
                node.position_type = PositionType::Absolute;
                node.right = px(gutter.x + art.x * 0.025);
                node.top = px(gutter.y + art.y * if roster { 0.08 } else { 0.18 });
                node.width = px(art.x * if roster { 0.28 } else { 0.42 });
                node.height = if roster { px(art.y * 0.82) } else { Val::Auto };
                node.max_width = Val::Auto;
                node.row_gap = px(if size.y < 800. { 4. } else { 10. });
            }
        }
        if let Some(placement) = placement {
            match placement {
                Placement::Backdrop(login) => {
                    node.display = if *login == (menu.page == Page::Login) {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
                Placement::EmptyRoster => {
                    node.display = if menu.characters.is_empty() {
                        Display::Flex
                    } else {
                        Display::None
                    };
                }
                Placement::Entry | Placement::Rotation => {
                    node.position_type = PositionType::Absolute;
                    node.width = px(300.);
                    node.left = px(gutter.x + art.x * 0.40 - 150.);
                    node.top = px(gutter.y
                        + art.y
                            * if matches!(placement, Placement::Entry) {
                                0.83
                            } else {
                                0.94
                            });
                    node.row_gap = px(4.);
                }
                Placement::Status | Placement::Links => {
                    let login = menu.page == Page::Login;
                    node.position_type = PositionType::Absolute;
                    node.bottom = px(24.);
                    node.left = px(if matches!(placement, Placement::Links) {
                        gutter.x + 52.
                    } else {
                        gutter.x + art.x * 0.71
                    });
                    if matches!(placement, Placement::Status) {
                        node.display = if login { Display::None } else { Display::Flex };
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
            node.padding = UiRect::all(px(if size.y < 800. { 4. } else { 9. }));
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
                    login::Action::Settings | login::Action::Support | login::Action::Close
                )
        );
        let disabled = !always_available
            && (menu.pending
                || matches!(choice, Choice::Enter) && (menu.selected.is_none() || !preview.ready)
                || matches!(choice, Choice::Create) && !preview.ready
                || matches!(choice, Choice::New) && menu.characters.len() >= 8
                || matches!(choice, Choice::Cycle(3, _)) && menu.appearance.values()[0] == 1);
        if disabled {
            commands
                .entity(entity)
                .insert(bevy::ui::InteractionDisabled);
        } else {
            commands
                .entity(entity)
                .remove::<bevy::ui::InteractionDisabled>();
        }
        if matches!(choice,Choice::Select(i) if Some(*i)==menu.selected) {
            commands.entity(entity).insert(crate::ui_theme::Selected);
        } else {
            commands
                .entity(entity)
                .remove::<crate::ui_theme::Selected>();
        }
        color.0 = if link {
            Color::NONE
        } else if disabled {
            Color::srgb(0.08, 0.08, 0.08)
        } else if matches!(choice,Choice::Select(i) if Some(*i)==menu.selected) {
            Color::srgb(0.30, 0.19, 0.07)
        } else {
            Color::srgba(0.025, 0.11, 0.15, 0.96)
        };
    }
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
    let message = menu.message.clone();
    let preview_visible = menu.preview_visible();
    let character_count = menu.characters.len();
    let scale = world.resource::<UiScale>().0;
    let geometry: serde_json::Map<String, serde_json::Value> = world
        .query::<(
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&Choice>,
            Has<NameInput>,
            Has<login::Username>,
        )>()
        .iter(world)
        .filter_map(|(node, transform, choice, name, username)| {
            if node.size().min_element() <= 0. {
                return None;
            }
            let label = if username {
                "Username".to_string()
            } else if name {
                "Name".to_string()
            } else {
                format!("{:?}", choice?)
            };
            let factor = crate::fantasy_ui::window_factor(node, scale);
            let centre = transform.affine().translation * factor;
            let size = node.size() * factor;
            Some((
                label,
                serde_json::json!([
                    centre.x - size.x / 2.,
                    centre.y - size.y / 2.,
                    size.x,
                    size.y
                ]),
            ))
        })
        .collect();
    let disabled: Vec<_> = world
        .query_filtered::<&Choice, With<bevy::ui::InteractionDisabled>>()
        .iter(world)
        .map(|c| format!("{c:?}"))
        .collect();
    let ready = world.resource::<preview::Preview>().ready;
    let value = serde_json::json!({"page":page,"appearance":appearance,"message":message,"preview_ready":ready,"preview_visible":preview_visible,"character_count":character_count,"disabled":disabled,"widgets":geometry,"theme":crate::ui_theme::snapshot(world)});
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
    let position = ledge_anchor(window, size, feet.truncate());
    let art = artwork_size(window);
    for (screen, mut node) in world
        .query_filtered::<(&Screen, &mut Node), Without<Placement>>()
        .iter_mut(world)
    {
        if screen.0 == Page::Entering {
            node.left = px(position.x);
            node.top = px(position.y);
            node.height = px(art.y * 0.82);
            node.width = px(art.x * 0.34);
        }
    }
}
fn ledge_anchor(window: Vec2, viewport: Vec2, feet_ndc: Vec2) -> Vec2 {
    let art = artwork_size(window);
    (window - art) * 0.5 + art * Vec2::new(0.40, 0.81)
        - viewport * Vec2::new((feet_ndc.x + 1.) * 0.5, (1. - feet_ndc.y) * 0.5)
}

fn artwork_size(window: Vec2) -> Vec2 {
    // Match standard ImageNode::Auto containment, including wide-window gutters.
    let source = Vec2::new(1672., 941.);
    source * (window / source).min_element()
}

#[cfg(test)]
mod tests {
    use super::*;
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
