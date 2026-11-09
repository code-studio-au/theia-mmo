//! A fixed row pool for browsing bounded zone content with standard Bevy UI.
use super::*;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
use bevy::ui::InteractionDisabled;
pub(super) const WIDTH: f32 = 300.;
// Leave room for the title, paging and a two-line footer above Properties at
// the supported 3440 x 1440 authoring size. Every pooled row must be clickable.
const ROWS: usize = 18;

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
    applied_filter: Option<Filter>,
    revision: Option<u64>,
    selection: Option<Target>,
    selection_index: Option<usize>,
    pub scroll_events: u64,
    pub last_scroll: Option<serde_json::Value>,
}
impl Content {
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
}
#[derive(Component)]
pub(super) enum Label {
    Title,
    Page,
    Footer,
    Row(usize),
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
fn selection(session: &Session) -> Option<Target> {
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
            GlobalZIndex(3),
            Node {
                position_type: PositionType::Absolute,
                top: px(workspace::HEADER),
                right: px(0),
                bottom: px(workspace::INSPECTOR),
                width: px(WIDTH),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.05, 0.075)),
        ))
        .with_children(|panel| {
            panel.spawn((Label::Title, text("ZONE CONTENT", 20.)));
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
                                min_height: px(26),
                                padding: UiRect::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.10, 0.16, 0.22)),
                            children![text(caption, 13.)],
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
                                min_height: px(26),
                                padding: UiRect::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.10, 0.16, 0.22)),
                        ))
                        .with_children(|button| {
                            button.spawn(text(caption, 13.));
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
                                padding: UiRect::horizontal(px(4)),
                                overflow: Overflow::clip_x(),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.08, 0.11, 0.16)),
                        ))
                        .with_children(|button| {
                            button.spawn((Label::Row(index), text("", 13.), TextLayout::no_wrap()));
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
        });
}

pub(super) fn activate(
    event: On<PointerPress>,
    controls: Query<&Control>,
    windows: Query<&Window>,
    mut content: ResMut<Content>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
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
        Control::Filter(filter) => {
            content.filter = filter;
            content.page = 0;
        }
        Control::Previous => content.page = content.page.saturating_sub(1),
        Control::Next => content.page = (content.page + 1).min(content.pages() - 1),
        Control::Row(index) => {
            let Some(entry) = content.row(index) else {
                return;
            };
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
                return;
            }
            match target {
                Target::Terrain => {
                    if !session.terrain_mode {
                        session.set_mode(EditorMode::Sculpt);
                    }
                }
                Target::Spawn => {
                    view.target =
                        Vec3::from_array(zone.local(zone.spawn).expect("Validated spawn"))
                            + Vec3::Y;
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
        }
    }
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
    mut content: ResMut<Content>,
) {
    let Ok(window) = windows.single() else { return };
    if scroll.delta.y == 0. || !scroll.delta.y.is_finite() {
        return;
    }
    let cursor = window.cursor_position();
    let over_panel = cursor.is_some_and(|p| {
        p.x >= window.width() - WIDTH
            && p.x < window.width()
            && p.y >= workspace::HEADER
            && p.y < window.height() - workspace::INSPECTOR
    });
    let before = content.page;
    let blocked = !window.focused || busy(&session);
    if over_panel && !blocked {
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
        "page_before":before,"page_after":content.page,
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
) {
    let visible = session.play.is_none();
    for mut node in &mut root {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    let filtering = content.applied_filter != Some(content.filter);
    if content.revision != Some(session.editor.revision()) || filtering {
        let all = entries(session.editor.zone());
        content.total = all.len();
        content.entries = all
            .into_iter()
            .filter(|entry| content.filter.accepts(entry.target))
            .collect();
        content.revision = Some(session.editor.revision());
        content.applied_filter = Some(content.filter);
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
    if (selected != content.selection || (index != content.selection_index && !filtering))
        && let Some(index) = index
    {
        content.page = index / ROWS;
    }
    content.selection = selected;
    content.selection_index = index;
    content.page = content.page.min(content.pages() - 1);
    for (entity, control, hovered, mut node, mut background) in &mut controls {
        let disabled = busy(&session)
            || match *control {
                Control::Previous => content.page == 0,
                Control::Next => content.page + 1 == content.pages(),
                Control::Row(index) => content.row(index).is_none(),
                Control::Filter(_) => false,
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
                entry.is_some_and(|entry| Some(entry.target) == selected)
            }
            Control::Filter(filter) => content.filter == filter,
            _ => false,
        };
        background.0 = if disabled {
            Color::srgb(0.045, 0.06, 0.08)
        } else if active {
            Color::srgb(0.20, 0.34, 0.43)
        } else if hovered.0 {
            Color::srgb(0.16, 0.25, 0.33)
        } else {
            Color::srgb(0.08, 0.11, 0.16)
        };
    }
    for (label, mut value) in &mut labels {
        let next = match *label {
            Label::Title => {
                if content.filter == Filter::All {
                    format!("ZONE CONTENT | {}", content.entries.len())
                } else {
                    format!("ZONE CONTENT | {}/{}", content.entries.len(), content.total)
                }
            }
            Label::Page => format!("{}/{}", content.page + 1, content.pages()),
            Label::Footer => index.map_or(
                if selected.is_some() {
                    "Selection hidden by filter\nChoose All to reveal".into()
                } else if content.entries.is_empty() {
                    "No matching content\nChoose another filter".into()
                } else {
                    "Select content | F frames\nSpawn row frames the spawn".into()
                },
                |index| {
                    format!(
                        "{}\n{}",
                        content.entries[index].label,
                        if content.entries[index].target == Target::Terrain {
                            "F frames the terrain"
                        } else {
                            "F frames | Ctrl D copies"
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
mod tests {
    use super::*;
    use bevy::input_focus::InputFocus;

    // This app runs the actual UI setup/sync/input systems, with a data-only
    // Window. It has no Winit, renderer, display connection or OS input driver.
    fn app(origin: [f64; 3]) -> (App, Entity) {
        let mut zone = Zone {
            origin,
            spawn: [origin[0], origin[1] + 0.0625, origin[2] + 6.],
            ..Zone::default()
        };
        for id in 1..=43 {
            zone.objects.push(Object {
                id,
                name: format!("Block {id}"),
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
        window.set_cursor_position(Some(Vec2::new(3290., 350.)));
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
            count: 1,
            pointer: Pointer::new(
                PointerId::Mouse,
                Location {
                    target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(
                        window,
                    ))
                    .normalize(Some(window))
                    .unwrap(),
                    position: Vec2::new(3290., 150.),
                },
            ),
            hit: HitData::new(window, 0., None, None),
        });
        wheel(app, 0.);
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
            Some(Vec2::new(3139., 350.)),
            Some(Vec2::new(3290., 91.)),
            Some(Vec2::new(3290., 840.)),
            Some(Vec2::new(3440., 350.)),
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
            .set_cursor_position(Some(Vec2::new(3140., 92.)));
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
        use bevy::app::{HierarchyPropagatePlugin, PropagateSet, TaskPoolPlugin};
        use bevy::camera::{ComputedCameraValues, RenderTargetInfo, Viewport};
        use bevy::text::{FontCx, LayoutCx, RemSize, ScaleCx, TextIterScratch, TextPipeline};
        use bevy::ui::update::{propagate_ui_target_cameras, update_clipping_system};
        use bevy::ui::{
            ComputedUiRenderTargetInfo, ComputedUiTargetCamera, UiScale, ui_surface::UiSurface,
        };
        use bevy::ui::{sync_font_size_to_em_size, ui_layout_system, widget::measure_text_system};
        let (mut app, _) = app([0.; 3]);
        // Follow Bevy's own CPU layout harness, adding its real font measurement.
        // No UiPlugin/input backend, Winit, RenderPlugin, GPU or clipboard plugin.
        app.add_plugins(TaskPoolPlugin::default())
            .add_plugins(HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(
                PostUpdate,
            ))
            .add_plugins(HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(
                PostUpdate,
            ))
            .init_resource::<UiScale>()
            .init_resource::<UiSurface>()
            .init_resource::<Assets<Font>>()
            .init_resource::<TextPipeline>()
            .init_resource::<FontCx>()
            .init_resource::<LayoutCx>()
            .init_resource::<ScaleCx>()
            .init_resource::<TextIterScratch>()
            .init_resource::<RemSize>()
            .add_systems(
                PostUpdate,
                (
                    propagate_ui_target_cameras,
                    bevy::text::load_font_assets_into_font_collection,
                    bevy::text::detect_text_needs_rerender,
                    sync_font_size_to_em_size,
                    measure_text_system,
                    ui_layout_system,
                    update_clipping_system,
                )
                    .chain(),
            )
            .configure_sets(
                PostUpdate,
                PropagateSet::<ComputedUiTargetCamera>::default()
                    .after(propagate_ui_target_cameras)
                    .before(ui_layout_system),
            )
            .configure_sets(
                PostUpdate,
                PropagateSet::<ComputedUiRenderTargetInfo>::default()
                    .after(propagate_ui_target_cameras)
                    .before(sync_font_size_to_em_size),
            );
        app.world_mut()
            .resource_mut::<Assets<Font>>()
            .insert(
                bevy::asset::AssetId::default(),
                Font::from_bytes(bevy::text::DEFAULT_FONT_DATA.to_vec()),
            )
            .unwrap();
        app.world_mut().spawn((
            Camera2d,
            Camera {
                computed: ComputedCameraValues {
                    target_info: Some(RenderTargetInfo {
                        physical_size: UVec2::new(3440, 1440),
                        scale_factor: 1.,
                    }),
                    ..default()
                },
                viewport: Some(Viewport {
                    physical_size: UVec2::new(3440, 1440),
                    ..default()
                }),
                ..default()
            },
        ));
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
                if matches!(control, Control::Filter(_)) {
                    assert_eq!(computed.size().x, 86.);
                    assert!(computed.size().y >= 26.);
                    let position = transform.affine().translation;
                    let half = computed.size() * 0.5;
                    assert!(position.x - half.x >= 3140. && position.x + half.x <= 3440.);
                    assert!(position.y - half.y >= 92. && position.y + half.y <= 840.);
                }
                if !matches!(control, Control::Row(_)) || node.display == Display::None {
                    continue;
                }
                let position = transform.affine().translation;
                let half = computed.size() * 0.5;
                assert_eq!(computed.size().y, 24.);
                assert!(position.x - half.x >= 3140. && position.x + half.x <= 3440.);
                assert!(position.y - half.y >= 92. && position.y + half.y <= 840.);
                visible += 1;
            }
            assert_eq!(visible, if expected_page == 2 { 8 } else { 18 });
            for (label, computed, transform) in app
                .world_mut()
                .query::<(&Label, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
                .iter(app.world())
            {
                if matches!(label, Label::Title) {
                    assert!(computed.size().y >= 20.);
                }
                if matches!(label, Label::Footer) {
                    assert_eq!(computed.size().y, 44.);
                    assert!(transform.affine().translation.y + computed.size().y / 2. <= 840.);
                }
            }
        }
    }
}
