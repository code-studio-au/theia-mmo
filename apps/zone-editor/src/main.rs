//! P5-17/P3-15: standard Bevy greybox authoring; no trusted gameplay authority.
mod account_picker;
mod agent;
mod asset_browser;
mod asset_preview;
mod authoring_grid;
mod browser_icons;
mod brush_preview;
mod camera_views;
#[path = "../../common/classic_controls.rs"]
pub mod classic_controls;
mod cli;
#[path = "../../common/creature_asset.rs"]
mod creature_asset;
mod exit_dialog;
mod export;
mod export_dialog;
mod file_picker;
mod gameplay_export;
mod gizmo;
#[cfg(test)]
mod headless_ui;
mod markers;
mod menus;
mod naming;
mod network_playtest;
mod network_test_export;
mod outliner;
mod panels;
mod particle_anchors;
mod particle_export;
mod particle_inline;
mod particle_preview;
mod particle_texture_draft;
mod particle_workspace;
mod paths;
mod patrol_export;
mod placement;
#[path = "../../common/orbit.rs"]
pub mod play_orbit;
mod playtest;
mod playtest_avatar;
mod playtest_controls;
mod population_export;
mod properties;
mod property_tabs;
mod quest_area_authoring;
mod quest_area_export;
mod quest_area_preview;
mod quest_inspector;
mod scenery_collision;
mod scenery_export;
mod scenery_glb;
#[cfg(test)]
mod scenery_gpu_test;
mod scenery_library;
mod scenery_scene;
mod scrolling;
mod sections;
mod selection;
mod sidebar;
mod snapping;
mod spawn;
mod spawn_groups;
mod status_bar;
mod terrain;
mod test_players;
mod theme;
mod transform_target;
#[cfg(test)]
mod valley_gpu_test;
mod vegetation;
mod viewport;
mod viewport_tools;
mod workspace;
use crate::particle_workspace::ZoneAppExt;
use bevy::picking::hover::Hovered;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::path::PathBuf;
use theia_zone::{Command, Edit, Editor, Object, Pose, Primitive, Zone};

const CANVAS_LEFT: f32 = sidebar::WIDTH + workspace::INSPECTOR_WIDTH;
fn canvas_cursor(window: &Window, session: &Session) -> Option<Vec2> {
    window
        .cursor_position()
        .filter(|p| {
            !session.ui.input_owned
                && !viewport_tools::blocks(window, session)
                && !session.inline_editing
                && p.x > session.ui.canvas_left() + 6.
                && p.x < window.width()
                && p.y > workspace::HEADER
                && p.y
                    < window.height()
                        - session
                            .ui
                            .canvas_bottom(window.height(), session.play.is_none())
        })
        .map(|p| session.ui.window_to_viewport(p, window.scale_factor()))
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
    ui: panels::Layout,
    assets: asset_browser::State,
    editor: Editor,
    saved_zone: Option<Zone>,
    selected: Option<u64>,
    particle_selected: Option<u64>,
    quest_area_selected: Option<u64>,
    spawn_group_selected: Option<u64>,
    spawn_boundary_point: Option<usize>,
    scenery_selected: Option<u64>,
    test_player_selected: Option<u64>,
    scenery_ground_requested: Option<(u64, u64, u64)>,
    scenery_play_requested: Option<(u64, u64)>,
    inspect_spawn: bool,
    palette: Option<Primitive>,
    placement_size: [f64; 3],
    placement_preview: Option<placement::Preview>,
    snap: bool,
    move_step: f64,
    show_move_handles: bool,
    gizmo_tool: gizmo::Tool,
    status: String,
    path: Option<PathBuf>,
    last_directory: PathBuf,
    save_as_requested: bool,
    next_id: u64,
    drag: Option<Drag>,
    play: Option<playtest::Playtest>,
    player_model: bool,
    network_active: bool,
    account_picker_open: bool,
    player_animation: playtest_avatar::Preview,
    player_clips: Vec<String>,
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
    document_id: u64,
    inline_editing: bool,
    inline_dirty: bool,
    inline_action: Option<Action>,
    particle_save_requested: bool,
    exit_confirmation: bool,
    file_dialog: Option<file_picker::Draft>,
    export_dialog: Option<export_dialog::Draft>,
    quest_context: Option<quest_inspector::Context>,
    creature_context: Option<spawn_groups::Context>,
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
        let last_directory = path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        static DOCUMENT_IDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Session {
            document_id: DOCUMENT_IDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            ui: panels::Layout::default(),
            assets: asset_browser::State::default(),
            editor,
            saved_zone,
            selected: None,
            particle_selected: None,
            quest_area_selected: None,
            spawn_group_selected: None,
            spawn_boundary_point: None,
            scenery_selected: None,
            test_player_selected: None,
            scenery_ground_requested: None,
            scenery_play_requested: None,
            inspect_spawn: false,
            palette: None,
            placement_size: [1.; 3],
            placement_preview: None,
            snap: true,
            move_step: 1.,
            show_move_handles: true,
            gizmo_tool: gizmo::Tool::Move,
            status: "Ready - choose a primitive, then click the ground".into(),
            path: Some(path),
            last_directory,
            save_as_requested: false,
            next_id,
            drag: None,
            play: None,
            network_active: false,
            account_picker_open: false,
            player_model: true,
            player_animation: playtest_avatar::Preview::Auto,
            player_clips: vec![],
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
            inline_editing: false,
            inline_dirty: false,
            inline_action: None,
            particle_save_requested: false,
            exit_confirmation: false,
            file_dialog: None,
            export_dialog: None,
            quest_context: None,
            creature_context: None,
        }
    }
    fn placement_dimensions(&self, primitive: Primitive) -> [f64; 3] {
        let [x, y, z] = self.placement_size;
        match primitive {
            Primitive::Box => [x, y, z],
            Primitive::Sphere => [x; 3],
            Primitive::Cylinder => [x, y, x],
        }
    }
    fn place_primitive(&mut self, primitive: Primitive, hit: [f64; 3]) {
        let pose = match placement::pose(self, primitive, hit) {
            Ok(pose) => pose,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let object = placement::object(self, primitive, pose);
        let id = object.id;
        let Some(next) = id.checked_add(1) else {
            self.status = "Object IDs exhausted".into();
            return;
        };
        self.next_id = next;
        self.edit(vec![Edit::Place { object }]);
        if self.editor.zone().objects.iter().any(|o| o.id == id) {
            self.selected = Some(id);
        }
    }
    fn path_label(&self) -> std::borrow::Cow<'_, str> {
        self.path
            .as_ref()
            .map_or_else(|| "Untitled".into(), |path| path.to_string_lossy())
    }
    fn picker_path(&self) -> PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| self.last_directory.join("untitled.zone.json"))
    }
    fn assign_path(&mut self, path: PathBuf) {
        self.last_directory = path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        self.path = Some(path);
        self.save_as_requested = false;
    }
    fn new_document(&mut self, view: &mut View) -> Result<bool, String> {
        if self.has_modal()
            || self.play.is_some()
            || self.drag.is_some()
            || self.stroke.is_some()
            || self.path_draft.is_some()
        {
            return Err(
                "Finish or cancel the preview / form and leave playtest before creating a zone"
                    .into(),
            );
        }
        let zone = Zone::default();
        if self.path.is_none() && self.editor.zone() == &zone {
            self.status = "Already an empty untitled zone".into();
            return Ok(false);
        }
        self.editor.replace(zone, self.editor.revision())?;
        self.path = None;
        self.saved_zone = None;
        self.save_as_requested = false;
        self.set_mode(EditorMode::Objects);
        self.reconcile();
        view.target = Vec3::ZERO;
        view.distance = 32.;
        self.status =
            "New untitled zone - Save chooses a filename; Undo restores prior content".into();
        Ok(true)
    }
    fn has_modal(&self) -> bool {
        self.name_edit.is_some()
            || self.transform_edit.is_some()
            || self.file_dialog.is_some()
            || self.export_dialog.is_some()
            || self.exit_confirmation
            || self.account_picker_open
    }
    fn set_mode(&mut self, mode: EditorMode) {
        self.inspect_spawn = false;
        self.placement_preview = None;
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
        self.particle_selected = None;
        self.quest_area_selected = None;
        self.spawn_group_selected = None;
        self.spawn_boundary_point = None;
        self.scenery_selected = None;
        self.test_player_selected = None;
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
            || self.inline_dirty
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
        if !self
            .editor
            .zone()
            .particles
            .iter()
            .any(|p| Some(p.id) == self.particle_selected)
        {
            self.particle_selected = None;
        }
        if !self
            .editor
            .zone()
            .quest_areas
            .iter()
            .any(|a| Some(a.id) == self.quest_area_selected)
        {
            self.quest_area_selected = None;
        }
        if !self
            .editor
            .zone()
            .spawn_groups
            .iter()
            .any(|g| Some(g.id) == self.spawn_group_selected)
        {
            self.spawn_group_selected = None;
            self.spawn_boundary_point = None;
        }
        if !self
            .editor
            .zone()
            .scenery
            .iter()
            .any(|p| Some(p.id) == self.scenery_selected)
        {
            self.scenery_selected = None;
        }
        if !self
            .editor
            .zone()
            .test_players
            .iter()
            .any(|p| Some(p.id) == self.test_player_selected)
        {
            self.test_player_selected = None;
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
    AddParticle,
    AddQuestArea,
    AddSpawnGroup,
    ToggleSpawnGroup,
    BindSpawnCreature,
    SpawnBoundaryPoint(i8),
    SpawnBoundaryWhole,
    SpawnBoundaryInsert,
    SpawnBoundaryRemove,
    AddTestPlayer,
    TestPlayerEnabled,
    TestPlayerController(bool),
    TestPlayerRoute(i8),
    CreateStarterBasin,
    CreateStarterValley,
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
struct PaletteShortcut;
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
    // Validate context before prepare can write an explicitly requested output.
    let creature_context = options
        .creature_definition
        .as_deref()
        .map(spawn_groups::Context::load)
        .transpose()?;
    let editor = options.prepare()?;
    let scenery_library = options
        .scenery_library
        .as_deref()
        .map(scenery_library::Library::load)
        .transpose()?
        .unwrap_or_default();
    if options.check_scenery {
        println!("{}", scenery_library.check_headless()?);
        return Ok(());
    }
    if options.check_playtest {
        let checked = playtest::Playtest::with_library(editor.zone(), &scenery_library)
            .and_then(|play| editor.zone().global(play.feet.metres()));
        let mut report = cli::summary(&editor);
        report["playtest"] = serde_json::json!({
            "ready":checked.is_ok(),"requested_spawn":editor.zone().spawn,
            "resolved_spawn":checked.as_ref().ok(),"error":checked.as_ref().err(),
        });
        println!("{report}");
        return checked.map(|_| ());
    }
    if let Some(path) = &options.export_quest_areas {
        let report = quest_area_export::publish(
            editor.zone(),
            options
                .quest_catalog
                .as_deref()
                .expect("Checked quest catalog"),
            path,
            options.content_id.as_deref().expect("Checked catalog ID"),
            options.revision.expect("Checked catalog revision"),
        )?;
        println!("{report}");
        return Ok(());
    }
    if let Some(path) = &options.export {
        let report = if options.population {
            let baker = options
                .navigation_baker
                .clone()
                .map(Ok)
                .unwrap_or_else(population_export::default_baker)?;
            let publish = if options.network_test {
                population_export::publish_network
            } else {
                population_export::publish
            };
            publish(
                editor.zone(),
                path,
                options.content_id.as_deref().expect("Checked export ID"),
                options.revision.expect("Checked export revision"),
                population_export::Settings {
                    creature: options
                        .population_creature
                        .as_deref()
                        .expect("Checked creature profile"),
                    baker: &baker,
                    spacing: options.population_spacing.unwrap_or(3.),
                    appearance: options.creature_asset.as_deref(),
                    gameplay: options.gameplay_template.as_deref(),
                    access_path: options.population_access_path,
                },
                &scenery_library,
            )?
        } else if options.network_test {
            network_test_export::publish(
                editor.zone(),
                path,
                options.content_id.as_deref().expect("Checked export ID"),
                options.revision.expect("Checked export revision"),
                options.gameplay_template.as_deref(),
                &scenery_library,
            )?
        } else {
            export::publish_with_assets(
                editor.zone(),
                path,
                options.content_id.as_deref().expect("Checked export ID"),
                options.revision.expect("Checked export revision"),
                options.gameplay_template.as_deref(),
                &scenery_library,
            )?
        };
        println!(
            "{}",
            serde_json::to_string(&report).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let quest_context = options
        .gameplay_template
        .as_deref()
        .map(quest_inspector::Context::load)
        .transpose()?;
    if options.headless {
        let mut report = cli::summary(&editor);
        if let Some(context) = &creature_context {
            report["creature_definition"] = context.summary();
        }
        println!("{report}");
        return Ok(());
    }
    let initial_target = camera_views::terrain_target(editor.zone());
    let path = options.editor_path();
    let saved_zone = Zone::load(&path).ok();
    let mut app = App::new();
    if let Some(directory) = &options.player_assets {
        playtest_avatar::install(&mut app, directory)?;
    }
    scenery_library.register(&mut app);
    app.insert_resource(scenery_library);
    let mut session = Session::new(editor, path, saved_zone);
    session.quest_context = quest_context;
    session.creature_context = creature_context;
    match options.capture_view {
        Some(cli::CaptureView::Spawn) => session.inspect_spawn = true,
        Some(cli::CaptureView::TestPlayer) => {
            session.test_player_selected = Some(
                session
                    .editor
                    .zone()
                    .test_players
                    .first()
                    .ok_or("--capture-view test-player requires a test player")?
                    .id,
            )
        }
        Some(cli::CaptureView::PropertyActions) => {
            session.selected = Some(
                session
                    .editor
                    .zone()
                    .objects
                    .first()
                    .ok_or("--capture-view property-actions requires an object")?
                    .id,
            );
        }
        Some(
            cli::CaptureView::Particle
            | cli::CaptureView::ParticlePreview
            | cli::CaptureView::ParticleBind
            | cli::CaptureView::ParticleLibrary,
        ) => {
            session.particle_selected = Some(
                session
                    .editor
                    .zone()
                    .particles
                    .first()
                    .ok_or("--capture-view particle requires a particle reference")?
                    .id,
            );
        }
        None => {}
    }
    app.insert_resource(session);
    app.insert_resource(View {
        target: initial_target.unwrap_or(Vec3::ZERO),
        yaw: 0.6,
        pitch: 0.8,
        show_grid: true,
        distance: if initial_target.is_some() { 120. } else { 32. },
        preset: camera_views::Preset::Perspective,
    })
    .insert_resource(network_playtest::Manager::new(
        options.player_assets.clone(),
        options.creature_asset.clone(),
    ))
    .add_systems(Update, network_playtest::poll)
    .init_resource::<test_players::Scene>()
    .init_resource::<scenery_scene::Scene>()
    .insert_resource(Time::<Fixed>::from_hz(30.))
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Theia - Zone Editor".into(),
                    resolution: (3440, 1440).into(),
                    mode: bevy::window::WindowMode::BorderlessFullscreen(
                        bevy::window::MonitorSelection::Primary,
                    ),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(theme::plugin)
    .add_plugins(sidebar::plugin)
    .add_plugins(property_tabs::plugin)
    .add_plugins(viewport_tools::plugin)
    .add_plugins(browser_icons::plugin)
    .add_plugins(asset_browser::plugin)
    .add_plugins(scrolling::plugin)
    .add_plugins(spawn_groups::plugin)
    .add_plugins(properties::summary_plugin)
    .add_plugins(menus::plugin)
    .add_plugins(particle_workspace::plugin)
    .add_plugins(particle_inline::plugin)
    .add_plugins(exit_dialog::plugin)
    .add_plugins(account_picker::plugin)
    .init_resource::<particle_preview::State>()
    .init_resource::<particle_preview::LibraryRoot>()
    .add_observer(workspace::activate)
    .add_observer(workspace::change)
    .add_observer(workspace::toggle)
    .add_observer(activate)
    .add_observer(outliner::activate)
    .add_observer(naming::activate)
    .add_observer(properties::activate)
    .add_observer(properties::slider_change)
    .add_observer(file_picker::activate)
    .add_observer(export_dialog::activate)
    .add_observer(particle_preview::relay_toast)
    .add_observer(particle_preview::respawn_emitters)
    .add_observer(particle_preview::respawn_colliders)
    .add_systems(
        Startup,
        (
            viewport::setup,
            placement::setup.after(viewport::setup),
            authoring_grid::setup,
            gizmo::setup,
            terrain::setup,
            paths::setup,
            markers::setup,
            particle_anchors::setup,
            quest_area_preview::setup,
            vegetation::setup,
            setup_ui,
            workspace::setup,
            panels::setup,
            outliner::setup,
            naming::setup,
            properties::setup,
            file_picker::setup,
            export_dialog::setup,
        ),
    )
    .add_zone_systems(
        (
            (
                panels::resize,
                panels::apply,
                agent::poll,
                file_picker::keyboard,
                export_dialog::open,
            )
                .chain(),
            naming::keyboard,
            (properties::scrub, properties::keyboard).chain(),
            keyboard,
            vegetation::pointer,
            markers::pointer,
            paths::pointer,
            terrain::pointer,
            viewport::pointer,
            (playtest_controls::input, camera_controls)
                .chain()
                .after(keyboard),
            terrain::sync,
            terrain::palette,
            (
                viewport::sync,
                gizmo::sync,
                authoring_grid::sync,
                placement::sync,
            )
                .chain(),
            paths::sync,
            (
                markers::sync,
                particle_anchors::sync,
                quest_area_preview::sync,
                scenery_scene::add_requested,
                scenery_scene::sync,
                test_players::sync,
            )
                .chain(),
            vegetation::sync,
            (outliner::scroll, outliner::sync).chain(),
            workspace::sync,
            present,
            status_bar::sync,
        )
            .chain(),
    )
    .add_systems(
        Update,
        (particle_preview::sync, particle_preview::playback)
            .chain()
            .after(particle_workspace::ZoneAuthoring),
    )
    .add_systems(
        PostUpdate,
        (
            naming::finish,
            (properties::finish, properties::slider_sync).chain(),
            file_picker::finish,
            export_dialog::finish,
        )
            .chain()
            .after(bevy::text::EditableTextSystems),
    )
    .add_systems(
        Update,
        viewport::capture
            .after(particle_workspace::ZoneAuthoring)
            .after(bevy_sprinkles_editor::hosting::AuthoringUpdate),
    )
    .add_systems(FixedUpdate, playtest::step)
    .add_systems(Last, viewport::report);
    if let Some(path) = options.capture {
        app.insert_resource(viewport::Capture {
            path,
            frames: 0,
            view: options.capture_view,
        });
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
        theme::font(if size >= 18. {
            14.
        } else if size >= 14. {
            12.
        } else {
            11.
        }),
        TextColor(theme::BODY),
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
fn tool_caption(title: &str) -> (&str, &str) {
    title
        .strip_suffix(']')
        .and_then(|title| title.rsplit_once(" ["))
        .unwrap_or((title, ""))
}

fn setup_tools(panel: &mut ChildSpawnerCommands) {
    sections::spawn(panel, "Tools", |panel| {
        for (index, (action, title)) in object_actions().into_iter().enumerate() {
            panel
                .spawn((
                    PaletteSlot(index),
                    action,
                    Button,
                    ActivateOnPress,
                    Hovered::default(),
                    Node {
                        height: Val::Auto,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: px(8),
                        ..theme::button_node()
                    },
                    BackgroundColor(theme::Variant::Default.background(false)),
                ))
                .with_children(|button| {
                    let (label, shortcut) = tool_caption(title);
                    button.spawn((PaletteLabel, theme::label(label, 12., theme::BODY)));
                    button.spawn((
                        PaletteLabel,
                        PaletteShortcut,
                        theme::Secondary,
                        theme::label(shortcut, 10., theme::MUTED),
                        TextLayout::no_wrap(),
                        Node {
                            flex_shrink: 0.,
                            ..default()
                        },
                    ));
                });
        }
    });
    panel.spawn((
        Help,
        text("", 13.),
        Node {
            display: Display::None,
            ..default()
        },
    ));
}
fn setup_ui(mut commands: Commands) {
    status_bar::spawn(&mut commands);
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
    if session.inline_editing {
        session.inline_action = Some(action);
        return;
    }
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
    if matches!(action, Action::Play) && session.network_active {
        session.status = "Stop the network playtest before starting offline F5".into();
        return;
    }
    if matches!(action, Action::Play) {
        if session.play.take().is_some() {
            session.status = "Returned to editing".into();
        } else if !session.editor.zone().scenery.is_empty() {
            session.scenery_play_requested = Some((session.document_id, session.editor.revision()));
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
        Action::CreateStarterBasin | Action::CreateStarterValley => {
            let revision = session.editor.revision();
            session.edit(vec![if matches!(action, Action::CreateStarterValley) {
                Edit::CreateStarterValley
            } else {
                Edit::CreateStarterBasin
            }]);
            if session.editor.revision() != revision {
                session.set_mode(EditorMode::Sculpt);
                selection::frame(session, view);
                session.status =
                    "Starter terrain created · sculpt and paint to refine the zone".into();
            }
        }
        Action::AddTestPlayer => test_players::add(session),
        Action::TestPlayerEnabled
        | Action::TestPlayerController(_)
        | Action::TestPlayerRoute(_) => test_players::configure(action, session),
        Action::AddQuestArea => quest_area_authoring::add(session),
        Action::AddSpawnGroup => spawn_groups::add(session),
        Action::ToggleSpawnGroup => spawn_groups::toggle(session),
        Action::BindSpawnCreature => spawn_groups::bind_creature(session),
        Action::SpawnBoundaryPoint(direction) => spawn_groups::boundary_action(
            session,
            workspace::Control::SpawnBoundaryPoint(direction),
        ),
        Action::SpawnBoundaryWhole => {
            spawn_groups::boundary_action(session, workspace::Control::SpawnBoundaryWhole)
        }
        Action::SpawnBoundaryInsert => {
            spawn_groups::boundary_action(session, workspace::Control::SpawnBoundaryInsert)
        }
        Action::SpawnBoundaryRemove => {
            spawn_groups::boundary_action(session, workspace::Control::SpawnBoundaryRemove)
        }
        Action::AddParticle => {
            let id = session.next_id;
            let Some(next) = id.checked_add(1) else {
                session.status = "Authoring IDs exhausted".into();
                return;
            };
            let revision = session.editor.revision();
            session.edit(vec![Edit::PutParticle {
                particle: theia_zone::particles::ParticlePlacement {
                    id,
                    name: format!("Particle {id}"),
                    effect: "vfx.unassigned".into(),
                    position: session.editor.zone().spawn,
                    yaw_degrees: 0.,
                    scale: 1.,
                },
            }]);
            if session.editor.revision() != revision {
                session.next_id = next;
                session.set_mode(EditorMode::Objects);
                session.particle_selected = Some(id);
                session.status =
                    "Particle reference added at spawn · set Effect ID in Attributes".into();
            }
        }
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
            session.inspect_spawn = false;
            session.particle_selected = None;
            session.quest_area_selected = None;
            session.spawn_group_selected = None;
            session.scenery_selected = None;
            session.test_player_selected = None;
            session.terrain_mode = false;
            session.terrain_paint = false;
            session.path_mode = false;
            session.marker_mode = false;
            session.vegetation_mode = false;
            session.palette = Some(primitive);
            session.status = "Click the ground to place".into();
        }
        Action::Ground => {
            if let Some(id) = session.scenery_selected {
                session.scenery_ground_requested =
                    Some((session.document_id, session.editor.revision(), id));
                return;
            }
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
                    session
                        .selected
                        .or(session.particle_selected)
                        .or(session.test_player_selected)
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
            session.particle_save_requested = true;
            let Some(path) = session.path.clone() else {
                session.save_as_requested = true;
                session.status = "Choose a filename with Save As".into();
                return;
            };
            session.status = match session.editor.zone().save(&path) {
                Ok(()) => {
                    session.saved_zone = Some(session.editor.zone().clone());
                    format!(
                        "Saved {}{}",
                        path.display(),
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
            let Some(path) = session.path.clone() else {
                session.status = "Save the untitled zone before reloading it".into();
                return;
            };
            session.status = match Zone::load(&path)
                .and_then(|zone| session.editor.replace(zone, session.editor.revision()))
            {
                Ok(()) => {
                    session.selected = None;
                    session.particle_selected = None;
                    session.quest_area_selected = None;
                    session.spawn_group_selected = None;
                    session.scenery_selected = None;
                    session.test_player_selected = None;
                    session.inspect_spawn = false;
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
            if session.vegetation_mode {
                vegetation::perform(Action::DeletePlant, session);
            } else if session.marker_mode {
                markers::perform(Action::DeleteMarker, session);
            } else if session.path_mode {
                paths::perform(Action::DeletePath, session);
            } else if let Some(id) = session.selected {
                session.edit(vec![Edit::Delete { id }]);
            } else if let Some(id) = session.particle_selected {
                session.edit(vec![Edit::DeleteParticle { id }]);
            } else if let Some(id) = session.scenery_selected {
                session.edit(vec![Edit::DeleteScenery { id }]);
            } else if let Some(id) = session.test_player_selected {
                session.edit(vec![Edit::DeleteTestPlayer { id }]);
            } else if let Some(id) = session.spawn_group_selected {
                session.edit(vec![Edit::DeleteSpawnGroup { id }]);
            } else if let Some(id) = session.quest_area_selected {
                session.edit(vec![Edit::DeleteQuestArea { id }]);
            } else {
                session.status = "Select a placement to delete".into();
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
            if creating && let Some(target) = camera_views::terrain_target(session.editor.zone()) {
                view.target = target;
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
    mut content: ResMut<outliner::Content>,
    mut requests: ResMut<workspace::Requests>,
    text_inputs: Query<(), With<bevy::text::EditableText>>,
    focus: Res<bevy::input_focus::InputFocus>,
    menu_focus: Query<(), With<bevy::ui_widgets::MenuItem>>,
    sliders: Query<
        (),
        (
            Or<(
                With<bevy::ui_widgets::Slider>,
                With<bevy::ui_widgets::MenuButton>,
            )>,
            Without<bevy::ui::InteractionDisabled>,
        ),
    >,
) {
    if focus
        .get()
        .is_some_and(|entity| text_inputs.contains(entity))
        || session.has_modal()
        || session.inline_editing
        || session.ui.input_owned
        || focus
            .get()
            .is_some_and(|entity| menu_focus.contains(entity))
    {
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
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if ctrl && !shift && keys.just_pressed(KeyCode::KeyN) {
        workspace::new_document(&mut session, &mut view, &mut content);
        return;
    }
    if ctrl && !shift && keys.just_pressed(KeyCode::KeyF) {
        if workspace::busy(&session) {
            session.status = "Finish the preview or leave playtest before finding content".into();
        } else {
            requests.find = true;
        }
        return;
    }
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
    // Focused widgets consume their keys through InputDispatch, while the
    // editor also polls ButtonInput. Respect that ownership before authoring
    // shortcuts can turn a slider adjustment into a second document edit.
    if focus.get().is_some_and(|entity| sliders.contains(entity))
        && [
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::PageUp,
            KeyCode::PageDown,
            KeyCode::Home,
            KeyCode::End,
        ]
        .into_iter()
        .any(|key| keys.just_pressed(key))
    {
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
    focus: Res<bevy::input_focus::InputFocus>,
    menu_items: Query<(), With<bevy::ui_widgets::MenuItem>>,
    mut view: ResMut<View>,
    mut panning: Local<Option<Vec2>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<crate::viewport::ZoneCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let enabled = window.focused
        && session.play.is_none()
        && !session.has_modal()
        && !session.inline_editing
        && session.drag.is_none()
        && !session.ui.input_owned
        && !session.ui.overlay_owned
        && !focus
            .get()
            .is_some_and(|entity| menu_items.contains(entity));
    let cursor = canvas_cursor(window, &session);
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
        camera_views::pan(
            &mut view,
            current - previous,
            (window.height() - workspace::HEADER - session.ui.canvas_bottom(window.height(), true))
                * window.scale_factor(),
        );
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
        if let Some(play) = &session.play {
            let target = Vec3::from_array(play.feet.metres()) + Vec3::Y;
            let desired = play.camera.unobstructed_eye(target);
            let fraction = play.camera_fraction(target, desired);
            *transform = Transform::from_translation(target + (desired - target) * fraction)
                .looking_at(target, Vec3::Y);
        }
    }
}
fn present(
    session: Res<Session>,
    mut windows: Query<&mut Window>,
    mut summaries: Query<&mut Text, With<Summary>>,
    mut info: Query<&mut Text, (With<Info>, Without<Summary>)>,
) {
    let dirty = session.is_dirty();
    for mut text in &mut summaries {
        **text = status_bar::message(&session);
    }
    if let Ok(mut window) = windows.single_mut() {
        let filename = session
            .path
            .as_ref()
            .map(|path| {
                path.file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
            })
            .unwrap_or_else(|| "Untitled".into());
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
    for mut text in &mut info {
        // Context and keyboard hints belong to the status bar. Keep only compact
        // authoring metadata here; editable values are rendered by the fields.
        **text = if let Some(subject) = properties::summary_subject(&session) {
            subject
        } else if session.play.is_some() {
            "Offline playtest".into()
        } else if session.vegetation_mode {
            vegetation::summary(&session)
                .lines()
                .next()
                .unwrap_or_default()
                .into()
        } else if session.marker_mode {
            markers::summary(&session)
                .lines()
                .next()
                .unwrap_or_default()
                .into()
        } else if session.path_mode {
            paths::summary(&session)
                .lines()
                .next()
                .unwrap_or_default()
                .into()
        } else if session.terrain_mode {
            format!("Terrain {:?}", session.brush_tool)
        } else if let Some(primitive) = session.palette {
            format!("Placing {primitive:?}")
        } else {
            "Select an object to inspect its attributes".into()
        };
    }
}
