use super::*;
use bevy::{
    input_focus::tab_navigation::{TabGroup, TabIndex},
    picking::hover::Hovered,
    text::EditableText,
    ui::widget::ViewportNode,
    ui_widgets::{Activate, ActivateOnPress, Button, TextInput},
};
const GOLD: Color = Color::srgb(0.85, 0.66, 0.34);
const CREAM: Color = Color::srgb(0.95, 0.9, 0.78);
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Choice {
    Login,
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
pub(super) struct Screen(Page);
#[derive(Component)]
pub(super) struct Label(usize);
#[derive(Component)]
pub(super) struct Message;
#[derive(Component)]
pub(super) struct Row(usize);
#[derive(Component)]
pub(super) struct FrontCamera;
fn text(value: &str, size: f32) -> impl Bundle {
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
fn button(parent: &mut ChildSpawnerCommands, choice: Choice, caption: &str) -> Entity {
    parent
        .spawn((
            choice,
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
fn column() -> Node {
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
        include_bytes!("../../../../content/ui/frontend/sunset.png"),
        bevy::image::ImageType::Extension("png"),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::default(),
        bevy::asset::RenderAssetUsages::default(),
    )
    .expect("Embedded title artwork");
    let image = images.add(image);
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
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.025, 0.04, 0.32)),
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
            root.spawn((
                Screen(Page::Login),
                Node {
                    width: px(480),
                    max_width: percent(95),
                    padding: UiRect::all(px(30)),
                    border: UiRect::all(px(1)),
                    ..column()
                },
                BackgroundColor(Color::srgba(0.018, 0.027, 0.033, 0.94)),
                BorderColor::all(GOLD),
            ))
            .with_children(|panel| {
                panel.spawn(text("Welcome to Theia", 27.));
                panel.spawn(text(
                    "Sign in securely with your account to choose or create an adventurer.",
                    17.,
                ));
                button(panel, Choice::Login, "Log In");
                panel.spawn(text(
                    "Sign-in opens in your browser. Return here to continue.",
                    14.,
                ));
            });
            root.spawn((
                Screen(Page::Characters),
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
                            ..column()
                        },
                    ))
                    .with_children(|body| {
                        body.spawn(text("Your adventurers", 26.));
                        body.spawn(text("Select a saved character, or start a new story.", 16.));
                        for i in 0..8 {
                            body.spawn((
                                Row(i),
                                Choice::Select(i),
                                Button,
                                ActivateOnPress,
                                TabIndex(0),
                                Hovered::default(),
                                Node {
                                    padding: UiRect::all(px(9)),
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BorderColor::all(GOLD),
                                BackgroundColor(Color::srgba(0.04, 0.08, 0.1, 0.95)),
                            ))
                            .with_child((Label(10 + i), text("", 18.)));
                        }
                        button(body, Choice::Enter, "Enter World");
                        button(body, Choice::New, "New Character");
                    });
            });
            root.spawn((
                Screen(Page::Create),
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
            // One shared render target follows the roster/create panels; it never enters the world camera query.
            root.spawn((
                Screen(Page::Entering),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(5),
                    top: percent(22),
                    width: percent(30),
                    height: percent(68),
                    ..column()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    ViewportNode::new(preview),
                    Visibility::Hidden,
                    Pickable::IGNORE,
                    Node {
                        width: percent(100),
                        flex_grow: 1.,
                        ..default()
                    },
                ));
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::Center,
                        column_gap: px(12),
                        ..default()
                    })
                    .with_children(|row| {
                        button(row, Choice::Rotate(false), "Turn left");
                        button(row, Choice::Rotate(true), "Turn right");
                    });
            });
            root.spawn((
                Message,
                text("", 16.),
                Node {
                    max_width: percent(85),
                    ..default()
                },
            ));
            button(root, Choice::Exit, "Exit Game");
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
    )>,
    mut labels: Query<(&mut Text, Option<&Label>, Has<Message>)>,
    mut buttons: Query<(Entity, &Choice, &mut BackgroundColor)>,
    mut cameras: Query<&mut Camera, With<FrontCamera>>,
) {
    for mut visibility in &mut viewports {
        *visibility = if preview.ready {
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
    for (entity, mut node, screen, row, root, title, roster) in &mut nodes {
        if roster {
            node.row_gap = px(if size.y < 800. { 4. } else { 10. });
        }
        if title {
            let height = match menu.page {
                Page::Login => (size.y * 0.30).clamp(120., 440.),
                Page::Create => (size.y - 760.).clamp(80., 400.),
                _ => (size.y * 0.24).clamp(
                    80.,
                    if size.y < 800. && menu.characters.len() > 4 {
                        100.
                    } else {
                        360.
                    },
                ),
            };
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
                matches!(menu.page, Page::Create | Page::Characters)
            } else {
                menu.page == screen.0
            };
            node.display = if show { Display::Flex } else { Display::None };
            if matches!(screen.0, Page::Characters | Page::Create) {
                node.width = px((size.x * 0.5).clamp(360., 760.));
                node.margin.left = px(size.x * 0.32);
                node.row_gap = px(if size.y < 800. { 4. } else { 10. });
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
        if root || screen.is_some() || row.is_some() {
            commands
                .entity(entity)
                .insert(if node.display == Display::None {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                });
        }
    }
    for (mut text, label, message) in &mut labels {
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
    for (entity, choice, mut color) in &mut buttons {
        let disabled = *choice != Choice::Exit
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
        color.0 = if disabled {
            Color::srgb(0.08, 0.08, 0.08)
        } else if matches!(choice,Choice::Select(i) if Some(*i)==menu.selected) {
            Color::srgb(0.2, 0.23, 0.16)
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
    let scale = world.resource::<UiScale>().0;
    let geometry: serde_json::Map<String, serde_json::Value> = world
        .query::<(
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&Choice>,
            Has<NameInput>,
        )>()
        .iter(world)
        .filter_map(|(node, transform, choice, name)| {
            if node.size().min_element() <= 0. {
                return None;
            }
            let label = if name {
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
    let ready = world.resource::<preview::Preview>().ready;
    let value = serde_json::json!({"page":page,"appearance":appearance,"message":message,"preview_ready":ready,"widgets":geometry});
    let _ = std::fs::write(path, serde_json::to_vec(&value).unwrap());
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

/// Anchor the rig's ground origin to the painted rock ledge, independent of
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
    for (screen, mut node) in world.query::<(&Screen, &mut Node)>().iter_mut(world) {
        if screen.0 == Page::Entering {
            node.left = px(position.x);
            node.top = px(position.y);
            node.height = px(art.y * 0.68);
            node.width = px(art.x * 0.30);
        }
    }
}
fn ledge_anchor(window: Vec2, viewport: Vec2, feet_ndc: Vec2) -> Vec2 {
    let art = artwork_size(window);
    (window - art) * 0.5 + art * Vec2::new(0.185, 0.785)
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
