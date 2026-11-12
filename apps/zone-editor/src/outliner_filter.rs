//! Compact content-type selection using the native menu lifecycle and Sprinkles field style.
use super::*;
use bevy::input_focus::{
    FocusCause, InputFocus,
    tab_navigation::{NavAction, TabIndex},
};
use bevy::ui::OverrideClip;
use bevy::ui_widgets::popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide};
use bevy::ui_widgets::{MenuAction, MenuButton, MenuEvent, MenuFocusState, MenuItem, MenuPopup};

pub(super) fn spawn(parent: &mut ChildSpawnerCommands) {
    let mut owner = parent.spawn(Node {
        width: percent(100),
        min_width: px(0),
        flex_shrink: 0.,
        ..default()
    });
    let anchor = owner.id();
    let mut button = Entity::PLACEHOLDER;
    let mut popup = Entity::PLACEHOLDER;
    owner.with_children(|root| {
        button = root
            .spawn((
                Button,
                MenuButton,
                menus::Trigger("Content filter"),
                Hovered::default(),
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(6),
                    ..theme::button_node()
                },
                BackgroundColor(theme::Variant::Default.background(false)),
            ))
            .with_children(|button| {
                button.spawn((
                    Label::FilterName,
                    theme::label("All types", 12., theme::BODY),
                    Pickable::IGNORE,
                ));
                button.spawn((
                    Pickable::IGNORE,
                    Node {
                        width: px(6),
                        height: px(6),
                        margin: UiRect::bottom(px(3)),
                        border: UiRect {
                            right: px(1),
                            bottom: px(1),
                            ..default()
                        },
                        ..default()
                    },
                    BorderColor::all(theme::MUTED),
                    UiTransform::from_rotation(Rot2::degrees(45.)),
                ));
            })
            .id();
        popup = root
            .spawn((
                menus::Popup("Content filter"),
                MenuPopup::default(),
                OverrideClip,
                Hovered::default(),
                GlobalZIndex(18),
                Popover {
                    window_margin: 8.,
                    positions: [PopoverSide::Bottom, PopoverSide::Top]
                        .map(|side| PopoverPlacement {
                            side,
                            align: PopoverAlign::Start,
                            gap: 4.,
                        })
                        .into(),
                },
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    padding: px(4).all(),
                    flex_direction: FlexDirection::Column,
                    border: px(1).all(),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER),
            ))
            .with_children(|menu| {
                for filter in [
                    Filter::All,
                    Filter::Objects,
                    Filter::Guides,
                    Filter::Markers,
                    Filter::Plants,
                    Filter::Particles,
                    Filter::QuestAreas,
                    Filter::Scenery,
                    Filter::Terrain,
                ] {
                    menu.spawn((
                        Control::Filter(filter),
                        MenuItem,
                        Hovered::default(),
                        TabIndex(0),
                        Node {
                            width: percent(100),
                            min_height: px(28),
                            padding: UiRect::axes(px(8), px(5)),
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(px(2)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_child((
                        theme::label(filter.caption(), 12., theme::BODY),
                        Pickable::IGNORE,
                    ));
                }
            })
            .id();
    });
    parent
        .commands()
        .entity(anchor)
        .observe(
            move |event: On<MenuEvent>,
                  session: Res<Session>,
                  windows: Query<&Window>,
                  mut popups: Query<(&mut Node, &mut MenuFocusState), With<MenuPopup>>,
                  mut focus: ResMut<InputFocus>| {
                let Ok((mut node, mut state)) = popups.get_mut(popup) else {
                    return;
                };
                let allowed = !busy(&session) && windows.single().is_ok_and(|w| w.focused);
                match event.action {
                    MenuAction::Open(direction) if allowed => {
                        node.display = Display::Flex;
                        *state = MenuFocusState::Opening(direction);
                    }
                    MenuAction::Toggle if allowed => {
                        if node.display == Display::None {
                            node.display = Display::Flex;
                            *state = MenuFocusState::Opening(NavAction::First);
                        } else {
                            node.display = Display::None;
                            *state = MenuFocusState::Closed;
                        }
                    }
                    MenuAction::CloseAll => {
                        node.display = Display::None;
                        *state = MenuFocusState::Closed;
                    }
                    MenuAction::FocusRoot => focus.set(button, FocusCause::Navigated),
                    _ => {}
                }
            },
        )
        .observe(
            |event: On<Activate>,
             controls: Query<&Control, With<MenuItem>>,
             session: Res<Session>,
             windows: Query<&Window>,
             mut content: ResMut<Content>,
             mut commands: Commands| {
                let Ok(Control::Filter(filter)) = controls.get(event.entity) else {
                    return;
                };
                if busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
                    return;
                }
                content.filter = *filter;
                content.page = 0;
                commands.trigger(MenuEvent {
                    source: event.entity,
                    action: MenuAction::CloseAll,
                });
            },
        );
}
