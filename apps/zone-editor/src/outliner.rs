//! A fixed row pool for browsing bounded zone content with standard Bevy UI.
use super::*;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::ui::InteractionDisabled;
pub(super) const WIDTH: f32 = 300.;
// Leave room for the title, paging and a two-line footer above Properties at
// the supported 3440 x 1440 authoring size. Every pooled row must be clickable.
const ROWS: usize = 18;
#[cfg(test)]
#[path = "outliner_scroll_tests.rs"]
mod scroll_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Terrain,
    Spawn,
    Object(u64),
    Guide(u64),
    Marker(u64),
    Plant(u64),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Filter {
    #[default]
    All,
    Objects,
    Guides,
    Markers,
    Plants,
    Terrain,
}
impl Filter {
    fn accepts(self, target: Target) -> bool {
        matches!(
            (self, target),
            (Self::All, _)
                | (Self::Objects, Target::Object(_))
                | (Self::Guides, Target::Guide(_))
                | (Self::Markers, Target::Marker(_))
                | (Self::Plants, Target::Plant(_))
                | (Self::Terrain, Target::Terrain)
        )
    }
}
#[derive(Clone)]
pub(super) struct Entry {
    pub target: Target,
    pub label: String,
}
#[derive(Resource, Default)]
pub(super) struct Content {
    pub entries: Vec<Entry>,
    pub page: usize,
    pub filter: Filter,
    pub total: usize,
    pub query: String,
    applied_page: Option<usize>,
    applied_filter: Option<Filter>,
    applied_query: String,
    revision: Option<u64>,
    selection: Option<Target>,
    selection_index: Option<usize>,
    pub scroll_events: u64,
    pub last_scroll: Option<serde_json::Value>,
}
impl Content {
    pub fn set_query(&mut self, query: &str) -> Result<(), String> {
        if query.len() > 128 || query.chars().any(char::is_control) {
            return Err("Search needs at most 128 UTF-8 bytes without control characters".into());
        }
        self.query = query.trim().into();
        self.page = 0;
        Ok(())
    }
    pub fn pages(&self) -> usize {
        self.entries.len().div_ceil(ROWS).max(1)
    }
    pub fn row(&self, index: usize) -> Option<&Entry> {
        self.entries.get(self.page * ROWS + index)
    }
}
#[derive(Component)]
pub(super) struct Root;
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Control {
    Previous,
    Next,
    Row(usize),
    Filter(Filter),
    Search,
    ClearSearch,
}
#[derive(Component)]
pub(super) enum Label {
    Title,
    Page,
    Footer,
    Row(usize),
    Search,
}

fn entries(zone: &Zone) -> Vec<Entry> {
    let mut rows = vec![];
    if let Some(patch) = &zone.terrain {
        rows.push(Entry {
            target: Target::Terrain,
            label: format!("Terrain | {} m samples", patch.sample_metres()),
        });
    }
    rows.push(Entry {
        target: Target::Spawn,
        label: "Frame playtest spawn".into(),
    });
    rows.extend(zone.objects.iter().map(|o| Entry {
        target: Target::Object(o.id),
        label: format!("{:?} #{} | {}", o.primitive, o.id, o.name),
    }));
    rows.extend(zone.paths.iter().map(|p| Entry {
        target: Target::Guide(p.id),
        label: format!(
            "{:?} #{} | {} ({} points)",
            p.kind,
            p.id,
            p.name,
            p.points.len()
        ),
    }));
    rows.extend(zone.markers.iter().map(|m| Entry {
        target: Target::Marker(m.id),
        label: format!("{:?} #{} | {}", m.kind, m.id, m.name),
    }));
    rows.extend(zone.plants.iter().map(|p| Entry {
        target: Target::Plant(p.id),
        label: format!("{:?} #{} | {}", p.kind, p.id, p.name),
    }));
    rows
}
pub(super) fn selection(session: &Session) -> Option<Target> {
    if session.play.is_some() {
        None
    } else if session.terrain_mode {
        session
            .editor
            .zone()
            .terrain
            .as_ref()
            .map(|_| Target::Terrain)
    } else if session.path_mode {
        session.path_selected.map(Target::Guide)
    } else if session.marker_mode {
        session.marker_selected.map(Target::Marker)
    } else if session.vegetation_mode {
        session.plant_selected.map(Target::Plant)
    } else {
        session.selected.map(Target::Object)
    }
}
pub(super) fn setup(mut commands: Commands) {
    commands.insert_resource(Content::default());
    commands
        .spawn((
            Root,
            panels::Region::Browser,
            GlobalZIndex(3),
            Node {
                position_type: PositionType::Absolute,
                top: px(workspace::HEADER),
                left: px(sidebar::WIDTH),
                bottom: px(52),
                width: px(WIDTH),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme::PANEL),
        ))
        .with_children(|root| {
            scrolling::pane(root, 12., scrolling::Pane::Browser, |panel| {
                panel.spawn((
                    Label::Title,
                    Text::new("Zone content"),
                    TextFont {
                        weight: FontWeight::SEMIBOLD,
                        ..theme::font(12.)
                    },
                    TextColor(theme::DISPLAY),
                ));
                panel
                    .spawn(Node {
                        column_gap: px(6),
                        ..default()
                    })
                    .with_children(|row| {
                        for (control, caption, width) in [
                            (Control::Search, "Find...", 158),
                            (Control::ClearSearch, "Clear search", 100),
                        ] {
                            row.spawn((
                                control,
                                Button,
                                ActivateOnPress,
                                Hovered::default(),
                                Node {
                                    width: px(width),
                                    overflow: Overflow::clip(),
                                    ..theme::button_node()
                                },
                                BackgroundColor(theme::Variant::Default.background(false)),
                            ))
                            .with_children(|button| {
                                if matches!(control, Control::Search) {
                                    button.spawn((
                                        Label::Search,
                                        theme::label(caption, 12., theme::BODY),
                                        TextLayout::no_wrap(),
                                    ));
                                } else {
                                    button.spawn((
                                        theme::label(caption, 12., theme::BODY),
                                        TextLayout::no_wrap(),
                                    ));
                                }
                            });
                        }
                    });
                panel
                    .spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(6),
                        row_gap: px(6),
                        ..default()
                    })
                    .with_children(|filters| {
                        for (filter, caption) in [
                            (Filter::All, "All"),
                            (Filter::Objects, "Objects"),
                            (Filter::Guides, "Guides"),
                            (Filter::Markers, "Markers"),
                            (Filter::Plants, "Plants"),
                            (Filter::Terrain, "Terrain"),
                        ] {
                            filters.spawn((
                                Control::Filter(filter),
                                Button,
                                ActivateOnPress,
                                Hovered::default(),
                                Node {
                                    width: px(86),
                                    ..theme::button_node()
                                },
                                BackgroundColor(theme::Variant::Default.background(false)),
                                children![theme::label(caption, 12., theme::BODY)],
                            ));
                        }
                    });
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|bar| {
                        for (action, caption) in
                            [(Control::Previous, "Previous"), (Control::Next, "Next")]
                        {
                            bar.spawn((
                                action,
                                Button,
                                ActivateOnPress,
                                Hovered::default(),
                                Node {
                                    width: px(80),
                                    ..theme::button_node()
                                },
                                BackgroundColor(theme::Variant::Default.background(false)),
                            ))
                            .with_children(|button| {
                                button.spawn(theme::label(caption, 12., theme::BODY));
                            });
                        }
                        bar.spawn((Label::Page, text("", 13.)));
                    });
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(2),
                        ..default()
                    })
                    .with_children(|list| {
                        for index in 0..ROWS {
                            list.spawn((
                                Control::Row(index),
                                Button,
                                ActivateOnPress,
                                Hovered::default(),
                                Node {
                                    height: px(24),
                                    min_height: px(24),
                                    padding: UiRect {
                                        left: px(4),
                                        right: px(28),
                                        ..default()
                                    },
                                    overflow: Overflow::clip_x(),
                                    display: Display::None,
                                    ..default()
                                },
                                BackgroundColor(theme::Variant::Ghost.background(false)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Label::Row(index),
                                    theme::label("", 12., theme::BODY),
                                    TextLayout::no_wrap(),
                                ));
                                menus::spawn_content(button, index);
                            });
                        }
                    });
                panel.spawn((
                    Label::Footer,
                    text("Select content | F frames", 13.),
                    TextLayout::no_wrap(),
                    Node {
                        height: px(44),
                        min_height: px(44),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                ));
            })
        });
}

pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    mut content: ResMut<Content>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut requests: ResMut<workspace::Requests>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    if busy(&session) || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    match *control {
        Control::Search => requests.find = true,
        Control::ClearSearch => {
            content.set_query("").expect("Empty query is valid");
        }
        Control::Filter(filter) => {
            content.filter = filter;
            content.page = 0;
        }
        Control::Previous => content.page = content.page.saturating_sub(1),
        Control::Next => content.page = (content.page + 1).min(content.pages() - 1),
        Control::Row(index) => {
            select_row(index, event.count, &content, &mut session, &mut view);
        }
    }
}

pub(super) fn select_row(
    index: usize,
    click_count: u8,
    content: &Content,
    session: &mut Session,
    view: &mut View,
) -> Option<Target> {
    if workspace::busy(session) {
        return None;
    }
    let entry = content.row(index)?;
    let target = entry.target;
    let label = entry.label.clone();
    let zone = session.editor.zone();
    let present = match target {
        Target::Terrain => zone.terrain.is_some(),
        Target::Spawn => true,
        Target::Object(id) => zone.objects.iter().any(|v| v.id == id),
        Target::Guide(id) => zone.paths.iter().any(|v| v.id == id),
        Target::Marker(id) => zone.markers.iter().any(|v| v.id == id),
        Target::Plant(id) => zone.plants.iter().any(|v| v.id == id),
    };
    if !present {
        session.status = "Content changed; select an updated row".into();
        return None;
    }
    match target {
        Target::Terrain => {
            if !session.terrain_mode {
                session.set_mode(EditorMode::Sculpt);
            }
        }
        Target::Spawn => {
            view.target =
                Vec3::from_array(zone.local(zone.spawn).expect("Validated spawn")) + Vec3::Y;
            view.distance = 12.;
        }
        Target::Object(id) => {
            session.set_mode(EditorMode::Objects);
            session.selected = Some(id);
        }
        Target::Guide(id) => {
            session.set_mode(EditorMode::Guides);
            session.path_selected = Some(id);
        }
        Target::Marker(id) => {
            session.set_mode(EditorMode::Markers);
            session.marker_selected = Some(id);
        }
        Target::Plant(id) => {
            session.set_mode(EditorMode::Plants);
            session.plant_selected = Some(id);
        }
    }
    session.status = if target == Target::Spawn {
        "Focused playtest spawn".into()
    } else {
        format!("Selected {label}")
    };
    if click_count >= 2 && target != Target::Spawn {
        if session.path_mode {
            paths::focus(session, view);
            session.status = format!("Focused {label}");
        } else {
            selection::frame(session, view);
        }
    }
    Some(target)
}

fn busy(session: &Session) -> bool {
    session.has_modal()
        || session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.path_draft.is_some()
}

pub(super) fn scroll(
    scroll: Res<AccumulatedMouseScroll>,
    windows: Query<&Window>,
    session: Res<Session>,
    focus: Option<Res<bevy::input_focus::InputFocus>>,
    menu_items: Query<(), With<bevy::ui_widgets::MenuItem>>,
    panes: Query<(&scrolling::Pane, &ComputedNode)>,
    mut content: ResMut<Content>,
) {
    let Ok(window) = windows.single() else { return };
    if scroll.delta.y == 0. || !scroll.delta.y.is_finite() {
        return;
    }
    let cursor = window.cursor_position();
    let over_panel = cursor.is_some_and(|p| {
        p.x >= sidebar::WIDTH
            && p.x < sidebar::WIDTH + session.ui.browser
            && p.y >= workspace::HEADER
            && p.y < window.height() - 52.
    });
    let before = content.page;
    let blocked = !window.focused
        || session.ui.input_owned
        || busy(&session)
        || focus
            .as_ref()
            .and_then(|f| f.get())
            .is_some_and(|entity| menu_items.contains(entity));
    // Short layouts scroll within the current bounded page. Full-height layouts
    // retain wheel paging; never apply both actions to one wheel gesture.
    let overflow = panes.iter().any(|(scope, node)| {
        matches!(scope, scrolling::Pane::Browser) && node.content_size().y > node.size().y + 0.5
    });
    if over_panel && !blocked && !overflow {
        if scroll.delta.y < 0. {
            content.page = (content.page + 1).min(content.pages() - 1);
        } else if scroll.delta.y > 0. {
            content.page = content.page.saturating_sub(1);
        }
    }
    content.scroll_events = content.scroll_events.saturating_add(1);
    content.last_scroll = Some(serde_json::json!({
        "delta":scroll.delta.y,"cursor":cursor.map(|point| point.to_array()),
        "focused":window.focused,"blocked":blocked,"over_panel":over_panel,
        "page_before":before,"page_after":content.page,"routed_to_pane":over_panel && overflow,
    }));
}

pub(super) fn sync(
    mut commands: Commands,
    session: Res<Session>,
    mut content: ResMut<Content>,
    mut root: Query<&mut Node, With<Root>>,
    mut controls: Query<
        (Entity, &Control, &Hovered, &mut Node, &mut BackgroundColor),
        Without<Root>,
    >,
    mut labels: Query<(&Label, &mut Text)>,
    mut panes: Query<(&scrolling::Pane, &mut ScrollPosition)>,
) {
    let visible = session.play.is_none();
    for mut node in &mut root {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    let filtering =
        content.applied_filter != Some(content.filter) || content.applied_query != content.query;
    if content.revision != Some(session.editor.revision()) || filtering {
        let all = entries(session.editor.zone());
        content.total = all.len();
        let query = content.query.to_lowercase();
        content.entries = all
            .into_iter()
            .filter(|entry| {
                content.filter.accepts(entry.target) && entry.label.to_lowercase().contains(&query)
            })
            .collect();
        content.revision = Some(session.editor.revision());
        content.applied_filter = Some(content.filter);
        content.applied_query = content.query.clone();
        if filtering {
            content.page = 0;
        }
    }
    let selected = selection(&session);
    let index = selected.and_then(|target| {
        content
            .entries
            .iter()
            .position(|entry| entry.target == target)
    });
    let reveal_selected =
        selected != content.selection || (index != content.selection_index && !filtering);
    if reveal_selected && let Some(index) = index {
        content.page = index / ROWS;
    }
    content.selection = selected;
    content.selection_index = index;
    content.page = content.page.min(content.pages() - 1);
    if content.applied_page != Some(content.page) || filtering {
        for (scope, mut position) in &mut panes {
            if matches!(scope, scrolling::Pane::Browser) {
                position.y = 0.;
            }
        }
        content.applied_page = Some(content.page);
    }
    for (entity, control, hovered, mut node, mut background) in &mut controls {
        let disabled = busy(&session)
            || match *control {
                Control::Previous => content.page == 0,
                Control::Next => content.page + 1 == content.pages(),
                Control::Row(index) => content.row(index).is_none(),
                Control::Filter(_) => false,
                Control::Search => false,
                Control::ClearSearch => content.query.is_empty(),
            };
        if disabled {
            commands.entity(entity).insert(InteractionDisabled);
        } else {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
        let active = match *control {
            Control::Row(index) => {
                let entry = content.row(index);
                node.display = if entry.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let active = entry.is_some_and(|entry| Some(entry.target) == selected);
                if active && reveal_selected {
                    commands.trigger(bevy::ui_widgets::ScrollIntoView { entity });
                }
                active
            }
            Control::Filter(filter) => content.filter == filter,
            _ => false,
        };
        background.0 = if disabled {
            theme::PANEL
        } else if active {
            theme::Variant::Active.background(false)
        } else if hovered.0 {
            theme::Variant::Ghost.background(true)
        } else {
            theme::Variant::Ghost.background(false)
        };
    }
    for (label, mut value) in &mut labels {
        let next = match *label {
            Label::Title => {
                if content.filter == Filter::All && content.query.is_empty() {
                    format!("Zone content · {}", content.entries.len())
                } else {
                    format!("Zone content · {}/{}", content.entries.len(), content.total)
                }
            }
            Label::Page => format!("{}/{}", content.page + 1, content.pages()),
            Label::Search => {
                if content.query.is_empty() {
                    "Find...".into()
                } else {
                    format!("Find: {}", content.query)
                }
            }
            Label::Footer => index.map_or(
                if selected.is_some() {
                    "Selection hidden by filter\nChoose All to reveal".into()
                } else if content.entries.is_empty() {
                    "No matching content\nChoose another filter".into()
                } else {
                    "Double-click content to frame\nSpawn row frames the spawn".into()
                },
                |index| {
                    format!(
                        "{}\n{}",
                        content.entries[index].label,
                        if content.entries[index].target == Target::Terrain {
                            "Double-click frames terrain"
                        } else {
                            "Double-click frames | F frames"
                        }
                    )
                },
            ),
            Label::Row(index) => content
                .row(index)
                .map_or(String::new(), |entry| entry.label.clone()),
        };
        if **value != next {
            **value = next;
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use bevy::input_focus::InputFocus;

    // This app runs the actual UI setup/sync/input systems, with a data-only
    // Window. It has no Winit, renderer, display connection or OS input driver.
    pub(crate) fn app(origin: [f64; 3]) -> (App, Entity) {
        let mut zone = Zone {
            origin,
            spawn: [origin[0], origin[1] + 0.0625, origin[2] + 6.],
            ..Zone::default()
        };
        for id in 1..=43 {
            zone.objects.push(Object {
                id,
                name: if id == 43 {
                    "Élan grove".into()
                } else {
                    format!("Block {id}")
                },
                primitive: Primitive::Box,
                pose: Pose {
                    position: [origin[0] + id as f64 * 4., origin[1] + 1., origin[2]],
                    yaw_degrees: 0.,
                    size: [2.; 3],
                },
            });
        }
        let mut editor = Editor::new(zone.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Temporary name".into(),
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        let mut app = App::new();
        app.insert_resource(Session::new(editor, PathBuf::new(), Some(zone)))
            .init_resource::<AccumulatedMouseScroll>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputFocus>()
            .init_resource::<workspace::Requests>()
            .insert_resource(View {
                target: Vec3::ZERO,
                yaw: 0.6,
                pitch: 0.8,
                distance: 32.,
                preset: camera_views::Preset::Perspective,
                show_grid: true,
            })
            .add_observer(activate)
            .add_systems(Startup, (setup, naming::setup))
            .add_systems(Update, (naming::keyboard, scroll, sync).chain());
        let mut window = Window {
            resolution: (3440, 1440).into(),
            focused: true,
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(150., 350.)));
        let window = app.world_mut().spawn(window).id();
        app.update();
        (app, window)
    }
    fn wheel(app: &mut App, delta: f32) {
        app.world_mut()
            .resource_mut::<AccumulatedMouseScroll>()
            .delta
            .y = delta;
        app.update();
    }
    fn page(app: &App) -> usize {
        app.world().resource::<Content>().page
    }
    fn disabled(app: &mut App, target: Control) -> bool {
        app.world_mut()
            .query::<(&Control, Has<InteractionDisabled>)>()
            .iter(app.world())
            .find(|(control, _)| {
                matches!(
                    (control, target),
                    (Control::Previous, Control::Previous) | (Control::Next, Control::Next)
                )
            })
            .unwrap()
            .1
    }
    fn press(app: &mut App, window: Entity, control: Control) {
        press_count(app, window, control, 1);
    }
    fn press_count(app: &mut App, window: Entity, control: Control, count: u8) {
        use bevy::picking::{
            backend::HitData,
            events::Pointer,
            pointer::{Location, PointerId},
        };
        let entity = app
            .world_mut()
            .query::<(Entity, &Control)>()
            .iter(app.world())
            .find(|(_, found)| **found == control)
            .unwrap()
            .0;
        app.world_mut().trigger(PointerPress {
            entity,
            button: PointerButton::Primary,
            count,
            pointer: Pointer::new(
                PointerId::Mouse,
                Location {
                    target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(
                        window,
                    ))
                    .normalize(Some(window))
                    .unwrap(),
                    position: Vec2::new(150., 150.),
                },
            ),
            hit: HitData::new(window, 0., None, None),
        });
        wheel(app, 0.);
    }
    #[test]
    fn double_click_frames_all_entry_kinds_at_local_and_far_origins_without_source_edits() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let (mut app, window) = app(origin);
            let mut template = Editor::new(Zone::default()).unwrap();
            template
                .execute(
                    serde_json::from_slice(include_bytes!(
                        "../../../content/zones/selection.commands.jsonl"
                    ))
                    .unwrap(),
                )
                .unwrap();
            let mut source = template.zone().clone();
            let shift = |position: [f64; 3]| std::array::from_fn(|i| position[i] + origin[i]);
            source.origin = shift(source.origin);
            source.spawn = shift(source.spawn);
            for object in &mut source.objects {
                object.pose.position = shift(object.pose.position);
            }
            for path in &mut source.paths {
                for point in &mut path.points {
                    *point = shift(*point);
                }
            }
            for marker in &mut source.markers {
                marker.position = shift(marker.position);
            }
            for plant in &mut source.plants {
                plant.position = shift(plant.position);
            }
            source.schema = 7;
            source.terrain = Some(theia_zone::terrain::HeightPatch::flat([
                (origin[0] / 128.).floor() as i64,
                (origin[2] / 128.).floor() as i64,
            ]));
            let mut editor = Editor::new(source.clone()).unwrap();
            editor
                .execute(Command::Apply {
                    expected_revision: 0,
                    edits: vec![Edit::RenameZone {
                        name: "Temporary name".into(),
                    }],
                })
                .unwrap();
            editor
                .execute(Command::Undo {
                    expected_revision: 1,
                })
                .unwrap();
            app.insert_resource(Session::new(editor, PathBuf::new(), Some(source.clone())))
                .insert_resource(Content::default());
            wheel(&mut app, 0.);
            for (filter, expected) in [
                (Filter::Objects, Vec3::new(400., 1.5, 400.)),
                (Filter::Guides, Vec3::new(395., 0.03, -380.)),
                (Filter::Markers, Vec3::new(-300., 0., 300.)),
                (Filter::Plants, Vec3::new(350., 20., -300.)),
                (Filter::Terrain, Vec3::new(63.875, 0., -63.5)),
            ] {
                press(&mut app, window, Control::Filter(filter));
                {
                    let mut view = app.world_mut().resource_mut::<View>();
                    view.target = Vec3::ZERO;
                    view.distance = 32.;
                    view.preset = camera_views::Preset::Front;
                }
                press(&mut app, window, Control::Row(0));
                assert_eq!(app.world().resource::<View>().target, Vec3::ZERO);
                assert_eq!(app.world().resource::<View>().distance, 32.);
                press_count(&mut app, window, Control::Row(0), 2);
                let view = app.world().resource::<View>();
                assert!(
                    view.target.abs_diff_eq(expected, 0.001),
                    "{filter:?}: {:?}",
                    view.target
                );
                assert!((8. ..=selection::MAX_VIEW_DISTANCE).contains(&view.distance));
                assert_eq!(view.preset, camera_views::Preset::Front);
            }
            // Spawn already frames on a single click; a double click must keep
            // framing the spawn even when the previous terrain mode remains active.
            press(&mut app, window, Control::Filter(Filter::All));
            press(&mut app, window, Control::Row(1));
            let spawn_target = Vec3::from_array(source.local(source.spawn).unwrap()) + Vec3::Y;
            assert_eq!(app.world().resource::<View>().target, spawn_target);
            press_count(&mut app, window, Control::Row(1), 2);
            assert_eq!(app.world().resource::<View>().target, spawn_target);
            paths::perform(
                Action::NewPath(theia_zone::paths::PathKind::Road),
                &mut app.world_mut().resource_mut::<Session>(),
            );
            press_count(&mut app, window, Control::Row(2), 2);
            assert!(app.world().resource::<Session>().path_draft.is_some());
            assert_eq!(app.world().resource::<View>().target, spawn_target);
            app.world_mut().resource_mut::<Session>().path_draft = None;
            app.world_mut()
                .entity_mut(window)
                .get_mut::<Window>()
                .unwrap()
                .focused = false;
            press_count(&mut app, window, Control::Row(2), 2);
            assert_eq!(app.world().resource::<View>().target, spawn_target);
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
    }
    #[test]
    fn content_search_form_apply_cancel_validation_and_clear_preserve_source_and_redo() {
        use bevy::text::EditableText;
        fn input(app: &mut App, value: &str) {
            let mut query = app
                .world_mut()
                .query_filtered::<&mut EditableText, With<naming::Field>>();
            query
                .single_mut(app.world_mut())
                .unwrap()
                .editor_mut()
                .set_text(value);
        }
        fn submit(app: &mut App, value: &str) {
            input(app, value);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            wheel(app, 0.);
        }
        let (mut app, window) = app([0.; 3]);
        app.add_systems(PostUpdate, naming::finish);
        let original = app.world().resource::<Session>().editor.zone().clone();
        assert!(
            app.world_mut()
                .query::<(&Control, Has<InteractionDisabled>)>()
                .iter(app.world())
                .find(|(control, _)| **control == Control::ClearSearch)
                .unwrap()
                .1
        );
        press(&mut app, window, Control::Search);
        assert!(app.world().resource::<Session>().has_modal());
        assert!(!app.world().resource::<Session>().is_dirty());
        submit(&mut app, "éLAN");
        assert!(!app.world().resource::<Session>().has_modal());
        assert_eq!(
            app.world().resource::<Content>().row(0).unwrap().target,
            Target::Object(43)
        );
        assert_eq!(app.world().resource::<Content>().entries.len(), 1);
        press(&mut app, window, Control::Filter(Filter::Markers));
        assert!(app.world().resource::<Content>().entries.is_empty());
        assert_eq!(app.world().resource::<Content>().query, "éLAN");
        press(&mut app, window, Control::Filter(Filter::All));
        press(&mut app, window, Control::Row(0));
        assert_eq!(app.world().resource::<Session>().selected, Some(43));
        press(&mut app, window, Control::Search);
        submit(&mut app, "bad\nquery");
        assert!(app.world().resource::<Session>().has_modal());
        assert!(!app.world().resource::<Session>().is_dirty());
        assert_eq!(app.world().resource::<Content>().query, "éLAN");
        submit(&mut app, &"é".repeat(65));
        assert!(app.world().resource::<Session>().has_modal());
        assert_eq!(app.world().resource::<Content>().query, "éLAN");
        submit(&mut app, &"é".repeat(64));
        assert!(
            !app.world().resource::<Session>().has_modal(),
            "status={}, draft={:?}",
            app.world().resource::<Session>().status,
            app.world()
                .resource::<Session>()
                .name_edit
                .as_ref()
                .map(|draft| &draft.text)
        );
        assert_eq!(app.world().resource::<Content>().query.len(), 128);
        assert!(app.world().resource::<Content>().entries.is_empty());
        press(&mut app, window, Control::ClearSearch);
        assert_eq!(app.world().resource::<Content>().entries.len(), 44);
        press(&mut app, window, Control::Search);
        submit(&mut app, "  #43  ");
        assert_eq!(app.world().resource::<Content>().query, "#43");
        assert_eq!(app.world().resource::<Content>().entries.len(), 1);
        press(&mut app, window, Control::Search);
        input(&mut app, "Block");
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        assert_eq!(app.world().resource::<Content>().query, "#43");
        press(&mut app, window, Control::Search);
        input(&mut app, "different search");
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = false;
        app.update();
        assert!(!app.world().resource::<Session>().has_modal());
        assert_eq!(app.world().resource::<Content>().query, "#43");
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = true;
        press(&mut app, window, Control::ClearSearch);
        let session = app.world().resource::<Session>();
        assert_eq!(session.editor.zone(), &original);
        assert_eq!(session.editor.revision(), 2);
        assert_eq!(session.editor.history(), (0, 1));
        assert!(!session.is_dirty());
        // Reusing the form must retain the existing authoring dirty contract.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);
        wheel(&mut app, 0.);
        assert!(app.world().resource::<Session>().is_dirty());
    }
    #[test]
    fn filter_buttons_select_each_content_kind_without_dirtying_or_losing_redo() {
        let (mut app, window) = app([0.; 3]);
        let mut template = Editor::new(Zone::default()).unwrap();
        template
            .execute(
                serde_json::from_slice(include_bytes!(
                    "../../../content/zones/selection.commands.jsonl"
                ))
                .unwrap(),
            )
            .unwrap();
        let mut source = app.world().resource::<Session>().editor.zone().clone();
        source.paths = template.zone().paths.clone();
        source.markers = template.zone().markers.clone();
        source.plants = template.zone().plants.clone();
        source.schema = 7;
        source.terrain = Some(theia_zone::terrain::HeightPatch::flat([0, 0]));
        let mut editor = Editor::new(source.clone()).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::RenameZone {
                    name: "Temporary name".into(),
                }],
            })
            .unwrap();
        editor
            .execute(Command::Undo {
                expected_revision: 1,
            })
            .unwrap();
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            session.editor = editor;
            session.saved_zone = Some(source.clone());
            session.selected = Some(43);
        }
        // Different source data can have the same revision after replacement
        // of the test fixture. In normal use revisions are strictly monotonic.
        app.world_mut().resource_mut::<Content>().revision = None;
        wheel(&mut app, 0.);
        for (filter, count, target) in [
            (Filter::Objects, 43, Target::Object(1)),
            (Filter::Guides, 1, Target::Guide(1002)),
            (Filter::Markers, 1, Target::Marker(1000)),
            (Filter::Plants, 1, Target::Plant(1001)),
            (Filter::Terrain, 1, Target::Terrain),
            (Filter::All, 48, Target::Terrain),
        ] {
            press(&mut app, window, Control::Filter(filter));
            // A filter change is not a new selection: later frames must retain
            // its first page rather than jumping back to the old selected ID.
            wheel(&mut app, 0.);
            wheel(&mut app, 0.);
            let content = app.world().resource::<Content>();
            assert_eq!(content.filter, filter);
            assert_eq!(content.page, 0);
            assert_eq!(content.entries.len(), count);
            assert_eq!(content.total, 48);
            assert_eq!(content.row(0).unwrap().target, target);
            let session = app.world().resource::<Session>();
            assert_eq!(session.selected, Some(43));
            assert_eq!(session.editor.zone(), &source);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
        press(&mut app, window, Control::Filter(Filter::Markers));
        press(&mut app, window, Control::Row(0));
        assert_eq!(
            app.world().resource::<Session>().marker_selected,
            Some(1000)
        );
        assert!(app.world().resource::<Session>().marker_mode);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);
        wheel(&mut app, 0.);
        assert!(app.world().resource::<Session>().name_edit.is_some());
        press(&mut app, window, Control::Filter(Filter::All));
        assert_eq!(app.world().resource::<Content>().filter, Filter::Markers);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().resource_mut::<Session>().name_edit = None;
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Apply {
                expected_revision: 2,
                edits: vec![Edit::DeleteMarker { id: 1000 }],
            })
            .unwrap();
        wheel(&mut app, 0.);
        assert_eq!(app.world().resource::<Content>().entries.len(), 0);
        assert_eq!(page(&app), 0);
        assert!(disabled(&mut app, Control::Previous));
        assert!(disabled(&mut app, Control::Next));
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        wheel(&mut app, 0.);
        assert_eq!(
            app.world().resource::<Content>().row(0).unwrap().target,
            Target::Marker(1000)
        );
        assert_eq!(app.world().resource::<Session>().editor.zone(), &source);
    }
    #[test]
    fn real_bevy_systems_page_all_rows_preserving_source_revision_and_pending_redo() {
        for origin in [
            [0.125, 0.25, -0.5],
            [128_000_000_000.125, 20.25, -128_000_000_000.5],
        ] {
            let (mut app, _) = app(origin);
            let before = app.world().resource::<Session>().editor.zone().clone();
            assert_eq!(app.world().resource::<Content>().pages(), 3);
            assert!(disabled(&mut app, Control::Previous));
            let mut ids = vec![];
            for index in 0..3 {
                assert_eq!(page(&app), index);
                ids.extend(
                    (0..ROWS)
                        .filter_map(|i| app.world().resource::<Content>().row(i).map(|e| e.target)),
                );
                // Keeping an earlier selected entry must not pull wheel browsing
                // back to its page when selection/source have not changed.
                if index == 0 {
                    app.world_mut().resource_mut::<Session>().selected = Some(17);
                    wheel(&mut app, 0.);
                }
                wheel(&mut app, -1.);
            }
            assert_eq!(page(&app), 2);
            assert!(disabled(&mut app, Control::Next));
            assert_eq!(
                ids,
                std::iter::once(Target::Spawn)
                    .chain((1..=43).map(Target::Object))
                    .collect::<Vec<_>>()
            );
            wheel(&mut app, 1.);
            wheel(&mut app, 1.);
            wheel(&mut app, 1.);
            assert_eq!(page(&app), 0);
            let session = app.world().resource::<Session>();
            assert_eq!(session.editor.zone(), &before);
            assert_eq!(session.editor.revision(), 2);
            assert_eq!(session.editor.history(), (0, 1));
            assert!(!session.is_dirty());
        }
    }
    #[test]
    fn focus_panel_edges_invalid_input_and_editor_drafts_gate_wheel_routing() {
        let (mut app, window) = app([0.; 3]);
        for point in [
            None,
            Some(Vec2::new(-1., 350.)),
            Some(Vec2::new(sidebar::WIDTH - 1., 350.)),
            Some(Vec2::new(150., workspace::HEADER - 1.)),
            Some(Vec2::new(150., 1388.)),
            Some(Vec2::new(sidebar::WIDTH + WIDTH, 350.)),
        ] {
            app.world_mut()
                .entity_mut(window)
                .get_mut::<Window>()
                .unwrap()
                .set_cursor_position(point);
            wheel(&mut app, -1.);
            assert_eq!(page(&app), 0);
        }
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .set_cursor_position(Some(Vec2::new(sidebar::WIDTH, workspace::HEADER)));
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = false;
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 0);
        assert_eq!(
            app.world()
                .resource::<Content>()
                .last_scroll
                .as_ref()
                .unwrap()["blocked"],
            true
        );
        app.world_mut()
            .entity_mut(window)
            .get_mut::<Window>()
            .unwrap()
            .focused = true;
        for delta in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.] {
            wheel(&mut app, delta);
            assert_eq!(page(&app), 0);
        }
        paths::perform(
            Action::NewPath(theia_zone::paths::PathKind::Road),
            &mut app.world_mut().resource_mut::<Session>(),
        );
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 0);
        assert!(disabled(&mut app, Control::Next));
        app.world_mut().resource_mut::<Session>().path_draft = None;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);
        wheel(&mut app, -1.);
        assert!(app.world().resource::<Session>().name_edit.is_some());
        assert_eq!(page(&app), 0);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut().resource_mut::<Session>().name_edit = None;
        wheel(&mut app, -0.125);
        assert_eq!(page(&app), 1);
        assert!(!disabled(&mut app, Control::Previous));
        assert!(!disabled(&mut app, Control::Next));
    }
    #[test]
    fn held_transform_and_actual_offline_playtest_block_content_navigation() {
        let (mut app, _) = app([0.; 3]);
        let original = app.world().resource::<Session>().editor.zone().clone();
        let pose = original.objects[0].pose.clone();
        let gesture = gizmo::Gesture::plane(
            Ray3d::new(Vec3::new(4., 8., 0.), Dir3::NEG_Y),
            Vec3::new(4., 1., 0.),
        )
        .unwrap();
        app.world_mut().resource_mut::<Session>().drag = Some(Drag {
            id: 1,
            before: pose.clone(),
            preview: pose,
            revision: 2,
            gesture,
        });
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 0);
        assert!(disabled(&mut app, Control::Next));
        app.world_mut().resource_mut::<Session>().drag = None;
        let play = playtest::Playtest::new(&original).unwrap();
        app.world_mut().resource_mut::<Session>().play = Some(play);
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 0);
        assert_eq!(
            app.world_mut()
                .query_filtered::<&Node, With<Root>>()
                .single(app.world())
                .unwrap()
                .display,
            Display::None
        );
        app.world_mut().resource_mut::<Session>().play = None;
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 1);
        assert!(!disabled(&mut app, Control::Next));
        assert_eq!(app.world().resource::<Session>().editor.zone(), &original);
        assert_eq!(app.world().resource::<Session>().editor.history(), (0, 1));
    }
    #[test]
    fn deletion_clamps_to_existing_page_and_undo_restores_complete_content() {
        let (mut app, _) = app([0.; 3]);
        wheel(&mut app, -1.);
        wheel(&mut app, -1.);
        assert_eq!(page(&app), 2);
        let original = app.world().resource::<Session>().editor.zone().clone();
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Apply {
                expected_revision: 2,
                edits: (1..=43).map(|id| Edit::Delete { id }).collect(),
            })
            .unwrap();
        wheel(&mut app, 0.);
        assert_eq!(page(&app), 0);
        assert_eq!(app.world().resource::<Content>().entries.len(), 1);
        assert!(disabled(&mut app, Control::Previous));
        assert!(disabled(&mut app, Control::Next));
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Undo {
                expected_revision: 3,
            })
            .unwrap();
        wheel(&mut app, 0.);
        assert_eq!(app.world().resource::<Content>().pages(), 3);
        assert_eq!(app.world().resource::<Session>().editor.zone(), &original);
    }
    #[test]
    fn standard_bevy_cpu_layout_keeps_every_row_and_long_footer_above_properties() {
        let (mut app, _) = app([0.; 3]);
        crate::headless_ui::add_layout(&mut app);
        app.world_mut()
            .resource_mut::<Session>()
            .editor
            .execute(Command::Apply {
                expected_revision: 2,
                edits: vec![Edit::Rename {
                    id: 17,
                    name: "A".repeat(48),
                }],
            })
            .unwrap();
        app.world_mut().resource_mut::<Session>().selected = Some(17);
        for _ in 0..3 {
            app.update();
        }
        for expected_page in 0..3 {
            if expected_page != 0 {
                wheel(&mut app, -1.);
            }
            assert_eq!(page(&app), expected_page);
            let mut visible = 0;
            for (control, node, computed, transform) in app
                .world_mut()
                .query::<(&Control, &Node, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                .iter(app.world())
            {
                if matches!(
                    control,
                    Control::Filter(_) | Control::Search | Control::ClearSearch
                ) {
                    assert_eq!(
                        computed.size().x,
                        match control {
                            Control::Search => 158.,
                            Control::ClearSearch => 100.,
                            _ => 86.,
                        }
                    );
                    assert!(computed.size().y >= 26.);
                    let position = transform.affine().translation;
                    let half = computed.size() * 0.5;
                    assert!(
                        position.x - half.x >= sidebar::WIDTH
                            && position.x + half.x <= sidebar::WIDTH + WIDTH
                    );
                    assert!(
                        position.y - half.y >= workspace::HEADER && position.y + half.y <= 1388.
                    );
                }
                if !matches!(control, Control::Row(_)) || node.display == Display::None {
                    continue;
                }
                let position = transform.affine().translation;
                let half = computed.size() * 0.5;
                assert_eq!(computed.size().y, 24.);
                assert!(
                    position.x - half.x >= sidebar::WIDTH
                        && position.x + half.x <= sidebar::WIDTH + WIDTH
                );
                assert!(position.y - half.y >= workspace::HEADER && position.y + half.y <= 1388.);
                visible += 1;
            }
            assert_eq!(visible, if expected_page == 2 { 8 } else { 18 });
            for (label, computed, transform) in app
                .world_mut()
                .query::<(&Label, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                .iter(app.world())
            {
                if matches!(label, Label::Title) {
                    assert!(computed.size().y >= 14.);
                }
                if matches!(label, Label::Footer) {
                    assert_eq!(computed.size().y, 44.);
                    assert!(transform.affine().translation.y + computed.size().y / 2. <= 1388.);
                }
            }
        }
    }
}
