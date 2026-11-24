//! Available scenery is separate from placed objects; placement keeps shared history.
use super::*;
use crate::particle_workspace::ZoneAppExt;
use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::text::EditableText;
use bevy::ui_widgets::{SelectAllOnFocus, TextInput};

#[derive(Default)]
pub(super) struct State {
    pub category: usize,
    pub selected: Option<usize>,
    query: String,
}
#[derive(Component)]
pub(super) struct Search;
#[derive(Component)]
struct Tile(usize);
#[derive(Component)]
struct Summary;
#[derive(Component)]
struct CategoryLabel;
#[derive(Component)]
struct SelectionLabel;

fn categories(library: &scenery_library::Library) -> Vec<&str> {
    library
        .assets
        .values()
        .map(|a| a.entry.category.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn matches(entry: &scenery_library::Entry, category: Option<&str>, query: &str) -> bool {
    category.is_none_or(|category| entry.category == category)
        && query.split_whitespace().all(|word| {
            [&entry.name, &entry.id, &entry.category]
                .iter()
                .any(|value| value.to_lowercase().contains(word))
        })
}
fn control(control: workspace::Control, title: &str) -> impl Bundle {
    (
        control,
        Button,
        Hovered::default(),
        theme::button_node(),
        BackgroundColor(theme::Variant::Default.background(false)),
        children![(Pickable::IGNORE, theme::label(title, 12., theme::BODY))],
    )
}
fn setup(
    mut commands: Commands,
    library: Res<scenery_library::Library>,
    preview: Res<asset_preview::Preview>,
) {
    let categories = categories(&library);
    commands
        .spawn((
            panels::Region::Assets,
            GlobalZIndex(5),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                right: px(0),
                bottom: px(status_bar::HEIGHT),
                flex_direction: FlexDirection::Column,
                border: UiRect::top(px(1)),
                ..default()
            },
            BackgroundColor(theme::PANEL),
            BorderColor::all(theme::BORDER),
        ))
        .with_children(|root| {
            root.spawn(Node {
                min_height: px(40),
                flex_shrink: 0.,
                padding: UiRect::axes(px(10), px(6)),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            })
            .with_children(|header| {
                header.spawn(theme::label("Assets", 14., theme::DISPLAY));
                let mut items = vec![(workspace::Control::AssetCategory(0), "All models", "")];
                items.extend(
                    categories
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (workspace::Control::AssetCategory(i + 1), *name, "")),
                );
                menus::spawn(header, "Category", &items);
                header.spawn((CategoryLabel, theme::label("All models", 12., theme::MUTED)));
                header.spawn(theme::label("Search", 12., theme::MUTED));
                header.spawn((
                    Search,
                    TextInput,
                    SelectAllOnFocus,
                    TabIndex(0),
                    Hovered::default(),
                    EditableText {
                        max_characters: Some(80),
                        allow_newlines: false,
                        ..default()
                    },
                    Node {
                        min_width: px(80),
                        max_width: px(280),
                        flex_grow: 1.,
                        ..theme::input_node()
                    },
                    BorderColor::all(theme::BORDER),
                    BackgroundColor(Color::NONE),
                    theme::input_font(),
                    TextColor(theme::BODY),
                    TextLayout::no_wrap(),
                    theme::text_cursor(),
                ));
                header.spawn((Summary, theme::label("", 12., theme::MUTED)));
                header.spawn(control(workspace::Control::AssetBrowser, "Close"));
            });
            root.spawn(Node {
                min_height: px(0),
                flex_grow: 1.,
                position_type: PositionType::Relative,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|body| {
                body.spawn(Node {
                    min_width: px(0),
                    min_height: px(0),
                    flex_grow: 1.,
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|body| {
                    scrolling::pane(body, 10., scrolling::Pane::Browser, |body| {
                        body.spawn(Node {
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(8),
                            row_gap: px(8),
                            width: percent(100),
                            ..default()
                        })
                        .with_children(|grid| {
                            for (index, asset) in library.assets.values().enumerate() {
                                grid.spawn((
                                    Tile(index),
                                    Node {
                                        width: px(156),
                                        ..default()
                                    },
                                ))
                                .with_children(|tile| {
                                    tile.spawn((
                                        workspace::Control::AssetSelect(index),
                                        Button,
                                        Hovered::default(),
                                        Node {
                                            width: percent(100),
                                            min_height: px(100),
                                            padding: UiRect::all(px(8)),
                                            row_gap: px(6),
                                            flex_direction: FlexDirection::Column,
                                            ..default()
                                        },
                                        BackgroundColor(theme::Variant::Default.background(false)),
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            Pickable::IGNORE,
                                            theme::label(&asset.entry.name, 13., theme::BODY),
                                        ));
                                        card.spawn((
                                            Pickable::IGNORE,
                                            theme::Secondary,
                                            theme::label(&asset.entry.category, 11., theme::MUTED),
                                        ));
                                        card.spawn((
                                            Pickable::IGNORE,
                                            theme::Secondary,
                                            theme::label(
                                                if asset.collision.is_some() {
                                                    "Solid scenery"
                                                } else {
                                                    "Decoration"
                                                },
                                                11.,
                                                theme::MUTED,
                                            ),
                                        ));
                                    });
                                });
                            }
                        });
                    });
                });
                asset_preview::panel(body, preview.camera);
            });
            root.spawn(Node {
                min_height: px(40),
                flex_shrink: 0.,
                padding: UiRect::axes(px(10), px(6)),
                column_gap: px(8),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|footer| {
                footer.spawn((
                    SelectionLabel,
                    theme::label("Select a model", 12., theme::MUTED),
                    Node {
                        flex_grow: 1.,
                        ..default()
                    },
                ));
                footer.spawn(control(workspace::Control::AssetPlace, "Place at spawn"));
            });
        });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync(
    mut session: ResMut<Session>,
    library: Res<scenery_library::Library>,
    mut tiles: Query<(&Tile, &mut Node)>,
    mut labels: Query<
        (
            &mut Text,
            Has<Summary>,
            Has<CategoryLabel>,
            Has<SelectionLabel>,
        ),
        Or<(With<Summary>, With<CategoryLabel>, With<SelectionLabel>)>,
    >,
    fields: Query<(Entity, &EditableText), With<Search>>,
    mut focus: ResMut<InputFocus>,
) {
    let visible = session.ui.assets_open && session.play.is_none();
    for (entity, text) in &fields {
        if visible {
            session.assets.query = text.value().to_string().to_lowercase();
        }
        if focus.get() == Some(entity) && !visible {
            focus.clear();
        }
    }
    let categories = categories(&library);
    let category = session
        .assets
        .category
        .checked_sub(1)
        .and_then(|i| categories.get(i).copied());
    let mut count = 0;
    for (tile, mut node) in &mut tiles {
        let shown = library
            .assets
            .values()
            .nth(tile.0)
            .is_some_and(|a| matches(&a.entry, category, &session.assets.query));
        node.display = if shown {
            count += 1;
            Display::Flex
        } else {
            Display::None
        };
    }
    // Filtering never leaves an invisible model armed for placement.
    if session.assets.selected.is_some_and(|i| {
        library
            .assets
            .values()
            .nth(i)
            .is_none_or(|a| !matches(&a.entry, category, &session.assets.query))
    }) {
        session.assets.selected = None;
    }
    for (mut text, summary, category_label, _) in &mut labels {
        text.0 = if summary {
            if library.assets.is_empty() {
                "No model library loaded".into()
            } else {
                format!("{count} / {} models", library.assets.len())
            }
        } else if category_label {
            category.unwrap_or("All models").into()
        } else {
            session
                .assets
                .selected
                .and_then(|i| library.assets.values().nth(i))
                .map_or_else(
                    || "Select a model to place".into(),
                    |a| format!("{} · {}", a.entry.name, a.entry.id),
                )
        };
    }
}
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(asset_preview::plugin)
        .add_systems(Startup, setup.after(asset_preview::setup))
        .add_zone_systems(
            sync.before(panels::resize)
                .before(workspace::sync)
                .before(keyboard),
        )
        .add_zone_systems(ownership.after(panels::resize).before(panels::apply))
        .add_systems(
            PostUpdate,
            finish_search.after(bevy::text::EditableTextSystems),
        );
}

// Release focus after editor shortcuts have been suppressed for this key press.
fn finish_search(
    mut focus: ResMut<InputFocus>,
    fields: Query<(), With<Search>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if focus.get().is_some_and(|e| fields.contains(e))
        && (keys.just_pressed(KeyCode::Escape)
            || keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter))
    {
        focus.clear();
    }
}

// A press in the browser owns the entire gesture, including release over the scene.
fn ownership(
    mut session: ResMut<Session>,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut active: Local<bool>,
) {
    let Ok(window) = windows.single() else { return };
    if !*active
        && session.ui.assets_open
        && session.play.is_none()
        && !workspace::busy(&session)
        && window.focused
        && mouse.get_just_pressed().next().is_some()
        && window.cursor_position().is_some_and(|p| {
            p.x > session.ui.canvas_left()
                && p.y >= window.height() - session.ui.canvas_bottom(window.height(), true)
                && p.y < window.height() - status_bar::HEIGHT
        })
    {
        *active = true;
    }
    if *active {
        session.ui.overlay_owned = true;
        if !window.focused
            || mouse.get_pressed().next().is_none()
            || keys.just_pressed(KeyCode::Escape)
        {
            *active = false;
        }
    }
}

#[cfg(test)]
#[path = "asset_browser_tests.rs"]
mod tests;
