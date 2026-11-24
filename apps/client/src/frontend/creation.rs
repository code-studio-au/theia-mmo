//! Reference composition over the real, versioned cosmetic catalogue.
use super::{ui::*, *};
use bevy::{input_focus::tab_navigation::TabIndex, text::EditableText, ui_widgets::TextInput};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Category {
    Hair,
    Face,
    Features,
    Outfit,
}
#[derive(Component, Clone, Copy)]
pub(super) enum Region {
    Heading,
    Navigation,
    Body,
    Options,
    Footer,
}
#[derive(Component)]
pub(super) struct PageContent(Category);
#[derive(Component)]
pub(super) struct Tile(usize);
#[derive(Component)]
pub(super) struct Thumbnail(usize, u8);

pub(super) fn select(current: Appearance, field: usize, value: u8) -> Appearance {
    let mut values = current.values();
    let Some(slot) = values.get_mut(field) else {
        return current;
    };
    *slot = value;
    if values[0] == 1 {
        values[3] = 0;
    }
    Appearance::new(values).unwrap_or(current)
}
pub(super) fn randomized(mut seed: u64) -> Appearance {
    // Cosmetic-only deterministic mixer. No gameplay or security randomness.
    let mut values = [0; 7];
    for (value, count) in values.iter_mut().zip(theia_core::appearance::CHOICE_COUNTS) {
        seed = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut mixed = seed;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d049bb133111eb);
        *value = ((mixed ^ (mixed >> 31)) % u64::from(count)) as u8;
    }
    if values[0] == 1 {
        values[3] = 0;
    }
    Appearance::new(values).expect("Bounded cosmetic choices")
}
fn label(parent: &mut ChildSpawnerCommands, caption: &str) {
    parent.spawn(text(caption, 22.)).insert(TextColor(GOLD));
}
fn panel(parent: &mut ChildSpawnerCommands, region: Region) -> Entity {
    parent
        .spawn((
            region,
            crate::ui_theme::Panel,
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(px(20)),
                row_gap: px(14),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(crate::ui_theme::PANEL),
        ))
        .id()
}
fn tiles(
    parent: &mut ChildSpawnerCommands,
    field: usize,
    captions: &[&str],
    atlas: Option<&Handle<Image>>,
) {
    parent
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(10),
            row_gap: px(10),
            ..default()
        })
        .with_children(|row| {
            for (value, caption) in captions.iter().enumerate() {
                let id = button(row, Choice::Appearance(field, value as u8), caption);
                row.commands().entity(id).insert((
                    crate::ui_theme::Style::Slot,
                    Tile(field),
                    AccessibleLabel::new(format!(
                        "{}: {caption}",
                        theia_core::appearance::LABELS[field]
                    )),
                    crate::ui_theme::slot::frame(Node {
                        width: percent(if field == 0 { 46. } else { 30. }),
                        height: px(if field == 0 { 62. } else { 100. }),
                        padding: UiRect {
                            left: px(8),
                            right: px(8),
                            top: px(8),
                            bottom: px(20),
                        },
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    }),
                ));
                if let Some(atlas) = atlas {
                    row.commands().entity(id).with_child((
                        Thumbnail(field, value as u8),
                        ImageNode {
                            image: atlas.clone(),
                            rect: Some(thumbnail_rect(0, field, value as u8)),
                            ..default()
                        },
                        Pickable::IGNORE,
                        Node {
                            width: px(72),
                            height: px(72),
                            min_height: px(0),
                            ..default()
                        },
                    ));
                }
            }
        });
}
fn swatches(parent: &mut ChildSpawnerCommands, field: usize, colours: &[[u8; 3]]) {
    parent
        .spawn(Node {
            column_gap: px(8),
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            for (value, colour) in colours.iter().enumerate() {
                let id = button(row, Choice::Appearance(field, value as u8), "");
                row.commands()
                    .entity(id)
                    .insert((
                        crate::ui_theme::Style::Slot,
                        AccessibleLabel::new(format!(
                            "{} {}",
                            theia_core::appearance::LABELS[field],
                            value + 1
                        )),
                        crate::ui_theme::slot::frame(Node {
                            width: px(46),
                            height: px(46),
                            padding: UiRect::all(px(6)),
                            ..default()
                        }),
                    ))
                    .with_child((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb_u8(colour[0], colour[1], colour[2])),
                        Pickable::IGNORE,
                    ));
            }
        });
}
fn unavailable(parent: &mut ChildSpawnerCommands, caption: &str) {
    let id = crate::fantasy_ui::kit::disabled(parent, caption);
    parent.commands().entity(id).insert(Node {
        width: percent(100),
        min_height: px(38),
        padding: UiRect::axes(px(16), px(5)),
        align_items: AlignItems::Center,
        ..default()
    });
}
fn navigation_unavailable(parent: &mut ChildSpawnerCommands, caption: &str, sprite: &'static str) {
    let id = crate::fantasy_ui::kit::disabled(parent, caption);
    parent
        .commands()
        .entity(id)
        .insert(Node {
            width: percent(100),
            min_height: px(50),
            padding: UiRect {
                left: px(44),
                right: px(8),
                top: px(8),
                bottom: px(8),
            },
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child((
            crate::ui_theme::Sprite(sprite),
            crate::ui_theme::Muted,
            Node {
                position_type: PositionType::Absolute,
                left: px(10),
                width: px(26),
                height: px(26),
                ..default()
            },
        ));
}
pub(super) fn setup(root: &mut ChildSpawnerCommands, atlas: Option<Handle<Image>>) {
    root.spawn((Screen(Page::Create), Placement::Creation, Node::default()))
        .with_children(|root| {
            root.spawn((
                Region::Heading,
                crate::ui_theme::Plate,
                Node {
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(px(40), px(8)),
                    ..default()
                },
            ))
            .with_child(text("Character Customisation", 30.));
            let navigation = panel(root, Region::Navigation);
            root.commands().entity(navigation).with_children(|nav| {
                navigation_unavailable(nav, "Race", "icon.party");
                for (category, caption) in [
                    (Category::Hair, "Hair"),
                    (Category::Face, "Face"),
                    (Category::Features, "Features"),
                    (Category::Outfit, "Outfit"),
                ] {
                    let id = button(nav, Choice::Category(category), caption);
                    nav.commands().entity(id).insert((
                        crate::ui_theme::Style::Tab,
                        Node {
                            width: percent(100),
                            min_height: px(50),
                            padding: UiRect {
                                left: px(44),
                                right: px(8),
                                top: px(8),
                                bottom: px(8),
                            },
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));
                    nav.commands().entity(id).with_children(|item| {
                        let icon = item
                            .spawn(Node {
                                position_type: PositionType::Absolute,
                                left: px(10),
                                width: px(26),
                                height: px(26),
                                ..default()
                            })
                            .id();
                        match category {
                            Category::Hair => {
                                item.commands()
                                    .entity(icon)
                                    .insert(crate::ui_theme::Icon(34));
                            }
                            Category::Face => {
                                item.commands()
                                    .entity(icon)
                                    .insert(crate::ui_theme::Icon(35));
                            }
                            Category::Features => {
                                item.commands()
                                    .entity(icon)
                                    .insert(crate::ui_theme::Sprite("icon.starburst"));
                            }
                            Category::Outfit => {
                                item.commands()
                                    .entity(icon)
                                    .insert(crate::ui_theme::Sprite("icon.armour"));
                            }
                        }
                    });
                }
                navigation_unavailable(nav, "Markings", "icon.feather");
            });
            let body = panel(root, Region::Body);
            root.commands().entity(body).with_children(|body| {
                crate::ui_theme::heading(body, "Appearance");
                label(body, "Body");
                tiles(body, 0, &["Masculine", "Feminine"], atlas.as_ref());
                label(body, "Skin Tone");
                swatches(
                    body,
                    4,
                    &[
                        [255, 204, 174],
                        [209, 162, 117],
                        [144, 104, 80],
                        [113, 76, 57],
                    ],
                );
                unavailable(body, "Body Type");
                unavailable(body, "Height");
                unavailable(body, "Build");
            });
            let options = panel(root, Region::Options);
            root.commands().entity(options).with_children(|options| {
                for category in [
                    Category::Hair,
                    Category::Face,
                    Category::Features,
                    Category::Outfit,
                ] {
                    options
                        .spawn((
                            PageContent(category),
                            Node {
                                row_gap: px(18),
                                ..column()
                            },
                        ))
                        .with_children(|body| match category {
                            Category::Hair => {
                                crate::ui_theme::heading(body, "Hair Style");
                                tiles(
                                    body,
                                    2,
                                    &["None", "Style 1", "Style 2", "Style 3", "Style 4"],
                                    atlas.as_ref(),
                                );
                                label(body, "Hair Colour");
                                swatches(
                                    body,
                                    5,
                                    &[[67, 54, 34], [40, 38, 36], [218, 179, 107], [165, 80, 42]],
                                );
                                unavailable(body, "Highlights");
                                unavailable(body, "Hair Accessory");
                            }
                            Category::Face => {
                                crate::ui_theme::heading(body, "Face Shape");
                                tiles(
                                    body,
                                    1,
                                    &["Face 1", "Face 2", "Face 3", "Face 4"],
                                    atlas.as_ref(),
                                );
                                unavailable(body, "Eye Colour");
                            }
                            Category::Features => {
                                crate::ui_theme::heading(body, "Facial Hair");
                                tiles(
                                    body,
                                    3,
                                    &["None", "Style 1", "Style 2", "Style 3"],
                                    atlas.as_ref(),
                                );
                                unavailable(body, "Scars");
                            }
                            Category::Outfit => {
                                crate::ui_theme::heading(body, "Outfit");
                                tiles(body, 6, &["Outfit 1", "Outfit 2"], atlas.as_ref());
                                unavailable(body, "Accessories");
                            }
                        });
                }
            });
            let footer = panel(root, Region::Footer);
            root.commands().entity(footer).with_children(|row| {
                button(row, Choice::Randomize, "Randomize");
                row.spawn(Node {
                    flex_grow: 1.,
                    ..default()
                });
                row.spawn(Node {
                    width: px(320),
                    row_gap: px(4),
                    ..column()
                })
                .with_children(|name| {
                    name.spawn(Node {
                        column_gap: px(6),
                        ..default()
                    })
                    .with_children(|title| {
                        title.spawn(text("Character name", 16.));
                        title
                            .spawn((NameError, text("", 14.)))
                            .insert(TextColor(Color::srgb(1., 0.42, 0.32)));
                    });
                    name.spawn((
                        NameInput,
                        TabIndex(0),
                        TextInput,
                        EditableText {
                            max_characters: Some(20),
                            allow_newlines: false,
                            ..default()
                        },
                        crate::ui_theme::input_box(false, false),
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
                });
                row.spawn(Node {
                    flex_grow: 1.,
                    ..default()
                });
                button(row, Choice::Back, "Back");
                button(row, Choice::Create, "Create Character");
            });
        });
}
fn widths(size: Vec2) -> (f32, f32, f32) {
    (
        (size.x * 0.12).clamp(140., 190.),
        (size.x * 0.18).clamp(220., 300.),
        (size.x * 0.29).clamp(300., 470.),
    )
}
/// Centre, ground line and drag width, shared by the model and its controls.
pub(super) fn stage(size: Vec2, page: Page) -> Vec3 {
    if page != Page::Create {
        return Vec3::new(size.x * 0.4, size.y * 0.81, size.x * 0.28);
    }
    let (rail, body, options) = widths(size);
    let left = 16. + rail + 12. + body + 12.;
    let right = size.x - 28. - options;
    Vec3::new(
        (left + right) * 0.5,
        size.y - 190.,
        (right - left).max(100.),
    )
}
#[allow(clippy::type_complexity)]
pub(super) fn sync(
    menu: Res<Menu>,
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut thumbnails: Query<
        (&Thumbnail, &mut ImageNode, &mut Node),
        (Without<Region>, Without<PageContent>, Without<Tile>),
    >,
    mut regions: Query<
        (&Region, &mut Node),
        (Without<PageContent>, Without<Tile>, Without<Thumbnail>),
    >,
    mut pages: Query<
        (&PageContent, &mut Node),
        (Without<Region>, Without<Tile>, Without<Thumbnail>),
    >,
    mut tiles: Query<
        (&Tile, &mut Node),
        (Without<Region>, Without<PageContent>, Without<Thumbnail>),
    >,
) {
    let size = windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()) / scale.0)
        .unwrap_or(Vec2::new(1600., 900.));
    let (rail, body, options) = widths(size);
    let top = (size.y * 0.15).clamp(100., 170.);
    for (region, mut node) in &mut regions {
        let (x, y, w, h) = match region {
            Region::Heading => ((size.x - 520.) * 0.5, 12., 520., 56.),
            Region::Navigation => (16., top, rail, size.y - top - 120.),
            Region::Body => (28. + rail, top, body, size.y - top - 120.),
            Region::Options => (size.x - options - 16., top, options, size.y - top - 120.),
            Region::Footer => (16., size.y - 112., size.x - 32., 96.),
        };
        node.left = px(x);
        node.top = px(y);
        node.width = px(w);
        node.height = px(h);
        if matches!(region, Region::Footer) {
            node.flex_direction = FlexDirection::Row;
            node.align_items = AlignItems::Center;
            node.column_gap = px(20);
            node.padding = UiRect::axes(px(24), px(8));
        } else if matches!(region, Region::Navigation | Region::Body | Region::Options) {
            node.padding = UiRect::all(px(if size.y < 800. { 12. } else { 20. }));
            node.row_gap = px(if size.y < 800. { 10. } else { 14. });
        }
    }
    for (thumbnail, mut image, mut node) in &mut thumbnails {
        let extent = if thumbnail.0 == 0 {
            68.
        } else if size.y < 800. {
            42.
        } else {
            76.
        };
        node.width = px(extent);
        node.height = px(extent);
        image.rect = Some(thumbnail_rect(
            menu.appearance.values()[0],
            thumbnail.0,
            thumbnail.1,
        ));
    }
    for (tile, mut node) in &mut tiles {
        node.height = px(if tile.0 == 0 {
            120.
        } else if size.y < 800. {
            94.
        } else {
            136.
        });
    }
    for (content, mut node) in &mut pages {
        node.display = if menu.category == content.0 {
            Display::Flex
        } else {
            Display::None
        };
        node.row_gap = px(if size.y < 800. { 10. } else { 18. });
    }
}

pub(super) fn thumbnail_rect(body: u8, field: usize, value: u8) -> Rect {
    let index = if field == 0 {
        usize::from(value) * 16
    } else {
        usize::from(body) * 16
            + match field {
                1 => 1,
                2 => 5,
                3 => 10,
                6 => 14,
                _ => 0,
            }
            + usize::from(value)
    };
    Rect::from_corners(
        Vec2::new((index % 8 * 256) as f32, (index / 8 * 256) as f32),
        Vec2::new(
            (index % 8 * 256 + 256) as f32,
            (index / 8 * 256 + 256) as f32,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_selection_and_randomization_preserve_supported_combinations() {
        let beard = select(Appearance::default(), 3, 3);
        assert_eq!(beard.values()[3], 3);
        let feminine = select(beard, 0, 1);
        assert_eq!(feminine.values()[3], 0);
        assert_eq!(select(feminine, 3, 2), feminine);
        assert_eq!(select(beard, 7, 1), beard);
        assert_eq!(select(beard, 2, 255), beard);
        for seed in 0..1024 {
            let appearance = randomized(seed);
            assert_eq!(Appearance::new(appearance.values()), Ok(appearance));
        }
    }
    #[test]
    fn central_drag_area_stays_between_panels_at_supported_allocations() {
        for size in [
            Vec2::new(3440., 1400.),
            Vec2::new(1720., 1400.),
            Vec2::new(1720., 700.),
            Vec2::new(1280., 720.),
        ] {
            let (rail, body, options) = widths(size);
            let stage = stage(size, Page::Create);
            assert!(stage.x - stage.z / 2. >= 28. + rail + body);
            assert!(stage.x + stage.z / 2. <= size.x - 16. - options);
            assert!(stage.y + 56. < size.y - 100.);
        }
    }
}
