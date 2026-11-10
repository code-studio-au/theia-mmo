//! Sprinkles UI adaptation, upstream 0e978f5 (MIT); see third_party/sprinkles.
//! Presentation only: all edits retain Theia's validated command/history path.
use super::*;
#[path = "theme_checkbox.rs"]
pub(super) mod checkbox;
#[path = "theme_dialog.rs"]
pub(super) mod dialog;
use bevy::asset::uuid_handle;
use bevy::color::palettes::tailwind;

pub(super) const FONT: Handle<Font> = uuid_handle!("581fa8f8-a50c-4a6b-9b7c-b18200da7483");
pub(super) const PRIMARY: Color = Color::Srgba(tailwind::BLUE_500);
pub(super) const PANEL: Color = Color::Srgba(tailwind::ZINC_800);
pub(super) const BORDER: Color = Color::Srgba(tailwind::ZINC_700);
pub(super) const BODY: Color = Color::Srgba(tailwind::ZINC_200);
pub(super) const MUTED: Color = Color::Srgba(tailwind::ZINC_400);
pub(super) const DISPLAY: Color = Color::Srgba(tailwind::ZINC_50);
pub(super) const INPUT: Color = Color::Srgba(tailwind::ZINC_900);

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
    fn foreground(self) -> Color {
        match self {
            Self::Primary | Self::Destructive => DISPLAY,
            Self::Active => PRIMARY,
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
            text_states,
            input_states,
            numeric_frames,
            checkbox::sync,
        )
            .chain(),
    );
}
// Bevy 0.20 sends release to the previously hovered entity. Activating a tool
// can reflow the inspector before release, leaving the original button Pressed.
// Clear residual state after native click observers run, once all pointers are
// physically up (or focus is lost). This never emits an activation.
fn release_buttons(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<bevy::input::touch::Touches>,
    windows: Query<&Window>,
    buttons: Query<Entity, (With<Button>, With<bevy::ui::Pressed>)>,
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
    mut buttons: Query<(Entity, &mut Node, Option<&Variant>), (NewButton, Without<checkbox::Root>)>,
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
#[allow(clippy::type_complexity)]
fn text_states(
    buttons: Query<
        (&BackgroundColor, Has<bevy::ui::InteractionDisabled>),
        Or<(With<Button>, With<bevy::ui_widgets::Checkbox>)>,
    >,
    parents: Query<&ChildOf>,
    mut texts: Query<(Entity, &mut TextColor), With<Text>>,
) {
    for (entity, mut color) in &mut texts {
        for ancestor in parents.iter_ancestors(entity) {
            if let Ok((background, disabled)) = buttons.get(ancestor) {
                let state = if disabled {
                    Variant::Disabled
                } else if background.0 == Variant::Destructive.background(false)
                    || background.0 == Variant::Destructive.background(true)
                {
                    Variant::Destructive
                } else if background.0 == Variant::Primary.background(false)
                    || background.0 == Variant::Primary.background(true)
                {
                    Variant::Primary
                } else if background.0 == Variant::Active.background(false)
                    || background.0 == Variant::Active.background(true)
                {
                    Variant::Active
                } else {
                    Variant::Default
                };
                color.0 = state.foreground();
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
        With<bevy::ui_widgets::TextInput>,
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

fn numeric_frames(
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    mut frames: Query<(&Children, &mut BorderColor), With<properties::FieldRow>>,
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
