//! Sprinkles' compact icon/label sidebar, adapted to Theia's workspaces.
//! Upstream MIT UI and Remix Icon artwork: third_party/sprinkles/PROVENANCE.md.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::asset::{embedded_asset, load_embedded_asset};

pub(super) const WIDTH: f32 = 72.;
const TABS: [(EditorMode, &str); 6] = [
    (EditorMode::Objects, "Objects"),
    (EditorMode::Sculpt, "Sculpt"),
    (EditorMode::Paint, "Paint"),
    (EditorMode::Guides, "Guides"),
    (EditorMode::Markers, "Markers"),
    (EditorMode::Plants, "Plants"),
];
#[derive(Resource)]
pub(super) struct Icons([Handle<Image>; 6]);
#[derive(Component)]
pub(super) struct RailButton;
#[derive(Component)]
struct Plate(EditorMode);
#[derive(Component)]
struct Icon(EditorMode);

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "sidebar_icons/ri-box-2-fill.png");
    embedded_asset!(app, "sidebar_icons/ri-hashtag.png");
    embedded_asset!(app, "sidebar_icons/ri-folder-image-line.png");
    embedded_asset!(app, "sidebar_icons/ri-node-tree.png");
    embedded_asset!(app, "sidebar_icons/ri-information-fill.png");
    embedded_asset!(app, "sidebar_icons/ri-seedling-fill.png");
    let server = app.world().resource::<AssetServer>();
    let icons = Icons([
        load_embedded_asset!(server, "sidebar_icons/ri-box-2-fill.png"),
        load_embedded_asset!(server, "sidebar_icons/ri-hashtag.png"),
        load_embedded_asset!(server, "sidebar_icons/ri-folder-image-line.png"),
        load_embedded_asset!(server, "sidebar_icons/ri-node-tree.png"),
        load_embedded_asset!(server, "sidebar_icons/ri-information-fill.png"),
        load_embedded_asset!(server, "sidebar_icons/ri-seedling-fill.png"),
    ]);
    app.insert_resource(icons)
        .add_observer(activate)
        .add_zone_systems(sync);
}

pub(super) fn setup(commands: &mut Commands, icons: Option<&Icons>) {
    commands
        .spawn((
            GlobalZIndex(4),
            Node {
                position_type: PositionType::Absolute,
                top: px(workspace::HEADER),
                bottom: px(0),
                left: px(0),
                width: px(WIDTH),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(12)),
                row_gap: px(12),
                border: UiRect::right(px(1)),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|rail| {
            for (index, (mode, label)) in TABS.into_iter().enumerate() {
                rail.spawn((
                    RailButton,
                    workspace::Control::Mode(mode),
                    Button,
                    ActivateOnPress,
                    Hovered::default(),
                    theme::Variant::Ghost,
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(2),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|tab| {
                    tab.spawn((
                        Plate(mode),
                        Pickable::IGNORE,
                        Node {
                            width: px(28),
                            height: px(28),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border_radius: BorderRadius::all(px(4)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|plate| {
                        plate.spawn((
                            Icon(mode),
                            Pickable::IGNORE,
                            ImageNode {
                                image: icons
                                    .map(|icons| icons.0[index].clone())
                                    .unwrap_or_default(),
                                color: theme::BODY,
                                ..default()
                            },
                            Node {
                                width: px(16),
                                height: px(16),
                                ..default()
                            },
                        ));
                    });
                    tab.spawn((
                        Text::new(label),
                        theme::font(10.),
                        TextColor(theme::BODY),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

// Native Activate supports pointer, keyboard and accessibility through one path.
#[allow(clippy::too_many_arguments)]
pub(super) fn activate(
    event: On<Activate>,
    buttons: Query<&workspace::Control, With<RailButton>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut requests: ResMut<workspace::Requests>,
    mut content: ResMut<outliner::Content>,
) {
    if !windows.single().is_ok_and(|window| window.focused) {
        return;
    }
    if let Ok(control) = buttons.get(event.entity) {
        workspace::execute(
            *control,
            &mut session,
            &mut view,
            &mut requests,
            &mut content,
        );
    }
}

fn sync(
    session: Res<Session>,
    buttons: Query<(&workspace::Control, &Hovered), With<RailButton>>,
    mut plates: Query<(&Plate, &mut BackgroundColor)>,
    mut icons: Query<(&Icon, &mut ImageNode)>,
) {
    let active = workspace::mode(&session);
    for (control, hovered) in &buttons {
        let workspace::Control::Mode(mode) = *control else {
            continue;
        };
        let enabled = control.enabled(&session);
        let selected = active == mode;
        let variant = if !enabled {
            theme::Variant::Disabled
        } else if selected {
            theme::Variant::Active
        } else {
            theme::Variant::Ghost
        };
        for (plate, mut color) in &mut plates {
            if plate.0 == mode {
                color.0 = variant.background(hovered.0);
            }
        }
        for (icon, mut image) in &mut icons {
            if icon.0 == mode {
                image.color = if !enabled {
                    theme::MUTED
                } else if selected {
                    theme::PRIMARY
                } else {
                    theme::BODY
                };
            }
        }
    }
}
