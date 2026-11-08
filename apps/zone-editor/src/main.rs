//! P5-17/P3-15: standard Bevy greybox authoring; no trusted gameplay authority.
mod agent;
mod cli;
mod export;
mod markers;
mod paths;
mod playtest;
mod terrain;
mod viewport;
use bevy::picking::hover::Hovered;
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    ui_widgets::{Activate, ActivateOnPress, Button},
};
use std::path::PathBuf;
use theia_zone::{Command, Edit, Editor, Object, Pose, Primitive, Zone};

const PANEL: f32 = 300.;
#[derive(Resource)]
struct Session {
    editor: Editor,
    selected: Option<u64>,
    palette: Option<Primitive>,
    snap: bool,
    status: String,
    path: PathBuf,
    next_id: u64,
    drag: Option<Drag>,
    play: Option<playtest::Playtest>,
    heading: f32,
    terrain_mode: bool,
    brush_tool: terrain::Tool,
    brush_radius: f64,
    brush_point: Option<Vec3>,
    stroke: Option<terrain::Stroke>,
    path_mode: bool,
    marker_mode: bool,
    marker_selected: Option<u64>,
    marker_palette: Option<theia_zone::markers::MarkerKind>,
    path_selected: Option<u64>,
    path_draft: Option<paths::Draft>,
}
struct Drag {
    id: u64,
    before: Pose,
    preview: Pose,
    revision: u64,
    offset: Vec3,
}
impl Session {
    fn execute(&mut self, command: Command) {
        match self.editor.execute(command) {
            Ok(()) => {
                self.status = format!("Revision {} accepted", self.editor.revision());
                self.reconcile();
            }
            Err(error) => self.status = error,
        }
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
        if !self
            .editor
            .zone()
            .markers
            .iter()
            .any(|m| Some(m.id) == self.marker_selected)
        {
            self.marker_selected = None;
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
    fn snapped(&self, position: [f64; 3]) -> [f64; 3] {
        if self.snap {
            std::array::from_fn(|i| {
                theia_zone::snap(position[i], self.editor.zone().origin[i], 1.)
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
    Play,
    Spawn,
    Delete,
    TerrainToggle,
    Brush(terrain::Tool),
    Radius(i8),
    SelectMarker,
    PlaceMarker(theia_zone::markers::MarkerKind),
    NextMarker,
    DeleteMarker,
    MarkerRadius(i8),
    NewPath(theia_zone::paths::PathKind),
    FinishPath,
    PopPath,
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
        println!(
            "{}",
            serde_json::json!({"revision":editor.revision(),"objects":editor.zone().objects.len(),"paths":editor.zone().paths.len(),"markers":editor.zone().markers.len(),"history":editor.history(),"valid":true})
        );
        return Ok(());
    }
    let next_id = editor.zone().next_id().unwrap_or(u64::MAX);
    let initial_target = editor.zone().terrain.as_ref().map(|patch| {
        let start = patch.origin().expect("Validated terrain");
        let local = editor
            .zone()
            .local([start[0], editor.zone().origin[1], start[2]])
            .expect("Validated terrain frame");
        Vec3::new(local[0] + 64., 0., local[2] + 64.)
    });
    let mut app = App::new();
    app.insert_resource(Session {
        editor,
        selected: None,
        palette: None,
        snap: true,
        status: "Ready - choose a primitive, then click the ground".into(),
        path: options
            .output
            .or(options.zone)
            .unwrap_or("build/zone-draft.json".into()),
        next_id,
        drag: None,
        play: None,
        heading: 0.,
        terrain_mode: false,
        brush_tool: terrain::Tool::Raise,
        brush_radius: 12.,
        brush_point: None,
        stroke: None,
        path_mode: false,
        marker_mode: false,
        marker_selected: None,
        marker_palette: None,
        path_selected: None,
        path_draft: None,
    })
    .insert_resource(View {
        target: initial_target.unwrap_or(Vec3::ZERO),
        yaw: 0.6,
        pitch: 0.8,
        distance: if initial_target.is_some() { 120. } else { 32. },
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
    .add_observer(activate)
    .add_systems(
        Startup,
        (
            viewport::setup,
            terrain::setup,
            paths::setup,
            markers::setup,
            setup_ui,
        ),
    )
    .add_systems(
        Update,
        (
            agent::poll,
            keyboard,
            camera_controls,
            markers::pointer,
            paths::pointer,
            terrain::pointer,
            viewport::pointer,
            terrain::sync,
            terrain::palette,
            viewport::sync,
            paths::sync,
            markers::sync,
            present,
            viewport::capture,
        )
            .chain(),
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
        (Action::Snap, "Toggle 1 m snap [G]"),
        (Action::Undo, "Undo [Ctrl Z]"),
        (Action::Redo, "Redo [Ctrl Y]"),
        (Action::Delete, "Delete selected [Del]"),
        (Action::Spawn, "Spawn at view centre"),
        (Action::Save, "Save zone [Ctrl S]"),
        (Action::Load, "Load zone [Ctrl O]"),
        (Action::Play, "Enter / leave playtest [F5]"),
    ]
}
fn setup_ui(mut commands: Commands) {
    commands.spawn((Node { position_type: PositionType::Absolute, left: px(0), top: px(0), bottom: px(0),
        width: px(PANEL), padding: UiRect::all(px(14)), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() },
        BackgroundColor(Color::srgb(0.035, 0.05, 0.075))))
        .with_children(|panel| {
            panel.spawn(text("THEIA / ZONE STUDIO", 22.));
            panel.spawn(text("F1 Objects | F2 Terrain\nF3 Guides | F4 Markers", 12.));
            for (index,(action, title)) in object_actions().into_iter().enumerate() {
                panel.spawn((PaletteSlot(index), action, Button, ActivateOnPress, Hovered::default(), Node { min_height: px(26), padding: UiRect::all(px(4)), ..default() },
                    BackgroundColor(Color::srgb(0.10, 0.16, 0.22)))).with_children(|button| { button.spawn((PaletteLabel,text(title, 15.))); });
            }
            panel.spawn((Info, text("", 14.)));
            panel.spawn((Help,text("WASD pan | RMB orbit | wheel zoom\nDrag selected: X/Z\nArrows: X/Z | PgUp/Dn: Y\nR / Shift R: yaw +/-15 deg\n+ / -: uniform size\nPlay: W/S forward/back | A/D turn\nQ/E strafe | Space jump\nOffline collision sandbox", 13.)));
        });
    commands.spawn((
        Summary,
        text("", 16.),
        Node {
            position_type: PositionType::Absolute,
            left: px(PANEL + 12.),
            bottom: px(12),
            right: px(12),
            padding: UiRect::all(px(10)),
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
    if windows.single().is_ok_and(|w| w.focused)
        && let Ok(action) = actions.get(event.entity)
    {
        perform(*action, &mut session, &mut view);
    }
}
fn perform(action: Action, session: &mut Session, view: &mut View) {
    session.drag = None;
    session.stroke = None;
    if !matches!(
        action,
        Action::NewPath(_)
            | Action::FinishPath
            | Action::PopPath
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
        Action::Select => {
            session.palette = None;
            session.marker_palette = None;
        }
        Action::SelectMarker
        | Action::PlaceMarker(_)
        | Action::NextMarker
        | Action::DeleteMarker
        | Action::MarkerRadius(_) => markers::perform(action, session),
        Action::Place(primitive) => {
            session.terrain_mode = false;
            session.path_mode = false;
            session.marker_mode = false;
            session.palette = Some(primitive);
            session.status = "Click the ground to place".into();
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
                Ok(()) => format!(
                    "Saved {}{}",
                    session.path.display(),
                    if session.path_draft.is_some() {
                        " (guide draft still pending)"
                    } else {
                        ""
                    }
                ),
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
                    session.marker_selected = None;
                    session.reconcile();
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
            let origin = session.editor.zone().origin;
            let position = [
                origin[0] + f64::from(view.target.x),
                origin[1] + 0.1,
                origin[2] + f64::from(view.target.z),
            ];
            let mut position = position;
            if let Some(patch) = &session.editor.zone().terrain {
                position[1] += f64::from(
                    patch
                        .height(position[0], position[2])
                        .ok()
                        .flatten()
                        .unwrap_or(0.),
                );
            }
            session.edit(vec![Edit::SetSpawn { position }]);
        }
        Action::NewPath(_)
        | Action::FinishPath
        | Action::PopPath
        | Action::NextPath
        | Action::DeletePath
        | Action::PathWidth(_) => paths::perform(action, session),
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
        Action::Play => unreachable!(),
    }
}
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    mut view: ResMut<View>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        session.drag = None;
        session.stroke = None;
        session.path_draft = None;
        return;
    }
    if (keys.just_pressed(KeyCode::F1)
        || keys.just_pressed(KeyCode::F2)
        || keys.just_pressed(KeyCode::F3)
        || keys.just_pressed(KeyCode::F4))
        && session.play.is_none()
    {
        session.terrain_mode = keys.just_pressed(KeyCode::F2);
        session.path_mode = keys.just_pressed(KeyCode::F3);
        session.marker_mode = keys.just_pressed(KeyCode::F4);
        session.marker_selected = None;
        session.marker_palette = None;
        session.path_draft = None;
        session.path_selected = None;
        session.stroke = None;
        session.drag = None;
        session.palette = None;
        session.selected = None;
        session.status = if session.marker_mode {
            "Marker tools - NPC/quest authoring annotations".into()
        } else if session.path_mode {
            "Guide tools - choose a road or stream, then add ground points".into()
        } else if session.terrain_mode {
            "Terrain tools - create a patch, then click or drag to sculpt".into()
        } else {
            "Object tools".into()
        };
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
    } else if ctrl && keys.just_pressed(KeyCode::KeyS) {
        Some(Action::Save)
    } else if ctrl && keys.just_pressed(KeyCode::KeyO) {
        Some(Action::Load)
    } else if session.marker_mode && keys.just_pressed(KeyCode::Equal) {
        Some(Action::MarkerRadius(1))
    } else if session.marker_mode && keys.just_pressed(KeyCode::Minus) {
        Some(Action::MarkerRadius(-1))
    } else if session.path_mode && keys.just_pressed(KeyCode::Enter) {
        Some(Action::FinishPath)
    } else if session.path_mode && keys.just_pressed(KeyCode::Backspace) {
        Some(Action::PopPath)
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
    } else if keys.just_pressed(KeyCode::KeyG) {
        Some(Action::Snap)
    } else if keys.just_pressed(KeyCode::Delete) {
        Some(if session.marker_mode {
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
    if session.marker_mode {
        markers::transform(&keys, &mut session);
        return;
    }
    if session.path_mode {
        paths::translate(&keys, &mut session);
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
    let increment = if session.snap { 1. } else { 0.1 };
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
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if window.focused && session.play.is_none() {
        let outside = window.cursor_position().is_some_and(|p| p.x > PANEL);
        if outside && mouse.pressed(MouseButton::Right) {
            view.yaw -= motion.delta.x * 0.005;
            view.pitch = (view.pitch + motion.delta.y * 0.005).clamp(0.15, 1.5);
        }
        if outside {
            view.distance = (view.distance * (-scroll.delta.y * 0.1).exp()).clamp(3., 240.);
        }
        if !keys.pressed(KeyCode::ControlLeft) && !keys.pressed(KeyCode::ControlRight) {
            let axis = Vec3::new(
                (i32::from(keys.pressed(KeyCode::KeyD)) as f32)
                    - (i32::from(keys.pressed(KeyCode::KeyA)) as f32),
                0.,
                (i32::from(keys.pressed(KeyCode::KeyS)) as f32)
                    - (i32::from(keys.pressed(KeyCode::KeyW)) as f32),
            );
            view.target = (view.target
                + axis.normalize_or_zero() * time.delta_secs().min(0.1) * 16.)
                .clamp(Vec3::splat(-480.), Vec3::splat(480.));
        }
    }
    if let Ok(mut transform) = camera.single_mut() {
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
    mut summaries: Query<&mut Text, With<Summary>>,
    mut info: Query<&mut Text, (With<Info>, Without<Summary>)>,
) {
    for mut text in &mut summaries {
        **text = session.status.clone();
    }
    let selected = session
        .editor
        .zone()
        .objects
        .iter()
        .find(|o| Some(o.id) == session.selected);
    for mut text in &mut info {
        **text = format!(
            "{}\nrev {} | {} objects\nundo/redo {:?} | snap {}\n{}",
            session.editor.zone().name,
            session.editor.revision(),
            session.editor.zone().objects.len(),
            session.editor.history(),
            session.snap,
            if session.marker_mode {
                markers::summary(&session)
            } else if session.path_mode {
                paths::summary(&session)
            } else if session.terrain_mode {
                format!(
                    "Terrain {:?} | radius {} m\n{}",
                    session.brush_tool,
                    session.brush_radius,
                    if session.editor.zone().terrain.is_some() {
                        "128 m patch | 4 m samples"
                    } else {
                        "No terrain patch"
                    }
                )
            } else {
                selected.map_or("No selection".into(), |o| {
                    format!(
                        "#{} {}\npos {:.2?}\nsize {:.2?}\nyaw {:.0} deg",
                        o.id, o.name, o.pose.position, o.pose.size, o.pose.yaw_degrees
                    )
                })
            }
        );
    }
}
