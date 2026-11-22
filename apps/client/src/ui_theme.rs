//! Shared Theia presentation for native game controls. Decorations never take input.
use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::{Checked, InteractionDisabled, Pressed},
    ui_widgets::{
        Button, Checkbox, Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb,
        SliderValue, TrackClick, ValueChange,
    },
};
pub(crate) const GOLD: Color = Color::srgb(0.85, 0.66, 0.34);
pub(crate) const CREAM: Color = Color::srgb(0.95, 0.90, 0.78);
pub(crate) const PANEL: Color = Color::srgba(0.018, 0.033, 0.048, 0.97);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SourceStyles;
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component, Default, Clone, Copy)]
pub(crate) enum Style {
    Primary,
    #[default]
    Secondary,
    Link,
    Input,
}
#[derive(Component)]
pub(crate) struct Selected;
#[derive(Component)]
struct Decorated;
#[derive(Component)]
struct ButtonArt(Entity);
#[derive(Component)]
pub(crate) struct Icon(pub usize);
#[derive(Component)]
struct Vsync;
#[derive(Component)]
struct Scale;
#[derive(Component)]
struct Tick(Entity);
#[derive(Component)]
struct Thumb(Entity);
#[derive(Component)]
struct ScaleCaption;
#[derive(Resource)]
struct Art {
    frame: Handle<Image>,
    button: Handle<Image>,
    icons: Handle<Image>,
}
pub(crate) fn install(app: &mut App) {
    if app.world().contains_resource::<Art>() {
        return;
    }
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut load = |bytes: &[u8]| {
        images.add(
            Image::from_buffer(
                bytes,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::NONE,
                true,
                default(),
                default(),
            )
            .expect("Embedded Theia UI artwork"),
        )
    };
    let art = Art {
        frame: load(include_bytes!("../../../content/ui/theme/frame.png")),
        button: load(include_bytes!("../../../content/ui/theme/primary.png")),
        icons: load(include_bytes!("../../../content/ui/theme/icons.png")),
    };
    app.insert_resource(art)
        .add_observer(change_vsync)
        .add_observer(change_scale)
        .add_systems(
            PostUpdate,
            (decorate, present, preferences, widget_focus)
                .chain()
                .after(SourceStyles)
                .before(bevy::ui::UiSystems::Content),
        );
}
fn absolute() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(0),
        top: px(0),
        bottom: px(0),
        ..default()
    }
}
#[allow(clippy::type_complexity)]
fn decorate(
    mut commands: Commands,
    art: Res<Art>,
    panels: Query<Entity, (With<Panel>, Without<Decorated>)>,
    buttons: Query<(Entity, Option<&Style>, &Children), (With<Button>, Without<Decorated>)>,
    texts: Query<&Text>,
    images: Query<&ImageNode>,
    icons: Query<(Entity, &Icon), Without<ImageNode>>,
) {
    for entity in &panels {
        commands
            .entity(entity)
            .insert((Decorated, Outline::new(px(1), px(-1), GOLD)))
            .with_children(|p| {
                // Nine slicing retains corner detail at different panel sizes.
                p.spawn((
                    Pickable::IGNORE,
                    bevy::ui::IgnoreScroll(BVec2::TRUE),
                    Node {
                        left: px(-6),
                        right: px(-6),
                        top: px(-9),
                        bottom: px(-9),
                        ..absolute()
                    },
                    ImageNode {
                        image: art.frame.clone(),
                        image_mode: NodeImageMode::Sliced(bevy::sprite::TextureSlicer {
                            border: bevy::sprite::BorderRect::all(200.),
                            max_corner_scale: 0.16,
                            ..default()
                        }),
                        ..default()
                    },
                ));
            });
    }
    for (entity, style, children) in &buttons {
        if children.iter().any(|c| images.contains(c)) {
            continue;
        }
        if !children
            .iter()
            .any(|c| texts.get(c).is_ok_and(|t| !t.0.is_empty()))
        {
            continue;
        }
        let style = style.copied().unwrap_or_default();
        commands.entity(entity).insert((Decorated, style));
        if matches!(style, Style::Link | Style::Input) {
            continue;
        }
        commands
            .entity(entity)
            .insert(BorderColor::all(Color::NONE))
            .with_child((
                ButtonArt(entity),
                Pickable::IGNORE,
                ZIndex(-1),
                absolute(),
                ImageNode {
                    image: art.button.clone(),
                    rect: Some(Rect::new(0., 180., 2176., 532.)),
                    image_mode: NodeImageMode::Sliced(bevy::sprite::TextureSlicer {
                        border: bevy::sprite::BorderRect::axes(180., 70.),
                        max_corner_scale: 0.12,
                        ..default()
                    }),
                    ..default()
                },
            ));
    }
    for (entity, icon) in &icons {
        let x = (icon.0 % 3) as f32 * 512.;
        let y = (icon.0 / 3) as f32 * 512.;
        commands.entity(entity).insert((
            Pickable::IGNORE,
            ImageNode {
                image: art.icons.clone(),
                rect: Some(Rect::new(x + 100., y + 100., x + 412., y + 412.)),
                ..default()
            },
        ));
    }
}
#[allow(clippy::type_complexity)]
fn present(
    focus: Option<Res<InputFocus>>,
    mut buttons: Query<(
        Entity,
        &Style,
        Option<&Hovered>,
        Has<Pressed>,
        Has<InteractionDisabled>,
        Has<Selected>,
        &Children,
        &mut BackgroundColor,
    )>,
    mut arts: Query<(&ButtonArt, &mut ImageNode)>,
    mut texts: Query<&mut TextColor, With<Text>>,
    mut commands: Commands,
) {
    for (entity, style, hovered, pressed, disabled, selected, children, mut background) in
        &mut buttons
    {
        let hover = hovered.is_some_and(|h| h.0);
        let focused = focus.as_ref().is_some_and(|f| f.get() == Some(entity));
        let color = if disabled {
            Color::srgb(0.48, 0.49, 0.49)
        } else if pressed || selected {
            GOLD
        } else if hover {
            Color::srgb(1., 0.91, 0.64)
        } else {
            CREAM
        };
        for child in children {
            if let Ok(mut text) = texts.get_mut(*child)
                && text.0 != color
            {
                text.0 = color;
            }
        }
        if focused && !disabled {
            commands
                .entity(entity)
                .insert(Outline::new(px(1), px(3), GOLD));
        } else {
            commands.entity(entity).remove::<Outline>();
        }
        background.0 = if matches!(style, Style::Input) {
            Color::srgba(0., 0., 0., 0.85)
        } else {
            Color::NONE
        };
    }
    for (parent, mut image) in &mut arts {
        if let Ok((_, style, hover, pressed, disabled, selected, _, _)) = buttons.get(parent.0) {
            image.color = if disabled {
                Color::srgb(0.25, 0.27, 0.28)
            } else if pressed || selected {
                Color::srgb(1., 0.78, 0.40)
            } else if hover.is_some_and(|h| h.0) {
                Color::WHITE
            } else if matches!(style, Style::Primary) {
                Color::srgb(0.92, 0.92, 0.92)
            } else {
                Color::srgb(0.46, 0.49, 0.52)
            };
        }
    }
}
fn label(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(17.),
        TextColor(CREAM),
        Pickable::IGNORE,
    )
}
/// Shared, accessible controls bound to the same persisted preferences in all screens.
pub(crate) fn controls(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Vsync,
            Checkbox,
            TabIndex(0),
            Hovered::default(),
            AccessibleLabel::new("Vertical synchronisation"),
            Node {
                min_height: px(32),
                column_gap: px(12),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            let check = row.target_entity();
            row.spawn((
                Node {
                    width: px(22),
                    height: px(22),
                    border: UiRect::all(px(1)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(GOLD),
                Pickable::IGNORE,
            ))
            .with_children(|cell| {
                cell.spawn((
                    Tick(check),
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        width: px(16),
                        height: px(16),
                        ..default()
                    },
                ))
                .with_children(|mark| {
                    for (left, top, height, angle) in [(3., 7., 6., -42.), (8., 3., 12., 42.)] {
                        mark.spawn((
                            Pickable::IGNORE,
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(left),
                                top: px(top),
                                width: px(3),
                                height: px(height),
                                ..default()
                            },
                            UiTransform::from_rotation(Rot2::degrees(angle)),
                            BackgroundColor(GOLD),
                        ));
                    }
                });
            });
            row.spawn(label("Vertical synchronisation"));
        });
    parent.spawn((ScaleCaption, label("Interface size")));
    parent
        .spawn((
            Scale,
            Slider {
                track_click: TrackClick::Snap,
                ..default()
            },
            SliderRange::new(0.75, 1.5),
            SliderStep(0.05),
            SliderPrecision(2),
            SliderValue(1.),
            TabIndex(0),
            Hovered::default(),
            AccessibleLabel::new("Interface size"),
            Node {
                width: percent(100),
                height: px(28),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|slider| {
            let entity = slider.target_entity();
            slider.spawn((
                Pickable::IGNORE,
                Node {
                    width: percent(100),
                    height: px(6),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.02, 0.10, 0.15)),
                BorderColor::all(GOLD),
            ));
            slider
                .spawn((
                    Pickable::IGNORE,
                    Node {
                        right: px(20),
                        ..absolute()
                    },
                ))
                .with_children(|rail| {
                    rail.spawn((
                        Thumb(entity),
                        SliderThumb,
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(20),
                            height: px(28),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_child((
                        Pickable::IGNORE,
                        Node {
                            width: px(13),
                            height: px(13),
                            border: UiRect::all(px(2)),
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::degrees(45.)),
                        BackgroundColor(PANEL),
                        BorderColor::all(GOLD),
                    ));
                });
        });
}
fn change_vsync(
    event: On<ValueChange<bool>>,
    controls: Query<(), With<Vsync>>,
    mut vsync: Option<ResMut<crate::fantasy_ui::graphics::Vsync>>,
) {
    if controls.contains(event.source)
        && let Some(vsync) = vsync.as_mut()
    {
        vsync.0 = event.value;
    }
}
fn change_scale(
    event: On<ValueChange<f32>>,
    controls: Query<(), With<Scale>>,
    mut scale: Option<ResMut<crate::fantasy_ui::PreferredScale>>,
) {
    if controls.contains(event.source)
        && event.value.is_finite()
        && let Some(scale) = scale.as_mut()
    {
        scale.0 = event.value.clamp(0.75, 1.5);
    }
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn preferences(
    mut commands: Commands,
    vsync: Option<Res<crate::fantasy_ui::graphics::Vsync>>,
    scale: Option<Res<crate::fantasy_ui::PreferredScale>>,
    checks: Query<(Entity, Has<Checked>), With<Vsync>>,
    sliders: Query<(Entity, &SliderValue), With<Scale>>,
    mut ticks: Query<(&Tick, &mut Visibility)>,
    mut thumbs: Query<(&Thumb, &mut Node)>,
    mut captions: Query<&mut Text, With<ScaleCaption>>,
) {
    for (entity, checked) in &checks {
        let value = vsync.as_ref().is_some_and(|v| v.0);
        if checked != value {
            if value {
                commands.entity(entity).insert(Checked);
            } else {
                commands.entity(entity).remove::<Checked>();
            }
        }
    }
    for (tick, mut visibility) in &mut ticks {
        *visibility = if checks.get(tick.0).is_ok_and(|(_, c)| c) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(scale) = scale {
        for (entity, value) in &sliders {
            if value.0 != scale.0 {
                commands.entity(entity).insert(SliderValue(scale.0));
            }
        }
        for (thumb, mut node) in &mut thumbs {
            if sliders.contains(thumb.0) {
                node.left = percent((scale.0 - 0.75) / 0.75 * 100.);
            }
        }
        for mut text in &mut captions {
            text.0 = format!("Interface size  {:.0}%", scale.0 * 100.);
        }
    }
}

#[allow(clippy::type_complexity)]
fn widget_focus(
    mut commands: Commands,
    focus: Option<Res<InputFocus>>,
    widgets: Query<
        (Entity, Option<&Hovered>, Has<InteractionDisabled>),
        Or<(
            With<Checkbox>,
            With<Slider>,
            With<bevy::ui_widgets::TextInput>,
        )>,
    >,
) {
    for (entity, hover, disabled) in &widgets {
        let focused = focus.as_ref().is_some_and(|f| f.get() == Some(entity));
        if !disabled && (focused || hover.is_some_and(|h| h.0)) {
            commands.entity(entity).insert(Outline::new(
                px(if focused { 2. } else { 1. }),
                px(3),
                GOLD,
            ));
        } else {
            commands.entity(entity).remove::<Outline>();
        }
    }
}

/// Geometry only, for the existing opt-in native input harness.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let controls: Vec<_> = world.query::<(&ComputedNode, &bevy::ui::UiGlobalTransform, Has<Vsync>, Has<Scale>)>().iter(world)
        .filter(|(node,_,check,slider)| (*check || *slider) && node.size().min_element()>0.)
        .map(|(node,transform,check,_)| {
            let factor=crate::fantasy_ui::window_factor(node,scale);
            serde_json::json!({"name":if check {"vsync"} else {"scale"},"point":(transform.affine().translation*factor).to_array(),"size":(node.size()*factor).to_array()})
        }).collect();
    serde_json::json!({"controls":controls,"scale":world.get_resource::<crate::fantasy_ui::PreferredScale>().map(|v|v.0),"vsync":world.get_resource::<crate::fantasy_ui::graphics::Vsync>().map(|v|v.0)})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preference_controls_spawn_and_share_the_real_preferences() {
        let mut app = App::new();
        app.insert_resource(crate::fantasy_ui::PreferredScale(1.))
            .insert_resource(crate::fantasy_ui::graphics::Vsync(false))
            .add_observer(change_vsync)
            .add_observer(change_scale)
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Node::default()).with_children(controls);
            });
        app.update();
        let checkbox = app
            .world_mut()
            .query_filtered::<Entity, With<Vsync>>()
            .single(app.world())
            .unwrap();
        let slider = app
            .world_mut()
            .query_filtered::<Entity, With<Scale>>()
            .single(app.world())
            .unwrap();
        let thumb = app
            .world_mut()
            .query_filtered::<Entity, With<Thumb>>()
            .single(app.world())
            .unwrap();
        let ornament = app
            .world()
            .get::<Children>(thumb)
            .expect("The visible diamond must move with the thumb")[0];
        assert!(app.world().get::<UiTransform>(ornament).is_some());
        app.world_mut().trigger(ValueChange {
            source: checkbox,
            value: true,
            is_final: true,
        });
        app.world_mut().trigger(ValueChange {
            source: slider,
            value: 1.25_f32,
            is_final: true,
        });
        assert!(
            app.world()
                .resource::<crate::fantasy_ui::graphics::Vsync>()
                .0
        );
        assert_eq!(
            app.world()
                .resource::<crate::fantasy_ui::PreferredScale>()
                .0,
            1.25
        );
        app.world_mut().trigger(ValueChange {
            source: slider,
            value: f32::NAN,
            is_final: true,
        });
        assert_eq!(
            app.world()
                .resource::<crate::fantasy_ui::PreferredScale>()
                .0,
            1.25
        );
    }
}
