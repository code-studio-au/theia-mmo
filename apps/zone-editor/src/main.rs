//! P5-17/P3-15: standard Bevy greybox authoring; no trusted gameplay authority.
mod agent;
mod cli;
mod playtest;
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
                if !self
                    .editor
                    .zone()
                    .objects
                    .iter()
                    .any(|o| Some(o.id) == self.selected)
                {
                    self.selected = None;
                }
            }
            Err(error) => self.status = error,
        }
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
}
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
    if options.headless {
        println!(
            "{}",
            serde_json::json!({"revision":editor.revision(),"objects":editor.zone().objects.len(),"history":editor.history(),"valid":true})
        );
        return Ok(());
    }
    let next_id = editor
        .zone()
        .objects
        .iter()
        .map(|o| o.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
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
    })
    .insert_resource(View {
        target: Vec3::ZERO,
        yaw: 0.6,
        pitch: 0.8,
        distance: 32.,
    })
    .insert_resource(Time::<Fixed>::from_hz(30.))
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Theia - Zone Editor".into(),
            resolution: (1280, 800).into(),
            ..default()
        }),
        ..default()
    }))
    .add_observer(activate)
    .add_systems(Startup, (viewport::setup, setup_ui))
    .add_systems(
        Update,
        (
            agent::poll,
            keyboard,
            camera_controls,
            viewport::pointer,
            viewport::sync,
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
fn setup_ui(mut commands: Commands) {
    commands.spawn((Node { position_type: PositionType::Absolute, left: px(0), top: px(0), bottom: px(0),
        width: px(PANEL), padding: UiRect::all(px(14)), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() },
        BackgroundColor(Color::srgb(0.035, 0.05, 0.075))))
        .with_children(|panel| {
            panel.spawn(text("THEIA / ZONE STUDIO", 22.));
            panel.spawn(text("Greybox | metres | Y up", 14.));
            for (action, title) in [
                (Action::Select, "Select / drag [Esc]"), (Action::Place(Primitive::Box), "Place box [1]"),
                (Action::Place(Primitive::Sphere), "Place sphere [2]"), (Action::Place(Primitive::Cylinder), "Place cylinder [3]"),
                (Action::Snap, "Toggle 1 m snap [G]"), (Action::Undo, "Undo [Ctrl Z]"), (Action::Redo, "Redo [Ctrl Y]"),
                (Action::Delete, "Delete selected [Del]"), (Action::Spawn, "Spawn at view centre"),
                (Action::Save, "Save zone [Ctrl S]"), (Action::Load, "Load zone [Ctrl O]"),
                (Action::Play, "Enter / leave playtest [F5]"),
            ] {
                panel.spawn((action, Button, ActivateOnPress, Hovered::default(), Node { min_height: px(26), padding: UiRect::all(px(4)), ..default() },
                    BackgroundColor(Color::srgb(0.10, 0.16, 0.22)))).with_children(|button| { button.spawn(text(title, 15.)); });
            }
            panel.spawn((Info, text("", 14.)));
            panel.spawn(text("Edit: WASD pan | RMB orbit | wheel zoom\nDrag selected: X/Z\nArrows: X/Z | PgUp/Dn: Y\nR / Shift R: yaw +/-15 deg\n+ / -: uniform size\nPlay: W/S forward/back | A/D turn\nQ/E strafe | Space jump\nOffline collision sandbox", 13.));
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
    view: Res<View>,
) {
    if windows.single().is_ok_and(|w| w.focused)
        && let Ok(action) = actions.get(event.entity)
    {
        perform(*action, &mut session, &view);
    }
}
fn perform(action: Action, session: &mut Session, view: &View) {
    session.drag = None;
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
        Action::Select => session.palette = None,
        Action::Place(primitive) => {
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
                Ok(()) => format!("Saved {}", session.path.display()),
                Err(e) => e,
            };
        }
        Action::Load => {
            session.status = match Zone::load(&session.path)
                .and_then(|zone| session.editor.replace(zone, session.editor.revision()))
            {
                Ok(()) => {
                    session.selected = None;
                    session.next_id = session.next_id.max(
                        session
                            .editor
                            .zone()
                            .objects
                            .iter()
                            .map(|o| o.id)
                            .max()
                            .unwrap_or(0)
                            .saturating_add(1),
                    );
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
            session.edit(vec![Edit::SetSpawn { position }]);
        }
        Action::Play => unreachable!(),
    }
}
fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut session: ResMut<Session>,
    view: Res<View>,
) {
    if !windows.single().is_ok_and(|w| w.focused) {
        session.drag = None;
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
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(Action::Place(Primitive::Box))
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(Action::Place(Primitive::Sphere))
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(Action::Place(Primitive::Cylinder))
    } else if keys.just_pressed(KeyCode::KeyG) {
        Some(Action::Snap)
    } else if keys.just_pressed(KeyCode::Delete) {
        Some(Action::Delete)
    } else {
        None
    };
    if let Some(action) = action {
        perform(action, &mut session, &view);
        return;
    }
    if session.play.is_some() || session.drag.is_some() || ctrl {
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
            selected.map_or("No selection".into(), |o| format!(
                "#{} {}\npos {:.2?}\nsize {:.2?}\nyaw {:.0} deg",
                o.id, o.name, o.pose.position, o.pose.size, o.pose.yaw_degrees
            ))
        );
    }
}
