//! Reference-kit layouts. Unimplemented features remain visibly disabled widgets.
use super::display;
use crate::ui_theme::{self, Style};
pub(crate) mod menu;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::{BackgroundGradient, InteractionDisabled, LinearGradient},
    ui_widgets::{Activate, Button},
};
#[derive(Component)]
pub(crate) struct Preview;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Value {
    FrameCap,
    Distance,
    DisplayMode,
    Resolution,
}
impl Value {
    fn name(self) -> &'static str {
        match self {
            Self::FrameCap => "fps-menu",
            Self::Distance => "view-menu",
            Self::DisplayMode => "display-mode",
            Self::Resolution => "resolution",
        }
    }
}
#[derive(Component)]
struct DropToggle(Entity, Value);
#[derive(Component)]
struct DropCaption(Entity);
#[derive(Component)]
struct DropChevron;
#[derive(Component)]
struct DropList;
#[derive(Component, Default)]
struct ResolutionRows(Vec<[u32; 2]>);
#[derive(Component, Clone, Copy)]
enum DisplayOption {
    Mode(display::Mode),
    Size([u32; 2]),
}
#[derive(Resource, Default)]
struct OpenList(Option<Entity>);
#[derive(Component)]
struct Defaults;
#[derive(Component)]
struct SettingsClose;
pub(crate) fn install(app: &mut App) {
    menu::install(app);
    display::install(app);
    app.init_resource::<OpenList>()
        .init_resource::<Transaction>()
        .add_observer(activate)
        .add_observer(transaction_action)
        .add_systems(
            Update,
            (
                present,
                fit_settings,
                display_options,
                scroll_resolutions,
                dropdown_colors,
            ),
        )
        .add_systems(
            PostUpdate,
            transactions
                .after(crate::ui_theme::SourceStyles)
                .before(bevy::ui::UiSystems::Layout),
        );
}
pub(crate) fn text(value: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        Pickable::IGNORE,
    )
}
pub(crate) fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        min_width: px(0),
        ..default()
    }
}
pub(crate) fn icon(parent: &mut ChildSpawnerCommands, index: usize, muted: bool) {
    let mut e = parent.spawn((
        ui_theme::Icon(index + 6),
        Node {
            width: px(32),
            height: px(32),
            flex_shrink: 0.,
            ..default()
        },
    ));
    if muted {
        e.insert(ui_theme::Muted);
    }
}
pub(crate) fn disabled(parent: &mut ChildSpawnerCommands, caption: &str) -> Entity {
    parent
        .spawn((
            Button,
            InteractionDisabled,
            Hovered::default(),
            Style::Secondary,
            AccessibleLabel::new(format!("{caption} — unavailable")),
            Node {
                min_height: px(32),
                padding: UiRect::axes(px(16), px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_child(ui_theme::control_label(caption))
        .id()
}
pub(crate) fn disabled_link(parent: &mut ChildSpawnerCommands, caption: &str) {
    let e = disabled(parent, caption);
    parent.commands().entity(e).insert((
        Style::Link,
        Node {
            padding: UiRect::all(px(4)),
            ..default()
        },
    ));
}
pub(crate) fn tabs(parent: &mut ChildSpawnerCommands, captions: &[&str], active: usize) {
    parent
        .spawn(Node {
            column_gap: px(4),
            width: percent(100),
            ..default()
        })
        .with_children(|row| {
            for (i, caption) in captions.iter().enumerate() {
                let id = disabled(row, caption);
                row.commands().entity(id).insert(Style::Tab);
                row.commands().entity(id).insert(Node {
                    flex_grow: 1.,
                    min_height: px(32),
                    padding: UiRect::axes(px(12), px(4)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                });
                if i == active {
                    row.commands()
                        .entity(id)
                        .remove::<InteractionDisabled>()
                        .insert(ui_theme::Selected);
                }
            }
        });
}
pub(crate) fn title(parent: &mut ChildSpawnerCommands, caption: &str, icon_index: usize) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(px(10)),
            ..default()
        })
        .with_children(|r| {
            r.spawn((
                ui_theme::Plate,
                Node {
                    width: px(360),
                    height: px(50),
                    padding: UiRect::axes(px(32), px(6)),
                    column_gap: px(12),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                icon(p, icon_index, false);
                p.spawn((
                    text(caption, 28.),
                    TextColor(ui_theme::GOLD),
                    TextShadow {
                        offset: Vec2::ONE,
                        color: Color::BLACK,
                    },
                ));
            });
        });
}
pub(crate) fn section(parent: &mut ChildSpawnerCommands, caption: &str) {
    parent
        .spawn((
            ui_theme::Plate,
            Node {
                width: percent(100),
                height: px(32),
                padding: UiRect::horizontal(px(32)),
                align_items: AlignItems::Center,
                margin: UiRect::bottom(px(6)),
                ..default()
            },
        ))
        .with_child((text(caption, 22.), TextColor(ui_theme::GOLD)));
}

pub(crate) fn settings_panel_gradient(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(7),
            right: px(7),
            top: px(7),
            bottom: px(7),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            Color::srgba(0.065, 0.155, 0.24, 0.45).into(),
            (Color::srgba(0.012, 0.043, 0.07, 0.32), percent(53)).into(),
            Color::srgba(0.055, 0.12, 0.18, 0.42).into(),
        ])),
        Pickable::IGNORE,
    ));
}

pub(crate) fn settings_header(parent: &mut ChildSpawnerCommands, close: impl Component) {
    parent
        .spawn(Node {
            width: percent(100),
            height: px(34),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|header| {
            header
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: px(-38),
                    left: px(0),
                    right: px(0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ui_theme::Plate,
                        Node {
                            width: px(360),
                            height: px(57),
                            column_gap: px(12),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ))
                    .with_children(|plate| {
                        plate.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(18),
                                right: percent(18),
                                top: px(7),
                                bottom: px(7),
                                border_radius: BorderRadius::all(px(9)),
                                ..default()
                            },
                            BackgroundGradient::from(LinearGradient::to_right(vec![
                                Color::NONE.into(),
                                (Color::srgba(0.08, 0.27, 0.42, 0.52), percent(49)).into(),
                                Color::NONE.into(),
                            ])),
                            Pickable::IGNORE,
                        ));
                        highlight(plate, px(4), 64., true);
                        icon(plate, 7, false);
                        plate.spawn((
                            text("Settings", 30.),
                            TextColor(ui_theme::CREAM),
                            title_shadow(),
                        ));
                    });
                });
            header
                .spawn((
                    SettingsClose,
                    Footer::Cancel,
                    close,
                    Button,
                    Hovered::default(),
                    Style::Icon,
                    AccessibleLabel::new("Close Settings"),
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(-5),
                        right: px(2),
                        width: px(38),
                        height: px(38),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_child((text("×", 38.), TextColor(ui_theme::GOLD), title_shadow()));
        });
}

fn title_shadow() -> TextShadow {
    TextShadow {
        offset: Vec2::splat(1.5),
        color: Color::BLACK,
    }
}

fn highlight(parent: &mut ChildSpawnerCommands, top: Val, width: f32, diamond: bool) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            top,
            left: percent((100. - width) / 2.),
            width: percent(width),
            height: px(2),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_right(vec![
            Color::NONE.into(),
            (Color::srgba(0.36, 0.78, 1.0, 0.82), percent(48)).into(),
            Color::NONE.into(),
        ])),
        Pickable::IGNORE,
    ));
    if diamond {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(2),
                left: percent(50),
                width: px(5),
                height: px(5),
                ..default()
            },
            BackgroundColor(ui_theme::CREAM),
            UiTransform::from_rotation(Rot2::degrees(45.)),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn settings_page_heading(
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    icon_index: usize,
) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(48),
                column_gap: px(11),
                padding: UiRect::horizontal(px(14)),
                align_items: AlignItems::Center,
                border: UiRect::bottom(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.78, 0.58, 0.29, 0.56)),
            BackgroundGradient::from(LinearGradient::to_right(vec![
                Color::srgb(0.035, 0.104, 0.165).into(),
                Color::srgba(0.024, 0.072, 0.12, 0.22).into(),
            ])),
        ))
        .with_children(|row| {
            icon(row, icon_index, false);
            row.spawn((
                text(caption, 27.),
                TextColor(ui_theme::CREAM),
                title_shadow(),
            ));
            highlight(row, px(46), 80., false);
        });
}

pub(crate) fn settings_section(parent: &mut ChildSpawnerCommands, caption: &str) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(32),
                padding: UiRect {
                    left: px(26),
                    right: px(19),
                    ..default()
                },
                align_items: AlignItems::Center,
                border: UiRect::vertical(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.74, 0.56, 0.29, 0.55)),
            BackgroundGradient::from(LinearGradient::to_right(vec![
                Color::srgb(0.047, 0.191, 0.32).into(),
                (Color::srgb(0.035, 0.125, 0.216), percent(55)).into(),
                Color::srgba(0.021, 0.065, 0.113, 0.18).into(),
            ])),
        ))
        .with_children(|bar| {
            border_corners(bar, false, 22.);
            highlight(bar, px(0), 62., false);
            bar.spawn((
                text(caption, 22.),
                TextColor(ui_theme::CREAM),
                title_shadow(),
            ));
        });
}

pub(crate) fn settings_columns(
    parent: &mut ChildSpawnerCommands,
    left: impl FnOnce(&mut ChildSpawnerCommands),
    right: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: percent(100),
            column_gap: px(20),
            ..default()
        })
        .with_children(|row| {
            let column = Node {
                flex_grow: 1.,
                flex_basis: px(0),
                ..column()
            };
            row.spawn(column.clone()).with_children(left);
            row.spawn(column).with_children(right);
        });
}

pub(crate) fn settings_scale(parent: &mut ChildSpawnerCommands) {
    settings_section(parent, "UI Scale");
    ui_theme::scale_control(parent);
}

pub(crate) fn settings_sidebar() -> impl Bundle {
    (
        ui_theme::Panel,
        Node {
            width: px(220),
            flex_shrink: 0.,
            padding: UiRect::all(px(10)),
            row_gap: px(2),
            ..column()
        },
        BackgroundColor(ui_theme::PANEL),
    )
}

pub(crate) fn settings_tab<T: Component>(
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    index: usize,
    action: Option<T>,
) -> Entity {
    let enabled = action.is_some();
    let mut tab = parent.spawn((
        Button,
        Hovered::default(),
        Style::SettingsTab,
        AccessibleLabel::new(caption),
        Node {
            height: px(44),
            flex_shrink: 0.,
            padding: UiRect::horizontal(px(14)),
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    if let Some(action) = action {
        tab.insert(action);
    } else {
        tab.insert(InteractionDisabled);
    }
    tab.with_children(|row| {
        icon(row, index, !enabled);
        row.spawn(ui_theme::control_label(caption));
        row.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(0),
                left: px(12),
                right: px(12),
                height: px(1),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                Color::NONE.into(),
                (Color::srgba(0.77, 0.57, 0.30, 0.34), percent(55)).into(),
                Color::NONE.into(),
            ])),
            Pickable::IGNORE,
        ));
    });
    tab.id()
}

pub(crate) fn settings_watermark(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        ui_theme::Sprite("crest.tree-dark"),
        ui_theme::Muted,
        Node {
            width: px(82),
            height: px(95),
            align_self: AlignSelf::Center,
            margin: UiRect {
                top: Val::Auto,
                bottom: px(12),
                ..default()
            },
            ..default()
        },
    ));
}

fn border_corners(parent: &mut ChildSpawnerCommands, right_edge: bool, size: f32) {
    // Offset the transparent sprite margin so the gold meets the border.
    let inset = px(-(size * 0.135).round());
    let (left, right, horizontal_scale) = if right_edge {
        (Val::Auto, inset, -1.)
    } else {
        (inset, Val::Auto, 1.)
    };
    for (top, bottom, vertical_scale) in [(inset, Val::Auto, 1.), (Val::Auto, inset, -1.)] {
        parent.spawn((
            ui_theme::Sprite("ornament.corner-fine-left"),
            Node {
                position_type: PositionType::Absolute,
                left,
                right,
                top,
                bottom,
                width: px(size),
                height: px(size),
                ..default()
            },
            UiTransform::from_scale(Vec2::new(horizontal_scale, vertical_scale)),
            Pickable::IGNORE,
        ));
    }
}

pub(crate) fn settings_content<'a>(parent: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    let mut content = parent.spawn((
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(0),
            padding: UiRect::all(px(11)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            ..column()
        },
        BorderColor::all(Color::srgba(0.77, 0.59, 0.33, 0.43)),
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            Color::srgba(0.028, 0.086, 0.139, 0.42).into(),
            Color::srgba(0.01, 0.034, 0.052, 0.53).into(),
        ])),
    ));
    content.with_children(|panel| {
        border_corners(panel, false, 32.);
        border_corners(panel, true, 32.);
    });
    content
}
fn dropdown_row(
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    field: impl FnOnce(&mut ChildSpawnerCommands, Entity),
) {
    parent
        .spawn(Node {
            width: percent(100),
            column_gap: px(10),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|row| {
            let label = row.spawn(ui_theme::control_label(caption)).id();
            field(row, label);
        });
}
fn unsupported_row(parent: &mut ChildSpawnerCommands, caption: &str, value: &str) {
    dropdown_row(parent, caption, |row, label| {
        dropdown_field(
            row,
            label,
            format!("{caption} — unavailable"),
            ui_theme::control_label(value),
        )
        .insert(InteractionDisabled);
    });
}
pub(crate) fn unsupported_check(parent: &mut ChildSpawnerCommands, caption: &str) {
    parent
        .spawn((
            bevy::ui_widgets::Checkbox,
            InteractionDisabled,
            AccessibleLabel::new(format!("{caption} — unavailable")),
            ui_theme::checkbox_row(),
        ))
        .with_children(|r| {
            r.spawn((
                ui_theme::checkbox_box(),
                ui_theme::Sprite("checkbox.disabled"),
            ));
            r.spawn(ui_theme::control_label(caption))
                .insert(TextColor(ui_theme::DISABLED_TEXT));
        });
}
fn unsupported_slider(parent: &mut ChildSpawnerCommands, caption: &str, value: &str) {
    parent
        .spawn(Node {
            column_gap: px(10),
            align_items: AlignItems::Center,
            min_height: px(32),
            ..default()
        })
        .with_children(|r| {
            r.spawn(ui_theme::control_label(caption)).insert((
                TextColor(ui_theme::DISABLED_TEXT),
                Node {
                    width: px(110),
                    ..default()
                },
            ));
            r.spawn((
                bevy::ui_widgets::Slider::default(),
                InteractionDisabled,
                AccessibleLabel::new(format!("{caption} — unavailable")),
                Node {
                    height: px(12),
                    flex_grow: 1.,
                    ..default()
                },
                ui_theme::Sprite("slider.track"),
                ui_theme::Muted,
            ))
            .with_child((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(50),
                    top: px(-6),
                    width: px(20),
                    height: px(24),
                    ..default()
                },
                ui_theme::Sprite("slider.thumb"),
                ui_theme::Muted,
            ));
            r.spawn(ui_theme::control_label(value)).insert((
                TextColor(ui_theme::DISABLED_TEXT),
                Node {
                    width: px(50),
                    ..default()
                },
            ));
        });
}
fn chevron(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        DropChevron,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            top: px(9),
            width: px(8),
            height: px(8),
            border: UiRect {
                right: px(2),
                bottom: px(2),
                ..default()
            },
            ..default()
        },
        BorderColor::all(ui_theme::GOLD),
        UiTransform::from_rotation(Rot2::degrees(45.)),
        Pickable::IGNORE,
    ));
}
const DROPDOWN_WIDTH: f32 = 170.;
const CONTROL_HEIGHT: f32 = 32.;

fn dropdown_field<'a>(
    parent: &'a mut ChildSpawnerCommands,
    label: Entity,
    caption: String,
    value: impl Bundle,
) -> EntityCommands<'a> {
    let mut field = parent.spawn((
        Button,
        Hovered::default(),
        Style::Dropdown,
        AccessibleLabel::new(caption),
        Node {
            width: px(DROPDOWN_WIDTH),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            height: px(CONTROL_HEIGHT),
            flex_shrink: 0.,
            padding: UiRect {
                left: px(14),
                right: px(32),
                ..default()
            },
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    field.with_children(|field| {
        field.spawn(value);
        chevron(field);
    });
    let field_id = field.id();
    field.commands().entity(label).insert(DropCaption(field_id));
    field
}

fn dropdown_colors(
    buttons: Query<Has<InteractionDisabled>, With<Button>>,
    mut captions: Query<(&DropCaption, &mut TextColor)>,
    mut chevrons: Query<(&ChildOf, &mut BorderColor), With<DropChevron>>,
) {
    for (caption, mut text) in &mut captions {
        let color = if buttons.get(caption.0).is_ok_and(|disabled| disabled) {
            ui_theme::DISABLED_TEXT
        } else {
            ui_theme::CREAM
        };
        if text.0 != color {
            text.0 = color;
        }
    }
    for (parent, mut border) in &mut chevrons {
        let color = if buttons.get(parent.parent()).is_ok_and(|disabled| disabled) {
            ui_theme::DISABLED_TEXT
        } else {
            ui_theme::GOLD
        };
        let next = BorderColor::all(color);
        if *border != next {
            *border = next;
        }
    }
}

fn dropdown(
    parent: &mut ChildSpawnerCommands,
    caption: &str,
    value: Value,
    options: impl FnOnce(&mut ChildSpawnerCommands),
) {
    dropdown_row(parent, caption, |row, label| {
        row.spawn(Node {
            width: px(DROPDOWN_WIDTH),
            flex_shrink: 0.,
            ..column()
        })
        .with_children(|holder| {
            let down = matches!(value, Value::DisplayMode | Value::Resolution);
            let list = holder
                .spawn((
                    DropList,
                    GlobalZIndex(1400),
                    ui_theme::Panel,
                    Node {
                        position_type: PositionType::Absolute,
                        top: if down { percent(100) } else { Val::Auto },
                        bottom: if down { Val::Auto } else { percent(100) },
                        width: percent(100),
                        display: Display::None,
                        padding: UiRect::all(px(8)),
                        ..column()
                    },
                    BackgroundColor(ui_theme::PANEL),
                ))
                .with_children(options)
                .id();
            if value == Value::Resolution {
                holder.commands().entity(list).insert((
                    ResolutionRows::default(),
                    ScrollPosition::default(),
                    Hovered::default(),
                ));
            }
            dropdown_field(
                holder,
                label,
                caption.into(),
                (value, ui_theme::control_label("")),
            )
            .insert(DropToggle(list, value));
        });
    });
}

fn display_row(parent: &mut ChildSpawnerCommands, label: &str, value: Value) {
    dropdown(parent, label, value, |options| {
        if value == Value::DisplayMode {
            for mode in display::Mode::ALL {
                dropdown_option(options, DisplayOption::Mode(mode), mode.label());
            }
        }
    });
}

pub(crate) fn dropdown_option<'a>(
    parent: &'a mut ChildSpawnerCommands,
    action: impl Component,
    label: &str,
) -> EntityCommands<'a> {
    let mut option = parent.spawn((
        action,
        Button,
        Hovered::default(),
        Style::Dropdown,
        AccessibleLabel::new(label),
        Node {
            min_height: px(CONTROL_HEIGHT),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            padding: UiRect::horizontal(px(14)),
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        },
    ));
    option.with_child(ui_theme::control_label(label));
    option
}

fn display_options(
    mut commands: Commands,
    state: Res<display::State>,
    toggles: Query<(Entity, &DropToggle, Has<InteractionDisabled>)>,
    options: Query<(Entity, &DisplayOption, Has<InteractionDisabled>)>,
    mut rows: Query<(Entity, &mut ResolutionRows, &mut Node, &mut ScrollPosition)>,
) {
    for (entity, toggle, disabled) in &toggles {
        let enabled = match toggle.1 {
            Value::DisplayMode => state.editable(),
            Value::Resolution => state.resolution_editable(),
            _ => continue,
        };
        if enabled == disabled {
            if enabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
    }
    for (entity, option, disabled) in &options {
        if let DisplayOption::Mode(mode) = option
            && disabled == state.allows(*mode)
        {
            if disabled {
                commands.entity(entity).remove::<InteractionDisabled>();
            } else {
                commands.entity(entity).insert(InteractionDisabled);
            }
        }
    }
    if rows.is_empty() {
        return;
    }
    let sizes = state.choices();
    for (entity, mut displayed, mut node, mut scroll) in &mut rows {
        if displayed.0 != sizes {
            displayed.0.clone_from(&sizes);
            node.max_height = px(285);
            node.overflow = Overflow::scroll_y();
            scroll.y = 0.;
            commands
                .entity(entity)
                .despawn_related::<Children>()
                .with_children(|options| {
                    for &size in &sizes {
                        let label = display::size_label(size);
                        dropdown_option(options, DisplayOption::Size(size), &label);
                    }
                });
        }
    }
}

fn scroll_resolutions(
    opened: Res<OpenList>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut rows: Query<(&Hovered, &mut ScrollPosition), With<ResolutionRows>>,
) {
    for event in wheel.read() {
        if let Some(entity) = opened.0
            && let Ok((hovered, mut position)) = rows.get_mut(entity)
            && hovered.get()
        {
            let amount = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 38.,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };
            position.y = (position.y - amount).max(0.);
        }
    }
}
pub(crate) fn settings_body() -> Node {
    Node {
        column_gap: px(24),
        flex_grow: 1.,
        flex_basis: px(0),
        min_height: px(0),
        ..default()
    }
}
pub(crate) fn settings_footer() -> Node {
    Node {
        justify_content: JustifyContent::SpaceBetween,
        flex_shrink: 0.,
        margin: UiRect::top(px(12)),
        ..default()
    }
}
pub(crate) fn defaults(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Defaults,
            Button,
            Hovered::default(),
            Style::Secondary,
            Node {
                min_height: px(36),
                padding: UiRect::axes(px(20), px(6)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_child(ui_theme::control_label("Restore Defaults"));
}
#[allow(clippy::too_many_arguments)]
fn activate(
    event: On<Activate>,
    drops: Query<&DropToggle>,
    defaults: Query<(), With<Defaults>>,
    mut opened: ResMut<OpenList>,
    mut cap: Option<ResMut<super::graphics::FrameCap>>,
    mut vsync: Option<ResMut<super::graphics::Vsync>>,
    mut distance: Option<ResMut<super::graphics::ViewDistance>>,
    mut scale: Option<ResMut<super::PreferredScale>>,
    mut display: Option<ResMut<display::State>>,
    display_options: Query<(&DisplayOption, &ChildOf), Without<InteractionDisabled>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if let Some(ref mut state) = display
        && let Ok((option, parent)) = display_options.get(event.entity)
        && opened.0 == Some(parent.parent())
        && windows.single().is_ok_and(|window| window.focused)
    {
        match *option {
            DisplayOption::Mode(mode) => state.select_mode(mode),
            DisplayOption::Size(size) => state.select_size(size),
        }
    }
    if let Ok(toggle) = drops.get(event.entity) {
        opened.0 = if opened.0 == Some(toggle.0) {
            None
        } else {
            Some(toggle.0)
        };
    } else {
        opened.0 = None;
    }
    if defaults.contains(event.entity) {
        if let Some(ref mut state) = display {
            state.defaults();
        }
        if let Some(ref mut v) = cap {
            **v = default();
        }
        if let Some(ref mut v) = vsync {
            **v = default();
        }
        if let Some(ref mut v) = distance {
            **v = default();
        }
        if let Some(ref mut v) = scale {
            v.0 = 1.;
        }
    }
}
fn present(
    opened: Res<OpenList>,
    mut lists: Query<(Entity, &mut Node), With<DropList>>,
    mut labels: Query<(&Value, &mut Text)>,
    cap: Option<Res<super::graphics::FrameCap>>,
    distance: Option<Res<super::graphics::ViewDistance>>,
    display_state: Res<display::State>,
) {
    for (e, mut n) in &mut lists {
        let display = if opened.0 == Some(e) {
            Display::Flex
        } else {
            Display::None
        };
        if n.display != display {
            n.display = display;
        }
    }
    for (value, mut text) in &mut labels {
        let label = match value {
            Value::FrameCap => cap.as_deref().copied().unwrap_or_default().label(),
            Value::Distance => distance
                .as_deref()
                .copied()
                .unwrap_or_default()
                .label()
                .into(),
            Value::DisplayMode => display_state.label(),
            Value::Resolution => display_state.resolution_label(),
        };
        let next = label;
        if text.0 != next {
            text.0 = next;
        }
    }
}
/// Full reference graphics layout. The callback supplies only supported choices.
pub(crate) fn graphics(
    parent: &mut ChildSpawnerCommands,
    mut choices: impl FnMut(&mut ChildSpawnerCommands, Value),
) {
    parent
        .spawn(Node {
            column_gap: px(20),
            ..default()
        })
        .with_children(|top| {
            top.spawn(Node {
                width: percent(48),
                ..column()
            })
            .with_children(|display| {
                settings_section(display, "Display");
                display_row(display, "Display Mode", Value::DisplayMode);
                display_row(display, "Resolution", Value::Resolution);
                for (label, value) in [("Refresh Rate", "Desktop"), ("Monitor", "Primary")] {
                    unsupported_row(display, label, value);
                }
                for (label, value) in [
                    ("Brightness", "50%"),
                    ("Contrast", "50%"),
                    ("Gamma", "1.00"),
                ] {
                    unsupported_slider(display, label, value);
                }
            });
            top.spawn(Node {
                width: percent(52),
                ..column()
            })
            .with_children(|preset| {
                settings_section(preset, "Graphics Preset");
                preset
                    .spawn(Node {
                        column_gap: px(8),
                        ..default()
                    })
                    .with_children(|r| {
                        for title in ["Low", "Medium", "High", "Ultra", "Custom"] {
                            let id = disabled(r, title);
                            r.commands().entity(id).insert(Node {
                                flex_grow: 1.,
                                flex_basis: px(0),
                                min_width: px(0),
                                min_height: px(CONTROL_HEIGHT),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            });
                        }
                    });
                preset.spawn((
                    Preview,
                    Node {
                        width: percent(100),
                        height: px(225),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(12)),
                        ..default()
                    },
                    BorderColor::all(ui_theme::GOLD),
                    Pickable::IGNORE,
                ));
            });
        });
    parent
        .spawn(Node {
            column_gap: px(18),
            margin: UiRect::top(px(12)),
            ..default()
        })
        .with_children(|bottom| {
            bottom
                .spawn(Node {
                    width: percent(63),
                    ..column()
                })
                .with_children(|advanced| {
                    settings_section(advanced, "Advanced Graphics");
                    advanced
                        .spawn(Node {
                            column_gap: px(18),
                            ..default()
                        })
                        .with_children(|cols| {
                            cols.spawn(Node {
                                width: percent(50),
                                ..column()
                            })
                            .with_children(|quality| {
                                for caption in [
                                    "Texture Quality",
                                    "Shadow Quality",
                                    "Lighting Quality",
                                    "Effect Quality",
                                    "Terrain Quality",
                                    "Vegetation Quality",
                                ] {
                                    unsupported_row(quality, caption, "Unavailable");
                                }
                                dropdown(quality, "View Distance", Value::Distance, |options| {
                                    choices(options, Value::Distance)
                                });
                            });
                            cols.spawn(Node {
                                width: percent(50),
                                row_gap: px(0),
                                ..column()
                            })
                            .with_children(|effects| {
                                unsupported_row(effects, "Anti-Aliasing", "Unavailable");
                                unsupported_row(effects, "Anisotropic Filtering", "Unavailable");
                                for caption in [
                                    "Ambient Occlusion",
                                    "Screen Space Reflections",
                                    "Bloom",
                                    "Motion Blur",
                                    "Depth of Field",
                                    "Volumetric Clouds",
                                    "Weather Effects",
                                ] {
                                    unsupported_check(effects, caption);
                                }
                            });
                        });
                });
            bottom
                .spawn(Node {
                    width: percent(37),
                    ..column()
                })
                .with_children(|performance| {
                    settings_section(performance, "Performance");
                    ui_theme::vsync_control(performance);
                    dropdown(
                        performance,
                        "Frame Rate Limit",
                        Value::FrameCap,
                        |options| choices(options, Value::FrameCap),
                    );
                    unsupported_row(performance, "Background FPS", "30 FPS");
                    unsupported_check(performance, "Reduce CPU Load (Background)");
                    unsupported_check(performance, "Dynamic Resolution");
                    unsupported_slider(performance, "Quality Scaling", "100%");
                });
        });
}

/// Fixed reference composition fitted to the window independently of HUD scale.
#[derive(Component)]
pub(crate) struct SettingsFrame;
fn fit_settings(
    windows: Query<&Window>,
    scale: Res<bevy::ui::UiScale>,
    mut frames: Query<&mut UiTransform, With<SettingsFrame>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let logical = Vec2::new(window.width(), window.height()) / scale.0;
    let fit = ((logical.x - 32.) / 1440.)
        .min((logical.y - 32.) / 940.)
        .clamp(0.1, 1.);
    for mut transform in &mut frames {
        let next = UiTransform::from_scale(Vec2::splat(fit));
        if *transform != next {
            *transform = next;
        }
    }
}
#[derive(Component)]
pub(crate) struct SettingsSurface;
#[derive(Component, Clone, Copy)]
pub(crate) enum Footer {
    Apply,
    Accept,
    Cancel,
}
#[derive(Clone, Copy, PartialEq)]
struct Values {
    cap: u16,
    vsync: bool,
    distance: super::graphics::ViewDistance,
    scale: f32,
}
#[derive(Resource, Default)]
struct Transaction {
    baseline: Option<Values>,
}
pub(crate) fn footer_button(
    parent: &mut ChildSpawnerCommands,
    action: Footer,
    caption: &str,
) -> Entity {
    parent
        .spawn((
            action,
            Button,
            Hovered::default(),
            Style::Secondary,
            Node {
                width: px(130),
                min_height: px(38),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_child(ui_theme::control_label(caption))
        .id()
}
#[allow(clippy::too_many_arguments)]
fn transaction_action(
    event: On<Activate>,
    actions: Query<&Footer>,
    mut transaction: ResMut<Transaction>,
    mut cap: Option<ResMut<super::graphics::FrameCap>>,
    mut vsync: Option<ResMut<super::graphics::Vsync>>,
    mut distance: Option<ResMut<super::graphics::ViewDistance>>,
    mut scale: Option<ResMut<super::PreferredScale>>,
    mut display: Option<ResMut<display::State>>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    if let Some(ref mut display) = display {
        if matches!(action, Footer::Cancel) {
            display.clear();
        } else if let Ok(mut window) = windows.single_mut()
            && window.focused
        {
            display.preview(&mut window, std::time::Instant::now());
        }
    }
    let (Some(cap), Some(vsync), Some(distance), Some(scale)) =
        (&mut cap, &mut vsync, &mut distance, &mut scale)
    else {
        return;
    };
    if matches!(action, Footer::Cancel) {
        if let Some(base) = transaction.baseline {
            cap.0 = base.cap;
            vsync.0 = base.vsync;
            **distance = base.distance;
            scale.0 = base.scale;
        }
    } else {
        transaction.baseline = Some(Values {
            cap: cap.0,
            vsync: vsync.0,
            distance: **distance,
            scale: scale.0,
        });
    }
}
#[allow(clippy::too_many_arguments)]
fn transactions(
    mut commands: Commands,
    roots: Query<&Node, With<SettingsSurface>>,
    actions: Query<(Entity, &Footer)>,
    mut transaction: ResMut<Transaction>,
    mut cap: Option<ResMut<super::graphics::FrameCap>>,
    mut vsync: Option<ResMut<super::graphics::Vsync>>,
    mut distance: Option<ResMut<super::graphics::ViewDistance>>,
    mut scale: Option<ResMut<super::PreferredScale>>,
    mut display: Option<ResMut<display::State>>,
    mut opened: ResMut<OpenList>,
) {
    let (Some(cap), Some(vsync), Some(distance), Some(scale)) =
        (&mut cap, &mut vsync, &mut distance, &mut scale)
    else {
        return;
    };
    let current = Values {
        cap: cap.0,
        vsync: vsync.0,
        distance: **distance,
        scale: scale.0,
    };
    if roots.iter().any(|n| n.display != Display::None) {
        let baseline = *transaction.baseline.get_or_insert(current);
        let display_changed = display.as_deref().is_some_and(display::State::changed);
        for (e, a) in &actions {
            if matches!(a, Footer::Apply) {
                if baseline == current && !display_changed {
                    commands.entity(e).insert(InteractionDisabled);
                } else {
                    commands.entity(e).remove::<InteractionDisabled>();
                }
            }
        }
    } else {
        if opened.0.is_some() {
            opened.0 = None;
        }
        if let Some(display) = &mut display {
            display.clear();
        }
        if let Some(base) = transaction.baseline.take() {
            cap.0 = base.cap;
            vsync.0 = base.vsync;
            **distance = base.distance;
            scale.0 = base.scale;
        }
    }
}

pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let dropdowns:Vec<_>=world.query::<(&DropToggle,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world).filter(|(_,n,_)|n.size().min_element()>0.).map(|(d,n,t)|serde_json::json!({"name":d.1.name(),"point":(t.affine().translation*super::window_factor(n,scale)).to_array()})).collect();
    let display_options: Vec<_> = world.query::<(&DisplayOption, &AccessibleLabel, &ComputedNode, &bevy::ui::UiGlobalTransform)>().iter(world).filter(|(_,_,n,_)|n.size().min_element()>0.).map(|(_,label,n,t)|serde_json::json!({"name":&*label.0,"point":(t.affine().translation*super::window_factor(n,scale)).to_array()})).collect();
    let footer:Vec<_>=world.query_filtered::<(&Footer,&ComputedNode,&bevy::ui::UiGlobalTransform,Has<InteractionDisabled>),Without<SettingsClose>>().iter(world).filter(|(_,n,_,_)|n.size().min_element()>0.).map(|(a,n,t,disabled)|serde_json::json!({"name":match a{Footer::Apply=>"apply",Footer::Accept=>"ok",Footer::Cancel=>"cancel"},"disabled":disabled,"point":(t.affine().translation*super::window_factor(n,scale)).to_array()})).collect();
    serde_json::json!({"dropdowns":dropdowns,"footer":footer,"display_options":display_options,"display_confirmation":display::snapshot(world, scale)})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphics_controls_share_type_and_dropdown_dimensions_in_all_states() {
        let mut app = App::new();
        app.add_systems(Update, dropdown_colors);
        app.world_mut()
            .commands()
            .spawn(Node::default())
            .with_children(|parent| {
                graphics(parent, |list, _| {
                    dropdown_option(
                        list,
                        DisplayOption::Mode(display::Mode::Windowed),
                        "Test choice",
                    );
                });
            });
        app.update();
        let world = app.world_mut();
        let labels: Vec<_> = world
            .query::<(&DropCaption, &TextColor)>()
            .iter(world)
            .map(|(caption, color)| (caption.0, color.0))
            .collect();
        assert!(labels.len() >= 12);
        for (field, color) in labels {
            let node = world.get::<Node>(field).unwrap();
            assert_eq!(node.width, px(DROPDOWN_WIDTH));
            assert_eq!(node.height, px(CONTROL_HEIGHT));
            assert!(matches!(world.get::<Style>(field), Some(Style::Dropdown)));
            let disabled = world.get::<InteractionDisabled>(field).is_some();
            assert_eq!(
                color,
                if disabled {
                    ui_theme::DISABLED_TEXT
                } else {
                    ui_theme::CREAM
                }
            );
        }
        let mut fonts = world.query::<(&Text, &TextFont)>();
        for (text, font) in fonts.iter(world) {
            let expected = match text.0.as_str() {
                "Display" | "Graphics Preset" | "Advanced Graphics" | "Performance" => 22.,
                _ => ui_theme::CONTROL_TEXT_SIZE,
            };
            assert_eq!(font.font_size, FontSize::Px(expected), "{}", text.0);
        }
    }

    #[test]
    fn disabled_reference_controls_do_not_change_settings_and_cancel_restores_last_apply() {
        let mut app = App::new();
        app.init_resource::<OpenList>()
            .init_resource::<Transaction>()
            .init_resource::<super::super::graphics::FrameCap>()
            .init_resource::<super::super::graphics::Vsync>()
            .init_resource::<super::super::graphics::ViewDistance>()
            .insert_resource(super::super::PreferredScale(1.))
            .add_observer(activate)
            .add_observer(transaction_action)
            .add_systems(Update, transactions);
        let root = app
            .world_mut()
            .commands()
            .spawn((SettingsSurface, Node::default()))
            .with_children(|p| {
                graphics(p, |_, _| {});
                footer_button(p, Footer::Apply, "Apply");
                footer_button(p, Footer::Cancel, "Cancel");
            })
            .id();
        app.world_mut().flush();
        app.update();
        let disabled: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<InteractionDisabled>>()
            .iter(app.world())
            .collect();
        assert!(
            disabled.len() > 20,
            "Reference controls are visibly disabled, not omitted"
        );
        for entity in disabled {
            app.world_mut().trigger(Activate { entity });
        }
        assert_eq!(
            app.world().resource::<super::super::graphics::FrameCap>().0,
            60
        );
        app.world_mut()
            .resource_mut::<super::super::graphics::FrameCap>()
            .0 = 120;
        app.update();
        let apply = app
            .world_mut()
            .query::<(Entity, &Footer)>()
            .iter(app.world())
            .find(|(_, a)| matches!(a, Footer::Apply))
            .unwrap()
            .0;
        assert!(app.world().get::<InteractionDisabled>(apply).is_none());
        app.world_mut().trigger(Activate { entity: apply });
        app.world_mut()
            .resource_mut::<super::super::graphics::FrameCap>()
            .0 = 30;
        let cancel = app
            .world_mut()
            .query::<(Entity, &Footer)>()
            .iter(app.world())
            .find(|(_, a)| matches!(a, Footer::Cancel))
            .unwrap()
            .0;
        app.world_mut().trigger(Activate { entity: cancel });
        assert_eq!(
            app.world().resource::<super::super::graphics::FrameCap>().0,
            120
        );
        app.world_mut()
            .resource_mut::<super::super::graphics::FrameCap>()
            .0 = 240;
        app.world_mut().get_mut::<Node>(root).unwrap().display = Display::None;
        app.update();
        assert_eq!(
            app.world().resource::<super::super::graphics::FrameCap>().0,
            120,
            "Closing without Apply restores the last committed value"
        );
    }
}
