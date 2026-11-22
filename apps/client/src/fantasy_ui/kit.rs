//! Reference-kit layouts. Unimplemented features remain visibly disabled widgets.
use crate::ui_theme::{self, Style};
pub(crate) mod menu;
use bevy::{
    picking::{Pickable, hover::Hovered},
    prelude::*,
    text::FontSize,
    ui::InteractionDisabled,
    ui_widgets::{Activate, Button},
};
#[derive(Component)]
pub(crate) struct Preview;
#[derive(Component, Clone, Copy)]
pub(crate) enum Value {
    FrameCap,
    Distance,
}
#[derive(Component)]
struct DropToggle(Entity, Value);
#[derive(Component)]
struct DropList;
#[derive(Resource, Default)]
struct OpenList(Option<Entity>);
#[derive(Component)]
struct Defaults;
pub(crate) fn install(app: &mut App) {
    menu::install(app);
    app.init_resource::<OpenList>()
        .init_resource::<Transaction>()
        .add_observer(activate)
        .add_observer(transaction_action)
        .add_systems(Update, (present, fit_settings))
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
        .with_child(text(caption, 16.))
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
pub(crate) fn unsupported_row(parent: &mut ChildSpawnerCommands, caption: &str, value: &str) {
    parent
        .spawn(Node {
            width: percent(100),
            column_gap: px(10),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|r| {
            r.spawn((
                text(caption, 14.),
                TextColor(Color::srgb(0.45, 0.47, 0.50)),
                Node {
                    flex_grow: 1.,
                    ..default()
                },
            ));
            r.spawn((
                Button,
                InteractionDisabled,
                Hovered::default(),
                Style::Secondary,
                AccessibleLabel::new(format!("{caption} — unavailable")),
                dropdown_node(px(170)),
                BackgroundColor(Color::NONE),
            ))
            .with_children(|p| {
                p.spawn(text(value, 15.));
                chevron(p, true);
            });
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
            r.spawn((text(caption, 17.), TextColor(Color::srgb(0.48, 0.49, 0.49))));
        });
}
pub(crate) fn unsupported_slider(parent: &mut ChildSpawnerCommands, caption: &str, value: &str) {
    parent
        .spawn(Node {
            column_gap: px(10),
            align_items: AlignItems::Center,
            height: px(26),
            ..default()
        })
        .with_children(|r| {
            r.spawn((
                text(caption, 14.),
                TextColor(Color::srgb(0.45, 0.47, 0.50)),
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
            r.spawn((
                text(value, 14.),
                TextColor(Color::srgb(0.45, 0.47, 0.50)),
                Node {
                    width: px(50),
                    ..default()
                },
            ));
        });
}
fn chevron(parent: &mut ChildSpawnerCommands, disabled: bool) {
    parent.spawn((
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
        BorderColor::all(if disabled {
            Color::srgb(0.40, 0.40, 0.38)
        } else {
            ui_theme::GOLD
        }),
        UiTransform::from_rotation(Rot2::degrees(45.)),
        Pickable::IGNORE,
    ));
}
fn dropdown_node(width: Val) -> Node {
    Node {
        width,
        height: px(32),
        flex_shrink: 0.,
        padding: UiRect {
            left: px(14),
            right: px(32),
            ..default()
        },
        align_items: AlignItems::Center,
        ..default()
    }
}
pub(crate) fn dropdown(
    parent: &mut ChildSpawnerCommands,
    value: Value,
    options: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: px(170),
            ..column()
        })
        .with_children(|holder| {
            let list = holder
                .spawn((
                    DropList,
                    GlobalZIndex(1400),
                    ui_theme::Panel,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: percent(100),
                        width: percent(100),
                        display: Display::None,
                        padding: UiRect::all(px(8)),
                        ..column()
                    },
                    BackgroundColor(ui_theme::PANEL),
                ))
                .with_children(options)
                .id();
            holder
                .spawn((
                    DropToggle(list, value),
                    Button,
                    Hovered::default(),
                    Style::Secondary,
                    dropdown_node(percent(100)),
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|p| {
                    p.spawn((value, text("", 15.)));
                    chevron(p, false);
                });
        });
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
        .with_child(text("Restore Defaults", 16.));
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
) {
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
                section(display, "Display");
                for (label, value) in [
                    ("Display Mode", "Windowed"),
                    ("Resolution", "Current"),
                    ("Refresh Rate", "Desktop"),
                    ("Monitor", "Primary"),
                ] {
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
                section(preset, "Graphics Preset");
                preset
                    .spawn(Node {
                        column_gap: px(4),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|r| {
                        for title in ["Low", "Medium", "High", "Ultra", "Custom"] {
                            disabled(r, title);
                        }
                    });
                preset.spawn((
                    Preview,
                    Node {
                        width: percent(100),
                        height: px(182),
                        border: UiRect::all(px(1)),
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
                    section(advanced, "Advanced Graphics");
                    advanced
                        .spawn(Node {
                            column_gap: px(18),
                            ..default()
                        })
                        .with_children(|cols| {
                            cols.spawn(Node {
                                width: percent(53),
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
                                quality
                                    .spawn(Node {
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        column_gap: px(8),
                                        ..default()
                                    })
                                    .with_children(|row| {
                                        row.spawn(text("View Distance", 14.));
                                        dropdown(row, Value::Distance, |p| {
                                            choices(p, Value::Distance)
                                        });
                                    });
                            });
                            cols.spawn(Node {
                                width: percent(47),
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
                                    "Ray Tracing (High End)",
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
                    section(performance, "Performance");
                    ui_theme::vsync_control(performance);
                    performance
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|r| {
                            r.spawn(text("Frame Rate Limit", 14.));
                            dropdown(r, Value::FrameCap, |p| choices(p, Value::FrameCap));
                        });
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
        .with_child(text(caption, 18.))
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
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
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
        for (e, a) in &actions {
            if matches!(a, Footer::Apply) {
                if baseline == current {
                    commands.entity(e).insert(InteractionDisabled);
                } else {
                    commands.entity(e).remove::<InteractionDisabled>();
                }
            }
        }
    } else if let Some(base) = transaction.baseline.take() {
        cap.0 = base.cap;
        vsync.0 = base.vsync;
        **distance = base.distance;
        scale.0 = base.scale;
    }
}

pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let dropdowns:Vec<_>=world.query::<(&DropToggle,&ComputedNode,&bevy::ui::UiGlobalTransform)>().iter(world).filter(|(_,n,_)|n.size().min_element()>0.).map(|(d,n,t)|serde_json::json!({"name":match d.1{Value::FrameCap=>"fps-menu",Value::Distance=>"view-menu"},"point":(t.affine().translation*super::window_factor(n,scale)).to_array()})).collect();
    let footer:Vec<_>=world.query::<(&Footer,&ComputedNode,&bevy::ui::UiGlobalTransform,Has<InteractionDisabled>)>().iter(world).filter(|(_,n,_,_)|n.size().min_element()>0.).map(|(a,n,t,disabled)|serde_json::json!({"name":match a{Footer::Apply=>"apply",Footer::Accept=>"ok",Footer::Cancel=>"cancel"},"disabled":disabled,"point":(t.affine().translation*super::window_factor(n,scale)).to_array()})).collect();
    serde_json::json!({"dropdowns":dropdowns,"footer":footer})
}

#[cfg(test)]
mod tests {
    use super::*;
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
