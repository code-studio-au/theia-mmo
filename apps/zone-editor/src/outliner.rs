//! A fixed row pool for browsing bounded zone content with standard Bevy UI.
use super::*;
use bevy::picking::{events::PointerPress, pointer::PointerButton};
pub(super) const WIDTH: f32 = 300.;
const ROWS: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Terrain,
    Spawn,
    Object(u64),
    Guide(u64),
    Marker(u64),
    Plant(u64),
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
    revision: Option<u64>,
    selection: Option<Target>,
    selection_index: Option<usize>,
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
#[derive(Component, Clone, Copy, Debug)]
pub(super) enum Control {
    Previous,
    Next,
    Row(usize),
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
                top: px(0),
                right: px(0),
                bottom: px(0),
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
            panel.spawn((Label::Footer, text("Select content | F frames", 13.)));
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
    if session.play.is_some() || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    match *control {
        Control::Previous => content.page = content.page.saturating_sub(1),
        Control::Next => content.page = (content.page + 1).min(content.pages() - 1),
        Control::Row(index) => {
            if session.drag.is_some() || session.stroke.is_some() || session.path_draft.is_some() {
                session.status = "Finish or cancel the preview before selecting content".into();
                return;
            }
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

pub(super) fn sync(
    session: Res<Session>,
    mut content: ResMut<Content>,
    mut root: Query<&mut Node, With<Root>>,
    mut controls: Query<(&Control, &Hovered, &mut Node, &mut BackgroundColor), Without<Root>>,
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
    if content.revision != Some(session.editor.revision()) {
        content.entries = entries(session.editor.zone());
        content.revision = Some(session.editor.revision());
    }
    let selected = selection(&session);
    let index = selected.and_then(|target| {
        content
            .entries
            .iter()
            .position(|entry| entry.target == target)
    });
    if selected != content.selection || index != content.selection_index {
        if let Some(index) = index {
            content.page = index / ROWS;
        }
        content.selection = selected;
        content.selection_index = index;
    }
    content.page = content.page.min(content.pages() - 1);
    for (control, hovered, mut node, mut background) in &mut controls {
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
            _ => false,
        };
        background.0 = if active {
            Color::srgb(0.20, 0.34, 0.43)
        } else if hovered.0 {
            Color::srgb(0.16, 0.25, 0.33)
        } else {
            Color::srgb(0.08, 0.11, 0.16)
        };
    }
    for (label, mut value) in &mut labels {
        let next = match *label {
            Label::Title => format!("ZONE CONTENT | {}", content.entries.len()),
            Label::Page => format!("{}/{}", content.page + 1, content.pages()),
            Label::Footer => index.map_or(
                "Select content | F frames\nSpawn row frames the spawn".into(),
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
