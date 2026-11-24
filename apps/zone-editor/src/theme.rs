//! Sprinkles UI adaptation, upstream 0e978f5 (MIT); see third_party/sprinkles.
//! Presentation only: all edits retain Theia's validated command/history path.
use super::*;
use bevy_sprinkles_editor::ui::widgets::{
    button::CompatButton as SprinklesButton, text_edit::EditorTextEdit as SprinklesTextInput,
};
#[path = "theme_checkbox.rs"]
pub(super) mod checkbox;
#[path = "theme_dialog.rs"]
pub(super) mod dialog;
#[path = "theme_slider.rs"]
pub(super) mod slider;
use bevy::asset::uuid_handle;
use bevy::color::palettes::tailwind;
use bevy_sprinkles_editor::ui::tokens;

pub(super) const FONT: Handle<Font> = uuid_handle!("581fa8f8-a50c-4a6b-9b7c-b18200da7483");
pub(super) const PRIMARY: Color = Color::Srgba(tokens::PRIMARY_COLOR);
pub(super) const PANEL: Color = Color::Srgba(tokens::BACKGROUND_COLOR);
pub(super) const BORDER: Color = Color::Srgba(tokens::BORDER_COLOR);
pub(super) const BODY: Color = Color::Srgba(tokens::TEXT_BODY_COLOR);
pub(super) const MUTED: Color = Color::Srgba(tokens::TEXT_MUTED_COLOR);
pub(super) const DISPLAY: Color = Color::Srgba(tokens::TEXT_DISPLAY_COLOR);

// Adapted from Sprinkles' ButtonVariant. Keeping the native Bevy Button and
// Checkbox behavior avoids replacing Theia's focus, activation or edit guards.
#[derive(Component, Clone, Copy)]
pub(super) enum Variant {
    Default,
    Primary,
    Destructive,
    Active,
    Ghost,
    Disabled,
}
impl Variant {
    pub fn background(self, hovered: bool) -> Color {
        let (color, alpha) = match (self, hovered) {
            (Self::Default, false) => (BORDER, 0.5),
            (Self::Default, true) => (BORDER, 0.8),
            (Self::Destructive, false) => (Color::Srgba(tailwind::RED_500), 1.),
            (Self::Destructive, true) => (Color::Srgba(tailwind::RED_600), 0.9),
            (Self::Primary, false) => (PRIMARY, 1.),
            (Self::Primary, true) => (PRIMARY, 0.9),
            (Self::Active, false) => (PRIMARY, 0.1),
            (Self::Active, true) => (PRIMARY, 0.15),
            (Self::Ghost, true) => (BODY, 0.05),
            (Self::Ghost | Self::Disabled, false) | (Self::Disabled, true) => (BODY, 0.),
        };
        color.with_alpha(alpha)
    }
    pub(super) fn foreground(self) -> Color {
        match self {
            Self::Primary | Self::Destructive => DISPLAY,
            Self::Active => PRIMARY.lighter(0.05),
            Self::Disabled => MUTED,
            _ => BODY,
        }
    }
}
pub(super) fn button_color(enabled: bool, active: bool, hovered: bool) -> Color {
    variant(enabled, active).background(hovered)
}
fn variant(enabled: bool, active: bool) -> Variant {
    if !enabled {
        Variant::Disabled
    } else if active {
        Variant::Active
    } else {
        Variant::Default
    }
}
pub(super) fn font(size: f32) -> TextFont {
    TextFont {
        font: FONT.clone().into(),
        font_size: bevy::text::FontSize::Px(size),
        weight: if size >= 14. {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        },
        ..default()
    }
}
pub(super) fn label(value: &str, size: f32, color: Color) -> impl Bundle {
    let mut font = font(size);
    font.weight = if size >= 18. {
        FontWeight::SEMIBOLD
    } else {
        FontWeight::NORMAL
    };
    (Text::new(value), font, TextColor(color))
}
pub(super) fn field_label(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            weight: FontWeight::MEDIUM,
            ..font(10.)
        },
        TextColor(MUTED),
        Pickable::IGNORE,
    )
}

pub(super) fn button_node() -> Node {
    Node {
        height: px(28),
        min_height: px(28),
        padding: UiRect::horizontal(px(12)),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(4)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_shrink: 0.,
        ..default()
    }
}
fn color_variant(color: Color, disabled: bool) -> Variant {
    if disabled {
        return Variant::Disabled;
    }
    for variant in [
        Variant::Destructive,
        Variant::Primary,
        Variant::Active,
        Variant::Ghost,
    ] {
        if color == variant.background(false) || color == variant.background(true) {
            return variant;
        }
    }
    Variant::Default
}
#[allow(clippy::type_complexity)]
fn button_borders(
    mut buttons: Query<
        (
            &BackgroundColor,
            &mut BorderColor,
            &mut Node,
            Option<&Variant>,
            Has<bevy::ui::InteractionDisabled>,
        ),
        (With<Button>, Without<SprinklesButton>),
    >,
) {
    for (background, mut border, mut node, variant, disabled) in &mut buttons {
        let variant = if disabled {
            Variant::Disabled
        } else {
            variant
                .copied()
                .unwrap_or_else(|| color_variant(background.0, false))
        };
        let color = match variant {
            Variant::Primary | Variant::Active => PRIMARY,
            Variant::Destructive => Color::Srgba(tailwind::RED_500),
            _ => BORDER,
        };
        let width = UiRect::all(px(if matches!(variant, Variant::Default) {
            1.
        } else {
            0.
        }));
        if node.border != width {
            node.border = width;
        }
        let next = BorderColor::all(color);
        if *border != next {
            *border = next;
        }
    }
}
pub(super) fn input_font() -> TextFont {
    let mut value = font(12.);
    value.font_features = [bevy::text::FontFeatureTag::TABULAR_FIGURES].into();
    value
}
pub(super) fn input_node() -> Node {
    Node {
        min_height: px(28),
        padding: UiRect::all(px(6)),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(2)),
        ..default()
    }
}
pub(super) fn text_cursor() -> bevy::text::TextCursorStyle {
    bevy::text::TextCursorStyle {
        color: BODY,
        selection_color: PRIMARY.with_alpha(0.35),
        unfocused_selection_color: Color::NONE,
        selected_text_color: None,
        ..default()
    }
}

pub(super) fn load_font(mut fonts: ResMut<Assets<Font>>) {
    fonts
        .insert(
            FONT.id(),
            Font::from_bytes(include_bytes!("../assets/fonts/InterVariable.ttf").to_vec()),
        )
        .expect("embedded Inter font has a stable asset ID");
}
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_font).add_systems(
        PostUpdate,
        (
            release_buttons,
            decorate_buttons,
            simple_buttons,
            button_borders,
            text_states,
            input_states,
            numeric_frames,
            checkbox::sync,
            slider::sync,
        )
            .chain(),
    );
}
// Bevy 0.20 sends release to the previously hovered entity. Activating a tool
// can reflow the inspector before release, leaving the original button Pressed.
// Clear residual state after native click observers run, once all pointers are
// physically up (or focus is lost). This never emits an activation.
#[allow(clippy::type_complexity)]
fn release_buttons(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<bevy::input::touch::Touches>,
    windows: Query<&Window>,
    buttons: Query<
        Entity,
        (
            With<Button>,
            With<bevy::ui::Pressed>,
            Without<SprinklesButton>,
        ),
    >,
) {
    if windows.iter().any(|window| window.focused)
        && (mouse.get_pressed().next().is_some() || touches.iter().next().is_some())
    {
        return;
    }
    for entity in &buttons {
        commands.entity(entity).remove::<bevy::ui::Pressed>();
    }
}

type NewButton = Or<(Added<Button>, Added<bevy::ui_widgets::Checkbox>)>;
#[allow(clippy::type_complexity)]
fn decorate_buttons(
    mut commands: Commands,
    mut buttons: Query<
        (Entity, &mut Node, Option<&Variant>),
        (NewButton, Without<checkbox::Root>, Without<SprinklesButton>),
    >,
) {
    for (entity, mut node, variant) in &mut buttons {
        node.border = UiRect::all(px(if matches!(variant, Some(Variant::Ghost)) {
            0
        } else {
            1
        }));
        node.align_items = AlignItems::Center;
        node.border_radius = BorderRadius::all(px(4));
        commands.entity(entity).insert(BorderColor::all(BORDER));
    }
}
#[allow(clippy::type_complexity)]
fn simple_buttons(
    mut buttons: Query<
        (
            &Hovered,
            &mut BackgroundColor,
            Option<&properties::Control>,
            Option<&naming::Control>,
            Option<&Variant>,
        ),
        (
            With<Button>,
            Without<workspace::Control>,
            Without<outliner::Control>,
            Without<file_picker::Control>,
            Without<Action>,
            Without<SprinklesButton>,
        ),
    >,
) {
    for (hovered, mut background, property, name, variant) in &mut buttons {
        let primary = matches!(property, Some(properties::Control::Apply))
            || matches!(name, Some(naming::Control::Apply));
        background.0 = if let Some(variant) = variant {
            *variant
        } else if primary {
            Variant::Primary
        } else {
            Variant::Default
        }
        .background(hovered.0);
    }
}
#[derive(Component)]
pub(super) struct Secondary;

#[allow(clippy::type_complexity)]
fn text_states(
    buttons: Query<
        (&BackgroundColor, Has<bevy::ui::InteractionDisabled>),
        (
            Or<(
                With<Button>,
                With<bevy::ui_widgets::Checkbox>,
                With<bevy::ui_widgets::MenuItem>,
            )>,
            Without<SprinklesButton>,
        ),
    >,
    parents: Query<&ChildOf>,
    mut texts: Query<
        (Entity, &mut TextColor, Has<Secondary>),
        (With<Text>, Without<sections::Title>),
    >,
) {
    for (entity, mut color, secondary) in &mut texts {
        for ancestor in parents.iter_ancestors(entity) {
            if let Ok((background, disabled)) = buttons.get(ancestor) {
                let state = color_variant(background.0, disabled);
                color.0 = if secondary { MUTED } else { state.foreground() };
                break;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn input_states(
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    mut fields: Query<
        (
            Entity,
            &mut Node,
            &mut BorderColor,
            Has<properties::FramedInput>,
            &mut BackgroundColor,
        ),
        (
            With<bevy::ui_widgets::TextInput>,
            Without<SprinklesTextInput>,
        ),
    >,
) {
    for (entity, mut node, mut border, framed, mut background) in &mut fields {
        if framed {
            continue;
        }
        if node.border != UiRect::all(px(1))
            || node.border_radius != BorderRadius::all(px(2))
            || node.padding != UiRect::all(px(6))
            || node.min_height != px(28)
        {
            node.border = UiRect::all(px(1));
            node.border_radius = BorderRadius::all(px(2));
            node.padding = UiRect::all(px(6));
            node.min_height = px(28);
        }
        if background.0 != Color::NONE {
            background.0 = Color::NONE;
        }
        *border = BorderColor::all(if focus.as_ref().is_some_and(|f| f.get() == Some(entity)) {
            PRIMARY
        } else {
            BORDER
        });
    }
}

#[allow(clippy::type_complexity)]
fn numeric_frames(
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    mut frames: Query<
        (&Children, &mut BorderColor),
        Or<(With<properties::FieldRow>, With<properties::InlineFrame>)>,
    >,
) {
    let active = focus.as_ref().and_then(|focus| focus.get());
    for (children, mut border) in &mut frames {
        let color = if active.is_some_and(|entity| children.contains(&entity)) {
            PRIMARY
        } else {
            BORDER
        };
        if *border != BorderColor::all(color) {
            *border = BorderColor::all(color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::picking::{
        backend::HitData,
        events::{Pointer, PointerPress, PointerRelease},
        pointer::{Location, PointerButton, PointerId},
    };
    #[derive(Resource, Default)]
    struct Activations(u32);

    #[test]
    fn native_theia_styling_leaves_reused_sprinkles_widgets_untouched() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                decorate_buttons,
                simple_buttons,
                button_borders,
                text_states,
                input_states,
            )
                .chain(),
        );
        let original_color = Color::srgb(0.2, 0.4, 0.6);
        let original_border = BorderColor::all(original_color);
        let original_node = Node {
            border_radius: BorderRadius::all(px(9)),
            ..default()
        };
        let source_button = app
            .world_mut()
            .spawn((
                SprinklesButton,
                Hovered::default(),
                original_node.clone(),
                BackgroundColor(original_color),
                original_border,
            ))
            .id();
        let source_text = app
            .world_mut()
            .spawn((
                Text::new("Original"),
                TextColor(original_color),
                ChildOf(source_button),
            ))
            .id();
        let source_input = app
            .world_mut()
            .spawn((
                SprinklesTextInput,
                bevy::ui_widgets::TextInput,
                bevy::text::EditableText::new("1.5"),
                Node::default(),
                BackgroundColor(original_color),
                original_border,
            ))
            .id();
        let zone_button = app
            .world_mut()
            .spawn((
                Button,
                Node::default(),
                Hovered::default(),
                BackgroundColor(Color::BLACK),
            ))
            .id();
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            app.world().get::<Node>(source_button).unwrap(),
            &original_node
        );
        assert_eq!(
            app.world().get::<BackgroundColor>(source_button).unwrap().0,
            original_color
        );
        assert_eq!(
            app.world().get::<BorderColor>(source_button).unwrap(),
            &original_border
        );
        assert_eq!(
            app.world().get::<TextColor>(source_text).unwrap().0,
            original_color
        );
        assert_eq!(
            app.world().get::<Node>(source_input).unwrap().padding,
            UiRect::ZERO
        );
        assert_eq!(
            app.world().get::<BorderColor>(source_input).unwrap(),
            &original_border
        );
        assert_eq!(
            app.world().get::<BackgroundColor>(source_input).unwrap().0,
            original_color
        );
        assert_eq!(
            app.world().get::<Node>(zone_button).unwrap().border,
            UiRect::all(px(1))
        );
        assert_eq!(
            app.world().get::<BackgroundColor>(zone_button).unwrap().0,
            Variant::Default.background(false)
        );
    }
    #[test]
    fn reflowed_buttons_release_and_can_activate_again_without_replaying_commands() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<bevy::input::touch::Touches>()
            .init_resource::<Activations>()
            .add_plugins(bevy::ui_widgets::ButtonPlugin)
            .add_systems(PostUpdate, release_buttons)
            .add_observer(|_: On<Activate>, mut count: ResMut<Activations>| count.0 += 1);
        let window = app
            .world_mut()
            .spawn(Window {
                focused: true,
                ..default()
            })
            .id();
        let original = app
            .world_mut()
            .spawn((Button, ActivateOnPress, Node::default()))
            .id();
        let under_release = app.world_mut().spawn((Button, Node::default())).id();
        let pointer = Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::new(500., 350.),
            },
        );
        for expected in [1, 2] {
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.world_mut().trigger(PointerPress {
                entity: original,
                pointer: pointer.clone(),
                button: PointerButton::Primary,
                count: 1,
                hit: HitData::new(window, 0., None, None),
            });
            app.update();
            assert!(app.world().get::<bevy::ui::Pressed>(original).is_some());
            assert_eq!(app.world().resource::<Activations>().0, expected);
            app.world_mut().trigger(PointerRelease {
                entity: under_release,
                pointer: pointer.clone(),
                button: PointerButton::Primary,
                hit: HitData::new(window, 0., None, None),
            });
            assert!(app.world().get::<bevy::ui::Pressed>(original).is_some());
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            app.update();
            assert!(app.world().get::<bevy::ui::Pressed>(original).is_none());
            assert_eq!(app.world().resource::<Activations>().0, expected);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.world_mut()
            .entity_mut(original)
            .insert(bevy::ui::Pressed);
        app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
        app.update();
        assert!(app.world().get::<bevy::ui::Pressed>(original).is_none());
        assert_eq!(app.world().resource::<Activations>().0, 2);
    }
}
