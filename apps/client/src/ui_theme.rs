//! Shared native Theia controls, decorated with the approved regenerated sprites.
pub(crate) mod slot;
pub(crate) mod sprites;
use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    picking::{Pickable, hover::Hovered},
    prelude::*,
    ui::{Checked, InteractionDisabled, Pressed},
    ui_widgets::{
        Button, Checkbox, Slider, SliderPrecision, SliderRange, SliderStep, SliderThumb,
        SliderValue, TextInput, TrackClick, ValueChange,
    },
};
use sprites::Sprites;
pub(crate) const GOLD: Color = Color::srgb(0.85, 0.66, 0.34);
pub(crate) const CREAM: Color = Color::srgb(0.95, 0.90, 0.78);
pub(crate) const PANEL: Color = Color::srgb(0.018, 0.045, 0.075);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SourceStyles;
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component, Default, Clone, Copy)]
pub(crate) enum Style {
    Primary,
    Gold,
    #[default]
    Secondary,
    Link,
    Icon,
    Input,
    Roster,
    Close,
    Slot,
    Tab,
}
#[derive(Component)]
pub(crate) struct Selected;
#[derive(Component)]
pub(crate) struct Decorated;
#[derive(Component)]
struct ButtonArt(Entity);
/// Static decoration; never participates in input or gameplay.
#[derive(Component)]
pub(crate) struct Sprite(pub &'static str);
#[derive(Component)]
pub(crate) struct Icon(pub usize);
#[derive(Component)]
pub(crate) struct Muted;
#[derive(Component)]
pub(crate) struct Plate;
#[derive(Component)]
struct Vsync;
#[derive(Component)]
struct Scale;
#[derive(Component)]
struct Tick(Entity);
#[derive(Component)]
struct CheckArt(Entity);
#[derive(Component)]
struct Thumb(Entity);
#[derive(Component)]
struct ScaleCaption;
#[derive(Resource)]
struct Art {
    standalone: std::collections::BTreeMap<usize, Handle<Image>>,
    preview: Handle<Image>,
}
pub(crate) fn install(app: &mut App) {
    if app.world().contains_resource::<Art>() {
        return;
    }
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let sprites = Sprites::load(&mut images);
    let mut load = |b: &[u8]| images.add(sprites::decode(b));
    let mut standalone = std::collections::BTreeMap::new();
    macro_rules! icon {
        ($file:literal, $($index:literal),+) => {
            let image = load(include_bytes!(concat!("../../../content/ui/theme/standalone-v3/", $file, ".png")));
            $(standalone.insert($index, image.clone());)+
        };
    }
    icon!("lock", 1);
    icon!("eye", 2, 28);
    icon!("settings", 3, 13);
    icon!("support", 4, 30);
    icon!("exit", 5, 14);
    icon!("graphics", 15);
    icon!("controls", 17);
    icon!("keyboard", 18);
    icon!("interface", 19);
    icon!("accessibility", 22);
    icon!("language", 23);
    icon!("rotate", 31);
    icon!("zoom-out", 32);
    icon!("zoom-in", 33);
    icon!("hair", 34);
    icon!("face", 35);
    icon!("mail", 36);
    let art = Art {
        standalone,
        preview: load(include_bytes!(
            "../../../content/ui/frontend/character-courtyard-v1.png"
        )),
    };
    app.insert_resource(sprites)
        .insert_resource(art)
        .add_observer(change_vsync)
        .add_observer(change_scale)
        .add_systems(
            PostUpdate,
            (
                decorate,
                present,
                icon_hover,
                preferences,
                widget_focus,
                slider_focus,
            )
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
fn family(style: Style) -> Option<&'static str> {
    Some(match style {
        Style::Primary | Style::Gold => "button.primary",
        Style::Secondary => "button.secondary",
        Style::Input => "input.standard",
        Style::Roster | Style::Tab => "tab.chamfered",
        Style::Close => "control.close",
        Style::Link | Style::Icon | Style::Slot => return None,
    })
}
fn state(
    disabled: bool,
    pressed: bool,
    selected: bool,
    focused: bool,
    hovered: bool,
) -> &'static str {
    if disabled {
        "disabled"
    } else if pressed {
        "pressed"
    } else if selected {
        "selected"
    } else if focused {
        "focused"
    } else if hovered {
        "hover"
    } else {
        "default"
    }
}
fn icon_name(index: usize) -> Option<&'static str> {
    Some(match index {
        0 | 6 => "icon.character",
        1 => "control.lock.default",
        3 | 13 => "control.settings.default",
        4 => "icon.help",
        5 | 14 => "icon.exit",
        7 => "icon.bag",
        8 => "icon.swords",
        9 | 11 => "icon.book",
        10 => "icon.compass",
        12 => "icon.trophy",
        16 => "icon.audio-volume",
        20 => "icon.target-round",
        21 => "icon.party",
        22 => "icon.character",
        23 => "icon.compass",
        24 => "icon.shield",
        25 => "icon.coins",
        26 => "icon.crown",
        27 => "control.plus-round",
        29 => "icon.chat",
        _ => return None,
    })
}
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn decorate(
    mut commands: Commands,
    art: Res<Art>,
    sprites: Res<Sprites>,
    panels: Query<Entity, (With<Panel>, Without<Decorated>)>,
    buttons: Query<
        (
            Entity,
            Option<&Style>,
            Option<&Children>,
            Has<TextInput>,
            Has<crate::fantasy_ui::HeaderClose>,
        ),
        (Or<(With<Button>, With<TextInput>)>, Without<Decorated>),
    >,
    texts: Query<&Text>,
    images: Query<&ImageNode>,
    icons: Query<(Entity, &Icon, Has<Muted>), Without<ImageNode>>,
    plates: Query<Entity, (With<Plate>, Without<ImageNode>)>,
    previews: Query<Entity, (With<crate::fantasy_ui::kit::Preview>, Without<ImageNode>)>,
    statics: Query<(Entity, &Sprite, Has<Muted>), Without<ImageNode>>,
) {
    for entity in &panels {
        commands
            .entity(entity)
            .insert((
                Decorated,
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::NONE),
            ))
            .remove::<Outline>()
            .with_child((
                Pickable::IGNORE,
                ZIndex(-1),
                bevy::ui::IgnoreScroll(BVec2::TRUE),
                absolute(),
                sprites.node("panel.navy"),
            ));
    }
    for (entity, style, children, input, close) in &buttons {
        let children: Vec<Entity> = children.map(|c| c.iter().collect()).unwrap_or_default();
        if style.is_none() && !input && !close && children.iter().any(|c| images.contains(*c)) {
            continue;
        }
        if style.is_none()
            && !input
            && !close
            && !children
                .iter()
                .any(|c| texts.get(*c).is_ok_and(|t| !t.0.is_empty()))
        {
            continue;
        }
        let style = if input {
            Style::Input
        } else if close {
            Style::Close
        } else {
            style.copied().unwrap_or_default()
        };
        commands
            .entity(entity)
            .insert((Decorated, style, BorderColor::all(Color::NONE)))
            .remove::<Outline>();
        if close {
            for c in &children {
                if texts.contains(*c) {
                    commands.entity(*c).insert(Visibility::Hidden);
                }
            }
        }
        if let Some(family) = family(style) {
            let image = sprites.node(&format!("{family}.default"));
            if input {
                // TextInput paints its own glyphs on this entity. A child image
                // always paints above its parent's text, even with negative ZIndex.
                commands.entity(entity).insert((ButtonArt(entity), image));
            } else {
                commands.entity(entity).with_child((
                    ButtonArt(entity),
                    Pickable::IGNORE,
                    ZIndex(-1),
                    absolute(),
                    image,
                ));
            }
        }
    }
    for entity in &previews {
        commands
            .entity(entity)
            .insert(ImageNode::new(art.preview.clone()));
    }
    for entity in &plates {
        commands.entity(entity).insert(sprites.node("title.simple"));
    }
    for (entity, sprite, muted) in &statics {
        let mut node = sprites.node(sprite.0);
        if muted {
            node.color = Color::srgb(0.35, 0.35, 0.35);
        }
        commands.entity(entity).remove::<Outline>().insert((
            Pickable::IGNORE,
            node,
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
        ));
    }
    for (entity, icon, muted) in &icons {
        let mut node = if let Some(image) = art.standalone.get(&icon.0) {
            ImageNode::new(image.clone())
        } else if let Some(name) = icon_name(icon.0) {
            sprites.node(name)
        } else {
            continue;
        };
        if muted {
            node.color = Color::srgb(0.35, 0.35, 0.35);
        }
        commands.entity(entity).insert((Pickable::IGNORE, node));
    }
}
#[allow(clippy::type_complexity)]
fn present(
    focus: Option<Res<InputFocus>>,
    sprites: Res<Sprites>,
    mut buttons: Query<(
        Entity,
        &Style,
        Option<&Hovered>,
        Has<Pressed>,
        Has<InteractionDisabled>,
        Has<Selected>,
        Option<&Children>,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut arts: Query<(&ButtonArt, &mut ImageNode)>,
    mut texts: Query<&mut TextColor, With<Text>>,
    mut commands: Commands,
) {
    for (entity, style, hover, pressed, disabled, selected, children, mut background, mut border) in
        &mut buttons
    {
        let focused = focus.as_ref().is_some_and(|f| f.get() == Some(entity));
        let color = if disabled {
            Color::srgb(0.48, 0.49, 0.49)
        } else if pressed || selected {
            Color::WHITE
        } else if hover.is_some_and(|h| h.0) {
            Color::srgb(1., 0.91, 0.64)
        } else {
            CREAM
        };
        for child in children.into_iter().flat_map(|c| c.iter()) {
            if let Ok(mut t) = texts.get_mut(child)
                && t.0 != color
            {
                t.0 = color;
            }
        }
        if matches!(style, Style::Slot) {
            let (well, edge) = slot::colors(state(
                disabled,
                pressed,
                selected,
                focused,
                hover.is_some_and(|h| h.0),
            ));
            if background.0 != well {
                background.0 = well;
            }
            if *border != edge {
                *border = edge;
            }
            continue;
        }
        if matches!(style, Style::Link) && focused && !disabled {
            commands
                .entity(entity)
                .insert(Outline::new(px(1), px(3), GOLD));
        } else {
            commands.entity(entity).remove::<Outline>();
        }
        if background.0 != Color::NONE {
            background.0 = Color::NONE;
        }
        if *border != BorderColor::all(Color::NONE) {
            *border = BorderColor::all(Color::NONE);
        }
    }
    for (parent, mut image) in &mut arts {
        if let Ok((entity, style, hover, pressed, disabled, selected, _, _, _)) =
            buttons.get(parent.0)
        {
            let focused = focus.as_ref().is_some_and(|f| f.get() == Some(entity));
            let name = format!(
                "{}.{}",
                family(*style).expect("Decorated control family"),
                state(
                    disabled,
                    pressed,
                    selected || matches!(style, Style::Gold),
                    focused,
                    hover.is_some_and(|h| h.0)
                )
            );
            let next = sprites.node(&name);
            if image.image != next.image || image.rect != next.rect || image.color != next.color {
                *image = next;
            }
        }
    }
}
/// Only interactive link glyphs brighten. Field account/lock glyphs stay static.
fn icon_hover(
    focus: Option<Res<InputFocus>>,
    mut icons: Query<(&Icon, &ChildOf, &mut ImageNode, Has<Muted>)>,
    controls: Query<(Option<&Hovered>, Has<Pressed>, Has<InteractionDisabled>)>,
) {
    for (icon, parent, mut image, muted) in &mut icons {
        if !matches!(icon.0, 2..=5 | 31..=33) || muted {
            continue;
        }
        let Ok((hover, pressed, disabled)) = controls.get(parent.parent()) else {
            continue;
        };
        let brightness = if disabled {
            0.35
        } else if pressed {
            0.7
        } else if hover.is_some_and(|h| h.0)
            || focus
                .as_ref()
                .is_some_and(|f| f.get() == Some(parent.parent()))
        {
            1.25
        } else {
            1.0
        };
        image.color = Color::srgb(brightness, brightness, brightness);
    }
}
/// Shared text-field defaults. Width follows its containing form; icons reserve
/// inset space without changing the field's outer size.
pub(crate) const INPUT_ICON_TEXT_INSET: f32 = 52.;

pub(crate) fn input_box(leading_icon: bool, trailing_icon: bool) -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        height: px(48),
        min_height: px(48),
        flex_shrink: 0.,
        padding: UiRect {
            left: px(if leading_icon {
                INPUT_ICON_TEXT_INSET
            } else {
                14.
            }),
            right: px(if trailing_icon { 48 } else { 14 }),
            top: px(10),
            bottom: px(10),
        },
        align_items: AlignItems::Center,
        overflow: Overflow::clip(),
        ..default()
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
#[cfg(test)]
pub(crate) fn controls(parent: &mut ChildSpawnerCommands) {
    vsync_control(parent);
    scale_control(parent);
}
pub(crate) fn checkbox_row() -> Node {
    Node {
        min_height: px(32),
        column_gap: px(12),
        align_items: AlignItems::Center,
        ..default()
    }
}
pub(crate) fn checkbox_box() -> Node {
    Node {
        width: px(22),
        height: px(22),
        flex_shrink: 0.,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}
pub(crate) fn vsync_control(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Vsync,
            Checkbox,
            TabIndex(0),
            Hovered::default(),
            AccessibleLabel::new("Vertical synchronisation"),
            checkbox_row(),
        ))
        .with_children(|row| {
            let check = row.target_entity();
            row.spawn((
                CheckArt(check),
                Sprite("checkbox.default"),
                Pickable::IGNORE,
                checkbox_box(),
            ))
            .with_child((
                Tick(check),
                Sprite("checkbox.tick"),
                Pickable::IGNORE,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(18),
                    height: px(18),
                    ..default()
                },
            ));
            row.spawn(label("Vertical synchronisation"));
        });
}
pub(crate) fn scale_control(parent: &mut ChildSpawnerCommands) {
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
                    height: px(12),
                    ..default()
                },
                Sprite("slider.track"),
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
                            width: px(20),
                            height: px(24),
                            ..default()
                        },
                        Sprite("slider.thumb"),
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
    sprites: Res<Sprites>,
    mut checks: Query<(&CheckArt, &mut ImageNode)>,
    widgets: Query<
        (Entity, Option<&Hovered>, Has<InteractionDisabled>),
        Or<(With<Checkbox>, With<Slider>)>,
    >,
) {
    for (entity, hover, disabled) in &widgets {
        let focused = focus.as_ref().is_some_and(|f| f.get() == Some(entity));
        for (check, mut image) in &mut checks {
            if check.0 == entity {
                let next = sprites.node(&format!(
                    "checkbox.{}",
                    state(disabled, false, false, focused, hover.is_some_and(|h| h.0))
                ));
                if image.image != next.image || image.rect != next.rect {
                    *image = next;
                }
            }
        }
        commands.entity(entity).remove::<Outline>();
    }
}

fn slider_focus(
    focus: Option<Res<InputFocus>>,
    sliders: Query<(Option<&Hovered>, Has<InteractionDisabled>), With<Slider>>,
    thumbs: Query<(&Thumb, &Children)>,
    mut images: Query<&mut ImageNode>,
) {
    for (thumb, children) in &thumbs {
        let Ok((hover, disabled)) = sliders.get(thumb.0) else {
            continue;
        };
        let highlighted =
            focus.as_ref().is_some_and(|f| f.get() == Some(thumb.0)) || hover.is_some_and(|h| h.0);
        let brightness = if disabled {
            0.35
        } else if highlighted {
            1.25
        } else {
            1.0
        };
        for child in children.iter() {
            if let Ok(mut image) = images.get_mut(child) {
                image.color = Color::srgb(brightness, brightness, brightness);
            }
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
    let kit = crate::fantasy_ui::kit::snapshot(world);
    serde_json::json!({"sprite_package":"sprites-v3","kit":kit,"frame_cap":world.get_resource::<crate::fantasy_ui::graphics::FrameCap>().map(|c|c.0),"controls":controls,"scale":world.get_resource::<crate::fantasy_ui::PreferredScale>().map(|v|v.0),"vsync":world.get_resource::<crate::fantasy_ui::graphics::Vsync>().map(|v|v.0)})
}

/// Gold serif heading and divider shared by frontend panels.
pub(crate) fn heading(parent: &mut ChildSpawnerCommands, caption: &str) {
    parent
        .spawn((
            Node {
                width: percent(100),
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(px(8), px(7)),
                margin: UiRect::bottom(px(6)),
                border: UiRect::bottom(px(1)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.65, 0.49, 0.23, 0.65)),
            Pickable::IGNORE,
        ))
        .with_child((
            Text::new(caption),
            TextFont::from_font_size(24.),
            TextColor(GOLD),
            Pickable::IGNORE,
        ));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sprite_states_preserve_disabled_ownership_and_roster_focus_without_square_outline() {
        let mut app = App::new();
        app.insert_resource(sprites::Sprites::fixture())
            .insert_resource(Art {
                standalone: default(),
                preview: default(),
            })
            .init_resource::<InputFocus>()
            .add_systems(Update, (decorate, present).chain());
        let button = app
            .world_mut()
            .spawn((
                Button,
                Style::Roster,
                Selected,
                InteractionDisabled,
                Hovered(true),
                Node::default(),
                BackgroundColor(PANEL),
            ))
            .with_child((Text::new("Arden"), TextColor(CREAM)))
            .id();
        let input = app
            .world_mut()
            .spawn((TextInput, Node::default(), BackgroundColor(PANEL)))
            .id();
        app.world_mut()
            .resource_mut::<InputFocus>()
            .set(button, bevy::input_focus::FocusCause::Navigated);
        app.update();
        let art = app
            .world_mut()
            .query::<(Entity, &ButtonArt)>()
            .iter(app.world())
            .find(|(_, a)| a.0 == button)
            .unwrap()
            .0;
        let expected = app
            .world()
            .resource::<sprites::Sprites>()
            .node("tab.chamfered.disabled");
        assert_eq!(
            app.world().get::<ImageNode>(art).unwrap().rect,
            expected.rect
        );
        assert!(app.world().get::<InteractionDisabled>(button).is_some());
        assert!(app.world().get::<Outline>(button).is_none());
        assert!(
            app.world().get::<ImageNode>(input).is_some(),
            "Input art stays on the text entity, beneath its glyphs"
        );
        assert!(
            app.world().get::<Decorated>(input).is_some(),
            "Childless text inputs receive the skin"
        );
        app.world_mut()
            .entity_mut(button)
            .remove::<(InteractionDisabled, Selected)>();
        app.update();
        let expected = app
            .world()
            .resource::<sprites::Sprites>()
            .node("tab.chamfered.focused");
        assert_eq!(
            app.world().get::<ImageNode>(art).unwrap().rect,
            expected.rect
        );
        assert!(app.world().get::<Outline>(button).is_none());
        assert_eq!(state(true, true, true, true, true), "disabled");
        assert_eq!(state(false, true, true, true, true), "pressed");
        assert_eq!(state(false, false, true, true, true), "selected");
    }
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
        assert!(app.world().get::<Sprite>(ornament).is_some());
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
