//! A docked explorer over properties, with independent folding and a shared-width split.
use super::*;

const GRIP: f32 = 10.;
pub(super) const TITLE_HEIGHT: f32 = 32.;
const DIVIDER: f32 = 1.;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Width,
    Split,
}
#[derive(Clone, Copy)]
struct Resize {
    edge: Edge,
    start: Vec2,
    original: [f32; 2],
}
pub(super) struct Layout {
    pub width: f32,
    pub explorer_fraction: f32,
    pub explorer_collapsed: bool,
    pub properties_collapsed: bool,
    drag: Option<Resize>,
    pub input_owned: bool,
    pub overlay_owned: bool,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            width: workspace::INSPECTOR_WIDTH,
            explorer_fraction: 0.4,
            explorer_collapsed: false,
            properties_collapsed: false,
            drag: None,
            input_owned: false,
            overlay_owned: false,
        }
    }
}
impl Layout {
    pub fn canvas_left(&self) -> f32 {
        sidebar::WIDTH + self.width
    }
    pub fn explorer_height(&self, window_height: f32) -> f32 {
        let available = (window_height - workspace::HEADER - status_bar::HEIGHT - DIVIDER)
            .max(2. * TITLE_HEIGHT);
        if self.explorer_collapsed {
            TITLE_HEIGHT
        } else if self.properties_collapsed {
            available - TITLE_HEIGHT
        } else {
            let minimum = 96_f32.min(available / 2.);
            (available * self.explorer_fraction)
                .clamp(minimum, available - minimum)
                .round()
        }
    }
    pub fn properties_top(&self, height: f32) -> f32 {
        workspace::HEADER + self.explorer_height(height) + DIVIDER
    }
    pub fn window_to_viewport(&self, point: Vec2, scale: f32) -> Vec2 {
        (point - Vec2::new(self.canvas_left(), workspace::HEADER)) * scale
    }
    pub fn viewport_to_window(&self, point: Vec2, scale: f32) -> Vec2 {
        point / scale + Vec2::new(self.canvas_left(), workspace::HEADER)
    }
    fn collapsed(&self, region: Region) -> bool {
        match region {
            Region::Browser => self.explorer_collapsed,
            Region::Inspector => self.properties_collapsed,
            _ => false,
        }
    }
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Region {
    Browser,
    Inspector,
    Footer,
    Canvas,
}
#[derive(Component)]
pub(super) struct Toggle(pub(super) Region);
#[derive(Component)]
pub(super) struct Grip(Edge);
#[derive(Component)]
pub(super) struct Body(Region);
#[derive(Component)]
pub(super) struct Chevron(Region);

pub(super) fn header(parent: &mut ChildSpawnerCommands, region: Region, title: &str) -> Entity {
    let mut caption = Entity::PLACEHOLDER;
    parent
        .spawn((
            Toggle(region),
            Button,
            ActivateOnPress,
            Hovered::default(),
            theme::Variant::Ghost,
            Node {
                width: percent(100),
                height: px(TITLE_HEIGHT),
                min_height: px(TITLE_HEIGHT),
                flex_shrink: 0.,
                padding: UiRect::horizontal(px(12)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|header| {
            caption = header
                .spawn((theme::label(title, 12., theme::DISPLAY), Pickable::IGNORE))
                .id();
            header.spawn((
                Chevron(region),
                Pickable::IGNORE,
                Node {
                    width: px(6),
                    height: px(6),
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
        .observe(
            |event: On<Activate>,
             mut session: ResMut<Session>,
             windows: Query<&Window>,
             toggles: Query<&Toggle>| {
                if !windows.single().is_ok_and(|w| w.focused) || workspace::busy(&session) {
                    return;
                }
                let Ok(toggle) = toggles.get(event.entity) else {
                    return;
                };
                match toggle.0 {
                    Region::Browser => {
                        session.ui.explorer_collapsed = !session.ui.explorer_collapsed
                    }
                    Region::Inspector => {
                        session.ui.properties_collapsed = !session.ui.properties_collapsed
                    }
                    _ => {}
                }
            },
        );
    caption
}
pub(super) fn body(
    parent: &mut ChildSpawnerCommands,
    region: Region,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Body(region),
            Visibility::Inherited,
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.,
                flex_basis: px(0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(content);
}
pub(super) fn setup(mut commands: Commands) {
    for edge in [Edge::Width, Edge::Split] {
        commands
            .spawn((
                Grip(edge),
                GlobalZIndex(16),
                Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_child((
                Pickable::IGNORE,
                Node {
                    width: if edge == Edge::Width {
                        px(1)
                    } else {
                        percent(100)
                    },
                    height: if edge == Edge::Split {
                        px(1)
                    } else {
                        percent(100)
                    },
                    ..default()
                },
                BackgroundColor(theme::BORDER),
            ));
    }
}
fn can_resize(window: &Window, session: &Session, menu_open: bool) -> bool {
    window.focused
        && !menu_open
        && !session.has_modal()
        && session.play.is_none()
        && session.drag.is_none()
        && session.stroke.is_none()
        && session.path_draft.is_none()
}
pub(super) fn resize(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    scrollbars: Query<&Hovered, With<bevy::ui_widgets::Scrollbar>>,
    menus: Query<&bevy::ui_widgets::MenuFocusState, With<bevy::ui_widgets::MenuPopup>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let allowed = can_resize(
        window,
        &session,
        menus
            .iter()
            .any(|s| !matches!(s, bevy::ui_widgets::MenuFocusState::Closed)),
    );
    let cursor = window.cursor_position();
    let layout = &mut session.ui;
    layout.input_owned = false;
    if let Some(drag) = layout.drag {
        layout.input_owned = true;
        if !allowed || cursor.is_none() || keys.just_pressed(KeyCode::Escape) {
            [layout.width, layout.explorer_fraction] = drag.original;
            layout.drag = None;
            return;
        }
        let delta = cursor.unwrap() - drag.start;
        match drag.edge {
            Edge::Width => layout.width = (drag.original[0] + delta.x).clamp(300., 720.),
            Edge::Split => {
                layout.explorer_fraction = (drag.original[1]
                    + delta.y
                        / (window.height() - workspace::HEADER - status_bar::HEIGHT - DIVIDER)
                            .max(1.))
                .clamp(0.05, 0.95)
            }
        }
        if !mouse.pressed(MouseButton::Left) {
            layout.drag = None;
        }
    } else if allowed && mouse.just_pressed(MouseButton::Left) && !scrollbars.iter().any(|h| h.0) {
        let Some(cursor) = cursor else {
            return;
        };
        if cursor.y <= workspace::HEADER || cursor.y >= window.height() - status_bar::HEIGHT {
            return;
        }
        let edge = if (cursor.x - layout.canvas_left()).abs() <= GRIP / 2. {
            Some(Edge::Width)
        } else if cursor.x >= sidebar::WIDTH
            && cursor.x < layout.canvas_left()
            && !layout.explorer_collapsed
            && !layout.properties_collapsed
            && (cursor.y - layout.properties_top(window.height())).abs() <= GRIP / 2.
        {
            Some(Edge::Split)
        } else {
            None
        };
        if let Some(edge) = edge {
            layout.drag = Some(Resize {
                edge,
                start: cursor,
                original: [layout.width, layout.explorer_fraction],
            });
            layout.input_owned = true;
        }
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn apply(
    mut commands: Commands,
    windows: Query<(Entity, &Window, Option<&bevy::window::CursorIcon>)>,
    session: Res<Session>,
    mut regions: Query<(&Region, &mut Node), (Without<Grip>, Without<Body>)>,
    mut grips: Query<
        (&Grip, &Hovered, &mut Node, &mut BackgroundColor),
        (Without<Region>, Without<Body>),
    >,
    mut bodies: Query<(&Body, &mut Node, &mut Visibility), (Without<Region>, Without<Grip>)>,
    mut chevrons: Query<(&Chevron, &mut UiTransform)>,
    menus: Query<&bevy::ui_widgets::MenuFocusState, With<bevy::ui_widgets::MenuPopup>>,
    scrollbars: Query<&Hovered, With<bevy::ui_widgets::Scrollbar>>,
) {
    let Ok((window_entity, window, cursor)) = windows.single() else {
        return;
    };
    let menu_open = menus
        .iter()
        .any(|s| !matches!(s, bevy::ui_widgets::MenuFocusState::Closed));
    let allowed = can_resize(window, &session, menu_open);
    let over_scrollbar = scrollbars.iter().any(|h| h.0);
    let explorer_height = session.ui.explorer_height(window.height());
    for (region, mut node) in &mut regions {
        match region {
            Region::Browser => {
                node.left = px(sidebar::WIDTH);
                node.width = px(session.ui.width);
                node.top = px(workspace::HEADER);
                node.bottom = Val::Auto;
                node.height = px(explorer_height);
            }
            Region::Inspector => {
                node.left = px(sidebar::WIDTH);
                node.width = px(session.ui.width);
                node.top = px(session.ui.properties_top(window.height()));
                node.bottom = if session.ui.properties_collapsed {
                    Val::Auto
                } else {
                    px(status_bar::HEIGHT)
                };
                node.height = if session.ui.properties_collapsed {
                    px(TITLE_HEIGHT)
                } else {
                    Val::Auto
                };
            }
            Region::Footer => node.left = px(0),
            Region::Canvas => node.left = px(session.ui.canvas_left()),
        }
    }
    for (body, mut node, mut visibility) in &mut bodies {
        *visibility = if session.ui.collapsed(body.0) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        node.display = if session.ui.collapsed(body.0) {
            Display::None
        } else {
            Display::Flex
        };
    }
    for (chevron, mut pose) in &mut chevrons {
        *pose = UiTransform::from_rotation(Rot2::degrees(if session.ui.collapsed(chevron.0) {
            -45.
        } else {
            45.
        }));
    }
    let mut hovered_edge = None;
    for (grip, hovered, mut node, mut background) in &mut grips {
        let visible = session.play.is_none()
            && (grip.0 == Edge::Width
                || (!session.ui.explorer_collapsed && !session.ui.properties_collapsed));
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if allowed && visible && hovered.0 && !over_scrollbar {
            hovered_edge = Some(grip.0);
        }
        match grip.0 {
            Edge::Width => {
                node.left = px(session.ui.canvas_left() - GRIP / 2.);
                node.top = px(workspace::HEADER);
                node.bottom = px(status_bar::HEIGHT);
                node.width = px(GRIP);
                node.height = Val::Auto;
            }
            Edge::Split => {
                node.left = px(sidebar::WIDTH);
                node.top = px(workspace::HEADER + explorer_height - GRIP / 2.);
                node.bottom = Val::Auto;
                node.width = px(session.ui.width);
                node.height = px(GRIP);
            }
        }
        background.0 = if allowed && session.ui.drag.is_some_and(|d| d.edge == grip.0) {
            theme::PRIMARY.with_alpha(0.2)
        } else if hovered_edge == Some(grip.0) {
            theme::BORDER.with_alpha(0.4)
        } else {
            Color::NONE
        };
    }
    let active = allowed
        .then(|| session.ui.drag.map(|d| d.edge).or(hovered_edge))
        .flatten();
    let next = bevy::window::CursorIcon::System(match active {
        Some(Edge::Width) => bevy::window::SystemCursorIcon::ColResize,
        Some(Edge::Split) => bevy::window::SystemCursorIcon::RowResize,
        None => bevy::window::SystemCursorIcon::Default,
    });
    let owned_cursor = matches!(
        cursor,
        Some(bevy::window::CursorIcon::System(
            bevy::window::SystemCursorIcon::ColResize | bevy::window::SystemCursorIcon::RowResize
        ))
    );
    if (active.is_some() || owned_cursor) && cursor != Some(&next) {
        commands.entity(window_entity).insert(next);
    }
}

#[cfg(test)]
#[path = "panels_tests.rs"]
mod tests;
