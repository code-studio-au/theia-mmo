//! P5-17/P3-15: standard Bevy greybox authoring; no trusted gameplay authority.
mod agent;
mod authoring_grid;
mod brush_preview;
mod camera_views;
mod cli;
mod export;
mod file_picker;
mod gizmo;
#[cfg(test)]
mod headless_ui;
mod markers;
mod naming;
mod outliner;
mod paths;
mod playtest;
mod properties;
mod selection;
mod snapping;
mod spawn;
mod terrain;
mod vegetation;
mod viewport;
mod workspace;
use bevy::picking::hover::Hovered;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::path::PathBuf;
use theia_zone::{Command, Edit, Editor, Object, Pose, Primitive, Zone};

const PANEL: f32 = 300.;
fn canvas_cursor(window: &Window) -> Option<Vec2> {
    window.cursor_position().filter(|p| {
        p.x > PANEL
            && p.x < window.width() - outliner::WIDTH
            && p.y > workspace::HEADER
            && p.y < window.height() - 52.
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditorMode {
    Objects,
    Sculpt,
    Paint,
    Guides,
    Markers,
    Plants,
}

#[derive(Resource)]
struct Session {
    editor: Editor,
    saved_zone: Option<Zone>,
    selected: Option<u64>,
    palette: Option<Primitive>,
    snap: bool,
    move_step: f64,
    show_move_handles: bool,
    gizmo_tool: gizmo::Tool,
    status: String,
    path: PathBuf,
    next_id: u64,
    drag: Option<Drag>,
    play: Option<playtest::Playtest>,
    heading: f32,
    terrain_mode: bool,
    terrain_paint: bool,
    brush_tool: terrain::Tool,
    brush_radius: f64,
    brush_strength: f32,
    sculpt_blend: f32,
    paint_blend: f32,
    flatten_height: f64,
    brush_point: Option<Vec3>,
    stroke: Option<terrain::Stroke>,
    path_mode: bool,
    marker_mode: bool,
    vegetation_mode: bool,
    plant_selected: Option<u64>,
    plant_palette: Option<theia_zone::vegetation::PlantKind>,
    plant_scatter: bool,
    scatter_radius: f64,
    scatter_count: u32,
    plant_point: Option<[f64; 3]>,
    marker_selected: Option<u64>,
    marker_palette: Option<theia_zone::markers::MarkerKind>,
    path_selected: Option<u64>,
    path_point: Option<usize>,
    path_draft: Option<paths::Draft>,
    name_edit: Option<naming::Draft>,
    transform_edit: Option<properties::Draft>,
    file_dialog: Option<file_picker::Draft>,
}
struct Drag {
    id: u64,
    before: Pose,
    preview: Pose,
    revision: u64,
    gesture: gizmo::Gesture,
}
impl Session {
    fn new(editor: Editor, path: PathBuf, saved_zone: Option<Zone>) -> Self {
        let next_id = editor.zone().next_id().unwrap_or(u64::MAX);
        Session {
            editor,
            saved_zone,
            selected: None,
            palette: None,
            snap: true,
            move_step: 1.,
            show_move_handles: true,
            gizmo_tool: gizmo::Tool::Move,
            status: "Ready - choose a primitive, then click the ground".into(),
            path,
            next_id,
            drag: None,
            play: None,
            heading: 0.,
            terrain_mode: false,
            terrain_paint: false,
            brush_tool: terrain::Tool::Raise,
            brush_radius: 12.,
            brush_strength: 2.,
            sculpt_blend: 0.5,
            paint_blend: 1.,
            flatten_height: 0.,
            brush_point: None,
            stroke: None,
            path_mode: false,
            marker_mode: false,
            vegetation_mode: false,
            plant_selected: None,
            plant_palette: None,
            plant_scatter: false,
            scatter_radius: 8.,
            scatter_count: 8,
            plant_point: None,
            marker_selected: None,
            marker_palette: None,
            path_selected: None,
            path_point: None,
            path_draft: None,
            name_edit: None,
            transform_edit: None,
            file_dialog: None,
        }
    }
    fn has_modal(&self) -> bool {
        self.name_edit.is_some() || self.transform_edit.is_some() || self.file_dialog.is_some()
    }
    fn set_mode(&mut self, mode: EditorMode) {
        self.terrain_mode = matches!(mode, EditorMode::Sculpt | EditorMode::Paint);
        self.terrain_paint = mode == EditorMode::Paint;
        if self.terrain_paint {
            self.brush_tool = terrain::Tool::Paint(theia_zone::painting::Surface::Dirt);
        } else if matches!(self.brush_tool, terrain::Tool::Paint(_)) {
            self.brush_tool = terrain::Tool::Raise;
        }
        self.path_mode = mode == EditorMode::Guides;
        self.marker_mode = mode == EditorMode::Markers;
        self.vegetation_mode = mode == EditorMode::Plants;
        self.plant_selected = None;
        self.plant_palette = None;
        self.marker_selected = None;
        self.marker_palette = None;
        self.path_draft = None;
        self.path_selected = None;
        self.path_point = None;
        self.stroke = None;
        self.drag = None;
        self.palette = None;
        self.selected = None;
        self.status = if self.terrain_paint {
            "Terrain paint - choose a surface and click or drag".into()
        } else if self.vegetation_mode {
            "Vegetation tools - place or scatter tree/shrub proxies".into()
        } else if self.marker_mode {
            "Marker tools - NPC/quest authoring annotations".into()
        } else if self.path_mode {
            "Guide tools - choose a road or stream, then add ground points".into()
        } else if self.terrain_mode {
            "Terrain tools - create a patch, then click or drag to sculpt".into()
        } else {
            "Object tools".into()
        };
    }
    fn is_dirty(&self) -> bool {
        self.saved_zone.as_ref() != Some(self.editor.zone())
            || self.drag.is_some()
            || self.stroke.is_some()
            || self.path_draft.is_some()
            || self
                .name_edit
                .as_ref()
                .is_some_and(naming::Draft::is_authoring)
            || self.transform_edit.is_some()
            || self.file_dialog.is_some()
    }
    fn execute(&mut self, command: Command) {
        let points_before = self.selected_path_len();
        match self.editor.execute(command) {
            Ok(()) => {
                self.status = format!("Revision {} accepted", self.editor.revision());
                self.reconcile_after_edit(points_before);
            }
            Err(error) => self.status = error,
        }
    }
    fn selected_path_len(&self) -> Option<usize> {
        self.editor
            .zone()
            .paths
            .iter()
            .find(|p| Some(p.id) == self.path_selected)
            .map(|p| p.points.len())
    }
    fn reconcile_after_edit(&mut self, points_before: Option<usize>) {
        if points_before != self.selected_path_len() {
            self.path_point = None;
        }
        self.reconcile();
    }
    fn reconcile(&mut self) {
        if !self
            .editor
            .zone()
            .objects
            .iter()
            .any(|o| Some(o.id) == self.selected)
        {
            self.selected = None;
        }
        if !self
            .editor
            .zone()
            .paths
            .iter()
            .any(|p| Some(p.id) == self.path_selected)
        {
            self.path_selected = None;
        }
        if self.path_point.is_some_and(|index| {
            !self
                .editor
                .zone()
                .paths
                .iter()
                .any(|p| Some(p.id) == self.path_selected && index < p.points.len())
        }) {
            self.path_point = None;
        }
        if !self
            .editor
            .zone()
            .markers
            .iter()
            .any(|m| Some(m.id) == self.marker_selected)
        {
            self.marker_selected = None;
        }
        if !self
            .editor
            .zone()
            .plants
            .iter()
            .any(|p| Some(p.id) == self.plant_selected)
        {
            self.plant_selected = None;
        }
        self.next_id = self
            .next_id
            .max(self.editor.zone().next_id().unwrap_or(u64::MAX));
    }
    fn edit(&mut self, edits: Vec<Edit>) {
        self.execute(Command::Apply {
            expected_revision: self.editor.revision(),
            edits,
        });
    }
    fn grid(&self) -> snapping::Grid {
        snapping::Grid {
            origin: self.editor.zone().origin,
            step: self.move_step,
        }
    }
    fn snapped(&self, position: [f64; 3]) -> [f64; 3] {
        if self.snap {
            std::array::from_fn(|i| {
                self.grid()
                    .coordinate(position[i], i)
                    .unwrap_or(position[i])
            })
        } else {
            position
        }
    }
}
#[derive(Component, Clone, Copy, Debug)]
enum Action {
    Select,
    Place(Primitive),
    Undo,
    Redo,
    Save,
    Load,
    Snap,
    Ground,
    Play,
    Spawn,
    Delete,
    Duplicate,
    TerrainToggle,
    TerrainResolution,
    Brush(terrain::Tool),
    Radius(i8),
    FlattenHeight(i8),
    BrushStrength(i8),
    BrushBlend(i8),
    SelectPlant,
    Plant(theia_zone::vegetation::PlantKind, bool),
    NextPlant,
    DeletePlant,
    SelectMarker,
    PlaceMarker(theia_zone::markers::MarkerKind),
    NextMarker,
    DeleteMarker,
    MarkerRadius(i8),
    NewPath(theia_zone::paths::PathKind),
    FinishPath,
    GradePath,
    PopPath,
    InsertPoint,
    RemovePoint,
    NextPath,
    DeletePath,
    PathWidth(i8),
}
#[derive(Component)]
struct PaletteSlot(usize);
#[derive(Component)]
struct PaletteLabel;
#[derive(Component)]
struct Help;
#[derive(Component)]
struct Summary;
#[derive(Component)]
struct Info;
#[derive(Resource)]
struct View {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    preset: camera_views::Preset,
    show_grid: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Zone editor: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let options = cli::Options::parse()?;
    let editor = options.prepare()?;
    if options.check_playtest {
        let checked = playtest::Playtest::new(editor.zone())
            .and_then(|play| editor.zone().global(play.feet.metres()));
        let mut report = cli::summary(&editor);
        report["playtest"] = serde_json::json!({
            "ready":checked.is_ok(),"requested_spawn":editor.zone().spawn,
            "resolved_spawn":checked.as_ref().ok(),"error":checked.as_ref().err(),
        });
        println!("{report}");
        return checked.map(|_| ());
    }
    if let Some(path) = &options.export {
        let report = export::publish(
            editor.zone(),
            path,
            options.content_id.as_deref().expect("Checked export ID"),
            options.revision.expect("Checked export revision"),
        )?;
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if options.headless {
        println!("{}", cli::summary(&editor));
        return Ok(());
    }
    let initial_target = editor.zone().terrain.as_ref().map(|patch| {
        let start = patch.origin().expect("Validated terrain");
        let local = editor
            .zone()
            .local([start[0], editor.zone().origin[1], start[2]])
            .expect("Validated terrain frame");
        Vec3::new(local[0] + 64., 0., local[2] + 64.)
    });
    let path = options.editor_path();
    let saved_zone = Zone::load(&path).ok();
    let mut app = App::new();
    app.insert_resource(Session::new(editor, path, saved_zone));
    app.insert_resource(View {
        target: initial_target.unwrap_or(Vec3::ZERO),
        yaw: 0.6,
        pitch: 0.8,
        show_grid: true,
        distance: if initial_target.is_some() { 120. } else { 32. },
        preset: camera_views::Preset::Perspective,
    })
    .insert_resource(Time::<Fixed>::from_hz(30.))
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Theia - Zone Editor".into(),
            resolution: (3440, 1440).into(),
            mode: bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Primary,
            ),
            ..default()
        }),
        ..default()
    }))
    .add_observer(workspace::activate)
    .add_observer(workspace::change)
    .add_observer(workspace::toggle)
    .add_observer(activate)
    .add_observer(outliner::activate)
    .add_observer(naming::activate)
    .add_observer(properties::activate)
    .add_observer(properties::yaw_change)
    .add_observer(file_picker::activate)
    .add_systems(
        Startup,
        (
            viewport::setup,
            authoring_grid::setup,
            gizmo::setup,
            terrain::setup,
            paths::setup,
            markers::setup,
            vegetation::setup,
            setup_ui,
            workspace::setup,
            outliner::setup,
            naming::setup,
            properties::setup,
            file_picker::setup,
        ),
    )
    .add_systems(
        Update,
        (
            (agent::poll, file_picker::keyboard).chain(),
            naming::keyboard,
            properties::keyboard,
            keyboard,
            vegetation::pointer,
            markers::pointer,
            paths::pointer,
            terrain::pointer,
            viewport::pointer,
            camera_controls,
            terrain::sync,
            terrain::palette,
            (viewport::sync, gizmo::sync, authoring_grid::sync).chain(),
            paths::sync,
            markers::sync,
            vegetation::sync,
            (outliner::scroll, outliner::sync).chain(),
            workspace::sync,
            present,
            viewport::capture,
        )
            .chain(),
    )
    .add_systems(
        PostUpdate,
        (
            naming::finish,
            (properties::finish, properties::yaw_sync).chain(),
            file_picker::finish,
        )
            .chain()
            .after(bevy::text::EditableTextSystems),
    )
    .add_systems(FixedUpdate, playtest::step)
    .add_systems(Last, viewport::report);
    if let Some(path) = options.capture {
        app.insert_resource(viewport::Capture { path, frames: 0 });
    }
    if let Some(path) = options.report {
        app.insert_resource(viewport::Report { path, elapsed: 0. });
    }
    if let Some(path) = options.mailbox {
        app.insert_resource(agent::AgentMailbox::open(&path)?);
    }
    app.run();
    Ok(())
}
fn text(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        bevy::picking::Pickable::IGNORE,
    )
}
fn object_actions() -> [(Action, &'static str); 12] {
    [
        (Action::Select, "Select / drag [Esc]"),
        (Action::Place(Primitive::Box), "Place box [1]"),
        (Action::Place(Primitive::Sphere), "Place sphere [2]"),
        (Action::Place(Primitive::Cylinder), "Place cylinder [3]"),
        (Action::Snap, "Toggle grid snap [G]"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Delete, "Delete selected [Del]"),
        (Action::Spawn, "Spawn at view centre"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Load, "Reload zone [Ctrl Shift O]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(workspace::HEADER),
                bottom: px(0),
                width: px(PANEL),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.05, 0.075)),
        ))
        .with_children(|panel| {
            panel.spawn(text("TOOLS", 22.));
            panel.spawn(text(
                "Choose a workspace above.\nSettings are in Properties.",
                13.,
            ));
            for (index, (action, title)) in object_actions().into_iter().enumerate() {
                panel
                    .spawn((
                        PaletteSlot(index),
                        action,
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        Node {
                            min_height: px(26),
                            padding: UiRect::all(px(4)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.10, 0.16, 0.22)),
                    ))
                    .with_children(|button| {
                        button.spawn((PaletteLabel, text(title, 15.)));
                    });
            }
            panel.spawn((
                Help,
                text("", 13.),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
    commands.spawn((
        Summary,
        text("", 14.),
        TextLayout::no_wrap(),
        Node {
            position_type: PositionType::Absolute,
            left: px(PANEL + 12.),
            bottom: px(8),
            right: px(outliner::WIDTH + 12.),
            height: px(44),
            padding: UiRect::all(px(4)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.05, 0.075, 0.9)),
    ));
}
fn activate(
    event: On<Activate>,
    actions: Query<&Action>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
) {
    if !session.has_modal()
        && windows.single().is_ok_and(|w| w.focused)
        && let Ok(action) = actions.get(event.entity)
    {
        perform(*action, &mut session, &mut view);
    }
}
fn perform(action: Action, session: &mut Session, view: &mut View) {
    if session.has_modal() {
        return;
    }
    if matches!(action, Action::Duplicate) && (session.drag.is_some() || session.stroke.is_some()) {
        session.status = "Finish or cancel the preview before duplicating".into();
        return;
    }
    session.drag = None;
    session.stroke = None;
    if !matches!(
        action,
        Action::NewPath(_)
            | Action::FinishPath
            | Action::GradePath
            | Action::Ground
            | Action::Duplicate
            | Action::PopPath
            | Action::InsertPoint
            | Action::RemovePoint
            | Action::NextPath
            | Action::PathWidth(_)
            | Action::Save
            | Action::Snap
    ) {
        session.path_draft = None;
    }
    if matches!(action, Action::Play) {
        if session.play.take().is_some() {
            session.status = "Returned to editing".into();
        } else {
            match playtest::Playtest::new(session.editor.zone()) {
                Ok(play) => {
                    session.play = Some(play);
                    session.heading = 0.;
                    session.status = "Offline playtest - F5 returns to editing".into();
                }
                Err(error) => session.status = error,
            }
        }
        return;
    }
    if session.play.is_some() {
        session.status = "Leave playtest before editing".into();
        return;
    }
    match action {
        Action::Duplicate => selection::duplicate(session),
        Action::Select => {
            session.palette = None;
            session.marker_palette = None;
            session.plant_palette = None;
        }
        Action::SelectPlant | Action::Plant(_, _) | Action::NextPlant | Action::DeletePlant => {
            vegetation::perform(action, session)
        }
        Action::SelectMarker
        | Action::PlaceMarker(_)
        | Action::NextMarker
        | Action::DeleteMarker
        | Action::MarkerRadius(_) => markers::perform(action, session),
        Action::Place(primitive) => {
            session.terrain_mode = false;
            session.terrain_paint = false;
            session.path_mode = false;
            session.marker_mode = false;
            session.vegetation_mode = false;
            session.palette = Some(primitive);
            session.status = "Click the ground to place".into();
        }
        Action::Ground => {
            use theia_zone::projection::{Projection, Target};
            if session.path_draft.is_some() {
                session.status = "Finish or cancel the guide draft before grounding".into();
                return;
            }
            let targets = if session.path_mode {
                session
                    .path_selected
                    .and_then(|id| {
                        session
                            .editor
                            .zone()
                            .paths
                            .iter()
                            .find(|p| p.id == id)
                            .map(|p| match session.path_point {
                                Some(index) => vec![Target::GuidePoint { id, index }],
                                None => (0..p.points.len())
                                    .map(|index| Target::GuidePoint { id, index })
                                    .collect(),
                            })
                    })
                    .unwrap_or_default()
            } else {
                let id = if session.vegetation_mode {
                    session.plant_selected
                } else if session.marker_mode {
                    session.marker_selected
                } else if !session.terrain_mode {
                    session.selected
                } else {
                    None
                };
                id.map(|id| vec![Target::Placement { id }])
                    .unwrap_or_default()
            };
            session.edit(vec![Edit::ProjectToSurface {
                projection: Projection {
                    targets,
                    offset_metres: if session.path_mode { 0.03 } else { 0. },
                },
            }]);
        }
        Action::Snap => session.snap = !session.snap,
        Action::Undo => session.execute(Command::Undo {
            expected_revision: session.editor.revision(),
        }),
        Action::Redo => session.execute(Command::Redo {
            expected_revision: session.editor.revision(),
        }),
        Action::Save => {
            session.status = match session.editor.zone().save(&session.path) {
                Ok(()) => {
                    session.saved_zone = Some(session.editor.zone().clone());
                    format!(
                        "Saved {}{}",
                        session.path.display(),
                        if session.path_draft.is_some() {
                            " (guide draft still pending)"
                        } else {
                            ""
                        }
                    )
                }
                Err(e) => e,
            };
        }
        Action::Load => {
            session.status = match Zone::load(&session.path)
                .and_then(|zone| session.editor.replace(zone, session.editor.revision()))
            {
                Ok(()) => {
                    session.selected = None;
                    session.path_selected = None;
                    session.path_point = None;
                    session.marker_selected = None;
                    session.plant_selected = None;
                    session.reconcile();
                    session.saved_zone = Some(session.editor.zone().clone());
                    "Loaded zone (undo restores prior document)".into()
                }
                Err(e) => e,
            };
        }
        Action::Delete => {
            if let Some(id) = session.selected {
                session.edit(vec![Edit::Delete { id }]);
            }
        }
        Action::Spawn => {
            match spawn::position(session.editor.zone(), view.target, false, session.move_step) {
                Ok(position) => session.edit(vec![Edit::SetSpawn { position }]),
                Err(error) => session.status = error,
            }
        }
        Action::NewPath(_)
        | Action::FinishPath
        | Action::GradePath
        | Action::PopPath
        | Action::InsertPoint
        | Action::RemovePoint
        | Action::NextPath
        | Action::DeletePath
        | Action::PathWidth(_) => paths::perform(action, session),
        Action::TerrainResolution => {
            if let Some(patch) = &session.editor.zone().terrain {
                let resolution = if patch.resolution == theia_zone::terrain::Resolution::FourMetres
                {
                    theia_zone::terrain::Resolution::TwoMetres
                } else {
                    theia_zone::terrain::Resolution::FourMetres
                };
                session.edit(vec![Edit::ResampleTerrain { resolution }]);
            } else {
                session.status = "Create terrain before changing sample resolution".into();
            }
        }
        Action::TerrainToggle => {
            let creating = session.editor.zone().terrain.is_none();
            terrain::toggle(session);
            if creating && let Some(patch) = &session.editor.zone().terrain {
                let start = patch.origin().unwrap();
                let offset = session
                    .editor
                    .zone()
                    .local([start[0], session.editor.zone().origin[1], start[2]])
                    .unwrap();
                view.target = Vec3::new(offset[0] + 64., 0., offset[2] + 64.);
                view.distance = 120.;
            }
        }
        Action::Brush(tool) => session.brush_tool = tool,
        Action::Radius(direction) => {
            session.brush_radius = (session.brush_radius + f64::from(direction) * 4.).clamp(4., 32.)
        }
        Action::BrushStrength(direction) => {
            session.brush_strength =
                (session.brush_strength * if direction > 0 { 2. } else { 0.5 }).clamp(0.05, 8.);
            session.status = format!(
                "Raise/lower strength {:.3} m per stamp",
                session.brush_strength
            );
        }
        Action::BrushBlend(direction) => {
            let paint = matches!(session.brush_tool, terrain::Tool::Paint(_));
            let blend = if paint {
                &mut session.paint_blend
            } else {
                &mut session.sculpt_blend
            };
            *blend = (*blend * if direction > 0 { 2. } else { 0.5 }).clamp(0.05, 1.);
            session.status = format!(
                "{} blend {:.2}%",
                if paint { "Paint" } else { "Flatten/smooth" },
                *blend * 100.
            );
        }
        Action::FlattenHeight(direction) => {
            let increment = if session.snap { session.move_step } else { 0.1 };
            session.flatten_height =
                (session.flatten_height + f64::from(direction) * increment).clamp(-64., 64.);
            session.status = format!(
                "Flatten level {:.2} m relative to zone origin",
                session.flatten_height
            );
        }
        Action::Play => unreachable!(),
    }
}
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
) {
    if session.has_modal() {
        return;
    }
    if !windows.single().is_ok_and(|w| w.focused) {
        session.drag = None;
        session.stroke = None;
        session.path_draft = None;
        return;
    }
    if (keys.just_pressed(KeyCode::F1)
        || keys.just_pressed(KeyCode::F2)
        || keys.just_pressed(KeyCode::F3)
        || keys.just_pressed(KeyCode::F4)
        || keys.just_pressed(KeyCode::F6)
        || keys.just_pressed(KeyCode::F7))
        && session.play.is_none()
    {
        let mode = if keys.just_pressed(KeyCode::F2) {
            EditorMode::Sculpt
        } else if keys.just_pressed(KeyCode::F7) {
            EditorMode::Paint
        } else if keys.just_pressed(KeyCode::F3) {
            EditorMode::Guides
        } else if keys.just_pressed(KeyCode::F4) {
            EditorMode::Markers
        } else if keys.just_pressed(KeyCode::F6) {
            EditorMode::Plants
        } else {
            EditorMode::Objects
        };
        session.set_mode(mode);
        return;
    }
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let action = if keys.just_pressed(KeyCode::F5) {
        Some(Action::Play)
    } else if keys.just_pressed(KeyCode::Escape) {
        Some(Action::Select)
    } else if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        Some(Action::Undo)
    } else if ctrl && keys.just_pressed(KeyCode::KeyY) {
        Some(Action::Redo)
    } else if ctrl
        && !keys.pressed(KeyCode::ShiftLeft)
        && !keys.pressed(KeyCode::ShiftRight)
        && keys.just_pressed(KeyCode::KeyS)
    {
        Some(Action::Save)
    } else if ctrl && keys.just_pressed(KeyCode::KeyD) {
        Some(Action::Duplicate)
    } else if ctrl
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
        && keys.just_pressed(KeyCode::KeyO)
    {
        Some(Action::Load)
    } else if !ctrl
        && session.terrain_mode
        && !session.terrain_paint
        && session.brush_tool == terrain::Tool::Flatten
        && keys.just_pressed(KeyCode::PageUp)
    {
        Some(Action::FlattenHeight(1))
    } else if !ctrl
        && session.terrain_mode
        && !session.terrain_paint
        && session.brush_tool == terrain::Tool::Flatten
        && keys.just_pressed(KeyCode::PageDown)
    {
        Some(Action::FlattenHeight(-1))
    } else if !ctrl && session.terrain_mode && keys.just_pressed(KeyCode::BracketLeft) {
        Some(
            if matches!(
                session.brush_tool,
                terrain::Tool::Raise | terrain::Tool::Lower
            ) {
                Action::BrushStrength(-1)
            } else {
                Action::BrushBlend(-1)
            },
        )
    } else if !ctrl && session.terrain_mode && keys.just_pressed(KeyCode::BracketRight) {
        Some(
            if matches!(
                session.brush_tool,
                terrain::Tool::Raise | terrain::Tool::Lower
            ) {
                Action::BrushStrength(1)
            } else {
                Action::BrushBlend(1)
            },
        )
    } else if session.marker_mode && keys.just_pressed(KeyCode::Equal) {
        Some(Action::MarkerRadius(1))
    } else if session.marker_mode && keys.just_pressed(KeyCode::Minus) {
        Some(Action::MarkerRadius(-1))
    } else if session.path_mode && keys.just_pressed(KeyCode::Enter) {
        Some(Action::FinishPath)
    } else if session.path_mode && !ctrl && keys.just_pressed(KeyCode::Insert) {
        Some(Action::InsertPoint)
    } else if session.path_mode && keys.just_pressed(KeyCode::Backspace) {
        Some(if session.path_draft.is_some() {
            Action::PopPath
        } else {
            Action::RemovePoint
        })
    } else if session.path_mode && keys.just_pressed(KeyCode::Equal) {
        Some(Action::PathWidth(1))
    } else if session.path_mode && keys.just_pressed(KeyCode::Minus) {
        Some(Action::PathWidth(-1))
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(Action::Place(Primitive::Box))
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(Action::Place(Primitive::Sphere))
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(Action::Place(Primitive::Cylinder))
    } else if keys.just_pressed(KeyCode::KeyH) {
        Some(Action::Ground)
    } else if keys.just_pressed(KeyCode::KeyG) {
        Some(Action::Snap)
    } else if keys.just_pressed(KeyCode::Delete) {
        Some(if session.vegetation_mode {
            Action::DeletePlant
        } else if session.marker_mode {
            Action::DeleteMarker
        } else if session.path_mode {
            Action::DeletePath
        } else {
            Action::Delete
        })
    } else {
        None
    };
    if let Some(action) = action {
        perform(action, &mut session, &mut view);
        return;
    }
    if selection::keyboard(&keys, &mut session, &mut view) {
        return;
    }
    if session.vegetation_mode {
        vegetation::keyboard(&keys, &mut session);
        return;
    }
    if session.marker_mode {
        markers::transform(&keys, &mut session);
        return;
    }
    if session.path_mode {
        paths::translate(&keys, &mut session, &mut view);
        return;
    }
    if session.play.is_some()
        || session.drag.is_some()
        || session.stroke.is_some()
        || session.terrain_mode
        || ctrl
    {
        return;
    }
    let Some(object) = session
        .editor
        .zone()
        .objects
        .iter()
        .find(|o| Some(o.id) == session.selected)
    else {
        return;
    };
    let id = object.id;
    let mut pose = object.pose.clone();
    let increment = if session.snap { session.move_step } else { 0.1 };
    for (key, axis, sign) in [
        (KeyCode::ArrowLeft, 0, -1.),
        (KeyCode::ArrowRight, 0, 1.),
        (KeyCode::ArrowUp, 2, -1.),
        (KeyCode::ArrowDown, 2, 1.),
        (KeyCode::PageUp, 1, 1.),
        (KeyCode::PageDown, 1, -1.),
    ] {
        if keys.just_pressed(key) {
            pose.position[axis] += increment * sign;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        pose.yaw_degrees += if keys.pressed(KeyCode::ShiftLeft) {
            -15.
        } else {
            15.
        };
    }
    if keys.just_pressed(KeyCode::Equal) {
        pose.size = pose.size.map(|v| v * 1.25);
    }
    if keys.just_pressed(KeyCode::Minus) {
        pose.size = pose.size.map(|v| v / 1.25);
    }
    if pose != object.pose {
        session.edit(vec![Edit::Transform { id, pose }]);
    }
}
#[allow(clippy::too_many_arguments)]
fn camera_controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    windows: Query<&Window>,
    session: Res<Session>,
    mut view: ResMut<View>,
    mut panning: Local<Option<Vec2>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let enabled =
        window.focused && session.play.is_none() && !session.has_modal() && session.drag.is_none();
    let cursor = canvas_cursor(window);
    let outside = cursor.is_some();
    if !enabled
        || !outside
        || !(mouse.pressed(MouseButton::Middle) || mouse.just_released(MouseButton::Middle))
        || mouse.pressed(MouseButton::Left)
        || mouse.pressed(MouseButton::Right)
        || keys.just_pressed(KeyCode::Escape)
    {
        *panning = None;
    } else if mouse.just_pressed(MouseButton::Middle) {
        *panning = cursor;
    } else if let (Some(previous), Some(current)) = (*panning, cursor) {
        camera_views::pan(&mut view, current - previous, window.height());
        *panning = Some(current);
    }
    if mouse.just_released(MouseButton::Middle) {
        *panning = None;
    }
    if enabled {
        if outside && mouse.pressed(MouseButton::Right) && motion.delta != Vec2::ZERO {
            view.preset = camera_views::Preset::Perspective;
            view.yaw -= motion.delta.x * 0.005;
            view.pitch = (view.pitch + motion.delta.y * 0.005).clamp(0.15, 1.5);
        }
        if outside {
            view.distance = (view.distance * (-scroll.delta.y * 0.1).exp())
                .clamp(3., selection::MAX_VIEW_DISTANCE);
        }
        if !keys.pressed(KeyCode::ControlLeft) && !keys.pressed(KeyCode::ControlRight) {
            let (right, up) = view.preset.pan_axes();
            let horizontal = (i32::from(keys.pressed(KeyCode::KeyD))
                - i32::from(keys.pressed(KeyCode::KeyA))) as f32;
            let vertical = (i32::from(keys.pressed(KeyCode::KeyW))
                - i32::from(keys.pressed(KeyCode::KeyS))) as f32;
            let axis = right * horizontal + up * vertical;
            view.target =
                (view.target + axis.normalize_or_zero() * time.delta_secs().min(0.1) * 16.).clamp(
                    Vec3::splat(-theia_zone::ZONE_EXTENT as f32),
                    Vec3::splat(theia_zone::ZONE_EXTENT as f32),
                );
        }
    }
    if let Ok((mut transform, mut projection)) = camera.single_mut() {
        camera_views::projection(&view, session.play.is_some(), &mut projection);
        if session.play.is_none() {
            *transform = camera_views::transform(&view);
            return;
        }
        let (target, yaw, pitch, distance) = if let Some(play) = &session.play {
            (
                Vec3::from_array(play.feet.metres()) + Vec3::Y,
                session.heading,
                0.45,
                8.,
            )
        } else {
            (view.target, view.yaw, view.pitch, view.distance)
        };
        let offset = Vec3::new(
            yaw.sin() * pitch.cos(),
            pitch.sin(),
            yaw.cos() * pitch.cos(),
        ) * distance;
        *transform = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    }
}
fn present(
    session: Res<Session>,
    mut windows: Query<&mut Window>,
    mut summaries: Query<&mut Text, With<Summary>>,
    mut info: Query<&mut Text, (With<Info>, Without<Summary>)>,
) {
    let dirty = session.is_dirty();
    let path = session.path.to_string_lossy();
    let safe_path: String = path
        .chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect();
    let suffix: String = safe_path
        .chars()
        .rev()
        .take(180)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let display_path = if safe_path.chars().count() > 180 {
        format!("…{suffix}")
    } else {
        suffix
    };
    for mut text in &mut summaries {
        **text = format!(
            "{}  |  {}\n{}",
            if dirty { "Unsaved" } else { "Saved" },
            display_path,
            session.status
        );
    }
    if let Ok(mut window) = windows.single_mut() {
        let filename = session
            .path
            .file_name()
            .unwrap_or(session.path.as_os_str())
            .to_string_lossy();
        let title = format!(
            "{}{} — {} — Theia Zone Studio",
            if dirty { "* " } else { "" },
            session.editor.zone().name,
            filename
        );
        if window.title != title {
            window.title = title;
        }
    }
    let selected = session
        .editor
        .zone()
        .objects
        .iter()
        .find(|o| Some(o.id) == session.selected);
    for mut text in &mut info {
        **text = format!(
            "{}\n{}\n{}",
            session.editor.zone().name,
            if session.is_dirty() {
                "Unsaved"
            } else {
                "Saved"
            },
            if session.vegetation_mode {
                vegetation::summary(&session)
            } else if session.marker_mode {
                markers::summary(&session)
            } else if session.path_mode {
                paths::summary(&session)
            } else if session.terrain_mode {
                format!(
                    "Terrain {:?}\n{}{}",
                    session.brush_tool,
                    session.editor.zone().terrain.as_ref().map_or(
                        "No terrain patch".into(),
                        |p| format!("128 m patch | {} m samples", p.sample_metres())
                    ),
                    session.stroke.as_ref().map_or(String::new(), |s| format!(
                        "\nStroke {}/{} stamps",
                        s.stamps(),
                        theia_zone::MAX_EDITS
                    ))
                )
            } else {
                selected.map_or("No selection".into(), |o| {
                    let pose = session.drag.as_ref().filter(|d|d.id==o.id).map_or(&o.pose,|d|&d.preview);
                    format!("{}\nPosition (world metres)\n X {:.3}\n Y {:.3}\n Z {:.3}\nYaw {:.2} degrees\nSize (metres)\n X {:.3}  Y {:.3}  Z {:.3}",
                        o.name,pose.position[0],pose.position[1],pose.position[2],pose.yaw_degrees,
                        pose.size[0],pose.size[1],pose.size[2])
                })
            }
        );
    }
}
