//! Initial entry presentation. Authority and asset readiness remain independent.
mod artwork;
mod presentation;
#[cfg(test)]
mod tests;
mod ui;
pub(crate) use artwork::Artwork;
use bevy::prelude::*;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(crate) enum Phase {
    #[default]
    Loading,
    Playing,
    Failed,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct Checks {
    connection: bool,
    world: bool,
    characters: bool,
    renderer: bool,
    authority: bool,
}
impl Checks {
    fn complete(self) -> bool {
        self.connection && self.world && self.characters && self.renderer && self.authority
    }
    fn label(self) -> &'static str {
        if !self.connection {
            "Connecting to the world..."
        } else if !self.world {
            "Preparing the world..."
        } else if !self.characters {
            "Preparing characters..."
        } else if !self.authority {
            "Synchronizing with the world..."
        } else {
            "Preparing the view..."
        }
    }
}
#[derive(Resource, serde::Serialize)]
pub(crate) struct State {
    pub(crate) phase: Phase,
    checks: Checks,
    failure: Option<&'static str>,
    entered_with: Option<Checks>,
    settled_frames: u32,
    pub(crate) blocked_key_frames: u64,
    #[serde(skip)]
    held_keys: std::collections::HashSet<KeyCode>,
    #[serde(skip)]
    started: Instant,
}
impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Loading,
            checks: Checks::default(),
            failure: None,
            entered_with: None,
            settled_frames: 0,
            blocked_key_frames: 0,
            held_keys: default(),
            started: Instant::now(),
        }
    }
}
impl State {
    pub(crate) fn blocks_input(&self) -> bool {
        self.phase != Phase::Playing
    }
    fn advance(&mut self, checks: Checks, error: Option<&'static str>, timed_out: bool) {
        // Never mask a mid-play fault or silently recover from a startup failure.
        if self.phase != Phase::Loading {
            return;
        }
        self.checks = checks;
        self.settled_frames = if checks.complete() {
            self.settled_frames.saturating_add(1)
        } else {
            0
        };
        if let Some(message) = error.or(timed_out.then_some("The world took too long to prepare."))
        {
            self.fail(message);
        } else if self.settled_frames >= 4 {
            self.phase = Phase::Playing;
            self.entered_with = Some(checks);
            info!("Initial world loading complete; player controls enabled");
        }
    }
    pub(crate) fn fail(&mut self, message: &'static str) -> bool {
        if self.phase == Phase::Playing {
            return false;
        }
        self.phase = Phase::Failed;
        self.failure.get_or_insert(message);
        true
    }
    fn label(&self) -> &str {
        self.failure.unwrap_or_else(|| self.checks.label())
    }
    /// Six equally weighted readiness milestones, not bytes or elapsed time.
    /// Count the completed prefix so work finished out of order cannot make an
    /// unconnected client appear nearly ready. Retractions reduce the progress.
    fn progress_percent(&self) -> u32 {
        let stages = [
            self.checks.connection,
            self.checks.world,
            self.checks.characters,
            self.checks.authority,
            self.checks.renderer,
            self.settled_frames >= 3 && self.phase != Phase::Failed,
        ];
        (stages.into_iter().take_while(|ready| *ready).count() as u32 * 100) / 6
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<State>().init_resource::<Artwork>();
    if let Some(path) = std::env::var_os("THEIA_WORLD_INPUT_OS_QUALIFY") {
        app.insert_resource(Trace {
            path: path.into(),
            frames: 0,
            captured: false,
        })
        .add_systems(Last, trace);
    }
    app.world_mut()
        .resource_mut::<crate::network::Network>()
        .loading_blocked = true;
    app.add_systems(Startup, ui::setup)
        .add_systems(PostStartup, presentation::sync)
        .add_systems(Last, presentation::reveal)
        .add_systems(
            Update,
            (observe, presentation::sync, ui::present)
                .chain()
                .after(crate::network::present)
                .after(crate::reference_scene::tick)
                .after(crate::terrain_scene::tick)
                .after(crate::chunk_world::tick)
                .after(crate::chunk_world::drive)
                .after(crate::graybox::tick),
        )
        .add_observer(ui::cancel);
    install_input(app);
}
fn install_input(app: &mut App) {
    app.add_systems(
        PreUpdate,
        shield_keyboard
            .after(bevy::input::InputSystems)
            .before(bevy::picking::PickingSystems::ProcessInput)
            .before(crate::fantasy_ui::settings::keyboard)
            .before(crate::party::keyboard)
            .before(crate::combat::book::keyboard)
            .before(crate::camera::mode::input_policy),
    );
}
// Clear keyboard state while entry owns input. Keys held across entry must be
// released and pressed again; they cannot open panels, steer or trigger a spell.
pub(crate) fn shield_keyboard(
    mut state: ResMut<State>,
    frontend: Option<Res<crate::frontend::Menu>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    windows: Query<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    for event in events.read() {
        if event.state == bevy::input::ButtonState::Released {
            state.held_keys.remove(&event.key_code);
        }
    }
    if !state.blocks_input() {
        for key in &state.held_keys {
            keys.reset(*key);
        }
        return;
    }
    state.held_keys.extend(keys.get_pressed().copied());
    if !frontend.as_ref().is_some_and(|f| f.active())
        && windows.single().is_ok_and(|w| w.focused)
        && keys.just_pressed(KeyCode::Escape)
    {
        exit.write(if state.phase == Phase::Failed {
            AppExit::error()
        } else {
            AppExit::Success
        });
    }
    state.blocked_key_frames += u64::from(keys.get_pressed().next().is_some());
    let modifiers: Vec<_> = [
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
    ]
    .into_iter()
    .filter(|key| frontend.as_ref().is_some_and(|f| f.active()) && keys.pressed(*key))
    .collect();
    keys.reset_all();
    for key in modifiers {
        keys.press(key);
    }
}

/// Reuse the authority's exact epoch/report/ack barrier, including the current
/// owner chunk. An old or empty acknowledgement must not open the loading gate.
fn authority_ready(state: &theia_network::client::State, predicted: bool) -> bool {
    if !state.welcome.as_ref().is_some_and(|w| w.terrain_stream) {
        return !predicted || state.correction.is_some();
    }
    let Some(report) = &state.chunk_report else {
        return false;
    };
    let Some(owner) = state
        .authority_focus()
        .and_then(|p| p.chunk(theia_core::coordinates::INITIAL_CHUNK_METRES))
    else {
        return false;
    };
    let chunks: Vec<_> = report
        .chunks
        .iter()
        .map(|&[x, z]| theia_core::coordinates::ChunkId { x, z })
        .collect();
    chunks.contains(&owner)
        && state.chunk_permission_confirmed(&chunks)
        && (!predicted
            || state
                .correction
                .is_some_and(|c| c.state.ready_sequence == report.sequence))
}
pub(crate) fn observe(world: &mut World) {
    if crate::frontend::active(world) {
        world.resource_mut::<State>().started = Instant::now();
        return;
    }
    if !world.resource::<State>().blocks_input() {
        return;
    }
    let window_visible = world.query::<&Window>().iter(world).any(|w| w.visible);
    let network = world.resource::<crate::network::Network>();
    let remote = network.state();
    let pipelines = world.resource::<crate::capture_ready::CaptureReady>();
    let checks = Checks {
        connection: network.connected() && network.rendered_global().is_some(),
        world: world
            .get_resource::<crate::authored_scene::Scene>()
            .is_none_or(|s| s.ready())
            && (!world.contains_resource::<crate::terrain_scene::Config>()
                || world.contains_resource::<crate::terrain_scene::Ready>())
            && (!world.contains_resource::<crate::chunk_world::Config>()
                || world.contains_resource::<crate::chunk_world::Ready>()),
        characters: (!world.contains_resource::<crate::reference_scene::Config>()
            || world.contains_resource::<crate::reference_scene::Ready>())
            && crate::creature_visual::initial_assets_ready(world)
            && (!world.contains_resource::<crate::dialogue::Config>() || remote.guide.is_some()),
        renderer: pipelines.ready() && window_visible,
        authority: authority_ready(&remote, network.prediction.is_some()),
    };
    let error = if remote.error.is_some() {
        Some("The connection to the world was lost.")
    } else if pipelines.failed > 0 {
        Some("The renderer could not prepare the world.")
    } else {
        None
    };
    let mut state = world.resource_mut::<State>();
    let timed_out = state.started.elapsed().as_secs() > 120;
    state.advance(checks, error, timed_out);
    let blocked = state.blocks_input();
    world
        .resource_mut::<crate::network::Network>()
        .loading_blocked = blocked;
}
/// Preserve a visible startup failure for interactive entry; diagnostic clients
/// and failures after entering play retain their existing nonzero exit behavior.
pub(crate) fn fail_or_exit(world: &mut World, message: &'static str) {
    if !world
        .get_resource_mut::<State>()
        .is_some_and(|mut s| s.fail(message))
    {
        world.write_message(AppExit::error());
    }
}

pub(crate) fn failed(world: &World) -> bool {
    world
        .get_resource::<State>()
        .is_some_and(|s| s.phase == Phase::Failed)
}

pub(crate) fn exit_unless_presented(
    exit: &mut MessageWriter<AppExit>,
    state: Option<&mut State>,
    message: &'static str,
) {
    if !state.is_some_and(|s| s.fail(message)) {
        exit.write(AppExit::error());
    }
}
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(state) = world.get_resource::<State>() else {
        return serde_json::Value::Null;
    };
    let mut value = serde_json::to_value(state).unwrap();
    value["progress_percent"] = state.progress_percent().into();
    value["geometry"] = ui::geometry(world);
    value
}
#[derive(Resource)]
struct Trace {
    path: std::path::PathBuf,
    frames: u32,
    captured: bool,
}
fn trace(world: &mut World) {
    let mut trace = world.remove_resource::<Trace>().unwrap();
    trace.frames += 1;
    if !trace.captured
        && trace.frames >= 6
        && world.resource::<State>().phase == Phase::Loading
        && world.resource::<State>().checks.renderer
        && world
            .resource::<crate::capture_ready::CaptureReady>()
            .ready()
    {
        let value = snapshot(world);
        if value["geometry"]["art"][2].as_f64().unwrap_or(0.) > 0. {
            std::fs::write(
                trace.path.join("loading-screen.json"),
                serde_json::to_vec_pretty(&value).unwrap(),
            )
            .expect("Loading capture state");
            world
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(
                    trace.path.join("loading-screen.png"),
                ));
            trace.captured = true;
        }
    }
    world.insert_resource(trace);
}
